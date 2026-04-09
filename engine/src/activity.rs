//! Activity coefficient models for liquid-phase non-ideality.
//!
//! Activity coefficients (γᵢ) quantify how much a component's behavior in a
//! liquid mixture deviates from ideal solution (Raoult's law). A value of γ = 1
//! means ideal behavior; γ > 1 indicates positive deviation (components "dislike"
//! each other); γ < 1 indicates negative deviation (favorable interactions).
//!
//! These models are used in the γ-φ approach to VLE, where the liquid phase is
//! described by activity coefficients and the vapor phase by an EOS. This is
//! preferred over the φ-φ (EOS-only) approach for highly non-ideal liquid
//! mixtures such as water + alcohol, where cubic EOS gives poor liquid predictions.
//!
//! All 5 models are identical in both legacy codebases (VB6 `clsActivityMulticomp.cls`
//! and Pascal `TERMOIII.PAS`). Each model requires binary interaction parameters
//! (Aij) fit to experimental VLE data.

/// Activity coefficient model for liquid-phase non-ideality.
///
/// Each model computes ln(γᵢ) from composition and binary parameters (Aij),
/// and provides analytical excess enthalpy HE (via dGE/dT) for enthalpy
/// departure calculations in adiabatic flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ActivityModel {
    /// Ideal solution (γᵢ = 1 for all components at all compositions).
    /// No binary parameters needed. Used as a baseline or for nearly ideal mixtures.
    IdealSolution = 25,
    /// van Laar model. Two-suffix equation derived from van der Waals EOS.
    /// Parameters: A₁₂, A₂₁ (asymmetric). Good for simple non-polar mixtures.
    /// Cannot predict liquid-liquid immiscibility.
    VanLaar = 21,
    /// Wilson model. Local composition equation using molar volume ratios.
    /// Parameters: Λ₁₂, Λ₂₁ (related to energy differences). Good for
    /// polar/non-polar mixtures. Cannot predict liquid-liquid immiscibility.
    Wilson = 22,
    /// Scatchard-Hildebrand (regular solution theory). Based on solubility
    /// parameters (δᵢ) and liquid molar volumes (Vᵢᴸ). Semi-predictive —
    /// requires only pure-component data, no binary fitting needed.
    /// Limited to non-polar mixtures.
    ScatchardHildebrand = 23,
    /// Margules model. Two-suffix equation — the simplest empirical activity
    /// coefficient model. Parameters: A₁₂, A₂₁ (asymmetric). Useful for
    /// quick estimates but limited accuracy for strongly non-ideal systems.
    Margules = 24,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminant_values_match_legacy() {
        assert_eq!(ActivityModel::VanLaar as i32, 21);
        assert_eq!(ActivityModel::Wilson as i32, 22);
        assert_eq!(ActivityModel::ScatchardHildebrand as i32, 23);
        assert_eq!(ActivityModel::Margules as i32, 24);
        assert_eq!(ActivityModel::IdealSolution as i32, 25);
    }
}
