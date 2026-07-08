//! The practitioner-facing state API: [`SteamState`] and its constructors.
//!
//! Everything above this module computes single-region property surfaces; this
//! module assembles them into the state objects an engineer actually reaches
//! for — `(T,P)`, `(T,x)`, `(P,x)`, `(P,h)`, `(P,s)` — with region detection,
//! two-phase quality logic, and the classic saturation-table row ([`SatProps`]).
//!
//! ## The `(T,P)` trap
//!
//! Inside the two-phase dome `T` and `P` are **not** independent — specifying
//! both does not fix the quality. [`SteamState::tp`] therefore reports a point
//! that lands on the saturation line as [`Phase::TwoPhase`] with `x = None`
//! (and exposes the saturated-liquid property values); use [`SteamState::tx`],
//! [`SteamState::px`], or [`SteamState::ph`] to pin the quality.
//!
//! ## Units
//!
//! Inputs: `T` in **K**, `P` in **kPa absolute**, `h` in **kJ/kg**, `s` in
//! **kJ/(kg·K)**. Outputs are the mass-basis [`crate::Props`] set. Two-phase
//! states report mixture `v, u, h, s` (well-defined) and leave `cp, cv, w` as
//! `NaN` (undefined across a phase boundary).

use crate::props::Props;
use crate::regions::{Region, region_of};
use crate::{
    M_WATER, P_C_MPA, SteamError, T_C, backward, kpa_to_mpa, mpa_to_kpa, region1, region2, region3,
    region4, region5, regions, solve,
};

/// The qualitative phase of a [`SteamState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Single-phase liquid (compressed or saturated liquid).
    Liquid,
    /// Single-phase vapor (superheated or saturated vapor).
    Vapor,
    /// Two-phase liquid+vapor mixture (inside the dome).
    TwoPhase,
    /// Supercritical fluid (`T ≥ Tc` and `P ≥ Pc`).
    Supercritical,
}

/// A fully-resolved water/steam state at one point.
///
/// Units: `t` **K**, `p` **kPa**, `v` **m³/kg**, `rho` **kg/m³**, `u`/`h`
/// **kJ/kg**, `s`/`cp`/`cv` **kJ/(kg·K)**, `w` **m/s**. For two-phase states
/// `cp = cv = w = NaN` and `x = Some(quality)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SteamState {
    /// Temperature, **K**.
    pub t: f64,
    /// Pressure, **kPa absolute**.
    pub p: f64,
    /// IF97 region the point resolved to.
    pub region: Region,
    /// Qualitative phase.
    pub phase: Phase,
    /// Vapor quality (mass fraction vapor) if two-phase, else `None`.
    pub x: Option<f64>,
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
    /// Isobaric heat capacity, **kJ/(kg·K)** (`NaN` if two-phase).
    pub cp: f64,
    /// Isochoric heat capacity, **kJ/(kg·K)** (`NaN` if two-phase).
    pub cv: f64,
    /// Speed of sound, **m/s** (`NaN` if two-phase).
    pub w: f64,
}

/// A molar-basis view of a [`SteamState`], for interop with the molar-canon
/// `vle-thermo` engine. Converts via `M_water = 18.015268 kg/kmol`.
///
/// Units: `v` **cm³/mol**, `u`/`h` **kJ/kmol**, `s`/`cp` **kJ/(kmol·K)**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MolarState {
    /// Molar volume, **cm³/mol** (`= v[m³/kg]·M·1000`).
    pub v: f64,
    /// Molar internal energy, **kJ/kmol**.
    pub u: f64,
    /// Molar enthalpy, **kJ/kmol**.
    pub h: f64,
    /// Molar entropy, **kJ/(kmol·K)**.
    pub s: f64,
    /// Molar isobaric heat capacity, **kJ/(kmol·K)**.
    pub cp: f64,
}

