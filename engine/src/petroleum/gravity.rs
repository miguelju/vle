//! Gravity, the Watson characterization factor, and average boiling points.
//!
//! Three small families of quantities that every other module here depends on.
//!
//! **Specific gravity (SG)** in petroleum always means the 60/60 °F value: the
//! density of the liquid at 60 °F divided by the density of water at 60 °F. It
//! is dimensionless and it is *not* the same as density at 20 °C, which is what
//! a chemistry handbook usually prints. **API gravity** is a rescaling of SG
//! invented so that the numbers get *bigger* as the oil gets *lighter* — a
//! light sweet crude is ~40 °API, a heavy sour crude ~20 °API, and bitumen is
//! below 10 °API (it sinks in water).
//!
//! **Watson K** (also called the UOP characterization factor) is the single
//! most useful one-number summary of "what kind of hydrocarbon is this". It
//! compares a fraction's boiling point against its density, which is really a
//! test of how hydrogen-rich the molecules are:
//!
//! | Watson K | family |
//! |---|---|
//! | ~12.5–13.0 | paraffinic (straight-chain alkanes) |
//! | ~11–12 | naphthenic (saturated rings) |
//! | ~10 | aromatic (benzene rings) |
//!
//! **Average boiling points** exist because a *mixture* does not have one
//! boiling point, and different physical properties want different averages of
//! the distribution. Correlations are explicit about which one they want, and
//! using the wrong one is a real (and quiet) source of error.
//!
//! # References
//! - (31) Riazi 2005, ch. 2–3.
//! - (41) API *Technical Data Book*, Procedure 2B1.1 (average boiling points).

use super::PetroleumError;

/// Density of water at 60 °F relative to itself — the SG reference, kept named
/// so the definition reads as a definition rather than a bare `1.0`.
const WATER_SG_REFERENCE: f64 = 1.0;

/// Kelvin → Rankine. Most petroleum correlations were published in °R.
#[inline]
pub(super) fn k_to_r(t_k: f64) -> f64 {
    t_k * 1.8
}

/// Rankine → Kelvin.
#[inline]
pub(super) fn r_to_k(t_r: f64) -> f64 {
    t_r / 1.8
}

/// Kelvin → Fahrenheit. The API distillation-interconversion procedures are
/// published in °F, not °R — the offset matters and dropping it is a classic bug.
#[inline]
pub(super) fn k_to_f(t_k: f64) -> f64 {
    t_k * 1.8 - 459.67
}

/// Fahrenheit → Kelvin.
#[inline]
pub(super) fn f_to_k(t_f: f64) -> f64 {
    (t_f + 459.67) / 1.8
}

/// API gravity from specific gravity.
///
/// `°API = 141.5 / SG − 131.5`
///
/// # Arguments
/// * `sg` — specific gravity at 60/60 °F, **dimensionless**. Must be > 0.
///
/// # Returns
/// API gravity in **degrees API** (dimensionless by convention).
///
/// Water is SG = 1.000 → 10.0 °API exactly, which is the anchor the scale was
/// built around.
pub fn api_from_sg(sg: f64) -> Result<f64, PetroleumError> {
    if sg <= 0.0 || !sg.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "specific gravity must be positive and finite, got {sg}"
        )));
    }
    Ok(141.5 * WATER_SG_REFERENCE / sg - 131.5)
}

/// Specific gravity from API gravity — the exact inverse of [`api_from_sg`].
///
/// `SG = 141.5 / (°API + 131.5)`
///
/// # Arguments
/// * `api` — API gravity in **degrees API**. Must be > −131.5.
///
/// # Returns
/// Specific gravity at 60/60 °F, **dimensionless**.
pub fn sg_from_api(api: f64) -> Result<f64, PetroleumError> {
    let denom = api + 131.5;
    if denom <= 0.0 || !api.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "API gravity must exceed -131.5, got {api}"
        )));
    }
    Ok(141.5 * WATER_SG_REFERENCE / denom)
}

