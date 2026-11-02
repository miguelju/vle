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
//! - **Pressure** ([`solve_pressure`]) — the multiplicative update
//!   `P ← P·S` (bubble) / `P ← P/S` (dew) around an inner successive-
//!   substitution loop on K. The update is Newton on `ln P` with the
//!   Raoult-law slope `∂ln S/∂ln P ≈ −1` (bubble), so it needs no derivative
//!   and already converges in tens of µs.
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
use super::system::{SystemSpec, k_values, k_values_with_t_derivs};
use crate::eos::{LiquidModel, PhaseId, VaporModel};
use crate::mixture::{d_ln_phi_d_n, z_mix};

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

/// Solve for the saturation **pressure** at fixed `t` (bubble-P or dew-P).
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
/// Runs the `P ← P·S` iteration from the guess and accepts a result that is
/// not the trivial solution ([`is_trivial`]); otherwise — or for an unusable
/// guess — it runs the cold [`solve_pressure`], trivial-solution recovery
/// included. Same equation, same tolerance: the guess changes the cost, never
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
/// composition dependence of K that moves during a saturation-temperature
/// solve (the known phase is fixed). Returns `None` when the model pair does
/// not expose it, which keeps that solve on the one-SS-step update.
///
/// - Bubble (incipient vapor): ideal gas ⇒ zero; cubic ⇒ the mixture's exact
///   Jacobian on the vapor root; virial ⇒ `None`.
/// - Dew (incipient liquid): ideal solution ⇒ zero (γ ≡ 1 and the
///   Psat/φˢᵃᵗ/Poynting terms are composition-free); cubic ⇒ the exact
///   Jacobian on the liquid root; activity models and the refinery
///   correlations ⇒ `None` (no ∂ln γ/∂x is exposed).
///
/// `t` in **K**, `p` in **kPa absolute**; the result is dimensionless.
fn incipient_ln_phi_jacobian(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    inc: &[f64],
    point: Point,
) -> Option<Vec<Vec<f64>>> {
    let n = inc.len();
    let zeros = || Some(vec![vec![0.0; n]; n]);
    match point {
        Point::Bubble => match spec.vapor {
            VaporModel::IdealGas => zeros(),
            VaporModel::Cubic(eos) => {
                d_ln_phi_d_n(&spec.mixture_spec(eos), t, p, inc, PhaseId::Vapor).ok()
            }
            VaporModel::Virial => None,
        },
        Point::Dew => match spec.liquid {
            LiquidModel::IdealSolution => zeros(),
            LiquidModel::Cubic(eos) => {
                d_ln_phi_d_n(&spec.mixture_spec(eos), t, p, inc, PhaseId::Liquid).ok()
            }
            _ => None,
        },
    }
}

