//! IAPWS-IF97 steam tables across the FFI boundary (M13's `vle-steam`).
//!
//! Thin free-function wrappers over `vle_steam`'s constructors — this set
//! alone powers a complete steam-table iPhone app, which is why steam was
//! sequenced before the iOS milestone. Each function converts the engine's
//! `SteamState` / `SatProps` into a flat [`SteamStateData`] /
//! [`SatPropsData`] record for Swift.
//!
//! ## Units (mass basis — classic printed-steam-table units)
//!
//! `t` **K** · `p` **kPa absolute** · `v` **m³/kg** · `rho` **kg/m³** ·
//! `u`, `h` **kJ/kg** · `s`, `cp`, `cv` **kJ/(kg·K)** · `w` **m/s** ·
//! quality `x` dimensionless (mass fraction vapor).

use crate::error::VleFfiError;
use vle_thermo::steam;

/// IF97 region a state resolved to (mirrors `vle_steam::Region`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SteamRegion {
    /// Region 1 — compressed / subcooled liquid.
    One,
    /// Region 2 — superheated vapor up to 1073.15 K.
    Two,
    /// Region 3 — near-critical, Helmholtz f(ρ,T) formulation.
    Three,
    /// Region 4 — on the saturation line (two-phase).
    Saturated,
    /// Region 5 — high-temperature steam, 1073.15–2273.15 K.
    Five,
}

impl From<steam::Region> for SteamRegion {
    fn from(r: steam::Region) -> Self {
        match r {
            steam::Region::One => SteamRegion::One,
            steam::Region::Two => SteamRegion::Two,
            steam::Region::Three => SteamRegion::Three,
            steam::Region::Saturated => SteamRegion::Saturated,
            steam::Region::Five => SteamRegion::Five,
        }
    }
}

/// Qualitative phase of a steam state (mirrors `vle_steam::Phase`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SteamPhase {
    /// Single-phase liquid (compressed or saturated liquid).
    Liquid,
    /// Single-phase vapor (superheated or saturated vapor).
    Vapor,
    /// Two-phase liquid + vapor mixture (inside the dome).
    TwoPhase,
    /// Supercritical fluid (T ≥ Tc and P ≥ Pc).
    Supercritical,
}

impl From<steam::Phase> for SteamPhase {
    fn from(p: steam::Phase) -> Self {
        match p {
            steam::Phase::Liquid => SteamPhase::Liquid,
            steam::Phase::Vapor => SteamPhase::Vapor,
            steam::Phase::TwoPhase => SteamPhase::TwoPhase,
            steam::Phase::Supercritical => SteamPhase::Supercritical,
        }
    }
}

/// A fully-resolved water/steam state at one point.
///
/// Units as in the module docs. For two-phase states `cp`, `cv`, and `w`
/// are **NaN** (undefined inside the dome) and `quality` carries the vapor
/// mass fraction; single-phase states have `quality = nil`.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SteamStateData {
    /// Temperature, **K**.
    pub t: f64,
    /// Pressure, **kPa absolute**.
    pub p: f64,
    /// IF97 region the point resolved to.
    pub region: SteamRegion,
    /// Qualitative phase.
    pub phase: SteamPhase,
    /// Vapor quality (mass fraction vapor) if two-phase, else `nil`.
    pub quality: Option<f64>,
    /// Specific volume, **m³/kg**.
    pub v: f64,
    /// Density, **kg/m³**.
    pub rho: f64,
    /// Specific internal energy, **kJ/kg**.
    pub u: f64,
    /// Specific enthalpy, **kJ/kg**.
    pub h: f64,
    /// Specific entropy, **kJ/(kg·K)**.
    pub s: f64,
    /// Isobaric heat capacity, **kJ/(kg·K)** (NaN if two-phase).
    pub cp: f64,
    /// Isochoric heat capacity, **kJ/(kg·K)** (NaN if two-phase).
    pub cv: f64,
    /// Speed of sound, **m/s** (NaN if two-phase).
    pub w: f64,
}

impl From<steam::SteamState> for SteamStateData {
    fn from(s: steam::SteamState) -> Self {
        SteamStateData {
            t: s.t,
            p: s.p,
            region: s.region.into(),
            phase: s.phase.into(),
            quality: s.x,
            v: s.v,
            rho: s.rho,
            u: s.u,
            h: s.h,
            s: s.s,
            cp: s.cp,
            cv: s.cv,
            w: s.w,
        }
    }
}

/// One saturation-table row (the classic printed-steam-table entry).
///
/// Subscript `f` = saturated liquid, `g` = saturated vapor, `fg` = the
/// vapor−liquid difference (`h_fg` is the latent heat). Units as in the
/// module docs.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SatPropsData {
    /// Saturation temperature, **K**.
    pub t: f64,
    /// Saturation pressure, **kPa absolute**.
    pub p: f64,
    /// Saturated-liquid specific volume, **m³/kg**.
    pub v_f: f64,
    /// Saturated-vapor specific volume, **m³/kg**.
    pub v_g: f64,
    /// Saturated-liquid enthalpy, **kJ/kg**.
    pub h_f: f64,
    /// Saturated-vapor enthalpy, **kJ/kg**.
    pub h_g: f64,
    /// Latent heat of vaporization h_g − h_f, **kJ/kg**.
    pub h_fg: f64,
    /// Saturated-liquid entropy, **kJ/(kg·K)**.
    pub s_f: f64,
    /// Saturated-vapor entropy, **kJ/(kg·K)**.
    pub s_g: f64,
    /// Entropy of vaporization s_g − s_f, **kJ/(kg·K)**.
    pub s_fg: f64,
    /// Saturated-liquid internal energy, **kJ/kg**.
    pub u_f: f64,
    /// Saturated-vapor internal energy, **kJ/kg**.
    pub u_g: f64,
}

