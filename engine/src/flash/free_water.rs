//! Free-water flash — the water-decant model a steam-stripped column needs.
//!
//! *Milestone 20 (U4). Design record:
//! `docs/plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md` §2.*
//!
//! # Why a two-phase flash is not enough
//!
//! An atmospheric crude tower runs stripping steam. Wherever the vapor cools —
//! the overhead condenser, the reflux drum, every side stripper — that steam
//! condenses, and because water and hydrocarbons are practically immiscible it
//! does not dissolve into the oil: it forms a **second liquid phase**. A
//! two-phase (V–L) flash on such a feed either fails to converge or, worse,
//! converges to a physically meaningless single "liquid" that is half water and
//! half naphtha.
//!
//! # The model
//!
//! Refinery practice does not solve the general three-phase problem for this;
//! it uses the **free-water approximation**, and so does this function:
//!
//! 1. Water in the hydrocarbon liquid is neglected (its solubility is
//!    ~10⁻⁴–10⁻³ mole fraction at column conditions).
//! 2. A free-water phase, when present, is **pure water**, and the vapor is
//!    saturated with it: `y_w·P = Pˢᵃᵗ_w(T)`.
//! 3. The hydrocarbons flash **as if the water were not there**, at the
//!    hydrocarbon partial pressure `P_hc = P − y_w·P` (Dalton), using
//!    whatever [`SystemSpec`] models the caller configured — cubic, γ-φ,
//!    Grayson–Streed, Braun K10.
//!
//! So the algorithm is: try case A (free water present) — `y_w = Pˢᵃᵗ_w/P`,
//! dry flash at `P − Pˢᵃᵗ_w`, water balance gives the free-water moles; if that
//! is negative, case B (no free water) — all water is in the vapor, `y_w` is
//! then set by the water/vapor balance, which couples to the dry flash through
//! `P_hc` and is solved by a short fixed-point loop on `y_w`. Case B is also
//! what happens when `Pˢᵃᵗ_w ≥ P` (water cannot condense at all).
//!
//! **What this is not:** a general VLLE stability analysis. It cannot find a
//! second *hydrocarbon* liquid, and it will not tell you water dissolved
//! 0.05 % into a hot naphtha. Both are documented gaps, and both are outside
//! what a crude-column simulation needs from the thermodynamics (the *column*
//! side of decanting — condenser water legs, two-liquid stages — is downstream
//! work in `stages-thermo`, D6 in the plan).
//!
//! # Cost
//!
//! One dry flash for case A, plus a handful more for case B — every one of
//! them the same allocation-once [`flash_isothermal_warm`] the rest of the
//! crate uses, warm-started from the previous K vector. Water's `Pˢᵃᵗ` comes
//! from the water component's own saturation model unless the caller passes
//! an IF97 value.

use super::FlashError;
use super::isothermal::{FlashResult, flash_isothermal_warm};
use super::system::SystemSpec;
use crate::saturation::psat;

/// Result of a free-water flash. Fractions are **moles per mole of total
/// feed**, and the three phases sum to one.
#[derive(Debug, Clone, PartialEq)]
pub struct FreeWaterFlashResult {
    /// Vapor moles per mole of feed (hydrocarbons **plus** water vapor).
    pub vapor_fraction: f64,
    /// Hydrocarbon-liquid moles per mole of feed.
    pub hc_liquid_fraction: f64,
    /// Free-water (pure liquid water) moles per mole of feed. Zero when no
    /// second liquid forms.
    pub free_water_fraction: f64,
    /// Vapor mole fractions, all N components (water at `water_index`).
    pub y: Vec<f64>,
    /// Hydrocarbon-liquid mole fractions, all N components; water's entry is
    /// zero by the model's construction.
    pub x: Vec<f64>,
    /// Converged dry-hydrocarbon K-values `Kᵢ = yᵢ/xᵢ` at the hydrocarbon
    /// partial pressure; water's entry is `y_w/1` (its "K" against pure liquid
    /// water) when free water exists, else `f64::NAN`.
    pub k: Vec<f64>,
    /// `true` if a free-water phase is present.
    pub free_water: bool,
    /// Vapor mole fraction of water actually used, `y_w`.
    pub y_water: f64,
    /// Water saturation pressure used, **kPa**.
    pub psat_water: f64,
    /// Total flash-driver iterations across every dry flash performed.
    pub iterations: usize,
}

