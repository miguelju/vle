//! The whole pipeline: a crude assay in, a list of [`Component`]s out.
//!
//! Everything else in [`super`] is a piece of machinery. This is the thing you
//! actually call.
//!
//! ```text
//!   Assay { curve, gravity }
//!        │
//!        ├─ convert_curve  ─────────────▶  TBP basis
//!        ├─ cut_curve      ─────────────▶  N slices, each with Tb and a share
//!        ├─ gravity        ─────────────▶  SG per slice
//!        ├─ estimate       ─────────────▶  M, Tc, Pc, ω, Vc, Zc per slice
//!        ├─ ideal_gas_cp_coeffs ────────▶  Cp°(T) per slice
//!        └─ pseudo-Antoine ─────────────▶  Psat(T) per slice
//!                                          │
//!                                          ▼
//!                          Vec<Component> + mole fractions
//! ```
//!
//! The output plugs straight into [`crate::flash`] and [`crate::mixture`] with
//! no special casing anywhere: a pseudocomponent is an ordinary [`Component`]
//! whose numbers happen to have been correlated rather than measured.
//!
//! # Where the per-cut gravity comes from
//!
//! Property estimation needs a specific gravity for **each** cut, but an assay
//! usually reports one number for the whole barrel. Two ways to bridge that,
//! both in [`GravitySpec`]:
//!
//! - [`GravitySpec::Curve`] — you have a real gravity curve from the lab.
//!   Always use this if you have it.
//! - [`GravitySpec::ConstantWatsonK`] — you have only the bulk gravity. Assume
//!   the whole barrel is chemically uniform (one Watson K throughout) and let
//!   the boiling point do the rest: `SGᵢ = (1.8·Tbᵢ)^(1/3) / K_W`.
//!
//! The constant-`K_W` route has one free parameter, and this module pins it by
//! **requiring that the cuts blend back to the bulk gravity you supplied**.
//! That has a closed form — no iteration — and it means the characterized assay
//! conserves volume and mass exactly rather than approximately:
//!
//! ```text
//!   volume basis:   K_W = Σ vᵢ (1.8·Tbᵢ)^(1/3) / SG_bulk
//!   weight basis:   K_W = 1 / (SG_bulk · Σ wᵢ (1.8·Tbᵢ)^(−1/3))
//! ```
//!
//! Note the consequence, stated rather than buried: this `K_W` is anchored on
//! the **cubic** average boiling point, whereas the textbook definition of
//! Watson K uses the **mean** average. The two differ by a few hundredths on a
//! realistic crude. Exact gravity closure was judged worth more than agreeing
//! with the convention in the third decimal; [`Assay::conventional_watson_k`]
//! reports the textbook value for anyone who needs it.
//!
//! # Volume, weight and mole
//!
//! Distillation curves are volumetric — except ASTM D2887, which is a
//! chromatogram and therefore by weight. Getting that wrong silently misweights
//! the whole barrel, so [`Assay`] tracks the basis from the source curve and
//! converts to mole fractions accordingly:
//!
//! ```text
//!   volume basis:   nᵢ ∝ vᵢ · SGᵢ / Mᵢ
//!   weight basis:   nᵢ ∝ wᵢ / Mᵢ
//! ```

use super::cp::ideal_gas_cp_coeffs;
use super::cuts::{Cut, CutSpec, cut_curve};
use super::distillation::{DistillationBasis, DistillationCurve, convert_curve};
use super::gravity::{k_to_r, watson_k};
use super::properties::{PropertyMethod, PseudoProperties, ZcMethod, estimate};
use super::{PetroleumError, gravity};
use crate::saturation::SatPressureModel;
use crate::types::{Component, R_GAS};

/// Standard atmospheric pressure, **kPa** — the pressure a normal boiling point
/// is defined at, and therefore one of the two anchors of the pseudo-Antoine fit.
const P_ATM_KPA: f64 = 101.325;

/// How the assay's specific gravity is distributed across its cuts.
#[derive(Debug, Clone, PartialEq)]
pub enum GravitySpec {
    /// One bulk gravity for the whole assay; per-cut gravities follow from
    /// holding the Watson characterization factor constant.
    ///
    /// Use when the lab reported an API gravity and nothing else.
    ConstantWatsonK {
        /// Bulk specific gravity at 60/60 °F, **dimensionless**.
        bulk_sg: f64,
    },
    /// A measured gravity curve: specific gravity against cumulative fraction
    /// distilled. Interpolated linearly, flat outside its span.
    ///
    /// Use whenever you have it — it is strictly better information.
    Curve {
        /// Cumulative fraction distilled, **dimensionless**, strictly increasing.
        fractions: Vec<f64>,
        /// Specific gravity at 60/60 °F at each fraction, **dimensionless**.
        sg: Vec<f64>,
    },
}

/// One characterized pseudocomponent: where it came from, what it is, and the
/// [`Component`] that represents it downstream.
///
/// No `PartialEq`: [`Component`] does not implement it (it carries a `String`
/// name and a `Vec` of Antoine coefficients, and comparing floats for equality
/// is not something the crate wants to make easy).
#[derive(Debug, Clone)]
pub struct Pseudocomponent {
    /// The slice of the distillation curve this came from.
    pub cut: Cut,
    /// The correlated physical properties.
    pub properties: PseudoProperties,
    /// Mole fraction in the whole assay, **dimensionless**. Sums to 1 across
    /// the assay.
    pub mole_fraction: f64,
    /// The engine-ready component, in the crate's canonical units.
    pub component: Component,
}

