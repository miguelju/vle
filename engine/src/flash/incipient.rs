//! Shared incipient-phase machinery for bubble- and dew-point calculations
//! (Milestone 9, §K).
//!
//! Bubble and dew points are "one phase is present, an infinitesimal amount
//! of the other is just appearing" problems. Both reduce to the same inner
//! loop: at a trial `(T, P)`, converge the incipient phase's composition by
//! successive substitution on the K-values, then read off the sum that must
//! equal 1 at the true saturation point:
//!
//! - **Bubble point** — liquid `x` is known; the incipient vapor is
//!   `yᵢ ∝ Kᵢ·xᵢ`, and the saturation condition is `S = Σ Kᵢ·xᵢ = 1`.
//! - **Dew point** — vapor `y` is known; the incipient liquid is
//!   `xᵢ ∝ yᵢ/Kᵢ`, and the condition is `S = Σ yᵢ/Kᵢ = 1`.
//!
//! The outer driver (in [`super::bubble`] / [`super::dew`]) adjusts the free
//! variable (T or P) to drive `S → 1`. Two drivers, one per free variable:
//!
//! - **Pressure** ([`solve_pressure`]) — the same Newton loop as the
//!   temperature solve with P free ([`newton_pressure`]): a step on `ln P`
//!   with the exact `∂ln Kᵢ/∂P`, and the simultaneous `(N+1)`-variable step
//!   on `{ln K, ln P}` near the root (`OPTIMIZATION_PLAN_PART1.md` §17). If
//!   Newton declines, the multiplicative update `P ← P·S` (bubble) /
//!   `P ← P/S` (dew) around an inner successive-substitution loop on K —
//!   Newton on `ln P` with the Raoult-law slope `∂ln S/∂ln P ≈ −1`, needing
//!   no derivative — with its trivial-solution recovery.
//! - **Temperature** ([`solve_temperature`]) — Newton with the exact
//!   `∂ln Kᵢ/∂T` (see [`newton_temperature`]). Far from the root, a Newton
//!   step on `1/T` plus one successive-substitution step on the incipient
//!   composition. Close to it — and where SS is slow — the fully simultaneous
//!   `(N+1)`-variable Newton on `{ln K, ln T}` of `MODERNIZATION_PLAN.md` §K,
//!   using the incipient phase's exact `∂ln φ̂ᵢ/∂nⱼ`, which converges
//!   composition quadratically (`OPTIMIZATION_PLAN_PART1.md` §14). If Newton
//!   fails — an unsupported model pair, a non-finite step, no convergence, or
//!   a collapse onto the trivial solution `K ≡ 1` — it falls back to
//!   bisection on the pressure solver ([`bisect_temperature`]), which is ~250×
//!   slower but converges wherever a saturation pressure exists.

use super::FlashError;
use super::init::wilson_k_values;
use super::system::{
    SystemSpec, k_values, k_values_with_p_derivs, k_values_with_t_derivs, phase_ln_phi_jacobian,
};
use crate::eos::{LiquidModel, PhaseId, VaporModel};
use crate::mixture::z_mix;

/// Which saturation point — selects the known phase and the sum condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Point {
    /// Liquid known; incipient vapor; `S = Σ Kᵢ·xᵢ`.
    Bubble,
    /// Vapor known; incipient liquid; `S = Σ yᵢ/Kᵢ`.
    Dew,
}

/// Converge the incipient-phase composition at fixed `(t, p)` by successive
/// substitution on K, returning `(S, incipient_composition, converged_K)`.
///
/// `known` is the present phase (liquid `x` for a bubble point, vapor `y`
/// for a dew point). `k` is the starting K estimate (updated in place across
/// outer iterations by the caller for warm starts).
pub(crate) fn incipient_sum(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    known: &[f64],
    point: Point,
    k: &mut Vec<f64>,
    inner_max: usize,
) -> Result<(f64, Vec<f64>), FlashError> {
    let n = spec.n();
    let mut incipient = vec![0.0; n];
    let mut s = 0.0;
    for _ in 0..inner_max.max(1) {
        // Build + normalize the incipient composition from the current K.
        s = 0.0;
        for i in 0..n {
            let un = match point {
                Point::Bubble => k[i] * known[i], // yᵢ ∝ Kᵢ·xᵢ
                Point::Dew => known[i] / k[i],    // xᵢ ∝ yᵢ/Kᵢ
            };
            incipient[i] = un;
            s += un;
        }
        for v in incipient.iter_mut() {
            *v /= s;
        }
        // Recompute K with the present phase and the fresh incipient phase.
        let (x, y) = match point {
            Point::Bubble => (known, incipient.as_slice()),
            Point::Dew => (incipient.as_slice(), known),
        };
        let k_new = k_values(spec, t, p, x, y)?;
        let mut max_step = 0.0_f64;
        for i in 0..n {
            max_step = max_step.max((k_new[i] / k[i]).ln().abs());
        }
        *k = k_new;
        if max_step < 1e-10 {
            break;
        }
    }
    Ok((s, incipient))
}

/// Wilson-based initial pressure for a bubble/dew point at temperature `t`.
///
/// From the Wilson K `Kᵢ = (Pc,ᵢ/P)·exp[5.373(1+ωᵢ)(1−Tc,ᵢ/T)]` and the
/// saturation condition:
/// - **Bubble** (`Σ Kᵢxᵢ = 1`): `P₀ = Σ xᵢ·Pc,ᵢ·exp[…]`.
/// - **Dew** (`Σ yᵢ/Kᵢ = 1`): `P₀ = 1 / Σ (yᵢ/Pc,ᵢ)·exp[−…]`.
pub(crate) fn wilson_pressure_guess(spec: &SystemSpec, t: f64, known: &[f64], point: Point) -> f64 {
    let mut acc = 0.0;
    for (i, c) in spec.components.iter().enumerate() {
        let e = (5.373 * (1.0 + c.omega) * (1.0 - c.tc / t)).exp();
        match point {
            Point::Bubble => acc += known[i] * c.pc * e,
            Point::Dew => acc += known[i] / (c.pc * e),
        }
    }
    match point {
        Point::Bubble => acc,
        Point::Dew => 1.0 / acc,
    }
}

/// Initial K-values at `(t, p)` for the incipient loop (Wilson).
pub(crate) fn wilson_k_init(spec: &SystemSpec, t: f64, p: f64) -> Vec<f64> {
    wilson_k_values(spec.components, t, p)
}

/// Result of a saturation-point solve: the found variable (T or P), the
/// incipient-phase composition, and the converged K-values.
pub(crate) struct SatPoint {
    pub var: f64,
    pub incipient: Vec<f64>,
    pub k: Vec<f64>,
}

/// Solve for the saturation **pressure** at fixed `t` (bubble-P or dew-P):
/// `t` in **K**, the returned `var` in **kPa absolute**.
///
/// Tries [`newton_pressure`] first and falls back to
/// [`solve_pressure_multiplicative`] whenever Newton declines. Both solve the
/// same equation, so the fallback changes the cost, never the answer. The
/// rest of this comment describes the fallback.
///
/// Multiplicative outer update — `P ← P·S` (bubble) or `P ← P/S` (dew) —
/// which drives `S → 1` monotonically, warm-starting the K-values across
/// outer steps. Wilson seeds both P and K.
///
/// **Trivial-solution guard.** Near the mixture critical point the iteration
/// can walk into the region where the cubic has a single real root for the
/// known phase; both phases then take that root, `K ≡ 1` exactly, and
/// `S = 1` is satisfied at a meaningless pressure. Until v0.17.1 that was
/// returned as `Ok` — e.g. equimolar methane/ethane (PR) at 221.7 K gave a
/// bubble pressure of 5198 kPa against the true 4100 kPa
/// (`OPTIMIZATION_PLAN_PART1.md` §10). A converged K ≡ 1 is now accepted only
/// if the known phase still has distinct liquid and vapor roots (a pure
/// component or an exact azeotrope, where K = 1 is the right answer);
/// otherwise the solve restarts from a lower pressure, and if every restart
/// lands on the trivial solution too it falls back to [`inverse_pressure`]
/// — the saturation temperature solved by Newton, inverted by a secant on
/// `ln P`. [`FlashError::NoConvergence`] only if that fails as well.
pub(crate) fn solve_pressure(
    spec: &SystemSpec,
    t: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    // Newton on ln P first (§17); the multiplicative solver below is the
    // fallback wherever Newton cannot vouch for an answer.
    match newton_pressure(spec, t, known, point, tol, max_iter) {
        Some(sp) => Ok(sp),
        None => solve_pressure_multiplicative(spec, t, known, point, tol, max_iter),
    }
}

/// The pre-Newton saturation-pressure solver: `P ← P·S` with trivial-solution
/// restarts and the [`inverse_pressure`] last resort — see [`solve_pressure`],
/// which now runs it only when [`newton_pressure`] declines.
pub(crate) fn solve_pressure_multiplicative(
    spec: &SystemSpec,
    t: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    let mut p0 = wilson_pressure_guess(spec, t, known, point).max(1e-6);
    let p_seed = p0;
    // Restart budget: each restart halves the starting pressure.
    const RESTARTS: usize = 3;
    for _ in 0..=RESTARTS {
        let sp = multiplicative_pressure(spec, t, p0, known, point, tol, max_iter)?;
        if !is_trivial(spec, t, sp.var, known, &sp.incipient, point)? {
            return Ok(sp);
        }
        p0 = 0.5 * p0.min(sp.var);
    }
    inverse_pressure(spec, t, p_seed, known, point, tol, max_iter).ok_or(
        FlashError::NoConvergence {
            what: "saturation pressure (only the trivial solution K = 1 was found)",
            iters: max_iter,
            residual: 0.0,
        },
    )
}

