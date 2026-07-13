//! Component data across the wasm boundary + the bundled component database.
//!
//! [`ComponentData`] is the flat mirror of the engine's
//! `vle_thermo::types::Component`. On the JavaScript side it is a **plain
//! object** with camelCase keys (`{ name: "water", tc: 647.096, … }`):
//! `serde` derives the (de)serialization and `serde-wasm-bindgen` does the
//! JS conversion in the exported shims.
//!
//! The same record shape is used in both directions: returned by
//! [`db_component`] lookups and accepted by `VleSystem` constructors, so a
//! JS app can do `new VleSystem([dbComponent("water"), …], …)` or build a
//! fully custom component from an object literal — every field except
//! `name`, `tc`, `pc`, and `omega` may be omitted (zero / empty / Antoine
//! defaults apply, the engine's "not available" convention).

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::error::VleWasmError;
use vle_thermo::saturation::SatPressureModel;
use vle_thermo::types::Component;

/// Parse a saturation-model name (the `satModel` string field).
///
/// Case-insensitive: `"antoine"`, `"riedel"`, `"muller"`, `"rpm"`,
/// `"polynomial"`, `"maxwell"`.
pub(crate) fn parse_sat_model(s: &str) -> Result<SatPressureModel, VleWasmError> {
    match s.to_ascii_lowercase().as_str() {
        "antoine" => Ok(SatPressureModel::Antoine),
        "riedel" => Ok(SatPressureModel::Riedel),
        "muller" => Ok(SatPressureModel::Muller),
        "rpm" => Ok(SatPressureModel::RPM),
        "polynomial" => Ok(SatPressureModel::Polynomial),
        "maxwell" => Ok(SatPressureModel::Maxwell),
        _ => Err(VleWasmError::InvalidInput {
            message: format!(
                "unknown satModel {s:?} (expected antoine, riedel, muller, rpm, polynomial, or maxwell)"
            ),
        }),
    }
}

/// The inverse mapping, for records returned to JavaScript.
fn sat_model_name(m: SatPressureModel) -> String {
    match m {
        SatPressureModel::Antoine => "antoine",
        SatPressureModel::Riedel => "riedel",
        SatPressureModel::Muller => "muller",
        SatPressureModel::RPM => "rpm",
        SatPressureModel::Polynomial => "polynomial",
        SatPressureModel::Maxwell => "maxwell",
    }
    .to_string()
}

fn default_sat_model() -> String {
    "antoine".to_string()
}

/// One pure component, in **canonical engine units**.
///
/// Field units: `tc` **K**, `pc` **kPa absolute**, `vc` **cm³/mol**,
/// `zc`/`omega` dimensionless, `tb` **K**, `mw` **kg/kmol**, `cpCoeffs`
/// (5 ideal-gas Cp/R polynomial coefficients, dimensionless), `psatCoeffs`
/// (correlation coefficients for `satModel`), `dipoleMoment` **debye**,
/// `solubilityParam` **(cal/cm³)^0.5**, `liquidVolume` **cm³/mol**,
/// `zra`/`omegaSrk`/`mPolar`/`nPolar`/`gPolar`/`prsvK1` dimensionless model
/// parameters. Zero means "not available" for optional fields, same
/// convention as the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentData {
    /// Component name (matches the database's canonical casing).
    pub name: String,
    /// Critical temperature, **K**.
    pub tc: f64,
    /// Critical pressure, **kPa absolute**.
    pub pc: f64,
    /// Acentric factor ω, dimensionless.
    pub omega: f64,
    /// Critical molar volume, **cm³/mol**.
    #[serde(default)]
    pub vc: f64,
    /// Critical compressibility factor, dimensionless.
    #[serde(default)]
    pub zc: f64,
    /// Normal boiling temperature, **K**.
    #[serde(default)]
    pub tb: f64,
    /// Molar mass, **kg/kmol**.
    #[serde(default)]
    pub mw: f64,
    /// Ideal-gas Cp/R polynomial coefficients (exactly 5, or empty for
    /// "none available").
    #[serde(default)]
    pub cp_coeffs: Vec<f64>,
    /// Saturation-pressure coefficients for `satModel` (e.g. reduced
    /// Antoine `[a1, a2, a3]`).
    #[serde(default)]
    pub psat_coeffs: Vec<f64>,
    /// Dipole moment, **debye**.
    #[serde(default)]
    pub dipole_moment: f64,
    /// Solubility parameter δ, **(cal/cm³)^0.5** (Scatchard-Hildebrand).
    #[serde(default)]
    pub solubility_param: f64,
    /// Liquid molar volume, **cm³/mol** (Wilson, Poynting).
    #[serde(default)]
    pub liquid_volume: f64,
    /// Rackett compressibility ZRA, dimensionless.
    #[serde(default)]
    pub zra: f64,
    /// SRK-specific acentric factor, dimensionless.
    #[serde(default)]
    pub omega_srk: f64,
    /// Polar parameter m (Mathias-type α functions), dimensionless.
    #[serde(default)]
    pub m_polar: f64,
    /// Polar parameter n, dimensionless.
    #[serde(default)]
    pub n_polar: f64,
    /// Polar parameter g, dimensionless.
    #[serde(default)]
    pub g_polar: f64,
    /// PRSV κ₁ parameter, dimensionless.
    #[serde(default)]
    pub prsv_k1: f64,
    /// Saturation-pressure correlation name this component's `psatCoeffs`
    /// fit (see [`parse_sat_model`] for the accepted strings).
    #[serde(default = "default_sat_model")]
    pub sat_model: String,
}

