//! IF97 Region 2 — superheated vapor, up to 1073.15 K and 100 MPa.
//!
//! The dimensionless Gibbs energy splits into an **ideal-gas** part and a
//! **residual** part: `γ(π,τ) = γ°(π,τ) + γʳ(π,τ)` (R7-97(2012) Eqs. 15–17):
//!
//! - `γ° = ln π + Σ nᵢ° τ^Jᵢ°`         (ideal, Table 10, 9 terms)
//! - `γʳ = Σ nᵢ π^Iᵢ (τ−0.5)^Jᵢ`       (residual, Table 11, 43 terms)
//!
//! Reduced variables: `π = p/1 MPa`, `τ = 540 K/T`. Explicit in `(T, p)`.
//! Verification: R7-97 Table 15 (e.g. `T=700 K, p=30 MPa` →
//! `v=0.542946619×10⁻² m³/kg`, `h=0.263149474×10⁴ kJ/kg`,
//! `s=0.517540298×10¹ kJ/(kg·K)`).

use crate::R;
use crate::coefficients::{REGION2_IDEAL, REGION2_RES};
use crate::props::{Gibbs, Props, gibbs_props};
use crate::series::powers;

/// Reducing temperature `T*` for region 2, in **K** (`p* = 1 MPa`).
const TSTAR: f64 = 540.0;

// Exponent windows for the power tables (see [`crate::series`]). Ideal Table 10
// has `J° ∈ [−5, 3]` → window `[−7, 3]`; residual Table 11 has `I ∈ [1, 24]`
// and `J ∈ [0, 58]` → windows `[−1, 24]` and `[−2, 58]`.
const T0_LO: i32 = -7;
const T0_N: usize = 11;
const PI_LO: i32 = -1;
const PI_N: usize = 26;
const B_LO: i32 = -2;
const B_N: usize = 61;

/// The six dimensionless Gibbs derivatives `γ = γ° + γʳ`.
fn gamma(pi: f64, tau: f64) -> Gibbs {
    // Ideal part. γ° = ln π + Σ n τ^J.
    let pt = powers::<T0_N>(tau, T0_LO);
    let mut g = pi.ln();
    let mut gt = 0.0;
    let mut gtt = 0.0;
    for term in &REGION2_IDEAL {
        let (j, n) = (term.j, term.n);
        let fj = j as f64;
        let jt = (j - T0_LO) as usize;
        g += n * pt[jt];
        gt += n * fj * pt[jt - 1];
        gtt += n * fj * (fj - 1.0) * pt[jt - 2];
    }
    let mut d = Gibbs {
        g,
        gp: 1.0 / pi,
        gpp: -1.0 / (pi * pi),
        gt,
        gtt,
        gpt: 0.0,
    };

    // Residual part. γʳ = Σ n π^I (τ−0.5)^J.
    let b = tau - 0.5;
    let pp = powers::<PI_N>(pi, PI_LO);
    let pb = powers::<B_N>(b, B_LO);
    for term in &REGION2_RES {
        let (i, j, n) = (term.i, term.j, term.n);
        let (fi, fj) = (i as f64, j as f64);
        let ip = (i - PI_LO) as usize;
        let jb = (j - B_LO) as usize;
        let (p0, p1, p2) = (pp[ip], pp[ip - 1], pp[ip - 2]);
        let (b0, b1, b2) = (pb[jb], pb[jb - 1], pb[jb - 2]);
        d.g += n * p0 * b0;
        d.gp += n * fi * p1 * b0;
        d.gpp += n * fi * (fi - 1.0) * p2 * b0;
        d.gt += n * p0 * fj * b1;
        d.gtt += n * p0 * fj * (fj - 1.0) * b2;
        d.gpt += n * fi * p1 * fj * b1;
    }
    d
}

/// Full property set for a region-2 state point.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// Mass-basis [`Props`] (see that struct for units).
pub(crate) fn props(t: f64, p_kpa: f64) -> Props {
    let pi = p_kpa * 1e-3; // p* = 1 MPa
    let tau = TSTAR / t;
    gibbs_props(gamma(pi, tau), pi, tau, t, p_kpa)
}

