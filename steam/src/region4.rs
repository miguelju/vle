//! IF97 Region 4 — the saturation line, 273.15 K … 647.096 K.
//!
//! Region 4 is the two-phase boundary (the "dome"). Unlike the other regions
//! it is **not** a property surface — it is a single implicit equation that
//! relates saturation pressure and saturation temperature, and IF97 gives it
//! in a form that solves **closed-form both ways**:
//!
//! - [`psat`] — saturation pressure at a given temperature, `Psat(T)`.
//! - [`tsat`] — saturation temperature at a given pressure, `Tsat(P)`
//!   (the "backward" equation), also non-iterative.
//! - [`d_psat_d_t`] — the analytic derivative `dPsat/dT`, used by the
//!   Clausius–Clapeyron consistency test and by users doing their own
//!   derivative work. Per the project rule, this is analytic — never a
//!   finite difference.
//!
//! Reference: IAPWS R7-97(2012) §8, Eqs. (30)–(31), Table 34 (coefficients).
//! Verification values: Table 35 (`psat(300 K) = 0.353658941×10⁻² MPa`,
//! `psat(500 K) = 0.263889776×10¹ MPa`, `psat(600 K) = 0.123443146×10² MPa`)
//! and Table 36 (`Tsat(0.1 MPa) = 0.372755919×10³ K`,
//! `Tsat(1 MPa) = 0.453035632×10³ K`, `Tsat(10 MPa) = 0.584149488×10³ K`).
//!
//! Internal units follow IF97: temperature in **K**, pressure in **MPa**.
//! The public [`crate`] API converts to the repo-canonical kPa at its
//! boundary.

/// The ten region-4 coefficients `n₁ … n₁₀` from R7-97(2012) Table 34.
const N: [f64; 10] = [
    0.11670521452767e4,
    -0.72421316703206e6,
    -0.17073846940092e2,
    0.12020824702470e5,
    -0.32325550322333e7,
    0.14915108613530e2,
    -0.48232657361591e4,
    0.40511340542057e6,
    -0.23855557567849e0,
    0.65017534844798e3,
];

/// Lowest saturation temperature (triple-point temperature), in **K**.
pub const T_MIN: f64 = 273.15;
/// Highest saturation temperature (critical temperature), in **K**.
pub const T_MAX: f64 = 647.096;
/// Lowest saturation pressure `Psat(273.15 K)`, in **MPa**.
pub const P_MIN_MPA: f64 = 0.000_611_212_677;
/// Highest saturation pressure (critical pressure), in **MPa**.
pub const P_MAX_MPA: f64 = 22.064;

/// Saturation pressure at temperature `t`.
///
/// # Arguments
/// * `t` — Temperature in **K** (Kelvin), valid on `273.15 ≤ t ≤ 647.096`.
///
/// # Returns
/// Saturation pressure in **MPa**.
///
/// Ref: R7-97(2012) Eq. (30).
pub fn psat(t: f64) -> f64 {
    // ϑ = T/T* + n₉/(T/T* − n₁₀), with T* = 1 K.
    let theta = t + N[8] / (t - N[9]);
    let a = theta * theta + N[0] * theta + N[1];
    let b = N[2] * theta * theta + N[3] * theta + N[4];
    let c = N[5] * theta * theta + N[6] * theta + N[7];
    // Psat/p* = [2C / (−B + √(B²−4AC))]⁴, with p* = 1 MPa.
    let base = 2.0 * c / (-b + (b * b - 4.0 * a * c).sqrt());
    base.powi(4)
}

/// Saturation temperature at pressure `p` (the backward equation).
///
/// # Arguments
/// * `p` — Pressure in **MPa**, valid on `Psat(273.15) ≤ p ≤ 22.064`.
///
/// # Returns
/// Saturation temperature in **K** (Kelvin).
///
/// Ref: R7-97(2012) Eq. (31).
pub fn tsat(p: f64) -> f64 {
    // β = (p/p*)^(1/4), p* = 1 MPa.
    let beta = p.powf(0.25);
    let e = beta * beta + N[2] * beta + N[5];
    let f = N[0] * beta * beta + N[3] * beta + N[6];
    let g = N[1] * beta * beta + N[4] * beta + N[7];
    let d = 2.0 * g / (-f - (f * f - 4.0 * e * g).sqrt());
    let inner = (N[9] + d) * (N[9] + d) - 4.0 * (N[8] + N[9] * d);
    (N[9] + d - inner.sqrt()) / 2.0
}

