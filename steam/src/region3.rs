//! IF97 Region 3 — near-critical, 623.15–863.15 K above the B23 line.
//!
//! Region 3 is the one region IF97 gives as a **Helmholtz** free energy
//! `f(ρ,T)` rather than a Gibbs `g(p,T)`: `φ(δ,τ) = n₁ ln δ + Σ nᵢ δ^Iᵢ τ^Jᵢ`
//! (R7-97(2012) Eq. 28), with `δ = ρ/ρc`, `τ = Tc/T`. Because it is explicit in
//! density, a `(T, p)` query must **iterate on ρ** until the implied pressure
//! matches — done here with **Brent's method** (per the repo algorithm rules;
//! never finite differences for the property derivatives themselves, which stay
//! analytic).
//!
//! Verification: R7-97 Table 33, which gives properties at `(ρ, T)` inputs
//! (e.g. `ρ=500 kg/m³, T=650 K` → `p=0.255837018×10² MPa`,
//! `h=0.186343019×10⁴ kJ/kg`, `s=0.405427273×10¹ kJ/(kg·K)`).
//!
//! Internal units: `ρ` in **kg/m³**, `T` in **K**, `p` in **kPa**.

use crate::coefficients::{REGION3, REGION3_N1};
use crate::props::{Helmholtz, Props, helmholtz_props};
use crate::solve::{bracket, brent};
use crate::{R, RHO_C, SteamError, T_C};

/// The five dimensionless Helmholtz derivatives at reduced `(δ, τ)`.
fn phi(delta: f64, tau: f64) -> Helmholtz {
    // Leading n₁·ln δ term (and its δ-derivatives).
    let mut d = Helmholtz {
        f: REGION3_N1 * delta.ln(),
        fd: REGION3_N1 / delta,
        fdd: -REGION3_N1 / (delta * delta),
        ft: 0.0,
        ftt: 0.0,
        fdt: 0.0,
    };
    for term in &REGION3 {
        let (i, j, n) = (term.i, term.j, term.n);
        let (fi, fj) = (i as f64, j as f64);
        d.f += n * delta.powi(i) * tau.powi(j);
        d.fd += n * fi * delta.powi(i - 1) * tau.powi(j);
        d.fdd += n * fi * (fi - 1.0) * delta.powi(i - 2) * tau.powi(j);
        d.ft += n * delta.powi(i) * fj * tau.powi(j - 1);
        d.ftt += n * delta.powi(i) * fj * (fj - 1.0) * tau.powi(j - 2);
        d.fdt += n * fi * delta.powi(i - 1) * fj * tau.powi(j - 1);
    }
    d
}

