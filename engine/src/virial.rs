//! Truncated virial equation of state (second coefficient only).
//!
//! The virial EOS represents the compressibility factor as a power series
//! in density (or pressure): `Z = 1 + B·ρ + C·ρ² + …`. Truncating after
//! the second term gives the simplest non-ideal corrector to the
//! ideal-gas law:
//!
//! ```text
//!   Z = 1 + B·P/(R·T)
//! ```
//!
//! where `B(T)` is the temperature-dependent **second virial coefficient**
//! (units **cm³/mol** when paired with R in kJ/(kmol·K) and P in kPa, with
//! a careful 1e3 factor — see `pitzer_b` below).
//!
//! The Pitzer correlation (Ref (5), Abbott 1989) gives `B` in
//! corresponding-states form:
//!
//! ```text
//!   B·Pc/(R·Tc) = B⁰(Tr) + ω · B¹(Tr)
//!   B⁰(Tr) = 0.083 − 0.422 / Tr^1.6
//!   B¹(Tr) = 0.139 − 0.172 / Tr^4.2
//! ```
//!
//! The truncated form is valid at **low to moderate reduced pressures**
//! (roughly Pr < Tr / 2). Beyond that, cubic EOS is the better tool —
//! `engine::eos` is the production path. Virial stays in the toolkit
//! because (a) it gives clean, closed-form fugacity / departure
//! expressions that are useful for K-value initialization, and (b) the
//! Chapter IV validation cases include a sanity check against it.
//!
//! # Source
//! - VB6 `legacy/vb6/clsVirial.cls` (pure) and `clsVirialMulticomp.cls`
//!   (mixture). The Pitzer B⁰/B¹ coefficients here are bit-identical to
//!   those used by the legacy code.

use crate::types::{Component, R_GAS};
use thiserror::Error;

/// Errors raised by the virial layer.
#[derive(Debug, Error, PartialEq)]
pub enum VirialError {
    /// Pure-component or mixture input was inconsistent (zero or
    /// negative critical properties, mismatched composition vector).
    #[error("invalid input to virial layer: {0}")]
    InvalidInput(String),
}

/// Pitzer B⁰(Tr) — non-polar component contribution.
///
/// `B⁰ = 0.083 − 0.422 / Tr^1.6`.
///
/// Returned value is **dimensionless** (it's the simple-fluid part of
/// the reduced second virial coefficient `B·Pc/(R·Tc)`).
pub fn pitzer_b0(tr: f64) -> f64 {
    0.083 - 0.422 / tr.powf(1.6)
}

/// Pitzer B¹(Tr) — acentric-factor correction.
///
/// `B¹ = 0.139 − 0.172 / Tr^4.2`.
///
/// Dimensionless. Multiplied by ω in the full Pitzer expression.
pub fn pitzer_b1(tr: f64) -> f64 {
    0.139 - 0.172 / tr.powf(4.2)
}

/// Second virial coefficient `B(T)` for a pure component, in **cm³/mol**.
///
/// `B·Pc/(R·Tc) = B⁰ + ω·B¹` →  `B = (R·Tc/Pc)·(B⁰ + ω·B¹)`.
///
/// The 1e3 factor converts the canonical-unit product
/// `R[kJ/(kmol·K)] · Tc[K] / Pc[kPa] = m³/kmol` to **cm³/mol**.
///
/// # Arguments
/// * `comp` — Component (uses `tc`, `pc`, `omega`).
/// * `t` — Temperature in **K**.
///
/// # Returns
/// `B(T)` in **cm³/mol**.
pub fn pitzer_b(comp: &Component, t: f64) -> f64 {
    let tr = t / comp.tc;
    let b_reduced = pitzer_b0(tr) + comp.omega * pitzer_b1(tr);
    // R·Tc/Pc in m³/kmol → ·1000 → cm³/mol  (1 m³/kmol = 1000 cm³/mol).
    1000.0 * R_GAS * comp.tc / comp.pc * b_reduced
}