/// Saturation pressure at `t` (**K**) by inverting the saturation
/// *temperature*: solve `f(P) = ln(T_sat(P) / t) = 0` by a safeguarded
/// secant on `ln P`, each `T_sat` from [`newton_temperature`].
///
/// The fallback for the near-critical region where `P ← P·S` collapses onto
/// the trivial solution: Newton on `1/T` carries the exact `∂ln K/∂T` and
/// stays on the physical branch there (CO₂/n-butane at 310 K, x_CO₂ = 0.8:
/// the multiplicative update finds only `K ≡ 1`, this finds 6.2 MPa).
/// `T_sat` rises with `P` along a saturation branch, so the secant is well
/// posed; a pressure where Newton declines (past the end of the branch) is
/// treated as "too far" and the step is halved back toward the last good
/// point. Calls `newton_temperature`, never `solve_temperature`, whose
/// bisection fallback would recurse into [`solve_pressure`]. Returns `None`
/// if it cannot vouch for an answer. `p_seed` in **kPa**.
fn inverse_pressure(
    spec: &SystemSpec,
    t: f64,
    p_seed: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Option<SatPoint> {
    let eval = |ln_p: f64| -> Option<(f64, SatPoint)> {
        let sp = newton_temperature(spec, ln_p.exp(), known, point, tol, max_iter)?;
        Some(((sp.var / t).ln(), sp))
    };
    // A first good point: walk down from the seed until Newton answers.
    let mut ln_p = p_seed.ln();
    let mut a = None;
    for _ in 0..12 {
        if let Some(r) = eval(ln_p) {
            a = Some((ln_p, r));
            break;
        }
        ln_p -= 0.2;
    }
    let (mut x0, (mut f0, sp0)) = a?;
    if f0.abs() < tol {
        return Some(SatPoint {
            var: x0.exp(),
            ..sp0
        });
    }
    // Second point: Clausius–Clapeyron-sized step in the right direction
    // (T_sat too high ⇒ lower P).
    let mut x1 = x0 - 0.05 * f0.signum();
    let (mut f1, mut sp1) = loop {
        match eval(x1) {
            Some(r) => break r,
            None => {
                x1 = 0.5 * (x0 + x1);
                if (x1 - x0).abs() < 1e-6 {
                    return None;
                }
            }
        }
    };
    for _ in 0..max_iter {
        if f1.abs() < tol {
            return Some(SatPoint {
                var: x1.exp(),
                ..sp1
            });
        }
        let slope = (f1 - f0) / (x1 - x0);
        if !slope.is_finite() || slope <= 0.0 {
            return None;
        }
        let mut x2 = x1 - (f1 / slope).clamp(-0.3, 0.3);
        let (f2, sp2) = loop {
            match eval(x2) {
                Some(r) => break r,
                None => {
                    x2 = 0.5 * (x1 + x2);
                    if (x2 - x1).abs() < 1e-9 {
                        return None;
                    }
                }
            }
        };
        (x0, f0) = (x1, f1);
        (x1, f1, sp1) = (x2, f2, sp2);
    }
    None
}

/// The `P ← P·S` / `P ← P/S` iteration from a given starting pressure `p0`
/// (**kPa**), with Wilson K at `(t, p0)`.
fn multiplicative_pressure(
    spec: &SystemSpec,
    t: f64,
    p0: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    multiplicative_pressure_k(
        spec,
        t,
        p0,
        wilson_k_init(spec, t, p0),
        known,
        point,
        tol,
        max_iter,
    )
}

/// [`multiplicative_pressure`] from given starting K-values.
#[allow(clippy::too_many_arguments)]
fn multiplicative_pressure_k(
    spec: &SystemSpec,
    t: f64,
    p0: f64,
    mut k: Vec<f64>,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    let mut p = p0;
    for iter in 0..max_iter {
        let (s, incipient) = incipient_sum(spec, t, p, known, point, &mut k, 40)?;
        if (s - 1.0).abs() < tol {
            return Ok(SatPoint {
                var: p,
                incipient,
                k,
            });
        }
        p = match point {
            Point::Bubble => p * s,
            Point::Dew => p / s,
        }
        .max(1e-9);
        if iter + 1 == max_iter {
            return Err(FlashError::NoConvergence {
                what: "saturation pressure",
                iters: max_iter,
                residual: (s - 1.0).abs(),
            });
        }
    }
    unreachable!("loop returns via convergence or NoConvergence")
}

/// Is a converged saturation point the **trivial solution** — both phases on
/// the same cubic root, so `K ≡ 1` and `S = 1` hold at any pressure?
///
/// Neither `K ≈ 1` nor `S = 1` decides it on its own. A pure component at its
/// vapor pressure and an exact azeotrope have `K = 1` legitimately; and the
/// successive-substitution loop *approaches* the trivial solution slowly, so
/// a not-yet-trivial `K = 1.0002` already satisfies `|S − 1| < tol`. The
/// physical test is density: the known phase on its root (liquid for a bubble
/// point) and the incipient phase on its root (vapor) must differ. At the
/// trivial solution their Z factors coincide; at a genuine saturation point
/// — even 0.1 in `ln K` from the critical point — they do not.
///
/// Applies to φ-φ systems with the same cubic in both phases; γ-φ systems
/// (and virial vapors) cannot collapse this way and are never trivial.
/// `t` in **K**, `p` in **kPa**.
fn is_trivial(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    known: &[f64],
    incipient: &[f64],
    point: Point,
) -> Result<bool, FlashError> {
    let (VaporModel::Cubic(ev), LiquidModel::Cubic(el)) = (spec.vapor, spec.liquid) else {
        return Ok(false);
    };
    if ev != el {
        return Ok(false);
    }
    let (x, y) = match point {
        Point::Bubble => (known, incipient),
        Point::Dew => (incipient, known),
    };
    let ms = spec.mixture_spec(el);
    let thermo = |e: crate::mixture::MixError| FlashError::Thermo(e.to_string());
    let zl = z_mix(&ms, t, p, x, PhaseId::Liquid).map_err(thermo)?;
    let zv = z_mix(&ms, t, p, y, PhaseId::Vapor).map_err(thermo)?;
    Ok((zv - zl).abs() < TRIVIAL_Z * zv.abs().max(zl.abs()))
}

/// Relative Z-factor gap below which the two phases of a saturation point are
/// the same phase (the trivial solution). Equimolar methane/ethane (PR) has a
/// gap of ≈ 0.25 one envelope step (|ln K₁| ≈ 0.09) from its critical point;
/// the slowly-converging trivial approach it replaces had a gap below 10⁻³.
const TRIVIAL_Z: f64 = 1e-2;

/// Solve for the saturation **temperature** at fixed `p` (bubble-T or dew-T).
///
/// `p` in **kPa absolute**; the returned `var` is the temperature in **K**.
/// Tries [`newton_temperature`] first and falls back to
/// [`bisect_temperature`] whenever Newton declines (returns `None`) — see the
/// module docs for when that happens. Both paths solve the same equation, so
/// the fallback changes the cost, never the answer.
pub(crate) fn solve_temperature(
    spec: &SystemSpec,
    p: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    // `Option::map_or_else`-style chaining: `Some(sp)` short-circuits to
    // `Ok(sp)`; `None` runs the (slow, robust) bisection instead.
    match newton_temperature(spec, p, known, point, tol, max_iter) {
        Some(sp) => Ok(sp),
        None => bisect_temperature(spec, p, known, point, tol, max_iter),
    }
}

/// Is a caller's starting guess usable: a finite positive value and, if
/// K-values are given, the right length and all finite and positive.
fn guess_ok(spec: &SystemSpec, value: f64, k0: Option<&[f64]>) -> bool {
    value.is_finite()
        && value > 0.0
        && k0.is_none_or(|k| k.len() == spec.n() && k.iter().all(|v| v.is_finite() && *v > 0.0))
}

/// [`solve_temperature`] **warm-started** from a caller's guess: `t0` in
/// **K**, optional K-values `k0` (Wilson at `(t0, p)` when absent).
///
/// Runs [`newton_temperature_from`] from the guess; if that declines — or
/// the guess is unusable — it runs the cold [`solve_temperature`]. Both solve
/// the same equation to the same tolerance, so a guess changes the cost,
/// never the answer: a good guess (the previous stage's or iteration's
/// temperature and K) saves the Wilson seeding and most Newton steps, a bad
/// one costs one declined attempt.
#[allow(clippy::too_many_arguments)]
pub(crate) fn solve_temperature_from(
    spec: &SystemSpec,
    p: f64,
    known: &[f64],
    point: Point,
    t0: f64,
    k0: Option<&[f64]>,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    if guess_ok(spec, t0, k0) {
        let warm = match k0 {
            Some(k) => newton_temperature_from(spec, p, known, point, t0, k, tol, max_iter),
            None => {
                let kw = wilson_k_init(spec, t0, p);
                newton_temperature_from(spec, p, known, point, t0, &kw, tol, max_iter)
            }
        };
        if let Some(sp) = warm {
            return Ok(sp);
        }
    }
    solve_temperature(spec, p, known, point, tol, max_iter)
}

/// [`solve_pressure`] **warm-started** from a caller's guess: `p0` in
/// **kPa**, optional K-values `k0` (Wilson at `(t, p0)` when absent).
///
/// Runs [`newton_pressure_from`] from the guess; if that declines, the
/// `P ← P·S` iteration from the same guess, accepting a result that is not
/// the trivial solution ([`is_trivial`]); otherwise — or for an unusable
/// guess — the cold [`solve_pressure`], trivial-solution recovery included. Same equation, same tolerance: the guess changes the cost, never
/// the answer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn solve_pressure_from(
    spec: &SystemSpec,
    t: f64,
    known: &[f64],
    point: Point,
    p0: f64,
    k0: Option<&[f64]>,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    if guess_ok(spec, p0, k0) {
        let k = k0.map_or_else(|| wilson_k_init(spec, t, p0), <[f64]>::to_vec);
        if let Some(sp) = newton_pressure_from(spec, t, known, point, p0, &k, tol, max_iter) {
            return Ok(sp);
        }
        if let Ok(sp) = multiplicative_pressure_k(spec, t, p0, k, known, point, tol, max_iter) {
            if !is_trivial(spec, t, sp.var, known, &sp.incipient, point)? {
                return Ok(sp);
            }
        }
    }
    solve_pressure(spec, t, known, point, tol, max_iter)
}

/// Below this `max |ln Kᵢ|` a converged temperature solve is treated as the
/// trivial solution (both phases on the same EOS root, `K ≡ 1`) and handed to
/// the bisection fallback. A genuine close-boiling mixture (α ≈ 1.05) sits
/// near `|ln K| ≈ 0.025`, four orders of magnitude above this; a pure
/// component or an exact azeotrope also lands here, and the fallback solves
/// those correctly too — rejecting is always safe, only slower.
const TRIVIAL_LN_K: f64 = 1e-6;

/// Largest Newton step allowed in `u = 1/T`, as a fraction of `u` (≈ 20 % of
/// T). Keeps a first step from a poor seed out of the supercritical region
/// where the EOS has only one root.
const MAX_REL_STEP: f64 = 0.2;

