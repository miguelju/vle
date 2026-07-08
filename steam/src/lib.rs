//! # `vle-steam` — IAPWS-IF97 industrial water/steam properties
//!
//! Steam tables are the single most-used thermodynamic reference in chemical
//! engineering practice — sizing reboilers and condensers, steam-header
//! balances, flash-steam recovery, turbine and valve calculations. Every
//! printed steam table in a modern handbook is *computed from* one open
//! standard, the **IAPWS Industrial Formulation 1997 (IF97)**, so this crate
//! implements that standard directly rather than interpolating tabulated data.
//!
//! This is, in effect, "VLE for water only": a companion to the multicomponent
//! [`vle-thermo`](https://crates.io/crates/vle-thermo) engine, kept as a
//! separate crate because IF97 is a self-contained formulation with zero
//! coupling to the mixture-EOS machinery — and dependency-free (pure `f64`
//! math), so it stays trivially portable to a static-library / FFI build.
//!
//! ## What is implemented
//!
//! | Region | Physical domain | Equation |
//! |---|---|---|
//! | 1 | Compressed / subcooled liquid, 273.15–623.15 K | Gibbs `g(p,T)` |
//! | 2 | Superheated vapor, ≤ 1073.15 K, ≤ 100 MPa | Gibbs `g(p,T)` |
//! | 3 | Near-critical, 623.15–863.15 K above B23 | Helmholtz `f(ρ,T)` |
//! | 4 | **Saturation line**, 273.15–647.096 K | closed-form both ways |
//! | 5 | High-T steam, 1073.15–2273.15 K, ≤ 50 MPa | Gibbs `g(p,T)` |
//!
//! Plus backward equations `T(p,h)` and `T(p,s)` for regions 1–2 (making
//! PH/PS flash essentially non-iterative), and analytic derivatives
//! throughout (never finite differences).
//!
//! ## Units
//!
//! The public API takes **T in K** and **P in kPa absolute** (the repo canon,
//! matching [`vle-thermo`]) and returns **mass-basis** properties — what every
//! steam-table user expects:
//!
//! - specific enthalpy `h`, internal energy `u` — **kJ/kg**
//! - specific entropy `s`, heat capacities `cp`, `cv` — **kJ/(kg·K)**
//! - specific volume `v` — **m³/kg**; density `ρ` — **kg/m³**
//! - speed of sound `w` — **m/s**
//!
//! Internally the IF97 equations run in their native units (MPa, kJ/kg);
//! conversion happens once at the API boundary. Every function documents its
//! units per the project's units rule.
//!
//! ## References
//!
//! - IAPWS. *Revised Release on the IAPWS Industrial Formulation 1997 for the
//!   Thermodynamic Properties of Water and Steam*; IAPWS R7-97(2012), 2012.
//! - Wagner, W.; Kretzschmar, H.-J. *International Steam Tables*, 3rd ed.;
//!   Springer: Berlin, 2019.

mod backward;
mod coefficients;
mod props;
mod region1;
mod region2;
mod region3;
pub mod region4;
mod region5;
pub mod regions;
mod solve;
mod state;

pub use props::Props;
pub use state::{
    MolarState, Phase, SatProps, SteamState, latent_heat, latent_heat_at_p, sat_p, sat_t,
};

use core::fmt;

// ── Fundamental constants (IAPWS R7-97(2012) §1) ─────────────────────────

/// Specific gas constant of ordinary water, **kJ/(kg·K)**.
///
/// `R = 0.461526 kJ/(kg·K)` — the IF97 value (Eq. 1). Note this is the
/// *specific* (mass-basis) constant, not the molar `R` the engine uses.
pub const R: f64 = 0.461526;

/// Critical temperature of water, **K** (R7-97 §1).
pub const T_C: f64 = 647.096;

/// Critical pressure of water, **MPa** (R7-97 §1).
pub const P_C_MPA: f64 = 22.064;

/// Critical density of water, **kg/m³** (R7-97 §1).
pub const RHO_C: f64 = 322.0;

/// Molar mass of water, **kg/kmol** (IAPWS value) — used by the `.molar()`
/// view to convert mass-basis properties to the engine's molar canon.
pub const M_WATER: f64 = 18.015268;

// ── Error type ───────────────────────────────────────────────────────────

/// Errors returned by the state constructors and property queries.
///
/// The crate is dependency-free, so this hand-rolls [`fmt::Display`] and
/// [`std::error::Error`] instead of deriving them with `thiserror`.
#[derive(Debug, Clone, PartialEq)]
pub enum SteamError {
    /// A `(T, p)` (or derived) state point fell outside the IF97 validity
    /// envelope. Carries the offending temperature (**K**) and pressure
    /// (**kPa**).
    OutOfRange {
        /// Temperature in **K**.
        t: f64,
        /// Pressure in **kPa absolute**.
        p: f64,
    },
    /// A quality (vapor fraction) argument was outside `0 ≤ x ≤ 1`.
    InvalidQuality(f64),
    /// A saturation query was requested outside `273.15 K ≤ T ≤ 647.096 K`
    /// (or the equivalent pressure range).
    OutOfSaturationRange(f64),
    /// An iterative inner solve (e.g. region-3 density) failed to converge.
    NoConvergence(&'static str),
}

impl fmt::Display for SteamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SteamError::OutOfRange { t, p } => {
                write!(f, "state (T={t} K, P={p} kPa) is outside the IF97 range")
            }
            SteamError::InvalidQuality(x) => {
                write!(f, "quality x={x} is outside [0, 1]")
            }
            SteamError::OutOfSaturationRange(v) => {
                write!(f, "saturation query {v} is outside 273.15 K … 647.096 K")
            }
            SteamError::NoConvergence(what) => {
                write!(f, "iterative solve did not converge: {what}")
            }
        }
    }
}

