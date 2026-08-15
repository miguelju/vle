# Second opinion: composition-derivative strategy for a cubic-EOS mixture core

You are a senior Rust performance engineer with a strong numerical-methods
background. You are being asked for an **independent design opinion**, not for
agreement. If you think the premise below is wrong, say so.

## The system

`vle-thermo` is a vapor-liquid-equilibrium engine (Rust core, PyO3/UniFFI/wasm
bindings). Canonical internal units: temperature **K**, pressure **kPa
absolute**, dimensionless EOS parameters (A, B, U, W).

The mixture core is written **once**, generic over a scalar type
`D: DualNum<f64> + Copy` (the `num-dual` crate, version **0.11.2**, is already
a dependency). The same source serves three paths:

- `D = f64` — the plain value path.
- `D = Dual64` — exact first derivatives (composition, and separately T or P).
- `D = Dual2_64` — second derivatives for a Gibbs-Helmholtz consistency check.

This "algebra written once, generic over the scalar" property is load-bearing.
An earlier proposal to build a separate flattened `PreparedModel` struct was
rejected specifically because it implied a second implementation of every
mixing rule, which would then have to be kept in sync forever.

## The problem

`d_ln_phi_d_n` returns the Jacobian `jac[i][j] = ∂ln φ̂ᵢ/∂nⱼ`.

For classical / IVDW mixing with a 2-parameter EOS there is a hand-derived
analytic closed form, and it is fast. For **every other case** — Wong-Sandler,
Huron-Vidal, MHV1, MHV2, and any 3-parameter EOS (Schmidt-Wenzel, Patel-Teja) —
there is no closed form, and the code falls back to dual numbers: **one full
`Dual64` evaluation per Jacobian column**, in a loop over `j`. Each of those
sweeps re-runs the entire mixture evaluation (pure-component parameters, the
n² mixing algebra, the activity model, the cubic Z-root solve) to extract a
single column.

Measured on this machine with `cargo bench`, n = 4 components:

| benchmark | time |
|---|---|
| `d_ln_phi_d_n_classical_n4` (analytic closed form) | **252 ns** |
| `d_ln_phi_d_n_wong_sandler_n4` (n dual sweeps) | **1285 ns** |

So the dual path costs **5.1×** the analytic path at n = 4, and the gap grows
with n because the number of sweeps grows with n while each sweep is already
at least O(n²).

The project's own performance plan calls this "the largest remaining
algorithmic win" and also "the highest accuracy risk", and has deferred it
twice. That is why an outside opinion is wanted.

## Your task

Propose — and write — the change that removes the per-column sweep, so the
whole Jacobian comes out of **one** pass over the mixture algebra.

### Hard constraints

1. **The mixing algebra stays written once.** Whatever you propose must flow
   through the existing generic functions (`mixture_params`,
   `mixture_params_with`, `quad_a`, `ln_gamma_all_generic`, `z_mix_generic`,
   `ln_phi_from_params_generic`). Forking the mixing rules into a second
   implementation is an automatic rejection.
2. **No `unsafe`.** The project has an explicit rule against trading memory
   safety for bounds-check removal in a thermodynamics library.
3. **The public signature of `d_ln_phi_d_n` must not change.** It is exposed
   through PyO3, UniFFI (Swift/Kotlin) and wasm-bindgen; the returned
   `Vec<Vec<f64>>` is part of three published surfaces.
4. **Accuracy is non-negotiable and is tested.** The Jacobian must stay
   symmetric (a Gibbs-Duhem consequence) to ~1e-9 relative, and must match a
   central-difference oracle to 1e-5 (classical) / 1e-4 (exotic rules and
   3-parameter EOS). Existing tests assert exactly this.
5. **No heap allocation inside iteration loops** where it can be avoided. The
   working buffer type is `SmallVec<[D; 8]>`, chosen so mixtures up to 8
   components never touch the heap.
