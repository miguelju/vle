//! Shared property bundle and the dimensionless-derivative → property maps.
//!
//! Every region produces the same set of mass-basis properties; only the
//! *route* differs — regions 1, 2, 5 go through the Gibbs derivatives
//! `γ_π, γ_ππ, γ_τ, γ_ττ, γ_πτ` ([`gibbs_props`]); region 3 goes through the
//! Helmholtz derivatives `φ_δ, φ_δδ, φ_τ, φ_ττ, φ_δτ` ([`helmholtz_props`]).
//! Centralising the two maps here keeps the region files to just their series
//! sums, and keeps the unit bookkeeping in one auditable place.
//!
//! ## Units bookkeeping (the one tricky spot)
//!
//! With `R = 0.461526 kJ/(kg·K)`, `T` in **K** and `p` in **kPa**, the
//! specific volume comes out in **m³/kg directly** — because `1 kPa = 1 kJ/m³`,
//! so `v = R·T·(πγ_π)/p` has units `(kJ/kg)/(kJ/m³) = m³/kg`. Enthalpy,
//! internal energy and entropy land in **kJ/kg** and **kJ/(kg·K)** for free.
//! Only the speed of sound needs the `kJ→J` factor of 1000, since `w²` is an
//! energy-per-mass (`m²/s² = J/kg`).

use crate::R;

/// A full mass-basis property set at one single-phase state point.
///
/// Units: `v` in **m³/kg**, `rho` in **kg/m³**, `u`/`h` in **kJ/kg**,
/// `s`/`cp`/`cv` in **kJ/(kg·K)**, `w` (speed of sound) in **m/s**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Props {
    /// Specific volume, **m³/kg**.
    pub v: f64,
    /// Density, **kg/m³** (`1/v`).
    pub rho: f64,
    /// Specific internal energy, **kJ/kg**.
    pub u: f64,
    /// Specific enthalpy, **kJ/kg**.
    pub h: f64,
    /// Specific entropy, **kJ/(kg·K)**.
    pub s: f64,
    /// Isobaric specific heat capacity, **kJ/(kg·K)**.
    pub cp: f64,
    /// Isochoric specific heat capacity, **kJ/(kg·K)**.
    pub cv: f64,
    /// Speed of sound, **m/s**.
    pub w: f64,
}

/// The six dimensionless Gibbs derivatives at a `(π, τ)` point.
///
/// `g` is `γ` itself; `gp = γ_π`, `gpp = γ_ππ`, `gt = γ_τ`, `gtt = γ_ττ`,
/// `gpt = γ_πτ`. For regions 2 and 5 these are the sums of the ideal and
/// residual parts.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Gibbs {
    pub g: f64,
    pub gp: f64,
    pub gpp: f64,
    pub gt: f64,
    pub gtt: f64,
    pub gpt: f64,
}

/// The five dimensionless Helmholtz derivatives at a `(δ, τ)` point (region 3).
///
/// `f` is `φ`; `fd = φ_δ`, `fdd = φ_δδ`, `ft = φ_τ`, `ftt = φ_ττ`,
/// `fdt = φ_δτ`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Helmholtz {
    pub f: f64,
    pub fd: f64,
    pub fdd: f64,
    pub ft: f64,
    pub ftt: f64,
    pub fdt: f64,
}

/// Map Gibbs derivatives to the full property set (regions 1, 2, 5).
///
/// # Arguments
/// * `d` — the six Gibbs derivatives at `(π, τ)`.
/// * `pi`, `tau` — the region's reduced pressure and temperature.
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// A [`Props`] with mass-basis units as documented on that struct.
///
/// Ref: R7-97(2012) Table 3 (region 1) / Table 12 (region 2) property maps.
pub(crate) fn gibbs_props(d: Gibbs, pi: f64, tau: f64, t: f64, p_kpa: f64) -> Props {
    // v = R·T·(π γ_π)/p  →  m³/kg (because 1 kPa = 1 kJ/m³).
    let v = R * t * pi * d.gp / p_kpa;
    let h = R * t * tau * d.gt;
    let u = R * t * (tau * d.gt - pi * d.gp);
    let s = R * (tau * d.gt - d.g);
    let cp = R * (-tau * tau * d.gtt);
    // cv/R = −τ²γ_ττ + (γ_π − τγ_πτ)²/γ_ππ.
    let num = d.gp - tau * d.gpt;
    let cv = R * (-tau * tau * d.gtt + num * num / d.gpp);
    // w²/(RT) = γ_π² / [ (γ_π − τγ_πτ)²/(τ²γ_ττ) − γ_ππ ].  ×1000 for kJ→J.
    let w2 = 1000.0 * R * t * d.gp * d.gp / (num * num / (tau * tau * d.gtt) - d.gpp);
    Props {
        v,
        rho: 1.0 / v,
        u,
        h,
        s,
        cp,
        cv,
        w: w2.sqrt(),
    }
}

/// Map Helmholtz derivatives to the full property set (region 3).
///
/// # Arguments
/// * `d` — the five Helmholtz derivatives at `(δ, τ)`.
/// * `delta`, `tau` — reduced density and temperature.
/// * `rho` — Density in **kg/m³** (region 3 is parameterised by ρ, not p).
/// * `t` — Temperature in **K**.
///
/// # Returns
/// A [`Props`]; the pressure implied by `(ρ, T)` is available separately via
/// [`crate::region3::pressure`].
///
/// Ref: R7-97(2012) Table 31 property map.
pub(crate) fn helmholtz_props(d: Helmholtz, delta: f64, tau: f64, rho: f64, t: f64) -> Props {
    let v = 1.0 / rho;
    // u/(RT) = τ φ_τ ;  h/(RT) = τ φ_τ + δ φ_δ ;  s/R = τ φ_τ − φ.
    let u = R * t * (tau * d.ft);
    let h = R * t * (tau * d.ft + delta * d.fd);
    let s = R * (tau * d.ft - d.f);
    // cv/R = −τ² φ_ττ.
    let cv = R * (-tau * tau * d.ftt);
    // cp/R = −τ²φ_ττ + (δφ_δ − δτφ_δτ)² / (2δφ_δ + δ²φ_δδ).
    let g1 = delta * d.fd - delta * tau * d.fdt;
    let g2 = 2.0 * delta * d.fd + delta * delta * d.fdd;
    let cp = R * (-tau * tau * d.ftt + g1 * g1 / g2);
    // w²/(RT) = 2δφ_δ + δ²φ_δδ − (δφ_δ − δτφ_δτ)²/(τ²φ_ττ).  ×1000 kJ→J.
    let w2 = 1000.0 * R * t * (g2 - g1 * g1 / (tau * tau * d.ftt));
    Props {
        v,
        rho,
        u,
        h,
        s,
        cp,
        cv,
        w: w2.sqrt(),
    }
}