/// Wilson-correlation saturation temperature at `p` (**kPa**), in **K**.
///
/// With `ln Kᵢᵂ = ln(Pc,ᵢ/P) + aᵢ(1 − Tc,ᵢ·u)`, `aᵢ = 5.373(1+ωᵢ)`, `u = 1/T`,
/// each `ln Kᵢ` is *linear* in `u`, so Newton on `g(u) = ln S(u)` (bubble
/// `S = Σxᵢ Kᵢ`, dew `S = Σyᵢ/Kᵢ`) converges in a handful of steps. Returns
/// `None` if the iteration does not settle on a positive finite T.
fn wilson_temperature(spec: &SystemSpec, p: f64, known: &[f64], point: Point) -> Option<f64> {
    // Seed from the composition-weighted critical temperature: Wilson's
    // saturation T is always below Tc, typically 0.6–0.8 Tc at modest P.
    let tc_mix: f64 = spec
        .components
        .iter()
        .zip(known)
        .map(|(c, z)| z * c.tc)
        .sum();
    let mut u = 1.0 / (0.7 * tc_mix);
    for _ in 0..50 {
        let (mut s, mut ds_du) = (0.0, 0.0);
        for (c, &z) in spec.components.iter().zip(known) {
            let a = 5.373 * (1.0 + c.omega);
            let ln_k = (c.pc / p).ln() + a * (1.0 - c.tc * u);
            let dln_k_du = -a * c.tc;
            match point {
                Point::Bubble => {
                    let w = z * ln_k.exp();
                    s += w;
                    ds_du += w * dln_k_du;
                }
                Point::Dew => {
                    let w = z * (-ln_k).exp();
                    s += w;
                    ds_du -= w * dln_k_du;
                }
            }
        }
        let g = s.ln();
        let dg_du = ds_du / s;
        if !g.is_finite() || dg_du == 0.0 || !dg_du.is_finite() {
            return None;
        }
        let du = (-g / dg_du).clamp(-MAX_REL_STEP * u, MAX_REL_STEP * u);
        u += du;
        if du.abs() < 1e-12 * u {
            break;
        }
    }
    let t = 1.0 / u;
    (t.is_finite() && t > 0.0).then_some(t)
}

