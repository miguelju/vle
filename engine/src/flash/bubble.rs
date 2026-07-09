//! Bubble-point calculations — Milestone 9, §K.
//!
//! The bubble point is where a liquid of composition `x` first boils: an
//! infinitesimal bubble of vapor `y` appears. Given the liquid composition
//! and one of `{T, P}`, solve for the other plus the incipient vapor `y`,
//! from the saturation condition `Σ Kᵢ·xᵢ = 1` (see [`super::incipient`]).
//!
//! - [`bubble_pressure`] — given `(T, x)`, find `(P, y)`.
//! - [`bubble_temperature`] — given `(P, x)`, find `(T, y)`.

use super::FlashError;
use super::incipient::{Point, solve_pressure, solve_temperature};
use super::system::SystemSpec;

/// A converged bubble/dew point.
#[derive(Debug, Clone, PartialEq)]
pub struct SaturationResult {
    /// The solved-for variable: pressure in **kPa** (for `*_pressure`) or
    /// temperature in **K** (for `*_temperature`).
    pub value: f64,
    /// The incipient-phase composition (vapor `y` for a bubble point, liquid
    /// `x` for a dew point), length N, sums to 1.
    pub incipient: Vec<f64>,
    /// Converged equilibrium ratios Kᵢ.
    pub k: Vec<f64>,
}

/// Bubble-point **pressure** at fixed temperature.
///
/// # Arguments
/// * `spec` — the mixture model.
/// * `t` — Temperature in **K**.
/// * `x` — liquid mole fractions (length N, sum to 1).
/// * `tol` — tolerance on `|Σ Kᵢxᵢ − 1|`.
/// * `max_iter` — outer iteration cap.
///
/// # Returns
/// [`SaturationResult`] with `value` = bubble pressure in **kPa** and
/// `incipient` = the incipient vapor `y`.
///
/// # Errors
/// [`FlashError`] on dimension mismatch, thermodynamic failure, or
/// non-convergence.
pub fn bubble_pressure(
    spec: &SystemSpec,
    t: f64,
    x: &[f64],
    tol: f64,
    max_iter: usize,
) -> Result<SaturationResult, FlashError> {
    check_len(spec, x)?;
    let sp = solve_pressure(spec, t, x, Point::Bubble, tol, max_iter)?;
    Ok(SaturationResult {
        value: sp.var,
        incipient: sp.incipient,
        k: sp.k,
    })
}

