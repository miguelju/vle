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

use crate::R;
use crate::coefficients::REGION1;
use crate::props::{Gibbs, Props, gibbs_props};
use crate::series::powers;

/// Reducing pressure `p*` for region 1, in **MPa**.
const PSTAR_MPA: f64 = 16.53;
/// Reducing temperature `T*` for region 1, in **K**.
const TSTAR: f64 = 1386.0;

// Exponent windows for the power tables (see [`crate::series`]). Table 2 has
// `I ∈ [0, 32]` and `J ∈ [−41, 17]`; the second derivatives reach two below
// each, giving the windows `[−2, 32]` and `[−43, 17]`.
const A_LO: i32 = -2;
const A_N: usize = 35;
const B_LO: i32 = -43;
const B_N: usize = 61;

/// The six dimensionless Gibbs derivatives at reduced `(π, τ)`.
fn gamma(pi: f64, tau: f64) -> Gibbs {
    let a = 7.1 - pi; // (7.1 − π) ≥ 1.05 over the whole region
    let b = tau - 1.222; // (τ − 1.222) ≥ 1.0 over the whole region
    let pa = powers::<A_N>(a, A_LO);
    let pb = powers::<B_N>(b, B_LO);
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
        // aᵏ = a^(I−k), bᵏ = b^(J−k) — the three powers each sum needs.
        let ia = (i - A_LO) as usize;
        let jb = (j - B_LO) as usize;
        let (a0, a1, a2) = (pa[ia], pa[ia - 1], pa[ia - 2]);
        let (b0, b1, b2) = (pb[jb], pb[jb - 1], pb[jb - 2]);
        d.g += n * a0 * b0;
        d.gp -= n * fi * a1 * b0;
        d.gpp += n * fi * (fi - 1.0) * a2 * b0;
        d.gt += n * a0 * fj * b1;
        d.gtt += n * a0 * fj * (fj - 1.0) * b2;
        d.gpt -= n * fi * a1 * fj * b1;
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

/// `(∂ρ/∂p)_T` for a region-1 state.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// `(∂ρ/∂p)_T` in **(kg/m³)/kPa**.
///
/// Analytic: `v = R·T·γ_π/p*` (the `π` cancels the explicit `p`), so one more
/// derivative in `π` gives `(∂v/∂p)_T = R·T·γ_ππ/p*²`, and
/// `(∂ρ/∂p)_T = −(∂v/∂p)_T / v²`. Needed by the R15-11 critical enhancement
/// (see [`crate::transport`]); never finite-differenced.
pub(crate) fn drho_dp(t: f64, p_kpa: f64) -> f64 {
    let pstar_kpa = PSTAR_MPA * 1e3;
    let pi = p_kpa / pstar_kpa;
    let tau = TSTAR / t;
    let d = gamma(pi, tau);
    let v = R * t * d.gp / pstar_kpa;
    let dv_dp = R * t * d.gpp / (pstar_kpa * pstar_kpa);
    -dv_dp / (v * v)
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

    /// Test oracle: the literal `powi`-per-derivative formulation the
    /// power-table [`gamma`] replaced. Kept so the optimization is pinned to
    /// the original algebra rather than only to the three Table-5 points.
    fn gamma_powi(pi: f64, tau: f64) -> Gibbs {
        let a = 7.1 - pi;
        let b = tau - 1.222;
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

    /// The power-table series must agree with the `powi` oracle everywhere in
    /// region 1, not just at the acceptance points.
    ///
    /// Measured worst case over this grid: **1.73e-11**, on `γ_π` — the sum
    /// with the most cancellation, where terms orders of magnitude larger than
    /// the total amplify a last-ulp difference. The assertion sits just above
    /// that, so a real numerical regression trips it. (IF97 states its own
    /// acceptance tables to 1e-8; `table_5_verification` holds both
    /// formulations to that.)
    #[test]
    fn power_table_matches_powi_oracle() {
        for t in [273.16, 300.0, 400.0, 500.0, 600.0, 623.15] {
            for p_kpa in [1.0, 100.0, 1_000.0, 10_000.0, 50_000.0, 100_000.0] {
                let pi = (p_kpa * 1e-3) / PSTAR_MPA;
                let tau = TSTAR / t;
                let (fast, slow) = (gamma(pi, tau), gamma_powi(pi, tau));
                assert_relative_eq!(fast.g, slow.g, max_relative = 3e-11);
                assert_relative_eq!(fast.gp, slow.gp, max_relative = 3e-11);
                assert_relative_eq!(fast.gpp, slow.gpp, max_relative = 3e-11);
                assert_relative_eq!(fast.gt, slow.gt, max_relative = 3e-11);
                assert_relative_eq!(fast.gtt, slow.gtt, max_relative = 3e-11);
                assert_relative_eq!(fast.gpt, slow.gpt, max_relative = 3e-11);
            }
        }
    }
}