/// A crude assay: a distillation curve plus a gravity, and the choices about
/// how to turn them into pseudocomponents.
#[derive(Debug, Clone, PartialEq)]
pub struct Assay {
    /// The distillation curve, on any [`DistillationBasis`].
    pub curve: DistillationCurve,
    /// Where per-cut gravities come from.
    pub gravity: GravitySpec,
    /// Which critical-property correlation family to use.
    pub property_method: PropertyMethod,
    /// Which Zc correlation to use where the property method needs one.
    pub zc_method: ZcMethod,
    /// Prefix for generated component names — `"PC"` gives `PC-1`, `PC-2`, ….
    pub name_prefix: String,
}

/// Linear interpolation of `ys` against `xs` at `x`, flat outside the span.
///
/// Flat rather than extrapolated: a gravity curve extrapolated past its ends
/// can go negative, and a negative specific gravity would poison every
/// correlation downstream with no obvious symptom.
fn interpolate_flat(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if x <= xs[0] {
        return ys[0];
    }
    let n = xs.len();
    if x >= xs[n - 1] {
        return ys[n - 1];
    }
    let hi = xs.partition_point(|&v| v < x).max(1);
    let lo = hi - 1;
    let w = (x - xs[lo]) / (xs[hi] - xs[lo]);
    ys[lo] + w * (ys[hi] - ys[lo])
}

/// Rackett compressibility from the acentric factor, **dimensionless**.
///
/// `Z_RA = 0.29056 − 0.08775·ω` — Yamada & Gunn (1973). Populating
/// [`Component::zra`] is what lets [`crate::liquid_volume::liquid_molar_volume`]
/// produce a liquid volume for a pseudocomponent, which the Poynting factor and
/// the Wilson activity model both need.
/// Hildebrand solubility parameter at 298.15 K, **(cal/cm³)^½**, from a
/// reduced-Antoine fit `ln(P/Pc) = a₁ − a₂/(a₃ + T)` and the liquid molar
/// volume: `δ² = (ΔHᵥₐₚ − RT)/Vᴸ`, `ΔHᵥₐₚ = R·T²·d ln P/dT` (Clausius–Clapeyron,
/// ideal vapor). Returns 0.0 (the "unknown" sentinel) if the inputs cannot give
/// a positive value, so a bad cut degrades Grayson–Streed to γ = 1 rather than
/// to a NaN.
fn solubility_parameter_from_antoine(psat_coeffs: &[f64], vl_cm3_mol: f64) -> f64 {
    let t = 298.15;
    if psat_coeffs.len() < 3 || vl_cm3_mol <= 0.0 {
        return 0.0;
    }
    let (a2, a3) = (psat_coeffs[1], psat_coeffs[2]);
    // d ln P / dT = a₂/(a₃ + T)²
    let dlnp_dt = a2 / ((a3 + t) * (a3 + t));
    // R in cal/(mol·K) so δ comes out in (cal/cm³)^½ with V in cm³/mol.
    const R_CAL: f64 = 1.987_204;
    let dh_vap = R_CAL * t * t * dlnp_dt; // cal/mol
    let energy = dh_vap - R_CAL * t;
    if !energy.is_finite() || energy <= 0.0 {
        return 0.0;
    }
    (energy / vl_cm3_mol).sqrt()
}

fn rackett_zra(omega: f64) -> f64 {
    0.290_56 - 0.087_75 * omega
}

/// Reduced-Antoine coefficients `[a₁, a₂, a₃]` anchored on the two points a
/// pseudocomponent actually knows.
///
/// The crate's Antoine form is `ln(Psat/Pc) = a₁ − a₂/(a₃ + T)`, T in **K**.
/// A pseudocomponent has no measured vapor-pressure data at all — but it has
/// two exact points on its own curve:
///
/// - `Psat(Tb) = 1 atm`, which is the *definition* of the normal boiling point
///   and the one number the whole characterization was built around;
/// - `Psat(Tc) = Pc`, the critical point.
///
/// Two conditions determine two coefficients, so `a₃` is set to zero and the
/// fit becomes the two-point Clausius–Clapeyron line in `ln P` against `1/T`:
///
/// ```text
///   a₂ = ln(Pc / 1 atm) / (1/Tb − 1/Tc)
///   a₁ = a₂ / Tc
/// ```
///
/// Anchoring on those two points rather than fitting a corresponding-states
/// correlation matters: it guarantees the pseudocomponent boils at the
/// temperature the assay says it boils at. A Riedel or Lee–Kesler `Psat` would
/// be smoother across the middle but would miss the boiling point by a few
/// kelvin, and the boiling point is the measurement.
fn pseudo_antoine_from_boiling_point(
    tb: f64,
    tc: f64,
    pc: f64,
) -> Result<[f64; 3], PetroleumError> {
    if tb >= tc {
        return Err(PetroleumError::InvalidInput(format!(
            "a cut's boiling point ({tb} K) is at or above its estimated critical \
             temperature ({tc} K) — the correlation is being used outside its range"
        )));
    }
    if pc <= P_ATM_KPA {
        return Err(PetroleumError::InvalidInput(format!(
            "a cut's estimated critical pressure ({pc} kPa) is below atmospheric"
        )));
    }
    let a2 = (pc / P_ATM_KPA).ln() / (1.0 / tb - 1.0 / tc);
    Ok([a2 / tc, a2, 0.0])
}