6. Today n is typically 2-10. A planned future track needs the same code to
   run at **n ≈ 300** (petroleum pseudocomponents in a crude distillation
   column). Say explicitly how your design behaves as n grows, and whether it
   should change shape at some n.

### What to deliver

1. **Diagnosis** — where the 1285 ns actually goes, and what the theoretical
   floor is for a single-pass design. Be quantitative.
2. **Design** — the exact Rust types you would use. Name them concretely. If a
   crate already in the dependency tree provides what you need, use it and say
   so rather than hand-rolling; if you think hand-rolling is genuinely better
   here, argue why.
3. **The code** — a complete, compiling replacement for `d_ln_phi_d_n` plus any
   new helpers or trait bounds it needs. Not a sketch. If a generic bound
   elsewhere has to be relaxed or widened, show that edit too.
4. **Accuracy analysis** — where your version could lose precision relative to
   the current per-column sweep, and why the two tolerance requirements above
   still hold.
5. **Prediction** — what you expect the `d_ln_phi_d_n_wong_sandler_n4`
   benchmark to read after the change, and what it would read at n = 20. State
   these as numbers; they will be measured against your answer.
6. **Risks and what you would reject** — anything you considered and dismissed,
   with the reason.

Be concrete and terse. Assume the reader knows both Rust and thermodynamics.


## The code

**`engine/src/mixture.rs:66-90` — imports and the working-buffer type**

```rust
//! - VB6 mixing rules: legacy/vb6/clsQbicsMulticomp.cls:395-643

// This module is dense parallel-array numerical code: `for i in 0..n` loops
// that index several like-length vectors (xᵢ, Aᵢ, Bᵢ, Āᵢ, kij[i][j], …) in
// lockstep read far closer to the underlying math (ΣΣ xᵢxⱼ Aᵢⱼ) than the
// equivalent `.iter().zip().enumerate()` chains would. Allow the range-loop
// lint here rather than obscure the formulas.
#![allow(clippy::needless_range_loop)]

use num_dual::DualNum;
use smallvec::SmallVec;
use thiserror::Error;

use crate::activity::{ActivityModel, ln_gamma_all_generic};
use crate::eos::{ChaoSeaderSpecies, CubicEos, PhaseId, chao_seader_ln_phi};
use crate::mixing::MixingRule;
use crate::numerics::cubic::solve_real;
use crate::types::Component;

/// Stack-allocated working buffer: mixtures up to 8 components never touch
/// the heap (PERFORMANCE_PROPOSAL §C3). Larger mixtures spill transparently.
type Buf<D> = SmallVec<[D; 8]>;

/// Errors from the mixture core.
#[derive(Debug, Error, PartialEq)]
```

**`engine/src/mixture.rs:108-160` — the spec types the API takes**

