//! Multicomponent mixture core: mixing rules, mixture Z-factor, partial
//! fugacity coefficients, and the exact composition-derivative layer.
//!
//! ## Architecture (Milestone 8.3 — PERFORMANCE_PROPOSAL §B)
//!
//! Everything is written ONCE against the generalized cubic form
//!
//! ```text
//!   Z³ + (U − B − 1)·Z² + (A + W − U − B·U)·Z − (A·B + W + B·W) = 0
//! ```
//!
//! Each of the 22 EOS × 8 a/b mixing rules (plus the 3-parameter
//! C-mixing rules) only supplies the four dimensionless mixture groups
//! (A, B, U, W) and their **mole-number derivatives**
//!
//! ```text
//!   Āᵢ = (1/n)·∂(n²A)/∂nᵢ    B̄ᵢ = ∂(nB)/∂nᵢ
//!   Ūᵢ = ∂(nU)/∂nᵢ           W̄ᵢ = (1/n)·∂(n²W)/∂nᵢ
//! ```
//!
//! The partial fugacity coefficient then has one closed form for every
//! EOS/rule combination (derived from the residual Helmholtz energy of
//! the generalized cubic; the Müller et al. (9) expressions — research
//! paper Eqs 2.28–2.34 — are the 2-parameter specialization):
//!
//! ```text
//!   ln φ̂ᵢ = −ln(Z−B) + B̄ᵢ/(Z−B) − (Āᵢ−A)·Ĩ − A·Z/q(Z)
//!            + A·[ J₁·(Ūᵢ−U) + J₀·(W̄ᵢ−2W) ]
//!
//!   q(Z) = Z² + U·Z + W
//!   Ĩ  = ∫_Z^∞ dz/q       (log / degenerate / arctan branches)
//!   J₀ = ∫_Z^∞ dz/q²  = (2Z+U)/(D²·q(Z)) − 2·Ĩ/D²,   D² = U²−4W
//!   J₁ = ∫_Z^∞ z·dz/q² = 1/(2·q(Z)) − (U/2)·J₀
//! ```
//!
//! ## Exact derivatives, two ways (§L)
//!
//! The whole computation is generic over the scalar type `D: DualNum<f64>`
//! (the `num-dual` crate (27)). With `D = f64` it is the plain value path;
//! with `D = Dual64` the SAME code carries exact ∂/∂nⱼ along — machine-
//! precision derivatives for every mixing rule including Wong-Sandler and
//! MHV1/2, with no finite differences anywhere (FD survives only as a test
//! oracle). For classical mixing + 2-parameter EOS there is additionally a
//! hand-derived analytic Jacobian (`d_ln_phi_d_n`), cross-validated against
//! the dual-number path in the tests.
//!
//! ## Legacy fidelity notes
//!
//! - The VB6 GE-rule fugacity closings hard-code the PR `(1±√2)` integral
//!   (`Funcion2`, clsQbicsMulticomp.cls:249). This port uses the general
//!   Ĩ integral instead, so HV/MHV/WS work for every EOS family — for PR
//!   the two coincide. (PASCAL_VB6_COMPARISON gotcha; the thesis only ever
//!   exercised these rules with PR.)
//! - The VB6 MHV2 branch reads a stale `lnGamma` (clsQbicsMulticomp.cls:349
//!   — the variable is never assigned inside the MHV2 branch); this port
//!   uses the correct ln γᵢ.
//! - IIVDW's kij matrix is deliberately asymmetric (kij[i][j] ≠ kij[j][i]).
//!
//! # References
//! - (9) Müller et al. — general multicomponent fugacity expressions
//! - (21) Orbey & Sandler — Wong-Sandler mixing rules
//! - (26) Michelsen & Mollerup — generalized mixture core architecture
//! - (27) num-dual crate — dual-number automatic differentiation
//! - (4) Da Silva & Báez (1989) — 3-parameter C mixing, Chao-Seader,
//!   legacy/pascal/TERMOII.PAS
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
pub enum MixError {
    /// Input slice lengths disagree (components vs mole fractions vs kij).
    #[error("dimension mismatch: {0}")]
    Dimension(String),
    /// The chosen (EOS, mixing rule, GE model) combination is not valid —
    /// e.g. a GE-based rule without an activity model, a C-parameter rule
    /// passed as the a/b rule, or a GE rule with a 3-parameter EOS (the
    /// legacy programs never supported that pairing either).
    #[error("unsupported combination: {0}")]
    Unsupported(String),
    /// The cubic solver failed (non-finite coefficients).
    #[error("cubic solver failed: {0}")]
    Cubic(#[from] crate::numerics::cubic::CubicError),
    /// No physical root above B for the requested phase.
    #[error("no real root above B={big_b:.6e} for phase {phase:?}")]
    NoRootForPhase { phase: PhaseId, big_b: f64 },
}

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
///
/// Evaluates to −0.62322540138 (PR family), −ln 2 (RKS family), −1 (VdW
/// family) — exactly the constants VB6 tabulates in `C_cal`
/// (clsQbicsMulticomp.cls:1638-1659), but computed from the family
/// constants so any future family is automatically covered.
pub fn hv_c_constant(eos: CubicEos) -> f64 {
    let fc = crate::eos::family_constants(eos);
    -i_tilde(1.0, fc.k1, fc.k2)
}

/// kij lookup that treats an empty matrix as all-zero.
#[inline]
fn kij_at(kij: &[Vec<f64>], i: usize, j: usize) -> f64 {
    if kij.is_empty() { 0.0 } else { kij[i][j] }
}

// ===========================================================================
// The generalized-cubic integrals, generic over the scalar type.
// ===========================================================================

/// `Ĩ(Z; U, W) = ∫_Z^∞ dz / (z² + U·z + W)` — the generalized attractive
/// integral (research paper Eqs 2.31–2.33). Three branches on D² = U²−4W;
/// branch selection uses the real part, so dual evaluation stays on the
/// branch of the value path (each branch is smooth in its interior).
fn i_tilde_generic<D: DualNum<f64> + Copy>(z: D, u: D, w: D) -> D {
    let disc = u * u - w * 4.0;
    let scale = (u.re() * u.re()).max(4.0 * w.re().abs()).max(1e-300);
    if disc.re().abs() <= 1e-12 * scale {
        // Degenerate (double root): ∫ dz/(z + U/2)² = 1/(Z + U/2).
        (z + u * 0.5).recip()
    } else if disc.re() > 0.0 {
        let delta = disc.sqrt();
        ((z * 2.0 + u + delta) / (z * 2.0 + u - delta)).ln() / delta
    } else {
        // Complex roots: (2/Δ')·(π/2 − atan((2Z+U)/Δ')), Δ' = √(4W−U²).
        let delta = (-disc).sqrt();
        (-((z * 2.0 + u) / delta).atan() + std::f64::consts::FRAC_PI_2) * (delta.recip() * 2.0)
    }
}

/// f64 shortcut for [`i_tilde_generic`] (used by [`hv_c_constant`]).
fn i_tilde(z: f64, u: f64, w: f64) -> f64 {
    i_tilde_generic(z, u, w)
}

// ===========================================================================
// Per-component dimensionless EOS parameters at (T, P).
// ===========================================================================

/// Per-component pure values the mixing rules combine: Aᵢ, Bᵢ (and the
/// dimensionless third parameter Cᵢ for Patel-Teja variants). Generic over the
/// scalar type `D` (M12.3): with `D = f64` these are plain values; with a dual
/// seeded on T or P they carry the exact ∂/∂T or ∂/∂P the mixture derivative
/// paths propagate. They depend on (T, P) but never on composition.
#[derive(Debug, Clone)]
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

/// Validate the spec against the input sizes and rule/EOS pairing rules.
fn validate(spec: &MixtureSpec, x_len: usize) -> Result<(), MixError> {
    let n = spec.components.len();
    if n != x_len {
        return Err(MixError::Dimension(format!(
            "components.len()={n} but composition.len()={x_len}"
        )));
    }
    if !spec.kij.is_empty() && (spec.kij.len() != n || spec.kij.iter().any(|r| r.len() != n)) {
        return Err(MixError::Dimension(format!("kij must be empty or {n}×{n}")));
    }
    let ge_based = matches!(
        spec.rule,
        MixingRule::WongSandler
            | MixingRule::HuronVidalOriginal
            | MixingRule::HuronVidalSimplified
            | MixingRule::MHV1
            | MixingRule::MHV2
    );
    if ge_based && spec.ge.is_none() {
        return Err(MixError::Unsupported(format!(
            "mixing rule {:?} requires an activity model (GeSpec)",
            spec.rule
        )));
    }
    if ge_based && spec.eos.is_three_parameter() {
        return Err(MixError::Unsupported(format!(
            "GE-based rule {:?} with 3-parameter EOS {:?} is not supported \
             (the legacy programs pair 3-parameter EOS with classical a/b \
             mixing only)",
            spec.rule, spec.eos
        )));
    }
    if matches!(
        spec.rule,
        MixingRule::PatelTejaC | MixingRule::PatelTejaUSBC | MixingRule::SchmidtWenzelC
    ) {
        return Err(MixError::Unsupported(format!(
            "{:?} is a C-parameter rule; pass Classical/IVDW as the a/b rule \
             (the C rule is implied by the 3-parameter EOS variant)",
            spec.rule
        )));
    }
    Ok(())
}

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
        for j in 0..n {
            b += x[j] * bi[j];
        }
        b
    };

    // GE helper: ln γᵢ vector + Gᴱ/RT for the coupled activity model.
    let ge_terms = |x: &[D]| -> (Buf<D>, D) {
        let ge = spec.ge.expect("validated: GE rule has GeSpec");
        let mut lng: Buf<D> = smallvec::smallvec![D::from(0.0); n];
        ln_gamma_all_generic(ge.model, x, ge.aij, ge.alpha, ge.vl, ge.delta, t, &mut lng);
        let mut g_rt = D::from(0.0);
        for i in 0..n {
            g_rt += x[i] * lng[i];
        }
        (lng, g_rt)
    };

    let fc = crate::eos::family_constants(spec.eos);

    // --- mixture (A, B) + (Āᵢ, B̄ᵢ) per rule -----------------------------
    let (big_a, big_b, a_bar, b_bar): (D, D, Buf<D>, Buf<D>) = match spec.rule {
        // Classical one-fluid and IVDW share the same math (VB6 implements
        // them as separate cases with identical formulas — see the
        // extraction of clsQbicsMulticomp.cls:455-476 vs :552-568).
        MixingRule::Classical | MixingRule::IVDW => {
            let sq = &pure.sqrt_a;
            let a_ij = |i: usize, j: usize| sq[i] * sq[j] * (1.0 - kij_at(spec.kij, i, j));
            let (a, a_bar) = quad_a(n, x, a_ij);
            let b = lin_b(x);
            let b_bar: Buf<D> = (0..n).map(|i| bi[i]).collect();
            (a, b, a_bar, b_bar)
        }

        // IIVDW: composition-dependent kij, km(i,j) = kij·xᵢ + kji·xⱼ
        // (VB6 clsQbicsMulticomp.cls:478-517). Because Aᵢⱼ now depends on
        // x, Āᵢ picks up extra terms; derived from Āᵢ = 2A + ∂A/∂xᵢ −
        // Σₖ xₖ·∂A/∂xₖ (Euler / projective identity):
        //   ∂A/∂xᵢ  = 2Σⱼ xⱼ√(AᵢAⱼ)(1−kmᵢⱼ) − xᵢΣⱼ xⱼ√(AᵢAⱼ)(kᵢⱼ+kⱼᵢ)
        //   Σₖxₖ∂A/∂xₖ = 2A − Σₖⱼ xₖ²xⱼ√(AₖAⱼ)(kₖⱼ+kⱼₖ)
        //   ⇒ Āᵢ = ∂A/∂xᵢ + Σₖⱼ xₖ²xⱼ√(AₖAⱼ)(kₖⱼ+kⱼₖ)
        MixingRule::IIVDW => {
            let sq = &pure.sqrt_a;
            let sqrt_aa = |i: usize, j: usize| -> D { sq[i] * sq[j] };
            let mut a = D::from(0.0);
            let mut a_bar: Buf<D> = smallvec::smallvec![D::from(0.0); n];
            // sum3 = Σₖⱼ xₖ²xⱼ√(AₖAⱼ)(kₖⱼ + kⱼₖ)
            let mut sum3 = D::from(0.0);
            for k in 0..n {
                let mut inner = D::from(0.0);
                for j in 0..n {
                    inner +=
                        x[j] * (sqrt_aa(k, j) * (kij_at(spec.kij, k, j) + kij_at(spec.kij, j, k)));
                }
                sum3 += x[k] * x[k] * inner;
            }
            for i in 0..n {
                let mut row = D::from(0.0); // Σⱼ xⱼ√(AᵢAⱼ)(1−kmᵢⱼ)
                let mut row_k = D::from(0.0); // Σⱼ xⱼ√(AᵢAⱼ)(kᵢⱼ+kⱼᵢ)
                for j in 0..n {
                    let km = x[i] * kij_at(spec.kij, i, j) + x[j] * kij_at(spec.kij, j, i);
                    row += x[j] * sqrt_aa(i, j) * (-km + 1.0);
                    row_k +=
                        x[j] * (sqrt_aa(i, j) * (kij_at(spec.kij, i, j) + kij_at(spec.kij, j, i)));
                }
                a += x[i] * row;
                a_bar[i] = row * 2.0 - x[i] * row_k + sum3;
            }
            let b = lin_b(x);
            let b_bar: Buf<D> = (0..n).map(|i| bi[i]).collect();
            (a, b, a_bar, b_bar)
        }

        // Wong-Sandler (21) — VB6 clsQbicsMulticomp.cls:422-453 in
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

        // Huron-Vidal original / simplified, MHV1 — all share the linear-b,
        // α-combination structure with different ᾱᵢ (VB6 :519-531, :606-624,
        // :533-550):
        //   α = A/B = Σxᵢαᵢ + Gᴱ/(c·RT)                     (HOV)
        //   α = Σxᵢαᵢ + [Gᴱ/RT + Σxᵢ·ln(B/Bᵢ)]/c            (HVS; c = c*)
        //   same with c = q₁                                 (MHV1)
        MixingRule::HuronVidalOriginal | MixingRule::HuronVidalSimplified | MixingRule::MHV1 => {
            let (lng, g_rt) = ge_terms(x);
            let c = match spec.rule {
                MixingRule::MHV1 => MHV1_Q1,
                _ => hv_c_constant(spec.eos),
            };
            let b = lin_b(x);
            let mut alpha_sum = D::from(0.0);
            for i in 0..n {
                alpha_sum += x[i] * (ai[i] / bi[i]);
            }
            let with_b_log = spec.rule != MixingRule::HuronVidalOriginal;
            let alpha_mix = if with_b_log {
                let mut blog = D::from(0.0);
                for i in 0..n {
                    blog += x[i] * (b / bi[i]).ln();
                }
                alpha_sum + (g_rt + blog) / c
            } else {
                alpha_sum + g_rt / c
            };
            let a = b * alpha_mix;
            let mut a_bar: Buf<D> = smallvec::smallvec![D::from(0.0); n];
            let b_bar: Buf<D> = (0..n).map(|i| bi[i]).collect();
            for i in 0..n {
                let alpha_bar_i = if with_b_log {
                    // ᾱᵢ = αᵢ + [lnγᵢ + ln(B/Bᵢ) + Bᵢ/B − 1]/c
                    (lng[i] + (b / bi[i]).ln() + b.recip() * bi[i] - 1.0) / c + ai[i] / bi[i]
                } else {
                    lng[i] / c + ai[i] / bi[i]
                };
                a_bar[i] = alpha_bar_i * b + alpha_mix * bi[i];
            }
            (a, b, a_bar, b_bar)
        }

        // MHV2 — quadratic in α (VB6 :571-603):
        //   q₁·α + q₂·α² = Σxᵢ(q₁αᵢ + q₂αᵢ²) + Gᴱ/RT + Σxᵢ·ln(B/Bᵢ)
        //   ᾱᵢ = [q₁αᵢ + q₂(αᵢ²+α²) + lnγᵢ + ln(B/Bᵢ) + Bᵢ/B − 1]/(q₁+2q₂α)
        MixingRule::MHV2 => {
            let (lng, g_rt) = ge_terms(x);
            let (q1, q2) = (MHV2_Q1, MHV2_Q2);
            let b = lin_b(x);
            let mut rhs = g_rt;
            for i in 0..n {
                let alpha_i = ai[i] / bi[i];
                rhs += x[i] * (alpha_i * q1 + alpha_i * alpha_i * q2);
                rhs += x[i] * (b / bi[i]).ln();
            }
            // q₂α² + q₁α − rhs = 0 → α = [−q₁ ± √(q₁² + 4q₂·rhs)]/(2q₂);
            // legacy picks the larger root (:593-597).
            let disc = (rhs * (4.0 * q2) + q1 * q1).sqrt();
            let r1 = (disc - q1) / (2.0 * q2);
            let r2 = (-disc - q1) / (2.0 * q2);
            let alpha_mix = if r1.re() >= r2.re() { r1 } else { r2 };
            let a = b * alpha_mix;
            let denom = alpha_mix * (2.0 * q2) + q1;
            let mut a_bar: Buf<D> = smallvec::smallvec![D::from(0.0); n];
            let b_bar: Buf<D> = (0..n).map(|i| bi[i]).collect();
            for i in 0..n {
                let alpha_i = ai[i] / bi[i];
                // ᾱᵢ = [q₁αᵢ + q₂(αᵢ²+α²) + lnγᵢ + ln(B/Bᵢ) + Bᵢ/B − 1]/(q₁+2q₂α).
                // Note: this uses the CORRECT lnγᵢ — the VB6 MHV2 branch
                // reads a stale variable here (legacy bug, module docs).
                let alpha_bar_i = (lng[i]
                    + (b / bi[i]).ln()
                    + b.recip() * bi[i]
                    + (alpha_mix * alpha_mix + alpha_i * alpha_i) * q2
                    + alpha_i * q1
                    - 1.0)
                    / denom;
                a_bar[i] = alpha_bar_i * b + alpha_mix * bi[i];
            }
            (a, b, a_bar, b_bar)
        }

        // C-parameter rules rejected in validate().
        MixingRule::PatelTejaC | MixingRule::PatelTejaUSBC | MixingRule::SchmidtWenzelC => {
            unreachable!("validated")
        }
    };

    // --- (U, W) + derivatives ---------------------------------------------
    let (u, w, u_bar, w_bar): (D, D, Buf<D>, Buf<D>) = if !spec.eos.is_three_parameter() {
        // 2-parameter families: U = k1·B, W = k2·B² exactly, so
        // Ūᵢ = k1·B̄ᵢ and W̄ᵢ = 2·k2·B·B̄ᵢ.
        let u = big_b * fc.k1;
        let w = big_b * big_b * fc.k2;
        let u_bar: Buf<D> = b_bar.iter().map(|&bb| bb * fc.k1).collect();
        let w_bar: Buf<D> = b_bar.iter().map(|&bb| big_b * bb * (2.0 * fc.k2)).collect();
        (u, w, u_bar, w_bar)
    } else {
        three_param_uw(spec, x, pure, big_b, &b_bar)?
    };

    Ok(MixtureParams {
        big_a,
        big_b,
        u,
        w,
        a_bar,
        b_bar,
        u_bar,
        w_bar,
    })
}

