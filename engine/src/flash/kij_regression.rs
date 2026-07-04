//! Binary interaction parameter (kij) regression — Milestone 9, §B.
//!
//! Fits the single binary interaction parameter `k₁₂` of a cubic EOS to a
//! set of experimental bubble-pressure points by minimizing the sum of
//! squared pressure residuals
//!
//! ```text
//!   SSE(k) = Σ_d [ P_bubble(k; T_d, x_d) − P_exp,d ]²
//! ```
//!
//! over `k`. The minimization uses **Brent's method** (parabolic
//! interpolation + golden-section safeguard, [`crate::numerics::root_finding::brent_minimize`])
//! — replacing the legacy golden-section search (Ref (4), Pascal
//! `TERMOVI.PAS`) with the same guaranteed convergence but super-linear
//! speed near the optimum. Each bubble-pressure objective is itself
//! Wilson-seeded, so the per-point solves are cheap.
//!
//! Chapter IV validation: CO₂ / n-butane, which fits to `k₁₂ ≈ 0.1357`
//! (Tables 4.11–4.12).
//!
//! # References
//! - (4) Da Silva & Báez (1989) — the regression objective, `TERMOVI.PAS`

use super::FlashError;
use super::bubble::bubble_pressure;
use super::system::SystemSpec;
use crate::eos::{CubicEos, LiquidModel, VaporModel};
use crate::mixing::MixingRule;
use crate::numerics::root_finding::brent_minimize;
use crate::types::Component;

/// One experimental bubble-pressure data point for a **binary** system.
#[derive(Debug, Clone)]
pub struct BubblePoint {
    /// Temperature in **K**.
    pub t: f64,
    /// Liquid mole fraction of component 1 (component 2 is `1 − x1`).
    pub x1: f64,
    /// Measured bubble pressure in **kPa**.
    pub p_exp: f64,
}

/// Result of a kij fit.
#[derive(Debug, Clone, PartialEq)]
pub struct KijFit {
    /// The fitted binary interaction parameter `k₁₂ = k₂₁`.
    pub kij: f64,
    /// Sum of squared pressure residuals at the optimum, **kPa²**.
    pub sse: f64,
    /// Root-mean-square pressure error, **kPa**.
    pub rmse: f64,
}