/// Watson (UOP) characterization factor.
///
/// `K_W = (T_b in °R)^(1/3) / SG`
///
/// The cube root of the boiling point in **Rankine** is not an aesthetic
/// choice — the correlation is only dimensionally meaningful on the absolute
/// Fahrenheit scale it was fit on, so the conversion happens here rather than
/// being pushed onto the caller.
///
/// # Arguments
/// * `tb` — normal boiling point (for a fraction, its mean average boiling
///   point) in **K**.
/// * `sg` — specific gravity at 60/60 °F, **dimensionless**.
///
/// # Returns
/// Watson K, **dimensionless**.
pub fn watson_k(tb: f64, sg: f64) -> Result<f64, PetroleumError> {
    if tb <= 0.0 || !tb.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "boiling point must be positive and finite, got {tb} K"
        )));
    }
    if sg <= 0.0 || !sg.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "specific gravity must be positive and finite, got {sg}"
        )));
    }
    Ok(k_to_r(tb).cbrt() / sg)
}

/// The five average boiling points of a petroleum fraction, all in **K**.
///
/// They are ordered `wabp ≥ vabp ≥ cabp ≥ meabp ≥ mabp` for any real fraction,
/// and all five collapse to the same number for a narrow cut. Which one a
/// correlation wants is not interchangeable:
///
/// | average | typical use |
/// |---|---|
/// | `vabp` | the raw volumetric mean; the input to the others |
/// | `wabp` | liquid density, viscosity |
/// | `mabp` | ideal-gas properties, molecular weight |
/// | `cabp` | pseudocritical temperature |
/// | `meabp` | Watson K, critical properties, enthalpy — **the usual default** |
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AverageBoilingPoint {
    /// Volume average boiling point, **K**.
    pub vabp: f64,
    /// Weight average boiling point, **K**.
    pub wabp: f64,
    /// Molar average boiling point, **K**.
    pub mabp: f64,
    /// Cubic average boiling point, **K**.
    pub cabp: f64,
    /// Mean average boiling point, **K**. The one most correlations want.
    pub meabp: f64,
}

/// Estimate all five average boiling points from an ASTM D86 curve.
///
/// This is API Procedure 2B1.1: the volume average comes straight from the
/// 10/30/50/70/90 % points, and the other four are obtained from it with
/// empirical corrections driven by the curve's **10–90 % slope**, which is what
/// measures "how wide-boiling is this material".
///
/// The published correlations are in **°F**, and are written out below in °F so
/// they can be checked against the source line by line.
///
/// # Arguments
/// * `d86_10`, `d86_30`, `d86_50`, `d86_70`, `d86_90` — ASTM D86 temperatures at
///   10, 30, 50, 70 and 90 volume percent distilled, each in **K**.
///
/// # Returns
/// All five averages, in **K**.
///
/// # Errors
/// Returns [`PetroleumError::InvalidInput`] if the 90 % point is below the 10 %
/// point (a decreasing distillation curve is not physical).
pub fn average_boiling_points(
    d86_10: f64,
    d86_30: f64,
    d86_50: f64,
    d86_70: f64,
    d86_90: f64,
) -> Result<AverageBoilingPoint, PetroleumError> {
    let (t10, t30, t50, t70, t90) = (
        k_to_f(d86_10),
        k_to_f(d86_30),
        k_to_f(d86_50),
        k_to_f(d86_70),
        k_to_f(d86_90),
    );
    if t90 < t10 {
        return Err(PetroleumError::InvalidInput(format!(
            "D86 90% point ({d86_90} K) is below the 10% point ({d86_10} K)"
        )));
    }

    // Volume average: the plain arithmetic mean of the five cut points, °F.
    let vabp = (t10 + t30 + t50 + t70 + t90) / 5.0;

    // The 10–90 % slope, °F per volume percent. 80 is the span in percent
    // between the two points, not a fitted constant.
    let sl = (t90 - t10) / 80.0;

    // (VABP − 32) is the volume average expressed above the freezing point of
    // water; the correlations were fit in that variable. Guard the fractional
    // powers against a fraction whose VABP is below 32 °F (an LPG-range cut),
    // where the real power would be complex — there the corrections are
    // negligible anyway, so clamping to zero is both safe and accurate.
    let v32 = (vabp - 32.0).max(0.0);
    let sl = sl.max(0.0);

    // Each correction is exp(a + b·(VABP−32)^p + c·SL^q), in °F.
    let d_wabp = (-3.062_123 - 0.018_29 * v32.powf(0.6667) + 4.458_18 * sl.powf(0.25)).exp();
    let d_mabp = (-0.563_793 - 0.007_981 * v32.powf(0.6667) + 3.047_29 * sl.powf(0.333)).exp();
    let d_cabp = (-0.235_89 - 0.069_06 * v32.powf(0.45) + 1.885_8 * sl.powf(0.45)).exp();
    let d_meabp = (-0.944_02 - 0.008_65 * v32.powf(0.6667) + 2.997_91 * sl.powf(0.333)).exp();

    Ok(AverageBoilingPoint {
        vabp: f_to_k(vabp),
        // The weight average is pulled *up* (heavier molecules weigh more, so
        // weighting by mass favours the high-boiling tail); the other three are
        // pulled down. See the module docs for why the ordering is fixed.
        wabp: f_to_k(vabp + d_wabp),
        mabp: f_to_k(vabp - d_mabp),
        cabp: f_to_k(vabp - d_cabp),
        meabp: f_to_k(vabp - d_meabp),
    })
}

