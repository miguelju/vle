//! Activity coefficient models for liquid-phase non-ideality.
//!
//! Activity coefficients (γᵢ) quantify how much a component's behavior in a
//! liquid mixture deviates from ideal solution (Raoult's law). A value of γ = 1
//! means ideal behavior; γ > 1 indicates positive deviation (components "dislike"
//! each other); γ < 1 indicates negative deviation (favorable interactions).
//!
//! These models are used in the γ-φ approach to VLE, where the liquid phase is
//! described by activity coefficients and the vapor phase by an EOS. This is
//! preferred over the φ-φ (EOS-only) approach for highly non-ideal liquid
//! mixtures such as water + alcohol, where cubic EOS gives poor liquid predictions.
//!
//! All 5 models are identical in both legacy codebases (VB6 `clsActivityMulticomp.cls`
//! and Pascal `TERMOIII.PAS`). Each model requires binary interaction parameters
//! (Aij) fit to experimental VLE data.
//!
//! # The `aij` matrix convention
//!
//! Every multicomponent function below takes an N×N `aij` slice-of-slices whose
//! meaning depends on the model (this mirrors the legacy `CoefAIJ` matrix):
//!
//! - **Wilson** — `aij[i][j] = λᵢⱼ − λᵢᵢ`, an interaction *energy* in
//!   **kJ/kmol**. The diagonal is unused (Λᵢᵢ ≡ 1).
//! - **van Laar** — `aij[i][j]` are dimensionless, with `aij[i][i] = 0`.
//! - **Margules** — binary only: `aij[0][1] = A₁₂`, `aij[1][0] = A₂₁`,
//!   dimensionless.
//! - **Scatchard-Hildebrand** / **Ideal** — `aij` is ignored (Scatchard uses the
//!   solubility parameters `delta` and the liquid volumes `vl`).
//!
//! # Excess properties (eqs 2.44–2.46 of the research paper)
//!
//! `Gᴱ = RT Σᵢ xᵢ ln γᵢ`, `Hᴱ = −T² ∂(Gᴱ/T)/∂T`, `Sᴱ = (Hᴱ − Gᴱ)/T`. Per
//! CLAUDE.md the temperature derivative is **analytical**. For Wilson the only
//! temperature dependence is the Boltzmann factor in Λᵢⱼ, which differentiates to
//! a closed form; for the other models the legacy programs treat `Gᴱ` as
//! temperature-independent over the excess-property derivative (`Hᴱ = Gᴱ`,
//! `Sᴱ = 0`), and we reproduce that exactly.

/// Activity coefficient model for liquid-phase non-ideality.
///
/// Each model computes ln(γᵢ) from composition and binary parameters (Aij),
/// and provides analytical excess enthalpy HE (via dGE/dT) for enthalpy
/// departure calculations in adiabatic flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum ActivityModel {
    /// Ideal solution (γᵢ = 1 for all components at all compositions).
    /// No binary parameters needed. Used as a baseline or for nearly ideal mixtures.
    IdealSolution = 25,
    /// van Laar model. Two-suffix equation derived from van der Waals EOS.
    /// Parameters: A₁₂, A₂₁ (asymmetric). Good for simple non-polar mixtures.
    /// Cannot predict liquid-liquid immiscibility.
    VanLaar = 21,
    /// Wilson model. Local composition equation using molar volume ratios.
    /// Parameters: Λ₁₂, Λ₂₁ (related to energy differences). Good for
    /// polar/non-polar mixtures. Cannot predict liquid-liquid immiscibility.
    Wilson = 22,
    /// Scatchard-Hildebrand (regular solution theory). Based on solubility
    /// parameters (δᵢ) and liquid molar volumes (Vᵢᴸ). Semi-predictive —
    /// requires only pure-component data, no binary fitting needed.
    /// Limited to non-polar mixtures.
    ScatchardHildebrand = 23,
    /// Margules model. Two-suffix equation — the simplest empirical activity
    /// coefficient model. Parameters: A₁₂, A₂₁ (asymmetric). Useful for
    /// quick estimates but limited accuracy for strongly non-ideal systems.
    Margules = 24,
}