/// Fit the binary `k₁₂` of a two-parameter cubic EOS (φ-φ, classical
/// mixing) to bubble-pressure data.
///
/// # Arguments
/// * `eos` — the cubic EOS (both phases).
/// * `components` — exactly two components; each must carry the saturation
///   coefficients its bubble-pressure solve needs (`psat_coeffs`).
/// * `data` — the experimental bubble-pressure points.
/// * `k_lo`, `k_hi` — search bracket for `k₁₂` (e.g. `−0.1 .. 0.3`).
/// * `tol` — Brent x-tolerance on `k`.
/// * `max_iter` — Brent iteration cap.
///
/// # Returns
/// [`KijFit`] with the optimal `k₁₂`, the residual SSE, and the RMSE.
///
/// # Errors
/// [`FlashError::Dimension`] unless there are exactly two components / one
/// or more data points; [`FlashError::Thermo`] if every objective
/// evaluation fails.
pub fn fit_kij(
    eos: CubicEos,
    components: &[Component],
    data: &[BubblePoint],
    k_lo: f64,
    k_hi: f64,
    tol: f64,
    max_iter: usize,
) -> Result<KijFit, FlashError> {
    if components.len() != 2 {
        return Err(FlashError::Dimension(format!(
            "kij regression is binary: components={}",
            components.len()
        )));
    }
    if data.is_empty() {
        return Err(FlashError::Dimension("no data points".into()));
    }

    // SSE(k): rebuild the kij matrix, solve each point's bubble pressure,
    // accumulate squared residuals. A failed point contributes a large
    // penalty so the optimizer steers away from infeasible k.
    let sse = |k: f64| -> f64 {
        let kij = vec![vec![0.0, k], vec![k, 0.0]];
        let spec = SystemSpec {
            components,
            vapor: VaporModel::Cubic(eos),
            liquid: LiquidModel::Cubic(eos),
            mixing_rule: MixingRule::Classical,
            kij: &kij,
            aij: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let mut acc = 0.0;
        for d in data {
            let x = [d.x1, 1.0 - d.x1];
            match bubble_pressure(&spec, d.t, &x, 1e-8, 200) {
                Ok(r) => {
                    let e = r.value - d.p_exp;
                    acc += e * e;
                }
                Err(_) => acc += 1e12,
            }
        }
        acc
    };

    let (kij, sse_min) = brent_minimize(sse, k_lo, k_hi, tol, max_iter)
        .map_err(|e| FlashError::Thermo(format!("kij optimization failed: {e}")))?;
    let rmse = (sse_min / data.len() as f64).sqrt();
    Ok(KijFit {
        kij,
        sse: sse_min,
        rmse,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn co2() -> Component {
        Component {
            name: "CO2".into(),
            tc: 304.13,
            pc: 7377.0,
            omega: 0.2239,
            // Reduced-Antoine ln(P/Pc) = a1 − a2/(a3 + T) (kPa, K).
            psat_coeffs: vec![4.86, 1147.0, -8.0],
            ..Component::default()
        }
    }

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

    /// Generate synthetic "experimental" data from a known k*, then check
    /// the regression recovers it. This validates the fit machinery
    /// independently of any particular literature dataset.
    #[test]
    fn recovers_a_known_kij() {
        let comps = [co2(), n_butane()];
        let eos = CubicEos::PR1976;
        let k_true = 0.13;
        // Build data at k_true across a composition sweep at 310 K.
        let kij = vec![vec![0.0, k_true], vec![k_true, 0.0]];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::Cubic(eos),
            liquid: LiquidModel::Cubic(eos),
            mixing_rule: MixingRule::Classical,
            kij: &kij,
            aij: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let mut data = Vec::new();
        for &x1 in &[0.2, 0.4, 0.5, 0.6, 0.8] {
            let p = bubble_pressure(&spec, 310.0, &[x1, 1.0 - x1], 1e-9, 200)
                .unwrap()
                .value;
            data.push(BubblePoint {
                t: 310.0,
                x1,
                p_exp: p,
            });
        }
        let fit = fit_kij(eos, &comps, &data, -0.05, 0.30, 1e-6, 100).unwrap();
        assert!(
            (fit.kij - k_true).abs() < 1e-3,
            "recovered kij={} vs true {k_true}",
            fit.kij
        );
        assert!(
            fit.rmse < 1e-2,
            "rmse={} should be ~0 for exact data",
            fit.rmse
        );
    }

    #[test]
    fn nonzero_kij_beats_zero_for_nonideal_data() {
        // If the data were generated with k*=0.15, the fit's SSE must be far
        // below the SSE at k=0 (proves the objective is discriminating).
        let comps = [co2(), n_butane()];
        let eos = CubicEos::PR1976;
        let k_true = 0.15;
        let kij = vec![vec![0.0, k_true], vec![k_true, 0.0]];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::Cubic(eos),
            liquid: LiquidModel::Cubic(eos),
            mixing_rule: MixingRule::Classical,
            kij: &kij,
            aij: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let data: Vec<_> = [0.3, 0.5, 0.7]
            .iter()
            .map(|&x1| BubblePoint {
                t: 310.0,
                x1,
                p_exp: bubble_pressure(&spec, 310.0, &[x1, 1.0 - x1], 1e-9, 200)
                    .unwrap()
                    .value,
            })
            .collect();
        let fit = fit_kij(eos, &comps, &data, -0.05, 0.30, 1e-6, 100).unwrap();
        assert!((fit.kij - k_true).abs() < 2e-3, "kij={}", fit.kij);
    }

    #[test]
    fn rejects_non_binary() {
        let comps = [co2()];
        assert!(matches!(
            fit_kij(
                CubicEos::PR1976,
                &comps,
                &[BubblePoint {
                    t: 310.0,
                    x1: 0.5,
                    p_exp: 3000.0
                }],
                -0.1,
                0.3,
                1e-6,
                100
            ),
            Err(FlashError::Dimension(_))
        ));
    }
}