/// Volume-average boiling point of a discrete set of cuts, **K**.
///
/// Useful once an assay has already been sliced: the averages above are for a
/// *curve*, this is for a *list*.
///
/// # Arguments
/// * `tb` — per-cut boiling points in **K**.
/// * `fractions` — per-cut volume (or mole, or weight — whichever average you
///   want) fractions. Need not sum to exactly 1; they are normalized.
///
/// # Returns
/// The weighted mean boiling point in **K**.
pub fn weighted_boiling_point(tb: &[f64], fractions: &[f64]) -> Result<f64, PetroleumError> {
    if tb.len() != fractions.len() {
        return Err(PetroleumError::InvalidInput(format!(
            "boiling points ({}) and fractions ({}) differ in length",
            tb.len(),
            fractions.len()
        )));
    }
    if tb.is_empty() {
        return Err(PetroleumError::InvalidInput("no cuts given".into()));
    }
    let total: f64 = fractions.iter().sum();
    if total <= 0.0 {
        return Err(PetroleumError::InvalidInput(
            "fractions must sum to a positive number".into(),
        ));
    }
    Ok(tb.iter().zip(fractions).map(|(t, f)| t * f).sum::<f64>() / total)
}

/// Cubic-average boiling point of a discrete set of cuts, **K**.
///
/// `CABP = [Σ vᵢ · Tbᵢ^(1/3)]³` — the average that pseudocritical-temperature
/// correlations ask for. Because `x^(1/3)` is concave, Jensen's inequality
/// guarantees `CABP ≤ VABP`, which is the mathematical reason for the ordering
/// documented on [`AverageBoilingPoint`].
///
/// # Arguments
/// * `tb` — per-cut boiling points in **K**.
/// * `volume_fractions` — per-cut volume fractions; normalized internally.
///
/// # Returns
/// The cubic average boiling point in **K**.
pub fn cubic_boiling_point(tb: &[f64], volume_fractions: &[f64]) -> Result<f64, PetroleumError> {
    if tb.len() != volume_fractions.len() {
        return Err(PetroleumError::InvalidInput(format!(
            "boiling points ({}) and fractions ({}) differ in length",
            tb.len(),
            volume_fractions.len()
        )));
    }
    if tb.is_empty() {
        return Err(PetroleumError::InvalidInput("no cuts given".into()));
    }
    let total: f64 = volume_fractions.iter().sum();
    if total <= 0.0 {
        return Err(PetroleumError::InvalidInput(
            "fractions must sum to a positive number".into(),
        ));
    }
    let root: f64 = tb
        .iter()
        .zip(volume_fractions)
        .map(|(t, v)| v * t.cbrt())
        .sum::<f64>()
        / total;
    Ok(root.powi(3))
}

