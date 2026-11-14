//! Flash, bubble/dew, critical-point, and parameter-regression algorithms
//! (Milestone 9, Phase 15 of MODERNIZATION_PLAN.md).
//!
//! This module is the top of the VLE calculation stack: it turns the
//! fugacity/energy building blocks from the EOS, activity, virial, and
//! mixture layers into the phase-equilibrium answers a user actually asks
//! for — "at this T and P, how does my mixture split into liquid and
//! vapor?" and its inverses (bubble/dew), plus the mixture critical point
//! and the kij/Aij parameter fits.
//!
//! ## Algorithm choices (Track A of PERFORMANCE_PROPOSAL.md)
//!
//! The thesis-era iteration schemes are replaced by the modern
//! Michelsen-derived methodology. (The legacy two-stage schemes were planned
//! as test oracles but never built; the bisection saturation solver is the
//! in-tree reference.) The Newton loops that exist consume the Milestone 8.3
//! analytic/AD composition Jacobians (§L). See the per-file docs:
//!
//! - [`init`] — Wilson K-value initialization (§I)
//! - [`stability`] — tangent-plane-distance stability analysis (§I)
//! - [`isothermal`] — GDEM-accelerated successive substitution, Rachford-Rice
//!   via Halley inside the Leibovici–Neoschil window (§J, §F), finished by
//!   Newton on the vapor mole numbers where SS is slow.
//! - [`bubble`] / [`dew`] — Newton on 1/T (temperature) or ln P (pressure)
//!   with one SS step on the incipient composition, then the simultaneous
//!   (N+1)-variable step near the root; fallbacks are bisection (temperature)
//!   and the multiplicative update (pressure) (§K).
//! - [`envelope`] — Michelsen phase-envelope continuation, exact Jacobian (§K)
//! - [`adiabatic`] — warm-started nested PH flash, bisection outer loop (§M)
//! - [`critical`] — Heidemann with dual-number Helmholtz derivatives (§G)
//! - [`kij_regression`] / [`aij_regression`] — Brent / Levenberg-Marquardt
//!   (the Aij Jacobian is a finite difference over the bubble-pressure solve)
//! - [`free_water`] — the water-decant flash (M20)
//!
//! ## The two thermodynamic paths
//!
//! K-values `Kᵢ = yᵢ/xᵢ` come from equal component fugacities across the
//! phases, and the engine supports both classical routes (see [`SystemSpec`]
//! and [`k_values`]):
//!
//! - **φ-φ** — an EOS models *both* phases; `Kᵢ = φ̂ᵢᴸ/φ̂ᵢⱽ` from the liquid
//!   and vapor mixture fugacity coefficients (Chapter IV cases 1, 2, 7).
//! - **γ-φ** — an activity model for the liquid, an EOS/virial/ideal vapor;
//!   `Kᵢ = γᵢ·Psat,ᵢ·φᵢˢᵃᵗ / (φ̂ᵢⱽ·P)` (modified Raoult; Chapter IV cases
//!   3–6). With an ideal vapor this collapses to `Kᵢ = γᵢ·Psat,ᵢ/P`.
//! - **Refinery methods (M20)** — `LiquidModel::GraysonStreed`
//!   (`Kᵢ = νᵢγᵢ/φ̂ᵢⱽ`) and `LiquidModel::BraunK10` (`Kᵢ = Pᵢᴹᴮ/(φ̂ᵢⱽP)`),
//!   plus [`free_water::flash_free_water`] for steam-stripped feeds.

use thiserror::Error;

pub mod adiabatic;
pub mod aij_regression;
pub mod bubble;
pub mod critical;
pub mod dew;
pub mod envelope;
pub mod free_water;
pub mod init;
pub mod isothermal;
pub mod kij_regression;
pub mod stability;

mod incipient;
mod system;

