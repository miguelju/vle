//! Refinery thermodynamics — the methods a crude column is validated against.
//!
//! *Milestone 20 / Phase 27. Design record:
//! `docs/plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md` §2 (U4, U5).*
//!
//! # Why a cubic EOS is not the end of the story
//!
//! Everything else in this crate models a hydrocarbon mixture with a cubic
//! equation of state, and for light and medium hydrocarbons that is excellent.
//! A refinery column, though, is validated against a different, older set of
//! methods — because those are what the plant data were fitted with, because
//! heavy cuts stress a cubic EOS where it is weakest (liquid density, enthalpy
//! far from the critical point), and because refinery practice standardised on
//! them decades before cubic EOS were trusted for liquids. A simulator that
//! cannot reproduce a Grayson–Streed / Lee–Kesler case cannot be checked against
//! the literature. So this module adds, alongside the EOS route:
//!
//! | need | method | where |
//! |---|---|---|
//! | K-values for hydrogen-rich, heavy service | **Grayson–Streed** `Kᵢ = νᵢγᵢ/φ̂ᵢⱽ` | [`crate::flash::system`] via `LiquidModel::GraysonStreed`; νᵢ in [`crate::eos::regular_solution_ln_nu`] |
//! | K-values for heavy fractions at low pressure | **Braun K10** from Maxwell–Bonnell | `LiquidModel::BraunK10`; the closed-form inversion in [`crate::petroleum::vapor_pressure`] |
//! | enthalpy / entropy departure | **Lee–Kesler** three-parameter corresponding states | [`lee_kesler`] |
//! | heavy-cut liquid density | **Peneloux** volume translation of the cubic EOS | [`volume_translation`] |
//! | stripping steam → a second liquid | **free-water decant flash** | [`crate::flash::free_water`] |
//!
//! # Performance framing
//!
//! Every function here is written for the *outer loop* of an inside-out column
//! solver: it is called once per stage per outer iteration with hundreds of
//! pseudocomponents, and must cost O(N) (K-values, Peneloux) or O(N²) once
//! (Lee–Kesler mixing rules) with no allocation inside any loop. The
//! Grayson–Streed νᵢ and Braun K10 constants are hoisted into the flash's
//! per-`(T, P)` cache so an iteration pays only for the vapor φ̂ and the
//! regular-solution γ.
//!
//! # References
//! - (42) Grayson, H. G.; Streed, C. W. Vapor-Liquid Equilibria for High
//!   Temperature, High Pressure Hydrogen-Hydrocarbon Systems. *6th World
//!   Petroleum Congress*, Frankfurt, **1963**, Sect. VII, Paper 20, 233–245.
//! - (43) Chao, K. C.; Seader, J. D. A General Correlation of Vapor-Liquid
//!   Equilibria in Hydrocarbon Mixtures. *AIChE J.* **1961**, *7* (4), 598–605.
//! - (37) Lee, B. I.; Kesler, M. G. A Generalized Thermodynamic Correlation Based
//!   on Three-Parameter Corresponding States. *AIChE J.* **1975**, *21* (3), 510–527.
//! - (44) Péneloux, A.; Rauzy, E.; Fréze, R. A Consistent Correction for
//!   Redlich-Kwong-Soave Volumes. *Fluid Phase Equilib.* **1982**, *8*, 7–23.
//! - (45) Plöcker, U.; Knapp, H.; Prausnitz, J. M. Calculation of High-Pressure
//!   Vapor-Liquid Equilibria from a Corresponding-States Correlation with
//!   Emphasis on Asymmetric Mixtures. *Ind. Eng. Chem. Process Des. Dev.*
//!   **1978**, *17* (3), 324–332.
//! - (40) Maxwell, J. B.; Bonnell, L. S. *Vapor Pressure Charts for Petroleum
//!   Engineers*; Esso Research: Linden, NJ, **1955**. — API Procedure 5A1.19
//!   (the basis of the Braun K10 charts).

pub mod lee_kesler;
pub mod volume_translation;

pub use lee_kesler::{
    LkDeparture, LkPseudoCritical, lee_kesler_departure, lee_kesler_departure_mix,
    lee_kesler_pseudocritical, lee_kesler_reduced,
};
pub use volume_translation::{
    peneloux_shift, peneloux_shift_mix, translated_liquid_density, translated_molar_volume,
};

/// Errors raised by the refinery-thermodynamics routines.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RefineryError {
    /// A temperature, pressure, composition or property was outside the
    /// physically meaningful range, or slices disagreed in length.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// The Lee–Kesler reduced-volume solve did not converge.
    #[error("no convergence: {0}")]
    NoConvergence(String),
    /// The requested combination (e.g. a Peneloux shift for a three-parameter
    /// EOS) is not defined.
    #[error("unsupported: {0}")]
    Unsupported(String),
}

/// Whether an error describes bad *input* rather than a failed solve — the
/// PyO3 layer maps this to `ValueError` vs `RuntimeError`.
pub fn refinery_error_is_input(e: &RefineryError) -> bool {
    matches!(
        e,
        RefineryError::InvalidInput(_) | RefineryError::Unsupported(_)
    )
}
