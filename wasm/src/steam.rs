//! IAPWS-IF97 steam tables across the wasm boundary (M13's `vle-steam`).
//!
//! Thin free-function wrappers over `vle_steam`'s constructors — this set
//! alone powers a complete steam-table web app. Each function converts the
//! engine's `SteamState` / `SatProps` into a plain JS object
//! ([`SteamStateData`] / [`SatPropsData`], camelCase keys).
//!
//! ## Units (mass basis — classic printed-steam-table units)
//!
//! `t` **K** · `p` **kPa absolute** · `v` **m³/kg** · `rho` **kg/m³** ·
//! `u`, `h` **kJ/kg** · `s`, `cp`, `cv` **kJ/(kg·K)** · `w` **m/s** ·
//! quality `x` dimensionless (mass fraction vapor).

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::error::VleWasmError;
use vle_thermo::steam;

/// IF97 region name a state resolved to (mirrors `vle_steam::Region`):
/// `"one"` (compressed liquid), `"two"` (superheated vapor), `"three"`
/// (near-critical), `"saturated"` (on the dome), `"five"` (high-T steam).
fn region_name(r: steam::Region) -> &'static str {
    match r {
        steam::Region::One => "one",
        steam::Region::Two => "two",
        steam::Region::Three => "three",
        steam::Region::Saturated => "saturated",
        steam::Region::Five => "five",
    }
}

/// Qualitative phase name (mirrors `vle_steam::Phase`): `"liquid"`,
/// `"vapor"`, `"twoPhase"`, or `"supercritical"`.
fn phase_name(p: steam::Phase) -> &'static str {
    match p {
        steam::Phase::Liquid => "liquid",
        steam::Phase::Vapor => "vapor",
        steam::Phase::TwoPhase => "twoPhase",
        steam::Phase::Supercritical => "supercritical",
    }
}

/// A fully-resolved water/steam state at one point.
///
/// Units as in the module docs. For two-phase states `cp`, `cv`, and `w`
/// are **NaN** (undefined inside the dome) and `quality` carries the vapor
/// mass fraction; single-phase states have `quality: undefined`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamStateData {
    /// Temperature, **K**.
    pub t: f64,
    /// Pressure, **kPa absolute**.
    pub p: f64,
    /// IF97 region the point resolved to (see [`region_name`]).
    pub region: &'static str,
    /// Qualitative phase (see [`phase_name`]).
    pub phase: &'static str,
    /// Vapor quality (mass fraction vapor) if two-phase, else `undefined`.
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
            region: region_name(s.region),
            phase: phase_name(s.phase),
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
/// Subscript `F` = saturated liquid, `G` = saturated vapor, `Fg` = the
/// vapor−liquid difference (`hFg` is the latent heat). Units as in the
/// module docs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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

/// Serialize a record into a plain JS object (shared shim helper).
fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsError::new(&e.to_string()))
}

/// State from temperature and pressure. `t` **K**, `p` **kPa absolute**.
///
/// # Errors
/// Throws if `(t, p)` is outside the IF97 envelope or lands exactly on the
/// saturation line (use [`steam_tx`] / [`steam_px`] there).
#[wasm_bindgen(js_name = steamTp)]
pub fn steam_tp(t: f64, p: f64) -> Result<JsValue, JsError> {
    let s = steam::SteamState::tp(t, p).map_err(VleWasmError::from)?;
    to_js(&SteamStateData::from(s))
}

/// Two-phase state from temperature and quality. `t` **K**, `x` in `[0, 1]`
/// (mass fraction vapor).
///
/// # Errors
/// Throws if `t` is outside the saturation range (273.15–647.096 K) or `x`
/// outside `[0, 1]`.
#[wasm_bindgen(js_name = steamTx)]
pub fn steam_tx(t: f64, x: f64) -> Result<JsValue, JsError> {
    let s = steam::SteamState::tx(t, x).map_err(VleWasmError::from)?;
    to_js(&SteamStateData::from(s))
}

/// Two-phase state from pressure and quality. `p` **kPa absolute**, `x` in
/// `[0, 1]` (mass fraction vapor).
///
/// # Errors
/// Throws if `p` is outside the saturation range or `x` outside `[0, 1]`.
#[wasm_bindgen(js_name = steamPx)]
pub fn steam_px(p: f64, x: f64) -> Result<JsValue, JsError> {
    let s = steam::SteamState::px(p, x).map_err(VleWasmError::from)?;
    to_js(&SteamStateData::from(s))
}