```rust

/// Activity-model coupling for the GE-based mixing rules (WS, HOV, HVS,
/// MHV1, MHV2). Parameter conventions match [`crate::activity::ln_gamma`]:
/// `aij` in the model's units (kJ/kmol for Wilson), `vl` in cm³/mol,
/// `delta` in (cal/cm³)^0.5.
#[derive(Debug, Clone, Copy)]
pub struct GeSpec<'a> {
    /// Which activity model supplies Gᴱ and ln γᵢ.
    pub model: ActivityModel,
    /// N×N binary interaction matrix (may be empty for IdealSolution /
    /// Scatchard-Hildebrand).
    pub aij: &'a [Vec<f64>],
    /// NRTL non-randomness matrix αᵢⱼ (N×N, symmetric) — used **only** when
    /// `model` is NRTL; empty ⇒ ignored.
    pub alpha: &'a [Vec<f64>],
    /// Liquid molar volumes in **cm³/mol** (Wilson, Scatchard).
    pub vl: &'a [f64],
    /// Solubility parameters in **(cal/cm³)^0.5** (Scatchard only).
    pub delta: &'a [f64],
}

/// Everything that defines the thermodynamic model of a mixture:
/// the EOS, the mixing rule, the components, the kij matrix, and (for
/// GE-based rules) the coupled activity model.
#[derive(Debug, Clone, Copy)]
pub struct MixtureSpec<'a> {
    /// Cubic EOS variant applied to the mixture.
    pub eos: CubicEos,
    /// a/b mixing rule. For 3-parameter EOS pass `Classical` or `IVDW`
    /// (the C-parameter rule is implied by the EOS variant, matching the
    /// legacy Pascal wiring: SW → √A-weighted ω, PT → linear, PT-USB →
    /// √B-weighted).
    pub rule: MixingRule,
    /// Component list.
    pub components: &'a [Component],
    /// kij binary interaction matrix, N×N. Symmetric for every rule except
    /// IIVDW (which reads kij[i][j] and kij[j][i] separately). Pass an
    /// empty slice for all-zero kij.
    pub kij: &'a [Vec<f64>],
    /// Activity coupling — required by WS/HOV/HVS/MHV1/MHV2, ignored (may
    /// be `None`) for the classical rules.
    pub ge: Option<GeSpec<'a>>,
}

/// MHV1 q₁ constant. VB6 clsQbicsMulticomp.cls:536.
const MHV1_Q1: f64 = -0.593;
/// MHV2 q₁, q₂ constants. VB6 clsQbicsMulticomp.cls:574-575.
const MHV2_Q1: f64 = -0.478;
const MHV2_Q2: f64 = -0.0047;

/// The Huron-Vidal reference constant c* for the EOS family — the value of
/// the attractive integral at the infinite-pressure packing limit
/// (V = b ⇒ Z-analog 1, in B units): `c* = −Ĩ(z=1; U=k1, W=k2)`.
```

**`engine/src/mixture.rs:216-310` — pure-component params and the mixture-params struct**