/// One full Newton step on the (n+1) unknowns `{ln K₁…ln Kₙ, ln T}`
/// (`MODERNIZATION_PLAN.md` §K; Michelsen & Mollerup (26) ch. 12).
///
/// With the incipient phase derived from the unknowns — bubble
/// `yᵢ = Kᵢxᵢ/Σ Kx`, dew `xᵢ = (yᵢ/Kᵢ)/Σ y/K` — the equations are
/// ```text
///   Fᵢ    = ln Kᵢ − ln Kᵢᵐᵒᵈᵉˡ(T, x, y) = 0      i = 1…n
///   Fₙ₊₁ = ln Σ Kᵢxᵢ  (bubble)   or   ln Σ yᵢ/Kᵢ  (dew)  = 0
/// ```
/// and, writing `Gᵢⱼ = ∂ln φ̂ᵢ/∂nⱼ` of the incipient phase at its normalized
/// composition `ŵ` (ln φ̂ is intensive, so the mole-number derivative at
/// total S is `G/S`, and `∂nⱼ/∂ln Kⱼ = ±nⱼ`):
/// ```text
///   ∂Fᵢ/∂ln Kⱼ   = δᵢⱼ + Gᵢⱼ·ŵⱼ                  (both points)
///   ∂Fᵢ/∂ln T    = −T·∂ln Kᵢᵐᵒᵈᵉˡ/∂T
///   ∂Fₙ₊₁/∂ln Kⱼ = +ŵⱼ (bubble), −ŵⱼ (dew);   ∂Fₙ₊₁/∂ln T = 0
/// ```
/// Returns `(Δln K, Δln T)`, already scaled by the step limits, or `None`
/// if the system is singular or the step is not finite.
#[allow(clippy::too_many_arguments)]
fn full_newton_step(
    point: Point,
    t: f64,
    ln_k: &[f64],
    model_ln_k: &[f64],
    d_ln_k_d_t: &[f64],
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
        (true, false) => -t * d_ln_k_d_t[r],
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
    if dlt.abs() > MAX_REL_STEP {
        scale = scale.min(MAX_REL_STEP / dlt.abs());
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
    temperature_iteration(spec, p, known, point, t0, k0, tol, max_iter, true).map(|(sp, _)| sp)
}

/// The loop behind [`newton_temperature_from`], also returning the number of
/// K evaluations it took. `allow_full = false` pins it to the one-SS-step
/// update — the v0.18.0 algorithm — which the tests use as the reference the
/// full Newton step must agree with and beat.
#[allow(clippy::too_many_arguments)]
fn temperature_iteration(
    spec: &SystemSpec,
    p: f64,
    known: &[f64],
    point: Point,
    t0: f64,
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

    let mut t = t0;
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
        // (x, y) ordering for the K evaluation: the known phase is the
        // liquid for a bubble point and the vapor for a dew point.
        let (x, y) = match point {
            Point::Bubble => (known, incipient.as_slice()),
            Point::Dew => (incipient.as_slice(), known),
        };
        // `.ok()?` turns any FlashError (including `Unsupported`) into a
        // `None` return — the signal to fall back to bisection.
        let d = k_values_with_t_derivs(spec, t, p, x, y).ok()?;

        let (mut s, mut ds_dt) = (0.0, 0.0);
        // `zip` walks the three slices in lockstep — no index bounds checks.
        for ((&k, &z), &dlnk) in d.k.iter().zip(known).zip(&d.d_ln_k_d_t) {
            let w = weight(k, z);
            s += w;
            ds_dt += w * dlnk;
        }
        // ∂ln S/∂T: K rising with T raises the bubble sum and lowers the dew sum.
        let dlns_dt = match point {
            Point::Bubble => ds_dt / s,
            Point::Dew => -ds_dt / s,
        };
        let ln_s = s.ln();
        if !ln_s.is_finite() || !dlns_dt.is_finite() {
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
                    var: t,
                    incipient: ss_incipient,
                    k: d.k,
                },
                iter + 1,
            ));
        }

        // A saturation sum must rise with T at a bubble point and fall at a
        // dew point; the opposite sign means we are off the physical branch.
        let wrong_sign = match point {
            Point::Bubble => dlns_dt <= 0.0,
            Point::Dew => dlns_dt >= 0.0,
        };
        if wrong_sign {
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
            let taken = (residual < last_full_residual)
                .then(|| incipient_ln_phi_jacobian(spec, t, p, &incipient, point))
                .flatten()
                .and_then(|g| {
                    full_newton_step(
                        point,
                        t,
                        &ln_k,
                        &model_ln_k,
                        &d.d_ln_k_d_t,
                        &incipient,
                        ln_sum,
                        &g,
                    )
                });
            if let Some((dlk, dlt)) = taken {
                for (v, dv) in ln_k.iter_mut().zip(&dlk) {
                    *v += dv;
                }
                let t_new = t * dlt.exp();
                let k_new: Vec<f64> = ln_k.iter().map(|v| v.exp()).collect();
                if let (true, Some((inc, _))) = (t_new.is_finite() && t_new > 0.0, implied(&k_new))
                {
                    t = t_new;
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

        // --- One SS step on composition + Newton on u = 1/T. ---
        incipient = ss_incipient;
        ln_k = d.k.iter().map(|k| k.ln()).collect();
        let u = 1.0 / t;
        let dlns_du = -t * t * dlns_dt;
        let du = (-ln_s / dlns_du).clamp(-MAX_REL_STEP * u, MAX_REL_STEP * u);
        t = 1.0 / (u + du);
        if !t.is_finite() || t <= 0.0 {
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

    /// Run [`temperature_iteration`] with and without the full Newton step
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
        let (full, n_full) = temperature_iteration(spec, p, known, point, t0, k0, tol, 200, true)
            .unwrap_or_else(|| panic!("full Newton declined {point:?} at p={p}"));
        let (ss, n_ss) = temperature_iteration(spec, p, known, point, t0, k0, tol, 200, false)
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

    /// A model pair with no incipient composition Jacobian (γ-φ dew point:
    /// no ∂ln γ/∂x is exposed) must run the SS-only update unchanged —
    /// identical iteration count and answer.
    #[test]
    fn full_newton_is_skipped_where_no_composition_jacobian_exists() {
        let mw = methanol_water();
        let aij = vec![vec![0.0, 0.5853], vec![0.3458, 0.0]];
        let spec = SystemSpec {
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            aij: &aij,
            ..rks(&mw)
        };
        let z = [0.4, 0.6];
        assert!(incipient_ln_phi_jacobian(&spec, 350.0, 101.325, &z, Point::Dew).is_none());
        let ans = newton_temperature(&spec, 101.325, &z, Point::Dew, 1e-10, 200).unwrap();
        let k_near: Vec<f64> = ans.k.iter().map(|k| k * 1.05).collect();
        let (full, ss) = full_vs_ss(&spec, 101.325, &z, Point::Dew, ans.var + 0.5, &k_near);
        assert_eq!(full, ss);
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
}