pub use system::{
    KValueDerivs, StageThermo, SystemSpec, k_values, k_values_with_derivs, k_values_with_t_derivs,
    ln_k_values_into, phase_cp, phase_enthalpy_entropy, stage_thermo_into,
};

/// Errors from the flash / equilibrium layer.
#[derive(Debug, Error, PartialEq)]
pub enum FlashError {
    /// Input slice lengths disagree (components vs composition vs K vs kij).
    #[error("dimension mismatch: {0}")]
    Dimension(String),
    /// An argument is shaped correctly but numerically invalid — a non-finite
    /// or negative mole fraction, a non-positive or non-finite Kᵢ, an empty
    /// mixture, or a non-positive tolerance. Caught at the API boundary so a
    /// bad input surfaces as a clear error instead of a silent NaN that only
    /// shows up as a non-convergence hundreds of iterations later.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// An iteration hit its cap without meeting the tolerance.
    #[error("{what} did not converge in {iters} iterations (residual {residual:.3e})")]
    NoConvergence {
        what: &'static str,
        iters: usize,
        residual: f64,
    },
    /// The Rachford-Rice window is degenerate — every Kᵢ = 1 (no driving
    /// force) or all Kᵢ on one side of 1 with no bracketable root.
    #[error("Rachford-Rice has no bracketable root (Kmax={kmax:.4}, Kmin={kmin:.4})")]
    NoRachfordRiceRoot { kmax: f64, kmin: f64 },
    /// A downstream thermodynamic evaluation failed (EOS root, fugacity).
    #[error("thermodynamic evaluation failed: {0}")]
    Thermo(String),
    /// The requested model combination is not supported by this path.
    #[error("unsupported: {0}")]
    Unsupported(String),
}

/// How far a composition's sum may be from 1 before a flash or saturation
/// solve rejects it. Wide enough for a composition that was normalized in
/// floating point or typed to six decimals; far too narrow for one that was
/// never normalized.
pub const COMPOSITION_SUM_TOL: f64 = 1e-6;

/// Reject a composition a flash or saturation solve cannot mean anything
/// for: a non-finite or negative mole fraction, or a sum further than
/// [`COMPOSITION_SUM_TOL`] from 1. `name` is the argument's name in the
/// error message (`"z"`, `"x"`, `"y"`). A zero mole fraction is allowed.
///
/// Until v0.20.0 only non-finite and negative entries were checked, and only
/// by the isothermal flash: a feed summing to 1.2 came back as a two-phase
/// result whose phases also summed to 1.2, and an unnormalized bubble-point
/// composition failed 200 iterations later with an unrelated error.
///
/// # Errors
/// [`FlashError::InvalidInput`] naming the offending entry or the sum.
pub(crate) fn check_composition(name: &str, w: &[f64]) -> Result<(), FlashError> {
    if w.is_empty() {
        return Err(FlashError::InvalidInput("empty mixture".into()));
    }
    if let Some(i) = w.iter().position(|&wi| !wi.is_finite() || wi < 0.0) {
        return Err(FlashError::InvalidInput(format!("{name}[{i}]={}", w[i])));
    }
    let sum: f64 = w.iter().sum();
    if (sum - 1.0).abs() > COMPOSITION_SUM_TOL {
        return Err(FlashError::InvalidInput(format!(
            "{name} sums to {sum}, not 1 (tolerance {COMPOSITION_SUM_TOL:e}); normalize it first"
        )));
    }
    Ok(())
}

/// Reject a temperature (**K**) or absolute pressure (**kPa**) that is not
/// finite and strictly positive. `what` is the quantity with its unit, e.g.
/// `"T (K)"`. A negative absolute temperature or pressure used to reach the
/// solvers and come back as a confident single-phase answer.
///
/// # Errors
/// [`FlashError::InvalidInput`] naming the quantity and its value.
pub(crate) fn check_positive(what: &str, value: f64) -> Result<(), FlashError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(FlashError::InvalidInput(format!(
            "{what} = {value}; it must be finite and > 0"
        )))
    }
}