/// Newton solve for the saturation temperature at `p` (**kPa absolute**).
///
/// The unknown is `u = 1/T` because `ln Kᵢ` is close to linear in `1/T`
/// (Clausius–Clapeyron), which makes `ln S` nearly linear too and Newton
/// close to one-step accurate. Each iteration, at the current `(T, x, y)`:
///
/// 1. evaluate `Kᵢ` and the exact `∂ln Kᵢ/∂T` (`k_values_with_t_derivs`);
/// 2. form `wᵢ = Kᵢxᵢ` (bubble) or `wᵢ = yᵢ/Kᵢ` (dew) and `S = Σwᵢ`, so
///    `∂ln S/∂T = ±Σ wᵢ·∂ln Kᵢ/∂T / S` (`+` bubble, `−` dew);
/// 3. replace the incipient composition by `wᵢ/S` (one successive-substitution
///    step);
/// 4. take the Newton step `Δu = −ln S / (∂ln S/∂u)`, with
///    `∂/∂u = −T²·∂/∂T`, limited to [`MAX_REL_STEP`] —
///
/// or, replacing steps 3–4 once close to the root and where SS contracts
/// slowly, the full simultaneous Newton step ([`newton_temperature_from`]).
///
/// Converged when `|ln S| < tol` **and** the incipient composition moved less
/// than `tol`, so the returned K belong to the returned composition.
///
/// Returns `None` — never an error — whenever it cannot vouch for the answer,
/// so the caller can fall back: the model pair has no `∂ln K/∂T` (virial
/// vapor, Chao–Seader/Grayson–Streed/Braun K10 liquid), a K evaluation fails,
/// a step goes non-finite or non-positive, the slope has the wrong sign, the
/// loop hits `max_iter`, or it converged onto the trivial solution
/// ([`TRIVIAL_LN_K`]).
pub(crate) fn newton_temperature(
    spec: &SystemSpec,
    p: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Option<SatPoint> {
    let t0 = wilson_temperature(spec, p, known, point)?;
    // Seed the incipient phase from Wilson K at the Wilson temperature.
    let kw = wilson_k_init(spec, t0, p);
    newton_temperature_from(spec, p, known, point, t0, &kw, tol, max_iter)
}

/// Residual level (`|ln S|` and the largest incipient-composition change)
/// below which [`newton_temperature_from`] switches from the one-SS-step
/// update to the full simultaneous Newton step. Chosen by measurement — see
/// `OPTIMIZATION_PLAN_PART1.md` §14.
const FULL_NEWTON_SWITCH: f64 = 1e-2;

/// SS contraction ratio (this composition change ÷ the previous one) above
/// which the full Newton step is worth its cost. A full step costs ≈ 1.7 SS
/// iterations (it adds `d_ln_phi_d_n` and an (n+1)² LU solve to the K
/// evaluation), so a solve whose SS already contracts faster than ≈ 0.07 per
/// step finishes sooner on SS. Measured in `OPTIMIZATION_PLAN_PART1.md` §14.
const FULL_NEWTON_MIN_RATIO: f64 = 0.1;

/// Largest change in any `ln Kᵢ` one full Newton step may make; a larger
/// step is scaled down as a whole (direction kept).
const MAX_LN_K_STEP: f64 = 1.0;

/// ∂ln φ̂ᵢ/∂nⱼ of the **incipient** phase at its composition `inc` — the only
/// composition dependence of K that moves during a saturation solve (the
/// known phase is fixed). The incipient phase is the vapor at a bubble point
/// and the liquid at a dew point; see [`phase_ln_phi_jacobian`] for which
/// model pairs expose it. `None` keeps the solve on the one-SS-step update.
///
/// `t` in **K**, `p` in **kPa absolute**; the result is dimensionless.
fn incipient_ln_phi_jacobian(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    inc: &[f64],
    point: Point,
) -> Option<Vec<Vec<f64>>> {
    let phase = match point {
        Point::Bubble => PhaseId::Vapor,
        Point::Dew => PhaseId::Liquid,
    };
    phase_ln_phi_jacobian(spec, t, p, inc, phase)
}

/// One full Newton step on the (n+1) unknowns `{ln K₁…ln Kₙ, ln v}`, where
/// the free variable `v` is T (a saturation temperature) or P (a saturation
/// pressure) (`MODERNIZATION_PLAN.md` §K; Michelsen & Mollerup (26) ch. 12).
///
/// With the incipient phase derived from the unknowns — bubble
/// `yᵢ = Kᵢxᵢ/Σ Kx`, dew `xᵢ = (yᵢ/Kᵢ)/Σ y/K` — the equations are
/// ```text
///   Fᵢ    = ln Kᵢ − ln Kᵢᵐᵒᵈᵉˡ(T, P, x, y) = 0   i = 1…n
///   Fₙ₊₁ = ln Σ Kᵢxᵢ  (bubble)   or   ln Σ yᵢ/Kᵢ  (dew)  = 0
/// ```
/// and, writing `Gᵢⱼ = ∂ln φ̂ᵢ/∂nⱼ` of the incipient phase at its normalized
/// composition `ŵ` (ln φ̂ is intensive, so the mole-number derivative at
/// total S is `G/S`, and `∂nⱼ/∂ln Kⱼ = ±nⱼ`):
/// ```text
///   ∂Fᵢ/∂ln Kⱼ   = δᵢⱼ + Gᵢⱼ·ŵⱼ                  (both points)
///   ∂Fᵢ/∂ln v    = −v·∂ln Kᵢᵐᵒᵈᵉˡ/∂v
///   ∂Fₙ₊₁/∂ln Kⱼ = +ŵⱼ (bubble), −ŵⱼ (dew);   ∂Fₙ₊₁/∂ln v = 0
/// ```
/// `d_ln_k_d_ln_v` is `v·∂ln Kᵢᵐᵒᵈᵉˡ/∂v` (dimensionless) and `max_dlv` the
/// largest allowed `|Δln v|`. Returns `(Δln K, Δln v)`, already scaled by the
/// step limits, or `None` if the system is singular or the step is not finite.
#[allow(clippy::too_many_arguments)]
fn full_newton_step(
    point: Point,
    ln_k: &[f64],
    model_ln_k: &[f64],
    d_ln_k_d_ln_v: &[f64],
    max_dlv: f64,
    inc: &[f64],
    ln_sum: f64,
    jac_inc: &[Vec<f64>],
) -> Option<(Vec<f64>, f64)> {
    let n = ln_k.len();
    let sign = match point {
        Point::Bubble => 1.0,
        Point::Dew => -1.0,
    };
    // `from_fn` fills the dense (n+1)×(n+1) matrix entry by entry; the last
    // row/column is the sum equation / the ln T unknown.
    let jac = nalgebra::DMatrix::from_fn(n + 1, n + 1, |r, c| match (r < n, c < n) {
        (true, true) => f64::from(u8::from(r == c)) + jac_inc[r][c] * inc[c],
        (true, false) => -d_ln_k_d_ln_v[r],
        (false, true) => sign * inc[c],
        (false, false) => 0.0,
    });
    let rhs = nalgebra::DVector::from_fn(n + 1, |r, _| {
        if r < n {
            -(ln_k[r] - model_ln_k[r])
        } else {
            -ln_sum
        }
    });
    let delta = jac.lu().solve(&rhs)?;
    if delta.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // Scale the whole step so neither limit is exceeded; the direction is
    // what Newton got right, only the length is distrusted far from the root.
    let max_dlk = delta.rows(0, n).amax();
    let dlt = delta[n];
    let mut scale: f64 = 1.0;
    if max_dlk > MAX_LN_K_STEP {
        scale = scale.min(MAX_LN_K_STEP / max_dlk);
    }
    if dlt.abs() > max_dlv {
        scale = scale.min(max_dlv / dlt.abs());
    }
    Some((
        delta.rows(0, n).iter().map(|v| v * scale).collect(),
        dlt * scale,
    ))
}

/// The saturation-temperature Newton iteration of [`newton_temperature`]
/// from a caller's starting point: temperature `t0` in **K** and K-values
/// `k0` (which seed the incipient composition, `wᵢ ∝ Kᵢxᵢ` or `yᵢ/Kᵢ`). `p`
/// in **kPa**. Returns `None` exactly when [`newton_temperature`] would.
///
/// Two update rules share one acceptance test:
/// - **One-SS-step Newton on 1/T** (the v0.18.0 method, always available):
///   `ln K ← ln Kᵐᵒᵈᵉˡ` and a Newton step in `u = 1/T` on `ln S`. Composition
///   converges linearly.
/// - **Full simultaneous Newton** on `{ln K, ln T}` ([`full_newton_step`]),
///   once `|ln S|` and the composition change are both below
///   [`FULL_NEWTON_SWITCH`] and the model pair exposes the incipient phase's
///   composition Jacobian ([`incipient_ln_phi_jacobian`]). Converges
///   quadratically, which is what lets a realistic warm start pay
///   (`OPTIMIZATION_PLAN_PART1.md` §11, §14).
///
/// A full step that fails (singular, non-finite) or does not reduce the
/// residual hands the rest of the solve back to the first rule. The
/// acceptance test is the v0.18.0 one in both cases — `|ln S| < tol` and an
/// SS composition change below `tol`, both evaluated with the model K — so
/// the switch changes the path, never the answer it accepts.
#[allow(clippy::too_many_arguments)]
pub(crate) fn newton_temperature_from(
    spec: &SystemSpec,
    p: f64,
    known: &[f64],
    point: Point,
    t0: f64,
    k0: &[f64],
    tol: f64,
    max_iter: usize,
) -> Option<SatPoint> {
    saturation_iteration(
        spec,
        Free::Temperature,
        p,
        known,
        point,
        t0,
        k0,
        tol,
        max_iter,
        true,
    )
    .map(|(sp, _)| sp)
}

/// Newton solve for the saturation **pressure** at `t` (**K**), Wilson-seeded
/// — the pressure counterpart of [`newton_temperature`], on the unknown
/// `ln P` (`OPTIMIZATION_PLAN_PART1.md` §17).
///
/// Each iteration evaluates `Kᵢ` and the exact `∂ln Kᵢ/∂P` at the current
/// `(P, x, y)`, then either takes one successive-substitution step on the
/// incipient composition plus a Newton step on `ln P` with the exact slope
/// `∂ln S/∂ln P` — which is the `P ← P·S` update when that slope is the
/// Raoult-law −1 — or, near the root and where SS is slow, the simultaneous
/// (n+1)-variable Newton step on `{ln K, ln P}`.
///
/// Returns `None` exactly where [`newton_temperature`] would (no `∂ln K/∂P`
/// for the model pair, a failed K evaluation, a non-finite step, the slope on
/// the wrong side, `max_iter`, or the trivial solution), and the caller falls
/// back to the multiplicative solver.
pub(crate) fn newton_pressure(
    spec: &SystemSpec,
    t: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Option<SatPoint> {
    let p0 = wilson_pressure_guess(spec, t, known, point).max(1e-6);
    let kw = wilson_k_init(spec, t, p0);
    newton_pressure_from(spec, t, known, point, p0, &kw, tol, max_iter)
}

/// [`newton_pressure`] from a caller's starting point: pressure `p0` in
/// **kPa absolute** and K-values `k0`. `t` in **K**.
#[allow(clippy::too_many_arguments)]
pub(crate) fn newton_pressure_from(
    spec: &SystemSpec,
    t: f64,
    known: &[f64],
    point: Point,
    p0: f64,
    k0: &[f64],
    tol: f64,
    max_iter: usize,
) -> Option<SatPoint> {
    saturation_iteration(
        spec,
        Free::Pressure,
        t,
        known,
        point,
        p0,
        k0,
        tol,
        max_iter,
        true,
    )
    .map(|(sp, _)| sp)
}

/// Which of (T, P) a saturation solve is looking for; the other one is fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Free {
    Temperature,
    Pressure,
}

/// Largest Newton step allowed in `ln P` (a factor of e in pressure). The
/// multiplicative update this replaces had no limit; the exact slope can be
/// far from −1 near the critical point, where an unlimited step leaves the
/// two-root region.
const MAX_LN_P_STEP: f64 = 1.0;

/// The loop behind [`newton_temperature_from`] and [`newton_pressure_from`],
/// also returning the number of K evaluations it took. `fixed` is the
/// specified variable (P in **kPa** for a temperature solve, T in **K** for a
/// pressure solve) and `v0` the starting value of the free one.
/// `allow_full = false` pins it to the one-SS-step update — for temperature,
/// the v0.18.0 algorithm — which the tests use as the reference the full
/// Newton step must agree with and beat.
#[allow(clippy::too_many_arguments)]
fn saturation_iteration(
    spec: &SystemSpec,
    free: Free,
    fixed: f64,
    known: &[f64],
    point: Point,
    v0: f64,
    k0: &[f64],
    tol: f64,
    max_iter: usize,
    allow_full: bool,
) -> Option<(SatPoint, usize)> {
    // `wᵢ` before normalization: Kᵢxᵢ (bubble) or yᵢ/Kᵢ (dew).
    let weight = |k: f64, z: f64| match point {
        Point::Bubble => k * z,
        Point::Dew => z / k,
    };
    // The incipient phase implied by a set of K: normalized weights, plus
    // the log of their sum (the saturation residual in the K unknowns).
    let implied = |k: &[f64]| -> Option<(Vec<f64>, f64)> {
        let mut w: Vec<f64> = k.iter().zip(known).map(|(&k, &z)| weight(k, z)).collect();
        let s: f64 = w.iter().sum();
        if !s.is_finite() || s <= 0.0 {
            return None;
        }
        w.iter_mut().for_each(|v| *v /= s);
        Some((w, s.ln()))
    };

    // `v` is the free variable: T in K or P in kPa.
    let mut v = v0;
    let mut ln_k: Vec<f64> = k0.iter().map(|k| k.ln()).collect();
    let (mut incipient, _) = implied(k0)?;
    // Full Newton is tried only for model pairs that expose the incipient
    // phase's composition Jacobian, and abandoned for good after a failure.
    let mut full_newton = allow_full;
    // Residual norm after the last full step, to check the next one reduced it.
    let mut last_full_residual = f64::INFINITY;
    // Previous SS composition change, for the contraction ratio; `None`
    // before the first step (a caller's guess, or Wilson on a cold solve).
    let mut prev_step: Option<f64> = None;
    // Once a full step has been taken, the solve stays on full Newton.
    let mut in_full = false;

    for iter in 0..max_iter.max(1) {
        let (t, p) = match free {
            Free::Temperature => (v, fixed),
            Free::Pressure => (fixed, v),
        };
        // (x, y) ordering for the K evaluation: the known phase is the
        // liquid for a bubble point and the vapor for a dew point.
        let (x, y) = match point {
            Point::Bubble => (known, incipient.as_slice()),
            Point::Dew => (incipient.as_slice(), known),
        };
        // `.ok()?` turns any FlashError (including `Unsupported`) into a
        // `None` return — the signal to fall back.
        let d = match free {
            Free::Temperature => k_values_with_t_derivs(spec, t, p, x, y),
            Free::Pressure => k_values_with_p_derivs(spec, t, p, x, y),
        }
        .ok()?;
        // ∂ln Kᵢ/∂v for the free variable v.
        let d_ln_k_d_v = match free {
            Free::Temperature => &d.d_ln_k_d_t,
            Free::Pressure => &d.d_ln_k_d_p,
        };

        let (mut s, mut ds_dv) = (0.0, 0.0);
        // `zip` walks the three slices in lockstep — no index bounds checks.
        for ((&k, &z), &dlnk) in d.k.iter().zip(known).zip(d_ln_k_d_v) {
            let w = weight(k, z);
            s += w;
            ds_dv += w * dlnk;
        }
        // ∂ln S/∂v: K rising raises the bubble sum and lowers the dew sum.
        let dlns_dv = match point {
            Point::Bubble => ds_dv / s,
            Point::Dew => -ds_dv / s,
        };
        let ln_s = s.ln();
        if !ln_s.is_finite() || !dlns_dv.is_finite() {
            return None;
        }

        // The SS composition this model K implies, and how far it is from
        // the current one — the acceptance test's second half.
        let mut step = 0.0_f64;
        let ss_incipient: Vec<f64> =
            d.k.iter()
                .zip(known)
                .zip(&incipient)
                .map(|((&k, &z), &inc)| {
                    let new = weight(k, z) / s;
                    step = step.max((new - inc).abs());
                    new
                })
                .collect();

        if ln_s.abs() < tol && step < tol {
            let max_ln_k = d.k.iter().fold(0.0_f64, |m, k| m.max(k.ln().abs()));
            // Trivial by K, or the two phases on one root (same density) —
            // see `is_trivial`. Either way, let the fallback decide.
            if max_ln_k < TRIVIAL_LN_K
                || is_trivial(spec, t, p, known, &ss_incipient, point).unwrap_or(true)
            {
                return None;
            }
            return Some((
                SatPoint {
                    var: v,
                    incipient: ss_incipient,
                    k: d.k,
                },
                iter + 1,
            ));
        }

        // On the physical branch K rises with T and falls with P, so a
        // bubble sum rises with T and falls with P, and a dew sum does the
        // opposite. Any other sign means we are off that branch.
        let rising = dlns_dv > 0.0;
        let should_rise = matches!(
            (free, point),
            (Free::Temperature, Point::Bubble) | (Free::Pressure, Point::Dew)
        );
        if dlns_dv == 0.0 || rising != should_rise {
            return None;
        }

        // --- Full simultaneous Newton, when close enough and available. ---
        // Slow SS (or no history yet: the first iteration from a guess) is
        // where the quadratic step pays; fast SS finishes cheaper on its own.
        let slow_ss = prev_step.is_none_or(|prev| step > FULL_NEWTON_MIN_RATIO * prev);
        prev_step = Some(step);
        // Below √tol the composition is one SS step from done, so a full
        // step (≈ 1.7 SS steps of work) cannot pay for itself.
        let worth_it = in_full || (slow_ss && step > tol.sqrt());
        if full_newton && worth_it && ln_s.abs() < FULL_NEWTON_SWITCH && step < FULL_NEWTON_SWITCH {
            let model_ln_k: Vec<f64> = d.k.iter().map(|k| k.ln()).collect();
            let (_, ln_sum) = implied(&ln_k.iter().map(|v| v.exp()).collect::<Vec<_>>())?;
            // Residual of the (n+1) equations at the current unknowns.
            let residual = ln_k
                .iter()
                .zip(&model_ln_k)
                .fold(ln_sum.abs(), |m, (a, b)| m.max((a - b).abs()));
            let d_ln_k_d_ln_v: Vec<f64> = d_ln_k_d_v.iter().map(|dk| v * dk).collect();
            let max_dlv = match free {
                Free::Temperature => MAX_REL_STEP,
                Free::Pressure => MAX_LN_P_STEP,
            };
            let taken = (residual < last_full_residual)
                .then(|| incipient_ln_phi_jacobian(spec, t, p, &incipient, point))
                .flatten()
                .and_then(|g| {
                    full_newton_step(
                        point,
                        &ln_k,
                        &model_ln_k,
                        &d_ln_k_d_ln_v,
                        max_dlv,
                        &incipient,
                        ln_sum,
                        &g,
                    )
                });
            if let Some((dlk, dlv)) = taken {
                for (lk, dv) in ln_k.iter_mut().zip(&dlk) {
                    *lk += dv;
                }
                let v_new = v * dlv.exp();
                let k_new: Vec<f64> = ln_k.iter().map(|lk| lk.exp()).collect();
                if let (true, Some((inc, _))) = (v_new.is_finite() && v_new > 0.0, implied(&k_new))
                {
                    v = v_new;
                    incipient = inc;
                    last_full_residual = residual;
                    in_full = true;
                    continue;
                }
                return None;
            }
            // Declined (unsupported pair, singular, or the residual grew):
            // finish on the one-SS-step update.
            full_newton = false;
        }

        // --- One SS step on composition + a Newton step on the free variable. ---
        incipient = ss_incipient;
        ln_k = d.k.iter().map(|k| k.ln()).collect();
        v = match free {
            // Newton on u = 1/T, in which ln S is close to linear.
            Free::Temperature => {
                let u = 1.0 / v;
                let dlns_du = -v * v * dlns_dv;
                let du = (-ln_s / dlns_du).clamp(-MAX_REL_STEP * u, MAX_REL_STEP * u);
                1.0 / (u + du)
            }
            // Newton on ln P; ∂ln S/∂ln P = P·∂ln S/∂P.
            Free::Pressure => {
                let dlnp = (-ln_s / (v * dlns_dv)).clamp(-MAX_LN_P_STEP, MAX_LN_P_STEP);
                v * dlnp.exp()
            }
        };
        if !v.is_finite() || v <= 0.0 {
            return None;
        }
    }
    None
}

/// Saturation temperature by **bisection** on the pressure solver — the
/// robust fallback behind [`newton_temperature`].
///
/// Rather than root-find the sum condition `S(T) = 1` directly, we invert the
/// robust saturation-**pressure** solver: `P_sat(T)` is smooth and strictly
/// increasing in T, so we bracket-bisect for the `T*` where `P_sat(T*) = p`.
/// This deliberately avoids the trivial-K filtering a direct `S(T)` objective
/// needs — that filter rejects the real root for *close-boiling* φ-φ systems
/// (relative volatility α≈1) whose equilibrium K genuinely sit near 1, which
/// is exactly where the true bubble/dew T lives. [`solve_pressure`] carries
/// no such filter and converges wherever a saturation pressure exists.
///
/// **Cost:** every point of the bracketing scan (up to 97, on a 50–1000 K grid)
/// and every bisection step runs a full, cold-started [`solve_pressure`] —
/// thousands of K evaluations in all. Measured on a 10-component SRK mixture:
/// ~3.9 ms for a bubble point and ~34 ms for a dew point, against ~16 µs for
/// [`newton_temperature`]. A final incipient solve at `(T*, p)` returns the
/// composition + K.
pub(crate) fn bisect_temperature(
    spec: &SystemSpec,
    p: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    // Objective: h(T) = ln(P_sat(T) / p), strictly increasing in T and zero
    // at the saturation temperature. `None` where the pressure solve can't
    // converge (well outside the physical saturation range — e.g. T ≳ the
    // mixture pseudo-critical). Works for both φ-φ and γ-φ liquids because
    // `solve_pressure` handles both; no trivial-K filtering is involved.
    let h = |t: f64| -> Option<f64> {
        solve_pressure(spec, t, known, point, tol, max_iter)
            .ok()
            .map(|sp| sp.var)
            .filter(|pt| pt.is_finite() && *pt > 0.0)
            .map(|pt| (pt / p).ln())
    };
    // Scan upward for the first sign change; since h is monotone we stop at
    // the (unique) bracket around the root, keeping the endpoint values so we
    // don't re-solve there.
    //
    // An undefined h right after a negative one means the scan stepped past
    // the end of the saturation curve (near the mixture critical point the
    // pressure solve finds only the trivial solution and refuses, since
    // v0.17.1). The root, if any, lies before that end, so the step counts as
    // a bracket whose high side is "undefined"; the bisection below treats
    // undefined midpoints as high, and the residual check at the end rejects
    // the case where `p` is above the whole curve and there is no root.
    // (Before v0.17.1 those undefined points were trivial "pressures" that
    // happened to bracket the same way — and nothing checked the final
    // residual.)
    let (t_start, t_end, steps) = (50.0, 1000.0, 96);
    let mut prev: Option<(f64, f64)> = None;
    let mut bracket: Option<(f64, f64, f64)> = None; // (lo, hi, h_lo)
    for i in 0..=steps {
        let t = t_start + (t_end - t_start) * i as f64 / steps as f64;
        let ht = match h(t) {
            Some(v) => v,
            None => {
                if let Some((tp, hp)) = prev {
                    if hp < 0.0 {
                        bracket = Some((tp, t, hp));
                        break;
                    }
                }
                prev = None;
                continue;
            }
        };
        if let Some((tp, hp)) = prev {
            if hp * ht <= 0.0 {
                bracket = Some((tp, t, hp));
                break;
            }
        }
        prev = Some((t, ht));
    }
    let (mut lo, mut hi, mut h_lo) = bracket.ok_or(FlashError::NoConvergence {
        what: "saturation temperature bracket",
        iters: steps,
        residual: f64::NAN,
    })?;
    let mut t_star = 0.5 * (lo + hi);
    for iter in 0..max_iter {
        t_star = 0.5 * (lo + hi);
        // Undefined at the midpoint ⇒ past the end of the saturation curve,
        // i.e. on the high side of the root.
        let ht = h(t_star);
        if let Some(v) = ht {
            if v.abs() < tol {
                break;
            }
        }
        if (hi - lo) < 1e-9 {
            // The bracket has collapsed without the residual reaching `tol`:
            // either `p` is above the end of the saturation curve (no root)
            // or h jumps here. Neither is an answer.
            return Err(FlashError::NoConvergence {
                what: "saturation temperature (bracket collapsed without a root)",
                iters: iter + 1,
                residual: ht.map_or(f64::INFINITY, f64::abs),
            });
        }
        match ht {
            Some(v) if v * h_lo > 0.0 => {
                lo = t_star;
                h_lo = v;
            }
            _ => hi = t_star,
        }
        if iter + 1 == max_iter {
            return Err(FlashError::NoConvergence {
                what: "saturation temperature",
                iters: max_iter,
                residual: ht.map_or(f64::INFINITY, f64::abs),
            });
        }
    }
    // Recover composition + K at the solution.
    let mut k = wilson_k_init(spec, t_star, p);
    let (_, incipient) = incipient_sum(spec, t_star, p, known, point, &mut k, 60)?;
    Ok(SatPoint {
        var: t_star,
        incipient,
        k,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::ActivityModel;
    use crate::eos::{CubicEos, LiquidModel, VaporModel};
    use crate::mixing::MixingRule;
    use crate::types::Component;

    fn comp(name: &str, tc: f64, pc: f64, omega: f64) -> Component {
        Component {
            name: name.into(),
            tc,
            pc,
            omega,
            ..Component::default()
        }
    }

    /// A φ-φ spec on RKS (Soave) with no kij.
    fn rks(components: &[Component]) -> SystemSpec<'_> {
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

    /// PR methane/ethane and CO₂/n-butane (k = 0.13) — both have bubble
    /// points the multiplicative pressure update used to "solve" as the
    /// trivial solution `K ≡ 1`.
    fn near_critical_pairs() -> ([Component; 2], [Component; 2]) {
        let c = |name: &str, tc, pc, omega| Component {
            name: name.into(),
            tc,
            pc,
            omega,
            ..Component::default()
        };
        (
            [
                c("methane", 190.564, 4599.0, 0.0115),
                c("ethane", 305.32, 4872.0, 0.0995),
            ],
            [
                c("CO2", 304.13, 7377.0, 0.2239),
                c("n-butane", 425.12, 3796.0, 0.200),
            ],
        )
    }

    fn pr_spec<'a>(components: &'a [Component], kij: &'a [Vec<f64>]) -> SystemSpec<'a> {
        SystemSpec {
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
            kij,
            ..rks(components)
        }
    }

    #[test]
    fn bubble_pressure_never_returns_the_trivial_solution() {
        // Regression (v0.17.1). Before: methane/ethane at 221.7 K returned
        // 5198 kPa (true 4100 kPa, from the phase envelope), CO₂/n-butane at
        // 310 K, x = 0.8 returned 6753 kPa (true ≈ 6.2 MPa) — both with
        // K ≡ 1 and y = x, as Ok. Now: a genuine K, and the answer agrees
        // with the independent Newton saturation-temperature solve.
        let (me, co2_c4) = near_critical_pairs();
        let kij = [vec![0.0, 0.13], vec![0.13, 0.0]];
        for (comps, kij, t, x) in [
            (&me, &[][..], 221.726, [0.5, 0.5]),
            (&me, &[][..], 233.368, [0.5, 0.5]),
            (&co2_c4, &kij[..], 310.0, [0.75, 0.25]),
            (&co2_c4, &kij[..], 310.0, [0.8, 0.2]),
        ] {
            let spec = pr_spec(comps, kij);
            let sp = solve_pressure(&spec, t, &x, Point::Bubble, 1e-10, 200).unwrap();
            assert!(sp.k[0] > 1.1, "{}: trivial K {:?}", comps[0].name, sp.k);
            let tb = newton_temperature(&spec, sp.var, &x, Point::Bubble, 1e-10, 200).unwrap();
            assert!(
                (tb.var - t).abs() < 1e-4,
                "{} at {t} K: bubble P {} gives bubble T {}",
                comps[0].name,
                sp.var,
                tb.var
            );
        }
        // The known envelope point (221.726 K, 4100.0 kPa).
        let spec = pr_spec(&me, &[]);
        let sp = solve_pressure(&spec, 221.726, &[0.5, 0.5], Point::Bubble, 1e-10, 200).unwrap();
        assert!((sp.var - 4100.0).abs() < 1.0, "P = {}", sp.var);
    }

    #[test]
    fn near_critical_bubble_points_decline_rather_than_answer_wrongly() {
        // Methane/ethane 50/50 (PR) at 251.565 K: the phase envelope's bubble
        // point is 6277.0 kPa. Within ~15 K of the critical point the
        // incipient solvers may not find it; what they must not do is return
        // anything else (before v0.17.1: 4639.5 kPa, and bubble T at
        // 6277 kPa came back as 230.68 K).
        let (me, _) = near_critical_pairs();
        let spec = pr_spec(&me, &[]);
        if let Ok(sp) = solve_pressure(&spec, 251.565, &[0.5, 0.5], Point::Bubble, 1e-10, 200) {
            assert!((sp.var - 6277.0).abs() < 1.0, "bubble P {}", sp.var);
        }
        if let Ok(sp) = solve_temperature(&spec, 6277.0, &[0.5, 0.5], Point::Bubble, 1e-10, 200) {
            assert!((sp.var - 251.565).abs() < 1e-2, "bubble T {}", sp.var);
        }
    }

    fn methanol_water() -> [Component; 2] {
        [
            Component {
                liquid_volume: 40.7,
                psat_coeffs: vec![5.20, 3200.0, -35.0],
                ..comp("methanol", 512.6, 8097.0, 0.564)
            },
            Component {
                liquid_volume: 18.07,
                psat_coeffs: vec![5.11, 3800.0, -46.0],
                ..comp("water", 647.1, 22064.0, 0.344)
            },
        ]
    }

    /// Newton must accept the case (no silent fallback) and agree with the
    /// bisection oracle on T and on the incipient composition.
    fn assert_newton_matches_bisection(spec: &SystemSpec, p: f64, known: &[f64], point: Point) {
        let tol = 1e-9;
        let nw = newton_temperature(spec, p, known, point, tol, 200)
            .unwrap_or_else(|| panic!("Newton declined {point:?} at p={p}, z={known:?}"));
        let bi = bisect_temperature(spec, p, known, point, tol, 200).unwrap();
        assert!(
            (nw.var - bi.var).abs() < 1e-6,
            "{point:?} z={known:?}: Newton T={} vs bisection T={}",
            nw.var,
            bi.var
        );
        for (a, b) in nw.incipient.iter().zip(&bi.incipient) {
            assert!(
                (a - b).abs() < 1e-6,
                "{point:?} z={known:?}: incipient {a} vs {b}"
            );
        }
    }

    #[test]
    fn newton_matches_bisection_phi_phi_binary() {
        let comps = [
            comp("n-butane", 425.12, 3796.0, 0.200),
            comp("n-heptane", 540.2, 2740.0, 0.350),
        ];
        let spec = rks(&comps);
        for p in [101.325, 1000.0, 2000.0] {
            for x1 in [0.05, 0.4, 0.9] {
                let z = [x1, 1.0 - x1];
                assert_newton_matches_bisection(&spec, p, &z, Point::Bubble);
                assert_newton_matches_bisection(&spec, p, &z, Point::Dew);
            }
        }
    }

    #[test]
    fn newton_matches_bisection_close_boiling() {
        // benzene/cyclohexane, α ≈ 1.02: K sit near 1 but well clear of the
        // trivial-solution guard, so Newton must handle them itself.
        let comps = [
            comp("benzene", 562.02, 4907.277, 0.211),
            comp("cyclohexane", 553.6, 4080.5, 0.2096),
        ];
        let spec = rks(&comps);
        for x1 in [0.01, 0.3, 0.7, 0.99] {
            let z = [x1, 1.0 - x1];
            assert_newton_matches_bisection(&spec, 101.325, &z, Point::Bubble);
            assert_newton_matches_bisection(&spec, 101.325, &z, Point::Dew);
        }
    }

    #[test]
    fn newton_matches_bisection_gamma_phi() {
        let comps = methanol_water();
        let aij = vec![vec![0.0, 0.5853], vec![0.3458, 0.0]];
        let spec = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            aij: &aij,
            ..rks(&comps)
        };
        for x1 in [0.1, 0.5, 0.9] {
            let z = [x1, 1.0 - x1];
            assert_newton_matches_bisection(&spec, 101.325, &z, Point::Bubble);
            assert_newton_matches_bisection(&spec, 101.325, &z, Point::Dew);
        }
    }

    #[test]
    fn pure_component_falls_back_and_still_solves() {
        // A pure component has K ≡ 1 at saturation — indistinguishable from
        // the trivial solution, so Newton must decline and bisection answer.
        let comps = [comp("n-heptane", 540.2, 2740.0, 0.350)];
        let spec = rks(&comps);
        assert!(newton_temperature(&spec, 101.325, &[1.0], Point::Bubble, 1e-9, 200).is_none());
        let sp = solve_temperature(&spec, 101.325, &[1.0], Point::Bubble, 1e-9, 200).unwrap();
        assert!(
            sp.var > 340.0 && sp.var < 400.0,
            "pure n-heptane Tb = {}",
            sp.var
        );
    }

    #[test]
    fn model_without_dlnk_dt_falls_back_and_still_solves() {
        // Virial vapor has no ∂ln K/∂T yet → Newton declines, bisection solves.
        let comps = methanol_water();
        let spec = SystemSpec {
            vapor: VaporModel::Virial,
            liquid: LiquidModel::IdealSolution,
            ..rks(&comps)
        };
        let z = [0.5, 0.5];
        assert!(newton_temperature(&spec, 101.325, &z, Point::Bubble, 1e-9, 200).is_none());
        let sp = solve_temperature(&spec, 101.325, &z, Point::Bubble, 1e-9, 200).unwrap();
        // The fixture's Antoine constants are illustrative, so check the
        // saturation condition Σ Kᵢxᵢ = 1 rather than a literature T.
        let k = k_values(&spec, sp.var, 101.325, &z, &sp.incipient).unwrap();
        let s: f64 = k.iter().zip(&z).map(|(k, x)| k * x).sum();
        assert!((s - 1.0).abs() < 1e-6, "Σ Kx = {s} at T = {}", sp.var);
    }

    /// The case that motivated the Newton path: the 10-component light
    /// hydrocarbon SRK mixture of the downstream column library's benchmark.
    #[cfg(feature = "component-db")]
    #[test]
    fn newton_matches_bisection_ten_component_srk() {
        let names = [
            "propane",
            "isobutane",
            "n-butane",
            "isopentane",
            "n-pentane",
            "n-hexane",
            "n-heptane",
            "n-octane",
            "n-nonane",
            "n-decane",
        ];
        let comps: Vec<Component> = names
            .iter()
            .map(|n| crate::db::component(n).unwrap())
            .collect();
        let spec = rks(&comps);
        let z = vec![0.1; 10];
        for p in [200.0, 1000.0] {
            assert_newton_matches_bisection(&spec, p, &z, Point::Bubble);
            assert_newton_matches_bisection(&spec, p, &z, Point::Dew);
        }
    }

    /// Run [`saturation_iteration`] with and without the full Newton step
    /// from the same start, and require the same answer. Returns the two
    /// K-evaluation counts `(full, ss_only)`.
    #[allow(clippy::too_many_arguments)]
    fn full_vs_ss(
        spec: &SystemSpec,
        p: f64,
        known: &[f64],
        point: Point,
        t0: f64,
        k0: &[f64],
    ) -> (usize, usize) {
        let tol = 1e-10;
        let (full, n_full) = saturation_iteration(
            spec,
            Free::Temperature,
            p,
            known,
            point,
            t0,
            k0,
            tol,
            200,
            true,
        )
        .unwrap_or_else(|| panic!("full Newton declined {point:?} at p={p}"));
        let (ss, n_ss) = saturation_iteration(
            spec,
            Free::Temperature,
            p,
            known,
            point,
            t0,
            k0,
            tol,
            200,
            false,
        )
        .unwrap_or_else(|| panic!("SS-only declined {point:?} at p={p}"));
        assert!(
            (full.var - ss.var).abs() < 1e-7,
            "{point:?} p={p}: full {} vs SS-only {}",
            full.var,
            ss.var
        );
        for (a, b) in full.incipient.iter().zip(&ss.incipient) {
            assert!(
                (a - b).abs() < 1e-7,
                "{point:?} p={p}: incipient {a} vs {b}"
            );
        }
        (n_full, n_ss)
    }

    /// Near the critical locus SS on composition contracts slowly, which is
    /// where the simultaneous (n+1) Newton step pays: same answer, far fewer
    /// K evaluations, cold (Wilson seed) and warm (answer nudged by 0.5 K and
    /// 5 % on K). `OPTIMIZATION_PLAN_PART1.md` §14.
    #[test]
    fn full_newton_cuts_iterations_near_critical_and_agrees() {
        let (me, _) = near_critical_pairs();
        let spec = pr_spec(&me, &[]);
        let z = [0.5, 0.5];
        for (p, point) in [
            (5500.0, Point::Bubble),
            (5500.0, Point::Dew),
            (6000.0, Point::Bubble),
        ] {
            let t0 = wilson_temperature(&spec, p, &z, point).unwrap();
            let kw = wilson_k_init(&spec, t0, p);
            let (cold_full, cold_ss) = full_vs_ss(&spec, p, &z, point, t0, &kw);
            assert!(
                2 * cold_full <= cold_ss,
                "{point:?} {p} kPa cold: full {cold_full} vs SS-only {cold_ss} evaluations"
            );
            let ans = newton_temperature(&spec, p, &z, point, 1e-10, 200).unwrap();
            let k_near: Vec<f64> = ans.k.iter().map(|k| k * 1.05).collect();
            let (warm_full, warm_ss) = full_vs_ss(&spec, p, &z, point, ans.var + 0.5, &k_near);
            assert!(
                2 * warm_full <= warm_ss,
                "{point:?} {p} kPa warm: full {warm_full} vs SS-only {warm_ss} evaluations"
            );
        }
    }

    /// Every model pair the full step supports agrees with the SS-only
    /// reference, and never takes more K evaluations from a warm guess —
    /// φ-φ RKS binary, close-boiling, and a γ-φ bubble point (ideal-gas
    /// vapor, so the incipient Jacobian is zero). On the φ-φ binary it must
    /// also be *quadratic*: 3 evaluations from a 0.5 K / 5 % guess (SS-only
    /// takes 5–7). A wrong Jacobian still converges — the acceptance test is
    /// the SS one — but needs 4–6, so this is what catches it.
    #[test]
    fn full_newton_agrees_with_ss_only_across_model_pairs() {
        let butane_heptane = [
            comp("n-butane", 425.12, 3796.0, 0.200),
            comp("n-heptane", 540.2, 2740.0, 0.350),
        ];
        let benzene_chx = [
            comp("benzene", 562.02, 4907.277, 0.211),
            comp("cyclohexane", 553.6, 4080.5, 0.2096),
        ];
        let mw = methanol_water();
        let aij = vec![vec![0.0, 0.5853], vec![0.3458, 0.0]];
        let gamma_phi = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            aij: &aij,
            ..rks(&mw)
        };
        let cases: [(SystemSpec, f64, &[Point], usize); 3] = [
            (
                rks(&butane_heptane),
                1000.0,
                &[Point::Bubble, Point::Dew],
                3,
            ),
            (
                rks(&benzene_chx),
                101.325,
                &[Point::Bubble, Point::Dew],
                200,
            ),
            (gamma_phi, 101.325, &[Point::Bubble], 200),
        ];
        for (spec, p, points, max_full) in &cases {
            for &point in *points {
                for x1 in [0.1, 0.5, 0.9] {
                    let z = [x1, 1.0 - x1];
                    let ans = newton_temperature(spec, *p, &z, point, 1e-10, 200).unwrap();
                    let k_near: Vec<f64> = ans.k.iter().map(|k| k * 1.05).collect();
                    let (full, ss) = full_vs_ss(spec, *p, &z, point, ans.var + 0.5, &k_near);
                    assert!(full <= ss, "{point:?} z={z:?}: full {full} > SS-only {ss}");
                    assert!(
                        full <= *max_full,
                        "{point:?} z={z:?}: {full} evaluations, quadratic needs ≤ {max_full}"
                    );
                }
            }
        }
    }

    /// A model pair with no incipient composition Jacobian (a Grayson–Streed
    /// liquid at a dew point) must run the SS-only update unchanged.
    #[test]
    fn full_newton_is_skipped_where_no_composition_jacobian_exists() {
        let mw = methanol_water();
        let spec = SystemSpec {
            liquid: LiquidModel::GraysonStreed,
            ..rks(&mw)
        };
        let z = [0.4, 0.6];
        assert!(incipient_ln_phi_jacobian(&spec, 350.0, 101.325, &z, Point::Dew).is_none());
    }

    /// Warm-started solve ≡ cold solve, from a near guess (the cold answer
    /// nudged — a neighboring stage's result) and from deliberately bad or
    /// malformed guesses (which must fall back, not fail or drift).
    fn assert_warm_matches_cold(spec: &SystemSpec, p: f64, t: f64, z: &[f64]) {
        use crate::flash::bubble::{
            SaturationResult, bubble_pressure, bubble_pressure_from, bubble_temperature,
            bubble_temperature_from,
        };
        use crate::flash::dew::{
            dew_pressure, dew_pressure_from, dew_temperature, dew_temperature_from,
        };
        let tol = 1e-10;
        let n = z.len();
        let close = |a: f64, b: f64, what: &str| {
            assert!(
                (a - b).abs() < 1e-6 * b.abs().max(1.0),
                "{what}: warm {a} vs cold {b}"
            );
        };
        let bad_k = vec![1.0; n + 1]; // wrong length
        let nan_k = vec![f64::NAN; n];

        type TFn = fn(&SystemSpec, f64, &[f64], f64, usize) -> Result<SaturationResult, FlashError>;
        type TFrom = fn(
            &SystemSpec,
            f64,
            &[f64],
            f64,
            Option<&[f64]>,
            f64,
            usize,
        ) -> Result<SaturationResult, FlashError>;
        let temperature: [(&str, TFn, TFrom); 2] = [
            ("bubble T", bubble_temperature, bubble_temperature_from),
            ("dew T", dew_temperature, dew_temperature_from),
        ];
        for (what, cold_fn, warm_fn) in temperature {
            let cold = cold_fn(spec, p, z, tol, 200).unwrap();
            let k_near: Vec<f64> = cold.k.iter().map(|k| k * 1.05).collect();
            for (t0, k0) in [
                (cold.value + 0.5, Some(k_near.as_slice())),
                (cold.value - 2.0, None),
                (cold.value, Some(cold.k.as_slice())),
                (50.0, None),               // absurd T
                (f64::NAN, None),           // unusable
                (cold.value, Some(&bad_k)), // malformed K
                (cold.value, Some(&nan_k)),
            ] {
                let warm = warm_fn(spec, p, z, t0, k0, tol, 200).unwrap();
                close(warm.value, cold.value, what);
                for (a, b) in warm.incipient.iter().zip(&cold.incipient) {
                    close(*a, *b, what);
                }
            }
        }
        let pressure: [(&str, TFn, TFrom); 2] = [
            ("bubble P", bubble_pressure, bubble_pressure_from),
            ("dew P", dew_pressure, dew_pressure_from),
        ];
        for (what, cold_fn, warm_fn) in pressure {
            let cold = cold_fn(spec, t, z, tol, 200).unwrap();
            for (p0, k0) in [
                (cold.value * 1.02, Some(cold.k.as_slice())),
                (cold.value * 0.9, None),
                (-1.0, None),
                (cold.value, Some(&bad_k)),
            ] {
                let warm = warm_fn(spec, t, z, p0, k0, tol, 200).unwrap();
                close(warm.value, cold.value, what);
            }
        }
    }

    #[test]
    fn warm_start_matches_cold_phi_phi_binary() {
        let comps = [
            comp("n-butane", 425.12, 3796.0, 0.200),
            comp("n-heptane", 540.2, 2740.0, 0.350),
        ];
        let spec = rks(&comps);
        for (p, t) in [(101.325, 330.0), (1000.0, 420.0)] {
            for x1 in [0.1, 0.5, 0.9] {
                assert_warm_matches_cold(&spec, p, t, &[x1, 1.0 - x1]);
            }
        }
    }

    #[test]
    fn warm_start_falls_back_from_a_trivial_guess() {
        // Methane/n-pentane at 350 K: a warm start from the trivial solution
        // (the 27 479 kPa the pre-v0.17.1 solver returned, K ≡ 1) must not
        // be accepted — it must fall back and find the real 12 383 kPa.
        let comps = [
            comp("methane", 190.56, 4599.0, 0.011),
            comp("n-pentane", 469.7, 3370.0, 0.252),
        ];
        let spec = rks(&comps);
        let z = [0.5, 0.5];
        let cold = crate::flash::bubble::bubble_pressure(&spec, 350.0, &z, 1e-9, 200).unwrap();
        let warm = crate::flash::bubble::bubble_pressure_from(
            &spec,
            350.0,
            &z,
            27_479.0,
            Some(&[1.0, 1.0]),
            1e-9,
            200,
        )
        .unwrap();
        assert!(cold.k[0] > 1.5, "cold K {:?}", cold.k);
        assert!(
            (warm.value - cold.value).abs() < 1e-3,
            "warm {} cold {}",
            warm.value,
            cold.value
        );
    }

    #[cfg(feature = "component-db")]
    #[test]
    fn warm_start_matches_cold_ten_component_srk() {
        let names = [
            "propane",
            "isobutane",
            "n-butane",
            "isopentane",
            "n-pentane",
            "n-hexane",
            "n-heptane",
            "n-octane",
            "n-nonane",
            "n-decane",
        ];
        let comps: Vec<Component> = names
            .iter()
            .map(|n| crate::db::component(n).unwrap())
            .collect();
        let spec = rks(&comps);
        assert_warm_matches_cold(&spec, 1000.0, 400.0, &[0.1; 10]);
    }

    // -----------------------------------------------------------------
    // Newton on ln P (OPTIMIZATION_PLAN_PART1 §17) and the γ-φ dew-point
    // full step (§16).
    // -----------------------------------------------------------------

    /// Newton and the multiplicative solver must land on the same pressure
    /// and incipient composition wherever both answer.
    fn assert_pressure_solvers_agree(spec: &SystemSpec, t: f64, known: &[f64], point: Point) {
        let tol = 1e-10;
        let nw = newton_pressure(spec, t, known, point, tol, 200)
            .unwrap_or_else(|| panic!("Newton declined {point:?} at T={t}, z={known:?}"));
        let mu = solve_pressure_multiplicative(spec, t, known, point, tol, 200).unwrap();
        assert!(
            (nw.var / mu.var - 1.0).abs() < 1e-7,
            "{point:?} T={t} z={known:?}: Newton P={} vs multiplicative P={}",
            nw.var,
            mu.var
        );
        for (a, b) in nw.incipient.iter().zip(&mu.incipient) {
            assert!(
                (a - b).abs() < 1e-7,
                "{point:?} T={t}: incipient {a} vs {b}"
            );
        }
    }

    #[test]
    fn newton_pressure_matches_multiplicative_phi_phi() {
        let comps = [
            comp("n-butane", 425.12, 3796.0, 0.200),
            comp("n-heptane", 540.2, 2740.0, 0.350),
        ];
        let spec = rks(&comps);
        for t in [300.0, 350.0, 400.0] {
            for x1 in [0.05, 0.4, 0.9] {
                let z = [x1, 1.0 - x1];
                assert_pressure_solvers_agree(&spec, t, &z, Point::Bubble);
                assert_pressure_solvers_agree(&spec, t, &z, Point::Dew);
            }
        }
    }

    #[test]
    fn newton_pressure_matches_multiplicative_gamma_phi() {
        let comps = methanol_water();
        let aij = vec![vec![0.0, 0.5853], vec![0.3458, 0.0]];
        for vapor in [VaporModel::IdealGas, VaporModel::Cubic(CubicEos::RKS1972)] {
            let spec = SystemSpec {
                vapor,
                liquid: LiquidModel::Activity(ActivityModel::VanLaar),
                aij: &aij,
                ..rks(&comps)
            };
            for t in [298.15, 340.0, 370.0] {
                for x1 in [0.1, 0.5, 0.9] {
                    let z = [x1, 1.0 - x1];
                    assert_pressure_solvers_agree(&spec, t, &z, Point::Bubble);
                    assert_pressure_solvers_agree(&spec, t, &z, Point::Dew);
                }
            }
        }
    }

    #[test]
    fn newton_pressure_matches_multiplicative_ten_component_srk() {
        let comps: Vec<Component> = [
            ("propane", 369.83, 4248.0, 0.152),
            ("isobutane", 407.8, 3640.0, 0.184),
            ("n-butane", 425.12, 3796.0, 0.200),
            ("isopentane", 460.4, 3380.0, 0.228),
            ("n-pentane", 469.7, 3370.0, 0.252),
            ("n-hexane", 507.6, 3025.0, 0.301),
            ("n-heptane", 540.2, 2740.0, 0.350),
            ("n-octane", 568.7, 2490.0, 0.399),
            ("n-nonane", 594.6, 2290.0, 0.445),
            ("n-decane", 617.7, 2110.0, 0.490),
        ]
        .iter()
        .map(|&(n, tc, pc, w)| comp(n, tc, pc, w))
        .collect();
        let spec = rks(&comps);
        for t in [320.0, 400.0, 480.0] {
            assert_pressure_solvers_agree(&spec, t, &[0.1; 10], Point::Bubble);
            assert_pressure_solvers_agree(&spec, t, &[0.1; 10], Point::Dew);
        }
    }

    /// The CO₂/n-butane bubble point the multiplicative update could only
    /// reach through the secant-on-ln-P inversion of the temperature solver
    /// (it found K ≡ 1 from every restart): Newton on ln P finds it directly.
    #[test]
    fn newton_pressure_solves_a_point_the_multiplicative_update_cannot() {
        let (_, cb) = near_critical_pairs();
        let kij = vec![vec![0.0, 0.13], vec![0.13, 0.0]];
        let spec = pr_spec(&cb, &kij);
        let z = [0.8, 0.2];
        let direct = multiplicative_pressure(&spec, 310.0, 3000.0, &z, Point::Bubble, 1e-10, 200)
            .ok()
            .filter(|sp| {
                !is_trivial(&spec, 310.0, sp.var, &z, &sp.incipient, Point::Bubble).unwrap()
            });
        assert!(direct.is_none(), "the multiplicative update now solves it");
        let nw = newton_pressure(&spec, 310.0, &z, Point::Bubble, 1e-10, 200)
            .expect("Newton on ln P declined");
        let reference =
            solve_pressure_multiplicative(&spec, 310.0, &z, Point::Bubble, 1e-10, 200).unwrap();
        assert!(
            (nw.var / reference.var - 1.0).abs() < 1e-6,
            "Newton {} vs inverse-secant {}",
            nw.var,
            reference.var
        );
        assert!(nw.var > 6000.0 && nw.var < 6400.0, "P = {}", nw.var);
    }

    /// Pressure solve: the full (n+1) step must give the SS-only answer in
    /// fewer K evaluations where SS is slow.
    #[test]
    fn full_newton_pressure_agrees_with_ss_only_and_is_shorter_near_critical() {
        let (me, _) = near_critical_pairs();
        let spec = pr_spec(&me, &[]);
        let z = [0.5, 0.5];
        let run = |allow_full: bool| {
            let p0 = wilson_pressure_guess(&spec, 240.0, &z, Point::Bubble);
            let k0 = wilson_k_init(&spec, 240.0, p0);
            saturation_iteration(
                &spec,
                Free::Pressure,
                240.0,
                &z,
                Point::Bubble,
                p0,
                &k0,
                1e-10,
                500,
                allow_full,
            )
            .expect("pressure solve declined")
        };
        let (full, n_full) = run(true);
        let (ss, n_ss) = run(false);
        assert!(
            (full.var / ss.var - 1.0).abs() < 1e-7,
            "{} vs {}",
            full.var,
            ss.var
        );
        assert!(n_full < n_ss, "full {n_full} evaluations vs SS-only {n_ss}");
    }

    /// A model pair with no ∂ln K/∂P (virial vapor) must still solve, through
    /// the multiplicative fallback.
    #[test]
    fn pressure_solve_falls_back_where_newton_has_no_derivative() {
        let comps = methanol_water();
        let aij = vec![vec![0.0, 0.5853], vec![0.3458, 0.0]];
        let spec = SystemSpec {
            vapor: VaporModel::Virial,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            aij: &aij,
            ..rks(&comps)
        };
        let z = [0.4, 0.6];
        assert!(newton_pressure(&spec, 340.0, &z, Point::Bubble, 1e-10, 200).is_none());
        let a = solve_pressure(&spec, 340.0, &z, Point::Bubble, 1e-10, 200).unwrap();
        let b = solve_pressure_multiplicative(&spec, 340.0, &z, Point::Bubble, 1e-10, 200).unwrap();
        assert_eq!(a.var, b.var);
    }

    /// γ-φ dew point: with ∂ln γ/∂n available the full step runs, agrees with
    /// the SS-only answer, and takes no more K evaluations.
    #[test]
    fn gamma_phi_dew_point_takes_the_full_newton_step() {
        let mw = methanol_water();
        let aij = vec![vec![0.0, 0.5853], vec![0.3458, 0.0]];
        let spec = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            aij: &aij,
            ..rks(&mw)
        };
        let z = [0.4, 0.6];
        let g = incipient_ln_phi_jacobian(&spec, 350.0, 101.325, &z, Point::Dew)
            .expect("an activity liquid exposes its composition Jacobian");
        // ln γ is intensive: Σⱼ xⱼ·Gᵢⱼ = 0 for every i.
        for row in &g {
            let s: f64 = row.iter().zip(&z).map(|(a, b)| a * b).sum();
            assert!(s.abs() < 1e-12, "row sum {s}");
        }
        let ans = newton_temperature(&spec, 101.325, &z, Point::Dew, 1e-10, 200).unwrap();
        let k_near: Vec<f64> = ans.k.iter().map(|k| k * 1.05).collect();
        let (full, ss) = full_vs_ss(&spec, 101.325, &z, Point::Dew, ans.var + 0.5, &k_near);
        assert!(full <= ss, "full {full} evaluations vs SS-only {ss}");
    }

    /// Evidence harness for `OPTIMIZATION_PLAN_PART1.md` §16: γ-φ dew
    /// temperatures with and without the full Newton step. Run with
    /// `cargo test --release -p vle-thermo --lib harness_gamma_phi_dew -- --ignored --nocapture`.
    #[test]
    #[ignore = "measurement harness; prints a table"]
    fn harness_gamma_phi_dew() {
        use std::time::Instant;
        let mw = methanol_water();
        let a_vl = vec![vec![0.0, 0.5853], vec![0.3458, 0.0]];
        let a_nrtl = vec![vec![0.0, -1100.0], vec![3500.0, 0.0]];
        let alpha = vec![vec![0.0, 0.3], vec![0.3, 0.0]];
        let van_laar = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            aij: &a_vl,
            ..rks(&mw)
        };
        let nrtl = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::Nrtl),
            aij: &a_nrtl,
            alpha: &alpha,
            ..rks(&mw)
        };
        println!(
            "{:<22} {:>6} {:>9} {:>9} {:>9} {:>9}",
            "system / start", "y1", "ev full", "ev SS", "µs full", "µs SS"
        );
        for (label, spec) in [("van Laar", &van_laar), ("NRTL", &nrtl)] {
            for y1 in [0.1, 0.4, 0.7, 0.9] {
                let z = [y1, 1.0 - y1];
                let ans = newton_temperature(spec, 101.325, &z, Point::Dew, 1e-10, 200).unwrap();
                let t_cold = wilson_temperature(spec, 101.325, &z, Point::Dew).unwrap();
                let k_cold = wilson_k_init(spec, t_cold, 101.325);
                let k_warm: Vec<f64> = ans.k.iter().map(|k| k * 1.05).collect();
                for (start, t0, k0) in [("cold", t_cold, &k_cold), ("warm", ans.var + 0.5, &k_warm)]
                {
                    let run = |full: bool| {
                        let reps = 2000;
                        let mut evals = 0;
                        let clock = Instant::now();
                        for _ in 0..reps {
                            evals = saturation_iteration(
                                spec,
                                Free::Temperature,
                                101.325,
                                &z,
                                Point::Dew,
                                t0,
                                k0,
                                1e-10,
                                200,
                                full,
                            )
                            .unwrap()
                            .1;
                        }
                        (evals, clock.elapsed().as_secs_f64() * 1e6 / reps as f64)
                    };
                    let (ef, tf) = run(true);
                    let (es, ts) = run(false);
                    println!(
                        "{:<22} {:>6} {:>9} {:>9} {:>9.2} {:>9.2}",
                        format!("{label} {start}"),
                        y1,
                        ef,
                        es,
                        tf,
                        ts
                    );
                }
            }
        }
    }

    /// Evidence harness for `OPTIMIZATION_PLAN_PART1.md` §16–§17 — not a
    /// pass/fail test. Run with
    /// `cargo test --release -p vle-thermo --lib harness_saturation -- --ignored --nocapture`.
    #[test]
    #[ignore = "measurement harness; prints a table"]
    fn harness_saturation_pressure_newton() {
        use std::time::Instant;
        let (me, cb) = near_critical_pairs();
        let kij_cb = vec![vec![0.0, 0.13], vec![0.13, 0.0]];
        let bh = [
            comp("n-butane", 425.12, 3796.0, 0.200),
            comp("n-heptane", 540.2, 2740.0, 0.350),
        ];
        let mp = [
            comp("methane", 190.564, 4599.0, 0.0115),
            comp("n-pentane", 469.7, 3370.0, 0.252),
        ];
        let cases: Vec<(&str, SystemSpec, f64, f64)> = vec![
            ("methane/ethane PR", pr_spec(&me, &[]), 130.0, 300.0),
            (
                "CO2/n-butane PR k=0.13",
                pr_spec(&cb, &kij_cb),
                230.0,
                420.0,
            ),
            ("n-butane/n-heptane RKS", rks(&bh), 280.0, 540.0),
            ("methane/n-pentane PR", pr_spec(&mp, &[]), 150.0, 460.0),
        ];
        println!(
            "{:<26} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>9} {:>9}",
            "system/point", "pts", "agree", "differ", "nw+", "mu+", "none", "new µs", "old µs"
        );
        for (label, spec, t_lo, t_hi) in &cases {
            for point in [Point::Bubble, Point::Dew] {
                let (mut pts, mut agree, mut differ, mut nw_only, mut mu_only, mut none) =
                    (0, 0, 0, 0, 0, 0);
                let (mut t_new, mut t_old) = (0.0, 0.0);
                for i in 0..60 {
                    let t = t_lo + (t_hi - t_lo) * i as f64 / 59.0;
                    for j in 1..10 {
                        let x1 = j as f64 / 10.0;
                        let z = [x1, 1.0 - x1];
                        pts += 1;
                        let reps = 20;
                        let start = Instant::now();
                        let mut new = None;
                        for _ in 0..reps {
                            new = solve_pressure(spec, t, &z, point, 1e-10, 200).ok();
                        }
                        t_new += start.elapsed().as_secs_f64() * 1e6 / reps as f64;
                        let start = Instant::now();
                        let mut old = None;
                        for _ in 0..reps {
                            old =
                                solve_pressure_multiplicative(spec, t, &z, point, 1e-10, 200).ok();
                        }
                        t_old += start.elapsed().as_secs_f64() * 1e6 / reps as f64;
                        match (&new, &old) {
                            (Some(a), Some(b)) if (a.var / b.var - 1.0).abs() < 1e-6 => agree += 1,
                            (Some(a), Some(b)) => {
                                differ += 1;
                                println!(
                                    "    differ {label} {point:?} T={t:.2} x1={x1}: new {} old {}",
                                    a.var, b.var
                                );
                            }
                            (Some(_), None) => nw_only += 1,
                            (None, Some(_)) => mu_only += 1,
                            (None, None) => none += 1,
                        }
                    }
                }
                println!(
                    "{:<26} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>9.2} {:>9.2}",
                    format!("{label} {point:?}"),
                    pts,
                    agree,
                    differ,
                    nw_only,
                    mu_only,
                    none,
                    t_new / pts as f64,
                    t_old / pts as f64
                );
            }
        }
    }
}