/// A saturation-table row: the classic printed-steam-table entry.
///
/// Units: `t` **K**, `p` **kPa**, `v_*` **m³/kg**, `h_*`/`u_*` **kJ/kg**,
/// `s_*` **kJ/(kg·K)**. Subscript `f` = saturated liquid, `g` = saturated
/// vapor, `fg` = the vapor–liquid difference (`h_fg` is the latent heat).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SatProps {
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
    /// Latent heat of vaporization `h_g − h_f`, **kJ/kg**.
    pub h_fg: f64,
    /// Saturated-liquid entropy, **kJ/(kg·K)**.
    pub s_f: f64,
    /// Saturated-vapor entropy, **kJ/(kg·K)**.
    pub s_g: f64,
    /// Entropy of vaporization `s_g − s_f`, **kJ/(kg·K)**.
    pub s_fg: f64,
    /// Saturated-liquid internal energy, **kJ/kg**.
    pub u_f: f64,
    /// Saturated-vapor internal energy, **kJ/kg**.
    pub u_g: f64,
}

/// Evaluate the single-phase property surface for a known region.
fn props_for_region(t: f64, p_kpa: f64, region: Region) -> Result<Props, SteamError> {
    match region {
        Region::One => Ok(region1::props(t, p_kpa)),
        Region::Two => Ok(region2::props(t, p_kpa)),
        Region::Five => Ok(region5::props(t, p_kpa)),
        Region::Three => region3::props_tp(t, p_kpa),
        Region::Saturated => Err(SteamError::OutOfRange { t, p: p_kpa }),
    }
}

/// Classify the phase of a single-phase point.
fn classify(t: f64, p_kpa: f64, region: Region) -> Phase {
    if t >= T_C && p_kpa >= mpa_to_kpa(P_C_MPA) {
        return Phase::Supercritical;
    }
    match region {
        Region::One => Phase::Liquid,
        Region::Two | Region::Five => Phase::Vapor,
        Region::Three => {
            if t < T_C {
                let ps = mpa_to_kpa(region4::psat(t));
                if p_kpa >= ps {
                    Phase::Liquid
                } else {
                    Phase::Vapor
                }
            } else {
                Phase::Supercritical
            }
        }
        Region::Saturated => Phase::TwoPhase,
    }
}

impl SteamState {
    /// Assemble a single-phase state from a computed property set.
    fn from_single(t: f64, p_kpa: f64, pr: Props, region: Region) -> Self {
        SteamState {
            t,
            p: p_kpa,
            region,
            phase: classify(t, p_kpa, region),
            x: None,
            v: pr.v,
            rho: pr.rho,
            u: pr.u,
            h: pr.h,
            s: pr.s,
            cp: pr.cp,
            cv: pr.cv,
            w: pr.w,
        }
    }

    /// Assemble a two-phase state by mixing the saturation row at quality `x`.
    fn from_quality(sat: &SatProps, x: f64) -> Self {
        let v = sat.v_f + x * (sat.v_g - sat.v_f);
        let phase = if x <= 0.0 {
            Phase::Liquid
        } else if x >= 1.0 {
            Phase::Vapor
        } else {
            Phase::TwoPhase
        };
        SteamState {
            t: sat.t,
            p: sat.p,
            region: Region::Saturated,
            phase,
            x: Some(x),
            v,
            rho: 1.0 / v,
            u: sat.u_f + x * (sat.u_g - sat.u_f),
            h: sat.h_f + x * sat.h_fg,
            s: sat.s_f + x * sat.s_fg,
            cp: f64::NAN,
            cv: f64::NAN,
            w: f64::NAN,
        }
    }