impl Assay {
    /// Build an assay from a distillation curve and a gravity.
    ///
    /// Defaults to [`PropertyMethod::ApiRiaziDaubert1987`],
    /// [`ZcMethod::LeeKesler`] and the name prefix `"PC"`; override with
    /// [`with_property_method`], [`with_zc_method`] and [`with_name_prefix`].
    ///
    /// [`with_property_method`]: Assay::with_property_method
    /// [`with_zc_method`]: Assay::with_zc_method
    /// [`with_name_prefix`]: Assay::with_name_prefix
    ///
    /// # Errors
    /// [`PetroleumError::InvalidInput`] if a gravity is non-physical;
    /// [`PetroleumError::CutPoints`] if a gravity curve is not sorted.
    pub fn new(curve: DistillationCurve, gravity: GravitySpec) -> Result<Self, PetroleumError> {
        match &gravity {
            GravitySpec::ConstantWatsonK { bulk_sg } => {
                if *bulk_sg <= 0.0 || !bulk_sg.is_finite() {
                    return Err(PetroleumError::InvalidInput(format!(
                        "bulk specific gravity must be positive and finite, got {bulk_sg}"
                    )));
                }
            }
            GravitySpec::Curve { fractions, sg } => {
                if fractions.len() != sg.len() {
                    return Err(PetroleumError::CutPoints(format!(
                        "gravity curve has {} fractions but {} gravities",
                        fractions.len(),
                        sg.len()
                    )));
                }
                if fractions.is_empty() {
                    return Err(PetroleumError::CutPoints("gravity curve is empty".into()));
                }
                for (i, &x) in fractions.iter().enumerate() {
                    if i > 0 && x <= fractions[i - 1] {
                        return Err(PetroleumError::CutPoints(format!(
                            "gravity-curve fractions must strictly increase at index {i}"
                        )));
                    }
                }
                for (i, &g) in sg.iter().enumerate() {
                    if g <= 0.0 || !g.is_finite() {
                        return Err(PetroleumError::InvalidInput(format!(
                            "gravity-curve sg[{i}] = {g} is not a positive finite gravity"
                        )));
                    }
                }
            }
        }
        Ok(Self {
            curve,
            gravity,
            property_method: PropertyMethod::default(),
            zc_method: ZcMethod::default(),
            name_prefix: "PC".to_string(),
        })
    }

    /// Choose the critical-property correlation family.
    pub fn with_property_method(mut self, method: PropertyMethod) -> Self {
        self.property_method = method;
        self
    }

    /// Choose the Zc correlation used where the property method needs one.
    pub fn with_zc_method(mut self, method: ZcMethod) -> Self {
        self.zc_method = method;
        self
    }

