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
//! Michelsen-derived methodology; the legacy two-stage schemes are kept
//! only as test oracles. All Newton loops consume the Milestone 8.3
//! analytic/AD composition Jacobians (§L). See the per-file docs:
//!
//! - [`init`] — Wilson K-value initialization (§I)
//! - [`stability`] — tangent-plane-distance stability analysis (§I)
//! - [`isothermal`] — GDEM-accelerated SS → Newton, Rachford-Rice via
//!   Halley inside the Leibovici–Neoschil window (§J, §F)
//! - `bubble` / `dew` — log-variable Newton (§K) *(later in M9)*
//! - `critical` — Heidemann with analytic Helmholtz derivatives (§G)
//! - `kij_regression` / `aij_regression` — Brent / Levenberg-Marquardt
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

use thiserror::Error;

pub mod adiabatic;
pub mod aij_regression;
pub mod bubble;
pub mod critical;
pub mod dew;
pub mod envelope;
pub mod init;
pub mod isothermal;
pub mod kij_regression;
pub mod stability;

mod incipient;
mod system;

pub use system::{SystemSpec, k_values};

/// Errors from the flash / equilibrium layer.
#[derive(Debug, Error, PartialEq)]
pub enum FlashError {
    /// Input slice lengths disagree (components vs composition vs K vs kij).
    #[error("dimension mismatch: {0}")]
    Dimension(String),
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