```rust
struct PureParams<D> {
    big_a: Buf<D>,
    big_b: Buf<D>,
    /// PT/PT-USB dimensionless cᵢ·P/(R·T) (recovered as Uᵢ − Bᵢ from the
    /// pure state); unused (empty) for other EOS.
    big_c: Buf<D>,
    /// √Aᵢ, precomputed (audit Part 2 §4).
    ///
    /// The classical and IIVDW cross terms need `√(AᵢAⱼ)`, which the mixing
    /// loop used to evaluate as `(ai[i]*ai[j]).sqrt()` — **n² square roots**
    /// for a quantity that only ever needs `n` of them. Hoisting turns
    /// `√(AᵢAⱼ)` into `√Aᵢ·√Aⱼ`, two multiplies. (The two differ in the last
    /// ulp; every downstream test compares against tolerances derived from
    /// the physics, not bit patterns.) The Schmidt-Wenzel C rule reuses the
    /// same values.
    sqrt_a: Buf<D>,
    /// √Bᵢ, precomputed — the PT-USB C rule's √B weighting. Empty unless the
    /// EOS needs it.
    sqrt_b: Buf<D>,
}

fn pure_params<D: DualNum<f64> + Copy>(
    eos: CubicEos,
    rule: MixingRule,
    t: D,
    p: D,
    comps: &[Component],
) -> PureParams<D> {
    let n = comps.len();
    let mut big_a = Buf::with_capacity(n);
    let mut big_b = Buf::with_capacity(n);
    let mut big_c = Buf::new();
    let mut sqrt_a = Buf::new();
    let mut sqrt_b = Buf::new();
    let pt_family = matches!(eos, CubicEos::PatelTeja | CubicEos::PatelTejaUSB);
    // Only the rules that actually form √(AᵢAⱼ) cross terms pay for √Aᵢ; the
    // GE-based rules (Wong-Sandler, Huron-Vidal, MHV) never touch it. Likewise
    // √Bᵢ exists only for the PT-USB C-parameter weighting.
    let needs_sqrt_a = matches!(
        rule,
        MixingRule::Classical | MixingRule::IVDW | MixingRule::IIVDW
    ) || eos == CubicEos::SchmidtWenzel;
    let needs_sqrt_b = eos == CubicEos::PatelTejaUSB;
    for comp in comps {
        // Generic (A, B, U, W) so T/P duals flow through α and the P/(RT)
        // factors (eos.rs). Composition never enters here.
        let (a, b, u, _w) = crate::eos::eos_dimensionless_generic(eos, t, p, comp);
        big_a.push(a);
        big_b.push(b);
        if needs_sqrt_a {
            sqrt_a.push(a.sqrt());
        }
        if needs_sqrt_b {
            sqrt_b.push(b.sqrt());
        }
        if pt_family {
            // PT denominator: U = B + C, W = −B·C ⇒ Cᵢ = Uᵢ − Bᵢ.
            big_c.push(u - b);
        }
    }
    PureParams {
        big_a,
        big_b,
        big_c,
        sqrt_a,
        sqrt_b,
    }
}

// ===========================================================================
// Mixture parameters + mole-number derivatives, per rule, generic over D.
// ===========================================================================

/// Dimensionless mixture parameters and their mole-number derivatives at
/// one (T, P, composition) point. This is the mixture-side analog of
/// [`EosState`](crate::eos::EosState) (M8.2 §C2): computed once, consumed by
/// Z / fugacity / departure code.
pub struct MixtureParams<D> {
    /// Mixture A. **Dimensionless.**
    pub big_a: D,
    /// Mixture B. **Dimensionless.**
    pub big_b: D,
    /// Mixture U (= k1·B for 2-parameter EOS). **Dimensionless.**
    pub u: D,
    /// Mixture W (= k2·B² for 2-parameter EOS). **Dimensionless.**
    pub w: D,
    /// Āᵢ = (1/n)·∂(n²A)/∂nᵢ per component.
    pub a_bar: Buf<D>,
    /// B̄ᵢ = ∂(nB)/∂nᵢ per component.
    pub b_bar: Buf<D>,
    /// Ūᵢ = ∂(nU)/∂nᵢ per component.
    pub u_bar: Buf<D>,
    /// W̄ᵢ = (1/n)·∂(n²W)/∂nᵢ per component.
    pub w_bar: Buf<D>,
}
```

**`engine/src/mixture.rs:358-430` — the generic entry points and the n² quadratic form**