/// State from pressure and enthalpy (the boiler/turbine workhorse).
/// `p` **kPa absolute**, `h` **kJ/kg**.
///
/// # Errors
/// Throws if the `(p, h)` point is outside IF97 or the inner temperature
/// solve fails to converge.
#[wasm_bindgen(js_name = steamPh)]
pub fn steam_ph(p: f64, h: f64) -> Result<JsValue, JsError> {
    let s = steam::SteamState::ph(p, h).map_err(VleWasmError::from)?;
    to_js(&SteamStateData::from(s))
}

/// State from pressure and entropy (isentropic-process endpoint).
/// `p` **kPa absolute**, `s` **kJ/(kg·K)**.
///
/// # Errors
/// Throws if the `(p, s)` point is outside IF97 or the inner temperature
/// solve fails to converge.
#[wasm_bindgen(js_name = steamPs)]
pub fn steam_ps(p: f64, s: f64) -> Result<JsValue, JsError> {
    let st = steam::SteamState::ps(p, s).map_err(VleWasmError::from)?;
    to_js(&SteamStateData::from(st))
}

/// Full saturation-table row at temperature `t` **K**.
///
/// # Errors
/// Throws if `t` is outside 273.15–647.096 K.
#[wasm_bindgen(js_name = steamSatT)]
pub fn steam_sat_t(t: f64) -> Result<JsValue, JsError> {
    let row = steam::sat_t(t).map_err(VleWasmError::from)?;
    to_js(&SatPropsData::from(row))
}

/// Full saturation-table row at pressure `p` **kPa absolute**.
///
/// # Errors
/// Throws if `p` is outside the saturation-pressure range
/// (≈0.6117–22064 kPa).
#[wasm_bindgen(js_name = steamSatP)]
pub fn steam_sat_p(p: f64) -> Result<JsValue, JsError> {
    let row = steam::sat_p(p).map_err(VleWasmError::from)?;
    to_js(&SatPropsData::from(row))
}

/// Saturation pressure of water at `t` **K**, in **kPa absolute**
/// (IF97 region-4 closed form).
///
/// # Errors
/// Throws if `t` is outside the saturation range.
#[wasm_bindgen(js_name = steamPsat)]
pub fn steam_psat(t: f64) -> Result<f64, JsError> {
    Ok(steam::psat(t).map_err(VleWasmError::from)?)
}

/// Saturation temperature of water at `p` **kPa absolute**, in **K**
/// (IF97 region-4 closed-form inverse).
///
/// # Errors
/// Throws if `p` is outside the saturation-pressure range.
#[wasm_bindgen(js_name = steamTsat)]
pub fn steam_tsat(p: f64) -> Result<f64, JsError> {
    Ok(steam::tsat(p).map_err(VleWasmError::from)?)
}

/// Latent heat of vaporization h_fg at temperature `t` **K**, in **kJ/kg**.
///
/// # Errors
/// Throws if `t` is outside the saturation range.
#[wasm_bindgen(js_name = steamLatentHeat)]
pub fn steam_latent_heat(t: f64) -> Result<f64, JsError> {
    Ok(steam::latent_heat(t).map_err(VleWasmError::from)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // IAPWS-IF97 Table 5 verification point: region 1, T = 300 K,
    // P = 3 MPa = 3000 kPa → v = 0.100215168e-2 m³/kg, h = 115.331273 kJ/kg.
    #[test]
    fn if97_region1_verification_point() {
        let s = SteamStateData::from(steam::SteamState::tp(300.0, 3000.0).unwrap());
        assert_eq!(s.region, "one");
        assert_eq!(s.phase, "liquid");
        assert!((s.v - 0.100215168e-2).abs() < 1e-10, "v = {}", s.v);
        assert!((s.h - 115.331273).abs() < 1e-5, "h = {}", s.h);
        assert!(s.quality.is_none());
    }

    // The kitchen benchmark: water boils at ~373.12 K at 1 atm, with a
    // latent heat of ~2256.5 kJ/kg.
    #[test]
    fn boiling_at_one_atmosphere() {
        let row = SatPropsData::from(steam::sat_p(101.325).unwrap());
        assert!((row.t - 373.12).abs() < 0.05, "t_sat = {}", row.t);
        assert!((row.h_fg - 2256.5).abs() < 1.0, "h_fg = {}", row.h_fg);
    }

    #[test]
    fn two_phase_state_carries_quality() {
        let s = SteamStateData::from(steam::SteamState::px(101.325, 0.5).unwrap());
        assert_eq!(s.phase, "twoPhase");
        assert_eq!(s.quality, Some(0.5));
        assert!(s.cp.is_nan(), "cp undefined inside the dome");
    }
}
