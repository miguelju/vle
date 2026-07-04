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

/// `(S, non_trivial)` at `(t, p)` from a fresh Wilson K start — the
/// temperature-solve objective (the saturation condition is `S = 1`).
///
/// `non_trivial` is `false` when the converged K collapse to ≈1, which for a
/// φ-φ cubic system marks the single-root region where the "vapor" and
/// "liquid" roots coincide and `S = Σx = 1` *spuriously*. The scan skips
/// those points so the bisection can't latch onto that false crossing.
fn s_at(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    known: &[f64],
    point: Point,
) -> Result<(f64, bool), FlashError> {
    let mut k = wilson_k_init(spec, t, p);
    let (s, _) = incipient_sum(spec, t, p, known, point, &mut k, 40)?;
    let max_abs_ln_k = k.iter().map(|ki| ki.ln().abs()).fold(0.0_f64, f64::max);
    Ok((s, max_abs_ln_k > 0.02))
}

/// Solve for the saturation **temperature** at fixed `p` (bubble-T or dew-T).
///
/// `g(T) = ln S(T)` is monotone in T (increasing for a bubble point —
/// higher T ⇒ larger `Σ Kx`; decreasing for a dew point — larger `Σ y/K` at
/// low T). We scan a wide temperature range for a sign change of `g` (in
/// **either** direction), skipping the trivial single-root region, then
/// bisect — derivative-free and guaranteed to converge once bracketed. A
/// final incipient solve at `T*` returns the composition + K.
pub(crate) fn solve_temperature(
    spec: &SystemSpec,
    p: f64,
    known: &[f64],
    point: Point,
    tol: f64,
    max_iter: usize,
) -> Result<SatPoint, FlashError> {
    // Only a cubic (φ-φ) liquid exhibits the trivial single-root crossing;
    // for a γ-φ liquid every point is "non-trivial" so no point is skipped.
    let filter_trivial = matches!(spec.liquid, crate::eos::LiquidModel::Cubic(_));
    let g = |t: f64| -> Option<f64> {
        match s_at(spec, t, p, known, point) {
            Ok((s, nt)) if s.is_finite() && s > 0.0 && (nt || !filter_trivial) => Some(s.ln()),
            _ => None,
        }
    };
    let (t_start, t_end, steps) = (50.0, 1000.0, 96);
    let mut prev: Option<(f64, f64)> = None;
    let mut bracket: Option<(f64, f64)> = None;
    for i in 0..=steps {
        let t = t_start + (t_end - t_start) * i as f64 / steps as f64;
        let gt = match g(t) {
            Some(v) => v,
            None => {
                prev = None;
                continue;
            }
        };
        if let Some((tp, gp)) = prev {
            // Sign change in either direction brackets the root.
            if gp * gt <= 0.0 {
                bracket = Some((tp, t));
                break;
            }
        }
        prev = Some((t, gt));
    }
    let (mut lo, mut hi) = bracket.ok_or(FlashError::NoConvergence {
        what: "saturation temperature bracket",
        iters: steps,
        residual: f64::NAN,
    })?;
    // Direction-agnostic bisection: track the sign at `lo`.
    let mut g_lo = g(lo).ok_or(FlashError::Thermo("g(lo) failed".into()))?;
    let mut t_star = 0.5 * (lo + hi);
    for iter in 0..max_iter {
        t_star = 0.5 * (lo + hi);
        let gt = g(t_star).ok_or(FlashError::Thermo("g(mid) failed".into()))?;
        if gt.abs() < tol || (hi - lo) < 1e-9 {
            break;
        }
        if gt * g_lo > 0.0 {
            lo = t_star;
            g_lo = gt;
        } else {
            hi = t_star;
        }
        if iter + 1 == max_iter {
            return Err(FlashError::NoConvergence {
                what: "saturation temperature",
                iters: max_iter,
                residual: gt.abs(),
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