/// Mixture (U, W) + derivatives for the 3-parameter EOS (Ref (4),
/// legacy/pascal/TERMOII.PAS:234-262).
///
/// - Schmidt-Wenzel: C = Σxᵢ√Aᵢ·ωᵢ / Σxᵢ√Aᵢ (√A-weighted acentric factor);
///   U = (1+3C)·B, W = −3C·B².
/// - Patel-Teja: C = Σxᵢ·Cᵢ (linear); U = B + C, W = −B·C.
/// - Patel-Teja USB: C = Σxᵢ√Bᵢ·Cᵢ / Σxᵢ√Bᵢ; same (U, W) shape as PT.
///
/// Derivatives via ∂(nQ)/∂nᵢ = Q + ∂Q/∂xᵢ − Σₖxₖ∂Q/∂xₖ for degree-0 Q(x)
/// (and the (1/n)∂(n²·)/∂nᵢ analog for W). For the weighted averages,
/// Σₖxₖ·∂C/∂xₖ = 0, which simplifies the algebra below.
#[allow(clippy::type_complexity)]
fn three_param_uw<D: DualNum<f64> + Copy>(
    spec: &MixtureSpec,
    x: &[D],
    pure: &PureParams<D>,
    big_b: D,
    b_bar: &[D],
) -> Result<(D, D, Buf<D>, Buf<D>), MixError> {
    let n = x.len();
    match spec.eos {
        CubicEos::SchmidtWenzel => {
            // C = F/E, F = Σxⱼ√Aⱼωⱼ, E = Σxⱼ√Aⱼ. ∂C/∂xᵢ = √Aᵢ(ωᵢ−C)/E.
            let mut f_num = D::from(0.0);
            let mut e_den = D::from(0.0);
            for j in 0..n {
                let sa = pure.sqrt_a[j];
                f_num += x[j] * (sa * spec.components[j].omega);
                e_den += x[j] * sa;
            }
            let c = f_num / e_den;
            let u = (c * 3.0 + 1.0) * big_b;
            let w = c * big_b * big_b * (-3.0);
            let mut u_bar = Buf::with_capacity(n);
            let mut w_bar = Buf::with_capacity(n);
            for i in 0..n {
                let dc = (-c + spec.components[i].omega) * pure.sqrt_a[i] / e_den;
                // Ū: nU = (1+3C)·nB → Ūᵢ = (1+3C)B̄ᵢ + 3B·∂(nC)/∂nᵢ|proj
                // with ∂(nC)... collapsing to dc (Σxₖ∂C/∂xₖ = 0):
                u_bar.push((c * 3.0 + 1.0) * b_bar[i] + big_b * dc * 3.0);
                // W̄: n²W = −3C(nB)² → W̄ᵢ = −3[dc·B² + 2C·B·B̄ᵢ]
                w_bar.push((dc * big_b * big_b + c * big_b * b_bar[i] * 2.0) * (-3.0));
            }
            Ok((u, w, u_bar, w_bar))
        }
        CubicEos::PatelTeja | CubicEos::PatelTejaUSB => {
            let ci = &pure.big_c;
            let (c, dc_dx): (D, Buf<D>) = if spec.eos == CubicEos::PatelTeja {
                // Linear: C = Σxᵢ·Cᵢ; ∂(nC)/∂nᵢ = Cᵢ.
                let mut c = D::from(0.0);
                for j in 0..n {
                    c += x[j] * ci[j];
                }
                (c, (0..n).map(|i| ci[i]).collect())
            } else {
                // √B-weighted: C = Σxⱼ√Bⱼ·Cⱼ / Σxⱼ√Bⱼ.
                // ∂C/∂xᵢ = √Bᵢ(Cᵢ−C)/E with Σxₖ∂C/∂xₖ = 0, so
                // ∂(nC)/∂nᵢ = C + ∂C/∂xᵢ.
                let mut num = D::from(0.0);
                let mut e_den = D::from(0.0);
                for j in 0..n {
                    let sb = pure.sqrt_b[j];
                    num += x[j] * (sb * ci[j]);
                    e_den += x[j] * sb;
                }
                let c = num / e_den;
                let bars: Buf<D> = (0..n)
                    .map(|i| c + (-c + ci[i]) * pure.sqrt_b[i] / e_den)
                    .collect();
                (c, bars)
            };
            // U = B + C, W = −B·C.
            let u = big_b + c;
            let w = -(big_b * c);
            let mut u_bar = Buf::with_capacity(n);
            let mut w_bar = Buf::with_capacity(n);
            for i in 0..n {
                let c_bar_i = dc_dx[i]; // ∂(nC)/∂nᵢ
                u_bar.push(b_bar[i] + c_bar_i);
                // n²W = −(nB)(nC) → (1/n)∂/∂nᵢ = −[B̄ᵢ·C + B·C̄ᵢ]
                w_bar.push(-(b_bar[i] * c + big_b * c_bar_i));
            }
            Ok((u, w, u_bar, w_bar))
        }
        _ => unreachable!("three_param_uw called for 2-parameter EOS"),
    }
}