    /// State from temperature and pressure.
    ///
    /// # Arguments
    /// * `t` — Temperature in **K**.
    /// * `p_kpa` — Pressure in **kPa absolute**.
    ///
    /// A point on the saturation line reports [`Phase::TwoPhase`] with
    /// `x = None` (quality is undetermined by `(T,P)` alone).
    pub fn tp(t: f64, p_kpa: f64) -> Result<Self, SteamError> {
        match region_of(t, kpa_to_mpa(p_kpa)) {
            Some(Region::Saturated) => {
                let sat = sat_t(t)?;
                let mut st = Self::from_quality(&sat, 0.0);
                st.phase = Phase::TwoPhase;
                st.x = None;
                Ok(st)
            }
            Some(r) => Ok(Self::from_single(
                t,
                p_kpa,
                props_for_region(t, p_kpa, r)?,
                r,
            )),
            None => Err(SteamError::OutOfRange { t, p: p_kpa }),
        }
    }

    /// State from temperature and vapor quality (mass fraction vapor).
    ///
    /// # Arguments
    /// * `t` — Temperature in **K**, on `273.15 ≤ t ≤ 647.096`.
    /// * `x` — Quality, `0 ≤ x ≤ 1`.
    pub fn tx(t: f64, x: f64) -> Result<Self, SteamError> {
        if !(0.0..=1.0).contains(&x) {
            return Err(SteamError::InvalidQuality(x));
        }
        Ok(Self::from_quality(&sat_t(t)?, x))
    }

    /// State from pressure and vapor quality.
    ///
    /// # Arguments
    /// * `p_kpa` — Pressure in **kPa absolute**, between triple and critical.
    /// * `x` — Quality, `0 ≤ x ≤ 1`.
    pub fn px(p_kpa: f64, x: f64) -> Result<Self, SteamError> {
        if !(0.0..=1.0).contains(&x) {
            return Err(SteamError::InvalidQuality(x));
        }
        Ok(Self::from_quality(&sat_p(p_kpa)?, x))
    }

    /// State from pressure and specific enthalpy (isenthalpic / PH flash).
    ///
    /// # Arguments
    /// * `p_kpa` — Pressure in **kPa absolute**.
    /// * `h` — Specific enthalpy in **kJ/kg**.
    ///
    /// If `h` falls between the saturated-liquid and -vapor enthalpies at
    /// `Tsat(P)`, the result is two-phase with `x = (h − h_f)/h_fg`; otherwise
    /// it is single-phase (temperature found via the backward equation or a
    /// bracketed forward solve).
    pub fn ph(p_kpa: f64, h: f64) -> Result<Self, SteamError> {
        let p_mpa = kpa_to_mpa(p_kpa);
        if (region4::P_MIN_MPA..=P_C_MPA).contains(&p_mpa) {
            let sat = sat_p(p_kpa)?;
            if (sat.h_f..=sat.h_g).contains(&h) {
                let x = (h - sat.h_f) / sat.h_fg;
                return Ok(Self::from_quality(&sat, x));
            }
        }
        let (t, region) = t_from_ph(p_kpa, h)?;
        Ok(Self::from_single(
            t,
            p_kpa,
            props_for_region(t, p_kpa, region)?,
            region,
        ))
    }

    /// State from pressure and specific entropy (isentropic / PS flash).
    ///
    /// # Arguments
    /// * `p_kpa` — Pressure in **kPa absolute**.
    /// * `s` — Specific entropy in **kJ/(kg·K)**.
    ///
    /// Two-phase if `s` lies between `s_f` and `s_g` at `Tsat(P)`; otherwise
    /// single-phase. The workhorse for isentropic turbine/pump calculations.
    pub fn ps(p_kpa: f64, s: f64) -> Result<Self, SteamError> {
        let p_mpa = kpa_to_mpa(p_kpa);
        if (region4::P_MIN_MPA..=P_C_MPA).contains(&p_mpa) {
            let sat = sat_p(p_kpa)?;
            if (sat.s_f..=sat.s_g).contains(&s) {
                let x = (s - sat.s_f) / sat.s_fg;
                return Ok(Self::from_quality(&sat, x));
            }
        }
        let (t, region) = t_from_ps(p_kpa, s)?;
        Ok(Self::from_single(
            t,
            p_kpa,
            props_for_region(t, p_kpa, region)?,
            region,
        ))
    }