    /// Choose the prefix for generated component names.
    pub fn with_name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.name_prefix = prefix.into();
        self
    }

    /// Whether the curve's abscissa is weight rather than volume.
    fn is_weight_basis(&self) -> bool {
        self.curve.basis.is_weight_basis()
    }

    /// Per-cut specific gravities, **dimensionless**.
    ///
    /// For [`GravitySpec::Curve`] each cut's gravity is read off the curve at
    /// its mid-fraction. For [`GravitySpec::ConstantWatsonK`] the closed-form
    /// `K_W` documented in the module header is computed first, then applied.
    fn cut_gravities(&self, cuts: &[Cut]) -> Result<Vec<f64>, PetroleumError> {
        match &self.gravity {
            GravitySpec::Curve { fractions, sg } => Ok(cuts
                .iter()
                .map(|c| interpolate_flat(fractions, sg, 0.5 * (c.x_lower + c.x_upper)))
                .collect()),
            GravitySpec::ConstantWatsonK { bulk_sg } => {
                // Pick K_W so the cuts blend back to the bulk gravity exactly.
                // Both branches are one-liners because SGᵢ ∝ 1/K_W, so the
                // blending rule is linear in 1/K_W and inverts in closed form.
                let kw = if self.is_weight_basis() {
                    // Weight blending of gravity is reciprocal: 1/SG = Σ wᵢ/SGᵢ.
                    let s: f64 = cuts.iter().map(|c| c.fraction / k_to_r(c.tb).cbrt()).sum();
                    1.0 / (bulk_sg * s)
                } else {
                    // Volume blending is direct: SG = Σ vᵢ SGᵢ.
                    let s: f64 = cuts.iter().map(|c| c.fraction * k_to_r(c.tb).cbrt()).sum();
                    s / bulk_sg
                };
                if kw <= 0.0 || !kw.is_finite() {
                    return Err(PetroleumError::InvalidInput(format!(
                        "could not find a constant Watson K reproducing a bulk \
                         gravity of {bulk_sg} on this curve (got {kw})"
                    )));
                }
                Ok(cuts.iter().map(|c| k_to_r(c.tb).cbrt() / kw).collect())
            }
        }
    }

    /// The textbook Watson K of the whole assay, **dimensionless**.
    ///
    /// Computed the conventional way — from the **mean** average boiling point
    /// of the cuts and the volume-average gravity — rather than the
    /// cubic-average value the [`GravitySpec::ConstantWatsonK`] closure uses
    /// internally. Report this one; see the module docs for why they differ.
    ///
    /// # Arguments
    /// * `spec` — how to cut the curve, since the averages are over cuts.
    ///
    /// # Returns
    /// The assay's Watson characterization factor, **dimensionless**.
    pub fn conventional_watson_k(&self, spec: &CutSpec) -> Result<f64, PetroleumError> {
        let pcs = self.characterize(spec)?;
        let tb: Vec<f64> = pcs.iter().map(|p| p.cut.tb).collect();
        let vol: Vec<f64> = pcs.iter().map(|p| p.cut.fraction).collect();
        let mole: Vec<f64> = pcs.iter().map(|p| p.mole_fraction).collect();
        let sg: Vec<f64> = pcs.iter().map(|p| p.properties.sg).collect();

        // MeABP = (MABP + CABP)/2 — the standard construction.
        let mabp = gravity::weighted_boiling_point(&tb, &mole)?;
        let cabp = gravity::cubic_boiling_point(&tb, &vol)?;
        let meabp = 0.5 * (mabp + cabp);
        let bulk_sg = gravity::weighted_boiling_point(&sg, &vol)?;
        watson_k(meabp, bulk_sg)
    }

    /// Characterize the assay into pseudocomponents.
    ///
    /// # Arguments
    /// * `spec` — how to slice the distillation curve.
    ///
    /// # Returns
    /// One [`Pseudocomponent`] per cut, light end first, with `mole_fraction`
    /// summing to 1.
    ///
    /// # Errors
    /// Anything the underlying stages can raise. The most likely in practice is
    /// [`PetroleumError::InvalidInput`] from the Watson-K window of the
    /// heat-capacity correlation, which a very aromatic or very paraffinic cut
    /// can fall outside.
    pub fn characterize(&self, spec: &CutSpec) -> Result<Vec<Pseudocomponent>, PetroleumError> {
        // 1. Get onto TBP, which is what every correlation downstream assumes.
        //    An EFV source needs a gravity to convert, so hand it the bulk one.
        let bulk_sg = match &self.gravity {
            GravitySpec::ConstantWatsonK { bulk_sg } => Some(*bulk_sg),
            GravitySpec::Curve { fractions, sg } => Some(interpolate_flat(fractions, sg, 0.5)),
        };
        let tbp = convert_curve(&self.curve, DistillationBasis::Tbp, bulk_sg)?;

        // 2. Slice it.
        let cuts = cut_curve(&tbp, spec)?;

        // 3. Gravity per slice.
        let gravities = self.cut_gravities(&cuts)?;

        // 4. Properties per slice, and the Component that carries them.
        let mut out = Vec::with_capacity(cuts.len());
        for (cut, &sg) in cuts.iter().zip(&gravities) {
            let properties = estimate(self.property_method, cut.tb, sg, self.zc_method)?;
            let cp_coeffs =
                ideal_gas_cp_coeffs(properties.watson_k, properties.mw).map_err(|e| {
                    PetroleumError::InvalidInput(format!(
                        "cut {} (Tb = {:.1} K, SG = {:.4}): {e}",
                        cut.index, cut.tb, sg
                    ))
                })?;
            let psat_coeffs =
                pseudo_antoine_from_boiling_point(cut.tb, properties.tc, properties.pc)?;
            let zra = rackett_zra(properties.omega);
            // Saturated-liquid volume at 25 °C from the same Rackett
            // correlation, so `Component::liquid_volume` — which the Poynting
            // factor and the Wilson/Scatchard-Hildebrand models read — is
            // consistent with `zra` rather than left at zero.
            let liquid_volume = 1000.0 * R_GAS * properties.tc / properties.pc
                * zra.powf(rackett_exponent(298.15 / properties.tc));

            out.push(Pseudocomponent {
                cut: *cut,
                properties,
                // Filled in once every cut's molecular weight is known.
                mole_fraction: 0.0,
                component: Component {
                    name: format!("{}-{}", self.name_prefix, cut.index + 1),
                    tc: properties.tc,
                    pc: properties.pc,
                    vc: properties.vc,
                    zc: properties.zc,
                    omega: properties.omega,
                    tb: cut.tb,
                    mw: properties.mw,
                    cp_coeffs,
                    psat_coeffs: psat_coeffs.to_vec(),
                    sat_model: SatPressureModel::Antoine,
                    zra,
                    liquid_volume,
                    // COSTALD's acentric factor is a fitted SRK value that a
                    // pseudocomponent has no way to know; the plain acentric
                    // factor is the standard stand-in.
                    omega_srk: properties.omega,
                    // Carried so the Braun K10 K-value path (M20) can apply the
                    // Maxwell-Bonnell Watson-K correction per cut.
                    watson_k: properties.watson_k,
                    // Regular-solution solubility parameter for Grayson-Streed
                    // (M20): δ = √((ΔHᵥₐₚ − RT)/Vᴸ) at 25 °C, with ΔHᵥₐₚ from
                    // Clausius-Clapeyron on the cut's own Antoine fit. Cuts
                    // land in the 7-9 (cal/cm³)^½ band real hydrocarbons occupy.
                    solubility_param: solubility_parameter_from_antoine(
                        &psat_coeffs,
                        liquid_volume,
                    ),
                    ..Default::default()
                },
            });
        }

        // 5. Volume (or weight) fractions to mole fractions.
        let moles: Vec<f64> = out
            .iter()
            .zip(&gravities)
            .map(|(p, &sg)| {
                if self.is_weight_basis() {
                    p.cut.fraction / p.properties.mw
                } else {
                    p.cut.fraction * sg / p.properties.mw
                }
            })
            .collect();
        let total: f64 = moles.iter().sum();
        if total <= 0.0 || !total.is_finite() {
            return Err(PetroleumError::InvalidInput(
                "the characterized cuts carry no material".into(),
            ));
        }
        for (p, n) in out.iter_mut().zip(&moles) {
            p.mole_fraction = n / total;
        }
        Ok(out)
    }

    /// Characterize, and return just what a flash calculation needs.
    ///
    /// # Arguments
    /// * `spec` — how to slice the distillation curve.
    ///
    /// # Returns
    /// `(components, mole_fractions)` — ready to hand to
    /// [`crate::mixture::MixtureSpec`] or [`crate::flash`].
    pub fn mixture(&self, spec: &CutSpec) -> Result<(Vec<Component>, Vec<f64>), PetroleumError> {
        let pcs = self.characterize(spec)?;
        Ok((
            pcs.iter().map(|p| p.component.clone()).collect(),
            pcs.iter().map(|p| p.mole_fraction).collect(),
        ))
    }
}

