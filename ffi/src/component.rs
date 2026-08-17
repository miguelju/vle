//! Component data across the FFI boundary + the bundled component database.
//!
//! [`ComponentData`] is the flat, owned mirror of the engine's
//! `vle_thermo::types::Component`. UniFFI's `#[derive(uniffi::Record)]`
//! turns it into a Swift `struct` with memberwise fields — a pure *value*
//! (copied across the boundary), unlike [`crate::system::VleSystem`] which
//! is a by-reference object.
//!
//! The same record type is used in both directions: returned by
//! [`db_component`] lookups and accepted by `VleSystem` constructors, so a
//! Swift app can do `VleSystem(components: [try dbComponent(name: "water"),
//! …])` or build a fully custom component from literals.

use crate::error::VleFfiError;
use vle_thermo::saturation::SatPressureModel;
use vle_thermo::types::Component;

/// Saturation-pressure correlation selector (mirrors the engine's
/// `SatPressureModel`). Governs how Psat(T) is evaluated on the γ-φ path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SatModel {
    /// Antoine: ln(P/Pc) = a₁ − a₂/(a₃ + T). Ref (4), TERMOI.PAS.
    Antoine,
    /// Riedel corresponding-states correlation (Tc, Pc, ω).
    Riedel,
    /// Müller reduced-property correlation.
    Muller,
    /// Reduced-pressure model (corresponding states, Tr/Pr).
    Rpm,
    /// DIPPR-style database polynomial.
    Polynomial,
    /// Maxwell equal-area construction (needs a cubic EOS; exact).
    Maxwell,
}

impl From<SatModel> for SatPressureModel {
    fn from(m: SatModel) -> Self {
        match m {
            SatModel::Antoine => SatPressureModel::Antoine,
            SatModel::Riedel => SatPressureModel::Riedel,
            SatModel::Muller => SatPressureModel::Muller,
            SatModel::Rpm => SatPressureModel::RPM,
            SatModel::Polynomial => SatPressureModel::Polynomial,
            SatModel::Maxwell => SatPressureModel::Maxwell,
        }
    }
}

impl From<SatPressureModel> for SatModel {
    fn from(m: SatPressureModel) -> Self {
        match m {
            SatPressureModel::Antoine => SatModel::Antoine,
            SatPressureModel::Riedel => SatModel::Riedel,
            SatPressureModel::Muller => SatModel::Muller,
            SatPressureModel::RPM => SatModel::Rpm,
            SatPressureModel::Polynomial => SatModel::Polynomial,
            SatPressureModel::Maxwell => SatModel::Maxwell,
        }
    }
}

/// One pure component, in **canonical engine units**.
///
/// Field units: `tc` **K**, `pc` **kPa abs**, `vc` **cm³/mol**, `zc`/`omega`
/// dimensionless, `tb` **K**, `mw` **kg/kmol**, `cp_coeffs` (5 ideal-gas
/// Cp/R polynomial coefficients, dimensionless), `psat_coeffs` (correlation
/// coefficients for `sat_model`), `dipole_moment` **debye**,
/// `solubility_param` **(cal/cm³)^0.5**, `liquid_volume` **cm³/mol**,
/// `zra`/`omega_srk`/`m_polar`/`n_polar`/`g_polar`/`prsv_k1` dimensionless
/// model parameters. Zero means "not available" for optional fields, same
/// convention as the engine.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ComponentData {
    /// Component name (matches the database's canonical casing).
    pub name: String,
    /// Critical temperature, **K**.
    pub tc: f64,
    /// Critical pressure, **kPa absolute**.
    pub pc: f64,
    /// Critical molar volume, **cm³/mol**.
    pub vc: f64,
    /// Critical compressibility factor, dimensionless.
    pub zc: f64,
    /// Acentric factor ω, dimensionless.
    pub omega: f64,
    /// Normal boiling temperature, **K**.
    pub tb: f64,
    /// Molar mass, **kg/kmol**.
    pub mw: f64,
    /// Ideal-gas Cp/R polynomial coefficients (exactly 5, or empty for
    /// "none available").
    pub cp_coeffs: Vec<f64>,
    /// Saturation-pressure coefficients for `sat_model` (e.g. reduced
    /// Antoine `[a1, a2, a3]`).
    pub psat_coeffs: Vec<f64>,
    /// Dipole moment, **debye**.
    pub dipole_moment: f64,
    /// Solubility parameter δ, **(cal/cm³)^0.5** (Scatchard-Hildebrand).
    pub solubility_param: f64,
    /// Liquid molar volume, **cm³/mol** (Wilson, Poynting).
    pub liquid_volume: f64,
    /// Rackett compressibility ZRA, dimensionless.
    pub zra: f64,
    /// SRK-specific acentric factor, dimensionless.
    pub omega_srk: f64,
    /// Polar parameter m (Mathias-type α functions), dimensionless.
    pub m_polar: f64,
    /// Polar parameter n, dimensionless.
    pub n_polar: f64,
    /// Polar parameter g, dimensionless.
    pub g_polar: f64,
    /// PRSV κ₁ parameter, dimensionless.
    pub prsv_k1: f64,
    /// Watson characterization factor K_W, dimensionless (M20; 0 = unknown).
    pub watson_k: f64,
    /// Saturation-pressure correlation this component's `psat_coeffs` fit.
    pub sat_model: SatModel,
}

