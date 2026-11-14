//! Isothermal (PT) flash — Milestone 9, §J + §F.
//!
//! Given a feed composition `z` at fixed `(T, P)`, find the vapor fraction
//! `β`, the liquid composition `x`, and the vapor composition `y` at
//! equilibrium. Two nested pieces:
//!
//! 1. **Rachford-Rice** (§F) — the inner scalar solve for `β` at fixed
//!    K-values, via Halley's method (cubic convergence) inside the
//!    Leibovici–Neoschil window with a bisection safeguard. Guaranteed to
//!    converge; supports negative flash.
//! 2. **Outer K-loop** (§J) — Wilson-initialized successive substitution
//!    accelerated by the General Dominant Eigenvalue Method (GDEM) every
//!    few iterations. Each SS step: solve RR for `β`, split into `x`/`y`,
//!    recompute K from the fugacity models, repeat to composition
//!    tolerance.
//! 3. **Stability fallback on non-convergence** — if the K-loop exhausts its
//!    iteration cap (in practice: near the critical locus), a TPD stability
//!    test (`flash/stability.rs`) decides between "stable single phase" and
//!    a retry from the trial-phase K. The full test is *not* run before a
//!    two-phase solve: on a 21 600-point hydrocarbon grid it changed no
//!    converged answer (`OPTIMIZATION_PLAN_PART1.md` §10).
//! 4. **Single-phase check** — before a cubic flash returns "single phase",
//!    a pure-component tangent-plane screen (one feed ln φ + one pure cubic
//!    per component) looks for a phase the K-loop could not see — water out
//!    of a hydrocarbon, the case where every Wilson Kᵢ < 1. It proves
//!    instability when it fires and cannot fire on a stable feed; when it
//!    does, the multi-seed stability test and a retry find the split
//!    (`OPTIMIZATION_PLAN_PART1.md` §13).
//!
//! 5. **Newton finish** — once the ln-K residual is small and successive
//!    substitution is contracting slowly, the loop switches to Newton on the
//!    two-phase Gibbs energy in the vapor mole numbers (`newton_step`), with
//!    the exact composition Jacobian of each phase. Same acceptance test, so
//!    the same answers; it pays near the critical locus and is not taken on
//!    a flash SS + GDEM finishes quickly (`OPTIMIZATION_PLAN_PART1.md` §18).
//! 6. **Model re-seed (non-cubic systems)** — before a γ-φ flash accepts
//!    Wilson's "single phase" at iteration 0, it asks the model once: the
//!    loop restarts from the model's K at `x = y = z` (§19 there).
//!
//! The single-phase check (4) also runs the full stability test when the
//! screen's smallest pure-trial distance is positive but below a margin —
//! the case where the incipient phase is a mixture (§20 there).
//!
//! Two rules about what a result means (§21 there). A converged split whose
//! K-values are all within 1e-6 of 1 is the trivial solution and is returned
//! as single phase. And the liquid/vapor label of a single-phase φ-φ result
//! (β = 0 or 1) comes from the feed's own density — the lower-Gibbs cubic
//! root, or the side of the EOS critical volume — not from the last K of
//! the iteration, which near the critical locus is rounding noise.
//!
//! # References
//! - (7) Michelsen (1982) Part I — stability analysis (the fallback)
//! - (19) Michelsen (1982) Part II — phase-split framework
//! - (23) Leibovici & Neoschil (1992) — the Rachford-Rice window
//! - (25) Crowe & Nishio (1975) — GDEM acceleration

// Parallel-array flash math (zᵢ, Kᵢ, xᵢ, yᵢ, ln-K residuals indexed in
// lockstep) — index loops mirror the equations; allow the range-loop lint.
#![allow(clippy::needless_range_loop)]

use smallvec::SmallVec;

use super::FlashError;
use super::init::wilson_ln_k;
use super::stability::{SCREEN_MARGIN, Seeds, Stability, pure_component_screen, stability_cached};
use super::system::min_gibbs_ln_phi_cached_into;
use super::system::{SystemSpec, SystemTpCache, ln_k_values_cached_into, phase_ln_phi_jacobian};
use crate::eos::{LiquidModel, PhaseId, VaporModel};

// ===========================================================================
// Reusable workspace (Part 1 §1 of the performance audit).
// ===========================================================================

/// Inline capacity for every per-component buffer in the flash.
///
/// Matched deliberately to the mixture core's `Buf<D> = SmallVec<[D; 8]>` so
/// the whole engine has **one** inline-width policy rather than two. Mixtures
/// wider than 8 spill to the heap — but a [`FlashWorkspace`] is built once per
/// flash and reused across every outer iteration, so a spill costs a single
/// allocation amortized over the entire solve, not one per iteration.
const INLINE_COMPONENTS: usize = 8;

/// A stack-resident vector of per-component values.
type WorkVec = SmallVec<[f64; INLINE_COMPONENTS]>;

/// Scratch buffers reused across every iteration of one isothermal flash.
///
/// The driver previously allocated `x`, `y`, `k_new`, the residual, and a fresh
/// `ln K` vector on **every** outer iteration — for a converging 8-component
/// flash that is roughly fifty allocator round trips for buffers whose size
/// never changes. Every one of those now lives here and is written in place.
///
/// Kept private to this module for now: the type is a performance detail, not
/// something a caller should have to name. When the Rayon batch path wants a
/// per-worker workspace (Part 2 §10 of the audit) this grows a public
/// `flash_isothermal_with_workspace` entry point; committing published-crate
/// API surface before that consumer exists would be premature.
struct FlashWorkspace {
    /// Current iterate, in log space — the variable the flash actually solves.
    ln_k: WorkVec,
    /// Freshly evaluated ln K from the fugacity models.
    ln_k_new: WorkVec,
    /// The plain successive-substitution target saved before a GDEM step, so a
    /// bad extrapolation can be undone without re-deriving it.
    ln_k_ss: WorkVec,
    /// Candidate accelerated iterate, committed only if it passes its bounds.
    trial: WorkVec,
    /// `exp(ln_k)` — the one exponentiation per iteration, for Rachford-Rice.
    k: WorkVec,
    /// Liquid mole fractions at the current β.
    x: WorkVec,
    /// Vapor mole fractions at the current β.
    y: WorkVec,
    /// ln-K residual `r = ln K_new − ln K`.
    r: WorkVec,
    /// The previous iteration's residual, for the GDEM eigenvalue estimate.
    r_prev: WorkVec,
}

impl FlashWorkspace {
    fn new(n: usize) -> Self {
        let zeros = || -> WorkVec { smallvec::smallvec![0.0; n] };
        Self {
            ln_k: zeros(),
            ln_k_new: zeros(),
            ln_k_ss: zeros(),
            trial: zeros(),
            k: zeros(),
            x: zeros(),
            y: zeros(),
            r: zeros(),
            r_prev: zeros(),
        }
    }
}

/// Result of an isothermal flash.
#[derive(Debug, Clone, PartialEq)]
pub struct FlashResult {
    /// Vapor fraction β = V/F, **dimensionless**. In `[0, 1]` for a genuine
    /// two-phase split. A single-phase feed gets exactly 0 (liquid) or 1
    /// (vapor); for a φ-φ system that label is density-based, and above the
    /// critical point it is a convention, not a physical distinction.
    pub beta: f64,
    /// Liquid mole fractions xᵢ (length N, sum to 1).
    pub x: Vec<f64>,
    /// Vapor mole fractions yᵢ (length N, sum to 1).
    pub y: Vec<f64>,
    /// Converged equilibrium ratios Kᵢ = yᵢ/xᵢ.
    pub k: Vec<f64>,
    /// Number of outer (SS/GDEM) iterations taken. An ordinary single-phase
    /// exit reports 0; a result produced by the non-convergence stability
    /// fallback reports at least the caller's `max_iter`.
    pub iterations: usize,
    /// `true` if the feed split into two phases; `false` if the flash
    /// resolved to a single phase (β clamped to 0 or 1).
    pub two_phase: bool,
}

// ===========================================================================
// Rachford-Rice (§F) — preprocessed terms + safeguarded Halley.
// ===========================================================================

/// Components whose `|Kᵢ − 1|` is below this are excluded from the pole
/// bracket: they contribute `~1e-14·zᵢ` to `f` (far under any usable
/// tolerance) while placing a pole at `±1e14` that would blow the
/// Leibovici–Neoschil window up to a numerically useless width.
const RR_DEGENERATE_C: f64 = 1e-14;

/// Rachford-Rice residual and its first two derivatives, from one summation
/// pass (§F):
/// ```text
///   f   =  Σ zᵢ·cᵢ/(1+β·cᵢ),      cᵢ = Kᵢ − 1
///   f'  = −Σ zᵢ·cᵢ²/(1+β·cᵢ)²
///   f'' =  2·Σ zᵢ·cᵢ³/(1+β·cᵢ)³
/// ```
///
/// Part 1 §3 of the performance audit proposed hoisting `cᵢ = Kᵢ − 1` and
/// `zᵢcᵢ` into a precomputed `[{c, zc}]` array, since both are invariant for a
/// whole solve. **Benchmarked and rejected**: the hoist saves one subtraction
/// and one multiply per component per evaluation — cheap ALU work that already
/// hides under the division's latency — while the preparation pass costs more
/// than that for a solve that converges in a handful of iterations, and the
/// array-of-structs layout it produces strides the loads that `z` and `k`
/// deliver contiguously. Measured at n = 2…8, it was a 30–200 % *regression*.
/// The `c`-invariance argument is sound; the arithmetic it saves is not the
/// arithmetic this loop is limited by.
#[inline]
fn rr_fdd(z: &[f64], k: &[f64], beta: f64) -> (f64, f64, f64) {
    let mut f = 0.0;
    let mut df = 0.0;
    let mut ddf = 0.0;
    for (&zi, &ki) in z.iter().zip(k) {
        let c = ki - 1.0;
        // One division per component; `inv` is then reused for every order.
        let inv = (1.0 + beta * c).recip();
        let base = zi * c * inv;
        let q = c * inv;
        f += base;
        df -= base * q;
        ddf += 2.0 * base * q * q;
    }
    (f, df, ddf)
}

/// Reject `(z, k)` inputs that are shaped correctly but numerically unusable.
///
/// Kept **out** of the per-iteration path and run only where untrusted values
/// enter — the public [`rachford_rice`] and the flash driver's entry, once.
/// This is Part 2 §13 of the performance audit ("separate checked APIs from
/// unchecked hot kernels") applied to the flash: validation costs about a
/// nanosecond per component, which is a fifth of the entire Rachford-Rice
/// solve at small `n`, and re-paying it on every outer iteration for values the
/// engine itself just produced is waste. The old `fold(f64::MIN, f64::max)`
/// bracket swallowed a NaN silently, so a bad input surfaced as an exhausted
/// iteration budget hundreds of steps later instead of as an error here.
///
/// # Errors
/// [`FlashError::InvalidInput`] for an empty mixture, a non-finite or negative
/// `zᵢ`, or a non-finite / non-positive `Kᵢ`.
fn rr_validate(z: &[f64], k: &[f64]) -> Result<(), FlashError> {
    if z.is_empty() {
        return Err(FlashError::InvalidInput("empty mixture".into()));
    }
    for (i, (&zi, &ki)) in z.iter().zip(k).enumerate() {
        if !zi.is_finite() || zi < 0.0 {
            return Err(FlashError::InvalidInput(format!("z[{i}]={zi}")));
        }
        if !ki.is_finite() || ki <= 0.0 {
            return Err(FlashError::InvalidInput(format!("K[{i}]={ki}")));
        }
    }
    Ok(())
}