impl From<Component> for ComponentData {
    fn from(c: Component) -> Self {
        ComponentData {
            name: c.name,
            tc: c.tc,
            pc: c.pc,
            omega: c.omega,
            vc: c.vc,
            zc: c.zc,
            tb: c.tb,
            mw: c.mw,
            cp_coeffs: c.cp_coeffs.to_vec(),
            psat_coeffs: c.psat_coeffs,
            dipole_moment: c.dipole_moment,
            solubility_param: c.solubility_param,
            liquid_volume: c.liquid_volume,
            zra: c.zra,
            omega_srk: c.omega_srk,
            m_polar: c.m_polar,
            n_polar: c.n_polar,
            g_polar: c.g_polar,
            prsv_k1: c.prsv_k1,
            sat_model: sat_model_name(c.sat_model),
        }
    }
}

// `TryFrom` (not `From`) because the record can be structurally invalid:
// `cpCoeffs` must have exactly 5 entries or be empty, and `satModel` must
// name a known correlation.
impl TryFrom<ComponentData> for Component {
    type Error = VleWasmError;

    fn try_from(d: ComponentData) -> Result<Component, VleWasmError> {
        let cp_coeffs = match d.cp_coeffs.len() {
            0 => [0.0; 5],
            5 => [
                d.cp_coeffs[0],
                d.cp_coeffs[1],
                d.cp_coeffs[2],
                d.cp_coeffs[3],
                d.cp_coeffs[4],
            ],
            n => {
                return Err(VleWasmError::InvalidInput {
                    message: format!(
                        "component {:?}: cpCoeffs must have 0 or 5 entries (got {n})",
                        d.name
                    ),
                });
            }
        };
        Ok(Component {
            name: d.name,
            tc: d.tc,
            pc: d.pc,
            vc: d.vc,
            zc: d.zc,
            omega: d.omega,
            tb: d.tb,
            mw: d.mw,
            cp_coeffs,
            psat_coeffs: d.psat_coeffs,
            dipole_moment: d.dipole_moment,
            solubility_param: d.solubility_param,
            liquid_volume: d.liquid_volume,
            zra: d.zra,
            omega_srk: d.omega_srk,
            m_polar: d.m_polar,
            n_polar: d.n_polar,
            g_polar: d.g_polar,
            prsv_k1: d.prsv_k1,
            sat_model: parse_sat_model(&d.sat_model)?,
        })
    }
}

/// Names of every component bundled in the database, sorted alphabetically.
///
/// Same catalogue as `vle_thermo::db::available()` / Python's
/// `vle.components.available()` — all surfaces read one JSON file.
#[wasm_bindgen(js_name = dbAvailable)]
pub fn db_available() -> Vec<String> {
    vle_thermo::db::available()
}

/// Look up a bundled component by name (case-insensitive, trimmed).
///
/// Returns a plain object in canonical engine units (see [`ComponentData`]).
///
/// # Errors
/// Throws `Error("component not found …")` if the name is not in the
/// bundled catalogue — check spelling against [`db_available`] (no aliases:
/// `"H2O"` is a miss, `"water"` is a hit).
#[wasm_bindgen(js_name = dbComponent)]
pub fn db_component(name: String) -> Result<JsValue, JsError> {
    let data = vle_thermo::db::component(&name)
        .map(ComponentData::from)
        .ok_or(VleWasmError::NotFound { name })?;
    serde_wasm_bindgen::to_value(&data).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_roundtrips_through_the_record() {
        let water: ComponentData = vle_thermo::db::component("Water")
            .expect("water is bundled")
            .into();
        // IAPWS critical point, engine units (K, kPa).
        assert!((water.tc - 647.0).abs() < 1.0, "tc = {}", water.tc);
        assert!((water.pc - 22064.0).abs() < 100.0, "pc = {}", water.pc);
        // Record → engine Component → record must be lossless.
        let engine: Component = water.clone().try_into().unwrap();
        let back = ComponentData::from(engine);
        assert_eq!(water, back);
    }

    #[test]
    fn object_literal_defaults_apply() {
        // The JS story: `{name, tc, pc, omega}` alone is a valid component.
        // serde_json stands in for serde-wasm-bindgen — same Deserialize.
        let d: ComponentData = serde_json::from_value(serde_json::json!({
            "name": "custom", "tc": 500.0, "pc": 3000.0, "omega": 0.25
        }))
        .unwrap();
        assert_eq!(d.sat_model, "antoine");
        assert!(d.psat_coeffs.is_empty());
        let c: Component = d.try_into().unwrap();
        assert_eq!(c.cp_coeffs, [0.0; 5]);
    }

    #[test]
    fn bad_cp_coeffs_rejected() {
        let mut c: ComponentData = vle_thermo::db::component("Water").unwrap().into();
        c.cp_coeffs = vec![1.0, 2.0]; // neither 0 nor 5 entries
        assert!(matches!(
            Component::try_from(c),
            Err(VleWasmError::InvalidInput { .. })
        ));
    }

    #[test]
    fn unknown_sat_model_rejected() {
        assert!(matches!(
            parse_sat_model("wagner"),
            Err(VleWasmError::InvalidInput { .. })
        ));
    }
}