```rust
/// Compute the mixture parameters and their mole-number derivatives.
///
/// `x` is the (normalized) composition, generic over the scalar type: with
/// `D = f64` this is the plain value path; with a dual type the parameters
/// carry exact composition derivatives (§L). `t` in **K**, `p` in **kPa
/// absolute**.
pub fn mixture_params<D: DualNum<f64> + Copy>(
    spec: &MixtureSpec,
    t: D,
    p: D,
    x: &[D],
) -> Result<MixtureParams<D>, MixError> {
    validate(spec, x.len())?;
    let pure = pure_params(spec.eos, spec.rule, t, p, spec.components);
    mixture_params_with(spec, t, x, &pure)
}

/// [`mixture_params`] against an **already-built** pure-component state.
///
/// This is the split audit Part 2 §1 asks for, done without duplicating a line
/// of the mixing rules. `pure_params` — every component's α(Tr), Aᵢ, Bᵢ, Cᵢ and
/// their square roots — depends on `(T, P, eos)` and **never on composition**,
/// yet a PT flash rebuilt it on every fugacity evaluation: twice per K-value
/// call, ~10 K-value calls per solve, so ~20 times for a quantity that changes
/// once. Making it a parameter lets [`TpCache`] hoist it out of the loop while
/// the composition-dependent mixing algebra below stays written exactly once,
/// generic over the scalar type, for both the value and dual-number paths.
///
/// The caller is responsible for `pure` having been built from the same
/// `(spec.eos, T, P, components)`; [`TpCache`] enforces that.
/// Classical quadratic mixing `A = ΣᵢΣⱼ xᵢxⱼAᵢⱼ` with its exact partial
/// `Āᵢ = 2·Σⱼ xⱼAᵢⱼ` (valid for composition-independent Aᵢⱼ).
///
/// Generic over the closure type rather than taking `&dyn Fn`. That matters:
/// the cross-parameter closure is invoked **n² times** here, and behind a
/// trait object every one of those is an indirect call LLVM can neither inline
/// nor vectorize through. Monomorphizing it turned out to be worth
/// substantially more than hoisting the square roots out of the same loop
/// (audit Part 2 §4 proposed the hoist; measurement said the indirect call was
/// the actual obstacle — with `&dyn Fn` in place, removing n² `sqrt` calls
/// barely moved the benchmark).
#[inline]
fn quad_a<D: DualNum<f64> + Copy, F: Fn(usize, usize) -> D>(
    n: usize,
    x: &[D],
    a_ij: F,
) -> (D, Buf<D>) {
    let mut a = D::from(0.0);
    let mut a_bar: Buf<D> = smallvec::smallvec![D::from(0.0); n];
    for i in 0..n {
        let mut row = D::from(0.0);
        for j in 0..n {
            row += x[j] * a_ij(i, j);
        }
        a += x[i] * row;
        a_bar[i] = row * 2.0;
    }
    (a, a_bar)
}

fn mixture_params_with<D: DualNum<f64> + Copy>(
    spec: &MixtureSpec,
    t: D,
    x: &[D],
    pure: &PureParams<D>,
) -> Result<MixtureParams<D>, MixError> {
    let n = x.len();
    let (ai, bi) = (&pure.big_a, &pure.big_b);

    // B is linear (B̄ᵢ = Bᵢ) for every rule except Wong-Sandler, which
    // derives (B, B̄ᵢ) from its own quadratic bij combination below.
    let lin_b = |x: &[D]| -> D {
        let mut b = D::from(0.0);
```

**`engine/src/mixture.rs:505-546` — the Wong-Sandler branch — the slow case**

```rust
        // dimensionless form (b − a/RT ↔ B − A since A = aP/(RT)²,
        // B = bP/RT):
        //   Qᵂ = ΣΣ xᵢxⱼ·bijᵂ,  bijᵂ = ½[(Bᵢ−Aᵢ)+(Bⱼ−Aⱼ)](1−kᵢⱼ)
        //   Dᵂ = Σ xᵢ·αᵢ + Gᴱ/(c*·RT),  αᵢ = Aᵢ/Bᵢ
        //   B  = Qᵂ/(1−Dᵂ),  A = B·Dᵂ
        // Derivatives (legacy derivada1/derivada2, :285-291):
        //   B̄ᵢ = 2Σⱼxⱼbijᵂ/(1−Dᵂ) − Qᵂ(1−D̄ᵢᵂ)/(1−Dᵂ)²,  D̄ᵢᵂ = αᵢ + lnγᵢ/c*
        //   Āᵢ = Dᵂ·B̄ᵢ + B·D̄ᵢᵂ
        MixingRule::WongSandler => {
            let (lng, g_rt) = ge_terms(x);
            let c_star = hv_c_constant(spec.eos);
            let bij_ws = |i: usize, j: usize| -> D {
                ((bi[i] - ai[i]) + (bi[j] - ai[j])) * (0.5 * (1.0 - kij_at(spec.kij, i, j)))
            };
            let mut q_ws = D::from(0.0);
            let mut row_ws: Buf<D> = smallvec::smallvec![D::from(0.0); n];
            for i in 0..n {
                let mut row = D::from(0.0);
                for j in 0..n {
                    row += x[j] * bij_ws(i, j);
                }
                q_ws += x[i] * row;
                row_ws[i] = row;
            }
            let mut d_ws = g_rt / c_star;
            for i in 0..n {
                d_ws += x[i] * (ai[i] / bi[i]);
            }
            let one_minus_d = -d_ws + 1.0;
            let b = q_ws / one_minus_d;
            let a = b * d_ws;
            let mut b_bar: Buf<D> = smallvec::smallvec![D::from(0.0); n];
            let mut a_bar: Buf<D> = smallvec::smallvec![D::from(0.0); n];
            for i in 0..n {
                let d_bar_i = lng[i] / c_star + ai[i] / bi[i];
                let b_bar_i = row_ws[i] * 2.0 / one_minus_d
                    - q_ws * (-d_bar_i + 1.0) / (one_minus_d * one_minus_d);
                b_bar[i] = b_bar_i;
                a_bar[i] = d_ws * b_bar_i + b * d_bar_i;
            }
            (a, b, a_bar, b_bar)
        }
```