/// `(Kmax, Kmin)` over the components that can actually influence the root.
///
/// Two classes are excluded from the extremes (Part 1 §3 of the performance
/// audit). Both contribute `~0` to every sum already, so leaving them in
/// [`rr_fdd`] is harmless — but letting them set the bracket is not:
///
/// - `zᵢ = 0` — absent from the feed, yet its Kᵢ would still pin a pole and
///   could make an unsolvable system look bracketable (or vice versa).
/// - `|Kᵢ − 1| ≤ 1e-14` — a pole at `±1e14`, which inflates the
///   Leibovici–Neoschil window to a numerically useless width.
///
/// Returns the degenerate `(1, 1)` when no component has a driving force, so
/// [`rr_solve`] raises the same `NoRachfordRiceRoot` a one-sided K set gives —
/// which the flash driver already resolves to a single phase.
///
/// Assumes [`rr_validate`] has passed; with a NaN present the result is a
/// degenerate bracket rather than an error.
#[inline]
fn rr_bracket(z: &[f64], k: &[f64]) -> (f64, f64) {
    let mut kmax = f64::NEG_INFINITY;
    let mut kmin = f64::INFINITY;
    for (&zi, &ki) in z.iter().zip(k) {
        // Branchless select so the reduction still vectorizes: an inactive
        // component contributes the identity of each extreme.
        let usable = zi != 0.0 && (ki - 1.0).abs() > RR_DEGENERATE_C;
        kmax = kmax.max(if usable { ki } else { f64::NEG_INFINITY });
        kmin = kmin.min(if usable { ki } else { f64::INFINITY });
    }
    if kmax == f64::NEG_INFINITY {
        return (1.0, 1.0);
    }
    (kmax, kmin)
}

/// Move an endpoint off a pole by a step that is safe at **both** ends of the
/// magnitude range: a small fraction of the interval when the interval is
/// wide, and a few ulps of the endpoint itself when it is not.
///
/// The previous fixed `1e-10·span` offset failed in both directions — too
/// coarse for a narrow window, and too fine to actually move a large-magnitude
/// endpoint to a different representable number.
#[inline]
fn nudge_off_pole(v: f64, span: f64, upward: bool) -> f64 {
    let delta = (1e-10 * span)
        .max(v.abs() * 16.0 * f64::EPSILON)
        .max(f64::MIN_POSITIVE);
    if upward { v + delta } else { v - delta }
}

/// Solve for β given prepared terms and the active-K extremes.
///
/// Halley's method inside a sign bracket, with a Brent-style safeguard: if the
/// bracket has not at least halved over two iterations, the step is forced to a
/// bisection. That keeps the guaranteed geometric bracket reduction of pure
/// bisection while letting Halley's cubic convergence do the work whenever it
/// behaves — the previous kernel accepted any in-bracket Halley step, including
/// ones from a catastrophically ill-conditioned denominator.
fn rr_solve(
    z: &[f64],
    k: &[f64],
    kmax: f64,
    kmin: f64,
    tol: f64,
    max_iter: usize,
) -> Result<f64, FlashError> {
    if !tol.is_finite() || tol <= 0.0 {
        return Err(FlashError::InvalidInput(format!("tolerance={tol}")));
    }
    // A bracketable interior root needs Kmax > 1 > Kmin.
    if kmax <= 1.0 || kmin >= 1.0 {
        return Err(FlashError::NoRachfordRiceRoot { kmax, kmin });
    }
    // Leibovici–Neoschil window (open interval between the two bounding
    // poles). f(β_lo⁺) = +∞, f(β_hi⁻) = −∞.
    let beta_lo = 1.0 / (1.0 - kmax); // < 0
    let beta_hi = 1.0 / (1.0 - kmin); // > 1
    let span = beta_hi - beta_lo;
    let mut lo = nudge_off_pole(beta_lo, span, true);
    let mut hi = nudge_off_pole(beta_hi, span, false);

    // Part 1 §3 of the audit also proposed probing f(0) and f(1) first and
    // narrowing the bracket to [0, 1] on the physical case. **Benchmarked and
    // rejected**: for a real mixture the Leibovici–Neoschil poles already sit
    // just outside [0, 1] (they are set by the most and least volatile
    // components), so the narrowing saves essentially no iterations while the
    // two probes cost two extra divisions per component — measured as a 25–200 %
    // regression on the Rachford-Rice benches at n = 2…8.
    let mut beta = 0.5 * (lo + hi);
    // Bracket widths one and two iterations ago, for the halving safeguard.
    let mut width_prev = hi - lo;
    let mut width_prev2 = hi - lo;
    let mut last_f = f64::NAN;

    for iter in 0..max_iter {
        let (f, df, ddf) = rr_fdd(z, k, beta);
        last_f = f;
        if f.abs() <= tol {
            return Ok(beta);
        }
        // Tighten the bracket using the sign of f (f decreasing ⇒ f>0 is the
        // lower side).
        if f > 0.0 {
            lo = beta;
        } else {
            hi = beta;
        }
        let width = hi - lo;
        // β is pinned to machine precision: a tighter |f| is unreachable, and
        // continuing would burn the whole iteration budget for nothing.
        if width <= 4.0 * f64::EPSILON * beta.abs().max(1.0) {
            return Ok(0.5 * (lo + hi));
        }

        // Halley step β − 2·f·f' / (2·f'² − f·f''), rejected when the
        // denominator is not meaningfully non-zero *relative to the terms that
        // formed it* — near a critical point every Kᵢ → 1 and f, f', f'' vanish
        // at different rates, which is exactly where an absolute `!= 0` test
        // accepts a garbage quotient.
        let denom = 2.0 * df * df - f * ddf;
        let scale = (2.0 * df * df).abs() + (f * ddf).abs();
        let halley_ok = denom.is_finite() && denom.abs() > 32.0 * f64::EPSILON * scale;
        // Brent's safeguard: force a bisection whenever the bracket has failed
        // to halve over the last two steps. Bisection alone guarantees
        // geometric bracket reduction but converges linearly; Halley alone can
        // creep toward one endpoint indefinitely. Interleaving them this way
        // keeps Halley's cubic convergence in the common case and a guaranteed
        // rate in the pathological one — the previous kernel accepted *any*
        // in-bracket Halley step, with no progress guarantee at all.
        let must_bisect = width > 0.5 * width_prev2;
        width_prev2 = width_prev;
        width_prev = width;
        let next = if halley_ok && !must_bisect {
            beta - 2.0 * f * df / denom
        } else {
            f64::NAN
        };
        beta = if next.is_finite() && next > lo && next < hi {
            next
        } else {
            0.5 * (lo + hi)
        };
        if iter + 1 == max_iter {
            let (f, _, _) = rr_fdd(z, k, beta);
            return Err(FlashError::NoConvergence {
                what: "Rachford-Rice",
                iters: max_iter,
                residual: f.abs(),
            });
        }
    }
    // Only reachable with max_iter == 0, which never met the cap check above.
    Err(FlashError::NoConvergence {
        what: "Rachford-Rice",
        iters: max_iter,
        residual: last_f.abs(),
    })
}

/// Solve the Rachford-Rice equation for the vapor fraction β at fixed
/// K-values (§F).
///
/// Halley's method inside the Leibovici–Neoschil window
/// `β ∈ (1/(1−Kmax), 1/(1−Kmin))`, where f is monotone decreasing and
/// pole-free, with a bisection safeguard — so the iteration cannot diverge.
/// The returned β may lie outside `[0, 1]` (negative flash) when the feed
/// is single-phase; callers clamp as needed.
///
/// # Arguments
/// * `z` — feed mole fractions (length N, sum to 1).
/// * `k` — equilibrium ratios Kᵢ (length N).
/// * `tol` — convergence tolerance on |f(β)|; must be finite and positive.
/// * `max_iter` — iteration cap.
///
/// # Errors
/// [`FlashError::Dimension`] on length mismatch;
/// [`FlashError::InvalidInput`] for an empty mixture, a non-finite/negative
/// `zᵢ`, a non-finite/non-positive `Kᵢ`, or a non-positive tolerance;
/// [`FlashError::NoRachfordRiceRoot`] if there is no interior root
/// (`Kmax ≤ 1` or `Kmin ≥ 1` — the mixture cannot be two-phase at these K);
/// [`FlashError::NoConvergence`] if the cap is hit.
pub fn rachford_rice(z: &[f64], k: &[f64], tol: f64, max_iter: usize) -> Result<f64, FlashError> {
    let n = z.len();
    if k.len() != n {
        return Err(FlashError::Dimension(format!("z={n}, k={}", k.len())));
    }
    rr_validate(z, k)?;
    let (kmax, kmin) = rr_bracket(z, k);
    rr_solve(z, k, kmax, kmin, tol, max_iter)
}

/// Split a feed `z` into liquid `x` and vapor `y` at vapor fraction `β`
/// and K-values `k`: `xᵢ = zᵢ/(1+β(Kᵢ−1))`, `yᵢ = Kᵢ·xᵢ` — written into
/// caller-owned buffers.
#[inline]
fn split_into(z: &[f64], k: &[f64], beta: f64, x: &mut [f64], y: &mut [f64]) {
    for i in 0..z.len() {
        x[i] = z[i] / (1.0 + beta * (k[i] - 1.0));
        y[i] = k[i] * x[i];
    }
}

// ===========================================================================
// GDEM acceleration (§J) — now inside a trust region (Part 1 §5 of the audit).
// ===========================================================================

/// Reject the eigenvalue estimate above this: as `μ → 1` the amplification
/// `1/(1−μ)` diverges, and an unbounded extrapolation is precisely how a
/// slowly-converging near-critical state turns into an overflow.
const GDEM_MU_MAX: f64 = 0.95;

/// Hard cap on the extrapolation factor even for an accepted `μ`.
const GDEM_GAIN_MAX: f64 = 4.0;

/// `|ln K|` beyond which a candidate iterate is refused. `exp(80)` is already
/// far outside any physical equilibrium ratio, so a proposal that reaches it is
/// a runaway, not a solution.
const LN_K_BOUND: f64 = 80.0;

/// GDEM amplification factor from the last two ln-K residual vectors
/// (Crowe & Nishio, Ref (25)).
///
/// One-mode GDEM: `μ = (r·r_prev)/(r_prev·r_prev)`, and the accelerated update
/// is `ln K ← ln K + r/(1−μ)`. Returns `None` — meaning "take a plain
/// successive-substitution step" — when the estimate is unusable or too
/// aggressive to trust.
fn gdem_gain(r: &[f64], r_prev: &[f64]) -> Option<f64> {
    let mut num = 0.0;
    let mut den = 0.0;
    for (&ri, &rp) in r.iter().zip(r_prev) {
        num += ri * rp;
        den += rp * rp;
    }
    // Written as explicit finite + magnitude tests rather than a negated
    // comparison: a NaN residual must be *refused*, and `NaN <= x` is false.
    if !den.is_finite() || den <= f64::MIN_POSITIVE {
        return None;
    }
    let mu = num / den;
    if !mu.is_finite() || mu <= 0.0 || mu >= GDEM_MU_MAX {
        return None;
    }
    Some((1.0 / (1.0 - mu)).min(GDEM_GAIN_MAX))
}

/// Build the accelerated iterate `ln K + gain·r` into `trial`, returning
/// `false` (and leaving `trial` unusable) if any component leaves the
/// finite, bounded region. Writing to a separate buffer means a rejected
/// proposal never corrupts the current iterate.
fn gdem_trial(ln_k: &[f64], r: &[f64], gain: f64, trial: &mut [f64]) -> bool {
    for i in 0..ln_k.len() {
        let candidate = ln_k[i] + gain * r[i];
        if !candidate.is_finite() || !(-LN_K_BOUND..=LN_K_BOUND).contains(&candidate) {
            return false;
        }
        trial[i] = candidate;
    }
    true
}

