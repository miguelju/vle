//! Ideal-gas heat capacity of a petroleum fraction.
//!
//! Every enthalpy balance in a distillation column needs `Cp°` for each
//! component. For a named compound this crate reads a measured polynomial out of
//! the component database into [`Component::cp_coeffs`]. A pseudocomponent has
//! no such measurement — nobody has ever put "the 340–360 °C cut of Arab Light"
//! in a calorimeter — so it has to be *correlated*, and Kesler–Lee's is the
//! correlation the API adopted (Procedure 7D3.6).
//!
//! [`Component::cp_coeffs`]: crate::types::Component::cp_coeffs
//!
//! # The correlation
//!
//! A quadratic in temperature whose three coefficients depend on the Watson
//! characterization factor alone:
//!
//! ```text
//!   Cp° [Btu/(lb·°F)] = A₀ + A₁·T + A₂·T²          T in °R
//!   A₀ = −0.33886 + 0.02827·K_W
//!   A₁ = (−0.9291 + 1.1543·K_W − 0.0368·K_W²)×10⁻⁴
//!   A₂ = −1.6658×10⁻⁷
//! ```
//!
//! That `K_W` is the *whole* composition dependence is the striking part: one
//! number distinguishing a paraffin from an aromatic is apparently enough to
//! predict a heat capacity to a few percent. It works because ideal-gas `Cp` is
//! essentially a count of vibrational modes per unit mass, and hydrogen content
//! — which is what `K_W` measures — is what sets that count.
//!
//! # Accuracy, and one term that is deliberately missing
//!
//! Checked in this repo against measured `Cp°` polynomials for ten pure
//! hydrocarbons over **300–1000 K**:
//!
//! | family | worst deviation |
//! |---|---|
//! | n-paraffins (n-C5 … n-C10) | **2.9 %** |
//! | aromatics (benzene, toluene) | **3.1 %** |
//! | naphthenes (cyclohexane, methylcyclohexane) | **15.9 %** |
//!
//! **The naphthene column is the honest cost of an unimplemented term.**
//! Kesler–Lee's paper carries a correction factor `CF`, applied when
//! `10 < K_W < 12.8`, precisely to fix ring compounds — and naphthenes are the
//! only one of the three families that sits inside that window. `CF` is **not
//! implemented here**, because the published coefficients that multiply it could
//! not be verified against a primary source while this was written, and shipping
//! unverified constants in a thermodynamic correlation is worse than shipping a
//! documented gap. The consequence is stated above and asserted in the tests: on
//! a naphthenic cut, expect `Cp°` to be low by up to ~16 % at high temperature.
//! For a paraffinic or aromatic assay the correlation is as good as its
//! published accuracy. Adding `CF` is a well-defined follow-on.
//!
//! # References
//! - (36) Kesler, M. G.; Lee, B. I. Improve Prediction of Enthalpy of Fractions.
//!   *Hydrocarbon Process.* **1976**, *55* (3), 153–158.
//! - (41) API *Technical Data Book — Petroleum Refining*, Procedure 7D3.6.

use super::PetroleumError;
use super::gravity::k_to_r;
use crate::types::R_GAS;

/// 1 Btu(IT)/(lb·°F) in kJ/(kg·K) — exact by definition of the IT Btu.
const BTU_LB_F_TO_KJ_KG_K: f64 = 4.1868;

/// The Watson-K window Kesler–Lee fitted, outside which the correlation is an
/// extrapolation. Reported by [`ideal_gas_cp_coeffs`] as an error rather than
/// silently returning a number nobody should trust.
const KW_MIN: f64 = 9.5;
/// Upper end of the fitted Watson-K window.
const KW_MAX: f64 = 13.5;

/// The three Kesler–Lee coefficients, in the published units
/// (Btu/(lb·°F), T in °R).
fn kesler_lee_coefficients(kw: f64) -> (f64, f64, f64) {
    let a0 = -0.338_86 + 0.028_27 * kw;
    let a1 = (-0.9291 + 1.1543 * kw - 0.0368 * kw * kw) * 1e-4;
    let a2 = -1.6658e-7;
    (a0, a1, a2)
}

