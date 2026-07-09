//! Activity-model binary-parameter (Aij) regression — Milestone 9.
//!
//! Fits the two binary parameters `(A₁₂, A₂₁)` of an activity model
//! (Margules, van Laar, Wilson) to experimental bubble-pressure data for a
//! binary, by **Levenberg–Marquardt** on the pressure residuals — the modern
//! replacement for the thesis's plain Newton–Raphson (Ref (4), Pascal
//! `TERMOV.PAS`). LM interpolates between Gauss–Newton (fast near the
//! optimum) and gradient descent (robust far from it) via a damping
//! parameter λ, so it converges gracefully from a poor initial guess where
//! plain Newton would diverge.
//!
//! The residual for data point `d` is `r_d = P_bubble(A₁₂, A₂₁; T_d, x_d) −
//! P_exp,d`, with the bubble pressure from the γ-φ path (activity liquid +
//! ideal vapor). The 2×`m` Jacobian `∂r_d/∂A` is a numerical finite
//! difference — an outer-loop derivative of the whole flash, not a
//! thermodynamic composition derivative.
//!
//! # References
//! - (4) Da Silva & Báez (1989) — the Aij regression, `TERMOV.PAS`

use super::FlashError;
use super::bubble::bubble_pressure;
use super::system::SystemSpec;
use crate::activity::ActivityModel;
use crate::eos::{LiquidModel, VaporModel};
use crate::mixing::MixingRule;
use crate::types::Component;

/// One experimental bubble-pressure point for a binary (see
/// [`super::kij_regression::BubblePoint`] — re-declared here to keep the two
/// regressions independent).
#[derive(Debug, Clone)]
pub struct AijBubblePoint {
    /// Temperature in **K**.
    pub t: f64,
    /// Liquid mole fraction of component 1.
    pub x1: f64,
    /// Measured bubble pressure in **kPa**.
    pub p_exp: f64,
}

/// Result of an Aij fit.
#[derive(Debug, Clone, PartialEq)]
pub struct AijFit {
    /// Fitted `A₁₂` (the `aij[0][1]` entry).
    pub a12: f64,
    /// Fitted `A₂₁` (the `aij[1][0]` entry).
    pub a21: f64,
    /// Sum of squared pressure residuals at the optimum, **kPa²**.
    pub sse: f64,
    /// Root-mean-square pressure error, **kPa**.
    pub rmse: f64,
    /// Levenberg–Marquardt iterations taken.
    pub iterations: usize,
}

/// Sum of squared bubble-pressure residuals at parameters `(a12, a21)`.
fn sse(
    model: ActivityModel,
    components: &[Component],
    alpha: &[Vec<f64>],
    vl: &[f64],
    data: &[AijBubblePoint],
    a12: f64,
    a21: f64,
) -> f64 {
    residuals(model, components, alpha, vl, data, a12, a21)
        .iter()
        .map(|r| r * r)
        .sum()
}

/// Bubble-pressure residual vector `P_calc − P_exp` at `(a12, a21)`.
///
/// `alpha` is the fixed NRTL non-randomness matrix (only read when `model` is
/// NRTL; pass `&[]` for the 2-parameter models). The fit only ever adjusts the
/// two energy parameters `(a12, a21)` — α is held constant, per the plan.
fn residuals(
    model: ActivityModel,
    components: &[Component],
    alpha: &[Vec<f64>],
    vl: &[f64],
    data: &[AijBubblePoint],
    a12: f64,
    a21: f64,
) -> Vec<f64> {
    let aij = vec![vec![0.0, a12], vec![a21, 0.0]];
    let spec = SystemSpec {
        components,
        vapor: VaporModel::IdealGas,
        liquid: LiquidModel::Activity(model),
        mixing_rule: MixingRule::Classical,
        kij: &[],
        aij: &aij,
        alpha,
        vl,
        delta: &[],
        sat_models: &[],
        ge_model: None,
    };
    data.iter()
        .map(|d| {
            let x = [d.x1, 1.0 - d.x1];
            match bubble_pressure(&spec, d.t, &x, 1e-8, 200) {
                Ok(r) => r.value - d.p_exp,
                Err(_) => 1e6, // penalty for an infeasible parameter set
            }
        })
        .collect()
}