/// Isothermal (PT) flash by Wilson-initialized, GDEM-accelerated successive
/// substitution (§J).
///
/// # Arguments
/// * `spec` — the mixture's thermodynamic model.
/// * `t` — Temperature in **K**; `p` — Pressure in **kPa absolute**.
/// * `z` — feed mole fractions (length N, sum to 1).
/// * `tol` — composition-residual tolerance (‖Δ ln K‖∞).
/// * `max_iter` — outer iteration cap.
///
/// # Returns
/// A [`FlashResult`]. If the feed is single-phase at `(T, P)` the result has
/// `two_phase = false` and `β` clamped to 0 (liquid) or 1 (vapor) with
/// `x = y = z`.
///
/// If the successive-substitution loop exhausts `max_iter` on a cubic (φ-φ)
/// system, a TPD stability test decides the outcome instead of returning the
/// error straight away: a **stable** feed returns the single-phase result
/// (with `iterations == max_iter`), an **unstable** one is re-flashed once
/// from the stability test's trial-phase K. That fallback may spend up to
/// ~10 × `max_iter` further iterations, and only after a failure. See `OPTIMIZATION_PLAN_PART1.md`
/// §10 for the measurement behind this.
///
/// # Errors
/// [`FlashError`] on dimension mismatch, thermodynamic failure, or
/// non-convergence that the stability fallback could not resolve.
pub fn flash_isothermal(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    tol: f64,
    max_iter: usize,
) -> Result<FlashResult, FlashError> {
    flash_isothermal_warm(spec, t, p, z, None, tol, max_iter)
}

/// Isothermal flash with an optional **warm-start** K-value vector (§J, §M).
///
/// Identical to [`flash_isothermal`] but seeds the K-loop from `k_init` when
/// given (e.g. the previous temperature's converged K in the adiabatic
/// flash's nested loop), falling back to Wilson otherwise. `k_init`, when
/// present, must have length N.
///
/// A warm start that gives Rachford–Rice **no root at the first iteration**
/// is discarded and the solve restarts from Wilson, so a stale K cannot by
/// itself declare the feed single-phase. Callers should still only carry K
/// forward from a `two_phase` result — a single-phase result's K is
/// `exp(ln K)` of whatever iterate RR rejected, not an equilibrium ratio.
pub fn flash_isothermal_warm(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    k_init: Option<&[f64]>,
    tol: f64,
    max_iter: usize,
) -> Result<FlashResult, FlashError> {
    flash_impl(spec, t, p, z, k_init, tol, max_iter, true)
}

/// The body of [`flash_isothermal_warm`]; `newton` enables the Newton finish.
#[allow(clippy::too_many_arguments)]
fn flash_impl(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    k_init: Option<&[f64]>,
    tol: f64,
    max_iter: usize,
    newton: bool,
) -> Result<FlashResult, FlashError> {
    let n = spec.n();
    if z.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, z={}",
            z.len()
        )));
    }

    // The feed is loop-invariant, so validate it **once** here instead of on
    // every outer iteration. The K-values the loop feeds Rachford-Rice are the
    // engine's own `exp(ln K)`, so they need no per-iteration re-checking.
    if let Some(i) = z.iter().position(|&zi| !zi.is_finite() || zi < 0.0) {
        return Err(FlashError::InvalidInput(format!("z[{i}]={}", z[i])));
    }

    // One workspace for the whole solve: every per-iteration buffer below is
    // written in place rather than allocated (Part 1 §1 of the audit).
    let mut ws = FlashWorkspace::new(n);
    // One (T, P) cache for the whole solve: the pure-component EOS pass and the
    // γ-φ Psat/φˢᵃᵗ/Poynting/activity-matrix work are composition-independent,
    // so they happen **once** here instead of twice per outer iteration
    // (Part 2 §1 and §5 of the audit).
    let cache = SystemTpCache::new(spec, t, p)?;

    // Warm-start from the supplied K if it is shaped right *and* usable — a
    // caller's stale K vector with a zero or negative entry would otherwise
    // seed the log-space iterate with −∞/NaN.
    let warm_ok =
        matches!(k_init, Some(k0) if k0.len() == n && k0.iter().all(|&v| v.is_finite() && v > 0.0));
    if warm_ok {
        let k0 = k_init.expect("checked above");
        for i in 0..n {
            ws.ln_k[i] = k0[i].ln();
        }
    } else {
        for (i, comp) in spec.components.iter().enumerate() {
            ws.ln_k[i] = wilson_ln_k(comp, t, p);
        }
    }

    let mode = Loop {
        seeded_warm: warm_ok,
        check_single_phase: true,
        newton,
    };
    match ss_gdem(spec, &cache, t, p, z, &mut ws, mode, tol, max_iter) {
        // The loop ran out of iterations: ask the stability test before
        // giving up (see `stability_fallback`).
        Err(err @ FlashError::NoConvergence { .. }) => {
            stability_fallback(spec, &cache, t, p, z, &ws, tol, max_iter).unwrap_or(Err(err))
        }
        other => other,
    }
}

/// Iteration budget of the non-convergence fallback, as a multiple of the
/// caller's `max_iter` — see [`stability_fallback`].
const FALLBACK_ITER_FACTOR: usize = 10;

/// What to do when the SS/GDEM loop exhausts `max_iter` — Step 0 of
/// `OPTIMIZATION_NEXT_STEPS.md`, on the evidence of `OPTIMIZATION_PLAN_PART1.md`
/// §10.
///
/// Near the critical locus the K-values of a feed that is in fact a single
/// phase can drift toward 1 without Rachford–Rice ever losing its root, so the
/// loop runs out of iterations instead of declaring "single phase". A TPD
/// stability test (Ref (7)) answers the question the loop could not:
///
/// * **Stable**, with both trial phases converged → the single-phase answer.
///   On the §10 hydrocarbon grid this is every failure point.
/// * **Unstable** → the trial phase is a non-trivial K estimate; re-run the
///   flash once from it and accept the result only if it is a genuine split.
/// * Anything else (a non-cubic system, a trial that did not converge, a zero
///   feed component, the retry failing) → `None`, and the caller returns the
///   original `NoConvergence`. The fallback never manufactures an answer the
///   stability test did not vouch for.
///
/// Costs nothing on the normal path: it runs only after a failure. The
/// stability trials and the retry each get [`FALLBACK_ITER_FACTOR`] ×
/// `max_iter` iterations, so a failing call can spend up to roughly that
/// multiple of the caller's budget before it returns.
///
/// The single-phase label (β = 0 or 1) comes from the sign of the
/// Rachford–Rice residual at β = 0 under the last iterate's K, the same rule
/// as every other single-phase exit. Near the critical point "liquid" versus
/// "vapor" is a convention, not a measurable distinction; x = y = z either
/// way. The returned `iterations` is the full `max_iter` that was spent (plus
/// the retry's iterations on the unstable branch), so a caller can tell this
/// answer from an ordinary single-phase exit, which reports 0.
///
/// # References
/// - (7) Michelsen (1982) Part I — stability analysis
#[allow(clippy::too_many_arguments)]
fn stability_fallback(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    t: f64,
    p: f64,
    z: &[f64],
    ws: &FlashWorkspace,
    tol: f64,
    max_iter: usize,
) -> Option<Result<FlashResult, FlashError>> {
    // Stability needs every zᵢ > 0 (dᵢ = ln zᵢ); a feed with an absent
    // component keeps the original error rather than a partial verdict.
    if z.iter().any(|&zi| zi <= 0.0) {
        return None;
    }
    // Near the critical locus both the TPD trials and the retried flash
    // converge slowly: on the §10 grid two methane/n-pentane points needed
    // > 200 trial iterations (and a 200-iteration trial *defaults* to
    // "stable" — which would have been the wrong answer at both) and 351 /
    // 833 flash iterations from the trial K.
    let budget = max_iter.saturating_mul(FALLBACK_ITER_FACTOR);
    // `Err` here is a non-cubic liquid model or a fugacity failure — either
    // way there is no verdict to act on.
    let v = stability_cached(spec, cache, t, p, z, budget, Seeds::WilsonThenPure).ok()?;
    match v.verdict {
        Stability::Stable if v.conclusive => {
            let mut res = single_phase(spec, t, p, z, &ws.k);
            res.iterations = max_iter;
            Some(Ok(res))
        }
        Stability::Stable => None,
        Stability::Unstable { .. } => {
            let mut r = retry_from_trial(spec, cache, t, p, z, &v.verdict, tol, budget)?;
            r.iterations += max_iter;
            Some(Ok(r))
        }
    }
}

/// How one run of the outer loop behaves.
#[derive(Debug, Clone, Copy)]
struct Loop {
    /// The starting ln K is a caller's rather than Wilson's, which enables
    /// the iteration-0 Wilson re-seed.
    seeded_warm: bool,
    /// Run [`single_phase_check`] before every single-phase exit. The retries
    /// that check and the fallback make pass `false`, so a retry can never
    /// recurse.
    check_single_phase: bool,
    /// Allow the Newton finish ([`newton_step`]) near the solution.
    newton: bool,
}

/// ln-K residual below which the loop may switch from successive
/// substitution to the Newton finish. Chosen by measurement —
/// `OPTIMIZATION_PLAN_PART1.md` §18.
const NEWTON_SWITCH: f64 = 1e-2;

/// SS contraction ratio (this residual ÷ the previous one) above which the
/// Newton step is worth its cost; a faster-contracting solve finishes sooner
/// on SS + GDEM. Chosen by measurement — `OPTIMIZATION_PLAN_PART1.md` §18.
const NEWTON_MIN_RATIO: f64 = 0.3;

/// A Newton step is cut back to this fraction of the distance that would
/// empty a component out of one phase.
const NEWTON_BOUNDARY_FRACTION: f64 = 0.9;