**`engine/src/mixture.rs:760-861` — Z root, fugacity from params, and the generic evaluator**

```rust
fn z_mix_generic<D: DualNum<f64> + Copy>(
    pars: &MixtureParams<D>,
    phase: PhaseId,
) -> Result<D, MixError> {
    let (a, b, u, w) = (pars.big_a, pars.big_b, pars.u, pars.w);
    let c2 = u - b - 1.0;
    let c1 = a + w - u - b * u;
    let c0 = -(a * b + w + b * w);
    let (roots, count) = solve_real(1.0, c2.re(), c1.re(), c0.re())?;
    // Physical-root selection by direct comparison (same policy as the
    // pure-component path): liquid = smallest root > B, vapor = largest.
    let mut z0: Option<f64> = None;
    for &r in &roots[..count] {
        if r <= b.re() {
            continue;
        }
        z0 = Some(match (z0, phase) {
            (None, _) => r,
            (Some(cur), PhaseId::Liquid) => cur.min(r),
            (Some(cur), PhaseId::Vapor) => cur.max(r),
        });
    }
    let z0 = z0.ok_or(MixError::NoRootForPhase {
        phase,
        big_b: b.re(),
    })?;
    // Newton polish in D arithmetic to propagate derivatives through the
    // implicit root. Guarded: at a near-double root f'(Z) ≈ 0 and the
    // polish would blow up — skip it there (the value is still correct;
    // derivatives at a merging root are ill-defined anyway).
    let mut z = D::from(z0);
    for _ in 0..2 {
        let f = ((z + c2) * z + c1) * z + c0;
        let fp = (z * 3.0 + c2 * 2.0) * z + c1;
        if fp.re().abs() < 1e-10 {
            break;
        }
        z -= f / fp;
    }
    Ok(z)
}

/// ln φ̂ᵢ for every component from an **already-built** mixture state.
///
/// Split out of [`ln_phi_all_generic`] (Part 1 §8 of the performance audit) so
/// a caller that needs *both* cubic roots at one composition — the min-Gibbs
/// root selection the stability test drives — builds the composition-dependent
/// [`MixtureParams`] **once** and evaluates each candidate root against it.
/// Everything in `pars` (the pure Aᵢ/Bᵢ/Uᵢ/Wᵢ, the mixing rule's n² attractive
/// sum, the cubic's coefficients) is root-independent; only `z` differs between
/// the liquid and vapor branches.
///
/// Writes one value per component into `out`, which the caller sizes to
/// `pars.a_bar.len()`. Returns nothing and allocates nothing.
///
/// The closed form for the generalized cubic (module docs):
/// ```text
///   ln φ̂ᵢ = −ln(Z−B) + B̄ᵢ/(Z−B) − (Āᵢ−A)·Ĩ − A·Z/q(Z)
///            + A·[J₁(Ūᵢ−U) + J₀(W̄ᵢ−2W)]
/// ```
fn ln_phi_from_params_generic<D: DualNum<f64> + Copy>(
    pars: &MixtureParams<D>,
    z: D,
    out: &mut [D],
) {
    let (a, b, u, w) = (pars.big_a, pars.big_b, pars.u, pars.w);
    let q = (z + u) * z + w;
    let itilde = i_tilde_generic(z, u, w);
    // J₀ = (2Z+U)/(D²·q) − 2Ĩ/D², J₁ = 1/(2q) − (U/2)·J₀; D² = U²−4W.
    // Degenerate D² ≈ 0 (VdW): J₀ = 1/(3(Z+U/2)³), J₁ = ∫z/q² with
    // q = (z+U/2)² → J₁ = 1/(2(Z+U/2)²) − (U/2)·J₀.
    let disc = u * u - w * 4.0;
    let scale = (u.re() * u.re()).max(4.0 * w.re().abs()).max(1e-300);
    let j0 = if disc.re().abs() <= 1e-12 * scale {
        ((z + u * 0.5).powi(3) * 3.0).recip()
    } else {
        ((z * 2.0 + u) / q - itilde * 2.0) / disc
    };
    let j1 = (q * 2.0).recip() - u * 0.5 * j0;
    let ln_zb = (z - b).ln();
    let az_q = a * z / q;
    for i in 0..out.len() {
        out[i] = -ln_zb + pars.b_bar[i] / (z - b) - (pars.a_bar[i] - a) * itilde - az_q
            + a * (j1 * (pars.u_bar[i] - u) + j0 * (pars.w_bar[i] - w * 2.0));
    }
}