impl std::error::Error for SteamError {}

// ── Unit-conversion helpers at the API boundary ──────────────────────────

/// kPa → MPa (IF97's native pressure unit).
#[inline]
pub(crate) fn kpa_to_mpa(p_kpa: f64) -> f64 {
    p_kpa * 1e-3
}

/// MPa → kPa (repo-canonical pressure unit).
#[inline]
pub(crate) fn mpa_to_kpa(p_mpa: f64) -> f64 {
    p_mpa * 1e3
}

// ── Re-exports: the flat public surface ──────────────────────────────────

pub use region4::{d_psat_d_t as psat_derivative_mpa, psat as psat_mpa, tsat as tsat_mpa};
pub use regions::{Region, region_of};

/// Saturation pressure at temperature `t`.
///
/// # Arguments
/// * `t` — Temperature in **K**, valid on `273.15 ≤ t ≤ 647.096`.
///
/// # Returns
/// Saturation pressure in **kPa absolute** (repo canon).
pub fn psat(t: f64) -> Result<f64, SteamError> {
    if !(region4::T_MIN..=region4::T_MAX).contains(&t) {
        return Err(SteamError::OutOfSaturationRange(t));
    }
    Ok(mpa_to_kpa(region4::psat(t)))
}

/// Saturation temperature at pressure `p`.
///
/// # Arguments
/// * `p` — Pressure in **kPa absolute**, valid between the triple- and
///   critical-point saturation pressures.
///
/// # Returns
/// Saturation temperature in **K**.
pub fn tsat(p: f64) -> Result<f64, SteamError> {
    let p_mpa = kpa_to_mpa(p);
    if !(region4::P_MIN_MPA..=region4::P_MAX_MPA).contains(&p_mpa) {
        return Err(SteamError::OutOfSaturationRange(p));
    }
    Ok(region4::tsat(p_mpa))
}

/// Analytic derivative `dPsat/dT` of the saturation curve.
///
/// # Arguments
/// * `t` — Temperature in **K**.
///
/// # Returns
/// `dPsat/dT` in **kPa/K**.
pub fn psat_derivative(t: f64) -> Result<f64, SteamError> {
    if !(region4::T_MIN..=region4::T_MAX).contains(&t) {
        return Err(SteamError::OutOfSaturationRange(t));
    }
    Ok(mpa_to_kpa(region4::d_psat_d_t(t)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn public_psat_in_kpa() {
        // Table 35 value, converted to kPa: 0.353658941e-2 MPa = 3.53658941 kPa.
        assert_relative_eq!(psat(300.0).unwrap(), 3.53658941, max_relative = 1e-8);
    }

    #[test]
    fn public_tsat_in_kpa() {
        // Table 36: Tsat(0.1 MPa = 100 kPa) = 372.755919 K.
        assert_relative_eq!(tsat(100.0).unwrap(), 372.755919, max_relative = 1e-8);
    }

    #[test]
    fn saturation_range_guards() {
        assert!(matches!(
            psat(260.0),
            Err(SteamError::OutOfSaturationRange(_))
        ));
        assert!(matches!(
            psat(700.0),
            Err(SteamError::OutOfSaturationRange(_))
        ));
    }

    /// Region-boundary continuity: neighbouring regions must agree on `v, h, s`
    /// to IF97's stated inter-region consistency (well within 0.1%).
    #[test]
    fn region_boundary_continuity() {
        // 2 ↔ 3 seam: the B23 line at T = 700 K.
        {
            let t = 700.0;
            let p = mpa_to_kpa(regions::b23_p(t));
            let r2 = region2::props(t, p);
            let r3 = region3::props_tp(t, p).unwrap();
            assert_relative_eq!(r2.v, r3.v, max_relative = 1e-3);
            assert_relative_eq!(r2.h, r3.h, max_relative = 1e-3);
            assert_relative_eq!(r2.s, r3.s, max_relative = 1e-3);
        }
        // 1 ↔ 3 seam: the 623.15 K isotherm at 50 MPa.
        {
            let (t, p) = (623.15, 50_000.0);
            let r1 = region1::props(t, p);
            let r3 = region3::props_tp(t, p).unwrap();
            assert_relative_eq!(r1.v, r3.v, max_relative = 1e-3);
            assert_relative_eq!(r1.h, r3.h, max_relative = 1e-3);
            assert_relative_eq!(r1.s, r3.s, max_relative = 1e-3);
        }
        // 2 ↔ 5 seam: the 1073.15 K isotherm at 10 MPa.
        {
            let (t, p) = (1073.15, 10_000.0);
            let r2 = region2::props(t, p);
            let r5 = region5::props(t, p);
            assert_relative_eq!(r2.v, r5.v, max_relative = 1e-3);
            assert_relative_eq!(r2.h, r5.h, max_relative = 1e-3);
            assert_relative_eq!(r2.s, r5.s, max_relative = 1e-3);
        }
    }
}