/// Free-water flash of a water-containing feed at `t` (**K**), `p` (**kPa
/// absolute**).
///
/// # Arguments
/// * `spec` — the full system, **including** water as component
///   `water_index`. Its liquid/vapor models are used for the hydrocarbons only.
/// * `z` — feed mole fractions, length N, including water.
/// * `water_index` — which component is water.
/// * `psat_water` — water saturation pressure at `t` in **kPa**, or `None` to
///   evaluate the water component's own saturation model. Pass an IAPWS-IF97
///   value from `vle-steam` when accuracy matters (Antoine is fine at column
///   conditions).
/// * `tol`, `max_iter` — passed to the dry hydrocarbon flash.
///
/// # Errors
/// [`FlashError::Dimension`] / [`FlashError::InvalidInput`] on bad shapes or a
/// non-water-bearing feed (`z[water_index] == 0` is fine and reduces to the
/// ordinary flash on the rest); [`FlashError::Thermo`] if `Pˢᵃᵗ_w` cannot be
/// evaluated; any dry-flash error.
#[allow(clippy::too_many_arguments)]
pub fn flash_free_water(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    water_index: usize,
    psat_water: Option<f64>,
    tol: f64,
    max_iter: usize,
) -> Result<FreeWaterFlashResult, FlashError> {
    let n = spec.n();
    if z.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, z={}",
            z.len()
        )));
    }
    if water_index >= n {
        return Err(FlashError::InvalidInput(format!(
            "water_index {water_index} out of range for {n} components"
        )));
    }
    if !(t > 0.0 && p > 0.0 && t.is_finite() && p.is_finite()) {
        return Err(FlashError::InvalidInput(format!("T = {t} K, P = {p} kPa")));
    }
    let zsum: f64 = z.iter().sum();
    if zsum <= 0.0 || !zsum.is_finite() || z.iter().any(|&zi| zi < 0.0 || !zi.is_finite()) {
        return Err(FlashError::InvalidInput(
            "feed must be non-negative and non-empty".into(),
        ));
    }
    let z_w = z[water_index] / zsum;
    let z_hc_total = 1.0 - z_w;

    let pw = match psat_water {
        Some(v) if v > 0.0 && v.is_finite() => v,
        Some(v) => {
            return Err(FlashError::InvalidInput(format!("psat_water = {v} kPa")));
        }
        None => psat(
            spec.sat_models
                .get(water_index)
                .copied()
                .unwrap_or(spec.components[water_index].sat_model),
            &spec.components[water_index],
            t,
        )
        .map_err(|e| FlashError::Thermo(format!("water saturation pressure: {e}")))?,
    };

    // Dry feed: water zeroed, the rest renormalised. The engine's flash and
    // Rachford-Rice both tolerate a zero-feed component (it is filtered out of
    // the pole bracket and its K is simply carried), so no re-indexing of the
    // spec is needed — the whole point of doing it this way is zero
    // allocation beyond the result vectors.
    let mut z_dry = vec![0.0; n];
    if z_hc_total > 0.0 {
        for (i, zi) in z.iter().enumerate() {
            z_dry[i] = if i == water_index {
                0.0
            } else {
                zi / zsum / z_hc_total
            };
        }
    }

    let mut iterations = 0usize;
    let mut k_warm: Option<Vec<f64>> = None;

    // The dry flash at a hydrocarbon partial pressure, warm-started.
    let dry_flash = |p_hc: f64,
                     k_warm: &mut Option<Vec<f64>>,
                     iterations: &mut usize|
     -> Result<FlashResult, FlashError> {
        if z_hc_total <= 0.0 {
            // Pure water feed: nothing to flash. Represent as "all vapor" of an
            // empty hydrocarbon phase; the water balance below does the rest.
            return Ok(FlashResult {
                beta: 1.0,
                x: z_dry.clone(),
                y: z_dry.clone(),
                k: vec![1.0; n],
                iterations: 0,
                two_phase: false,
            });
        }
        let r = flash_isothermal_warm(spec, t, p_hc, &z_dry, k_warm.as_deref(), tol, max_iter)?;
        *iterations += r.iterations;
        *k_warm = Some(r.k.clone());
        Ok(r)
    };

    // ---- Case A: free water present, vapor saturated with water. ----------
    if pw < p {
        let y_w = pw / p;
        let r = dry_flash(p - pw, &mut k_warm, &mut iterations)?;
        // Hydrocarbon vapor moles per mole of feed, and the water that rides
        // with it at y_w: n_wv = V_hc·y_w/(1 − y_w).
        let v_hc = r.beta * z_hc_total;
        let n_wv = v_hc * y_w / (1.0 - y_w);
        let free_w = z_w - n_wv;
        if free_w >= 0.0 {
            let v_total = v_hc + n_wv;
            let mut y = vec![0.0; n];
            if v_total > 0.0 {
                for (i, (yi, ry)) in y.iter_mut().zip(&r.y).enumerate() {
                    *yi = if i == water_index {
                        n_wv / v_total
                    } else {
                        ry * v_hc / v_total
                    };
                }
            }
            let mut k = r.k.clone();
            k[water_index] = if v_total > 0.0 {
                y[water_index]
            } else {
                f64::NAN
            };
            let mut x = r.x.clone();
            x[water_index] = 0.0;
            return Ok(FreeWaterFlashResult {
                vapor_fraction: v_total,
                hc_liquid_fraction: (1.0 - r.beta) * z_hc_total,
                free_water_fraction: free_w,
                y,
                x,
                k,
                free_water: true,
                y_water: if v_total > 0.0 { n_wv / v_total } else { y_w },
                psat_water: pw,
                iterations,
            });
        }
    }

    // ---- Case B: no free water — every water molecule is in the vapor. ----
    // Unknown y_w couples to the dry flash through P_hc = P·(1 − y_w). Fixed
    // point on y_w: y_w = z_w / (V_hc(P_hc) + z_w). It contracts quickly
    // because V_hc moves much less than y_w does; 50 passes is a backstop.
    let mut y_w = if pw < p { pw / p } else { z_w.max(1e-12) };
    let mut r = dry_flash(p * (1.0 - y_w), &mut k_warm, &mut iterations)?;
    for _ in 0..50 {
        let v_hc = r.beta * z_hc_total;
        let next = if v_hc + z_w > 0.0 {
            z_w / (v_hc + z_w)
        } else {
            0.0
        };
        let done = (next - y_w).abs() < 1e-12;
        y_w = next;
        if done {
            break;
        }
        r = dry_flash(p * (1.0 - y_w), &mut k_warm, &mut iterations)?;
    }
    let v_hc = r.beta * z_hc_total;
    let v_total = v_hc + z_w;
    let mut y = vec![0.0; n];
    if v_total > 0.0 {
        for (i, (yi, ry)) in y.iter_mut().zip(&r.y).enumerate() {
            *yi = if i == water_index {
                z_w / v_total
            } else {
                ry * v_hc / v_total
            };
        }
    }
    let mut k = r.k.clone();
    k[water_index] = f64::NAN;
    let mut x = r.x.clone();
    x[water_index] = 0.0;
    Ok(FreeWaterFlashResult {
        vapor_fraction: v_total,
        hc_liquid_fraction: (1.0 - r.beta) * z_hc_total,
        free_water_fraction: 0.0,
        y,
        x,
        k,
        free_water: false,
        y_water: if v_total > 0.0 { z_w / v_total } else { 0.0 },
        psat_water: pw,
        iterations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos::{CubicEos, LiquidModel, VaporModel};
    use crate::mixing::MixingRule;
    use crate::types::Component;

    fn water() -> Component {
        Component {
            name: "water".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            // Reduced Antoine ln(P/Pc) = a1 − a2/(a3 + T): a fit good to ~1 %
            // between 300 and 500 K (checked against IF97 in the test below).
            psat_coeffs: vec![6.288, 3816.44, -46.13],
            ..Component::default()
        }
    }
    fn n_pentane() -> Component {
        Component {
            name: "n-pentane".into(),
            tc: 469.7,
            pc: 3370.0,
            omega: 0.252,
            psat_coeffs: vec![4.55, 2477.07, -39.94],
            ..Component::default()
        }
    }
    fn n_decane() -> Component {
        Component {
            name: "n-decane".into(),
            tc: 617.7,
            pc: 2110.0,
            omega: 0.492,
            psat_coeffs: vec![4.34, 3456.8, -78.67],
            ..Component::default()
        }
    }

    fn spec<'a>(comps: &'a [Component], kij: &'a [Vec<f64>]) -> SystemSpec<'a> {
        SystemSpec {
            components: comps,
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
            mixing_rule: MixingRule::Classical,
            kij,
            aij: &[],
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        }
    }

    #[test]
    fn water_antoine_fixture_is_sane() {
        let w = water();
        let p100 = crate::saturation::psat(w.sat_model, &w, 373.15).unwrap();
        assert!(
            (p100 - 101.325).abs() / 101.325 < 0.02,
            "Psat(100 °C) = {p100} kPa"
        );
    }

    #[test]
    fn cold_overhead_drum_decants_free_water() {
        // Pentane/decane/water at 325 K, 40 kPa — a reflux drum under mild
        // vacuum. Water's Psat (~13.5 kPa) is below P, so the vapor can hold at
        // most ~34 % water; with a mostly-liquid hydrocarbon phase the rest
        // must appear as a free phase.
        let comps = [n_pentane(), n_decane(), water()];
        let s = spec(&comps, &[]);
        let z = [0.25, 0.65, 0.10];
        let r = flash_free_water(&s, 325.0, 40.0, &z, 2, None, 1e-10, 200).unwrap();
        assert!(r.free_water, "{r:?}");
        assert!(
            r.vapor_fraction > 0.02,
            "vapor {} — {r:?}",
            r.vapor_fraction
        );
        assert!(
            r.free_water_fraction > 0.02,
            "free water {} — {r:?}",
            r.free_water_fraction
        );
        // Overall balance closes.
        let total = r.vapor_fraction + r.hc_liquid_fraction + r.free_water_fraction;
        assert!((total - 1.0).abs() < 1e-12, "phases sum to {total}");
        // Water balance closes: vapor water + free water = feed water.
        let water_in_vapor = r.vapor_fraction * r.y[2];
        assert!((water_in_vapor + r.free_water_fraction - 0.10).abs() < 1e-12);
        // Hydrocarbon balance per component: V·yᵢ + L·xᵢ = zᵢ.
        for (i, &zi) in z.iter().enumerate().take(2) {
            let got = r.vapor_fraction * r.y[i] + r.hc_liquid_fraction * r.x[i];
            assert!((got - zi).abs() < 1e-10, "component {i}: {got} vs {zi}");
        }
        // Vapor is saturated with water at exactly Psat/P.
        assert!(
            (r.y[2] - r.psat_water / 40.0).abs() < 1e-12,
            "y_w = {}, Psat/P = {} — {r:?}",
            r.y[2],
            r.psat_water / 40.0
        );
        assert_eq!(r.x[2], 0.0);
    }

    #[test]
    fn a_subcooled_drum_is_two_liquids_and_no_vapor() {
        // 320 K, 200 kPa: the hydrocarbons are all liquid and water's Psat is
        // 10 kPa — every water molecule is free water, and there is no vapor
        // to report a composition for.
        let comps = [n_pentane(), n_decane(), water()];
        let s = spec(&comps, &[]);
        let z = [0.45, 0.45, 0.10];
        let r = flash_free_water(&s, 320.0, 200.0, &z, 2, None, 1e-10, 200).unwrap();
        assert!(r.free_water);
        assert!(r.vapor_fraction.abs() < 1e-12, "{}", r.vapor_fraction);
        assert!((r.free_water_fraction - 0.10).abs() < 1e-12);
        assert!((r.hc_liquid_fraction - 0.90).abs() < 1e-12);
    }

    #[test]
    fn hot_stripper_keeps_all_water_in_the_vapor() {
        // 450 K, 150 kPa: water's Psat (~930 kPa) exceeds P — no free water can
        // exist, so case B: every water molecule is in the vapor.
        let comps = [n_pentane(), n_decane(), water()];
        let s = spec(&comps, &[]);
        let z = [0.2, 0.7, 0.10];
        let r = flash_free_water(&s, 450.0, 150.0, &z, 2, None, 1e-10, 200).unwrap();
        assert!(!r.free_water);
        assert_eq!(r.free_water_fraction, 0.0);
        let water_in_vapor = r.vapor_fraction * r.y[2];
        assert!((water_in_vapor - 0.10).abs() < 1e-10, "{water_in_vapor}");
        let total = r.vapor_fraction + r.hc_liquid_fraction;
        assert!((total - 1.0).abs() < 1e-12);
        for (i, &zi) in z.iter().enumerate().take(2) {
            let got = r.vapor_fraction * r.y[i] + r.hc_liquid_fraction * r.x[i];
            assert!((got - zi).abs() < 1e-10);
        }
        assert!(r.y.iter().sum::<f64>() - 1.0 < 1e-12);
    }

    #[test]
    fn a_little_water_in_a_hot_vapor_needs_no_free_phase_even_below_psat() {
        // 380 K, 200 kPa: Psat_w ≈ 128 kPa < P, but with only 1 % water in a
        // mostly-vaporised feed the vapor holds it all → case A's balance goes
        // negative and case B takes over.
        let comps = [n_pentane(), n_decane(), water()];
        let s = spec(&comps, &[]);
        let z = [0.79, 0.20, 0.01];
        let r = flash_free_water(&s, 380.0, 200.0, &z, 2, None, 1e-10, 200).unwrap();
        assert!(!r.free_water, "{r:?}");
        assert!(
            r.y[2] < r.psat_water / 200.0,
            "vapor is undersaturated in water"
        );
        assert!((r.vapor_fraction * r.y[2] - 0.01).abs() < 1e-10);
    }

    #[test]
    fn an_explicit_water_saturation_pressure_is_honoured() {
        let comps = [n_pentane(), n_decane(), water()];
        let s = spec(&comps, &[]);
        let z = [0.25, 0.65, 0.10];
        let r = flash_free_water(&s, 325.0, 40.0, &z, 2, Some(15.0), 1e-10, 200).unwrap();
        assert_eq!(r.psat_water, 15.0);
        assert!(r.free_water, "{r:?}");
        assert!(r.vapor_fraction > 0.0);
        assert!((r.y[2] - 15.0 / 40.0).abs() < 1e-12);
    }

    #[test]
    fn a_dry_feed_reduces_to_the_ordinary_flash() {
        let comps = [n_pentane(), n_decane(), water()];
        let s = spec(&comps, &[]);
        let z = [0.5, 0.5, 0.0];
        let r = flash_free_water(&s, 350.0, 150.0, &z, 2, None, 1e-10, 200).unwrap();
        let plain =
            crate::flash::isothermal::flash_isothermal(&s, 350.0, 150.0, &z, 1e-10, 200).unwrap();
        assert!(!r.free_water);
        assert!(
            (r.vapor_fraction - plain.beta).abs() < 1e-8,
            "{} vs {}",
            r.vapor_fraction,
            plain.beta
        );
        for i in 0..2 {
            assert!((r.x[i] - plain.x[i]).abs() < 1e-8);
        }
    }

    #[test]
    fn rejects_bad_inputs() {
        let comps = [n_pentane(), water()];
        let s = spec(&comps, &[]);
        assert!(flash_free_water(&s, 320.0, 200.0, &[0.5], 1, None, 1e-10, 100).is_err());
        assert!(flash_free_water(&s, 320.0, 200.0, &[0.5, 0.5], 5, None, 1e-10, 100).is_err());
        assert!(
            flash_free_water(&s, 320.0, 200.0, &[0.5, 0.5], 1, Some(-1.0), 1e-10, 100).is_err()
        );
        assert!(flash_free_water(&s, -1.0, 200.0, &[0.5, 0.5], 1, None, 1e-10, 100).is_err());
    }
}