    /// A molar-basis view of this state (via `M_water = 18.015268 kg/kmol`).
    pub fn molar(&self) -> MolarState {
        MolarState {
            v: self.v * M_WATER * 1000.0, // m³/kg → cm³/mol
            u: self.u * M_WATER,
            h: self.h * M_WATER,
            s: self.s * M_WATER,
            cp: self.cp * M_WATER,
        }
    }
}

/// Highest temperature at which the saturation split uses regions 1 & 2 rather
/// than the near-critical region-3 solve, in **K**.
const T_SAT_R12_MAX: f64 = regions::T_13;

/// Saturation-table row at a given temperature.
///
/// # Arguments
/// * `t` — Temperature in **K**, on `273.15 ≤ t ≤ 647.096`.
///
/// # Returns
/// A [`SatProps`] row. Below 623.15 K the saturated liquid/vapor come from
/// regions 1 and 2 at `Psat(T)`; in the near-critical band (623.15–647.096 K)
/// both come from region 3 at the saturated densities.
pub fn sat_t(t: f64) -> Result<SatProps, SteamError> {
    if !(region4::T_MIN..=region4::T_MAX).contains(&t) {
        return Err(SteamError::OutOfSaturationRange(t));
    }
    let p_kpa = mpa_to_kpa(region4::psat(t));
    let (liq, vap) = if t <= T_SAT_R12_MAX {
        (region1::props(t, p_kpa), region2::props(t, p_kpa))
    } else {
        let (rho_f, rho_g) = region3::saturated_densities(t)?;
        (
            region3::props_rho_t(rho_f, t),
            region3::props_rho_t(rho_g, t),
        )
    };
    Ok(SatProps {
        t,
        p: p_kpa,
        v_f: liq.v,
        v_g: vap.v,
        h_f: liq.h,
        h_g: vap.h,
        h_fg: vap.h - liq.h,
        s_f: liq.s,
        s_g: vap.s,
        s_fg: vap.s - liq.s,
        u_f: liq.u,
        u_g: vap.u,
    })
}

/// Saturation-table row at a given pressure.
///
/// # Arguments
/// * `p_kpa` — Pressure in **kPa absolute**, between triple and critical.
///
/// # Returns
/// A [`SatProps`] row at `Tsat(P)`.
pub fn sat_p(p_kpa: f64) -> Result<SatProps, SteamError> {
    let p_mpa = kpa_to_mpa(p_kpa);
    if !(region4::P_MIN_MPA..=region4::P_MAX_MPA).contains(&p_mpa) {
        return Err(SteamError::OutOfSaturationRange(p_kpa));
    }
    sat_t(region4::tsat(p_mpa))
}

/// Latent heat of vaporization at temperature `t`.
///
/// # Arguments
/// * `t` — Temperature in **K**.
///
/// # Returns
/// `h_fg = h_g − h_f` in **kJ/kg**.
pub fn latent_heat(t: f64) -> Result<f64, SteamError> {
    Ok(sat_t(t)?.h_fg)
}

/// Latent heat of vaporization at pressure `p`.
///
/// # Arguments
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// `h_fg = h_g − h_f` in **kJ/kg**.
pub fn latent_heat_at_p(p_kpa: f64) -> Result<f64, SteamError> {
    Ok(sat_p(p_kpa)?.h_fg)
}

/// Upper temperature bound for a single-phase forward solve at pressure `p`.
fn t_upper_for(p_kpa: f64) -> f64 {
    if kpa_to_mpa(p_kpa) > regions::P_MAX_R5 {
        regions::T_25 // region 5 (and its 50 MPa ceiling) unavailable
    } else {
        regions::T_MAX
    }
}