/// The Spencer-Danner Rackett exponent at reduced temperature `tr`.
///
/// Mirrors the branch in [`crate::liquid_volume::liquid_molar_volume`] so the
/// `liquid_volume` stamped onto a pseudocomponent is exactly what that function
/// would return for it. Duplicated rather than exported because it is three
/// lines and coupling the two modules for it would be worse.
fn rackett_exponent(tr: f64) -> f64 {
    if tr <= 0.75 {
        1.0 + (1.0 - tr).powf(2.0 / 7.0)
    } else {
        1.6 + 0.006_930_26 / (tr - 0.655)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liquid_volume::{VolumeModel, liquid_molar_volume};

    /// A light sweet crude: TBP 0-95 % over 310-790 K, 35 °API (SG 0.8498).
    fn crude_curve() -> DistillationCurve {
        DistillationCurve::new(
            DistillationBasis::Tbp,
            vec![0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95],
            vec![310.0, 380.0, 460.0, 540.0, 620.0, 730.0, 790.0],
        )
        .unwrap()
    }

    fn crude() -> Assay {
        Assay::new(
            crude_curve(),
            GravitySpec::ConstantWatsonK { bulk_sg: 0.8498 },
        )
        .unwrap()
    }

    // === The pipeline end to end =========================================

    #[test]
    fn characterizes_a_crude_into_usable_pseudocomponents() {
        let pcs = crude()
            .characterize(&CutSpec::EqualVolume { n: 25 })
            .unwrap();
        assert_eq!(pcs.len(), 25);

        let sum: f64 = pcs.iter().map(|p| p.mole_fraction).sum();
        assert!((sum - 1.0).abs() < 1e-12, "mole fractions sum to {sum}");

        for p in &pcs {
            let c = &p.component;
            assert!(c.tc > c.tb, "{}: Tc {} not above Tb {}", c.name, c.tc, c.tb);
            assert!(c.pc > P_ATM_KPA, "{}: Pc {}", c.name, c.pc);
            assert!(c.mw > 0.0 && c.mw < 2000.0, "{}: M {}", c.name, c.mw);
            assert!(c.vc > 0.0, "{}: Vc {}", c.name, c.vc);
            assert!((0.0..1.5).contains(&c.omega), "{}: ω {}", c.name, c.omega);
            assert!(c.zra > 0.0 && c.zra < 0.35, "{}: Zra {}", c.name, c.zra);
            assert!(c.liquid_volume > 0.0, "{}: VL {}", c.name, c.liquid_volume);
            assert_eq!(c.psat_coeffs.len(), 3);
            assert!(p.mole_fraction > 0.0);
        }
    }

    #[test]
    fn pseudocomponents_get_sequential_names() {
        let pcs = crude()
            .with_name_prefix("NAPHTHA")
            .characterize(&CutSpec::EqualVolume { n: 3 })
            .unwrap();
        let names: Vec<&str> = pcs.iter().map(|p| p.component.name.as_str()).collect();
        assert_eq!(names, ["NAPHTHA-1", "NAPHTHA-2", "NAPHTHA-3"]);
    }

    #[test]
    fn heavier_pseudocomponents_come_out_heavier() {
        // The whole characterization is worthless if the ordering is wrong.
        let pcs = crude()
            .characterize(&CutSpec::EqualVolume { n: 20 })
            .unwrap();
        for w in pcs.windows(2) {
            let (a, b) = (&w[0].component, &w[1].component);
            assert!(b.tb > a.tb, "{} -> {}: Tb fell", a.name, b.name);
            assert!(b.mw > a.mw, "{} -> {}: M fell", a.name, b.name);
            assert!(b.tc > a.tc, "{} -> {}: Tc fell", a.name, b.name);
            assert!(b.pc < a.pc, "{} -> {}: Pc rose", a.name, b.name);
            assert!(b.omega > a.omega, "{} -> {}: ω fell", a.name, b.name);
        }
    }

    #[test]
    fn scales_to_three_hundred_pseudocomponents() {
        // The target the whole petroleum track exists for.
        let (comps, z) = crude().mixture(&CutSpec::EqualVolume { n: 300 }).unwrap();
        assert_eq!(comps.len(), 300);
        assert_eq!(z.len(), 300);
        assert!((z.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(z.iter().all(|&x| x > 0.0));
    }

    // === Gravity closure ==================================================

    #[test]
    fn constant_watson_k_cuts_blend_back_to_the_bulk_gravity_exactly() {
        // The design decision documented in the module header. This is the
        // property that was bought by anchoring K_W on the cubic average, and
        // it is exact, not approximate.
        for bulk in [0.75, 0.8498, 0.92] {
            for n in [3, 25, 200] {
                let assay = Assay::new(
                    crude_curve(),
                    GravitySpec::ConstantWatsonK { bulk_sg: bulk },
                )
                .unwrap();
                let pcs = assay.characterize(&CutSpec::EqualVolume { n }).unwrap();
                let blended: f64 = pcs.iter().map(|p| p.cut.fraction * p.properties.sg).sum();
                assert!(
                    (blended - bulk).abs() < 1e-12,
                    "bulk {bulk}, n = {n}: cuts blend to {blended}"
                );
            }
        }
    }

    #[test]
    fn constant_watson_k_really_is_constant_across_the_cuts() {
        let pcs = crude()
            .characterize(&CutSpec::EqualVolume { n: 30 })
            .unwrap();
        let first = pcs[0].properties.watson_k;
        for p in &pcs {
            assert!(
                (p.properties.watson_k - first).abs() < 1e-9,
                "{}: K_W {} drifted from {first}",
                p.component.name,
                p.properties.watson_k
            );
        }
    }

    #[test]
    fn the_conventional_watson_k_is_close_to_but_not_equal_to_the_internal_one() {
        // The trade-off the module docs describe, quantified. If these ever
        // coincided the docs' caveat would be wrong; if they diverged a lot,
        // the trade-off would be a bad one.
        let spec = CutSpec::EqualVolume { n: 30 };
        let assay = crude();
        let internal = assay.characterize(&spec).unwrap()[0].properties.watson_k;
        let conventional = assay.conventional_watson_k(&spec).unwrap();
        let d = (conventional - internal).abs();
        assert!(
            d > 1e-6,
            "the two definitions gave the same K_W ({internal}); the module \
             docs claim they differ"
        );
        assert!(
            d < 0.2,
            "K_W definitions differ by {d:.4} — more than the 'a few hundredths' \
             the module docs claim"
        );
    }

    #[test]
    fn a_gravity_curve_is_followed_rather_than_assumed() {
        // With an explicit gravity curve the cuts must track it, and Watson K
        // must then vary across the barrel rather than staying pinned.
        let assay = Assay::new(
            crude_curve(),
            GravitySpec::Curve {
                fractions: vec![0.0, 0.5, 1.0],
                sg: vec![0.70, 0.85, 0.98],
            },
        )
        .unwrap();
        let pcs = assay.characterize(&CutSpec::EqualVolume { n: 10 }).unwrap();
        // Gravity rises monotonically, as the supplied curve does.
        for w in pcs.windows(2) {
            assert!(
                w[1].properties.sg > w[0].properties.sg,
                "gravity fell between cuts {} and {}",
                w[0].cut.index,
                w[1].cut.index
            );
        }
        // And it lands on the supplied curve, not on some assumed one.
        assert!(
            (pcs[0].properties.sg - 0.715).abs() < 0.02,
            "{}",
            pcs[0].properties.sg
        );
        assert!(
            (pcs[9].properties.sg - 0.967).abs() < 0.02,
            "{}",
            pcs[9].properties.sg
        );
        // Watson K now varies — it is an output, not an input.
        let spread = pcs
            .iter()
            .map(|p| p.properties.watson_k)
            .fold(f64::MIN, f64::max)
            - pcs
                .iter()
                .map(|p| p.properties.watson_k)
                .fold(f64::MAX, f64::min);
        assert!(
            spread > 0.5,
            "K_W spread only {spread:.3} on a varying-gravity assay"
        );
    }

    // === Basis handling ===================================================

    #[test]
    fn a_weight_basis_curve_is_weighted_as_weight() {
        // D2887 is a chromatogram: its abscissa is weight, not volume. Feeding
        // the same numbers on the two bases must give different mole fractions,
        // because the volume route multiplies by SG and the weight route does
        // not. If these agreed, the basis flag would be doing nothing.
        let sd = DistillationCurve::new(
            DistillationBasis::D2887,
            vec![0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95],
            vec![310.0, 380.0, 460.0, 540.0, 620.0, 730.0, 790.0],
        )
        .unwrap();
        let by_weight = Assay::new(sd, GravitySpec::ConstantWatsonK { bulk_sg: 0.8498 })
            .unwrap()
            .characterize(&CutSpec::EqualVolume { n: 12 })
            .unwrap();
        let by_volume = crude()
            .characterize(&CutSpec::EqualVolume { n: 12 })
            .unwrap();

        assert!((by_weight.iter().map(|p| p.mole_fraction).sum::<f64>() - 1.0).abs() < 1e-12);
        let differs = by_weight
            .iter()
            .zip(&by_volume)
            .any(|(a, b)| (a.mole_fraction - b.mole_fraction).abs() > 1e-4);
        assert!(
            differs,
            "weight and volume bases produced identical mole fractions"
        );
    }

    #[test]
    fn a_d86_assay_is_converted_before_it_is_cut() {
        // The pipeline's first step. A D86 assay must produce different (wider
        // boiling) pseudocomponents than the same numbers read as TBP.
        let d86 = DistillationCurve::new(
            DistillationBasis::D86,
            vec![0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95],
            vec![350.0, 380.0, 420.0, 460.0, 500.0, 550.0, 580.0],
        )
        .unwrap();
        let from_d86 = Assay::new(d86.clone(), GravitySpec::ConstantWatsonK { bulk_sg: 0.80 })
            .unwrap()
            .characterize(&CutSpec::EqualVolume { n: 8 })
            .unwrap();
        let as_tbp = Assay::new(
            DistillationCurve::new(
                DistillationBasis::Tbp,
                d86.fractions.clone(),
                d86.temperatures.clone(),
            )
            .unwrap(),
            GravitySpec::ConstantWatsonK { bulk_sg: 0.80 },
        )
        .unwrap()
        .characterize(&CutSpec::EqualVolume { n: 8 })
        .unwrap();

        assert!(
            from_d86[0].cut.tb < as_tbp[0].cut.tb,
            "the D86 route should put the lightest cut lower ({} vs {})",
            from_d86[0].cut.tb,
            as_tbp[0].cut.tb
        );
        assert!(
            from_d86[7].cut.tb > as_tbp[7].cut.tb,
            "the D86 route should put the heaviest cut higher"
        );
    }

    // === The Component the rest of the crate sees =========================

    #[test]
    fn the_pseudo_antoine_fit_reproduces_the_boiling_point_and_the_critical_point() {
        // The two anchors, checked through the crate's own saturation code
        // rather than through the fitting formula — so this catches a
        // coefficient-order or sign mistake, not just an algebra slip.
        let pcs = crude()
            .characterize(&CutSpec::EqualVolume { n: 15 })
            .unwrap();
        for p in &pcs {
            let c = &p.component;
            let at_tb = crate::saturation::psat(SatPressureModel::Antoine, c, c.tb).unwrap();
            assert!(
                (at_tb - P_ATM_KPA).abs() < 1e-6,
                "{}: Psat(Tb) = {at_tb} kPa, should be 1 atm",
                c.name
            );
            let at_tc = crate::saturation::psat(SatPressureModel::Antoine, c, c.tc).unwrap();
            assert!(
                (at_tc - c.pc).abs() / c.pc < 1e-9,
                "{}: Psat(Tc) = {at_tc} kPa, should be Pc = {}",
                c.name,
                c.pc
            );
        }
    }

    #[test]
    fn saturation_pressure_rises_with_temperature_for_every_pseudocomponent() {
        let pcs = crude()
            .characterize(&CutSpec::EqualVolume { n: 10 })
            .unwrap();
        for p in &pcs {
            let c = &p.component;
            let mut prev = f64::NEG_INFINITY;
            for step in 0..=10 {
                let t = c.tb + step as f64 * (c.tc - c.tb) / 10.0;
                let ps = crate::saturation::psat(SatPressureModel::Antoine, c, t).unwrap();
                assert!(ps > prev, "{}: Psat fell at {t} K", c.name);
                prev = ps;
            }
        }
    }

    #[test]
    fn the_stamped_liquid_volume_matches_what_the_rackett_code_computes() {
        // `Component::liquid_volume` is stamped here but read by
        // `liquid_volume::liquid_molar_volume` and the Poynting factor. If the
        // two ever disagreed, a pseudocomponent would carry a liquid volume
        // inconsistent with its own Zra.
        let pcs = crude()
            .characterize(&CutSpec::EqualVolume { n: 10 })
            .unwrap();
        for p in &pcs {
            let c = &p.component;
            let recomputed = liquid_molar_volume(VolumeModel::Rackett, c, 298.15);
            assert!(
                (c.liquid_volume - recomputed).abs() / recomputed < 1e-12,
                "{}: stamped {} vs recomputed {recomputed} cm³/mol",
                c.name,
                c.liquid_volume
            );
            assert!(
                (30.0..2000.0).contains(&c.liquid_volume),
                "{}: liquid volume {} cm³/mol is not hydrocarbon-like",
                c.name,
                c.liquid_volume
            );
        }
    }

    #[test]
    fn the_heat_capacity_polynomial_is_the_correlations_own() {
        let pcs = crude()
            .characterize(&CutSpec::EqualVolume { n: 8 })
            .unwrap();
        for p in &pcs {
            let c = &p.component;
            for t in [400.0, 700.0] {
                let via_component = crate::energy::ideal_cp(c, t);
                let direct =
                    super::super::cp::ideal_gas_cp_molar(p.properties.watson_k, c.mw, t).unwrap();
                assert!(
                    (via_component - direct).abs() < 1e-9,
                    "{} at {t} K: {via_component} vs {direct} kJ/(kmol·K)",
                    c.name
                );
            }
        }
    }

    #[test]
    fn pseudocomponents_drive_a_real_flash() {
        // The acceptance test for the whole milestone: characterize an assay,
        // hand the components straight to the isothermal flash with no special
        // casing, and get a two-phase split with a closed mass balance. If this
        // works, a pseudocomponent really is an ordinary Component.
        use crate::eos::{CubicEos, LiquidModel, VaporModel};
        use crate::flash::{SystemSpec, isothermal::flash_isothermal};

        let (comps, z) = crude().mixture(&CutSpec::EqualVolume { n: 12 }).unwrap();
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
            mixing_rule: crate::mixing::MixingRule::Classical,
            kij: &[],
            aij: &[],
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        // 500 K and 2 bar puts a wide crude squarely in two phases.
        let r = flash_isothermal(&spec, 500.0, 200.0, &z, 1e-10, 300).unwrap();
        assert!(
            r.two_phase,
            "expected a two-phase split, got β = {}",
            r.beta
        );
        assert!((0.0..1.0).contains(&r.beta), "β = {}", r.beta);

        // Component mass balance: z = β·y + (1−β)·x for every component.
        for i in 0..comps.len() {
            let recombined = r.beta * r.y[i] + (1.0 - r.beta) * r.x[i];
            assert!(
                (recombined - z[i]).abs() < 1e-9,
                "{}: z {} vs recombined {recombined}",
                comps[i].name,
                z[i]
            );
        }
        // And the light ends really do concentrate in the vapor.
        assert!(
            r.y[0] / r.x[0] > r.y[11] / r.x[11],
            "K-values are not ordered: light K = {}, heavy K = {}",
            r.y[0] / r.x[0],
            r.y[11] / r.x[11]
        );
    }

    // === Validation =======================================================

    #[test]
    fn rejects_a_malformed_gravity_specification() {
        assert!(Assay::new(crude_curve(), GravitySpec::ConstantWatsonK { bulk_sg: 0.0 }).is_err());
        assert!(
            Assay::new(
                crude_curve(),
                GravitySpec::ConstantWatsonK { bulk_sg: -1.0 }
            )
            .is_err()
        );
        assert!(
            Assay::new(
                crude_curve(),
                GravitySpec::Curve {
                    fractions: vec![0.0, 1.0],
                    sg: vec![0.8]
                }
            )
            .is_err()
        );
        assert!(
            Assay::new(
                crude_curve(),
                GravitySpec::Curve {
                    fractions: vec![1.0, 0.0],
                    sg: vec![0.8, 0.9]
                }
            )
            .is_err()
        );
        assert!(
            Assay::new(
                crude_curve(),
                GravitySpec::Curve {
                    fractions: vec![0.0, 1.0],
                    sg: vec![0.8, -0.9]
                }
            )
            .is_err()
        );
    }

    #[test]
    fn an_out_of_range_cut_is_reported_with_its_index() {
        // A tar-like assay pushes cuts outside the heat-capacity correlation's
        // Watson-K window. The error must name the offending cut so the user
        // can see which end of the barrel is the problem, rather than failing
        // anonymously somewhere inside the pipeline.
        let assay = Assay::new(
            crude_curve(),
            GravitySpec::ConstantWatsonK { bulk_sg: 1.15 },
        )
        .unwrap();
        let err = assay
            .characterize(&CutSpec::EqualVolume { n: 10 })
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("cut ") && msg.contains("Watson K"),
            "error should name the cut and the cause, got: {msg}"
        );
    }

    #[test]
    fn interpolate_flat_does_not_extrapolate() {
        // A gravity curve extrapolated past its ends can go negative, which
        // would poison every correlation with no obvious symptom.
        let xs = [0.2, 0.8];
        let ys = [0.7, 0.9];
        assert!((interpolate_flat(&xs, &ys, 0.0) - 0.7).abs() < 1e-12);
        assert!((interpolate_flat(&xs, &ys, 1.0) - 0.9).abs() < 1e-12);
        assert!((interpolate_flat(&xs, &ys, 0.5) - 0.8).abs() < 1e-12);
    }

    #[test]
    fn rackett_compressibility_is_in_the_hydrocarbon_range() {
        for omega in [0.0, 0.3, 0.6, 1.0] {
            let z = rackett_zra(omega);
            assert!((0.19..0.30).contains(&z), "Zra {z} at ω = {omega}");
        }
    }
}