/// dB/dT for a pure component, in **cm³/(mol·K)** — needed for H^R and S^R.
///
/// Differentiate the Pitzer correlation with respect to T (Tr = T/Tc):
///   dB⁰/dTr = 0.422 · 1.6 · Tr^(−2.6)
///   dB¹/dTr = 0.172 · 4.2 · Tr^(−5.2)
///   dB/dT  = (R·Tc/Pc)·(dB⁰/dTr + ω·dB¹/dTr) · (1/Tc)
///          = (R/Pc)·(dB⁰/dTr + ω·dB¹/dTr)
/// with the same 1e3 m³/kmol → cm³/mol unit factor.
pub fn pitzer_d_b_d_t(comp: &Component, t: f64) -> f64 {
    let tr = t / comp.tc;
    let db0_dtr = 0.422 * 1.6 * tr.powf(-2.6);
    let db1_dtr = 0.172 * 4.2 * tr.powf(-5.2);
    1000.0 * R_GAS / comp.pc * (db0_dtr + comp.omega * db1_dtr)
}

/// Compressibility factor `Z` from the truncated virial equation, pure.
///
/// `Z = 1 + B·P/(R·T)`. Returns a single value (the virial truncated at
/// the second coefficient has no liquid root — it's a vapor-only model).
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p` — Pressure in **kPa absolute**.
///
/// # Returns
/// Z, dimensionless.
pub fn z_factor_virial(comp: &Component, t: f64, p: f64) -> f64 {
    let b = pitzer_b(comp, t); // cm³/mol
    // P (kPa) · B (cm³/mol) / (R (kJ/(kmol·K)) · T (K))
    // Units: kPa·cm³/(mol) / (kJ/(kmol·K)·K) = kPa·cm³/mol / (kJ/kmol)
    //        = kPa·cm³/mol / (1000 J/mol·1) = kPa·cm³/(1000·J/mol).
    // Since 1 kPa·cm³ = 1 J/mol·1e-3 ... let's just multiply out:
    // R_GAS = 8.31451 kJ/(kmol·K) = 8.31451 J/(mol·K) (since kJ/kmol = J/mol).
    // So R·T in J/mol. B·P with B in cm³/mol = (1e-6 m³/mol), P in kPa = 1000 Pa:
    //   B·P → 1e-6 m³/mol · 1000 Pa = 1e-3 Pa·m³/mol = 1e-3 J/mol.
    // We want B·P/(R·T) dimensionless, R·T in J/mol → multiply B·P (cm³/mol·kPa)
    // by 1e-3 to convert to J/mol. Equivalently divide by 1000.
    1.0 + b * p / (1000.0 * R_GAS * t)
}

/// Fugacity coefficient `ln(φ)` from the truncated virial equation, pure.
///
/// Derivation (textbook): `ln(φ) = B·P/(R·T)`.
/// (Same units gymnastics as [`z_factor_virial`] — note the 1/1000.)
pub fn ln_phi_pure_virial(comp: &Component, t: f64, p: f64) -> f64 {
    let b = pitzer_b(comp, t);
    b * p / (1000.0 * R_GAS * t)
}

/// Departure enthalpy `H^R/(R·T)`, dimensionless.
///
/// `H^R/RT = (B − T·dB/dT) · P / (R·T)`.
pub fn h_departure_rt_virial(comp: &Component, t: f64, p: f64) -> f64 {
    let b = pitzer_b(comp, t);
    let db_dt = pitzer_d_b_d_t(comp, t);
    (b - t * db_dt) * p / (1000.0 * R_GAS * t)
}

/// Departure entropy `S^R/R`, dimensionless.
///
/// `S^R/R = −P · dB/dT / R`. (Multiplied by 1e−3 for the cm³ → m³ unit shift.)
pub fn s_departure_r_virial(comp: &Component, t: f64, p: f64) -> f64 {
    let db_dt = pitzer_d_b_d_t(comp, t);
    -p * db_dt / (1000.0 * R_GAS)
}

// ===========================================================================
// Multicomponent virial — second virial mixing.
// ===========================================================================

/// Build the binary B_{ij} matrix for a mixture using Pitzer combining
/// rules (Tcij = √(Tci·Tcj)·(1 − k_ij), Pcij = Zcij·R·Tcij/Vcij, …).
///
/// For M7.1 simplicity we use the simplest rules: Tcij = √(Tci·Tcj),
/// Pcij = (Pci + Pcj)/2, ωij = (ωi + ωj)/2. The Tsonopoulos/Hayden
/// O'Connell refinements (which the VB6 code optionally uses) are
/// deferred to a later milestone.
pub fn b_mix_matrix(components: &[Component], t: f64) -> Vec<Vec<f64>> {
    let n = components.len();
    let mut mat = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in i..n {
            let bij = if i == j {
                pitzer_b(&components[i], t)
            } else {
                // Mixing rules for cross-term cubic properties.
                let tc_ij = (components[i].tc * components[j].tc).sqrt();
                let pc_ij = 0.5 * (components[i].pc + components[j].pc);
                let omega_ij = 0.5 * (components[i].omega + components[j].omega);
                let tr = t / tc_ij;
                let b_reduced = pitzer_b0(tr) + omega_ij * pitzer_b1(tr);
                1000.0 * R_GAS * tc_ij / pc_ij * b_reduced
            };
            mat[i][j] = bij;
            mat[j][i] = bij;
        }
    }
    mat
}

/// Mixture second virial coefficient `B_mix(T, x)` in **cm³/mol**.
///
/// `B_mix = ΣᵢΣⱼ xᵢ xⱼ Bᵢⱼ` (Lewis-Randall quadratic mixing).
pub fn b_mix(components: &[Component], mole_fractions: &[f64], t: f64) -> Result<f64, VirialError> {
    if components.len() != mole_fractions.len() {
        return Err(VirialError::InvalidInput(format!(
            "components.len()={} but mole_fractions.len()={}",
            components.len(),
            mole_fractions.len()
        )));
    }
    let mat = b_mix_matrix(components, t);
    let n = components.len();
    let mut acc = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            acc += mole_fractions[i] * mole_fractions[j] * mat[i][j];
        }
    }
    Ok(acc)
}

/// Partial fugacity coefficient ln(φᵢ) for component i in a mixture.
///
/// `ln(φᵢ) = P/(R·T) · (2·Σⱼ xⱼ·Bᵢⱼ − B_mix)`.
///
/// Returns one ln(φᵢ) per component in the same index order as `components`.
pub fn ln_phi_mix_virial(
    components: &[Component],
    mole_fractions: &[f64],
    t: f64,
    p: f64,
) -> Result<Vec<f64>, VirialError> {
    let n = components.len();
    if n != mole_fractions.len() {
        return Err(VirialError::InvalidInput(format!(
            "components.len()={} but mole_fractions.len()={}",
            n,
            mole_fractions.len()
        )));
    }
    let mat = b_mix_matrix(components, t);
    let bmix = b_mix(components, mole_fractions, t)?;
    let factor = p / (1000.0 * R_GAS * t);
    let mut out = Vec::with_capacity(n);
    for row in &mat {
        let sum_j: f64 = row
            .iter()
            .zip(mole_fractions.iter())
            .map(|(b_ij, x_j)| b_ij * x_j)
            .sum();
        out.push(factor * (2.0 * sum_j - bmix));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_methane() -> Component {
        Component {
            name: "methane".into(),
            tc: 190.564,
            pc: 4599.0, // kPa
            omega: 0.0115,
            ..Component::default()
        }
    }

    #[test]
    fn pitzer_b0_b1_at_critical() {
        // At Tr = 1, B⁰ = 0.083 - 0.422 = -0.339; B¹ = 0.139 - 0.172 = -0.033.
        assert!((pitzer_b0(1.0) + 0.339).abs() < 1e-6);
        assert!((pitzer_b1(1.0) + 0.033).abs() < 1e-6);
    }

    #[test]
    fn virial_z_ideal_limit() {
        // At very low pressure, Z → 1 regardless of T.
        let c = dummy_methane();
        let z = z_factor_virial(&c, 300.0, 1.0); // 1 kPa
        assert!((z - 1.0).abs() < 1e-3);
    }
}