/// Pressure implied by a `(ρ, T)` state in region 3.
///
/// # Arguments
/// * `rho` — Density in **kg/m³**.
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Pressure in **kPa absolute**. (`p = ρ R T δ φ_δ`; the units work because
/// `kg/m³ · kJ/(kg·K) · K = kJ/m³ = kPa`.)
pub(crate) fn pressure(rho: f64, t: f64) -> f64 {
    let delta = rho / RHO_C;
    let tau = T_C / t;
    let d = phi(delta, tau);
    rho * R * t * delta * d.fd
}

/// Full property set for a region-3 state given by `(ρ, T)`.
///
/// # Arguments
/// * `rho` — Density in **kg/m³**.
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Mass-basis [`Props`] (see that struct for units).
pub(crate) fn props_rho_t(rho: f64, t: f64) -> Props {
    let delta = rho / RHO_C;
    let tau = T_C / t;
    helmholtz_props(phi(delta, tau), delta, tau, rho, t)
}

/// Full property set for a region-3 state given by `(T, p)` — solves for ρ.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// Mass-basis [`Props`], or [`SteamError::NoConvergence`] if the density solve
/// fails to bracket/converge.
pub(crate) fn props_tp(t: f64, p_kpa: f64) -> Result<Props, SteamError> {
    let rho = density_tp(t, p_kpa)?;
    Ok(props_rho_t(rho, t))
}

/// Solve `pressure(ρ, T) = p` for density ρ in region 3.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Target pressure in **kPa absolute**.
///
/// # Returns
/// Density in **kg/m³**.
///
/// Strategy: the region-3 isotherm `p(ρ)` is monotonic for `T ≥ Tc`; below the
/// critical temperature it has a van-der-Waals loop, so we pick the liquid
/// (high-ρ) or vapor (low-ρ) branch by comparing `p` to `Psat(T)` and scan for
/// a sign change before handing a valid bracket to Brent.
pub(crate) fn density_tp(t: f64, p_kpa: f64) -> Result<f64, SteamError> {
    let f = |rho: f64| pressure(rho, t) - p_kpa;

    // Region-3 density envelope: ~1 … ~1000 kg/m³ (ρc = 322 sits in the middle).
    let (lo, hi) = if t >= T_C {
        (1.0, 1000.0)
    } else {
        // Below Tc: choose branch by saturation pressure.
        let psat_kpa = crate::mpa_to_kpa(crate::region4::psat(t));
        if p_kpa >= psat_kpa {
            (RHO_C, 1000.0) // compressed liquid → dense branch
        } else {
            (1.0, RHO_C) // superheated vapor → light branch
        }
    };

    let (a, b) =
        bracket(&f, lo, hi, 64).ok_or(SteamError::NoConvergence("region-3 density bracket"))?;
    brent(&f, a, b, "region-3 density")
}

/// Saturated liquid and vapor densities in the near-critical band
/// (623.15 K < T ≤ 647.096 K), where the two-phase boundary lives inside
/// region 3 rather than between regions 1 and 2.
///
/// # Arguments
/// * `t` — Temperature in **K** (must be `623.15 < t ≤ 647.096`).
///
/// # Returns
/// `(ρ_f, ρ_g)` — saturated **liquid** and **vapor** densities in **kg/m³**.
///
/// Both are roots of `pressure(ρ, T) = Psat(T)`; the van-der-Waals loop gives
/// three crossings, and we take the outer (stable) two by scanning the dense
/// `[ρc, 1000]` and light `[1, ρc]` branches separately.
pub(crate) fn saturated_densities(t: f64) -> Result<(f64, f64), SteamError> {
    let p = crate::mpa_to_kpa(crate::region4::psat(t));
    let f = |rho: f64| pressure(rho, t) - p;
    let (a1, b1) = bracket(&f, RHO_C, 1000.0, 128)
        .ok_or(SteamError::NoConvergence("region-3 sat liquid bracket"))?;
    let rho_f = brent(&f, a1, b1, "region-3 sat liquid")?;
    let (a2, b2) = bracket(&f, 1.0, RHO_C, 128)
        .ok_or(SteamError::NoConvergence("region-3 sat vapor bracket"))?;
    let rho_g = brent(&f, a2, b2, "region-3 sat vapor")?;
    Ok((rho_f, rho_g))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// R7-97(2012) Table 33 — three verification points, given as `(ρ, T)`.
    #[test]
    fn table_33_verification() {
        // Columns: rho[kg/m³], T[K], p[kPa], h, u, s, cp, w.
        let cases = [
            (
                500.0,
                650.0,
                0.255837018e5,
                0.186343019e4,
                0.181226279e4,
                0.405427273e1,
                0.138935717e2,
                0.502005554e3,
            ),
            (
                200.0,
                650.0,
                0.222930643e5,
                0.237512401e4,
                0.226365868e4,
                0.485438792e1,
                0.446579342e2,
                0.383444594e3,
            ),
            (
                500.0,
                750.0,
                0.783095639e5,
                0.225868845e4,
                0.210206932e4,
                0.446971906e1,
                0.634165359e1,
                0.760696041e3,
            ),
        ];
        for (rho, t, p, h, u, s, cp, w) in cases {
            assert_relative_eq!(pressure(rho, t), p, max_relative = 1e-8);
            let r = props_rho_t(rho, t);
            assert_relative_eq!(r.h, h, max_relative = 1e-8);
            assert_relative_eq!(r.u, u, max_relative = 1e-8);
            assert_relative_eq!(r.s, s, max_relative = 1e-8);
            assert_relative_eq!(r.cp, cp, max_relative = 1e-8);
            assert_relative_eq!(r.w, w, max_relative = 1e-8);
        }
    }

    /// The `(T,p) → ρ` inverse solve must recover the Table-33 density.
    #[test]
    fn density_solve_round_trip() {
        // T=750 K ≥ ... below Tc but far from the dome; ρ=500 → p=78.31 MPa.
        let p = pressure(500.0, 750.0);
        let rho = density_tp(750.0, p).unwrap();
        assert_relative_eq!(rho, 500.0, max_relative = 1e-7);

        // A supercritical point: T=700 K > Tc, ρ=300.
        let p2 = pressure(300.0, 700.0);
        let rho2 = density_tp(700.0, p2).unwrap();
        assert_relative_eq!(rho2, 300.0, max_relative = 1e-7);
    }
}