fn check_kw(kw: f64) -> Result<(), PetroleumError> {
    if !kw.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "Watson K must be finite, got {kw}"
        )));
    }
    if !(KW_MIN..=KW_MAX).contains(&kw) {
        return Err(PetroleumError::InvalidInput(format!(
            "Watson K = {kw:.3} is outside the {KW_MIN}–{KW_MAX} window the \
             Kesler-Lee ideal-gas Cp correlation was fitted on"
        )));
    }
    Ok(())
}

/// Ideal-gas heat capacity of a petroleum fraction on a **mass** basis.
///
/// Ref (36), API Procedure 7D3.6. See the module docs for the accuracy and for
/// the `CF` naphthene correction that is not implemented.
///
/// # Arguments
/// * `watson_k` — Watson characterization factor, **dimensionless**. Must lie in
///   9.5–13.5.
/// * `t` — temperature in **K**.
///
/// # Returns
/// Ideal-gas `Cp°` in **kJ/(kg·K)**.
pub fn ideal_gas_cp_mass(watson_k: f64, t: f64) -> Result<f64, PetroleumError> {
    check_kw(watson_k)?;
    if t <= 0.0 || !t.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "temperature must be positive and finite, got {t} K"
        )));
    }
    let (a0, a1, a2) = kesler_lee_coefficients(watson_k);
    let t_r = k_to_r(t);
    Ok((a0 + a1 * t_r + a2 * t_r * t_r) * BTU_LB_F_TO_KJ_KG_K)
}

/// Ideal-gas heat capacity of a petroleum fraction on a **molar** basis.
///
/// # Arguments
/// * `watson_k` — Watson characterization factor, **dimensionless**.
/// * `mw` — molecular weight, **g/mol**.
/// * `t` — temperature in **K**.
///
/// # Returns
/// Ideal-gas `Cp°` in **kJ/(kmol·K)** — the crate's canonical molar energy unit.
pub fn ideal_gas_cp_molar(watson_k: f64, mw: f64, t: f64) -> Result<f64, PetroleumError> {
    if mw <= 0.0 || !mw.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "molecular weight must be positive and finite, got {mw} g/mol"
        )));
    }
    // kJ/(kg·K) × (kg/kmol) = kJ/(kmol·K); g/mol and kg/kmol are the same number.
    Ok(ideal_gas_cp_mass(watson_k, t)? * mw)
}

