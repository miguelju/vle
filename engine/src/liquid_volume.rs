//! Saturated-liquid molar volume correlations.
//!
//! Several VLE calculations need the molar volume of the *liquid* phase Vᵢᴸ:
//!
//! - the **Poynting factor** Fᵢ = exp[Vᵢᴸ(P − Pᵢˢᵃᵗ)/(RT)] (already in
//!   [`crate::saturation`]),
//! - the **Wilson** activity model, whose Λᵢⱼ carry the volume ratio Vⱼᴸ/Vᵢᴸ,
//! - the **Scatchard-Hildebrand** activity model, whose volume fractions Φᵢ are
//!   built from xᵢVᵢᴸ.
//!
//! Cubic EOS predict liquid volumes poorly, so the legacy programs use dedicated
//! corresponding-states correlations instead. Two are provided, matching the two
//! `vlModel` cases in VB6 `clsActivityMulticomp.cls:vl` and the Pascal `vl` field
//! (Ref (4)):
//!
//! - **Rackett** (Spencer-Danner modification) — needs only the Rackett
//!   compressibility ZRA. Robust and the usual default.
//! - **Thomson / COSTALD** (Hankinson & Thomson, Ref (18)) — needs a
//!   characteristic volume V* and an SRK acentric factor; slightly more accurate
//!   near the critical region.
//!
//! # References
//! - (18) Hankinson, R. W.; Thomson, G. H. *AIChE J.* **1979**, *25*, 653 — COSTALD.
//! - (4) Da Silva & Báez (1989), `legacy/pascal/TERMOIII.PAS`;
//!   VB6 `legacy/vb6/clsActivityMulticomp.cls:291`.

use crate::types::{Component, R_GAS};

/// Which saturated-liquid-volume correlation to use for a component.
///
/// The integer discriminants match the legacy `TADiPvlModel` enum in VB6
/// (`clsActivityMulticomp.cls:21`): `Rackett = 1`, `Thomson = 2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum VolumeModel {
    /// Spencer-Danner modified Rackett equation. Uses [`Component::zra`].
    Rackett = 1,
    /// Thomson / COSTALD correlation (Ref (18)). Uses [`Component::liquid_volume`]
    /// as the characteristic volume V* and [`Component::omega_srk`].
    Thomson = 2,
}

// --- Thomson / COSTALD universal constants (Ref (18), VB6 vl case 2) ---
// Vᵣ⁽⁰⁾ polynomial in (1 − Tr):
const A: f64 = -1.528_16;
const B: f64 = 1.439_07;
const C: f64 = -0.814_46;
const D: f64 = 0.190_454;
// Vᵣ⁽δ⁾ rational function in Tr:
const E: f64 = -0.296_123;
const F: f64 = 0.386_914;
const G: f64 = -0.042_527_258;
const H: f64 = -0.048_064_5;

/// Saturated-liquid molar volume Vᴸ for one component at temperature `t`.
///
/// Ref (4): Da Silva & Báez (1989); VB6 `clsActivityMulticomp.cls:291`.
///
/// # Arguments
/// * `model` — [`VolumeModel::Rackett`] or [`VolumeModel::Thomson`].
/// * `comp` — component; reads `tc` (**K**), `pc` (**kPa**), `zra`,
///   `liquid_volume` (V*, **cm³/mol**), `omega_srk`.
/// * `t` — temperature in **K**.
///
/// # Returns
/// Liquid molar volume in **cm³/mol**.
///
/// The `1000.0 * R_GAS` factor converts `R·Tc/Pc` (m³/kmol with R in
/// kJ/(kmol·K) and Pc in kPa) into cm³/mol — the same factor [`crate::virial`]
/// uses for the EOS `b` parameter.
pub fn liquid_molar_volume(model: VolumeModel, comp: &Component, t: f64) -> f64 {
    let tr = t / comp.tc;
    match model {
        VolumeModel::Rackett => {
            // Spencer-Danner exponent. The piecewise form keeps the exponent
            // finite as Tr → 1 (the high-Tr branch is a bounded rational fit;
            // the low-Tr branch is the classic 1 + (1−Tr)^(2/7)).
            let toh = if tr <= 0.75 {
                1.0 + (1.0 - tr).powf(2.0 / 7.0)
            } else {
                1.6 + 0.006_930_26 / (tr - 0.655)
            };
            1000.0 * R_GAS * comp.tc / comp.pc * comp.zra.powf(toh)
        }
        VolumeModel::Thomson => {
            // Vᵣ⁽⁰⁾: spherical-fluid reference, in powers of (1 − Tr)^(k/3).
            let om = 1.0 - tr;
            let vr0 = 1.0
                + A * om.powf(1.0 / 3.0)
                + B * om.powf(2.0 / 3.0)
                + C * om
                + D * om.powf(4.0 / 3.0);
            // Vᵣ⁽δ⁾: acentric-factor correction. The `Tr − 1.00001` denominator
            // is the legacy guard that keeps the pole just outside Tr = 1.
            let vrs = (E + F * tr + G * tr * tr + H * tr * tr * tr) / (tr - 1.000_01);
            comp.liquid_volume * vr0 * (1.0 - comp.omega_srk * vrs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> Component {
        Component {
            tc: 647.3,
            pc: 22_120.0,
            zra: 0.235,
            ..Default::default()
        }
    }

    #[test]
    fn rackett_water_is_about_18_cm3_per_mol() {
        // Liquid water near room temperature is ~18 cm³/mol. The Rackett
        // correlation should land in that neighbourhood.
        let v = liquid_molar_volume(VolumeModel::Rackett, &water(), 298.15);
        assert!((15.0..20.0).contains(&v), "got {v} cm³/mol");
    }

    #[test]
    fn rackett_volume_increases_with_temperature() {
        // Liquids expand as they warm: V(330 K) > V(298 K).
        let w = water();
        let lo = liquid_molar_volume(VolumeModel::Rackett, &w, 298.15);
        let hi = liquid_molar_volume(VolumeModel::Rackett, &w, 330.0);
        assert!(hi > lo);
    }

    #[test]
    fn thomson_recovers_characteristic_volume_scale() {
        // With V* = 100 cm³/mol and ω_SRK = 0, Thomson reduces to V*·Vᵣ⁽⁰⁾,
        // which is O(100) cm³/mol at a moderate reduced temperature.
        let comp = Component {
            tc: 500.0,
            pc: 4000.0,
            liquid_volume: 100.0,
            omega_srk: 0.0,
            ..Default::default()
        };
        // At Tr = 0.6 the reduced volume Vᵣ⁽⁰⁾ ≈ 0.39, so V ≈ 0.39·V* ≈ 39 cm³/mol
        // — below the characteristic volume, as expected for a sub-critical liquid.
        let v = liquid_molar_volume(VolumeModel::Thomson, &comp, 300.0);
        assert!((30.0..45.0).contains(&v), "got {v} cm³/mol");
    }

    #[test]
    fn high_tr_branch_is_finite() {
        // Above Tr = 0.75 the Rackett exponent switches branches; make sure the
        // result stays finite and positive right up near the critical point.
        let w = water();
        let v = liquid_molar_volume(VolumeModel::Rackett, &w, 0.97 * w.tc);
        assert!(v.is_finite() && v > 0.0, "got {v}");
    }
}