use crate::types::R_GAS;

/// Gas constant expressed in **cal/(mol·K)** (≈ 1.987), used by the
/// Scatchard-Hildebrand model where the solubility parameter is in
/// (cal/cm³)^0.5. `R_GAS · 0.23898` matches the legacy `R*0.23898` conversion.
const R_CAL: f64 = R_GAS * 0.23898;
/// Converts a cal/mol energy into the canonical **kJ/kmol** (= 1/0.23898).
const CAL_TO_KJ_PER_KMOL: f64 = 4.18445;

/// ln(γᵢ) for component `i` in a liquid mixture.
///
/// Ref (4): Da Silva & Báez (1989), `legacy/pascal/TERMOIII.PAS`; VB6
/// `legacy/vb6/clsActivityMulticomp.cls:74`. Formulas: research paper Table 2.3.
///
/// # Arguments
/// * `model` — the activity model.
/// * `i` — 0-based component index.
/// * `x` — mole fractions (length N).
/// * `aij` — N×N interaction matrix; see the module docs for the per-model
///   convention. May be empty for `IdealSolution`/`ScatchardHildebrand`.
/// * `vl` — liquid molar volumes Vᵢᴸ in **cm³/mol** (Wilson, Scatchard); may be
///   empty for the other models.
/// * `delta` — solubility parameters δᵢ in **(cal/cm³)^0.5** (Scatchard only).
/// * `temperature` — **K**.
///
/// # Returns
/// ln(γᵢ), dimensionless. γᵢ = exp(result).
pub fn ln_gamma(
    model: ActivityModel,
    i: usize,
    x: &[f64],
    aij: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let n = x.len();
    match model {
        ActivityModel::IdealSolution => 0.0,

        ActivityModel::ScatchardHildebrand => {
            // Volume-fraction-weighted average solubility parameter δ_mix.
            let v_tot: f64 = (0..n).map(|k| x[k] * vl[k]).sum();
            let delta_mix: f64 = (0..n).map(|k| x[k] * vl[k] * delta[k] / v_tot).sum();
            let d = delta[i] - delta_mix;
            // ln γᵢ = Vᵢ(δᵢ − δ_mix)² / (R_cal·T). R_cal keeps the cal-based δ²
            // consistent with the cm³/mol volume.
            vl[i] * d * d / (R_CAL * temperature)
        }

        ActivityModel::Margules => {
            // Two-suffix Margules — binary only (research paper Table 2.3).
            let (x1, x2) = (x[0], x[1]);
            let (a12, a21) = (aij[0][1], aij[1][0]);
            if i == 0 {
                x2 * x2 * (a12 + 2.0 * (a21 - a12) * x1)
            } else {
                x1 * x1 * (a21 + 2.0 * (a12 - a21) * x2)
            }
        }

        ActivityModel::VanLaar => {
            // Multicomponent van Laar (CoefAIJ diagonal = 0). For a binary this
            // reduces algebraically to the closed form in Table 2.3.
            let mut sum_ij = 0.0; // Σⱼ xⱼ Aᵢⱼ
            let mut sum_ji = 0.0; // Σⱼ xⱼ Aⱼᵢ
            for j in 0..n {
                sum_ij += x[j] * aij[i][j];
                sum_ji += x[j] * aij[j][i];
            }
            let denom = x[i] * sum_ij + (1.0 - x[i]) * sum_ji;
            if denom.abs() < f64::EPSILON {
                0.0
            } else {
                let ratio = sum_ji / denom;
                sum_ij * (1.0 - x[i]) * ratio * ratio
            }
        }

        ActivityModel::Wilson => {
            // ln γᵢ = 1 − ln(Σⱼ xⱼΛᵢⱼ) − Σₖ [xₖΛₖᵢ / Σⱼ(xⱼΛₖⱼ)].
            let denom_i: f64 = (0..n).map(|j| x[j] * wilson_lambda(i, j, aij, vl, temperature)).sum();
            let nested: f64 = (0..n)
                .map(|k| {
                    let dk: f64 =
                        (0..n).map(|j| x[j] * wilson_lambda(k, j, aij, vl, temperature)).sum();
                    x[k] * wilson_lambda(k, i, aij, vl, temperature) / dk
                })
                .sum();
            1.0 - denom_i.ln() - nested
        }
    }
}

