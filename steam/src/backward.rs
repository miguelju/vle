//! IF97 backward equations `T(p,h)` and `T(p,s)` for region 1.
//!
//! The forward Gibbs equations give `h(T,p)` and `s(T,p)`; a PH or PS flash
//! needs the *inverse* — `T` from `(p,h)` or `(p,s)`. IF97 supplies explicit
//! polynomial backward equations so this stays non-iterative in region 1.
//! (Region 2 also has official backward equations, split into three
//! sub-regions; this crate instead solves region-2 `T(p,h)`/`T(p,s)` by a
//! robust bracketed iteration on the forward equation — see
//! [`crate::state`] — which needs no extra coefficient tables and is still
//! only a handful of evaluations. The region-2 backward polynomials are a
//! deferred throughput optimisation.)
//!
//! Reference: R7-97(2012) §5.2.1 (Eq. 11, Table 6) and §5.2.2 (Eq. 13,
//! Table 8). Verification: Table 7 (`p=3 MPa, h=500 kJ/kg → T=391.798509 K`)
//! and Table 9 (`p=3 MPa, s=0.5 kJ/(kg·K) → T=307.842258 K`).
//!
//! Units: `p` in **kPa**, `h` in **kJ/kg**, `s` in **kJ/(kg·K)**, `T` in **K**.

use crate::coefficients::{REGION1_TPH, REGION1_TPS};

/// Backward `T(p, h)` for region 1.
///
/// # Arguments
/// * `p_kpa` — Pressure in **kPa absolute**.
/// * `h` — Specific enthalpy in **kJ/kg**.
///
/// # Returns
/// Temperature in **K**.
pub(crate) fn t_ph_region1(p_kpa: f64, h: f64) -> f64 {
    let pi = p_kpa * 1e-3; // p* = 1 MPa
    let eta = h / 2500.0; // h* = 2500 kJ/kg
    let mut theta = 0.0;
    for term in &REGION1_TPH {
        theta += term.n * pi.powi(term.i) * (eta + 1.0).powi(term.j);
    }
    theta // T* = 1 K
}

/// Backward `T(p, s)` for region 1.
///
/// # Arguments
/// * `p_kpa` — Pressure in **kPa absolute**.
/// * `s` — Specific entropy in **kJ/(kg·K)**.
///
/// # Returns
/// Temperature in **K**.
pub(crate) fn t_ps_region1(p_kpa: f64, s: f64) -> f64 {
    let pi = p_kpa * 1e-3; // p* = 1 MPa
    let sigma = s; // s* = 1 kJ/(kg·K)
    let mut theta = 0.0;
    for term in &REGION1_TPS {
        theta += term.n * pi.powi(term.i) * (sigma + 2.0).powi(term.j);
    }
    theta // T* = 1 K
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// R7-97(2012) Table 7 — backward `T(p,h)`, region 1.
    #[test]
    fn table_7_tph() {
        assert_relative_eq!(
            t_ph_region1(3_000.0, 500.0),
            0.391798509e3,
            max_relative = 1e-8
        );
        assert_relative_eq!(
            t_ph_region1(80_000.0, 500.0),
            0.378108626e3,
            max_relative = 1e-8
        );
        assert_relative_eq!(
            t_ph_region1(80_000.0, 1500.0),
            0.611041229e3,
            max_relative = 1e-8
        );
    }

    /// R7-97(2012) Table 9 — backward `T(p,s)`, region 1.
    #[test]
    fn table_9_tps() {
        assert_relative_eq!(
            t_ps_region1(3_000.0, 0.5),
            0.307842258e3,
            max_relative = 1e-8
        );
        assert_relative_eq!(
            t_ps_region1(80_000.0, 0.5),
            0.309979785e3,
            max_relative = 1e-8
        );
        assert_relative_eq!(
            t_ps_region1(80_000.0, 3.0),
            0.565899909e3,
            max_relative = 1e-8
        );
    }

    /// The backward equation must invert the forward one (region 1).
    #[test]
    fn backward_inverts_forward() {
        let (t, p) = (400.0, 20_000.0);
        let fwd = crate::region1::props(t, p);
        assert_relative_eq!(t_ph_region1(p, fwd.h), t, max_relative = 1e-4);
        assert_relative_eq!(t_ps_region1(p, fwd.s), t, max_relative = 1e-4);
    }
}
