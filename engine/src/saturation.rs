//! Saturation pressure models.
//!
//! Saturation (vapor) pressure Psat(T) is the pressure at which a pure component
//! boils at temperature T. These correlations are essential for:
//!
//! 1. **Initial K-value estimates** in flash calculations — Ki ≈ Psat_i(T) / P_system
//!    gives a starting point for iterative convergence.
//! 2. **Bubble/dew point initialization** — correlating Psat across components
//!    gives initial temperature or pressure guesses.
//! 3. **Validation** — comparing EOS-predicted saturation with correlation values
//!    confirms the EOS parameterization is correct.
//!
//! The models range from simple empirical (Antoine, 3 parameters) to thermodynamically
//! consistent (Maxwell equal-area, derived directly from the EOS).
//!
//! # References
//! - (4) Da Silva & Báez (1989) — Antoine correlation

/// Saturation (vapor) pressure correlation model.
///
/// Used to estimate pure-component saturation pressure Psat(T) in **kPa**
/// from temperature in **K**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum SatPressureModel {
    /// Antoine equation: ln(P/Pc) = a₁ - a₂/(a₃ + T).
    /// Simple 3-parameter correlation, widely tabulated. Accurate over limited
    /// temperature ranges. Ref (4), legacy/pascal/TERMOI.PAS.
    Antoine = 0,
    /// Riedel correlation. Extended corresponding-states method using Tc, Pc, ω.
    /// Better extrapolation than Antoine over wider temperature ranges.
    Riedel = 1,
    /// Müller correlation. Alternative reduced-property correlation.
    Muller = 2,
    /// Reduced-pressure model (RPM). Corresponding-states correlation using
    /// reduced properties (Tr, Pr).
    RPM = 3,
    /// Database polynomial. Coefficients from external property database (e.g., DIPPR).
    /// P = exp(A + B/T + C·ln(T) + D·T^E) or similar fitted form.
    Polynomial = 4,
    /// Maxwell equal-area construction. Thermodynamically exact — finds the pressure
    /// where the integral of (V dP) over the van der Waals loop equals zero.
    /// Requires a cubic EOS and iterative solution. Slowest but most consistent.
    Maxwell = 5,
}

// ===========================================================================
// M7.5 — Antoine saturation pressure (only).
//
// The four remaining models (Riedel, Müller, RPM, polynomial, Maxwell) are
// M7.4 deferred. Their dispatch branches in `psat` panic with
// `unimplemented!` and a pointer to the legacy source.
// ===========================================================================

use crate::types::Component;
use thiserror::Error;

/// Errors raised by the saturation-pressure layer.
#[derive(Debug, Error, PartialEq)]
pub enum SatError {
    /// The component is missing the `psat_coeffs` vector or it has the
    /// wrong length for the selected model (Antoine requires exactly 3).
    #[error("component {name:?}: expected {expected} Antoine coefficients, got {got}")]
    BadCoefficients {
        name: String,
        expected: usize,
        got: usize,
    },
    /// The model has not been ported yet — covered by M7.4.
    #[error("saturation model {0:?} not yet ported (M7.4 deferred)")]
    NotImplemented(SatPressureModel),
    /// Temperature is outside a physically meaningful range (≤ 0 K, or
    /// the Antoine denominator a3 + T is non-positive).
    #[error("temperature {0} K out of range for saturation correlation")]
    OutOfRange(f64),
}

/// Antoine vapor pressure: `ln(P_sat/Pc) = a1 − a2/(a3 + T)`.
///
/// # Arguments
/// * `comp` — Component (uses `pc` and `psat_coeffs`).
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Saturation pressure in **kPa absolute**.
///
/// # Errors
/// `BadCoefficients` if `psat_coeffs` does not have exactly 3 entries.
/// `OutOfRange` if `a3 + T ≤ 0` (would produce inf/NaN in the exp).
///
/// # Source
/// Reference (4): Da Silva & Báez (1989), `legacy/pascal/TERMOI.PAS`. The
/// form `ln(P/Pc) = a1 − a2/(a3 + T)` is the "reduced" Antoine the
/// Pascal program uses — coefficients are tabulated against the
/// component's own Pc, not against 1 atm. Some external Antoine tables
/// use `log10(P) = A − B/(C + T)` with a different sign on `C`; convert
/// before calling this function.
pub fn psat_antoine(comp: &Component, t: f64) -> Result<f64, SatError> {
    if comp.psat_coeffs.len() != 3 {
        return Err(SatError::BadCoefficients {
            name: comp.name.clone(),
            expected: 3,
            got: comp.psat_coeffs.len(),
        });
    }
    let a1 = comp.psat_coeffs[0];
    let a2 = comp.psat_coeffs[1];
    let a3 = comp.psat_coeffs[2];
    let denom = a3 + t;
    if denom <= 0.0 {
        return Err(SatError::OutOfRange(t));
    }
    Ok(comp.pc * (a1 - a2 / denom).exp())
}

/// Analytical derivative `dPsat/dT` for the Antoine form.
///
/// `Psat = Pc · exp(a1 − a2/(a3 + T))`
/// `dPsat/dT = Psat · a2 / (a3 + T)²`
///
/// Returns `dPsat/dT` in **kPa/K**.
pub fn d_psat_dt_antoine(comp: &Component, t: f64) -> Result<f64, SatError> {
    let psat = psat_antoine(comp, t)?;
    let a2 = comp.psat_coeffs[1];
    let a3 = comp.psat_coeffs[2];
    let denom = a3 + t;
    Ok(psat * a2 / (denom * denom))
}

/// Generic dispatch: compute saturation pressure for any model.
///
/// Only the [`SatPressureModel::Antoine`] branch is implemented in M7.1.
/// All others panic via [`SatError::NotImplemented`] so callers get a
/// runtime error instead of an undetected zero — covers M7.4.
pub fn psat(model: SatPressureModel, comp: &Component, t: f64) -> Result<f64, SatError> {
    match model {
        SatPressureModel::Antoine => psat_antoine(comp, t),
        other => Err(SatError::NotImplemented(other)),
    }
}
