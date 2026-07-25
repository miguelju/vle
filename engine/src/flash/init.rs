//! K-value initialization for flash calculations (Milestone 9, §I).
//!
//! Every flash, bubble/dew, and stability calculation needs a starting
//! estimate of the equilibrium ratios `Kᵢ = yᵢ/xᵢ`. The **Wilson
//! correlation** (Ref (29)) gives a good first guess from nothing but the
//! critical constants and acentric factor — data every [`Component`]
//! already carries:
//!
//! ```text
//!   Kᵢ = (Pc,ᵢ / P) · exp[ 5.373·(1 + ωᵢ)·(1 − Tc,ᵢ/T) ]
//! ```
//!
//! This supersedes the thesis's Raoult-only initialization for the φ-φ
//! (EOS-both-phases) path. The γ-φ (activity-model) path still seeds from
//! Raoult's law `Kᵢ = Psat,ᵢ/P` — see [`raoult_k_values`].
//!
//! Wilson is only a *starting* estimate: the flash iterations (successive
//! substitution → Newton) refine it to the true equilibrium K. Its value
//! is being cheap, always finite, and close enough that the stability
//! analysis (§I) and the flash's first Rachford-Rice solve land in the
//! right basin.
//!
//! # References
//! - (29) Wilson, G. M. (1969) — the K-value correlation.

use crate::saturation::{SatPressureModel, psat};
use crate::types::Component;

/// The Wilson-correlation constant `5.373 = ln(10)·(7/3)·... ` — the
/// standard literature value (Wilson 1969 / Michelsen & Mollerup).
const WILSON_C: f64 = 5.373;

/// Wilson K-value estimate for one component.
///
/// # Arguments
/// * `comp` — component (reads `tc` in **K**, `pc` in **kPa**, `omega`).
/// * `t` — Temperature in **K**.
/// * `p` — Pressure in **kPa absolute**.
///
/// # Returns
/// `Kᵢ = yᵢ/xᵢ` estimate, **dimensionless**.
pub fn wilson_k(comp: &Component, t: f64, p: f64) -> f64 {
    wilson_ln_k(comp, t, p).exp()
}

/// **ln** of the Wilson K-value estimate — `ln(Pc/P) + C·(1+ω)·(1 − Tc/T)`.
///
/// The isothermal flash iterates on `ln K` throughout (Part 1 §2 of the
/// performance audit), so it seeds from this directly rather than taking the
/// logarithm of [`wilson_k`]'s exponential. [`wilson_k`] is defined in terms of
/// this function, so the two cannot drift apart.
///
/// # Arguments
/// As [`wilson_k`]: `comp` supplies `tc` in **K** and `pc` in **kPa** plus the
/// acentric factor; `t` in **K**, `p` in **kPa absolute**.
///
/// # Returns
/// `ln Kᵢ`, **dimensionless**.
pub fn wilson_ln_k(comp: &Component, t: f64, p: f64) -> f64 {
    (comp.pc / p).ln() + WILSON_C * (1.0 + comp.omega) * (1.0 - comp.tc / t)
}

/// Wilson K-value estimates for every component in a mixture.
///
/// Arguments as in [`wilson_k`]; returns one K per component in input order.
pub fn wilson_k_values(components: &[Component], t: f64, p: f64) -> Vec<f64> {
    components.iter().map(|c| wilson_k(c, t, p)).collect()
}

/// Raoult's-law K-value estimates `Kᵢ = Psat,ᵢ(T)/P` — the γ-φ path seed.
///
/// Uses each component's saturation model to get Psat,ᵢ. Falls back to the
/// Wilson estimate for any component whose Psat evaluation fails (missing
/// correlation coefficients), so the return is always finite and usable.
///
/// # Arguments
/// * `components` — component list.
/// * `sat_models` — per-component saturation model (length = components, or
///   empty to use each component's own `sat_model` field).
/// * `t` — Temperature in **K**; `p` — Pressure in **kPa absolute**.
///
/// # Returns
/// One `Kᵢ` per component, **dimensionless**.
pub fn raoult_k_values(
    components: &[Component],
    sat_models: &[SatPressureModel],
    t: f64,
    p: f64,
) -> Vec<f64> {
    components
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let model = sat_models.get(i).copied().unwrap_or(c.sat_model);
            match psat(model, c, t) {
                Ok(ps) => ps / p,
                Err(_) => wilson_k(c, t, p),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn methane() -> Component {
        Component {
            name: "methane".into(),
            tc: 190.564,
            pc: 4599.0,
            omega: 0.0115,
            ..Component::default()
        }
    }

    fn n_butane() -> Component {
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            ..Component::default()
        }
    }

    #[test]
    fn wilson_k_light_heavy_ordering() {
        // At a fixed (T, P) the lighter/more-volatile component (methane)
        // must have the larger K (prefers the vapor). At 300 K, 2000 kPa.
        let km = wilson_k(&methane(), 300.0, 2000.0);
        let kb = wilson_k(&n_butane(), 300.0, 2000.0);
        assert!(km > kb, "methane K={km} should exceed butane K={kb}");
        assert!(km > 1.0, "methane (supercritical here) should be K>1: {km}");
    }

    #[test]
    fn wilson_k_at_critical_point_is_pc_over_p() {
        // At T = Tc the exponential term is exp(0) = 1, so Kᵢ = Pc/P exactly.
        let c = n_butane();
        let k = wilson_k(&c, c.tc, 1000.0);
        assert!((k - c.pc / 1000.0).abs() < 1e-12);
    }

    #[test]
    fn wilson_k_decreases_with_pressure() {
        // Kᵢ ∝ 1/P — doubling P halves the Wilson K.
        let c = n_butane();
        let k1 = wilson_k(&c, 350.0, 1000.0);
        let k2 = wilson_k(&c, 350.0, 2000.0);
        assert!((k1 / k2 - 2.0).abs() < 1e-12);
    }

    #[test]
    fn wilson_k_values_vector_matches_scalar() {
        let comps = [methane(), n_butane()];
        let ks = wilson_k_values(&comps, 300.0, 2000.0);
        assert_eq!(ks.len(), 2);
        assert!((ks[0] - wilson_k(&comps[0], 300.0, 2000.0)).abs() < 1e-15);
        assert!((ks[1] - wilson_k(&comps[1], 300.0, 2000.0)).abs() < 1e-15);
    }

    #[test]
    fn raoult_falls_back_to_wilson_without_sat_data() {
        // No Antoine coefficients → psat fails → Wilson fallback (finite).
        let comps = [methane()];
        let ks = raoult_k_values(&comps, &[], 300.0, 2000.0);
        assert!(ks[0].is_finite() && ks[0] > 0.0);
        assert!((ks[0] - wilson_k(&comps[0], 300.0, 2000.0)).abs() < 1e-12);
    }
}