impl From<steam::SatProps> for SatPropsData {
    fn from(s: steam::SatProps) -> Self {
        SatPropsData {
            t: s.t,
            p: s.p,
            v_f: s.v_f,
            v_g: s.v_g,
            h_f: s.h_f,
            h_g: s.h_g,
            h_fg: s.h_fg,
            s_f: s.s_f,
            s_g: s.s_g,
            s_fg: s.s_fg,
            u_f: s.u_f,
            u_g: s.u_g,
        }
    }
}

/// State from temperature and pressure. `t` **K**, `p` **kPa absolute**.
///
/// # Errors
/// [`VleFfiError::Steam`] if `(t, p)` is outside the IF97 envelope or lands
/// exactly on the saturation line (use [`steam_tx`]/[`steam_px`] there).
#[uniffi::export]
pub fn steam_tp(t: f64, p: f64) -> Result<SteamStateData, VleFfiError> {
    Ok(steam::SteamState::tp(t, p)?.into())
}

/// Two-phase state from temperature and quality. `t` **K**, `x` in `[0, 1]`
/// (mass fraction vapor).
///
/// # Errors
/// [`VleFfiError::Steam`] if `t` is outside the saturation range
/// (273.15–647.096 K) or `x` outside `[0, 1]`.
#[uniffi::export]
pub fn steam_tx(t: f64, x: f64) -> Result<SteamStateData, VleFfiError> {
    Ok(steam::SteamState::tx(t, x)?.into())
}

/// Two-phase state from pressure and quality. `p` **kPa absolute**, `x` in
/// `[0, 1]` (mass fraction vapor).
///
/// # Errors
/// [`VleFfiError::Steam`] if `p` is outside the saturation range or `x`
/// outside `[0, 1]`.
#[uniffi::export]
pub fn steam_px(p: f64, x: f64) -> Result<SteamStateData, VleFfiError> {
    Ok(steam::SteamState::px(p, x)?.into())
}

/// State from pressure and enthalpy (the boiler/turbine workhorse).
/// `p` **kPa absolute**, `h` **kJ/kg**.
///
/// # Errors
/// [`VleFfiError::Steam`] if the `(p, h)` point is outside IF97 or the
/// inner temperature solve fails to converge.
#[uniffi::export]
pub fn steam_ph(p: f64, h: f64) -> Result<SteamStateData, VleFfiError> {
    Ok(steam::SteamState::ph(p, h)?.into())
}

/// State from pressure and entropy (isentropic-process endpoint).
/// `p` **kPa absolute**, `s` **kJ/(kg·K)**.
///
/// # Errors
/// [`VleFfiError::Steam`] if the `(p, s)` point is outside IF97 or the
/// inner temperature solve fails to converge.
#[uniffi::export]
pub fn steam_ps(p: f64, s: f64) -> Result<SteamStateData, VleFfiError> {
    Ok(steam::SteamState::ps(p, s)?.into())
}

/// Full saturation-table row at temperature `t` **K**.
///
/// # Errors
/// [`VleFfiError::Steam`] if `t` is outside 273.15–647.096 K.
#[uniffi::export]
pub fn steam_sat_t(t: f64) -> Result<SatPropsData, VleFfiError> {
    Ok(steam::sat_t(t)?.into())
}

/// Full saturation-table row at pressure `p` **kPa absolute**.
///
/// # Errors
/// [`VleFfiError::Steam`] if `p` is outside the saturation-pressure range
/// (≈0.6117–22064 kPa).
#[uniffi::export]
pub fn steam_sat_p(p: f64) -> Result<SatPropsData, VleFfiError> {
    Ok(steam::sat_p(p)?.into())
}

/// Latent heat of vaporization h_fg at temperature `t` **K**, in **kJ/kg**.
///
/// # Errors
/// [`VleFfiError::Steam`] if `t` is outside the saturation range.
#[uniffi::export]
pub fn steam_latent_heat(t: f64) -> Result<f64, VleFfiError> {
    Ok(steam::latent_heat(t)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // IAPWS-IF97 Table 5 verification point: region 1, T = 300 K,
    // P = 3 MPa = 3000 kPa → v = 0.100215168e-2 m³/kg, h = 115.331273 kJ/kg.
    #[test]
    fn if97_region1_verification_point() {
        let s = steam_tp(300.0, 3000.0).unwrap();
        assert_eq!(s.region, SteamRegion::One);
        assert_eq!(s.phase, SteamPhase::Liquid);
        assert!((s.v - 0.100215168e-2).abs() < 1e-10, "v = {}", s.v);
        assert!((s.h - 115.331273).abs() < 1e-5, "h = {}", s.h);
        assert!(s.quality.is_none());
    }

    // The kitchen benchmark: water boils at ~373.12 K at 1 atm, with a
    // latent heat of ~2256.5 kJ/kg.
    #[test]
    fn boiling_at_one_atmosphere() {
        let row = steam_sat_p(101.325).unwrap();
        assert!((row.t - 373.12).abs() < 0.05, "t_sat = {}", row.t);
        assert!((row.h_fg - 2256.5).abs() < 1.0, "h_fg = {}", row.h_fg);
    }

    #[test]
    fn two_phase_state_carries_quality() {
        let s = steam_px(101.325, 0.5).unwrap();
        assert_eq!(s.phase, SteamPhase::TwoPhase);
        assert_eq!(s.quality, Some(0.5));
        assert!(s.cp.is_nan(), "cp undefined inside the dome");
    }

    #[test]
    fn out_of_range_maps_to_steam_error() {
        assert!(matches!(
            steam_tp(5000.0, 101.325),
            Err(VleFfiError::Steam { .. })
        ));
    }
}
