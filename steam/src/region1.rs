//! IF97 Region 1 — compressed / subcooled liquid, 273.15–623.15 K.
//!
//! The fundamental equation is the dimensionless Gibbs free energy
//! `γ(π,τ) = Σ nᵢ (7.1−π)^Iᵢ (τ−1.222)^Jᵢ` (R7-97(2012) Eq. 7), explicit in
//! `(T, p)` — no iteration. All properties follow from its analytic
//! derivatives via [`crate::props::gibbs_props`].
//!
//! Reduced variables: `π = p/16.53 MPa`, `τ = 1386 K/T`.
//! Verification: R7-97 Table 5 (e.g. `T=300 K, p=3 MPa` →
//! `v=0.100215168×10⁻² m³/kg`, `h=0.115331273×10³ kJ/kg`,
//! `s=0.392294792 kJ/(kg·K)`).

use crate::coefficients::REGION1;
use crate::props::{Gibbs, Props, gibbs_props};

/// Reducing pressure `p*` for region 1, in **MPa**.
const PSTAR_MPA: f64 = 16.53;
/// Reducing temperature `T*` for region 1, in **K**.
const TSTAR: f64 = 1386.0;

/// The six dimensionless Gibbs derivatives at reduced `(π, τ)`.
fn gamma(pi: f64, tau: f64) -> Gibbs {
    let a = 7.1 - pi; // (7.1 − π)
    let b = tau - 1.222; // (τ − 1.222)
    let mut d = Gibbs {
        g: 0.0,
        gp: 0.0,
        gpp: 0.0,
        gt: 0.0,
        gtt: 0.0,
        gpt: 0.0,
    };
    for term in &REGION1 {
        let (i, j, n) = (term.i, term.j, term.n);
        let (fi, fj) = (i as f64, j as f64);
        d.g += n * a.powi(i) * b.powi(j);
        d.gp += -n * fi * a.powi(i - 1) * b.powi(j);
        d.gpp += n * fi * (fi - 1.0) * a.powi(i - 2) * b.powi(j);
        d.gt += n * a.powi(i) * fj * b.powi(j - 1);
        d.gtt += n * a.powi(i) * fj * (fj - 1.0) * b.powi(j - 2);
        d.gpt += -n * fi * a.powi(i - 1) * fj * b.powi(j - 1);
    }
    d
}

/// Full property set for a region-1 state point.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// Mass-basis [`Props`] (see that struct for units).
pub(crate) fn props(t: f64, p_kpa: f64) -> Props {
    let pi = (p_kpa * 1e-3) / PSTAR_MPA;
    let tau = TSTAR / t;
    gibbs_props(gamma(pi, tau), pi, tau, t, p_kpa)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// R7-97(2012) Table 5 — three verification points, full precision.
    #[test]
    fn table_5_verification() {
        // Columns: T[K], p[kPa], v, h, u, s, cp, w.
        let cases = [
            (
                300.0,
                3_000.0,
                0.100215168e-2,
                0.115331273e3,
                0.112324818e3,
                0.392294792,
                0.417301218e1,
                0.150773921e4,
            ),
            (
                300.0,
                80_000.0,
                0.971180894e-3,
                0.184142828e3,
                0.106448356e3,
                0.368563852,
                0.401008987e1,
                0.163469054e4,
            ),
            (
                500.0,
                3_000.0,
                0.120241800e-2,
                0.975542239e3,
                0.971934985e3,
                0.258041912e1,
                0.465580682e1,
                0.124071337e4,
            ),
        ];
        for (t, p, v, h, u, s, cp, w) in cases {
            let r = props(t, p);
            assert_relative_eq!(r.v, v, max_relative = 1e-8);
            assert_relative_eq!(r.h, h, max_relative = 1e-8);
            assert_relative_eq!(r.u, u, max_relative = 1e-8);
            assert_relative_eq!(r.s, s, max_relative = 1e-8);
            assert_relative_eq!(r.cp, cp, max_relative = 1e-8);
            assert_relative_eq!(r.w, w, max_relative = 1e-8);
        }
    }

    /// `h = u + p·v` must hold identically. `p` in kPa, `v` in m³/kg → kJ/kg.
    #[test]
    fn h_u_pv_identity() {
        let (t, p) = (400.0, 20_000.0);
        let r = props(t, p);
        assert_relative_eq!(r.h, r.u + p * r.v, max_relative = 1e-12);
    }
}