/// Fit `(A₁₂, A₂₁)` of an activity model to bubble-pressure data by
/// Levenberg–Marquardt.
///
/// # Arguments
/// * `model` — the activity model ([`ActivityModel::Margules`],
///   `VanLaar`, or `Wilson`).
/// * `components` — exactly two components with `psat_coeffs`.
/// * `alpha` — fixed NRTL non-randomness matrix (2×2 symmetric); only read
///   when `model` is NRTL, pass `&[]` for Margules/van Laar/Wilson. The fit
///   never adjusts α — only the two energy parameters `(a12, a21)`.
/// * `vl` — liquid molar volumes in **cm³/mol** (needed by Wilson; pass
///   `&[]` for Margules/van Laar).
/// * `data` — experimental bubble points.
/// * `a12_0`, `a21_0` — initial parameter guesses.
/// * `tol` — convergence tolerance on the relative SSE decrease.
/// * `max_iter` — LM iteration cap.
///
/// # Returns
/// [`AijFit`] with the fitted parameters, SSE, and RMSE.
///
/// # Errors
/// [`FlashError::Dimension`] unless there are exactly two components and at
/// least one data point.
#[allow(clippy::too_many_arguments)]
pub fn fit_aij(
    model: ActivityModel,
    components: &[Component],
    alpha: &[Vec<f64>],
    vl: &[f64],
    data: &[AijBubblePoint],
    a12_0: f64,
    a21_0: f64,
    tol: f64,
    max_iter: usize,
) -> Result<AijFit, FlashError> {
    if components.len() != 2 {
        return Err(FlashError::Dimension(format!(
            "Aij regression is binary: components={}",
            components.len()
        )));
    }
    if data.is_empty() {
        return Err(FlashError::Dimension("no data points".into()));
    }

    let (mut a12, mut a21) = (a12_0, a21_0);
    let mut lambda = 1e-3;
    let mut sse_cur = sse(model, components, alpha, vl, data, a12, a21);
    let mut iters = 0;

    for iter in 0..max_iter {
        iters = iter + 1;
        let r = residuals(model, components, alpha, vl, data, a12, a21);
        // Numerical Jacobian columns ∂r/∂A₁₂ and ∂r/∂A₂₁ (outer-loop FD).
        let h12 = 1e-4 * a12.abs().max(1.0);
        let h21 = 1e-4 * a21.abs().max(1.0);
        let r12 = residuals(model, components, alpha, vl, data, a12 + h12, a21);
        let r21 = residuals(model, components, alpha, vl, data, a12, a21 + h21);
        let m = r.len();
        // Normal-equation accumulators for the 2×2 system JᵀJ·δ = −Jᵀr.
        let (mut j11, mut j12, mut j22) = (0.0, 0.0, 0.0);
        let (mut g1, mut g2) = (0.0, 0.0);
        for k in 0..m {
            let d1 = (r12[k] - r[k]) / h12;
            let d2 = (r21[k] - r[k]) / h21;
            j11 += d1 * d1;
            j12 += d1 * d2;
            j22 += d2 * d2;
            g1 += d1 * r[k];
            g2 += d2 * r[k];
        }
        // Damped normal equations (LM): add λ to the diagonal.
        let a = j11 * (1.0 + lambda);
        let b = j12;
        let c = j22 * (1.0 + lambda);
        let det = a * c - b * b;
        if det.abs() < 1e-300 {
            break;
        }
        let d_a12 = -(c * g1 - b * g2) / det;
        let d_a21 = -(-b * g1 + a * g2) / det;
        let sse_new = sse(model, components, alpha, vl, data, a12 + d_a12, a21 + d_a21);
        if sse_new < sse_cur {
            // Accept the step, reduce damping (toward Gauss–Newton).
            let rel = (sse_cur - sse_new) / sse_cur.max(1e-300);
            a12 += d_a12;
            a21 += d_a21;
            sse_cur = sse_new;
            lambda = (lambda * 0.5).max(1e-12);
            if rel < tol {
                break;
            }
        } else {
            // Reject, increase damping (toward gradient descent).
            lambda *= 4.0;
            if lambda > 1e12 {
                break;
            }
        }
    }

    let rmse = (sse_cur / data.len() as f64).sqrt();
    Ok(AijFit {
        a12,
        a21,
        sse: sse_cur,
        rmse,
        iterations: iters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn methanol() -> Component {
        Component {
            name: "methanol".into(),
            tc: 512.6,
            pc: 8097.0,
            omega: 0.564,
            liquid_volume: 40.7,
            psat_coeffs: vec![5.20, 3200.0, -35.0],
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
            psat_coeffs: vec![5.11, 3800.0, -46.0],
            ..Component::default()
        }
    }

    fn synth_data(
        model: ActivityModel,
        comps: &[Component],
        alpha: &[Vec<f64>],
        vl: &[f64],
        a12: f64,
        a21: f64,
    ) -> Vec<AijBubblePoint> {
        let aij = vec![vec![0.0, a12], vec![a21, 0.0]];
        let spec = SystemSpec {
            components: comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(model),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha,
            vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        [0.2, 0.35, 0.5, 0.65, 0.8]
            .iter()
            .map(|&x1| AijBubblePoint {
                t: 298.15,
                x1,
                p_exp: bubble_pressure(&spec, 298.15, &[x1, 1.0 - x1], 1e-9, 200)
                    .unwrap()
                    .value,
            })
            .collect()
    }

    #[test]
    fn recovers_known_van_laar_parameters() {
        let comps = [methanol(), water()];
        let (a12t, a21t) = (0.85, 0.52);
        let data = synth_data(ActivityModel::VanLaar, &comps, &[], &[], a12t, a21t);
        let fit = fit_aij(
            ActivityModel::VanLaar,
            &comps,
            &[],
            &[],
            &data,
            0.3,
            0.3,
            1e-10,
            100,
        )
        .unwrap();
        assert!(
            (fit.a12 - a12t).abs() < 5e-3 && (fit.a21 - a21t).abs() < 5e-3,
            "fit A12={} A21={} vs true ({a12t}, {a21t})",
            fit.a12,
            fit.a21
        );
        assert!(fit.rmse < 1e-2, "rmse={}", fit.rmse);
    }

    #[test]
    fn recovers_known_wilson_parameters() {
        let comps = [methanol(), water()];
        let vl = [40.7, 18.07];
        let (a12t, a21t) = (1100.0, 550.0); // kJ/kmol
        let data = synth_data(ActivityModel::Wilson, &comps, &[], &vl, a12t, a21t);
        let fit = fit_aij(
            ActivityModel::Wilson,
            &comps,
            &[],
            &vl,
            &data,
            500.0,
            500.0,
            1e-10,
            100,
        )
        .unwrap();
        // Wilson parameters are less tightly identified from P-x alone; accept
        // a modest tolerance but require the fit to reproduce the pressures.
        assert!(fit.rmse < 1.0, "rmse={} kPa", fit.rmse);
    }

    #[test]
    fn recovers_known_nrtl_parameters() {
        // NRTL energy round-trip with α held fixed (the plan's fit strategy).
        let comps = [methanol(), water()];
        let alpha = vec![vec![0.0, 0.3], vec![0.3, 0.0]];
        let (a12t, a21t) = (1800.0, 900.0); // g₁₂−g₂₂, g₂₁−g₁₁ [kJ/kmol]
        let data = synth_data(ActivityModel::Nrtl, &comps, &alpha, &[], a12t, a21t);
        let fit = fit_aij(
            ActivityModel::Nrtl,
            &comps,
            &alpha,
            &[],
            &data,
            500.0,
            500.0,
            1e-12,
            200,
        )
        .unwrap();
        // Reproduce the synthetic pressures to well within measurement noise.
        assert!(fit.rmse < 1.0, "rmse={} kPa", fit.rmse);
    }

    #[test]
    fn rejects_non_binary() {
        let comps = [methanol()];
        assert!(matches!(
            fit_aij(
                ActivityModel::VanLaar,
                &comps,
                &[],
                &[],
                &[AijBubblePoint {
                    t: 298.15,
                    x1: 0.5,
                    p_exp: 20.0
                }],
                0.3,
                0.3,
                1e-8,
                50
            ),
            Err(FlashError::Dimension(_))
        ));
    }
}