/// ln φ̂ᵢ for every component, generic over the scalar type: build the mixture
/// state at `x`, pick the requested cubic root, evaluate.
fn ln_phi_all_generic<D: DualNum<f64> + Copy>(
    spec: &MixtureSpec,
    t: D,
    p: D,
    x: &[D],
    phase: PhaseId,
) -> Result<Buf<D>, MixError> {
    let pars = mixture_params(spec, t, p, x)?;
    let z = z_mix_generic(&pars, phase)?;
    let mut out: Buf<D> = smallvec::smallvec![D::from(0.0); x.len()];
    ln_phi_from_params_generic(&pars, z, &mut out);
    Ok(out)
}
```

**`engine/src/mixture.rs:1170-1226` — THE TARGET — the per-column dual sweep**

```rust
}

/// Exact composition Jacobian ∂ln φ̂ᵢ/∂nⱼ (per **mole number**, evaluated
/// at total moles n = 1, i.e. at the given mole-fraction point).
///
/// Route (§L): classical mixing + 2-parameter EOS uses the hand-derived
/// analytic closed form; every other rule/EOS evaluates the SAME generic
/// value path with dual numbers (`num-dual`) — exact to machine precision,
/// never finite differences.
///
/// Returns `jac[i][j] = ∂ln φ̂ᵢ/∂nⱼ`. The matrix is symmetric in exact
/// arithmetic (a Gibbs-Duhem consequence — used as a test invariant).
pub fn d_ln_phi_d_n(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<Vec<Vec<f64>>, MixError> {
    let analytic = matches!(spec.rule, MixingRule::Classical | MixingRule::IVDW)
        && !spec.eos.is_three_parameter();
    if analytic {
        return d_ln_phi_d_n_classical(spec, t, p, x, phase);
    }
    let n = x.len();
    let mut jac = vec![vec![0.0; n]; n];
    // One dual sweep per column j: seed nⱼ with unit dual part. lnφ̂ is
    // intensive (degree-0 in n), so evaluating with moles = x and reading
    // the dual part gives ∂lnφ̂ᵢ/∂nⱼ at Σn = 1 directly. The generic path
    // normalizes internally.
    for j in 0..n {
        let moles: Buf<num_dual::Dual64> = (0..n)
            .map(|k| {
                let mut d = num_dual::Dual64::from(x[k]);
                if k == j {
                    d.eps = 1.0;
                }
                d
            })
            .collect();
        // Normalize to mole fractions in dual arithmetic (Σn carries a
        // dual part too, which is what makes the projective/intensive
        // correction happen automatically).
        let mut total = num_dual::Dual64::from(0.0);
        for m in &moles {
            total += *m;
        }
        let xd: Buf<num_dual::Dual64> = moles.iter().map(|&m| m / total).collect();
        // Composition carries the dual seed here; T and P are constants (real
        // dual parts) so only ∂/∂nⱼ is tracked.
        let (td, pd) = (num_dual::Dual64::from(t), num_dual::Dual64::from(p));
        let lnphi = ln_phi_all_generic(spec, td, pd, &xd, phase)?;
        for i in 0..n {
            jac[i][j] = lnphi[i].eps;
        }
    }
    Ok(jac)
```

**`engine/src/mixture.rs:1228-1250` — the analytic fast path it falls back from**

```rust

/// Hand-derived analytic ∂ln φ̂ᵢ/∂nⱼ for classical mixing (Classical/IVDW)
/// with a 2-parameter EOS (§L).
///
/// Chain rule through the mole-number derivatives of (A, B) and the
/// implicit derivative of Z through the cubic:
/// ```text
///   ∂A/∂nⱼ = Āⱼ − 2A          ∂B/∂nⱼ = Bⱼ − B
///   ∂Āᵢ/∂nⱼ = 2Aᵢⱼ − Āᵢ − (Āⱼ − 2A)·0   (Āᵢ is degree-1: = 2Aᵢⱼ − Āᵢ)
///   ∂Z/∂nⱼ = −(f_A·∂A/∂nⱼ + f_B·∂B/∂nⱼ)/f_Z
/// ```
/// and the derivative of the fugacity expression itself. Cross-validated
/// against the dual-number path and an FD oracle in the tests.
fn d_ln_phi_d_n_classical(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<Vec<Vec<f64>>, MixError> {
    let n = x.len();
    let pars = mixture_params::<f64>(spec, t, p, x)?;
    let z = z_mix_generic(&pars, phase)?;
```


## Reference: the invariants your change must not break

**`engine/src/mixture.rs` (test module) — the accuracy oracle**

```rust
    #[test]
    fn classical_analytic_jacobian_matches_fd_and_is_symmetric() {
        let fx = Fixture::pr_classical();
        let spec = fx.spec(MixingRule::IVDW);
        let x = [0.35, 0.65];
        let (t, p) = (350.0, 2000.0);
        for phase in [PhaseId::Vapor, PhaseId::Liquid] {
            let analytic = d_ln_phi_d_n(&spec, t, p, &x, phase).unwrap();
            let fd = jac_fd(&spec, t, p, &x, phase);
            assert_jac_close(&analytic, &fd, 1e-5, &format!("analytic-vs-fd {phase:?}"));
            // Symmetry (Gibbs-Duhem): ∂lnφ̂ᵢ/∂nⱼ = ∂lnφ̂ⱼ/∂nᵢ.
            assert!(
                (analytic[0][1] - analytic[1][0]).abs() < 1e-9 * analytic[0][1].abs().max(1.0),
                "symmetry {phase:?}: {} vs {}",
                analytic[0][1],
                analytic[1][0]
            );
        }
    }
```

## Reference: the benchmark that will grade you

**`engine/benches/engine_bench.rs:610-634` — criterion benchmark**

```rust
        g.bench_function("d_ln_phi_d_n_classical_n4", |b| {
            b.iter(|| {
                d_ln_phi_d_n(
                    black_box(&classical),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });
        g.bench_function("d_ln_phi_d_n_wong_sandler_n4", |b| {
            b.iter(|| {
                d_ln_phi_d_n(
                    black_box(&ws),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });

```


Baseline to beat: `d_ln_phi_d_n_wong_sandler_n4` = **1285 ns**.
The analytic classical path, for scale, is **252 ns**.
