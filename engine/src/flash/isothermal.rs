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
//!
//! The full Michelsen scheme finishes with a Newton step on `ln K` once the
//! residual is small; GDEM-accelerated SS already converges the Chapter IV
//! cases, and the Newton finish is layered on top (`flash_isothermal` uses
//! SS+GDEM; the analytic-Jacobian Newton polish is a follow-on refinement
//! tracked in the milestone).
//!
//! # References
//! - (19) Michelsen (1982) Part II — phase-split framework
//! - (23) Leibovici & Neoschil (1992) — the Rachford-Rice window
//! - (25) Crowe & Nishio (1975) — GDEM acceleration

// Parallel-array flash math (zᵢ, Kᵢ, xᵢ, yᵢ, ln-K residuals indexed in
// lockstep) — index loops mirror the equations; allow the range-loop lint.
#![allow(clippy::needless_range_loop)]

use smallvec::SmallVec;

use super::FlashError;
use super::init::wilson_ln_k;
use super::system::{SystemSpec, SystemTpCache, ln_k_values_cached_into};

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
    /// two-phase split; the driver clamps single-phase feeds to 0 or 1.
    pub beta: f64,
    /// Liquid mole fractions xᵢ (length N, sum to 1).
    pub x: Vec<f64>,
    /// Vapor mole fractions yᵢ (length N, sum to 1).
    pub y: Vec<f64>,
    /// Converged equilibrium ratios Kᵢ = yᵢ/xᵢ.
    pub k: Vec<f64>,
    /// Number of outer (SS/GDEM) iterations taken.
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
/// # Errors
/// [`FlashError`] on dimension mismatch, thermodynamic failure, or
/// non-convergence.
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
pub fn flash_isothermal_warm(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    k_init: Option<&[f64]>,
    tol: f64,
    max_iter: usize,
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
            Err(FlashError::NoRachfordRiceRoot { .. }) => {
                return Ok(single_phase(z, &ws.k));
            }
            Err(e) => return Err(e),
        };
        split_into(z, &ws.k, beta, &mut ws.x, &mut ws.y);

        // Recompute K from the fugacity models — in log form, so the residual
        // below is a plain subtraction instead of a division and a logarithm.
        ln_k_values_cached_into(spec, &cache, &ws.x, &ws.y, &mut ws.ln_k_new)?;

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
            if !(0.0..=1.0).contains(&beta) {
                return Ok(single_phase(z, &ws.k));
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

        // GDEM acceleration on ln K every few iterations.
        let mut accelerated = false;
        if have_r_prev && iter % 5 == 4 {
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
        }
        ws.r_prev.copy_from_slice(&ws.r);
        have_r_prev = true;

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

/// Build the single-phase result: decide liquid vs vapor from the sign of
/// the Rachford-Rice residual at β = 0, and set `x = y = z`.
fn single_phase(z: &[f64], k: &[f64]) -> FlashResult {
    let (f0, _, _) = rr_fdd(z, k, 0.0);
    // f(0) = Σ zᵢ(Kᵢ−1). > 0 ⇒ would-be vapor fraction positive but no
    // interior root ⇒ superheated vapor (β=1); ≤ 0 ⇒ subcooled liquid (β=0).
    let beta = if f0 > 0.0 { 1.0 } else { 0.0 };
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
}
