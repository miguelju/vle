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
//! - **Temperature** ([`solve_temperature`]) — **Newton on `1/T`** with the
//!   exact `∂ln Kᵢ/∂T` from [`k_values_with_derivs`], updating the incipient
//!   composition by one successive-substitution step per Newton step (see
//!   [`newton_temperature`]). If that fails — an unsupported model pair, a
//!   non-finite step, no convergence, or a collapse onto the trivial solution
//!   `K ≡ 1` — it falls back to bisection on the pressure solver
//!   ([`bisect_temperature`]), which is ~250× slower but converges wherever a
//!   saturation pressure exists.
//!
//! What this is **not**: the fully simultaneous `(N+1)`-variable Newton on
//! `{ln K, ln T}` of `MODERNIZATION_PLAN.md` §K. The Newton step here ignores
//! `∂Kᵢ/∂(incipient composition)`, so the composition converges linearly
//! (≈ 8 iterations on a 10-component SRK bubble point) rather than
//! quadratically. The full simultaneous form already exists for phase-envelope
//! tracing (`envelope.rs`) and is the remaining refinement here.

use super::FlashError;
use super::init::wilson_k_values;
use super::system::{SystemSpec, k_values, k_values_with_derivs};

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
pub(crate) fn solve_pressure(
    spec: &SystemSpec,
    t: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    let mut p = wilson_pressure_guess(spec, t, known, point).max(1e-6);
    let mut k = wilson_k_init(spec, t, p);
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
/// 1. evaluate `Kᵢ` and the exact `∂ln Kᵢ/∂T` ([`k_values_with_derivs`]);
/// 2. form `wᵢ = Kᵢxᵢ` (bubble) or `wᵢ = yᵢ/Kᵢ` (dew) and `S = Σwᵢ`, so
///    `∂ln S/∂T = ±Σ wᵢ·∂ln Kᵢ/∂T / S` (`+` bubble, `−` dew);
/// 3. replace the incipient composition by `wᵢ/S` (one successive-substitution
///    step);
/// 4. take the Newton step `Δu = −ln S / (∂ln S/∂u)`, with
///    `∂/∂u = −T²·∂/∂T`, limited to [`MAX_REL_STEP`].
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
    let n = spec.n();
    let mut t = wilson_temperature(spec, p, known, point)?;
    // Seed the incipient phase from Wilson K at the Wilson temperature.
    let mut incipient = vec![0.0; n];
    {
        let kw = wilson_k_init(spec, t, p);
        let mut s = 0.0;
        for i in 0..n {
            incipient[i] = match point {
                Point::Bubble => kw[i] * known[i],
                Point::Dew => known[i] / kw[i],
            };
            s += incipient[i];
        }
        incipient.iter_mut().for_each(|v| *v /= s);
    }

    for _ in 0..max_iter.max(1) {
        // (x, y) ordering for the K evaluation: the known phase is the
        // liquid for a bubble point and the vapor for a dew point.
        let (x, y) = match point {
            Point::Bubble => (known, incipient.as_slice()),
            Point::Dew => (incipient.as_slice(), known),
        };
        // `.ok()?` turns any FlashError (including `Unsupported`) into a
        // `None` return — the signal to fall back to bisection.
        let d = k_values_with_derivs(spec, t, p, x, y).ok()?;

        // `wᵢ` before normalization: Kᵢxᵢ (bubble) or yᵢ/Kᵢ (dew).
        let weight = |k: f64, z: f64| match point {
            Point::Bubble => k * z,
            Point::Dew => z / k,
        };
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

        let mut step = 0.0_f64;
        for ((inc, &k), &z) in incipient.iter_mut().zip(&d.k).zip(known) {
            let new = weight(k, z) / s;
            step = step.max((new - *inc).abs());
            *inc = new;
        }

        if ln_s.abs() < tol && step < tol {
            let max_ln_k = d.k.iter().fold(0.0_f64, |m, k| m.max(k.ln().abs()));
            if max_ln_k < TRIVIAL_LN_K {
                return None;
            }
            return Some(SatPoint {
                var: t,
                incipient,
                k: d.k,
            });
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
    let (t_start, t_end, steps) = (50.0, 1000.0, 96);
    let mut prev: Option<(f64, f64)> = None;
    let mut bracket: Option<(f64, f64, f64)> = None; // (lo, hi, h_lo)
    for i in 0..=steps {
        let t = t_start + (t_end - t_start) * i as f64 / steps as f64;
        let ht = match h(t) {
            Some(v) => v,
            None => {
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
        // The scan already proved h is defined at both ends; within a physical
        // saturation bracket the pressure solve converges at the midpoint too.
        // A failure here is unexpected — surface it rather than guess.
        let ht = h(t_star).ok_or(FlashError::Thermo("P_sat(mid) failed".into()))?;
        if ht.abs() < tol || (hi - lo) < 1e-9 {
            break;
        }
        if ht * h_lo > 0.0 {
            lo = t_star;
            h_lo = ht;
        } else {
            hi = t_star;
        }
        if iter + 1 == max_iter {
            return Err(FlashError::NoConvergence {
                what: "saturation temperature",
                iters: max_iter,
                residual: ht.abs(),
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
}