/// Blend Watson K factors by weight fraction.
///
/// `K_mix = Σ wᵢ Kᵢ`. Watson K is very nearly weight-additive, which is why an
/// assay can carry one bulk K and hand it to every cut.
///
/// # Arguments
/// * `kw` — per-cut Watson K values, **dimensionless**.
/// * `weight_fractions` — per-cut weight fractions; normalized internally.
///
/// # Returns
/// The blended Watson K, **dimensionless**.
pub fn blend_watson_k(kw: &[f64], weight_fractions: &[f64]) -> Result<f64, PetroleumError> {
    weighted_boiling_point(kw, weight_fractions)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- gravity ---------------------------------------------------------

    #[test]
    fn water_is_ten_degrees_api() {
        // The API scale was defined so that water (SG = 1) sits at exactly
        // 10 °API. If this drifts, the constants are wrong.
        assert!((api_from_sg(1.0).unwrap() - 10.0).abs() < 1e-12);
        assert!((sg_from_api(10.0).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn api_and_sg_round_trip() {
        for sg in [0.6, 0.7342, 0.8829, 1.05] {
            let api = api_from_sg(sg).unwrap();
            let back = sg_from_api(api).unwrap();
            assert!((back - sg).abs() < 1e-12, "sg {sg} -> api {api} -> {back}");
        }
    }

    #[test]
    fn lighter_oil_has_higher_api() {
        // The whole point of the API scale: bigger number = lighter oil.
        let light = api_from_sg(0.75).unwrap();
        let heavy = api_from_sg(0.95).unwrap();
        assert!(light > heavy, "light {light} should exceed heavy {heavy}");
    }

    #[test]
    fn rejects_nonphysical_gravity() {
        assert!(api_from_sg(0.0).is_err());
        assert!(api_from_sg(-0.5).is_err());
        assert!(sg_from_api(-131.5).is_err());
        assert!(sg_from_api(-200.0).is_err());
    }

    // --- Watson K --------------------------------------------------------

    #[test]
    fn watson_k_separates_paraffins_from_aromatics() {
        // n-heptane: Tb = 371.55 K, SG = 0.6882 -> paraffinic, K ≈ 12.7
        // benzene:   Tb = 353.22 K, SG = 0.8829 -> aromatic,   K ≈ 9.7
        // This is the single check that the °R conversion inside is right: in
        // K the same inputs would give ~10.4 and ~8.0, which would misclassify
        // n-heptane as an aromatic.
        let paraffin = watson_k(371.55, 0.6882).unwrap();
        let aromatic = watson_k(353.219, 0.8829).unwrap();
        assert!(
            (12.5..13.0).contains(&paraffin),
            "n-heptane K_W = {paraffin}, expected ~12.7"
        );
        assert!(
            (9.5..10.0).contains(&aromatic),
            "benzene K_W = {aromatic}, expected ~9.7"
        );
    }

    #[test]
    fn watson_k_rejects_bad_input() {
        assert!(watson_k(0.0, 0.8).is_err());
        assert!(watson_k(400.0, 0.0).is_err());
        assert!(watson_k(f64::NAN, 0.8).is_err());
    }

    // --- temperature helpers ---------------------------------------------

    #[test]
    fn temperature_conversions_are_exact_at_known_anchors() {
        // Water freezes at 273.15 K = 491.67 °R = 32 °F.
        assert!((k_to_r(273.15) - 491.67).abs() < 1e-9);
        assert!((k_to_f(273.15) - 32.0).abs() < 1e-9);
        assert!((f_to_k(32.0) - 273.15).abs() < 1e-9);
        assert!((r_to_k(491.67) - 273.15).abs() < 1e-9);
    }

    // --- average boiling points ------------------------------------------

    #[test]
    fn average_boiling_points_respect_the_physical_ordering() {
        // A wide-boiling gas oil: D86 10/30/50/70/90 spanning ~100 K.
        let a = average_boiling_points(450.0, 480.0, 505.0, 530.0, 565.0).unwrap();
        assert!(
            a.wabp >= a.vabp,
            "WABP {} should be >= VABP {}",
            a.wabp,
            a.vabp
        );
        assert!(
            a.vabp >= a.cabp,
            "VABP {} should be >= CABP {}",
            a.vabp,
            a.cabp
        );
        assert!(
            a.cabp >= a.meabp,
            "CABP {} should be >= MeABP {}",
            a.cabp,
            a.meabp
        );
        assert!(
            a.meabp >= a.mabp,
            "MeABP {} should be >= MABP {}",
            a.meabp,
            a.mabp
        );
    }

    #[test]
    fn narrow_cut_collapses_every_average_onto_vabp() {
        // With a nearly flat curve the slope SL -> 0, every correction term
        // exp(... + c·SL^q) -> exp(-large) -> 0, and all five averages meet.
        let a = average_boiling_points(500.0, 500.2, 500.4, 500.6, 500.8).unwrap();
        for (name, v) in [
            ("wabp", a.wabp),
            ("mabp", a.mabp),
            ("cabp", a.cabp),
            ("meabp", a.meabp),
        ] {
            assert!(
                (v - a.vabp).abs() < 0.5,
                "{name} = {v} drifted from VABP {} on a narrow cut",
                a.vabp
            );
        }
    }

    #[test]
    fn average_boiling_points_reject_a_decreasing_curve() {
        assert!(average_boiling_points(560.0, 540.0, 520.0, 500.0, 480.0).is_err());
    }

    #[test]
    fn low_boiling_cut_below_freezing_stays_finite() {
        // An LPG-range cut has VABP < 32 °F, where (VABP-32)^0.6667 would be
        // complex. The clamp must keep every average finite rather than NaN.
        let a = average_boiling_points(250.0, 255.0, 260.0, 265.0, 270.0).unwrap();
        for v in [a.vabp, a.wabp, a.mabp, a.cabp, a.meabp] {
            assert!(v.is_finite(), "got a non-finite average: {v}");
        }
    }

    // --- discrete averages -----------------------------------------------

    #[test]
    fn cubic_average_never_exceeds_volume_average() {
        // Jensen's inequality on the concave cube root. Checked on a spread of
        // widths so a sign slip cannot hide on one case.
        for spread in [10.0, 50.0, 150.0] {
            let tb = [400.0 - spread, 400.0, 400.0 + spread];
            let v = [0.3, 0.4, 0.3];
            let vabp = weighted_boiling_point(&tb, &v).unwrap();
            let cabp = cubic_boiling_point(&tb, &v).unwrap();
            assert!(cabp <= vabp, "spread {spread}: CABP {cabp} > VABP {vabp}");
        }
    }

    #[test]
    fn averages_agree_for_a_single_cut() {
        let tb = [430.0];
        let f = [1.0];
        assert!((weighted_boiling_point(&tb, &f).unwrap() - 430.0).abs() < 1e-9);
        assert!((cubic_boiling_point(&tb, &f).unwrap() - 430.0).abs() < 1e-9);
    }

    #[test]
    fn discrete_averages_normalize_their_weights() {
        // Passing fractions that sum to 2 must give the same answer as ones
        // that sum to 1 — callers should not have to pre-normalize.
        let tb = [350.0, 450.0];
        let a = weighted_boiling_point(&tb, &[0.25, 0.75]).unwrap();
        let b = weighted_boiling_point(&tb, &[0.5, 1.5]).unwrap();
        assert!((a - b).abs() < 1e-12);
    }

    #[test]
    fn discrete_averages_reject_mismatched_lengths() {
        assert!(weighted_boiling_point(&[400.0, 450.0], &[1.0]).is_err());
        assert!(cubic_boiling_point(&[400.0], &[0.5, 0.5]).is_err());
        assert!(weighted_boiling_point(&[], &[]).is_err());
    }

    #[test]
    fn blended_watson_k_lands_between_its_endpoints() {
        let k = blend_watson_k(&[10.0, 13.0], &[0.5, 0.5]).unwrap();
        assert!((k - 11.5).abs() < 1e-12, "got {k}");
    }
}