/// Backward `T(p,h)`: the region-1 closed form gives a fast **seed** (accurate
/// to ~0.02 K per IAPWS), which a Newton polish on the forward `h(T,p)` refines
/// to exactness. Outside region 1, a bracketed forward Brent solve.
fn t_from_ph(p_kpa: f64, h: f64) -> Result<(f64, Region), SteamError> {
    let seed = backward::t_ph_region1(p_kpa, h);
    if region_of(seed, kpa_to_mpa(p_kpa)) == Some(Region::One) {
        // dh/dT|_p = cp → Newton converges in 2–3 steps to forward precision.
        let mut t = seed;
        for _ in 0..12 {
            let pr = region1::props(t, p_kpa);
            let dt = (h - pr.h) / pr.cp;
            t += dt;
            if dt.abs() < 1e-11 {
                break;
            }
        }
        if region_of(t, kpa_to_mpa(p_kpa)) == Some(Region::One) {
            return Ok((t, Region::One));
        }
    }
    let (lo, hi) = single_phase_t_bracket(p_kpa, |pr| pr.h, h)?;
    let f = |t: f64| forward_prop(t, p_kpa, |pr| pr.h) - h;
    let t = solve::brent(&f, lo, hi, "T(p,h) forward")?;
    let region = region_of(t, kpa_to_mpa(p_kpa)).ok_or(SteamError::OutOfRange { t, p: p_kpa })?;
    Ok((t, region))
}

/// Backward `T(p,s)`: region-1 seed + Newton polish (`ds/dT|_p = cp/T`), else a
/// bracketed forward Brent solve on `s(T,p)`.
fn t_from_ps(p_kpa: f64, s: f64) -> Result<(f64, Region), SteamError> {
    let seed = backward::t_ps_region1(p_kpa, s);
    if region_of(seed, kpa_to_mpa(p_kpa)) == Some(Region::One) {
        let mut t = seed;
        for _ in 0..12 {
            let pr = region1::props(t, p_kpa);
            let dt = (s - pr.s) * t / pr.cp; // ds/dT|_p = cp/T
            t += dt;
            if dt.abs() < 1e-11 {
                break;
            }
        }
        if region_of(t, kpa_to_mpa(p_kpa)) == Some(Region::One) {
            return Ok((t, Region::One));
        }
    }
    let (lo, hi) = single_phase_t_bracket(p_kpa, |pr| pr.s, s)?;
    let f = |t: f64| forward_prop(t, p_kpa, |pr| pr.s) - s;
    let t = solve::brent(&f, lo, hi, "T(p,s) forward")?;
    let region = region_of(t, kpa_to_mpa(p_kpa)).ok_or(SteamError::OutOfRange { t, p: p_kpa })?;
    Ok((t, region))
}

/// Evaluate one forward property (enthalpy or entropy) at `(t, p)`, dispatching
/// by region; NaN if the point is out of range (keeps the solver total).
fn forward_prop(t: f64, p_kpa: f64, pick: impl Fn(&Props) -> f64) -> f64 {
    match region_of(t, kpa_to_mpa(p_kpa)) {
        Some(r) => props_for_region(t, p_kpa, r)
            .map(|pr| pick(&pr))
            .unwrap_or(f64::NAN),
        None => f64::NAN,
    }
}

/// Build a temperature bracket for a single-phase `(P, target)` solve, choosing
/// the liquid or vapor side of the saturation seam by comparing `target` to the
/// saturated value.
fn single_phase_t_bracket(
    p_kpa: f64,
    pick: impl Fn(&Props) -> f64 + Copy,
    target: f64,
) -> Result<(f64, f64), SteamError> {
    let hi = t_upper_for(p_kpa);
    let p_mpa = kpa_to_mpa(p_kpa);
    if (region4::P_MIN_MPA..P_C_MPA).contains(&p_mpa) {
        // Subcritical: the seam is at Tsat; pick the side the target lands on.
        let sat = sat_p(p_kpa)?;
        let tsat = sat.t;
        let sat_val = pick(&region1::props(tsat, p_kpa)); // liquid-side value at Tsat
        // Compare target to the saturated-liquid value: below → liquid branch.
        if target < sat_val {
            Ok((region4::T_MIN, tsat - 1e-4))
        } else {
            Ok((tsat + 1e-4, hi))
        }
    } else {
        Ok((region4::T_MIN, hi))
    }
}