/// Wilson coefficient Λᵢⱼ = (Vⱼᴸ/Vᵢᴸ)·exp(−aᵢⱼ/RT), with Λᵢᵢ ≡ 1.
///
/// `aij[i][j] = λᵢⱼ − λᵢᵢ` in **kJ/kmol**; `vl` in cm³/mol (the ratio is
/// dimensionless, so the volume units cancel); `t` in K.
fn wilson_lambda(i: usize, j: usize, aij: &[Vec<f64>], vl: &[f64], t: f64) -> f64 {
    if i == j {
        1.0
    } else {
        (vl[j] / vl[i]) * (-aij[i][j] / (R_GAS * t)).exp()
    }
}

/// Excess Gibbs energy Gᴱ = RT Σᵢ xᵢ ln γᵢ.
///
/// Research paper eq (2.44). Arguments as in [`ln_gamma`].
///
/// # Returns
/// Gᴱ in **kJ/kmol**.
pub fn excess_gibbs(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let n = x.len();
    let sum: f64 = (0..n)
        .map(|i| x[i] * ln_gamma(model, i, x, aij, vl, delta, temperature))
        .sum();
    R_GAS * temperature * sum
}

/// Excess enthalpy Hᴱ = −T²·∂(Gᴱ/T)/∂T, evaluated **analytically**.
///
/// Research paper eq (2.45). For Wilson the derivative runs through the
/// Boltzmann factor in Λᵢⱼ and gives a closed form; for Margules, van Laar and
/// Scatchard-Hildebrand the legacy treatment is Hᴱ = Gᴱ (the parameters carry
/// the model's temperature dependence implicitly), which this reproduces.
/// Ideal solutions have Hᴱ = 0.
///
/// # Returns
/// Hᴱ in **kJ/kmol**.
pub fn excess_enthalpy(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let n = x.len();
    match model {
        ActivityModel::IdealSolution => 0.0,

        // Hᴱ = Σⱼ xⱼ Vⱼ(δⱼ − δ_mix)²  (in cal/mol), converted to kJ/kmol.
        // This equals Gᴱ; SE is therefore 0.
        ActivityModel::ScatchardHildebrand => {
            let v_tot: f64 = (0..n).map(|k| x[k] * vl[k]).sum();
            let delta_mix: f64 = (0..n).map(|k| x[k] * vl[k] * delta[k] / v_tot).sum();
            let sum: f64 = (0..n)
                .map(|j| {
                    let d = delta[j] - delta_mix;
                    x[j] * vl[j] * d * d
                })
                .sum();
            sum * CAL_TO_KJ_PER_KMOL
        }

        // Legacy convention: Hᴱ = Gᴱ for these two (SE = 0).
        ActivityModel::Margules | ActivityModel::VanLaar => {
            excess_gibbs(model, x, aij, vl, delta, temperature)
        }

        // Wilson, analytical: Hᴱ = Σⱼ xⱼ·(Σₖ xₖ aⱼₖ Λⱼₖ)/(Σₖ xₖ Λⱼₖ), with the
        // k = j term dropped (aⱼⱼ = 0, Λⱼⱼ = 1). aⱼₖ is already in kJ/kmol.
        ActivityModel::Wilson => (0..n)
            .map(|j| {
                let mut up = 0.0;
                let mut down = 0.0;
                for k in 0..n {
                    let lam = wilson_lambda(j, k, aij, vl, temperature);
                    down += x[k] * lam;
                    if k != j {
                        up += x[k] * aij[j][k] * lam;
                    }
                }
                x[j] * up / down
            })
            .sum(),
    }
}