/// One Newton step of the two-phase Gibbs-energy minimization in the vapor
/// mole numbers `vᵢ` (per mole of feed; `lᵢ = zᵢ − vᵢ`) — the second-order
/// finish of Michelsen's flash (Ref (19); Michelsen & Mollerup (26) ch. 10).
///
/// The gradient is the fugacity mismatch
/// `gᵢ = ln f̂ᵢⱽ − ln f̂ᵢᴸ = ln Kᵢ − ln Kᵢᵐᵒᵈᵉˡ = −rᵢ` and the Hessian
/// ```text
///   Hᵢⱼ = (1/β)·(δᵢⱼ/yᵢ − 1 + Gⱽᵢⱼ) + (1/(1−β))·(δᵢⱼ/xᵢ − 1 + Gᴸᵢⱼ)
/// ```
/// with `G = ∂ln φ̂ᵢ/∂nⱼ` of each phase at total moles 1
/// ([`phase_ln_phi_jacobian`]). `H·Δv = r` gives the step. Working in `v`
/// rather than in `{ln K, β}` is what keeps the system well conditioned near
/// the critical point: H is the Hessian of G, positive definite at a stable
/// two-phase solution, where the `ln K` Jacobian becomes rank-deficient as
/// `K → 1` (`OPTIMIZATION_PLAN_PART1.md` §5).
///
/// Writes the new iterate `ln Kᵢ = ln(yᵢ/xᵢ)` into `ln_k_out` and returns
/// `true`. Returns `false` — the caller stays on SS — when the model pair has
/// no composition Jacobian, a component is absent from the feed or a phase,
/// the system is singular, or the step would have to be cut to less than half
/// its length to keep every `0 < vᵢ < zᵢ` (too far out for Newton to be
/// trusted). `t` in **K**, `p` in **kPa absolute**.
#[allow(clippy::too_many_arguments)]
fn newton_step(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    beta: f64,
    x: &[f64],
    y: &[f64],
    r: &[f64],
    ln_k_out: &mut [f64],
) -> bool {
    let n = z.len();
    let positive = |v: &[f64]| v.iter().all(|&a| a > 0.0 && a.is_finite());
    if !(beta > 0.0 && beta < 1.0) || !positive(z) || !positive(x) || !positive(y) {
        return false;
    }
    let (Some(gv), Some(gl)) = (
        phase_ln_phi_jacobian(spec, t, p, y, PhaseId::Vapor),
        phase_ln_phi_jacobian(spec, t, p, x, PhaseId::Liquid),
    ) else {
        return false;
    };
    let (inv_v, inv_l) = (1.0 / beta, 1.0 / (1.0 - beta));
    // `from_fn` fills the dense n×n Hessian entry by entry.
    let h = nalgebra::DMatrix::from_fn(n, n, |i, j| {
        let d = f64::from(u8::from(i == j));
        inv_v * (d / y[i] - 1.0 + gv[i][j]) + inv_l * (d / x[i] - 1.0 + gl[i][j])
    });
    let rhs = nalgebra::DVector::from_column_slice(r);
    let Some(dv) = h.lu().solve(&rhs) else {
        return false;
    };
    if dv.iter().any(|v| !v.is_finite()) {
        return false;
    }
    // Fraction-to-the-boundary rule: keep every 0 < vᵢ < zᵢ.
    let mut scale: f64 = 1.0;
    for i in 0..n {
        let v = beta * y[i];
        let room = if dv[i] > 0.0 { z[i] - v } else { v };
        if dv[i].abs() > NEWTON_BOUNDARY_FRACTION * room {
            scale = scale.min(NEWTON_BOUNDARY_FRACTION * room / dv[i].abs());
        }
    }
    if scale < 0.5 || scale.is_nan() {
        return false;
    }
    let (mut v_tot, mut l_tot) = (0.0, 0.0);
    for i in 0..n {
        let v = beta * y[i] + scale * dv[i];
        v_tot += v;
        l_tot += z[i] - v;
        // Hold vᵢ and lᵢ in the output slot until the totals are known.
        ln_k_out[i] = (v / (z[i] - v)).ln();
    }
    // ln Kᵢ = ln(vᵢ/V) − ln(lᵢ/L) = ln(vᵢ/lᵢ) + ln(L/V).
    let shift = (l_tot / v_tot).ln();
    if !shift.is_finite() {
        return false;
    }
    for lk in ln_k_out.iter_mut() {
        *lk += shift;
    }
    ln_k_out
        .iter()
        .all(|lk| lk.is_finite() && lk.abs() <= LN_K_BOUND)
}

/// The outer loop (§J), starting from the ln K already in `ws.ln_k`:
/// GDEM-accelerated successive substitution, finished by Newton
/// ([`newton_step`]) where `mode.newton` allows and SS has become slow.
#[allow(clippy::too_many_arguments)]
fn ss_gdem(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    t: f64,
    p: f64,
    z: &[f64],
    ws: &mut FlashWorkspace,
    mode: Loop,
    tol: f64,
    max_iter: usize,
) -> Result<FlashResult, FlashError> {
    let n = z.len();
    let check_single_phase = mode.check_single_phase;
    // True while the iterate is still the caller's K (see the re-seed below).
    let mut seeded_warm = mode.seeded_warm;
    // Newton finish: allowed until a step fails or does not reduce the
    // residual; `in_newton` once a step has been taken; `newton_pending` is
    // the residual before the last step, checked on the next pass.
    let mut newton_ok = mode.newton;
    let mut in_newton = false;
    let mut newton_pending: Option<f64> = None;
    // Residual of the previous plain-SS pass, for the contraction ratio.
    let mut prev_resid: Option<f64> = None;
    // Has the model produced a K yet, or is the iterate still a seed
    // (Wilson's, or a caller's)? See the model re-seed below.
    let mut model_seen = false;
    let mut have_r_prev = false;
    // Residual recorded just before an accelerated step, so the *next*
    // iteration can tell whether the extrapolation actually helped.
    let mut gdem_pending: Option<f64> = None;

    for iter in 0..max_iter {
        // The single exponentiation per iteration: Rachford-Rice is the one
        // consumer that genuinely needs Kᵢ rather than ln Kᵢ.
        for i in 0..n {
            ws.k[i] = ws.ln_k[i].exp();
        }

        // Inner Rachford-Rice; if there's no interior root the feed is
        // single-phase at the current K — decide which side and return.
        let (kmax, kmin) = rr_bracket(z, &ws.k);
        let beta = match rr_solve(z, &ws.k, kmax, kmin, 1e-12, 200) {
            Ok(b) => b,
            Err(FlashError::NoRachfordRiceRoot { .. }) if seeded_warm && iter == 0 => {
                // A caller's warm-start K says "single phase" before a single
                // fugacity has been evaluated at this (T, P). That K came from
                // some *other* state, so it is not evidence about this one:
                // a K carried from a superheated-vapor result, for example,
                // reports "all vapor" at every lower temperature too (the
                // v0.17.0 adiabatic bug, OPTIMIZATION_PLAN_PART1 §9). Re-seed
                // from Wilson — exactly what a cold flash would have used — and
                // let *that* decide. Costs one Wilson pass, only in this case.
                for (i, comp) in spec.components.iter().enumerate() {
                    ws.ln_k[i] = wilson_ln_k(comp, t, p);
                }
                seeded_warm = false;
                continue;
            }
            Err(FlashError::NoRachfordRiceRoot { .. })
                if !model_seen && !matches!(spec.liquid, LiquidModel::Cubic(_)) =>
            {
                // Wilson's K says "single phase" and the model has not been
                // asked. Wilson knows only Tc, Pc and ω; a γ-φ model's K comes
                // from its own Psat correlations and activity coefficients,
                // and the two can disagree about which side of a bubble or
                // dew point this state is on. Re-seed once from the model's K
                // at x = y = z and let that decide (OPTIMIZATION_PLAN_PART1
                // §19). For a cubic pair that K is identically 1, which is
                // why the cubic path has `single_phase_check` instead.
                ws.x.copy_from_slice(z);
                ws.y.copy_from_slice(z);
                ln_k_values_cached_into(spec, cache, &ws.x, &ws.y, &mut ws.ln_k_new)?;
                ws.ln_k.copy_from_slice(&ws.ln_k_new);
                model_seen = true;
                seeded_warm = false;
                continue;
            }
            Err(FlashError::NoRachfordRiceRoot { .. }) => {
                if check_single_phase {
                    if let Some(r) = single_phase_check(spec, cache, t, p, z, tol, max_iter) {
                        return r;
                    }
                }
                return Ok(single_phase(spec, t, p, z, &ws.k));
            }
            Err(e) => return Err(e),
        };
        split_into(z, &ws.k, beta, &mut ws.x, &mut ws.y);

        // Recompute K from the fugacity models — in log form, so the residual
        // below is a plain subtraction instead of a division and a logarithm.
        ln_k_values_cached_into(spec, cache, &ws.x, &ws.y, &mut ws.ln_k_new)?;
        model_seen = true;

        // ln-K residual r = ln K_new − ln K.
        let mut resid = 0.0_f64;
        for i in 0..n {
            ws.r[i] = ws.ln_k_new[i] - ws.ln_k[i];
            resid = resid.max(ws.r[i].abs());
        }

        if resid <= tol {
            // Converged. If β landed outside [0,1] the feed is single-phase.
            for i in 0..n {
                ws.k[i] = ws.ln_k_new[i].exp();
            }
            // A converged K ≡ 1 is the trivial solution, whatever β the
            // Rachford–Rice solve happened to return for it.
            let trivial = ws.ln_k_new.iter().all(|lk| lk.abs() < TRIVIAL_LN_K);
            if trivial || !(0.0..=1.0).contains(&beta) {
                if check_single_phase {
                    if let Some(r) = single_phase_check(spec, cache, t, p, z, tol, max_iter) {
                        return r;
                    }
                }
                return Ok(single_phase(spec, t, p, z, &ws.k));
            }
            return Ok(FlashResult {
                beta,
                x: ws.x.to_vec(),
                y: ws.y.to_vec(),
                k: ws.k.to_vec(),
                iterations: iter + 1,
                two_phase: true,
            });
        }

        // A Newton step that did not reduce the residual is undone in favor
        // of the SS step saved beside it, and Newton is off for this solve.
        if let Some(resid_before) = newton_pending.take() {
            if resid >= resid_before || resid.is_nan() {
                ws.ln_k.copy_from_slice(&ws.ln_k_ss);
                newton_ok = false;
                in_newton = false;
                have_r_prev = false;
                prev_resid = None;
                continue;
            }
        }

        // Retrospective trust-region check: if the previous step was an
        // extrapolation and the residual grew, undo it and take the plain
        // successive-substitution step that was saved instead. This costs one
        // wasted model evaluation, but only in the case where the acceleration
        // was actively harmful.
        if let Some(resid_before) = gdem_pending.take() {
            if resid > resid_before {
                ws.ln_k.copy_from_slice(&ws.ln_k_ss);
                // The residual history spans a discarded iterate, so the GDEM
                // eigenvalue estimate is no longer meaningful — restart it.
                have_r_prev = false;
                continue;
            }
        }

        // Newton finish: close to the solution and SS contracting slowly (or
        // already on Newton). A declined step turns it off for good.
        let slow = prev_resid.is_some_and(|prev| resid > NEWTON_MIN_RATIO * prev);
        let mut newton_taken = false;
        if newton_ok && (in_newton || slow) && resid < NEWTON_SWITCH && beta > 0.0 && beta < 1.0 {
            if newton_step(spec, t, p, z, beta, &ws.x, &ws.y, &ws.r, &mut ws.trial) {
                ws.ln_k_ss.copy_from_slice(&ws.ln_k_new);
                std::mem::swap(&mut ws.ln_k, &mut ws.trial);
                newton_pending = Some(resid);
                in_newton = true;
                newton_taken = true;
                // The residual history no longer describes an SS sequence.
                have_r_prev = false;
                prev_resid = None;
            } else {
                newton_ok = false;
                in_newton = false;
            }
        }

        // GDEM acceleration on ln K every few iterations.
        let mut accelerated = newton_taken;
        if !newton_taken && have_r_prev && iter % 5 == 4 {
            if let Some(gain) = gdem_gain(&ws.r, &ws.r_prev) {
                if gdem_trial(&ws.ln_k, &ws.r, gain, &mut ws.trial) {
                    // Keep the plain-SS alternative before committing.
                    ws.ln_k_ss.copy_from_slice(&ws.ln_k_new);
                    std::mem::swap(&mut ws.ln_k, &mut ws.trial);
                    gdem_pending = Some(resid);
                    accelerated = true;
                }
            }
        }
        if !accelerated {
            // Plain SS step: ln K ← ln K + r  (i.e. K ← K_new).
            ws.ln_k.copy_from_slice(&ws.ln_k_new);
            prev_resid = Some(resid);
        } else if !newton_taken {
            // The ratio across an extrapolated step says nothing about SS.
            prev_resid = None;
        }
        if !newton_taken {
            ws.r_prev.copy_from_slice(&ws.r);
            have_r_prev = true;
        }

        if iter + 1 == max_iter {
            return Err(FlashError::NoConvergence {
                what: "isothermal flash",
                iters: max_iter,
                residual: resid,
            });
        }
    }
    // Only reachable with max_iter == 0, which never met the cap check above.
    Err(FlashError::NoConvergence {
        what: "isothermal flash",
        iters: max_iter,
        residual: f64::INFINITY,
    })
}

