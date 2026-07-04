//! Dew-point calculations — Milestone 9, §K.
//!
//! The dew point is where a vapor of composition `y` first condenses: an
//! infinitesimal drop of liquid `x` appears. Given the vapor composition and
//! one of `{T, P}`, solve for the other plus the incipient liquid `x`, from
//! the saturation condition `Σ yᵢ/Kᵢ = 1` (see [`super::incipient`]).
//!
//! - [`dew_pressure`] — given `(T, y)`, find `(P, x)`.
//! - [`dew_temperature`] — given `(P, y)`, find `(T, x)`.

use super::FlashError;
use super::bubble::SaturationResult;
use super::incipient::{Point, solve_pressure, solve_temperature};
use super::system::SystemSpec;

/// Dew-point **pressure** at fixed temperature.
///
/// # Arguments
/// * `spec` — the mixture model.
/// * `t` — Temperature in **K**.
/// * `y` — vapor mole fractions (length N, sum to 1).
/// * `tol` — tolerance on `|Σ yᵢ/Kᵢ − 1|`.
/// * `max_iter` — outer iteration cap.
///
/// # Returns
/// [`SaturationResult`] with `value` = dew pressure in **kPa** and
/// `incipient` = the incipient liquid `x`.
///
/// # Errors
/// [`FlashError`] on dimension mismatch, thermodynamic failure, or
/// non-convergence.
pub fn dew_pressure(
    spec: &SystemSpec,
    t: f64,
    y: &[f64],
    tol: f64,
    max_iter: usize,
) -> Result<SaturationResult, FlashError> {
    check_len(spec, y)?;
    let sp = solve_pressure(spec, t, y, Point::Dew, tol, max_iter)?;
    Ok(SaturationResult {
        value: sp.var,
        incipient: sp.incipient,
        k: sp.k,
    })
}

/// Dew-point **temperature** at fixed pressure.
///
/// # Arguments
/// * `spec` — the mixture model.
/// * `p` — Pressure in **kPa absolute**.
/// * `y` — vapor mole fractions (length N, sum to 1).
/// * `tol` — tolerance on `|ln Σ yᵢ/Kᵢ|`.
/// * `max_iter` — bisection iteration cap.
///
/// # Returns
/// [`SaturationResult`] with `value` = dew temperature in **K** and
/// `incipient` = the incipient liquid `x`.
///
/// # Errors
/// [`FlashError`] on dimension mismatch, thermodynamic failure, or if no
/// temperature bracket is found.
pub fn dew_temperature(
    spec: &SystemSpec,
    p: f64,
    y: &[f64],
    tol: f64,
    max_iter: usize,
) -> Result<SaturationResult, FlashError> {
    check_len(spec, y)?;
    let sp = solve_temperature(spec, p, y, Point::Dew, tol, max_iter)?;
    Ok(SaturationResult {
        value: sp.var,
        incipient: sp.incipient,
        k: sp.k,
    })
}

fn check_len(spec: &SystemSpec, phase: &[f64]) -> Result<(), FlashError> {
    if phase.len() != spec.n() {
        return Err(FlashError::Dimension(format!(
            "components={}, composition={}",
            spec.n(),
            phase.len()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::ActivityModel;
    use crate::eos::{CubicEos, LiquidModel, VaporModel};
    use crate::flash::bubble::bubble_pressure;
    use crate::flash::system::k_values;
    use crate::mixing::MixingRule;
    use crate::types::Component;

    fn n_butane() -> Component {
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            tb: 272.65,
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
            tb: 371.6,
            psat_coeffs: vec![4.02, 2911.0, -56.0],
            ..Component::default()
        }
    }

    fn rks(components: &[Component]) -> SystemSpec<'_> {
        SystemSpec {
            components,
            vapor: VaporModel::Cubic(CubicEos::RKS1972),
            liquid: LiquidModel::Cubic(CubicEos::RKS1972),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        }
    }

    #[test]
    fn dew_pressure_satisfies_saturation_condition() {
        let comps = [n_butane(), n_heptane()];
        let spec = rks(&comps);
        let y = [0.6, 0.4];
        let res = dew_pressure(&spec, 400.0, &y, 1e-10, 200).unwrap();
        let k = k_values(&spec, 400.0, res.value, &res.incipient, &y).unwrap();
        let s: f64 = (0..2).map(|i| y[i] / k[i]).sum();
        assert!((s - 1.0).abs() < 1e-7, "Σ y/K = {s}");
        assert!((res.incipient.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn dew_temperature_satisfies_saturation_condition() {
        let comps = [n_butane(), n_heptane()];
        let spec = rks(&comps);
        let y = [0.6, 0.4];
        let res = dew_temperature(&spec, 1000.0, &y, 1e-9, 200).unwrap();
        let k = k_values(&spec, res.value, 1000.0, &res.incipient, &y).unwrap();
        let s: f64 = (0..2).map(|i| y[i] / k[i]).sum();
        assert!((s - 1.0).abs() < 1e-5, "Σ y/K = {s} at T={}", res.value);
    }

    #[test]
    fn dew_pressure_at_or_below_bubble_pressure() {
        // For the same overall composition and T, the dew pressure (vapor)
        // must be ≤ the bubble pressure (liquid) — the two-phase envelope.
        let comps = [n_butane(), n_heptane()];
        let spec = rks(&comps);
        let t = 400.0;
        let z = [0.5, 0.5];
        let bub = bubble_pressure(&spec, t, &z, 1e-10, 200).unwrap();
        let dew = dew_pressure(&spec, t, &z, 1e-10, 200).unwrap();
        assert!(
            dew.value <= bub.value + 1e-6,
            "dew P {} should be ≤ bubble P {}",
            dew.value,
            bub.value
        );
    }

    #[test]
    fn dew_temperature_gamma_phi_wilson() {
        // γ-φ dew temperature with a Wilson liquid + ideal vapor
        // (2-propanol/water-like, Chapter IV case 4 shape).
        let a = Component {
            name: "2-propanol".into(),
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
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let y = [0.5, 0.5];
        let res = dew_temperature(&spec, 101.325, &y, 1e-8, 200).unwrap();
        let k = k_values(&spec, res.value, 101.325, &res.incipient, &y).unwrap();
        let s: f64 = (0..2).map(|i| y[i] / k[i]).sum();
        assert!((s - 1.0).abs() < 1e-4, "Σ y/K = {s} at T={}", res.value);
    }
}