/// Excess entropy Sᴱ = (Hᴱ − Gᴱ)/T.
///
/// Research paper eq (2.46), rearranged from Gᴱ = Hᴱ − T·Sᴱ. Arguments as in
/// [`ln_gamma`].
///
/// # Returns
/// Sᴱ in **kJ/(kmol·K)**.
pub fn excess_entropy(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let he = excess_enthalpy(model, x, aij, vl, delta, temperature);
    let ge = excess_gibbs(model, x, aij, vl, delta, temperature);
    (he - ge) / temperature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminant_values_match_legacy() {
        assert_eq!(ActivityModel::VanLaar as i32, 21);
        assert_eq!(ActivityModel::Wilson as i32, 22);
        assert_eq!(ActivityModel::ScatchardHildebrand as i32, 23);
        assert_eq!(ActivityModel::Margules as i32, 24);
        assert_eq!(ActivityModel::IdealSolution as i32, 25);
    }

    // A zeroed N×N matrix helper.
    fn zeros(n: usize) -> Vec<Vec<f64>> {
        vec![vec![0.0; n]; n]
    }

    #[test]
    fn ideal_solution_is_unity_and_zero_excess() {
        let x = [0.4, 0.6];
        let aij = zeros(2);
        for i in 0..2 {
            assert_eq!(ln_gamma(ActivityModel::IdealSolution, i, &x, &aij, &[], &[], 300.0), 0.0);
        }
        assert_eq!(excess_gibbs(ActivityModel::IdealSolution, &x, &aij, &[], &[], 300.0), 0.0);
        assert_eq!(excess_enthalpy(ActivityModel::IdealSolution, &x, &aij, &[], &[], 300.0), 0.0);
    }

    #[test]
    fn margules_matches_table_2_3_closed_form() {
        // ln γ₁ = x₂²[A₁₂ + 2(A₂₁−A₁₂)x₁]; ln γ₂ = x₁²[A₂₁ + 2(A₁₂−A₂₁)x₂].
        let x = [0.3, 0.7];
        let mut aij = zeros(2);
        aij[0][1] = 0.5; // A₁₂
        aij[1][0] = 0.8; // A₂₁
        let g1 = ln_gamma(ActivityModel::Margules, 0, &x, &aij, &[], &[], 300.0);
        let g2 = ln_gamma(ActivityModel::Margules, 1, &x, &aij, &[], &[], 300.0);
        let e1 = x[1] * x[1] * (0.5 + 2.0 * (0.8 - 0.5) * x[0]);
        let e2 = x[0] * x[0] * (0.8 + 2.0 * (0.5 - 0.8) * x[1]);
        assert!((g1 - e1).abs() < 1e-12);
        assert!((g2 - e2).abs() < 1e-12);
    }

    #[test]
    fn van_laar_reduces_to_binary_closed_form() {
        // Table 2.3: ln γ₁ = A₁₂/[1 + (A₁₂x₁)/(A₂₁x₂)]².
        let x = [0.35, 0.65];
        let (a12, a21) = (1.2, 0.9);
        let mut aij = zeros(2);
        aij[0][1] = a12;
        aij[1][0] = a21;
        let g1 = ln_gamma(ActivityModel::VanLaar, 0, &x, &aij, &[], &[], 300.0);
        let g2 = ln_gamma(ActivityModel::VanLaar, 1, &x, &aij, &[], &[], 300.0);
        let r1 = 1.0 + (a12 * x[0]) / (a21 * x[1]);
        let r2 = 1.0 + (a21 * x[1]) / (a12 * x[0]);
        assert!((g1 - a12 / (r1 * r1)).abs() < 1e-12);
        assert!((g2 - a21 / (r2 * r2)).abs() < 1e-12);
    }

    #[test]
    fn wilson_gamma_goes_to_one_for_zero_interaction() {
        // With aᵢⱼ = 0 and equal volumes, Λᵢⱼ = 1 and ln γᵢ = 0.
        let x = [0.5, 0.5];
        let aij = zeros(2);
        let vl = [40.0, 40.0];
        for i in 0..2 {
            let g = ln_gamma(ActivityModel::Wilson, i, &x, &aij, &vl, &[], 320.0);
            assert!(g.abs() < 1e-12, "ln γ should vanish, got {g}");
        }
    }

    #[test]
    fn scatchard_gamma_unity_when_solubility_params_equal() {
        // If δ₁ = δ₂ then δ_mix = δ and every (δᵢ − δ_mix) = 0 → γ = 1.
        let x = [0.3, 0.7];
        let vl = [75.0, 110.0];
        let delta = [9.0, 9.0];
        for i in 0..2 {
            let g = ln_gamma(ActivityModel::ScatchardHildebrand, i, &x, &zeros(2), &vl, &delta, 300.0);
            assert!(g.abs() < 1e-12, "got {g}");
        }
    }

    #[test]
    fn gibbs_duhem_excess_gibbs_is_symmetric_endpoints() {
        // Gᴱ must vanish at the pure-component limits for every model.
        let aij = {
            let mut a = zeros(2);
            a[0][1] = 800.0;
            a[1][0] = 1200.0;
            a
        };
        let vl = [58.0, 92.0];
        let delta = [7.4, 9.2];
        for model in [
            ActivityModel::Margules,
            ActivityModel::VanLaar,
            ActivityModel::Wilson,
            ActivityModel::ScatchardHildebrand,
        ] {
            for x in [[1.0 - 1e-9, 1e-9], [1e-9, 1.0 - 1e-9]] {
                // Gᴱ → 0 like x_min as a component vanishes; with x_min = 1e-9 the
                // residual is ~1e-3 kJ/kmol, vs. the hundreds–thousands a broken
                // endpoint would leave — so a generous bound still catches bugs.
                let ge = excess_gibbs(model, &x, &aij, &vl, &delta, 313.15);
                assert!(ge.abs() < 1e-1, "{model:?}: Gᴱ at pure limit = {ge}");
            }
        }
    }

    #[test]
    fn wilson_excess_enthalpy_matches_numerical_oracle() {
        // Verify the analytical Hᴱ against −T²·d(Gᴱ/T)/dT computed by central
        // difference. This is the test-oracle pattern CLAUDE.md mandates: the
        // analytical form is the shipping code, the numerical form only checks it.
        let x = [0.4, 0.6];
        let mut aij = zeros(2);
        aij[0][1] = 1500.0; // λ₁₂ − λ₁₁  [kJ/kmol]
        aij[1][0] = 2600.0; // λ₂₁ − λ₂₂
        let vl = [74.0, 18.0]; // e.g. 2-propanol / water
        let t = 333.15;
        let h = 1e-2;
        let g_over_t = |tt: f64| excess_gibbs(ActivityModel::Wilson, &x, &aij, &vl, &[], tt) / tt;
        let d = (g_over_t(t + h) - g_over_t(t - h)) / (2.0 * h);
        let he_num = -t * t * d;
        let he_ana = excess_enthalpy(ActivityModel::Wilson, &x, &aij, &vl, &[], t);
        assert!(
            (he_ana - he_num).abs() < 1e-2 * he_num.abs().max(1.0),
            "analytical {he_ana} vs numerical {he_num}"
        );
    }

    #[test]
    fn margules_van_laar_excess_enthalpy_equals_gibbs() {
        // Legacy convention: Hᴱ = Gᴱ (so Sᴱ = 0) for these two models.
        let x = [0.45, 0.55];
        let mut aij = zeros(2);
        aij[0][1] = 0.7;
        aij[1][0] = 1.1;
        for model in [ActivityModel::Margules, ActivityModel::VanLaar] {
            let ge = excess_gibbs(model, &x, &aij, &[], &[], 298.15);
            let he = excess_enthalpy(model, &x, &aij, &[], &[], 298.15);
            let se = excess_entropy(model, &x, &aij, &[], &[], 298.15);
            assert!((he - ge).abs() < 1e-9);
            assert!(se.abs() < 1e-9);
        }
    }
}