/// Kesler–Lee `Cp°` as the dimensionless polynomial coefficients
/// [`Component::cp_coeffs`] expects.
///
/// [`Component::cp_coeffs`]: crate::types::Component::cp_coeffs
///
/// `Component` stores heat capacity as `Cp°/R = a₀ + a₁T + a₂T² + a₃T³ + a₄T⁴`
/// with **T in K**, so a pseudocomponent built here drops straight into the
/// same [`crate::energy`] machinery every named compound uses — no special case
/// anywhere downstream. That is the whole point of this function.
///
/// The conversion is three factors stacked: Btu/(lb·°F) → kJ/(kg·K), mass basis
/// → molar basis via `mw`, and °R → K, which pulls a `1.8ᵏ` onto the `k`-th
/// coefficient. Kesler–Lee is quadratic, so `a₃` and `a₄` come back zero.
///
/// # Arguments
/// * `watson_k` — Watson characterization factor, **dimensionless**, in 9.5–13.5.
/// * `mw` — molecular weight, **g/mol**.
///
/// # Returns
/// Five coefficients of `Cp°/R` with **T in K**, **dimensionless**.
pub fn ideal_gas_cp_coeffs(watson_k: f64, mw: f64) -> Result<[f64; 5], PetroleumError> {
    check_kw(watson_k)?;
    if mw <= 0.0 || !mw.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "molecular weight must be positive and finite, got {mw} g/mol"
        )));
    }
    let (a0, a1, a2) = kesler_lee_coefficients(watson_k);
    // Btu/(lb·°F) -> kJ/(kg·K) -> kJ/(kmol·K) -> dimensionless Cp°/R.
    let scale = BTU_LB_F_TO_KJ_KG_K * mw / R_GAS;
    Ok([
        a0 * scale,
        // Each power of T_R = 1.8·T_K contributes another 1.8.
        a1 * 1.8 * scale,
        a2 * 1.8 * 1.8 * scale,
        0.0,
        0.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::petroleum::gravity::watson_k;

    /// `(name, Tb [K], SG 60/60, M [g/mol])`. The measured Cp°/R polynomials
    /// live in [`measured_cp_coeffs`] below, keyed by name — a const array of
    /// ten five-element arrays inline is unreadable.
    ///
    /// The polynomials are this crate's bundled component database, itself a
    /// degree-4 fit to Poling, Prausnitz & O'Connell's ideal-gas `Cp°/R`. They
    /// are the oracle: a correlation that claims to predict `Cp°` for an
    /// unmeasured cut had better reproduce the measured value for a compound
    /// whose `K_W` is in range.
    const REFERENCE: [(&str, f64, f64, f64); 10] = [
        ("n-pentane", 309.209, 0.6312, 72.1488),
        ("n-hexane", 341.866, 0.664, 86.1754),
        ("n-heptane", 371.55, 0.6882, 100.2019),
        ("n-octane", 398.794, 0.707, 114.2285),
        ("n-nonane", 423.913, 0.7219, 128.2551),
        ("n-decane", 447.27, 0.7342, 142.2817),
        ("benzene", 353.219, 0.8829, 78.1118),
        ("toluene", 383.746, 0.8719, 92.1384),
        ("cyclohexane", 353.865, 0.7834, 84.1595),
        ("methylcyclohexane", 374.01, 0.7748, 98.1861),
    ];

    /// The measured `Cp°/R` polynomials from the bundled component database
    /// (`python/src/vle/data/components.json`), a degree-4 fit to Poling,
    /// Prausnitz & O'Connell's ideal-gas Cp°/R over 200–1000 K. T in **K**.
    fn measured_cp_coeffs(name: &str) -> [f64; 5] {
        match name {
            "n-pentane" => [
                7.55395695206698,
                -0.0003679979028814063,
                0.00011845932493273191,
                -1.4938914867213207e-07,
                5.752967215414526e-11,
            ],
            "n-hexane" => [
                8.83094967483494,
                -0.000165999054017394,
                0.00014301918497281133,
                -1.831389563412154e-07,
                7.123959402505293e-11,
            ],
            "n-heptane" => [
                9.633945098783862,
                0.0041559763162275,
                0.00015493911704438188,
                -2.0065885650010049e-07,
                7.769955721149113e-11,
            ],
            "n-octane" => [
                10.823938317338229,
                0.004982971603407559,
                0.00017750898842486308,
                -2.3136868149321465e-07,
                8.979948825729673e-11,
            ],
            "n-nonane" => [
                12.151930749472886,
                0.004574973928474323,
                0.00020415883655467348,
                -2.677684740607602e-07,
                1.0464940363169358e-10,
            ],
            "n-decane" => [
                13.466923255690485,
                0.00413897641310502,
                0.00023126868206308312,
                -3.047682632090871e-07,
                1.1969931786635066e-10,
            ],
            "benzene" => [
                3.5509797639383125,
                -0.0061839647592780355,
                0.00014364918138263418,
                -1.9806887125971656e-07,
                8.233953076955149e-11,
            ],
            "toluene" => [
                3.865977968849799,
                0.0035579797240465165,
                0.00013355923888245565,
                -1.8658893668072264e-07,
                7.689956177044637e-11,
            ],
            "cyclohexane" => [
                4.034977005770514,
                -0.004432974737690908,
                0.00016833904068188385,
                -2.077488160963606e-07,
                7.74595585791775e-11,
            ],
            "methylcyclohexane" => [
                3.1479820605119238,
                0.01843789492748277,
                0.00013623922360995544,
                -1.8792892904447214e-07,
                7.363958034818791e-11,
            ],
            other => panic!("no reference Cp polynomial for {other}"),
        }
    }

    /// Measured `Cp°` in kJ/(kg·K) at `t` for one reference compound.
    fn measured_cp_mass(name: &str, mw: f64, t: f64) -> f64 {
        let a = measured_cp_coeffs(name);
        let cp_over_r: f64 = a
            .iter()
            .enumerate()
            .map(|(k, c)| c * t.powi(k as i32))
            .sum();
        cp_over_r * R_GAS / mw
    }

    /// Worst percentage deviation of the correlation from the measurement over
    /// 300–1000 K, for the named compounds.
    fn worst_over_temperature(names: &[&str]) -> (f64, String) {
        let mut worst: f64 = 0.0;
        let mut who = String::new();
        for row in REFERENCE.iter().filter(|r| names.contains(&r.0)) {
            let kw = watson_k(row.1, row.2).unwrap();
            for step in 0..=14 {
                let t = 300.0 + step as f64 * 50.0;
                let got = ideal_gas_cp_mass(kw, t).unwrap();
                let want = measured_cp_mass(row.0, row.3, t);
                let err = 100.0 * (got - want).abs() / want;
                if err > worst {
                    worst = err;
                    who = format!("{} at {t:.0} K", row.0);
                }
            }
        }
        (worst, who)
    }

    // === Accuracy against measured heat capacities =======================

    #[test]
    fn paraffins_are_predicted_to_three_percent() {
        let (e, who) = worst_over_temperature(&[
            "n-pentane",
            "n-hexane",
            "n-heptane",
            "n-octane",
            "n-nonane",
            "n-decane",
        ]);
        assert!(
            e < 2.9,
            "worst paraffin deviation {e:.2}% ({who}), expected < 2.9%"
        );
    }

    #[test]
    fn aromatics_are_predicted_to_three_percent() {
        let (e, who) = worst_over_temperature(&["benzene", "toluene"]);
        assert!(
            e < 3.2,
            "worst aromatic deviation {e:.2}% ({who}), expected < 3.2%"
        );
    }

    #[test]
    fn naphthenes_are_the_documented_weak_spot() {
        // This asserts the *gap* rather than the accuracy: the module docs say
        // naphthenes are off by up to ~16 % because the Kesler-Lee CF ring
        // correction is not implemented. If somebody implements CF, this test
        // fails — and the right response is to tighten it and rewrite the docs,
        // not to delete it.
        let (e, who) = worst_over_temperature(&["cyclohexane", "methylcyclohexane"]);
        assert!(
            e < 16.0,
            "worst naphthene deviation {e:.2}% ({who}) exceeds the documented 16%"
        );
        assert!(
            e > 5.0,
            "naphthene deviation is only {e:.2}% — if the CF correction has been \
             added, tighten this bound and update the module docs' accuracy table"
        );
    }

    // === Physical behaviour ==============================================

    #[test]
    fn heat_capacity_rises_with_temperature() {
        // More thermal energy opens more vibrational modes, so Cp° increases
        // monotonically over any range a distillation column operates in.
        let kw = 11.8;
        let mut prev = f64::NEG_INFINITY;
        for step in 0..=14 {
            let t = 300.0 + step as f64 * 50.0;
            let cp = ideal_gas_cp_mass(kw, t).unwrap();
            assert!(cp > prev, "Cp° fell at {t} K: {cp} after {prev}");
            prev = cp;
        }
    }

    #[test]
    fn paraffins_have_higher_mass_heat_capacity_than_aromatics() {
        // Per kilogram, a hydrogen-rich paraffin holds more heat than a
        // hydrogen-poor aromatic — the physical content of the K_W dependence.
        let paraffin = ideal_gas_cp_mass(12.7, 500.0).unwrap();
        let aromatic = ideal_gas_cp_mass(10.0, 500.0).unwrap();
        assert!(
            paraffin > aromatic,
            "paraffin Cp° {paraffin:.4} should exceed aromatic {aromatic:.4} kJ/(kg·K)"
        );
    }

    #[test]
    fn heat_capacity_is_in_a_sane_range_for_hydrocarbons() {
        // Hydrocarbon vapours sit around 1.5-4 kJ/(kg·K) across this window.
        for kw in [10.0, 11.5, 13.0] {
            for t in [300.0, 600.0, 1000.0] {
                let cp = ideal_gas_cp_mass(kw, t).unwrap();
                assert!(
                    (1.0..5.0).contains(&cp),
                    "Cp° = {cp} kJ/(kg·K) at K_W = {kw}, T = {t} K"
                );
            }
        }
    }

    // === The Component bridge ============================================

    #[test]
    fn coefficients_reproduce_the_direct_evaluation() {
        // The whole reason `ideal_gas_cp_coeffs` exists is to hand `Component`
        // a polynomial that means the same thing as calling the correlation.
        // Any slip in the °R -> K power scaling shows up here immediately.
        let (kw, mw) = (11.9, 180.0);
        let a = ideal_gas_cp_coeffs(kw, mw).unwrap();
        for step in 0..=14 {
            let t = 300.0 + step as f64 * 50.0;
            let from_poly: f64 = a
                .iter()
                .enumerate()
                .map(|(k, c)| c * t.powi(k as i32))
                .sum();
            let direct = ideal_gas_cp_molar(kw, mw, t).unwrap() / R_GAS;
            assert!(
                (from_poly - direct).abs() < 1e-10,
                "at {t} K: polynomial {from_poly} vs direct {direct}"
            );
        }
    }

    #[test]
    fn coefficients_feed_the_shared_energy_path() {
        // Build a Component the way `assay` will and evaluate it through
        // `crate::energy::ideal_cp`, which is what every enthalpy balance in
        // the crate actually calls. If this disagrees with the correlation,
        // pseudocomponents would silently carry a different Cp° from the one
        // documented here.
        let (kw, mw) = (11.9, 180.0);
        let comp = crate::types::Component {
            mw,
            cp_coeffs: ideal_gas_cp_coeffs(kw, mw).unwrap(),
            ..Default::default()
        };
        for t in [350.0, 550.0, 750.0] {
            let via_energy = crate::energy::ideal_cp(&comp, t);
            let direct = ideal_gas_cp_molar(kw, mw, t).unwrap();
            assert!(
                (via_energy - direct).abs() < 1e-9,
                "at {t} K: energy::ideal_cp {via_energy} vs correlation {direct} kJ/(kmol·K)"
            );
        }
    }

    #[test]
    fn quadratic_correlation_leaves_the_cubic_and_quartic_terms_zero() {
        let a = ideal_gas_cp_coeffs(12.0, 150.0).unwrap();
        assert_eq!(a[3], 0.0);
        assert_eq!(a[4], 0.0);
        // ... and the quadratic term is negative, as the published A₂ is.
        assert!(a[2] < 0.0, "a₂ = {} should be negative", a[2]);
    }

    #[test]
    fn molar_heat_capacity_scales_linearly_with_molecular_weight() {
        let single = ideal_gas_cp_molar(12.0, 100.0, 500.0).unwrap();
        let double = ideal_gas_cp_molar(12.0, 200.0, 500.0).unwrap();
        assert!((double - 2.0 * single).abs() < 1e-9);
    }

    // === Guard rails ======================================================

    #[test]
    fn refuses_to_extrapolate_outside_the_fitted_watson_k_window() {
        // Silently extrapolating a heat-capacity correlation is how a column
        // energy balance ends up quietly wrong, so this is an error, not a
        // warning.
        assert!(ideal_gas_cp_mass(8.0, 500.0).is_err());
        assert!(ideal_gas_cp_mass(15.0, 500.0).is_err());
        assert!(ideal_gas_cp_coeffs(8.0, 150.0).is_err());
        assert!(ideal_gas_cp_coeffs(f64::NAN, 150.0).is_err());
        // The edges of the window are inside it.
        assert!(ideal_gas_cp_mass(KW_MIN, 500.0).is_ok());
        assert!(ideal_gas_cp_mass(KW_MAX, 500.0).is_ok());
    }

    #[test]
    fn rejects_non_physical_temperature_and_molecular_weight() {
        assert!(ideal_gas_cp_mass(12.0, 0.0).is_err());
        assert!(ideal_gas_cp_mass(12.0, -10.0).is_err());
        assert!(ideal_gas_cp_molar(12.0, 0.0, 500.0).is_err());
        assert!(ideal_gas_cp_coeffs(12.0, -1.0).is_err());
    }
}
