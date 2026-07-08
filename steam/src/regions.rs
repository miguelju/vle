//! Region detection and the B23 boundary line.
//!
//! IF97 tiles the (T, p) plane into five regions, each with its own equation
//! (Gibbs `g(p,T)` in regions 1, 2, 5; Helmholtz `f(ρ,T)` in region 3; the
//! implicit saturation equation in region 4). Before evaluating any property
//! you must know **which** region a state point falls in — that is what
//! [`region_of`] does.
//!
//! The **B23 boundary** is the dividing line between region 2 (superheated
//! vapor) and region 3 (near-critical). IF97 gives it as a quadratic in `T`
//! that also inverts closed-form for `T(p)`.
//!
//! Reference: IAPWS R7-97(2012) §4 (B23, Eqs. 5–6, Table 1), §4 region map
//! (Fig. 1). Verification: Table 25 — `T = 0.623150000×10³ K` ⇄
//! `p = 0.165291643×10² MPa`.
//!
//! Internal units: temperature in **K**, pressure in **MPa**.

use crate::region4;

/// The five IF97 regions plus the saturation line (region 4).
///
/// `Saturated` is returned by [`region_of`] when a `(T, p)` point lies on the
/// saturation curve within tolerance — there the phase split is undetermined
/// by `(T, p)` alone (a classic student trap: inside the dome `T` and `P` are
/// not independent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// Compressed / subcooled liquid — region 1.
    One,
    /// Superheated vapor up to 1073.15 K — region 2.
    Two,
    /// Near-critical, evaluated via Helmholtz `f(ρ,T)` — region 3.
    Three,
    /// On the saturation line (two-phase boundary) — region 4.
    Saturated,
    /// High-temperature steam 1073.15–2273.15 K — region 5.
    Five,
}

// ── IF97 domain boundaries, in K and MPa ─────────────────────────────────

/// Lowest valid temperature (triple point), **K**.
pub const T_MIN: f64 = 273.15;
/// Region 1/3 dividing temperature (also region 2 lower branch), **K**.
pub const T_13: f64 = 623.15;
/// Upper temperature of the B23 line / region 3, **K**.
pub const T_23_MAX: f64 = 863.15;
/// Region 2/5 dividing temperature, **K**.
pub const T_25: f64 = 1073.15;
/// Highest valid temperature, **K**.
pub const T_MAX: f64 = 2273.15;
/// Highest valid pressure (regions 1–3), **MPa**.
pub const P_MAX: f64 = 100.0;
/// Highest valid pressure in region 5, **MPa**.
pub const P_MAX_R5: f64 = 50.0;
/// Lowest valid pressure, **MPa** (a hair above vacuum; IF97 needs `p > 0`).
pub const P_MIN: f64 = 0.0;

/// The five B23-boundary coefficients `n₁ … n₅` from R7-97(2012) Table 1.
const N23: [f64; 5] = [
    0.34805185628969e3,
    -0.11671859879975e1,
    0.10192970039326e-2,
    0.57254459862746e3,
    0.13918839778870e2,
];

/// Pressure on the B23 boundary at temperature `t` (region 2 ↔ 3 divider).
///
/// # Arguments
/// * `t` — Temperature in **K**, valid on `623.15 ≤ t ≤ 863.15`.
///
/// # Returns
/// Boundary pressure in **MPa**.
///
/// Ref: R7-97(2012) Eq. (5).
pub fn b23_p(t: f64) -> f64 {
    N23[0] + N23[1] * t + N23[2] * t * t
}

/// Temperature on the B23 boundary at pressure `p` (the inverse of [`b23_p`]).
///
/// # Arguments
/// * `p` — Pressure in **MPa**, valid on `16.5292 ≤ p ≤ 100`.
///
/// # Returns
/// Boundary temperature in **K**.
///
/// Ref: R7-97(2012) Eq. (6).
pub fn b23_t(p: f64) -> f64 {
    N23[3] + ((p - N23[4]) / N23[2]).sqrt()
}

