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
//! variable (T or P) to drive `S → 1`. The Wilson correlation seeds both the
//! initial K-values and a good first pressure guess. Full simultaneous
//! log-variable Newton on `{ln K, ln T/P}` is the planned refinement; the
//! successive-substitution core here already converges the Chapter IV cases.

use super::FlashError;
use super::init::wilson_k_values;
use super::system::{SystemSpec, k_values};

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
/// Rather than root-find the sum condition `S(T) = 1` directly, we invert the
/// robust saturation-**pressure** solver: `P_sat(T)` is smooth and strictly
/// increasing in T, so we bracket-bisect for the `T*` where `P_sat(T*) = p`.
/// This deliberately avoids the trivial-K filtering a direct `S(T)` objective
/// needs — that filter rejects the real root for *close-boiling* φ-φ systems
/// (relative volatility α≈1) whose equilibrium K genuinely sit near 1, which
/// is exactly where the true bubble/dew T lives. [`solve_pressure`] carries
/// no such filter and converges wherever a saturation pressure exists.
///
/// The wide bracketing scan uses the cheap closed-form Wilson pressure
/// estimate (no inner iteration); the bisection refines with the accurate
/// solver. A final incipient solve at `(T*, p)` returns the composition + K.
pub(crate) fn solve_temperature(
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