/// The check every single-phase exit of a cubic flash runs before it is
/// returned (OPTIMIZATION_PLAN_PART1 §13).
///
/// The flash calls a feed single-phase when Rachford–Rice has no root under
/// its current K — at a cold start that is Wilson's K, before any fugacity
/// has been evaluated. For a liquid–liquid split that is the wrong answer
/// whenever every Wilson Kᵢ < 1: on PR water/hydrocarbon it was wrong at 42 %
/// of a T–P grid. So, first, the pure-component TPD screen
/// ([`pure_component_screen`]): one mixture ln φ at the feed plus one pure
/// cubic per component. It *proves* instability when it fires and cannot
/// fire on a stable feed, so on hydrocarbon systems this is the whole cost.
/// When it fires, the multi-seed stability test supplies a trial-phase K and
/// the flash is re-run once from it (no further checks). A genuine split is
/// returned; otherwise `None` and the single-phase answer stands — the
/// screen proved instability the two-phase flash could not resolve, which a
/// three-phase region does (see the §13 notes).
fn single_phase_check(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    t: f64,
    p: f64,
    z: &[f64],
    tol: f64,
    max_iter: usize,
) -> Option<Result<FlashResult, FlashError>> {
    if !matches!(spec.liquid, LiquidModel::Cubic(_)) || z.iter().any(|&zi| zi <= 0.0) {
        return None;
    }
    let mut ln_phi_feed: WorkVec = smallvec::smallvec![0.0; z.len()];
    min_gibbs_ln_phi_cached_into(spec, cache, z, &mut ln_phi_feed).ok()?;
    let min_tm = pure_component_screen(spec, cache, t, p, z, &ln_phi_feed)?;
    // Far from every pure corner: accept the single-phase answer. Inside the
    // margin the full test decides — it costs n + 2 trials on a stable feed,
    // which is why it is not run on every exit (§20).
    if min_tm >= SCREEN_MARGIN {
        return None;
    }
    let budget = max_iter.saturating_mul(FALLBACK_ITER_FACTOR);
    // All n + 2 seeds, also inside the margin: the Wilson pair alone left 10
    // of the 32 tuning-sweep misses and 24 of the 87 held-out ones (§20).
    let v = stability_cached(spec, cache, t, p, z, budget, Seeds::WilsonThenPure).ok()?;
    retry_from_trial(spec, cache, t, p, z, &v.verdict, tol, budget).map(Ok)
}

/// Re-flash once from an unstable verdict's trial phase, trying both
/// orientations of its K estimate, and return the first genuine split.
///
/// Which side of the flash the incipient phase belongs on is not knowable in
/// advance for a near-pure trial: water condensing out of a vapor feed is the
/// `x` side, water separating from a hydrocarbon liquid is either. The
/// seed's own guess (`trial_k`) is tried first, then its inverse. Measured on
/// the §13 water/hydrocarbon sweep: the guess alone resolves 9 088 of the
/// grid's splits, orienting by the feed's single-phase label alone 8 211,
/// both in turn more than either (OPTIMIZATION_PLAN_PART1 §13). The retries
/// run without the single-phase check, so they cannot recurse.
#[allow(clippy::too_many_arguments)]
fn retry_from_trial(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    t: f64,
    p: f64,
    z: &[f64],
    verdict: &Stability,
    tol: f64,
    budget: usize,
) -> Option<FlashResult> {
    let Stability::Unstable { trial_k, .. } = verdict else {
        return None;
    };
    let mut retry = FlashWorkspace::new(z.len());
    for invert in [false, true] {
        for (dst, &k) in retry.ln_k.iter_mut().zip(trial_k) {
            *dst = if invert { -k.ln() } else { k.ln() };
        }
        // SS + GDEM only: the retries start far from the answer, on the
        // unstable side, and are the path §12–§13 validated.
        let mode = Loop {
            seeded_warm: false,
            check_single_phase: false,
            newton: false,
        };
        if let Ok(r) = ss_gdem(spec, cache, t, p, z, &mut retry, mode, tol, budget) {
            if r.two_phase {
                return Some(r);
            }
        }
    }
    None
}

/// Largest `|ln Kᵢ|` at which a converged split is treated as no split.
/// Below it the two "phases" differ by less than one part in a million in
/// every mole fraction: that is the trivial solution `x = y = z`, which
/// satisfies the equilibrium equations at any β, not a pair of phases.
const TRIVIAL_LN_K: f64 = 1e-6;