/// Relative tolerance (in pressure) for calling a point "on the saturation
/// line". `1e-6` is far tighter than IF97's own inter-region consistency but
/// loose enough to catch a `psat(T)` fed straight back as `p`.
const SAT_REL_TOL: f64 = 1e-6;

/// Classify a `(T, p)` state point into an IF97 region.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p` — Pressure in **MPa absolute**.
///
/// # Returns
/// `Some(region)` if the point is inside the IF97 validity envelope, or
/// `None` if it is out of range (too cold/hot, non-positive pressure, or above
/// the region's pressure ceiling).
pub fn region_of(t: f64, p: f64) -> Option<Region> {
    if !(T_MIN..=T_MAX).contains(&t) || p <= P_MIN {
        return None;
    }

    if t <= T_13 {
        // Below 623.15 K the split is by the saturation line.
        if p > P_MAX {
            return None;
        }
        let ps = region4::psat(t);
        if (p - ps).abs() <= SAT_REL_TOL * ps {
            Some(Region::Saturated)
        } else if p > ps {
            Some(Region::One)
        } else {
            Some(Region::Two)
        }
    } else if t <= T_23_MAX {
        // Between 623.15 and 863.15 K the split is the B23 line.
        if p > P_MAX {
            return None;
        }
        if p > b23_p(t) {
            Some(Region::Three)
        } else {
            Some(Region::Two)
        }
    } else if t <= T_25 {
        // 863.15–1073.15 K: all region 2 up to 100 MPa.
        if p > P_MAX { None } else { Some(Region::Two) }
    } else {
        // 1073.15–2273.15 K: region 5, capped at 50 MPa.
        if p > P_MAX_R5 {
            None
        } else {
            Some(Region::Five)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn b23_verification_table_25() {
        // R7-97(2012) Table 25.
        assert_relative_eq!(b23_p(0.623150000e3), 0.165291643e2, max_relative = 1e-8);
        assert_relative_eq!(b23_t(0.165291643e2), 0.623150000e3, max_relative = 1e-8);
    }

    #[test]
    fn b23_round_trip() {
        for &t in &[623.15, 700.0, 800.0, 863.15] {
            assert_relative_eq!(b23_t(b23_p(t)), t, max_relative = 1e-10);
        }
    }

    #[test]
    fn region_map_sample_points() {
        // Region 1: compressed liquid (T=300 K, p=3 MPa >> psat).
        assert_eq!(region_of(300.0, 3.0), Some(Region::One));
        // Region 2: low-pressure vapor (T=300 K, p=0.0035 MPa < psat).
        assert_eq!(region_of(300.0, 0.0035), Some(Region::Two));
        // Region 2 high-T: T=700 K below B23 (p=30 MPa < b23_p(700)≈78 MPa).
        assert_eq!(region_of(700.0, 30.0), Some(Region::Two));
        // Region 3: T=650 K, p=25 MPa > b23_p(650)≈17 MPa.
        assert_eq!(region_of(650.0, 25.0), Some(Region::Three));
        // Region 5: high-T steam.
        assert_eq!(region_of(1500.0, 0.5), Some(Region::Five));
    }

    #[test]
    fn region_map_saturation_and_out_of_range() {
        // A point placed exactly on psat(400 K) reads as Saturated.
        let ps = region4::psat(400.0);
        assert_eq!(region_of(400.0, ps), Some(Region::Saturated));
        // Out of range: too cold, non-positive p, over-pressure, over-temp.
        assert_eq!(region_of(200.0, 1.0), None);
        assert_eq!(region_of(400.0, 0.0), None);
        assert_eq!(region_of(400.0, 150.0), None);
        assert_eq!(region_of(2500.0, 1.0), None);
        assert_eq!(region_of(1500.0, 60.0), None); // region 5 over 50 MPa
    }
}