impl SatProps {
    /// A molar-basis latent heat `h_fg` in **kJ/kmol**.
    pub fn h_fg_molar(&self) -> f64 {
        self.h_fg * M_WATER
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn tp_regions_and_phases() {
        // Compressed liquid.
        let s = SteamState::tp(300.0, 3_000.0).unwrap();
        assert_eq!(s.region, Region::One);
        assert_eq!(s.phase, Phase::Liquid);
        assert_relative_eq!(s.h, 0.115331273e3, max_relative = 1e-7);
        // Superheated vapor.
        let s = SteamState::tp(700.0, 3.5).unwrap();
        assert_eq!(s.phase, Phase::Vapor);
        // Supercritical.
        let s = SteamState::tp(700.0, 30_000.0).unwrap();
        assert_eq!(s.phase, Phase::Supercritical);
    }

    #[test]
    fn sat_row_atmospheric() {
        // 100 kPa ≈ 1 atm: Tsat ≈ 372.76 K, h_fg ≈ 2257 kJ/kg (handbook).
        let sat = sat_p(100.0).unwrap();
        assert_relative_eq!(sat.t, 372.7559, max_relative = 1e-5);
        assert_relative_eq!(sat.h_fg, 2257.5, max_relative = 5e-4);
        // Saturated-vapor enthalpy at 1 atm ≈ 2675 kJ/kg.
        assert_relative_eq!(sat.h_g, 2675.5, max_relative = 5e-4);
    }

    #[test]
    fn tx_mixes_saturation_row() {
        let sat = sat_t(400.0).unwrap();
        let st = SteamState::tx(400.0, 0.5).unwrap();
        assert_eq!(st.phase, Phase::TwoPhase);
        assert_relative_eq!(st.h, sat.h_f + 0.5 * sat.h_fg, max_relative = 1e-12);
        assert_relative_eq!(st.s, 0.5 * (sat.s_f + sat.s_g), max_relative = 1e-12);
    }

    #[test]
    fn ph_two_phase_and_single() {
        // Two-phase: h between h_f and h_g at 100 kPa.
        let sat = sat_p(100.0).unwrap();
        let hmid = 0.5 * (sat.h_f + sat.h_g);
        let st = SteamState::ph(100.0, hmid).unwrap();
        assert_eq!(st.phase, Phase::TwoPhase);
        assert_relative_eq!(st.x.unwrap(), 0.5, max_relative = 1e-9);

        // Single-phase liquid: recover T from region-1 enthalpy.
        let ref_liq = region1::props(320.0, 5_000.0);
        let st = SteamState::ph(5_000.0, ref_liq.h).unwrap();
        assert_relative_eq!(st.t, 320.0, max_relative = 1e-6);

        // Single-phase vapor: recover T from region-2 enthalpy.
        let ref_vap = region2::props(800.0, 1_000.0);
        let st = SteamState::ph(1_000.0, ref_vap.h).unwrap();
        assert_relative_eq!(st.t, 800.0, max_relative = 1e-6);
    }

    #[test]
    fn ps_round_trip_single_phase() {
        let ref_vap = region2::props(600.0, 500.0);
        let st = SteamState::ps(500.0, ref_vap.s).unwrap();
        assert_relative_eq!(st.t, 600.0, max_relative = 1e-6);
    }

    #[test]
    fn near_critical_saturation_row() {
        // 640 K is inside the 623.15–647.096 K region-3 saturation band.
        let sat = sat_t(640.0).unwrap();
        assert!(sat.h_fg > 0.0, "latent heat must be positive below Tc");
        assert!(sat.v_g > sat.v_f, "vapor less dense than liquid");
        // Pressure must match Psat(640 K).
        assert_relative_eq!(sat.p, mpa_to_kpa(region4::psat(640.0)), max_relative = 1e-9);
    }
}