/// Build the single-phase result with `x = y = z` and β = 0 (liquid) or 1
/// (vapor). `t` in **K**, `p` in **kPa absolute**.
///
/// For a φ-φ system the label comes from the feed's own state
/// ([`crate::mixture::single_phase_label`]): the lower-Gibbs cubic root, or
/// with a single root the side of the EOS critical volume it lies on.
///
/// Until 2026-11-14 the label was the sign of `Σ zᵢ(Kᵢ − 1)` under the last K of
/// the iteration. Near and above the critical locus that K has collapsed to
/// 1 ± 1e-15 and the sign is rounding noise: on a methane/n-pentane grid the
/// label flipped 4 009 times between neighboring pressures. The adiabatic
/// flash reads this label to choose the cubic root for the enthalpy, so the
/// noise was not only cosmetic. That rule survives for the model pairs with
/// no single cubic to ask (γ-φ, Chao–Seader family, mixed EOS), where K
/// comes from two different models and does not collapse.
fn single_phase(spec: &SystemSpec, t: f64, p: f64, z: &[f64], k: &[f64]) -> FlashResult {
    let from_state = match (spec.vapor, spec.liquid) {
        (VaporModel::Cubic(ve), LiquidModel::Cubic(le)) if ve == le => {
            crate::mixture::single_phase_label(&spec.mixture_spec(le), t, p, z).ok()
        }
        _ => None,
    };
    let beta = match from_state {
        Some(PhaseId::Vapor) => 1.0,
        Some(PhaseId::Liquid) => 0.0,
        None => {
            // f(0) = Σ zᵢ(Kᵢ−1). > 0 ⇒ would-be vapor fraction positive but
            // no interior root ⇒ superheated vapor; ≤ 0 ⇒ subcooled liquid.
            let (f0, _, _) = rr_fdd(z, k, 0.0);
            if f0 > 0.0 { 1.0 } else { 0.0 }
        }
    };
    FlashResult {
        beta,
        x: z.to_vec(),
        y: z.to_vec(),
        k: k.to_vec(),
        iterations: 0,
        two_phase: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::ActivityModel;
    use crate::eos::{CubicEos, LiquidModel, VaporModel};
    use crate::flash::k_values;
    use crate::flash::stability::{Seeds, stability_cached};
    use crate::mixing::MixingRule;
    use crate::types::Component;

    /// Rachford-Rice residual f(β) at a returned root.
    fn rr_residual(z: &[f64], k: &[f64], beta: f64) -> f64 {
        rr_fdd(z, k, beta).0
    }

    // -----------------------------------------------------------------
    // Rachford-Rice — analytic + oracle checks.
    // -----------------------------------------------------------------

    #[test]
    fn rr_matches_hand_solution_binary() {
        // z = [0.5, 0.5], K = [2, 0.5]. f(β) = 0.5·1/(1+β) + 0.5·(−0.5)/(1−0.5β).
        // Solve: 0.5/(1+β) = 0.25/(1−0.5β) → 0.5(1−0.5β) = 0.25(1+β)
        // → 0.5 − 0.25β = 0.25 + 0.25β → 0.25 = 0.5β → β = 0.5.
        let beta = rachford_rice(&[0.5, 0.5], &[2.0, 0.5], 1e-12, 100).unwrap();
        assert!((beta - 0.5).abs() < 1e-10, "β={beta}");
    }

    #[test]
    fn rr_residual_is_zero_at_root() {
        let z = [0.3, 0.4, 0.3];
        let k = [3.0, 1.2, 0.4];
        let beta = rachford_rice(&z, &k, 1e-13, 100).unwrap();
        let f = rr_residual(&z, &k, beta);
        assert!(f.abs() < 1e-10, "f(β*)={f}");
        assert!((0.0..=1.0).contains(&beta), "β={beta} should be two-phase");
    }

    #[test]
    fn rr_negative_flash_root_outside_unit_interval() {
        // Feed dominated by the light component with K slightly two-phase:
        // the root can exceed 1 (negative flash) — must still solve.
        let z = [0.98, 0.02];
        let k = [1.05, 0.2];
        let beta = rachford_rice(&z, &k, 1e-12, 100).unwrap();
        let f = rr_residual(&z, &k, beta);
        assert!(f.abs() < 1e-9);
    }

    #[test]
    fn rr_rejects_numerically_invalid_inputs() {
        // Each of these used to travel silently into the iteration and emerge
        // as a NaN residual or an exhausted iteration budget.
        let bad: [(&[f64], &[f64]); 4] = [
            (&[f64::NAN, 0.5], &[2.0, 0.5]), // non-finite z
            (&[-0.1, 1.1], &[2.0, 0.5]),     // negative z
            (&[0.5, 0.5], &[2.0, 0.0]),      // K = 0
            (&[0.5, 0.5], &[2.0, f64::NAN]), // non-finite K
        ];
        for (z, k) in bad {
            assert!(
                matches!(
                    rachford_rice(z, k, 1e-12, 100),
                    Err(FlashError::InvalidInput(_))
                ),
                "expected InvalidInput for z={z:?}, k={k:?}"
            );
        }
        // Empty mixture and a nonsense tolerance.
        assert!(matches!(
            rachford_rice(&[], &[], 1e-12, 100),
            Err(FlashError::InvalidInput(_))
        ));
        assert!(matches!(
            rachford_rice(&[0.5, 0.5], &[2.0, 0.5], 0.0, 100),
            Err(FlashError::InvalidInput(_))
        ));
    }

    #[test]
    fn rr_ignores_absent_components() {
        // A component with zᵢ = 0 contributes exactly nothing to f, but its K
        // used to constrain the pole bracket anyway. Adding one with an
        // extreme K must leave the root untouched.
        let base = rachford_rice(&[0.5, 0.5], &[2.0, 0.5], 1e-13, 100).unwrap();
        let padded = rachford_rice(&[0.5, 0.5, 0.0], &[2.0, 0.5, 1.0e9], 1e-13, 100).unwrap();
        assert!(
            (base - padded).abs() < 1e-12,
            "absent component moved β: {base} vs {padded}"
        );
    }

    #[test]
    fn rr_degenerate_unity_k_reports_no_root() {
        // Every Kᵢ ≈ 1 ⇒ no driving force. The poles sit at ±1e15, so the old
        // window was numerically useless; the degenerate-term filter turns it
        // into the same clean "no bracketable root" the one-sided case gives.
        assert!(matches!(
            rachford_rice(&[0.5, 0.5], &[1.0, 1.0 + 1e-16], 1e-12, 100),
            Err(FlashError::NoRachfordRiceRoot { .. })
        ));
    }

    #[test]
    fn gdem_gain_is_bounded() {
        // μ → 1 is exactly the near-critical stall the unguarded factor turned
        // into an overflow: r ≈ r_prev gives μ ≈ 1 and 1/(1−μ) → ∞.
        let r = [1.0, 1.0];
        let r_prev = [1.0 + 1e-15, 1.0];
        match gdem_gain(&r, &r_prev) {
            // Either refused outright (μ ≥ 0.95) or capped — never unbounded.
            None => {}
            Some(g) => assert!(g <= GDEM_GAIN_MAX, "gain {g} exceeded the cap"),
        }
        // A well-conditioned μ = 0.5 must still accelerate: 1/(1−0.5) = 2.
        let g = gdem_gain(&[0.5, 0.5], &[1.0, 1.0]).unwrap();
        assert!((g - 2.0).abs() < 1e-12, "gain={g}");
        // A diverging residual (μ ≤ 0) must be refused, not extrapolated.
        assert!(gdem_gain(&[-1.0, -1.0], &[1.0, 1.0]).is_none());
    }

    #[test]
    fn gdem_trial_rejects_runaway_candidates() {
        let ln_k = [0.0, 0.0];
        let mut trial = [0.0; 2];
        // A step that stays inside the bound is accepted verbatim.
        assert!(gdem_trial(&ln_k, &[1.0, -1.0], 2.0, &mut trial));
        assert_eq!(trial, [2.0, -2.0]);
        // One that leaves it is refused wholesale (the caller falls back to a
        // plain SS step) rather than committing a half-written iterate.
        assert!(!gdem_trial(&ln_k, &[1000.0, 0.0], 4.0, &mut trial));
    }

    #[test]
    fn rr_rejects_single_phase_k() {
        // All K > 1 ⇒ no interior root.
        assert!(matches!(
            rachford_rice(&[0.5, 0.5], &[2.0, 1.5], 1e-12, 100),
            Err(FlashError::NoRachfordRiceRoot { .. })
        ));
        // All K < 1 ⇒ no interior root.
        assert!(matches!(
            rachford_rice(&[0.5, 0.5], &[0.9, 0.3], 1e-12, 100),
            Err(FlashError::NoRachfordRiceRoot { .. })
        ));
    }

    // -----------------------------------------------------------------
    // Full flash.
    // -----------------------------------------------------------------

    fn n_butane() -> Component {
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            psat_coeffs: vec![4.35, 2277.0, -30.0],
            ..Component::default()
        }
    }

    fn n_heptane() -> Component {
        Component {
            name: "n-heptane".into(),
            tc: 540.2,
            pc: 2740.0,
            omega: 0.350,
            psat_coeffs: vec![4.02, 2911.0, -56.0],
            ..Component::default()
        }
    }

    fn rks_system(components: &[Component]) -> SystemSpec<'_> {
        SystemSpec {
            components,
            vapor: VaporModel::Cubic(CubicEos::RKS1972),
            liquid: LiquidModel::Cubic(CubicEos::RKS1972),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &[],
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        }
    }

    #[test]
    fn flash_two_phase_mass_balance_and_equilibrium() {
        // n-butane/n-heptane, RKS, at a T/P inside the two-phase region
        // (420 K / 1000 kPa → β ≈ 0.56; higher P compresses to a single
        // liquid, so the conditions matter).
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let z = [0.5, 0.5];
        let res = flash_isothermal(&spec, 420.0, 1000.0, &z, 1e-10, 200).unwrap();
        assert!(res.two_phase, "expected a two-phase split");
        assert!((0.0..=1.0).contains(&res.beta));
        // Overall mass balance: β·yᵢ + (1−β)·xᵢ = zᵢ.
        for i in 0..2 {
            let recombined = res.beta * res.y[i] + (1.0 - res.beta) * res.x[i];
            assert!((recombined - z[i]).abs() < 1e-8, "mass balance comp {i}");
        }
        // Compositions sum to 1.
        assert!((res.x.iter().sum::<f64>() - 1.0).abs() < 1e-8);
        assert!((res.y.iter().sum::<f64>() - 1.0).abs() < 1e-8);
        // Equilibrium: Kᵢ = yᵢ/xᵢ.
        for i in 0..2 {
            assert!((res.k[i] - res.y[i] / res.x[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn flash_isofugacity_at_convergence() {
        // At the converged split the K-values must reproduce equal
        // fugacities: recomputing K from (x, y) leaves it unchanged.
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let res = flash_isothermal(&spec, 420.0, 1000.0, &[0.5, 0.5], 1e-11, 200).unwrap();
        let k_check = k_values(&spec, 420.0, 1000.0, &res.x, &res.y).unwrap();
        for i in 0..2 {
            assert!(
                (k_check[i] / res.k[i] - 1.0).abs() < 1e-6,
                "K comp {i} drifted"
            );
        }
    }

    #[test]
    fn flash_single_phase_high_pressure_liquid() {
        // At high pressure the mixture is subcooled liquid — single phase,
        // β = 0.
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let res = flash_isothermal(&spec, 350.0, 20000.0, &[0.5, 0.5], 1e-10, 200).unwrap();
        assert!(!res.two_phase);
        assert_eq!(res.beta, 0.0);
    }

    #[test]
    fn flash_gamma_phi_activity_liquid() {
        // γ-φ path: Wilson liquid + ideal vapor for a non-ideal binary.
        let a = Component {
            name: "a".into(),
            tc: 508.3,
            pc: 4762.0,
            omega: 0.665,
            liquid_volume: 76.8,
            psat_coeffs: vec![5.31, 3100.0, -60.0],
            ..Component::default()
        };
        let b = Component {
            name: "water".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            liquid_volume: 18.07,
            psat_coeffs: vec![5.11, 3800.0, -46.0],
            ..Component::default()
        };
        let comps = [a, b];
        let aij = vec![vec![0.0, 1100.0], vec![-250.0, 0.0]];
        let vl = [76.8, 18.07];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::Wilson),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let z = [0.5, 0.5];
        let res = flash_isothermal(&spec, 350.0, 80.0, &z, 1e-10, 300).unwrap();
        if res.two_phase {
            for i in 0..2 {
                let recombined = res.beta * res.y[i] + (1.0 - res.beta) * res.x[i];
                assert!((recombined - z[i]).abs() < 1e-8);
            }
        }
    }

    // -----------------------------------------------------------------
    // Stability fallback on non-convergence (OPTIMIZATION_NEXT_STEPS Step 0).
    // -----------------------------------------------------------------

    /// Equimolar methane/n-pentane on PR, with the bundled database's
    /// critical constants (copied here so the test needs no `component-db`).
    fn methane_n_pentane_pr() -> Vec<Component> {
        let c = |name: &str, tc, pc, omega| Component {
            name: name.into(),
            tc,
            pc,
            omega,
            ..Component::default()
        };
        vec![
            c("methane", 190.564, 4599.2, 0.01142),
            c("n-pentane", 469.7, 3367.5, 0.251),
        ]
    }

    fn pr_system(components: &[Component]) -> SystemSpec<'_> {
        SystemSpec {
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
            ..rks_system(components)
        }
    }

    /// Every point of the `examples/flash_stability_sweep.rs` 60 × 60 grid at
    /// which this mixture's flash returned `NoConvergence` before the
    /// fallback existed (T in K, P in kPa). All lie along the critical locus.
    const METHANE_PENTANE_FAILURES: [(f64, f64); 22] = [
        (195.7627, 5156.0458),
        (256.7797, 14975.9797),
        (279.6610, 17440.1137),
        (287.2881, 17440.1137),
        (294.9153, 17440.1137),
        (302.5424, 17440.1137),
        (310.1695, 17440.1137),
        (317.7966, 17440.1137),
        (325.4237, 17440.1137),
        (333.0508, 17440.1137),
        (340.6780, 17440.1137),
        (348.3051, 17440.1137),
        (371.1864, 14975.9797),
        (378.8136, 14975.9797),
        (394.0678, 12860.0061),
        (401.6949, 12860.0061),
        (409.3220, 11043.0009),
        (416.9492, 11043.0009),
        (424.5763, 9482.7225),
        (439.8305, 8142.8976),
        (447.4576, 6992.3781),
        (462.7119, 4427.5421),
    ];

    /// The two failure points that are genuinely two-phase: a 200-iteration
    /// TPD trial does not converge there and the public stability verdict
    /// defaults to "stable" — so a fallback that trusted it would return a
    /// wrong single-phase answer.
    const NEAR_CRITICAL_SPLITS: [(f64, f64); 2] = [(409.3220, 11043.0009), (424.5763, 9482.7225)];

    #[test]
    fn non_convergence_points_now_resolve() {
        let comps = methane_n_pentane_pr();
        let spec = pr_system(&comps);
        let z = [0.5, 0.5];
        for &(t, p) in &METHANE_PENTANE_FAILURES {
            let r = flash_isothermal(&spec, t, p, &z, 1e-10, 200)
                .unwrap_or_else(|e| panic!("({t} K, {p} kPa) still fails: {e:?}"));
            let split = NEAR_CRITICAL_SPLITS.contains(&(t, p));
            // Every single-phase answer came from the fallback, never from
            // the main loop. A genuine split may now come from the main loop:
            // the Newton finish (§18) converges (424.58 K, 9482.7 kPa) in 20
            // iterations where SS + GDEM ran out of 200.
            assert!(
                r.iterations >= 200 || split,
                "({t}, {p}) iterations {}",
                r.iterations
            );
            assert_eq!(r.two_phase, split, "phase count at ({t} K, {p} kPa)");
            if split {
                // A genuine split: mass balance and isofugacity hold.
                assert!(r.beta > 0.0 && r.beta < 1.0, "β={}", r.beta);
                for i in 0..2 {
                    let rec = r.beta * r.y[i] + (1.0 - r.beta) * r.x[i];
                    assert!((rec - z[i]).abs() < 1e-8);
                }
                let k_check = k_values(&spec, t, p, &r.x, &r.y).unwrap();
                for i in 0..2 {
                    assert!((k_check[i] / r.k[i] - 1.0).abs() < 1e-6);
                }
            } else {
                // A single phase the (converged) stability test vouches for.
                assert!(r.beta == 0.0 || r.beta == 1.0);
                assert_eq!(r.x, z.to_vec());
                assert_eq!(r.y, z.to_vec());
                let s =
                    crate::flash::stability::stability_analysis(&spec, t, p, &z, 20_000).unwrap();
                assert_eq!(s, Stability::Stable, "({t} K, {p} kPa)");
            }
        }
    }

    #[test]
    fn fallback_does_not_trust_an_unconverged_stability_verdict() {
        // At these two points the stability trials have not converged after
        // 200 iterations. "No split found yet" is not "stable": the fallback
        // must act on the converged verdict (unstable → a split), never on
        // the cut-off one (which, before §13, became a single-phase answer).
        let comps = methane_n_pentane_pr();
        let spec = pr_system(&comps);
        for &(t, p) in &NEAR_CRITICAL_SPLITS {
            let cache = SystemTpCache::new(&spec, t, p).unwrap();
            let v = stability_cached(&spec, &cache, t, p, &[0.5, 0.5], 200, Seeds::WilsonThenPure)
                .unwrap();
            assert!(!v.conclusive || matches!(v.verdict, Stability::Unstable { .. }));
            let long = crate::flash::stability::stability_analysis(&spec, t, p, &[0.5, 0.5], 2000);
            assert!(matches!(long.unwrap(), Stability::Unstable { .. }));
            assert!(
                flash_isothermal(&spec, t, p, &[0.5, 0.5], 1e-10, 200)
                    .unwrap()
                    .two_phase
            );
        }
    }

    #[test]
    fn fallback_keeps_the_error_where_stability_cannot_run() {
        // γ-φ (non-cubic liquid): no TPD test is available, so a
        // non-convergence stays a non-convergence. `max_iter = 1` forces it.
        let comps = [n_butane(), n_heptane()];
        let mut spec = rks_system(&comps);
        spec.liquid = LiquidModel::IdealSolution;
        spec.vapor = VaporModel::IdealGas;
        assert!(matches!(
            flash_isothermal(&spec, 360.0, 300.0, &[0.5, 0.5], 1e-12, 1),
            Err(FlashError::NoConvergence { .. })
        ));
        // A zero feed component: the stability reference ln zᵢ is −∞, so
        // there is no verdict to act on either.
        let three = [n_butane(), n_heptane(), n_butane()];
        let spec3 = pr_system(&three);
        assert!(matches!(
            flash_isothermal(&spec3, 420.0, 1000.0, &[0.5, 0.5, 0.0], 1e-12, 1),
            Err(FlashError::NoConvergence { .. })
        ));
    }

    #[test]
    fn normal_path_results_are_unchanged() {
        // The fallback runs only after `NoConvergence`: a converging flash
        // reports the main loop's iteration count, well under the cap.
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let r = flash_isothermal(&spec, 420.0, 1000.0, &[0.5, 0.5], 1e-10, 200).unwrap();
        assert!(r.two_phase && r.iterations < 200);
        let r = flash_isothermal(&spec, 350.0, 20000.0, &[0.5, 0.5], 1e-10, 200).unwrap();
        assert!(!r.two_phase && r.iterations == 0);
    }

    // -----------------------------------------------------------------
    // Stability check on the single-phase exit (OPTIMIZATION_PLAN_PART1 §13).
    // -----------------------------------------------------------------

    fn water_hexane() -> Vec<Component> {
        let c = |name: &str, tc, pc, omega| Component {
            name: name.into(),
            tc,
            pc,
            omega,
            ..Component::default()
        };
        vec![
            c("water", 647.096, 22064.0, 0.3443),
            c("n-hexane", 507.82, 3044.1, 0.3),
        ]
    }

    fn assert_split(spec: &SystemSpec, t: f64, p: f64, z: &[f64], r: &FlashResult) {
        assert!(
            r.two_phase && r.beta > 0.0 && r.beta < 1.0,
            "({t}, {p}): {r:?}"
        );
        for i in 0..z.len() {
            let rec = r.beta * r.y[i] + (1.0 - r.beta) * r.x[i];
            assert!((rec - z[i]).abs() < 1e-8, "mass balance {i}");
        }
        let k_check = k_values(spec, t, p, &r.x, &r.y).unwrap();
        for i in 0..z.len() {
            assert!((k_check[i] / r.k[i] - 1.0).abs() < 1e-6, "isofugacity {i}");
        }
    }

    #[test]
    fn liquid_liquid_split_behind_a_wilson_single_phase_exit_is_found() {
        // Water/n-hexane, PR, kij 0.5, 300 K / 100 kPa: Wilson K = [0.05,
        // 0.24], so Rachford–Rice has no root at iteration 0 and the flash
        // used to return "liquid" without evaluating a fugacity. The true
        // state is two liquids.
        let comps = water_hexane();
        let kij = vec![vec![0.0, 0.5], vec![0.5, 0.0]];
        let spec = SystemSpec {
            kij: &kij,
            ..pr_system(&comps)
        };
        let z = [0.5, 0.5];
        let r = flash_isothermal(&spec, 300.0, 100.0, &z, 1e-10, 200).unwrap();
        assert_split(&spec, 300.0, 100.0, &z, &r);
        // One phase water-rich, the other hexane-rich.
        let (wx, wy) = (r.x[0], r.y[0]);
        assert!(wx.max(wy) > 0.7 && wx.min(wy) < 0.05, "x_w={wx}, y_w={wy}");
    }

    #[test]
    fn water_condensing_out_of_a_vapor_single_phase_exit_is_found() {
        // Same system at 310.17 K / 11.65 kPa: the flash's Wilson exit said
        // "vapor"; stability (pure-water seed) says water condenses. Needs
        // the retry's second, inverted orientation.
        let comps = water_hexane();
        let kij = vec![vec![0.0, 0.5], vec![0.5, 0.0]];
        let spec = SystemSpec {
            kij: &kij,
            ..pr_system(&comps)
        };
        let (t, p, z) = (
            150.0 + 450.0 * 21.0 / 59.0,
            10.0 * 8000f64.powf(1.0 / 59.0),
            [0.5, 0.5],
        );
        let r = flash_isothermal(&spec, t, p, &z, 1e-10, 200).unwrap();
        assert_split(&spec, t, p, &z, &r);
    }

    #[test]
    fn a_near_pure_incipient_phase_is_resolved() {
        // Water/n-hexane at 150 K / 10 kPa: the incipient phase is almost
        // pure water and the trial K is extreme (≈ [2, 1e-67]). With only
        // the trial's own orientation the retry collapsed back to single
        // phase; the inverted one resolves it.
        let comps = water_hexane();
        let kij = vec![vec![0.0, 0.5], vec![0.5, 0.0]];
        let spec = SystemSpec {
            kij: &kij,
            ..pr_system(&comps)
        };
        let z = [0.5, 0.5];
        let r = flash_isothermal(&spec, 150.0, 10.0, &z, 1e-10, 200).unwrap();
        assert_split(&spec, 150.0, 10.0, &z, &r);
    }

    #[test]
    fn single_phase_check_does_not_touch_stable_or_non_cubic_feeds() {
        // Hydrocarbons: the screen stays quiet, the exit is the old one.
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let r = flash_isothermal(&spec, 350.0, 20000.0, &[0.5, 0.5], 1e-10, 200).unwrap();
        assert!(!r.two_phase && r.iterations == 0 && r.beta == 0.0);
        let r = flash_isothermal(&spec, 600.0, 100.0, &[0.5, 0.5], 1e-10, 200).unwrap();
        assert!(!r.two_phase && r.iterations == 0 && r.beta == 1.0);
    }

    // -----------------------------------------------------------------
    // The Newton finish (OPTIMIZATION_PLAN_PART1 §18).
    // -----------------------------------------------------------------

    /// Flash with (`true`) and without (`false`) the Newton finish.
    fn flash_ab(
        spec: &SystemSpec,
        t: f64,
        p: f64,
        z: &[f64],
        newton: bool,
    ) -> Result<FlashResult, FlashError> {
        flash_impl(spec, t, p, z, None, 1e-10, 400, newton)
    }

    /// Same phase count, and for a split the same β, x and y.
    fn assert_same_answer(a: &FlashResult, b: &FlashResult, what: &str) {
        assert_eq!(a.two_phase, b.two_phase, "{what}: phase count");
        if a.two_phase {
            assert!(
                (a.beta - b.beta).abs() < 1e-7,
                "{what}: β {} vs {}",
                a.beta,
                b.beta
            );
            for i in 0..a.x.len() {
                assert!((a.x[i] - b.x[i]).abs() < 1e-7, "{what}: x[{i}]");
                assert!((a.y[i] - b.y[i]).abs() < 1e-7, "{what}: y[{i}]");
            }
        }
    }

    fn alkanes(n: usize) -> Vec<Component> {
        let c = |name: &str, tc, pc, omega| Component {
            name: name.into(),
            tc,
            pc,
            omega,
            ..Component::default()
        };
        vec![
            c("methane", 190.564, 4599.2, 0.01142),
            c("propane", 369.83, 4248.0, 0.152),
            c("n-pentane", 469.7, 3367.5, 0.251),
            c("n-heptane", 540.2, 2740.0, 0.350),
            c("ethane", 305.32, 4872.0, 0.0995),
            c("n-butane", 425.12, 3796.0, 0.200),
            c("n-hexane", 507.6, 3025.0, 0.301),
            c("n-octane", 568.7, 2490.0, 0.399),
            c("n-nonane", 594.6, 2290.0, 0.445),
            c("n-decane", 617.7, 2110.0, 0.490),
        ]
        .into_iter()
        .take(n)
        .collect()
    }

    #[test]
    fn newton_finish_gives_the_ss_answer() {
        for n in [2usize, 4, 10] {
            let comps = alkanes(n);
            let z = vec![1.0 / n as f64; n];
            for spec in [pr_system(&comps), rks_system(&comps)] {
                for i in 0..12 {
                    let t = 180.0 + 30.0 * i as f64;
                    for p in [100.0, 1000.0, 4000.0, 9000.0] {
                        let (Ok(a), Ok(b)) = (
                            flash_ab(&spec, t, p, &z, true),
                            flash_ab(&spec, t, p, &z, false),
                        ) else {
                            panic!("flash failed at n={n} T={t} P={p}");
                        };
                        assert_same_answer(&a, &b, &format!("n={n} T={t} P={p}"));
                    }
                }
            }
        }
    }

    /// Close to the critical locus SS contracts slowly; the Newton finish
    /// must reach the same split in fewer outer iterations.
    #[test]
    fn newton_finish_cuts_iterations_near_the_critical_locus() {
        let comps = methane_n_pentane_pr();
        let spec = pr_system(&comps);
        let z = [0.5, 0.5];
        let (mut with, mut without, mut splits) = (0, 0, 0);
        for i in 0..20 {
            let t = 300.0 + 4.0 * i as f64;
            for p in [12000.0, 14000.0, 15500.0] {
                let (Ok(a), Ok(b)) = (
                    flash_ab(&spec, t, p, &z, true),
                    flash_ab(&spec, t, p, &z, false),
                ) else {
                    continue;
                };
                assert_same_answer(&a, &b, &format!("T={t} P={p}"));
                if a.two_phase && b.iterations < 400 {
                    splits += 1;
                    with += a.iterations;
                    without += b.iterations;
                }
            }
        }
        assert!(
            splits >= 10,
            "only {splits} near-critical splits on the grid"
        );
        assert!(
            2 * with < without,
            "Newton finish {with} iterations vs SS + GDEM {without} over {splits} splits"
        );
    }

    /// One Newton step from a slightly perturbed solution squares the error.
    #[test]
    fn newton_step_converges_quadratically() {
        let comps = alkanes(4);
        let spec = pr_system(&comps);
        let z = [0.25; 4];
        let (t, p) = (300.0, 2000.0);
        let sol = flash_ab(&spec, t, p, &z, false).unwrap();
        assert!(sol.two_phase);
        // Perturb ln K, split at the perturbed K, take one step.
        let k: Vec<f64> = sol
            .k
            .iter()
            .enumerate()
            .map(|(i, k)| k * (1.0 + 1e-3 * (i as f64 - 1.5)))
            .collect();
        let beta = rachford_rice(&z, &k, 1e-13, 200).unwrap();
        let (mut x, mut y) = (vec![0.0; 4], vec![0.0; 4]);
        split_into(&z, &k, beta, &mut x, &mut y);
        let model = k_values(&spec, t, p, &x, &y).unwrap();
        let r: Vec<f64> = model.iter().zip(&k).map(|(m, k)| (m / k).ln()).collect();
        let before = r.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        let mut ln_k = vec![0.0; 4];
        assert!(newton_step(&spec, t, p, &z, beta, &x, &y, &r, &mut ln_k));
        let after = ln_k
            .iter()
            .zip(&sol.k)
            .fold(0.0_f64, |m, (a, b)| m.max((a - b.ln()).abs()));
        assert!(before > 1e-4, "perturbation too small: {before}");
        assert!(
            after < 10.0 * before * before,
            "before {before}, after {after}"
        );
    }

    /// γ-φ flash (activity liquid, ideal-gas vapor): the Newton finish uses
    /// ∂ln γ/∂n and must give the SS answer.
    #[test]
    fn newton_finish_gives_the_ss_answer_gamma_phi() {
        let mk = |name: &str, tc, pc, omega, vl, ps: [f64; 3]| Component {
            name: name.into(),
            tc,
            pc,
            omega,
            liquid_volume: vl,
            psat_coeffs: ps.to_vec(),
            ..Component::default()
        };
        let comps = [
            mk("a", 508.3, 4762.0, 0.665, 76.8, [5.31, 3100.0, -60.0]),
            mk("water", 647.1, 22064.0, 0.344, 18.07, [5.11, 3800.0, -46.0]),
        ];
        let aij = vec![vec![0.0, 1100.0], vec![-250.0, 0.0]];
        let vl = [76.8, 18.07];
        let spec = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::Wilson),
            aij: &aij,
            vl: &vl,
            ..rks_system(&comps)
        };
        let mut splits = 0;
        for z1 in [0.2, 0.5, 0.8] {
            let z = [z1, 1.0 - z1];
            // Bracket the two-phase region with the saturation pressures.
            let pb = crate::flash::bubble::bubble_pressure(&spec, 350.0, &z, 1e-10, 200)
                .unwrap()
                .value;
            let pd = crate::flash::dew::dew_pressure(&spec, 350.0, &z, 1e-10, 200)
                .unwrap()
                .value;
            for i in 1..10 {
                let p = pd + (pb - pd) * i as f64 / 10.0;
                let a = flash_ab(&spec, 350.0, p, &z, true).unwrap();
                let b = flash_ab(&spec, 350.0, p, &z, false).unwrap();
                assert_same_answer(&a, &b, &format!("P={p} z1={z1}"));
                // Between the dew and bubble pressures the feed must split.
                // Wilson's K calls every one of these points vapor; before
                // the model re-seed (§19) the flash returned that.
                assert!(a.two_phase, "single phase at P={p}, z1={z1}");
                splits += usize::from(a.two_phase);
            }
        }
        assert!(splits > 0, "no two-phase point on the grid");
    }

    /// One of the 31 water/hydrocarbon points the pure-component screen left
    /// (`OPTIMIZATION_PLAN_PART1.md` §13, §20): water/n-decane 30/70 on PR,
    /// kij = 0.5. The incipient phase is ≈ 80 % water, the smallest pure tm
    /// is +0.11 — the screen is quiet — and the margin sends it to the full
    /// stability test, which finds the split.
    #[test]
    fn a_mixed_incipient_phase_behind_a_quiet_screen_is_found() {
        let c = |name: &str, tc, pc, omega| Component {
            name: name.into(),
            tc,
            pc,
            omega,
            ..Component::default()
        };
        let comps = [
            c("water", 647.096, 22064.0, 0.3443),
            c("n-decane", 617.7, 2103.0, 0.4884),
        ];
        let kij = vec![vec![0.0, 0.5], vec![0.5, 0.0]];
        let spec = SystemSpec {
            kij: &kij,
            ..pr_system(&comps)
        };
        let z = [0.3, 0.7];
        let (t, p) = (546.61, 5156.0);
        // The screen alone does not prove instability here.
        let cache = SystemTpCache::new(&spec, t, p).unwrap();
        let mut ln_phi = [0.0; 2];
        min_gibbs_ln_phi_cached_into(&spec, &cache, &z, &mut ln_phi).unwrap();
        let min_tm = pure_component_screen(&spec, &cache, t, p, &z, &ln_phi).unwrap();
        assert!(min_tm > 0.0 && min_tm < SCREEN_MARGIN, "min tm {min_tm}");
        let r = flash_isothermal(&spec, t, p, &z, 1e-10, 200).unwrap();
        assert_split(&spec, t, p, &z, &r);
    }

    /// A γ-φ feed between its dew and bubble pressures whose Wilson K-values
    /// are all above 1: the cold flash used to return "vapor" without asking
    /// the model. With the SS-only loop too — this is the model re-seed
    /// (§19), not the Newton finish.
    #[test]
    fn gamma_phi_wilson_single_phase_verdict_is_checked_against_the_model() {
        let mk = |name: &str, tc, pc, omega, vl, ps: [f64; 3]| Component {
            name: name.into(),
            tc,
            pc,
            omega,
            liquid_volume: vl,
            psat_coeffs: ps.to_vec(),
            ..Component::default()
        };
        let comps = [
            mk("a", 508.3, 4762.0, 0.665, 76.8, [5.31, 3100.0, -60.0]),
            mk("water", 647.1, 22064.0, 0.344, 18.07, [5.11, 3800.0, -46.0]),
        ];
        let aij = vec![vec![0.0, 1100.0], vec![-250.0, 0.0]];
        let vl = [76.8, 18.07];
        let spec = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::Wilson),
            aij: &aij,
            vl: &vl,
            ..rks_system(&comps)
        };
        let z = [0.5, 0.5];
        let pb = crate::flash::bubble::bubble_pressure(&spec, 350.0, &z, 1e-10, 200)
            .unwrap()
            .value;
        let pd = crate::flash::dew::dew_pressure(&spec, 350.0, &z, 1e-10, 200)
            .unwrap()
            .value;
        let p = 0.5 * (pb + pd);
        // Wilson calls it vapor: every K above 1.
        let kw = crate::flash::init::wilson_k_values(&comps, 350.0, p);
        assert!(kw.iter().all(|&k| k > 1.0), "Wilson K {kw:?}");
        for newton in [false, true] {
            let r = flash_ab(&spec, 350.0, p, &z, newton).unwrap();
            assert!(r.two_phase, "newton={newton}: single phase at P={p}");
            assert!(r.beta > 0.0 && r.beta < 1.0);
            let k = k_values(&spec, 350.0, p, &r.x, &r.y).unwrap();
            for i in 0..2 {
                assert!((k[i] / r.k[i] - 1.0).abs() < 1e-7);
            }
        }
    }

    /// Evidence harness for `OPTIMIZATION_PLAN_PART1.md` §18 — not a pass/fail
    /// test. Run with
    /// `cargo test --release -p vle-thermo --lib harness_newton_finish -- --ignored --nocapture`.
    #[test]
    #[ignore = "measurement harness; prints a table"]
    fn harness_newton_finish() {
        use std::time::Instant;
        let c2 = methane_n_pentane_pr();
        let c4 = alkanes(4);
        let c10 = alkanes(10);
        let me = alkanes(5)
            .into_iter()
            .filter(|c| c.name == "methane" || c.name == "ethane")
            .collect::<Vec<_>>();
        let cases: Vec<(&str, SystemSpec, usize)> = vec![
            ("methane/ethane", pr_system(&me), 2),
            ("methane/n-pentane", pr_system(&c2), 2),
            ("C1/C3/nC5/nC7", pr_system(&c4), 4),
            ("10 alkanes", pr_system(&c10), 10),
        ];
        println!(
            "{:<20} {:>6} {:>6} {:>7} {:>9} {:>9} {:>9} {:>9} {:>6} {:>6}",
            "mixture",
            "two",
            "differ",
            "slow",
            "it new",
            "it old",
            "µs new",
            "µs old",
            "f new",
            "f old"
        );
        for (label, spec, n) in &cases {
            let z = vec![1.0 / *n as f64; *n];
            let (mut two, mut differ, mut slow) = (0, 0, 0);
            let (mut it_new, mut it_old, mut f_new, mut f_old) = (0usize, 0usize, 0, 0);
            let (mut us_new, mut us_old) = (0.0, 0.0);
            for i in 0..60 {
                let t = 150.0 + 450.0 * i as f64 / 59.0;
                for j in 0..60 {
                    let p = 10.0 * (8000.0_f64).powf(j as f64 / 59.0);
                    let reps = 10;
                    let start = Instant::now();
                    let mut a = flash_impl(spec, t, p, &z, None, 1e-10, 200, true);
                    for _ in 1..reps {
                        a = flash_impl(spec, t, p, &z, None, 1e-10, 200, true);
                    }
                    let ta = start.elapsed().as_secs_f64() * 1e6 / reps as f64;
                    let start = Instant::now();
                    let mut b = flash_impl(spec, t, p, &z, None, 1e-10, 200, false);
                    for _ in 1..reps {
                        b = flash_impl(spec, t, p, &z, None, 1e-10, 200, false);
                    }
                    let tb = start.elapsed().as_secs_f64() * 1e6 / reps as f64;
                    f_new += usize::from(a.is_err());
                    f_old += usize::from(b.is_err());
                    let (Ok(a), Ok(b)) = (a, b) else { continue };
                    let same = a.two_phase == b.two_phase
                        && (!a.two_phase
                            || ((a.beta - b.beta).abs() < 1e-6
                                && a.x.iter().zip(&b.x).all(|(u, v)| (u - v).abs() < 1e-6)));
                    if !same {
                        differ += 1;
                        println!(
                            "    differ {label} T={t:.2} P={p:.1}: new two={} β={} / old two={} β={}",
                            a.two_phase, a.beta, b.two_phase, b.beta
                        );
                    }
                    if a.two_phase && b.two_phase {
                        two += 1;
                        it_new += a.iterations;
                        it_old += b.iterations;
                        us_new += ta;
                        us_old += tb;
                        slow += usize::from(b.iterations > 30);
                    }
                }
            }
            println!(
                "{:<20} {:>6} {:>6} {:>7} {:>9} {:>9} {:>9.0} {:>9.0} {:>6} {:>6}",
                label, two, differ, slow, it_new, it_old, us_new, us_old, f_new, f_old
            );
        }
    }

    /// A converged split must have two different phases. Before 2026-11-14 these
    /// points came back `two_phase` with every K within 1e-9 of 1 — the
    /// trivial solution with an arbitrary β.
    #[test]
    fn a_converged_split_never_has_identical_phases() {
        let comps = methane_n_pentane_pr();
        let spec = pr_system(&comps);
        let known = [
            (0.5, 434.0, 11200.0),
            (0.5, 474.0, 15000.0),
            (0.5, 476.0, 15500.0),
            (0.5, 476.0, 16500.0),
            (0.7, 466.0, 17800.0),
            (0.7, 442.0, 12064.0),
        ];
        for (z1, t, p) in known {
            let r = flash_isothermal(&spec, t, p, &[z1, 1.0 - z1], 1e-10, 200).unwrap();
            assert!(!r.two_phase, "z1={z1} T={t} P={p}: K={:?}", r.k);
        }
        // And nowhere on a grid across the critical region.
        for i in 0..40 {
            let t = 400.0 + 2.0 * i as f64;
            for j in 0..40 {
                let p = 9000.0 + 250.0 * j as f64;
                if let Ok(r) = flash_isothermal(&spec, t, p, &[0.5, 0.5], 1e-10, 200) {
                    let spread = r.k.iter().map(|k| k.ln().abs()).fold(0.0, f64::max);
                    assert!(
                        !r.two_phase || spread >= TRIVIAL_LN_K,
                        "T={t} P={p}: two phases with K={:?}",
                        r.k
                    );
                }
            }
        }
    }

    /// The single-phase label of a φ-φ flash comes from the feed's density,
    /// so along an isotherm it changes at most once on each side of the
    /// two-phase region. Under the old rule (sign of Σ z(K−1) with K ≈ 1) it
    /// flipped 4 009 times on a grid like this one.
    #[test]
    fn single_phase_label_does_not_speckle() {
        let comps = methane_n_pentane_pr();
        let spec = pr_system(&comps);
        let mut flips = 0;
        let mut rows = 0;
        for i in 0..26 {
            let t = 250.0 + 10.0 * i as f64;
            rows += 1;
            let mut prev: Option<f64> = None;
            for j in 0..196 {
                let p = 500.0 + 100.0 * j as f64;
                match flash_isothermal(&spec, t, p, &[0.7, 0.3], 1e-10, 200) {
                    Ok(r) if !r.two_phase => {
                        if prev.is_some_and(|b| b != r.beta) {
                            flips += 1;
                        }
                        prev = Some(r.beta);
                    }
                    // A two-phase stretch (or a failure) legitimately
                    // separates a vapor label from a liquid one.
                    _ => prev = None,
                }
            }
        }
        assert!(flips <= rows, "{flips} label flips over {rows} isotherms");
        // Unambiguous states keep the labels they always had.
        let cold = flash_isothermal(&spec, 250.0, 19000.0, &[0.7, 0.3], 1e-10, 200).unwrap();
        let hot = flash_isothermal(&spec, 500.0, 500.0, &[0.7, 0.3], 1e-10, 200).unwrap();
        assert!(!cold.two_phase && cold.beta == 0.0);
        assert!(!hot.two_phase && hot.beta == 1.0);
    }
}
