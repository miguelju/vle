//! Petroleum characterization — turning a crude assay into pseudocomponents.
//!
//! *Milestone 19 / Phase 26. Design record:
//! `docs/plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md` §2 (U1, U2).*
//!
//! # The problem this module solves
//!
//! Every other module in this crate assumes you can hand it a [`Component`] with
//! a critical temperature, a critical pressure and an acentric factor. That is a
//! fine assumption for methane or benzene — somebody measured those. It is a
//! useless assumption for **crude oil**, which is a mixture of many thousands of
//! individual molecules that nobody has ever separated, let alone measured.
//!
//! [`Component`]: crate::types::Component
//!
//! What a refinery *does* have is an **assay**: a distillation curve (how much
//! of the barrel boils away by what temperature) and a bulk density. The trick
//! the industry settled on decades ago is to slice that curve into a few hundred
//! narrow boiling cuts, pretend each slice is a single compound — a
//! **pseudocomponent** — and estimate its critical properties from just two
//! numbers: the slice's mid-boiling point `Tb` and its specific gravity `SG`.
//! That estimate is what this module does.
//!
//! ```text
//!   assay                cuts                  pseudocomponents
//!   ┌──────────┐        ┌──────────┐          ┌──────────────────┐
//!   │ D86 / SD │  ───▶  │ Tb₁, SG₁ │   ───▶   │ Tc, Pc, ω, M, Vc │
//!   │  curve   │  conv  │ Tb₂, SG₂ │  corre-  │   per cut …      │
//!   │ + gravity│  + cut │   …      │  lations │                  │
//!   └──────────┘        └──────────┘          └──────────────────┘
//!    distillation        cuts.rs               properties.rs
//!    .rs                                       cp.rs
//! ```
//!
//! # Module map
//!
//! | module | what it does |
//! |---|---|
//! | [`gravity`] | API gravity ↔ specific gravity, Watson K, average boiling points |
//! | [`distillation`] | interconvert D86 ↔ TBP ↔ D2887 (SimDist) ↔ EFV curves |
//! | [`cuts`] | slice a TBP curve into N narrow-boiling cuts |
//! | [`properties`] | Tb + SG → M, Tc, Pc, ω, Vc, Zc (four correlation families) |
//! | [`cp`] | ideal-gas Cp° for a fraction, as [`Component::cp_coeffs`] |
//! | [`vapor_pressure`] | Maxwell-Bonnell pressure ↔ boiling-point correction |
//! | [`assay`] | the whole pipeline: assay → `Vec<Component>` |
//!
//! [`Component::cp_coeffs`]: crate::types::Component::cp_coeffs
//!
//! # A note on units, because this module is where they bite
//!
//! Nearly every correlation here was published in **°R and psia**, and several
//! were published in **°F**. The crate's canonical units are **K and kPa**
//! (see the crate-level docs). Every public function in this module takes and
//! returns canonical units; the imperial conversion happens *inside*, next to
//! the correlation, with the published form written out in the units it was
//! published in so it can be checked against the paper. Do not "simplify" a
//! correlation by pre-converting its coefficients — that is exactly how a
//! transcription error becomes invisible.
//!
//! # Accuracy, honestly
//!
//! These are **correlations fit to light and medium petroleum**, and they are
//! only as good as that. Every correlation in [`properties`] was checked in this
//! repo against the measured Tc/Pc/ω/M of ten pure hydrocarbons from the bundled
//! component database (n-C5 through n-C10, benzene, toluene, cyclohexane,
//! methylcyclohexane) — see the tests in that module for the measured deviations
//! and which correlation wins where. In brief: Tc is good to ~1 %, Pc to ~5 %,
//! M to ~8 %, and everything degrades on aromatics and heavy residue.
//!
//! # Further reading
//!
//! [`docs/en/petroleum/README.md`](https://github.com/miguelju/vle/blob/main/docs/en/petroleum/README.md)
//! is the learning guide for this module: the domain from first principles,
//! every correlation written out in its published units, the validation
//! provenance, and the design decisions behind the API.
//!
//! # References
//!
//! - (31) Riazi, M. R. *Characterization and Properties of Petroleum Fractions*;
//!   ASTM Manual Series MNL50: West Conshohocken, PA, **2005**. — the standard
//!   text; the worked examples used as test oracles here are its Examples 3.2–3.6.
//! - (32) Riazi, M. R.; Daubert, T. E. Simplify Property Predictions.
//!   *Hydrocarbon Process.* **1980**, *59* (3), 115–116.
//! - (33) Riazi, M. R.; Daubert, T. E. Characterization Parameters for Petroleum
//!   Fractions. *Ind. Eng. Chem. Res.* **1987**, *26* (4), 755–759.
//! - (34) Riazi, M. R.; Daubert, T. E. Analytical Correlations Interconvert
//!   Distillation Curve Types. *Oil Gas J.* **1986**, *84*, 50–57.
//! - (35) Daubert, T. E. Petroleum Fraction Distillation Interconversion.
//!   *Hydrocarbon Process.* **1994**, *73* (9), 75–78. — API Procedures 3A1.1,
//!   3A3.1, 3A3.2.
//! - (36) Kesler, M. G.; Lee, B. I. Improve Prediction of Enthalpy of Fractions.
//!   *Hydrocarbon Process.* **1976**, *55* (3), 153–158.
//! - (37) Lee, B. I.; Kesler, M. G. A Generalized Thermodynamic Correlation Based
//!   on Three-Parameter Corresponding States. *AIChE J.* **1975**, *21* (3), 510–527.
//! - (38) Twu, C. H. An Internally Consistent Correlation for Predicting the
//!   Critical Properties and Molecular Weights of Petroleum and Coal-Tar Liquids.
//!   *Fluid Phase Equilib.* **1984**, *16*, 137–150.
//! - (39) Edmister, W. C.; Okamoto, K. K. Applied Hydrocarbon Thermodynamics,
//!   Part 13. *Pet. Refiner* **1959**, *38* (9), 271–288. — EFV conversions.
//! - (40) Maxwell, J. B.; Bonnell, L. S. *Vapor Pressure Charts for Petroleum
//!   Engineers*; Esso Research: Linden, NJ, **1955/1957**. — API Procedure 5A1.19.
//! - (41) API. *Technical Data Book — Petroleum Refining*, 6th ed.