impl From<Component> for ComponentData {
    fn from(c: Component) -> Self {
        ComponentData {
            name: c.name,
            tc: c.tc,
            pc: c.pc,
            vc: c.vc,
            zc: c.zc,
            omega: c.omega,
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
            watson_k: c.watson_k,
            sat_model: c.sat_model.into(),
        }
    }
}

// `TryFrom` (not `From`) because the record can be structurally invalid:
// `cp_coeffs` must have exactly 5 entries or be empty.
impl TryFrom<ComponentData> for Component {
    type Error = VleFfiError;

    fn try_from(d: ComponentData) -> Result<Component, VleFfiError> {
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
                return Err(VleFfiError::InvalidInput {
                    message: format!(
                        "component {:?}: cp_coeffs must have 0 or 5 entries (got {n})",
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
            watson_k: d.watson_k,
            sat_model: d.sat_model.into(),
        })
    }
}

/// Names of every component bundled in the database, sorted alphabetically.
///
/// Same catalogue as `vle_thermo::db::available()` / Python's
/// `vle.components.available()` — all three surfaces read one JSON file.
#[uniffi::export]
pub fn db_available() -> Vec<String> {
    vle_thermo::db::available()
}

/// Look up a bundled component by name (case-insensitive, trimmed).
///
/// Returns the component in canonical engine units (see [`ComponentData`]).
///
/// # Errors
/// [`VleFfiError::NotFound`] if the name is not in the bundled catalogue —
/// check spelling against [`db_available`] (no aliases: `"H2O"` is a miss,
/// `"water"` is a hit).
#[uniffi::export]
pub fn db_component(name: String) -> Result<ComponentData, VleFfiError> {
    vle_thermo::db::component(&name)
        .map(ComponentData::from)
        .ok_or(VleFfiError::NotFound { name })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_roundtrips_through_the_record() {
        let water = db_component("Water".into()).expect("water is bundled");
        // IAPWS critical point, engine units (K, kPa).
        assert!((water.tc - 647.0).abs() < 1.0, "tc = {}", water.tc);
        assert!((water.pc - 22064.0).abs() < 100.0, "pc = {}", water.pc);
        // Record → engine Component → record must be lossless.
        let engine: Component = water.clone().try_into().unwrap();
        let back = ComponentData::from(engine);
        assert_eq!(water, back);
    }

    #[test]
    fn unknown_component_is_not_found() {
        assert!(matches!(
            db_component("unobtainium".into()),
            Err(VleFfiError::NotFound { .. })
        ));
    }

    #[test]
    fn bad_cp_coeffs_rejected() {
        let mut c = db_component("Water".into()).unwrap();
        c.cp_coeffs = vec![1.0, 2.0]; // neither 0 nor 5 entries
        assert!(matches!(
            Component::try_from(c),
            Err(VleFfiError::InvalidInput { .. })
        ));
    }
}
