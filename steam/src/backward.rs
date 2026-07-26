//! IF97 backward equations `T(p,h)` and `T(p,s)` for regions 1 and 2.
//!
//! The forward Gibbs equations give `h(T,p)` and `s(T,p)`; a PH or PS flash
//! needs the *inverse* — `T` from `(p,h)` or `(p,s)`. IF97 supplies explicit
//! polynomial backward equations for both regions, region 2 split into three
//! sub-regions apiece, and all four are implemented here.
//!
//! Each is used as a **seed**, not as the final answer: [`crate::state`]
//! follows it with a Newton polish on the forward equation, so the forward
//! surface remains the accuracy authority while the polynomial supplies the
//! speed. Region 3 and region 5 still fall back to a bracketed forward solve —
//! region 3's supplementary backward equations are a deliberate deferral,
//! recorded in `steam_audit.md`.
//!
//! Solving region 2 by bracketed iteration instead — as this crate did until
//! the 13.8 audit — cost `inverse/ph_vapor` 20.0 µs and `inverse/ps_vapor`
//! 5.2 µs, against 1.9 µs and 1.8 µs for their region-1 counterparts, because
//! every iteration re-evaluated the 52-term region-2 surface.
//!
//! Reference: R7-97(2012) §5.2.1 (Eq. 11, Table 6), §5.2.2 (Eq. 13, Table 8),
//! §6.3.1 (Eqs. 22–24, Tables 20–22) and §6.3.2 (Eqs. 25–27, Tables 25–27).
//! Verification: Tables 7, 9, 24 and 29 — all asserted below.
//!
//! Units: `p` in **kPa**, `h` in **kJ/kg**, `s` in **kJ/(kg·K)**, `T` in **K**.

use crate::coefficients::{
    REGION1_TPH, REGION1_TPS, REGION2A_TPH, REGION2A_TPS, REGION2B_TPH, REGION2B_TPS, REGION2C_TPH,
    REGION2C_TPS, Term,
};

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

fn tph_sum(terms: &[Term], x: f64, y: f64) -> f64 {
    terms
        .iter()
        .map(|term| term.n * x.powi(term.i) * y.powi(term.j))
        .sum()
}

/// IF97 backward `T(p,h)` for region 2, including the 2a/2b/2c dispatch.
pub(crate) fn t_ph_region2(p_kpa: f64, h: f64) -> f64 {
    let p = p_kpa * 1e-3;
    let eta = h / 2000.0;
    if p <= 4.0 {
        tph_sum(&REGION2A_TPH, p, eta - 2.1)
    } else if p <= 6.546_699_678
        || h >= 2_652.657_190_842_8 + ((p - 4.525_757_890_594_8) / 1.280_900_273_013_6e-4).sqrt()
    {
        tph_sum(&REGION2B_TPH, p - 2.0, eta - 2.6)
    } else {
        tph_sum(&REGION2C_TPH, p + 25.0, eta - 1.8)
    }
}

/// Entropy reducing value `s*` for subregion 2a, **kJ/(kg·K)** (R7-97 Eq. 25).
const S_STAR_2A: f64 = 2.0;
/// Entropy reducing value `s*` for subregion 2b, **kJ/(kg·K)** (R7-97 Eq. 26).
///
/// This is **not** π/4. It merely resembles it to four decimals, which is why
/// the `approx_constant` lint has to be waived here: 0.7853 is the reducing
/// value printed in the standard, and rounding it to π/4 would change every
/// 2b temperature.
#[allow(clippy::approx_constant)]
const S_STAR_2B: f64 = 0.7853;
/// Entropy reducing value `s*` for subregion 2c, **kJ/(kg·K)** (R7-97 Eq. 27).
const S_STAR_2C: f64 = 2.9251;
/// The 2b/2c dividing isentrope, **kJ/(kg·K)** (R7-97 §6.3.2).
const S_2BC: f64 = 5.85;

/// IF97 backward `T(p,s)` for region 2, including the 2a/2b/2c dispatch.
///
/// # Arguments
/// * `p_kpa` — Pressure in **kPa absolute**.
/// * `s` — Specific entropy in **kJ/(kg·K)**.
///
/// # Returns
/// Temperature in **K** (`T* = 1 K`).
///
/// The subregion split is simpler than `T(p,h)`'s: the 2a/2b boundary is just
/// the 4 MPa isobar and the 2b/2c boundary the isentrope
/// `s = 5.85 kJ/(kg·K)` — no auxiliary boundary equation (R7-97(2012) §6.3.2).
/// Each subregion carries its own entropy reducing value `s*`, and 2a is the
/// one equation in IF97 with fractional pressure exponents; see
/// [`REGION2A_TPS`] for how those are stored.
///
/// Ref: IAPWS R7-97(2012) Eqs. (25)–(27), Tables 25–27.
pub(crate) fn t_ps_region2(p_kpa: f64, s: f64) -> f64 {
    let p = p_kpa * 1e-3; // p* = 1 MPa
    if p <= 4.0 {
        // 2a: Iᵢ are quarter-integers — raise π^¼ to the stored I×4.
        let (root, sigma) = (p.sqrt().sqrt(), s / S_STAR_2A);
        REGION2A_TPS
            .iter()
            .map(|t| t.n * root.powi(t.i) * (sigma - 2.0).powi(t.j))
            .sum()
    } else if s >= S_2BC {
        tph_sum(&REGION2B_TPS, p, 10.0 - s / S_STAR_2B)
    } else {
        tph_sum(&REGION2C_TPS, p, 2.0 - s / S_STAR_2C)
    }
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

    #[test]
    fn tables_21_to_23_tph_region2() {
        for (p, h, expected) in [
            (0.001, 3000.0, 0.534433241e3),
            (3.0, 4000.0, 0.101077577e4),
            (5.0, 4000.0, 0.101531583e4),
            (25.0, 3500.0, 0.875279054e3),
            (40.0, 2700.0, 0.743056411e3),
            (60.0, 3200.0, 0.882756860e3),
        ] {
            assert_relative_eq!(t_ph_region2(p * 1000.0, h), expected, max_relative = 1e-8);
        }
    }

    /// R7-97(2012) **Table 29** — the computer-program verification points for
    /// Eqs. (25)–(27), three per subregion, at the published precision.
    ///
    /// These are the acceptance test for the 120 transcribed coefficients: a
    /// single mistyped mantissa or power-of-ten would move at least one point
    /// in its subregion far outside 1e-8.
    #[test]
    fn table_29_tps_region2() {
        for (p, s, expected) in [
            // Eq. (25) — subregion 2a
            (0.1, 7.5, 0.399517097e3),
            (0.1, 8.0, 0.514127081e3),
            (2.5, 8.0, 0.103984917e4),
            // Eq. (26) — subregion 2b
            (8.0, 6.0, 0.600484040e3),
            (8.0, 7.5, 0.106495556e4),
            (90.0, 6.0, 0.103801126e4),
            // Eq. (27) — subregion 2c
            (20.0, 5.75, 0.697992849e3),
            (80.0, 5.25, 0.854011484e3),
            (80.0, 5.75, 0.949017998e3),
        ] {
            assert_relative_eq!(t_ps_region2(p * 1000.0, s), expected, max_relative = 1e-8);
        }
    }
}
