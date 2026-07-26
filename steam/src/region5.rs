//! IF97 Region 5 — high-temperature steam, 1073.15–2273.15 K, ≤ 50 MPa.
//!
//! Same Gibbs-energy split as region 2 (`γ = γ° + γʳ`) but with its own small
//! coefficient tables and reduced variables `π = p/1 MPa`, `τ = 1000 K/T`.
//! The residual here uses `τ^Jᵢ` directly (not `(τ−0.5)^Jᵢ`).
//! Ref: R7-97(2012) §7 (Eqs. 32–34), Tables 37–38.
//! Verification: Table 42 (e.g. `T=1500 K, p=0.5 MPa` →
//! `v=0.138455090×10¹ m³/kg`, `h=0.521976855×10⁴ kJ/kg`).

use crate::R;
use crate::coefficients::{REGION5_IDEAL, REGION5_RES};
use crate::props::{Gibbs, Props, gibbs_props};

/// Reducing temperature `T*` for region 5, in **K** (`p* = 1 MPa`).
const TSTAR: f64 = 1000.0;

fn gamma(pi: f64, tau: f64) -> Gibbs {
    // Ideal part. γ° = ln π + Σ n τ^J.
    let mut g = pi.ln();
    let mut gt = 0.0;
    let mut gtt = 0.0;
    for term in &REGION5_IDEAL {
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

    // Residual part. γʳ = Σ n π^I τ^J  (note: τ^J, not (τ−0.5)^J).
    for term in &REGION5_RES {
        let (i, j, n) = (term.i, term.j, term.n);
        let (fi, fj) = (i as f64, j as f64);
        d.g += n * pi.powi(i) * tau.powi(j);
        d.gp += n * fi * pi.powi(i - 1) * tau.powi(j);
        d.gpp += n * fi * (fi - 1.0) * pi.powi(i - 2) * tau.powi(j);
        d.gt += n * pi.powi(i) * fj * tau.powi(j - 1);
        d.gtt += n * pi.powi(i) * fj * (fj - 1.0) * tau.powi(j - 2);
        d.gpt += n * fi * pi.powi(i - 1) * fj * tau.powi(j - 1);
    }
    d
}

/// Full property set for a region-5 state point.
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

/// `(∂ρ/∂p)_T` for a region-5 state.
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

    /// R7-97(2012) Table 42 — three verification points.
    #[test]
    fn table_42_verification() {
        // Columns: T[K], p[kPa], v, h, u, s, cp, w.
        let cases = [
            (
                1500.0,
                500.0,
                0.138455090e1,
                0.521976855e4,
                0.452749310e4,
                0.965408875e1,
                0.261609445e1,
                0.917068690e3,
            ),
            (
                1500.0,
                30_000.0,
                0.230761299e-1,
                0.516723514e4,
                0.447495124e4,
                0.772970133e1,
                0.272724317e1,
                0.928548002e3,
            ),
            (
                2000.0,
                30_000.0,
                0.311385219e-1,
                0.657122604e4,
                0.563707038e4,
                0.853640523e1,
                0.288569882e1,
                0.106736948e4,
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
}