/// Bubble-point **temperature** at fixed pressure.
///
/// # Arguments
/// * `spec` — the mixture model.
/// * `p` — Pressure in **kPa absolute**.
/// * `x` — liquid mole fractions (length N, sum to 1).
/// * `tol` — tolerance on `|ln Σ Kᵢxᵢ|`.
/// * `max_iter` — bisection iteration cap.
///
/// # Returns
/// [`SaturationResult`] with `value` = bubble temperature in **K** and
/// `incipient` = the incipient vapor `y`.
///
/// # Errors
/// [`FlashError`] on dimension mismatch, thermodynamic failure, or if no
/// temperature bracket is found.
pub fn bubble_temperature(
    spec: &SystemSpec,
    p: f64,
    x: &[f64],
    tol: f64,
    max_iter: usize,
) -> Result<SaturationResult, FlashError> {
    check_len(spec, x)?;
    let sp = solve_temperature(spec, p, x, Point::Bubble, tol, max_iter)?;
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
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        }
    }

    #[test]
    fn bubble_pressure_satisfies_saturation_condition() {
        let comps = [n_butane(), n_heptane()];
        let spec = rks(&comps);
        let x = [0.4, 0.6];
        let res = bubble_pressure(&spec, 400.0, &x, 1e-10, 200).unwrap();
        // Σ Kᵢxᵢ = 1 and y = Kx normalized.
        let k = k_values(&spec, 400.0, res.value, &x, &res.incipient).unwrap();
        let s: f64 = (0..2).map(|i| k[i] * x[i]).sum();
        assert!((s - 1.0).abs() < 1e-7, "Σ Kx = {s}");
        assert!((res.incipient.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        assert!(res.value > 0.0);
    }

    #[test]
    fn bubble_temperature_satisfies_saturation_condition() {
        let comps = [n_butane(), n_heptane()];
        let spec = rks(&comps);
        let x = [0.4, 0.6];
        let res = bubble_temperature(&spec, 1000.0, &x, 1e-9, 200).unwrap();
        let k = k_values(&spec, res.value, 1000.0, &x, &res.incipient).unwrap();
        let s: f64 = (0..2).map(|i| k[i] * x[i]).sum();
        assert!((s - 1.0).abs() < 1e-5, "Σ Kx = {s} at T={}", res.value);
        assert!(res.value > 250.0 && res.value < 600.0, "T={}", res.value);
    }

    #[test]
    fn bubble_temperature_close_boiling_phi_phi() {
        // Regression: a *close-boiling* φ-φ pair (benzene/cyclohexane, both
        // boiling ~353 K at 1 atm, relative volatility ≈ 1.02) has its true
        // bubble T sitting inside the K≈1 band. The old S(T)=1 objective
        // filtered that band out as "trivial" and died mid-bracket
        // (`g(mid) failed`); the pressure-inversion solver must converge across
        // the whole composition range. See `flash/incipient.rs`.
        let benzene = Component {
            name: "benzene".into(),
            tc: 562.02,
            pc: 4907.277,
            omega: 0.211,
            tb: 353.219,
            ..Component::default()
        };
        let cyclohexane = Component {
            name: "cyclohexane".into(),
            tc: 553.6,
            pc: 4080.5,
            omega: 0.2096,
            tb: 353.865,
            ..Component::default()
        };
        let comps = [benzene, cyclohexane];
        let spec = rks(&comps);
        // Sweep the whole composition range at 1 atm — every point must solve
        // and land near the components' shared ~353 K boiling point.
        for i in 0..=10 {
            let x1 = 0.001 + 0.998 * i as f64 / 10.0;
            let x = [x1, 1.0 - x1];
            let res = bubble_temperature(&spec, 101.325, &x, 1e-9, 200)
                .unwrap_or_else(|e| panic!("x1={x1}: {e}"));
            let k = k_values(&spec, res.value, 101.325, &x, &res.incipient).unwrap();
            let s: f64 = (0..2).map(|j| k[j] * x[j]).sum();
            assert!((s - 1.0).abs() < 1e-4, "x1={x1}: Σ Kx = {s}");
            assert!(
                res.value > 345.0 && res.value < 362.0,
                "x1={x1}: bubble T {} K off the ~353 K band",
                res.value
            );
        }
    }

    #[test]
    fn bubble_pressure_between_pure_component_vapor_pressures() {
        // A bubble pressure of a binary must lie between the two pure-
        // component saturation pressures at the same T (ideal-ish bound).
        let comps = [n_butane(), n_heptane()];
        let spec = rks(&comps);
        let t = 380.0;
        let res = bubble_pressure(&spec, t, &[0.5, 0.5], 1e-10, 200).unwrap();
        let p1 = crate::saturation::psat(comps[0].sat_model, &comps[0], t).unwrap();
        let p2 = crate::saturation::psat(comps[1].sat_model, &comps[1], t).unwrap();
        let (lo, hi) = (p1.min(p2), p1.max(p2));
        assert!(
            res.value > lo * 0.5 && res.value < hi * 2.0,
            "bubble P {} outside plausible band [{lo}, {hi}]",
            res.value
        );
    }

    #[test]
    fn bubble_pressure_gamma_phi_van_laar() {
        // γ-φ bubble pressure with a van Laar liquid + ideal vapor.
        let a = Component {
            name: "methanol".into(),
            tc: 512.6,
            pc: 8097.0,
            omega: 0.564,
            liquid_volume: 40.7,
            psat_coeffs: vec![5.20, 3200.0, -35.0],
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
        let aij = vec![vec![0.0, 0.5], vec![0.9, 0.0]];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let x = [0.5, 0.5];
        let res = bubble_pressure(&spec, 298.15, &x, 1e-10, 200).unwrap();
        let k = k_values(&spec, 298.15, res.value, &x, &res.incipient).unwrap();
        let s: f64 = (0..2).map(|i| k[i] * x[i]).sum();
        assert!((s - 1.0).abs() < 1e-7, "Σ Kx = {s}");
    }
}