// ===========================================================================
// Mixture Z and partial fugacity coefficients.
// ===========================================================================

/// Solve the generalized cubic for Z at the given phase, generic over D.
///
/// The real root comes from Cardano on the real parts; the dual parts are
/// then recovered by Newton polish in D arithmetic (2 steps — exact for the
/// derivative components since the real part is already converged). This is
/// the standard implicit-function trick for differentiating through a root
/// solve (used by FeOS for the same purpose).
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

/// Mixture Z-factor for the requested phase.
///
/// `t` in **K**, `p` in **kPa absolute**, `x` mole fractions (must sum
/// to 1). Returns dimensionless Z.
pub fn z_mix(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<f64, MixError> {
    let pars = mixture_params::<f64>(spec, t, p, x)?;
    z_mix_generic::<f64>(&pars, phase)
}

/// Partial fugacity coefficients ln φ̂ᵢ for every component.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p` — Pressure in **kPa absolute**.
/// * `x` — Phase mole fractions (length N, sum to 1).
/// * `phase` — Which Z root the phase uses.
///
/// # Returns
/// One ln φ̂ᵢ per component, **dimensionless** (exponentiate for φ̂ᵢ).
pub fn ln_phi_mix(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<Vec<f64>, MixError> {
    let mut out = vec![0.0; x.len()];
    ln_phi_mix_into(spec, t, p, x, phase, &mut out)?;
    Ok(out)
}

// ===========================================================================
// Per-(T, P) cache — audit Part 2 §1.
// ===========================================================================

/// The composition-**independent** half of a mixture evaluation, held across
/// many compositions at one `(T, P)`.
///
/// A PT flash sweeps composition at fixed temperature and pressure: every
/// outer iteration re-splits the feed and re-evaluates both phases, but each
/// component's α(Tr), Aᵢ, Bᵢ, Cᵢ and √Aᵢ depend only on `(T, P, eos)`. Rebuilt
/// per evaluation that is roughly twenty redundant passes over the α dispatch
/// and its transcendentals for one solve. Build a `TpCache` once and hand it to
/// [`ln_phi_mix_cached_into`] instead.
///
/// The cache is keyed on `(eos, T, P, n)` and [`Self::matches`] checks that key,
/// so a stale cache is a detectable error rather than a silently wrong number.
///
/// ```ignore
/// let cache = TpCache::new(&spec, 350.0, 2000.0)?;
/// for x in compositions {
///     ln_phi_mix_cached_into(&spec, &cache, &x, PhaseId::Liquid, &mut out)?;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TpCache {
    eos: CubicEos,
    t: f64,
    p: f64,
    pure: PureParams<f64>,
}

impl TpCache {
    /// Build the cache for `spec` at `t` (**K**) and `p` (**kPa absolute**).
    ///
    /// # Errors
    /// [`MixError`] if the spec's shapes or rule/EOS pairing are invalid — the
    /// same validation [`mixture_params`] performs, done once here so the
    /// cached evaluations below can skip it.
    pub fn new(spec: &MixtureSpec, t: f64, p: f64) -> Result<Self, MixError> {
        validate(spec, spec.components.len())?;
        Ok(Self {
            eos: spec.eos,
            t,
            p,
            pure: pure_params(spec.eos, spec.rule, t, p, spec.components),
        })
    }

    /// Temperature the cache was built at, in **K**.
    #[inline]
    pub fn temperature(&self) -> f64 {
        self.t
    }

    /// Pressure the cache was built at, in **kPa absolute**.
    #[inline]
    pub fn pressure(&self) -> f64 {
        self.p
    }

    /// Number of components the cache was built for.
    #[inline]
    pub fn len(&self) -> usize {
        self.pure.big_a.len()
    }

    /// True when built for zero components.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether this cache is valid for `(spec, t, p)` — same EOS, same state,
    /// same component count.
    #[inline]
    pub fn matches(&self, spec: &MixtureSpec, t: f64, p: f64) -> bool {
        self.eos == spec.eos && self.t == t && self.p == p && self.len() == spec.components.len()
    }

    /// Error for a cache that does not match the requested state.
    fn mismatch(&self, spec: &MixtureSpec, t: f64, p: f64) -> MixError {
        MixError::Dimension(format!(
            "TpCache built for {:?} at T={} P={} with {} components, \
             but used for {:?} at T={t} P={p} with {} components",
            self.eos,
            self.t,
            self.p,
            self.len(),
            spec.eos,
            spec.components.len()
        ))
    }
}

/// [`ln_phi_mix_into`] reusing a prebuilt [`TpCache`].
///
/// Identical result; skips the per-component α / dimensionless-parameter pass
/// and the spec validation, both of which the cache already did.
///
/// # Errors
/// [`MixError::Dimension`] if `out.len() != x.len()` or the cache does not
/// match `(spec, t, p)`; otherwise as [`ln_phi_mix`].
pub fn ln_phi_mix_cached_into(
    spec: &MixtureSpec,
    cache: &TpCache,
    x: &[f64],
    phase: PhaseId,
    out: &mut [f64],
) -> Result<(), MixError> {
    if out.len() != x.len() {
        return Err(MixError::Dimension(format!(
            "out.len()={} but composition.len()={}",
            out.len(),
            x.len()
        )));
    }
    if !cache.matches(spec, cache.t, cache.p) || x.len() != cache.len() {
        return Err(cache.mismatch(spec, cache.t, cache.p));
    }
    let pars = mixture_params_with(spec, cache.t, x, &cache.pure)?;
    let z = z_mix_generic::<f64>(&pars, phase)?;
    ln_phi_from_params_generic(&pars, z, out);
    Ok(())
}

/// [`ln_phi_mix_min_gibbs_into`] reusing a prebuilt [`TpCache`].
///
/// Combines both Part 1 §8 (one mixture state, both cubic roots) and Part 2 §1
/// (one pure-component pass per state point) — the two together are what make
/// the tangent-plane stability test's inner loop cheap.
///
/// # Errors
/// As [`ln_phi_mix_min_gibbs_into`], plus [`MixError::Dimension`] for a cache
/// that does not match `spec`.
pub fn ln_phi_mix_min_gibbs_cached_into(
    spec: &MixtureSpec,
    cache: &TpCache,
    x: &[f64],
    out: &mut [f64],
) -> Result<(), MixError> {
    let n = x.len();
    if out.len() != n {
        return Err(MixError::Dimension(format!(
            "out.len()={} but composition.len()={n}",
            out.len()
        )));
    }
    if !cache.matches(spec, cache.t, cache.p) || n != cache.len() {
        return Err(cache.mismatch(spec, cache.t, cache.p));
    }
    let pars = mixture_params_with(spec, cache.t, x, &cache.pure)?;
    min_gibbs_from_params(&pars, x, out)
}

/// Shared tail of the two min-Gibbs entry points: evaluate both cubic roots
/// against one mixture state and keep the lower-Gibbs one.
fn min_gibbs_from_params(
    pars: &MixtureParams<f64>,
    x: &[f64],
    out: &mut [f64],
) -> Result<(), MixError> {
    let n = x.len();
    let mut trial: Buf<f64> = smallvec::smallvec![0.0; n];
    let mut best_g = f64::INFINITY;
    let mut found = false;
    let mut last_err: Option<MixError> = None;
    for phase in [PhaseId::Liquid, PhaseId::Vapor] {
        match z_mix_generic::<f64>(pars, phase) {
            Ok(z) => {
                ln_phi_from_params_generic(pars, z, &mut trial);
                let g: f64 = (0..n)
                    .filter(|&i| x[i] > 0.0)
                    .map(|i| x[i] * (x[i].ln() + trial[i]))
                    .sum();
                // Strictly-less keeps the liquid root on an exact tie, matching
                // the order the two branches are tried in.
                if !found || g < best_g {
                    best_g = g;
                    out.copy_from_slice(&trial);
                    found = true;
                }
            }
            Err(e) => last_err = Some(e),
        }
    }
    if found {
        Ok(())
    } else {
        Err(last_err.unwrap_or(MixError::NoRootForPhase {
            phase: PhaseId::Liquid,
            big_b: pars.big_b,
        }))
    }
}

/// [`ln_phi_mix`] written into a caller-owned slice — **no heap allocation**.
///
/// Same math, same result, but the caller supplies the destination instead of
/// receiving a fresh `Vec`. The flash's outer loop calls this once per phase
/// per iteration, so returning an owned vector there costs one allocator round
/// trip per call for a buffer whose size never changes (Part 1 §1 of the
/// performance audit).
///
/// Rust-idiom note for readers new to the pattern: an `_into` variant taking
/// `out: &mut [f64]` is the standard way to let a caller control allocation.
/// [`ln_phi_mix`] is written *on top of* this function rather than beside it,
/// so there is exactly one implementation of the thermodynamics.
///
/// # Arguments
/// As [`ln_phi_mix`], plus `out` — a slice of length N receiving ln φ̂ᵢ,
/// **dimensionless**.
///
/// # Errors
/// [`MixError::Dimension`] if `out.len() != x.len()`; otherwise as
/// [`ln_phi_mix`].
pub fn ln_phi_mix_into(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
    out: &mut [f64],
) -> Result<(), MixError> {
    if out.len() != x.len() {
        return Err(MixError::Dimension(format!(
            "out.len()={} but composition.len()={}",
            out.len(),
            x.len()
        )));
    }
    let pars = mixture_params::<f64>(spec, t, p, x)?;
    let z = z_mix_generic::<f64>(&pars, phase)?;
    ln_phi_from_params_generic(&pars, z, out);
    Ok(())
}

/// ln φ̂ᵢ at composition `x` using whichever cubic root gives the lower reduced
/// Gibbs energy `g = Σ xᵢ(ln xᵢ + ln φ̂ᵢ)` — written into a caller-owned slice.
///
/// This is the fugacity a tangent-plane stability test needs: at a candidate
/// single-phase composition, the physically realized phase is the lower-Gibbs
/// root. Both roots come from **one** [`MixtureParams`] build (Part 1 §8 of the
/// performance audit) — the previous route evaluated the whole mixture path
/// twice, recomputing the pure parameters, the mixing rule's n² sum and the
/// cubic coefficients for a state that differs only in which root is selected.
///
/// # Arguments
/// `t` in **K**, `p` in **kPa absolute**, `x` mole fractions (length N);
/// `out` a length-N slice receiving ln φ̂ᵢ, **dimensionless**.
///
/// # Errors
/// [`MixError::Dimension`] on a length mismatch; the underlying root error if
/// *neither* root is physical at this composition.
pub fn ln_phi_mix_min_gibbs_into(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    out: &mut [f64],
) -> Result<(), MixError> {
    let n = x.len();
    if out.len() != n {
        return Err(MixError::Dimension(format!(
            "out.len()={} but composition.len()={n}",
            out.len()
        )));
    }
    // Composition-dependent state: built once, shared by both root branches.
    let pars = mixture_params::<f64>(spec, t, p, x)?;
    min_gibbs_from_params(&pars, x, out)
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
}

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
    let fc = crate::eos::family_constants(spec.eos);
    let (k1, k2) = (fc.k1, fc.k2);
    let (a, b) = (pars.big_a, pars.big_b);
    let (u, w) = (pars.u, pars.w);
    let pure = pure_params(spec.eos, spec.rule, t, p, spec.components);
    let a_ij =
        |i: usize, j: usize| (1.0 - kij_at(spec.kij, i, j)) * pure.sqrt_a[i] * pure.sqrt_a[j];

    // Shared scalars.
    let q = z * z + u * z + w;
    let itilde = i_tilde(z, u, w);
    let disc = u * u - 4.0 * w;
    let scale = (u * u).max(4.0 * w.abs()).max(1e-300);
    let degenerate = disc.abs() <= 1e-12 * scale;
    let j0 = if degenerate {
        1.0 / (3.0 * (z + 0.5 * u).powi(3))
    } else {
        ((2.0 * z + u) / q - 2.0 * itilde) / disc
    };
    let j1 = 1.0 / (2.0 * q) - 0.5 * u * j0;

    // Cubic partials for the implicit ∂Z/∂nⱼ (U = k1·B, W = k2·B² folded in):
    //   f = Z³ + (U−B−1)Z² + (A+W−U−BU)Z − (AB+W+BW)
    let f_z = 3.0 * z * z + 2.0 * (u - b - 1.0) * z + (a + w - u - b * u);
    let f_a = z - b;
    // df/dB with U(B), W(B): dU/dB = k1, dW/dB = 2k2B.
    let f_b = (k1 - 1.0) * z * z + (2.0 * k2 * b - k1 - 2.0 * k1 * b) * z
        - (a + 2.0 * k2 * b + k2 * b * b + 2.0 * k2 * b * b);

    // ∂Ĩ/∂Z = −1/q; ∂Ĩ/∂U = −J₁; ∂Ĩ/∂W = −J₀ (definite-integral parameter
    // derivatives — differentiate under the integral sign).
    // ∂q/∂(Z,U,W) = (2Z+U, Z, 1).
    // J₀, J₁ derivatives are not needed: the fugacity expression groups
    // them with (Ūᵢ−U) and (W̄ᵢ−2W), which vanish identically for
    // classical mixing (Ūᵢ = k1·B̄ᵢ with B̄ᵢ = Bᵢ, and U = k1·B — they
    // do NOT vanish; see below). We therefore differentiate the full
    // expression term by term.
    let n_out = n;
    let mut jac = vec![vec![0.0; n_out]; n_out];
    for j in 0..n {
        let da = pars.a_bar[j] - 2.0 * a; // ∂A/∂nⱼ
        let db = pure.big_b[j] - b; // ∂B/∂nⱼ
        let du = k1 * db;
        let dw = 2.0 * k2 * b * db;
        let dz = -(f_a * da + f_b * db) / f_z;
        let dq = (2.0 * z + u) * dz + z * du + dw;
        let ditilde = -dz / q - j1 * du - j0 * dw;
        // dJ₀, dJ₁ via their closed forms.
        let dj0 = if degenerate {
            // J₀ = (Z+U/2)^(−3)/3 → dJ₀ = −(Z+U/2)^(−4)·(dz + du/2).
            -(z + 0.5 * u).powi(-4) * (dz + 0.5 * du)
        } else {
            // J₀ = [(2Z+U)/q − 2Ĩ]/D², D² = U²−4W, dD² = 2U·du − 4dw.
            let dd2 = 2.0 * u * du - 4.0 * dw;
            let num = (2.0 * z + u) / q - 2.0 * itilde;
            ((2.0 * dz + du) / q - (2.0 * z + u) * dq / (q * q) - 2.0 * ditilde) / disc
                - num * dd2 / (disc * disc)
        };
        let dj1 = -dq / (2.0 * q * q) - 0.5 * (du * j0 + u * dj0);

        for i in 0..n {
            let a_bar_i = pars.a_bar[i];
            let b_bar_i = pars.b_bar[i]; // = Bᵢ, constant in n
            let da_bar_i = 2.0 * a_ij(i, j) - a_bar_i; // ∂Āᵢ/∂nⱼ (degree-1)
            let du_bar_i = 0.0; // Ūᵢ = k1·Bᵢ constant
            let dw_bar_i = 2.0 * k2 * db * b_bar_i; // W̄ᵢ = 2k2·B·Bᵢ
            // Term-by-term derivative of
            //   lnφ̂ᵢ = −ln(Z−B) + B̄ᵢ/(Z−B) − (Āᵢ−A)Ĩ − A·Z/q
            //          + A[J₁(Ūᵢ−U) + J₀(W̄ᵢ−2W)]
            let d1 = -(dz - db) / (z - b);
            let d2 = -b_bar_i * (dz - db) / ((z - b) * (z - b));
            let d3 = -((da_bar_i - da) * itilde + (a_bar_i - a) * ditilde);
            let d4 = -(da * z / q + a * dz / q - a * z * dq / (q * q));
            let bracket = j1 * (k1 * b_bar_i - u) + j0 * (2.0 * k2 * b * b_bar_i - 2.0 * w);
            let dbracket = dj1 * (k1 * b_bar_i - u)
                + j1 * (du_bar_i - du)
                + dj0 * (2.0 * k2 * b * b_bar_i - 2.0 * w)
                + j0 * (dw_bar_i - 2.0 * dw);
            let d5 = da * bracket + a * dbracket;
            jac[i][j] = d1 + d2 + d3 + d4 + d5;
        }
    }
    Ok(jac)
}