/// Analytic derivative `dPsat/dT` of the saturation-pressure curve.
///
/// # Arguments
/// * `t` — Temperature in **K**.
///
/// # Returns
/// `dPsat/dT` in **MPa/K**.
///
/// Derived by differentiating Eq. (30) through the chain
/// `Psat = p*·β⁴(ϑ(T))`; used for the Clausius–Clapeyron consistency check.
pub fn d_psat_d_t(t: f64) -> f64 {
    let theta = t + N[8] / (t - N[9]);
    // dϑ/dT = 1 − n₉/(T − n₁₀)².
    let dtheta_dt = 1.0 - N[8] / ((t - N[9]) * (t - N[9]));

    let a = theta * theta + N[0] * theta + N[1];
    let b = N[2] * theta * theta + N[3] * theta + N[4];
    let c = N[5] * theta * theta + N[6] * theta + N[7];
    let a_p = 2.0 * theta + N[0];
    let b_p = 2.0 * N[2] * theta + N[3];
    let c_p = 2.0 * N[5] * theta + N[6];

    let disc = b * b - 4.0 * a * c;
    let d = disc.sqrt();
    // D' = (B·B' − 2(A'C + A·C')) / D.
    let d_p = (b * b_p - 2.0 * (a_p * c + a * c_p)) / d;

    let den = -b + d; // denominator of β
    let den_p = -b_p + d_p;
    let beta = 2.0 * c / den;
    // β' w.r.t. ϑ.
    let beta_p = 2.0 * (c_p * den - c * den_p) / (den * den);

    // Psat = p*·β⁴  ⇒ dPsat/dϑ = 4β³β';  ×dϑ/dT.  p* = 1 MPa.
    4.0 * beta.powi(3) * beta_p * dtheta_dt
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn psat_verification_table_35() {
        // R7-97(2012) Table 35 — full 9-significant-figure precision.
        assert_relative_eq!(psat(300.0), 0.353658941e-2, max_relative = 1e-8);
        assert_relative_eq!(psat(500.0), 0.263889776e1, max_relative = 1e-8);
        assert_relative_eq!(psat(600.0), 0.123443146e2, max_relative = 1e-8);
    }

    #[test]
    fn tsat_verification_table_36() {
        // R7-97(2012) Table 36.
        assert_relative_eq!(tsat(0.1), 0.372755919e3, max_relative = 1e-8);
        assert_relative_eq!(tsat(1.0), 0.453035632e3, max_relative = 1e-8);
        assert_relative_eq!(tsat(10.0), 0.584149488e3, max_relative = 1e-8);
    }

    #[test]
    fn psat_tsat_round_trip() {
        for &t in &[280.0, 350.0, 450.0, 550.0, 640.0] {
            let p = psat(t);
            assert_relative_eq!(tsat(p), t, max_relative = 1e-10);
        }
    }

    #[test]
    fn d_psat_matches_finite_difference() {
        // The analytic derivative is the product code; finite difference is
        // only the oracle here (per the project's derivative rule).
        for &t in &[300.0, 400.0, 500.0, 600.0] {
            let h = 1e-4;
            let fd = (psat(t + h) - psat(t - h)) / (2.0 * h);
            assert_relative_eq!(d_psat_d_t(t), fd, max_relative = 1e-6);
        }
    }

    #[test]
    fn critical_and_triple_endpoints() {
        // Psat at the critical temperature is the critical pressure.
        assert_relative_eq!(psat(T_MAX), P_MAX_MPA, max_relative = 1e-4);
        // Psat at the triple point matches the tabulated triple pressure.
        assert_relative_eq!(psat(T_MIN), P_MIN_MPA, max_relative = 1e-6);
    }
}