/// `(∂ρ/∂p)_T` for a region-2 state.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// `(∂ρ/∂p)_T` in **(kg/m³)/kPa**.
///
/// Analytic — see [`crate::region1::drho_dp`] for the derivation. Here
/// `p* = 1 MPa`, so the reducing pressure is 1000 kPa.
pub(crate) fn drho_dp(t: f64, p_kpa: f64) -> f64 {
    const PSTAR_KPA: f64 = 1e3;
    let pi = p_kpa / PSTAR_KPA;
    let tau = TSTAR / t;
    let d = gamma(pi, tau);
    let v = R * t * d.gp / PSTAR_KPA;
    let dv_dp = R * t * d.gpp / (PSTAR_KPA * PSTAR_KPA);
    -dv_dp / (v * v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// R7-97(2012) Table 15 — three verification points, full precision.
    #[test]
    fn table_15_verification() {
        // Columns: T[K], p[kPa], v, h, u, s, cp, w.
        let cases = [
            (
                300.0,
                3.5,
                0.394913866e2,
                0.254991145e4,
                0.241169160e4,
                0.852238967e1,
                0.191300162e1,
                0.427920172e3,
            ),
            (
                700.0,
                3.5,
                0.923015898e2,
                0.333568375e4,
                0.301262819e4,
                0.101749996e2,
                0.208141274e1,
                0.644289068e3,
            ),
            (
                700.0,
                30_000.0,
                0.542946619e-2,
                0.263149474e4,
                0.246861076e4,
                0.517540298e1,
                0.103505092e2,
                0.480386523e3,
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

    #[test]
    fn h_u_pv_identity() {
        let (t, p) = (700.0, 3.5);
        let r = props(t, p);
        assert_relative_eq!(r.h, r.u + p * r.v, max_relative = 1e-12);
    }

    /// Test oracle: the literal `powi`-per-derivative formulation the
    /// power-table [`gamma`] replaced.
    fn gamma_powi(pi: f64, tau: f64) -> Gibbs {
        let mut g = pi.ln();
        let mut gt = 0.0;
        let mut gtt = 0.0;
        for term in &REGION2_IDEAL {
            let (j, n) = (term.j, term.n);
            let fj = j as f64;
            g += n * tau.powi(j);
            gt += n * fj * tau.powi(j - 1);
            gtt += n * fj * (fj - 1.0) * tau.powi(j - 2);
        }
        let mut d = Gibbs {
            g,
            gp: 1.0 / pi,
            gpp: -1.0 / (pi * pi),
            gt,
            gtt,
            gpt: 0.0,
        };
        let b = tau - 0.5;
        for term in &REGION2_RES {
            let (i, j, n) = (term.i, term.j, term.n);
            let (fi, fj) = (i as f64, j as f64);
            d.g += n * pi.powi(i) * b.powi(j);
            d.gp += n * fi * pi.powi(i - 1) * b.powi(j);
            d.gpp += n * fi * (fi - 1.0) * pi.powi(i - 2) * b.powi(j);
            d.gt += n * pi.powi(i) * fj * b.powi(j - 1);
            d.gtt += n * pi.powi(i) * fj * (fj - 1.0) * b.powi(j - 2);
            d.gpt += n * fi * pi.powi(i - 1) * fj * b.powi(j - 1);
        }
        d
    }

    /// The power-table series must agree with the `powi` oracle across the
    /// region, including the hardest corner: `T → 1073 K`, where `τ − 0.5`
    /// shrinks to ~3e-3 and the `J = 58` powers reach ~1e-150.
    ///
    /// Measured worst case over this grid: **7.7e-14** — the best-behaved of
    /// the three regions, since `γʳ` here has far less cancellation than
    /// region 1's `γ_π`.
    #[test]
    fn power_table_matches_powi_oracle() {
        for t in [273.16, 400.0, 600.0, 800.0, 1000.0, 1073.15] {
            for p_kpa in [1.0, 100.0, 3_000.0, 20_000.0, 60_000.0, 100_000.0] {
                let pi = p_kpa * 1e-3;
                let tau = TSTAR / t;
                let (fast, slow) = (gamma(pi, tau), gamma_powi(pi, tau));
                assert_relative_eq!(fast.g, slow.g, max_relative = 2e-13);
                assert_relative_eq!(fast.gp, slow.gp, max_relative = 2e-13);
                assert_relative_eq!(fast.gpp, slow.gpp, max_relative = 2e-13);
                assert_relative_eq!(fast.gt, slow.gt, max_relative = 2e-13);
                assert_relative_eq!(fast.gtt, slow.gtt, max_relative = 2e-13);
                assert_relative_eq!(fast.gpt, slow.gpt, max_relative = 2e-13);
            }
        }
    }
}