// ===========================================================================
// Temperature / pressure derivatives of the partial fugacity coefficients
// (§L, M12.3). Both use the T/P-generic value path (`ln_phi_all_generic`)
// evaluated once with a first-order dual seeded on T (or P) and real
// composition — exact to machine precision, ≈2× a scalar lnφ̂ call, and
// uniform across every EOS × mixing-rule combination (26) Michelsen & Mollerup;
// (27) Rehner & Bauer for the dual-number AD.
//
// This is the "dual everywhere" branch of the §L strategy. The hand-analytic
// fast path for classical + 2-parameter EOS (differentiating the closed-form
// lnφ̂ᵢ through dA/dT and the implicit dZ/dT) is a deferred optimization —
// DERIVATIVE_RELEASE_PLAN.md §7 — because the dual route is already exact and
// cheap next to a flash, and it avoids the near-critical `∂f/∂Z → 0` pivot
// guard the analytic branch would need. The invariant tests (Gibbs–Helmholtz
// and the volumetric identity) pin correctness independently of the route.
// ===========================================================================

/// ∂ln φ̂ᵢ/∂T at constant P and composition, for every component.
///
/// # Arguments
/// * `t` — Temperature in **K**; `p` — pressure in **kPa absolute**.
/// * `x` — Phase mole fractions (length N, sum to 1), held fixed.
/// * `phase` — Which Z root the phase uses.
///
/// # Returns
/// One ∂ln φ̂ᵢ/∂T per component, in **1/K**.
pub fn d_ln_phi_d_t(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<Vec<f64>, MixError> {
    use num_dual::Dual64;
    // Seed T with a unit dual part; P and composition are real constants.
    let td = Dual64::new(t, 1.0);
    let pd = Dual64::from(p);
    let xd: Buf<Dual64> = x.iter().map(|&xi| Dual64::from(xi)).collect();
    let lnphi = ln_phi_all_generic(spec, td, pd, &xd, phase)?;
    Ok(lnphi.iter().map(|v| v.eps).collect())
}

/// ∂ln φ̂ᵢ/∂P at constant T and composition, for every component.
///
/// # Arguments
/// * `t` — Temperature in **K**; `p` — pressure in **kPa absolute**.
/// * `x` — Phase mole fractions (length N, sum to 1), held fixed.
/// * `phase` — Which Z root the phase uses.
///
/// # Returns
/// One ∂ln φ̂ᵢ/∂P per component, in **1/kPa**.
///
/// The composition-summed value obeys the exact volumetric identity
/// `Σᵢ xᵢ·∂ln φ̂ᵢ/∂P = (Z − 1)/P`, which the tests use as an independent check.
pub fn d_ln_phi_d_p(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<Vec<f64>, MixError> {
    use num_dual::Dual64;
    // Seed P with a unit dual part; T and composition are real constants.
    let td = Dual64::from(t);
    let pd = Dual64::new(p, 1.0);
    let xd: Buf<Dual64> = x.iter().map(|&xi| Dual64::from(xi)).collect();
    let lnphi = ln_phi_all_generic(spec, td, pd, &xd, phase)?;
    Ok(lnphi.iter().map(|v| v.eps).collect())
}

/// Residual (departure) isobaric heat capacity `Cp^R` of one phase, in
/// **kJ/(kmol·K)** (M12.4).
///
/// `Cp^R = ∂H^R/∂T` needs a *second* temperature derivative of the residual
/// Gibbs energy. Writing `g(T) = G^R/(RT) = Σᵢ xᵢ·ln φ̂ᵢ` (composition fixed),
/// `H^R = −R·T²·g'(T)` and hence
/// `Cp^R = dH^R/dT = −R·(2·T·g'(T) + T²·g''(T))`.
///
/// One second-order dual evaluation of the T-generic fugacity core
/// ([`ln_phi_all_generic`] with `num_dual::Dual2_64`) yields `g`, `g'` and
/// `g''` together — exact, no finite differences (27) Rehner & Bauer.
///
/// * `t` in **K**, `p` in **kPa absolute**, `x` mole fractions (fixed).
pub fn residual_cp(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<f64, MixError> {
    use num_dual::Dual2_64;
    const R: f64 = 8.31451; // kJ/(kmol·K)
    let n = x.len();
    // g(T), g'(T), g''(T) in one sweep; T carries the second-order seed while
    // P and composition are real constants.
    let (_, g1, g2) = num_dual::try_second_derivative(
        |td: Dual2_64| -> Result<Dual2_64, MixError> {
            let pd = Dual2_64::from(p);
            let xd: Buf<Dual2_64> = x.iter().map(|&xi| Dual2_64::from(xi)).collect();
            let lnphi = ln_phi_all_generic(spec, td, pd, &xd, phase)?;
            let mut g = Dual2_64::from(0.0);
            for i in 0..n {
                g += xd[i] * lnphi[i];
            }
            Ok(g)
        },
        t,
    )?;
    Ok(-R * (2.0 * t * g1 + t * t * g2))
}

// ===========================================================================
// Chao-Seader multicomponent liquid fugacity (Ref (4), TERMOII.PAS:386-405).
// ===========================================================================

/// Chao-Seader liquid-phase ln νᵢ for every component in a mixture.
///
/// The Chao-Seader pure-liquid correlation is composition-independent —
/// each component is evaluated at its own (Tr, Pr) — so the multicomponent
/// version is simply the per-component vector (the activity-coefficient
/// part of the Chao-Seader framework is handled separately by the
/// Scatchard-Hildebrand model). `species[i]` selects the coefficient set
/// per component (hydrogen and methane have special sets).
///
/// `t` in **K**, `p` in **kPa absolute**. Returns ln νᵢ, dimensionless.
pub fn chao_seader_ln_phi_mix(
    components: &[Component],
    species: &[ChaoSeaderSpecies],
    t: f64,
    p: f64,
) -> Result<Vec<f64>, MixError> {
    if components.len() != species.len() {
        return Err(MixError::Dimension(format!(
            "components.len()={} but species.len()={}",
            components.len(),
            species.len()
        )));
    }
    Ok(components
        .iter()
        .zip(species)
        .map(|(c, &s)| chao_seader_ln_phi(t, p, c, s))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------
    // Test fixtures — methane / n-pentane (PR-friendly hydrocarbons) and
    // a methanol / water pair for the GE-coupled rules.
    // -----------------------------------------------------------------

    fn methane() -> Component {
        Component {
            name: "methane".into(),
            tc: 190.564,
            pc: 4599.0,
            omega: 0.0115,
            ..Component::default()
        }
    }

    fn n_pentane() -> Component {
        Component {
            name: "n-pentane".into(),
            tc: 469.7,
            pc: 3370.0,
            omega: 0.252,
            ..Component::default()
        }
    }

    fn methanol() -> Component {
        Component {
            name: "methanol".into(),
            tc: 512.6,
            pc: 8097.0,
            omega: 0.564,
            liquid_volume: 40.7,
            ..Component::default()
        }
    }

    fn water() -> Component {
        Component {
            name: "water".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            liquid_volume: 18.07,
            ..Component::default()
        }
    }

    fn kij2(k: f64) -> Vec<Vec<f64>> {
        vec![vec![0.0, k], vec![k, 0.0]]
    }

    /// Classical PR binary spec used across several tests.
    struct Fixture {
        comps: Vec<Component>,
        kij: Vec<Vec<f64>>,
    }
    impl Fixture {
        fn pr_classical() -> Self {
            Self {
                comps: vec![methane(), n_pentane()],
                kij: kij2(0.023),
            }
        }
        fn spec(&self, rule: MixingRule) -> MixtureSpec<'_> {
            MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &self.comps,
                kij: &self.kij,
                ge: None,
            }
        }
    }

    /// van Laar GE coupling for methanol/water (A12/A21 in the thesis
    /// Table 4.6 neighborhood — the values only need to be plausible; the
    /// tests here check internal consistency, not data fits).
    fn van_laar_aij() -> Vec<Vec<f64>> {
        vec![vec![0.0, 0.847], vec![0.522, 0.0]]
    }

    const GE_RULES: [MixingRule; 5] = [
        MixingRule::WongSandler,
        MixingRule::HuronVidalOriginal,
        MixingRule::HuronVidalSimplified,
        MixingRule::MHV1,
        MixingRule::MHV2,
    ];

    // -----------------------------------------------------------------
    // Legacy constants.
    // -----------------------------------------------------------------

    /// A `TpCache` must reproduce the uncached path exactly — it only hoists
    /// composition-independent work, it does not change any arithmetic.
    #[test]
    fn tp_cache_matches_uncached_path() {
        let comps = [
            Component {
                name: "n-butane".into(),
                tc: 425.12,
                pc: 3796.0,
                omega: 0.200,
                ..Component::default()
            },
            Component {
                name: "n-heptane".into(),
                tc: 540.2,
                pc: 2740.0,
                omega: 0.350,
                ..Component::default()
            },
        ];
        let kij = vec![vec![0.0, 0.02], vec![0.02, 0.0]];
        let spec = MixtureSpec {
            eos: CubicEos::RKS1972,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &kij,
            ge: None,
        };
        let (t, p) = (400.0, 1500.0);
        let cache = TpCache::new(&spec, t, p).unwrap();
        assert!(cache.matches(&spec, t, p));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.temperature(), t);
        assert_eq!(cache.pressure(), p);

        for x in [[0.5, 0.5], [0.85, 0.15], [0.2, 0.8]] {
            for phase in [PhaseId::Liquid, PhaseId::Vapor] {
                let want = ln_phi_mix(&spec, t, p, &x, phase).unwrap();
                let mut got = vec![0.0; 2];
                ln_phi_mix_cached_into(&spec, &cache, &x, phase, &mut got).unwrap();
                for i in 0..2 {
                    assert_eq!(got[i], want[i], "x={x:?} {phase:?} comp {i}");
                }
            }
            // The min-Gibbs selection must agree too.
            let want = {
                let mut v = vec![0.0; 2];
                ln_phi_mix_min_gibbs_into(&spec, t, p, &x, &mut v).unwrap();
                v
            };
            let mut got = vec![0.0; 2];
            ln_phi_mix_min_gibbs_cached_into(&spec, &cache, &x, &mut got).unwrap();
            assert_eq!(got, want, "min-Gibbs at x={x:?}");
        }
    }

    /// A cache built for a different EOS must be rejected, not silently used.
    #[test]
    fn tp_cache_rejects_mismatched_spec() {
        let comps = [Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            ..Component::default()
        }];
        let rks = MixtureSpec {
            eos: CubicEos::RKS1972,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &[],
            ge: None,
        };
        let pr = MixtureSpec {
            eos: CubicEos::PR1976,
            ..rks
        };
        let cache = TpCache::new(&rks, 400.0, 1500.0).unwrap();
        assert!(!cache.matches(&pr, 400.0, 1500.0));
        let mut out = vec![0.0; 1];
        assert!(matches!(
            ln_phi_mix_cached_into(&pr, &cache, &[1.0], PhaseId::Liquid, &mut out),
            Err(MixError::Dimension(_))
        ));
    }

    #[test]
    fn hv_c_star_matches_exact_and_vb6_table() {
        // We compute c* = −Ĩ(z=1; k1, k2) exactly from the family
        // constants. The exact PR value is ln(√2−1)/√2 = −0.6232252401…;
        // RKS is −ln 2; VdW is −1.
        let s2 = std::f64::consts::SQRT_2;
        let pr_exact = (s2 - 1.0).ln() / s2;
        assert!((hv_c_constant(CubicEos::PR1976) - pr_exact).abs() < 1e-13);
        assert!((hv_c_constant(CubicEos::RKS1972) - (-std::f64::consts::LN_2)).abs() < 1e-12);
        assert!((hv_c_constant(CubicEos::VdW1870) - (-1.0)).abs() < 1e-12);
        // VB6 C_cal (clsQbicsMulticomp.cls:1638-1659) hard-codes
        // −0.62322540138 for PR — a rounded constant that agrees with the
        // exact value only to ~1e-6. We match VB6 to that precision (the
        // resulting Gᴱ/c* difference is far below the 1–5% validation gate).
        assert!((hv_c_constant(CubicEos::PR1976) - (-0.62322540138)).abs() < 1e-6);
    }

    // -----------------------------------------------------------------
    // Pure-component limit: every rule must collapse to ln_phi_pure.
    // -----------------------------------------------------------------

    #[test]
    fn pure_limit_matches_ln_phi_pure_classical_and_3param() {
        // x = [1] mixture of one component: mixture ln φ̂ must equal the
        // pure-component ln φ for every EOS (incl. 3-parameter) with
        // classical mixing.
        let comps = [n_pentane()];
        for eos in [
            CubicEos::PR1976,
            CubicEos::RKS1972,
            CubicEos::VdW1870,
            CubicEos::SchmidtWenzel,
            CubicEos::PatelTeja,
            CubicEos::PatelTejaUSB,
        ] {
            let spec = MixtureSpec {
                eos,
                rule: MixingRule::Classical,
                components: &comps,
                kij: &[],
                ge: None,
            };
            let got = ln_phi_mix(&spec, 400.0, 1500.0, &[1.0], PhaseId::Vapor).unwrap()[0];
            let want =
                crate::eos::ln_phi_pure(eos, 400.0, 1500.0, &comps[0], PhaseId::Vapor).unwrap();
            assert!(
                (got - want).abs() < 1e-10,
                "{eos:?}: mixture pure-limit {got} vs pure {want}"
            );
        }
    }

    #[test]
    fn pure_limit_matches_ln_phi_pure_ge_rules() {
        // Same collapse for the GE rules: with one component every GE model
        // has ln γ = 0 and Gᴱ = 0, so WS/HV/MHV all reduce to the pure EOS.
        let comps = [methanol()];
        let aij = vec![vec![0.0]];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &[40.7],
            delta: &[],
        };
        for rule in GE_RULES {
            let spec = MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &comps,
                kij: &[],
                ge: Some(ge),
            };
            let got = ln_phi_mix(&spec, 450.0, 500.0, &[1.0], PhaseId::Vapor).unwrap()[0];
            let want =
                crate::eos::ln_phi_pure(CubicEos::PR1976, 450.0, 500.0, &comps[0], PhaseId::Vapor)
                    .unwrap();
            assert!(
                (got - want).abs() < 1e-9,
                "{rule:?}: pure-limit {got} vs pure {want}"
            );
        }
    }

    // -----------------------------------------------------------------
    // Textbook PR oracle — the general (A,B,U,W) formula must reproduce
    // the classic PR mixture fugacity expression.
    // -----------------------------------------------------------------

    #[test]
    fn classical_pr_matches_textbook_form() {
        let fx = Fixture::pr_classical();
        let spec = fx.spec(MixingRule::IVDW);
        let x = [0.35, 0.65];
        let (t, p) = (350.0, 2000.0);
        let got = ln_phi_mix(&spec, t, p, &x, PhaseId::Vapor).unwrap();

        // Independent textbook implementation (Smith-Van Ness-Abbott /
        // Peng-Robinson 1976 paper form).
        let pure = pure_params(CubicEos::PR1976, MixingRule::Classical, t, p, &fx.comps);
        let a_ij =
            |i: usize, j: usize| (1.0 - fx.kij[i][j]) * (pure.big_a[i] * pure.big_a[j]).sqrt();
        let mut a = 0.0;
        let mut b = 0.0;
        for i in 0..2 {
            b += x[i] * pure.big_b[i];
            for j in 0..2 {
                a += x[i] * x[j] * a_ij(i, j);
            }
        }
        let z = z_mix(&spec, t, p, &x, PhaseId::Vapor).unwrap();
        let s2 = std::f64::consts::SQRT_2;
        for i in 0..2 {
            let a_bar = 2.0 * (x[0] * a_ij(i, 0) + x[1] * a_ij(i, 1));
            let want = (pure.big_b[i] / b) * (z - 1.0)
                - (z - b).ln()
                - a / (2.0 * s2 * b)
                    * (a_bar / a - pure.big_b[i] / b)
                    * ((z + (1.0 + s2) * b) / (z + (1.0 - s2) * b)).ln();
            assert!(
                (got[i] - want).abs() < 1e-10,
                "component {i}: general {} vs textbook {}",
                got[i],
                want
            );
        }
    }

    // -----------------------------------------------------------------
    // Ideal-gas limit: ln φ̂ᵢ → 0 as P → 0 for every rule.
    // -----------------------------------------------------------------

    #[test]
    fn ideal_gas_limit_all_rules() {
        let fx = Fixture::pr_classical();
        let x = [0.4, 0.6];
        for rule in [MixingRule::Classical, MixingRule::IVDW, MixingRule::IIVDW] {
            let spec = fx.spec(rule);
            let lnphi = ln_phi_mix(&spec, 400.0, 1e-4, &x, PhaseId::Vapor).unwrap();
            for (i, v) in lnphi.iter().enumerate() {
                assert!(v.abs() < 1e-6, "{rule:?} comp {i}: lnphi={v} at P→0");
            }
        }
        // GE rules with methanol/water + van Laar.
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        for rule in GE_RULES {
            let spec = MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &comps,
                kij: &[],
                ge: Some(ge),
            };
            let lnphi = ln_phi_mix(&spec, 450.0, 1e-4, &x, PhaseId::Vapor).unwrap();
            for (i, v) in lnphi.iter().enumerate() {
                assert!(v.abs() < 1e-4, "{rule:?} comp {i}: lnphi={v} at P→0");
            }
        }
    }

    // -----------------------------------------------------------------
    // Derivative core (§L): analytic vs dual vs FD oracle.
    // -----------------------------------------------------------------

    /// FD oracle: central difference on mole numbers of the intensive
    /// ln φ̂ᵢ (renormalizing inside). Test-only — production code never
    /// finite-differences (CLAUDE.md Algorithm Choices).
    fn jac_fd(spec: &MixtureSpec, t: f64, p: f64, x: &[f64], phase: PhaseId) -> Vec<Vec<f64>> {
        let n = x.len();
        let h = 1e-6;
        let eval = |moles: &[f64]| -> Vec<f64> {
            let tot: f64 = moles.iter().sum();
            let xn: Vec<f64> = moles.iter().map(|m| m / tot).collect();
            ln_phi_mix(spec, t, p, &xn, phase).unwrap()
        };
        let mut jac = vec![vec![0.0; n]; n];
        for j in 0..n {
            let mut plus = x.to_vec();
            plus[j] += h;
            let mut minus = x.to_vec();
            minus[j] -= h;
            let fp = eval(&plus);
            let fm = eval(&minus);
            for i in 0..n {
                jac[i][j] = (fp[i] - fm[i]) / (2.0 * h);
            }
        }
        jac
    }

    fn assert_jac_close(got: &[Vec<f64>], want: &[Vec<f64>], tol: f64, label: &str) {
        for i in 0..got.len() {
            for j in 0..got.len() {
                let (g, w) = (got[i][j], want[i][j]);
                let denom = w.abs().max(1.0);
                assert!(
                    ((g - w) / denom).abs() < tol,
                    "{label} [{i}][{j}]: got {g}, want {w}"
                );
            }
        }
    }

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

    #[test]
    fn dual_jacobian_matches_fd_for_exotic_rules() {
        // IIVDW (composition-dependent kij) exercises the dual path with a
        // non-trivial Āᵢ; the GE rules exercise it through ln γ / Gᴱ.
        let fx = Fixture::pr_classical();
        let x = [0.35, 0.65];
        let (t, p) = (350.0, 2000.0);
        {
            let spec = fx.spec(MixingRule::IIVDW);
            let dual = d_ln_phi_d_n(&spec, t, p, &x, PhaseId::Vapor).unwrap();
            let fd = jac_fd(&spec, t, p, &x, PhaseId::Vapor);
            assert_jac_close(&dual, &fd, 1e-5, "IIVDW dual-vs-fd");
        }
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        for rule in GE_RULES {
            let spec = MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &comps,
                kij: &kij2(0.05),
                ge: Some(ge),
            };
            let dual = d_ln_phi_d_n(&spec, 400.0, 300.0, &x, PhaseId::Vapor).unwrap();
            let fd = jac_fd(&spec, 400.0, 300.0, &x, PhaseId::Vapor);
            assert_jac_close(&dual, &fd, 1e-4, &format!("{rule:?} dual-vs-fd"));
        }
    }

    #[test]
    fn dual_jacobian_matches_fd_for_3param_eos() {
        // Schmidt-Wenzel and Patel-Teja mixture fugacity derivatives via
        // the dual path (independent Ū, W̄ — the hardest case).
        let comps = vec![methane(), n_pentane()];
        let kij = kij2(0.023);
        let x = [0.35, 0.65];
        for eos in [
            CubicEos::SchmidtWenzel,
            CubicEos::PatelTeja,
            CubicEos::PatelTejaUSB,
        ] {
            let spec = MixtureSpec {
                eos,
                rule: MixingRule::Classical,
                components: &comps,
                kij: &kij,
                ge: None,
            };
            let dual = d_ln_phi_d_n(&spec, 350.0, 2000.0, &x, PhaseId::Vapor).unwrap();
            let fd = jac_fd(&spec, 350.0, 2000.0, &x, PhaseId::Vapor);
            assert_jac_close(&dual, &fd, 1e-4, &format!("{eos:?} dual-vs-fd"));
        }
    }

    // -----------------------------------------------------------------
    // Mixing-rule level checks.
    // -----------------------------------------------------------------

    #[test]
    fn a_bar_euler_identity_every_rule() {
        // Euler homogeneity: Σᵢ xᵢ·Āᵢ = 2A and Σᵢ xᵢ·B̄ᵢ = B, for every
        // rule (a hard invariant of the (1/n)∂(n²A)/∂nᵢ definitions).
        let x = [0.35, 0.65];
        let fx = Fixture::pr_classical();
        let mut specs: Vec<MixtureSpec> = vec![
            fx.spec(MixingRule::Classical),
            fx.spec(MixingRule::IVDW),
            fx.spec(MixingRule::IIVDW),
        ];
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        let kij_ge = kij2(0.05);
        for rule in GE_RULES {
            specs.push(MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &comps,
                kij: &kij_ge,
                ge: Some(ge),
            });
        }
        for spec in &specs {
            let pars = mixture_params::<f64>(spec, 400.0, 800.0, &x).unwrap();
            let sum_a: f64 = (0..2).map(|i| x[i] * pars.a_bar[i]).sum();
            let sum_b: f64 = (0..2).map(|i| x[i] * pars.b_bar[i]).sum();
            let sum_u: f64 = (0..2).map(|i| x[i] * pars.u_bar[i]).sum();
            let sum_w: f64 = (0..2).map(|i| x[i] * pars.w_bar[i]).sum();
            assert!(
                (sum_a - 2.0 * pars.big_a).abs() < 1e-10 * pars.big_a.abs().max(1e-10),
                "{:?}: Σx·Ā = {sum_a} vs 2A = {}",
                spec.rule,
                2.0 * pars.big_a
            );
            assert!(
                (sum_b - pars.big_b).abs() < 1e-10 * pars.big_b.abs().max(1e-10),
                "{:?}: Σx·B̄ = {sum_b} vs B = {}",
                spec.rule,
                pars.big_b
            );
            assert!(
                (sum_u - pars.u).abs() < 1e-10 * pars.u.abs().max(1e-10),
                "{:?}: Σx·Ū = {sum_u} vs U = {}",
                spec.rule,
                pars.u
            );
            assert!(
                (sum_w - 2.0 * pars.w).abs() < 1e-10 * pars.w.abs().max(1e-10),
                "{:?}: Σx·W̄ = {sum_w} vs 2W = {}",
                spec.rule,
                2.0 * pars.w
            );
        }
    }

    #[test]
    fn three_param_euler_identity() {
        // Same Euler invariants for the 3-parameter (U, W) mixing.
        let comps = vec![methane(), n_pentane()];
        let kij = kij2(0.023);
        let x = [0.35, 0.65];
        for eos in [
            CubicEos::SchmidtWenzel,
            CubicEos::PatelTeja,
            CubicEos::PatelTejaUSB,
        ] {
            let spec = MixtureSpec {
                eos,
                rule: MixingRule::Classical,
                components: &comps,
                kij: &kij,
                ge: None,
            };
            let pars = mixture_params::<f64>(&spec, 350.0, 2000.0, &x).unwrap();
            let sum_u: f64 = (0..2).map(|i| x[i] * pars.u_bar[i]).sum();
            let sum_w: f64 = (0..2).map(|i| x[i] * pars.w_bar[i]).sum();
            assert!(
                (sum_u - pars.u).abs() < 1e-12 * pars.u.abs().max(1e-12),
                "{eos:?}: Σx·Ū = {sum_u} vs U = {}",
                pars.u
            );
            assert!(
                (sum_w - 2.0 * pars.w).abs() < 1e-12 * pars.w.abs().max(1e-12),
                "{eos:?}: Σx·W̄ = {sum_w} vs 2W = {}",
                2.0 * pars.w
            );
        }
    }

    #[test]
    fn ws_b_mix_satisfies_wong_sandler_construction() {
        // The WS b_mix must satisfy B = Q/(1−D) with A/B = D exactly
        // (Ref (21) construction).
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        let kij = kij2(0.1);
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::WongSandler,
            components: &comps,
            kij: &kij,
            ge: Some(ge),
        };
        let x = [0.4, 0.6];
        let (t, p) = (350.0, 200.0);
        let pars = mixture_params::<f64>(&spec, t, p, &x).unwrap();
        // Rebuild D from its definition and check A = B·D.
        let pure = pure_params(CubicEos::PR1976, MixingRule::Classical, t, p, &comps);
        let mut lng = [0.0; 2];
        crate::activity::ln_gamma_all(ActivityModel::VanLaar, &x, &aij, &[], &vl, &[], t, &mut lng);
        let g_rt: f64 = (0..2).map(|i| x[i] * lng[i]).sum();
        let c_star = hv_c_constant(CubicEos::PR1976);
        let d: f64 = (0..2)
            .map(|i| x[i] * pure.big_a[i] / pure.big_b[i])
            .sum::<f64>()
            + g_rt / c_star;
        assert!(
            (pars.big_a - pars.big_b * d).abs() < 1e-12,
            "A = {}, B·D = {}",
            pars.big_a,
            pars.big_b * d
        );
    }

    #[test]
    fn invalid_combinations_are_rejected() {
        let fx = Fixture::pr_classical();
        // GE rule without GeSpec.
        let spec = fx.spec(MixingRule::WongSandler);
        assert!(matches!(
            ln_phi_mix(&spec, 350.0, 100.0, &[0.5, 0.5], PhaseId::Vapor),
            Err(MixError::Unsupported(_))
        ));
        // C-parameter rule as a/b rule.
        let spec = fx.spec(MixingRule::PatelTejaC);
        assert!(matches!(
            ln_phi_mix(&spec, 350.0, 100.0, &[0.5, 0.5], PhaseId::Vapor),
            Err(MixError::Unsupported(_))
        ));
        // Dimension mismatch.
        let spec = fx.spec(MixingRule::IVDW);
        assert!(matches!(
            ln_phi_mix(&spec, 350.0, 100.0, &[1.0], PhaseId::Vapor),
            Err(MixError::Dimension(_))
        ));
    }

    #[test]
    fn chao_seader_mix_matches_pure_calls() {
        let comps = vec![methane(), n_pentane()];
        let species = vec![ChaoSeaderSpecies::Methane, ChaoSeaderSpecies::Normal];
        let got = chao_seader_ln_phi_mix(&comps, &species, 300.0, 500.0).unwrap();
        for (i, (c, &s)) in comps.iter().zip(&species).enumerate() {
            let want = chao_seader_ln_phi(300.0, 500.0, c, s);
            assert!((got[i] - want).abs() < 1e-15, "component {i}");
        }
    }

    // -----------------------------------------------------------------
    // T/P derivatives of ln φ̂ᵢ (§L, M12.3).
    // -----------------------------------------------------------------

    /// Central-difference oracle for ∂ln φ̂ᵢ/∂T (never production — CLAUDE.md).
    fn dlnphi_dt_fd(
        spec: &MixtureSpec,
        t: f64,
        p: f64,
        x: &[f64],
        phase: PhaseId,
        h: f64,
    ) -> Vec<f64> {
        let hi = ln_phi_mix(spec, t + h, p, x, phase).unwrap();
        let lo = ln_phi_mix(spec, t - h, p, x, phase).unwrap();
        hi.iter()
            .zip(&lo)
            .map(|(a, b)| (a - b) / (2.0 * h))
            .collect()
    }

    /// Central-difference oracle for ∂ln φ̂ᵢ/∂P.
    fn dlnphi_dp_fd(
        spec: &MixtureSpec,
        t: f64,
        p: f64,
        x: &[f64],
        phase: PhaseId,
        h: f64,
    ) -> Vec<f64> {
        let hi = ln_phi_mix(spec, t, p + h, x, phase).unwrap();
        let lo = ln_phi_mix(spec, t, p - h, x, phase).unwrap();
        hi.iter()
            .zip(&lo)
            .map(|(a, b)| (a - b) / (2.0 * h))
            .collect()
    }

    /// Build the classical + GE + 3-parameter spec matrix the plan names
    /// (methane/n-pentane PR, methanol/water van Laar over the 5 GE rules,
    /// and a Patel-Teja classical binary).
    fn derivative_spec_matrix<'a>(
        fx: &'a Fixture,
        pt: &'a Fixture,
        comps: &'a [Component],
        aij: &'a [Vec<f64>],
        vl: &'a [f64],
        kij_ge: &'a [Vec<f64>],
        ge: &'a GeSpec<'a>,
    ) -> Vec<MixtureSpec<'a>> {
        let mut specs = vec![fx.spec(MixingRule::Classical), fx.spec(MixingRule::IVDW)];
        specs.push(MixtureSpec {
            eos: CubicEos::PatelTeja,
            rule: MixingRule::Classical,
            components: &pt.comps,
            kij: &pt.kij,
            ge: None,
        });
        for rule in GE_RULES {
            specs.push(MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: comps,
                kij: kij_ge,
                ge: Some(*ge),
            });
        }
        let _ = (aij, vl); // referenced through `ge`
        specs
    }

    #[test]
    fn dlnphi_dt_dp_match_fd_across_matrix() {
        let fx = Fixture::pr_classical();
        let pt = Fixture {
            comps: vec![methane(), n_pentane()],
            kij: kij2(0.0),
        };
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        let kij_ge = kij2(0.05);
        let specs = derivative_spec_matrix(&fx, &pt, &comps, &aij, &vl, &kij_ge, &ge);
        let (t, p, x) = (360.0, 1500.0, [0.4, 0.6]);
        for spec in &specs {
            for phase in [PhaseId::Vapor, PhaseId::Liquid] {
                // Skip a phase that has no root for this spec/state.
                let Ok(dual_t) = d_ln_phi_d_t(spec, t, p, &x, phase) else {
                    continue;
                };
                let fd_t = dlnphi_dt_fd(spec, t, p, &x, phase, 1e-3);
                for i in 0..2 {
                    let tol = 1e-6 * dual_t[i].abs().max(1e-6) + 1e-9;
                    assert!(
                        (dual_t[i] - fd_t[i]).abs() <= tol,
                        "{:?} {phase:?} ∂lnφ{i}/∂T: dual={} fd={}",
                        spec.rule,
                        dual_t[i],
                        fd_t[i]
                    );
                }
                let dual_p = d_ln_phi_d_p(spec, t, p, &x, phase).unwrap();
                let fd_p = dlnphi_dp_fd(spec, t, p, &x, phase, 1e-1);
                for i in 0..2 {
                    let tol = 1e-6 * dual_p[i].abs().max(1e-6) + 1e-12;
                    assert!(
                        (dual_p[i] - fd_p[i]).abs() <= tol,
                        "{:?} {phase:?} ∂lnφ{i}/∂P: dual={} fd={}",
                        spec.rule,
                        dual_p[i],
                        fd_p[i]
                    );
                }
            }
        }
    }

    #[test]
    fn gibbs_helmholtz_identity_vs_departure_enthalpy() {
        // Σxᵢ ∂lnφ̂ᵢ/∂T = −H^R/(RT²) = −h_departure_rt_mix/T.
        //
        // An independent cross-check of `d_ln_phi_d_t`: the composition-summed
        // T-derivative must equal −H^R/(RT²) from the separately-derived
        // analytic departure enthalpy (`h_departure_rt_mix`, which does NOT go
        // through the dual path). Covers the classical cubic and every GE
        // rule, Wong-Sandler included, all to machine precision — a strong
        // end-to-end validation.
        //
        // Wong-Sandler originally failed this invariant by ~1%: the departure
        // enthalpy dropped the T·d(ln b_mix)/dT term (WS is the only rule
        // whose dimensional co-volume depends on T). Fixed via the δ
        // correction in `h_departure_rt_mix`; the WS-focused regression test
        // is `wong_sandler_departure_enthalpy_matches_gibbs_helmholtz` below.
        let fx = Fixture::pr_classical();
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        let kij_ge = kij2(0.05);
        let mut specs = vec![fx.spec(MixingRule::Classical)];
        for rule in GE_RULES {
            specs.push(MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &comps,
                kij: &kij_ge,
                ge: Some(ge),
            });
        }
        let (t, p, x) = (360.0, 1500.0, [0.4, 0.6]);
        for spec in &specs {
            for phase in [PhaseId::Vapor, PhaseId::Liquid] {
                let Ok(dt) = d_ln_phi_d_t(spec, t, p, &x, phase) else {
                    continue;
                };
                let sum_dt: f64 = (0..2).map(|i| x[i] * dt[i]).sum();
                let h_rt = crate::energy::h_departure_rt_mix(spec, t, p, &x, phase).unwrap();
                let gh = -h_rt / t;
                assert!(
                    (sum_dt - gh).abs() <= 1e-9 * gh.abs().max(1e-6) + 1e-12,
                    "{:?} {phase:?} Gibbs–Helmholtz: Σx·∂lnφ/∂T={sum_dt} vs −H^R/RT²={gh}",
                    spec.rule
                );
            }
        }
    }

    #[test]
    fn volumetric_identity_all_rules() {
        // Σxᵢ ∂lnφ̂ᵢ/∂P = (Z−1)/P — a pure value-path identity (no enthalpy
        // convention), so it holds tightly for EVERY rule including Wong-
        // Sandler. Independent check of `d_ln_phi_d_p`.
        let fx = Fixture::pr_classical();
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        let kij_ge = kij2(0.05);
        let mut specs = vec![fx.spec(MixingRule::Classical)];
        for rule in GE_RULES {
            specs.push(MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &comps,
                kij: &kij_ge,
                ge: Some(ge),
            });
        }
        let (t, p, x) = (360.0, 1500.0, [0.4, 0.6]);
        for spec in &specs {
            for phase in [PhaseId::Vapor, PhaseId::Liquid] {
                let Ok(dp) = d_ln_phi_d_p(spec, t, p, &x, phase) else {
                    continue;
                };
                let sum_dp: f64 = (0..2).map(|i| x[i] * dp[i]).sum();
                let z = z_mix(spec, t, p, &x, phase).unwrap();
                let vol = (z - 1.0) / p;
                assert!(
                    (sum_dp - vol).abs() <= 1e-9 * vol.abs().max(1e-9) + 1e-14,
                    "{:?} {phase:?} volumetric: Σx·∂lnφ/∂P={sum_dp} vs (Z−1)/P={vol}",
                    spec.rule
                );
            }
        }
    }

    #[test]
    fn wong_sandler_departure_enthalpy_matches_gibbs_helmholtz() {
        // Regression test for a fixed latent bug that M12.3's Gibbs–Helmholtz
        // invariant surfaced (DERIVATIVE_RELEASE_PLAN.md §7): for Wong-Sandler
        // the analytic departure enthalpy `h_departure_rt_mix` was ~1%
        // inconsistent with the exact ln φ̂ᵢ(T). Root cause: WS is the only
        // mixing rule whose dimensional co-volume b_mix = Q̃(T)/(1−D̃(T))
        // depends on temperature, and the enthalpy formula assumed
        // T·d(ln B)/dT = −1 (constant b), silently dropping the db/dT term.
        // `t_dln_a_dt_mix`'s WS branch was never wrong — its T·d(ln A)/dT
        // matches the FD oracle in `analytic_t_derivative_matches_oracle_ge_
        // rules`. The fix adds the δ = T·d(ln b_mix)/dT correction term in
        // `h_departure_rt_mix`; both sides now agree to machine precision.
        let comps = vec![methanol(), water()];
        let aij = van_laar_aij();
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
        };
        let kij_ge = kij2(0.05);
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::WongSandler,
            components: &comps,
            kij: &kij_ge,
            ge: Some(ge),
        };
        let (t, p, x) = (360.0, 1500.0, [0.4, 0.6]);
        for phase in [PhaseId::Vapor, PhaseId::Liquid] {
            let dt = d_ln_phi_d_t(&spec, t, p, &x, phase).unwrap();
            let sum_dt: f64 = (0..2).map(|i| x[i] * dt[i]).sum();
            let gh = -crate::energy::h_departure_rt_mix(&spec, t, p, &x, phase).unwrap() / t;
            let reldiff = ((sum_dt - gh) / gh.abs()).abs();
            assert!(
                reldiff < 1e-12,
                "WS {phase:?} Gibbs–Helmholtz inconsistency returned: \
                 reldiff={reldiff:.2e} (dual={sum_dt}, gh={gh})"
            );
        }
    }
}