pub mod assay;
pub mod cp;
pub mod cuts;
pub mod distillation;
pub mod gravity;
pub mod properties;
pub mod vapor_pressure;

pub use assay::{Assay, GravitySpec, Pseudocomponent};
pub use cp::{ideal_gas_cp_coeffs, ideal_gas_cp_mass, ideal_gas_cp_molar};
pub use cuts::{Cut, CutSpec, cut_curve};
pub use distillation::{DistillationBasis, DistillationCurve, convert_curve};
pub use gravity::{
    AverageBoilingPoint, api_from_sg, average_boiling_points, sg_from_api, watson_k,
};
pub use properties::{PropertyMethod, PseudoProperties, ZcMethod, acentric_lee_kesler, estimate};
pub use vapor_pressure::{boiling_point_at_pressure, normal_boiling_point, vapor_pressure};
/// Errors raised by the petroleum-characterization routines.
///
/// These are all *input* problems — a curve that is not sorted, a gravity that
/// is not physical, a cut count of zero. The correlations themselves are closed
/// algebraic forms and cannot fail once their inputs are sane.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PetroleumError {
    /// A distillation curve had fewer than two points, or its two axes had
    /// different lengths.
    #[error("invalid distillation curve: {0}")]
    Curve(String),

    /// Volume fractions were not strictly increasing, or fell outside [0, 1].
    #[error("invalid cut points: {0}")]
    CutPoints(String),

    /// A specific gravity, API gravity, temperature or pressure was outside the
    /// physically meaningful range.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// An iterative solve (Twu's molecular-weight inversion, the Maxwell-Bonnell
    /// vapor-pressure inversion) did not converge.
    #[error("no convergence: {0}")]
    NoConvergence(String),
}

/// Whether an error describes bad *input* rather than a failed solve.
///
/// The PyO3 layer uses this to pick between `ValueError` and `RuntimeError`,
/// matching how the rest of the bindings map errors. Kept here rather than in
/// the binding module so the classification lives next to the variants it
/// classifies and cannot drift from them.
pub fn petroleum_error_is_input(e: &PetroleumError) -> bool {
    matches!(
        e,
        PetroleumError::Curve(_) | PetroleumError::CutPoints(_) | PetroleumError::InvalidInput(_)
    )
}
