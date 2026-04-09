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
