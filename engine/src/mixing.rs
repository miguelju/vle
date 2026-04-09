//! Mixing rules for cubic equations of state.
//!
//! When applying a pure-component EOS to a mixture, you need rules to combine
//! individual a_i, b_i parameters into mixture-average values a_mix, b_mix.
//! The choice of mixing rule significantly affects prediction quality:
//!
//! - **Classical rules** (IVDW, IIVDW, Classical): Quadratic in composition,
//!   use binary interaction parameters kij. Simple but limited for non-ideal
//!   liquid mixtures.
//!
//! - **Excess-Gibbs-energy rules** (Wong-Sandler, Huron-Vidal, MHV1/2): Incorporate
//!   activity coefficient model information into the EOS mixing rule. This gives the
//!   EOS the accuracy of activity models for liquids while retaining correct behavior
//!   at high pressure. The key innovation of Wong-Sandler (Ref 21) is satisfying the
//!   quadratic composition dependence of the second virial coefficient.
//!
//! - **C-parameter rules** (PatelTejaC, PatelTejaUSBC, SchmidtWenzelC): Specific to
//!   three-parameter EOS from the Pascal codebase. These define how the third
//!   constant c is mixed — either by simple mole-fraction or weighted averaging.
//!
//! # References
//! - (21) Orbey & Sandler — Wong-Sandler mixing rules
//! - (4) Da Silva & Báez (1989) — C-parameter mixing for 3-param EOS

/// Mixing rule for combining pure-component EOS parameters into
/// mixture parameters (a_mix, b_mix, and optionally c_mix for 3-parameter EOS).
///
/// The first 8 variants (WongSandler through Classical) apply to the a and b
/// parameters of any two-parameter cubic EOS. The last 3 variants handle the
/// additional c parameter needed by Schmidt-Wenzel and Patel-Teja EOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum MixingRule {
    // --- From VB6 TADiPMR enum (a, b mixing for 2-param EOS) ---
    /// Wong-Sandler (1992). GE-based rule that satisfies the second virial
    /// coefficient boundary condition. Requires an activity model + kij. Ref (21).
    WongSandler = 26,
    /// Huron-Vidal original. GE-based rule at infinite pressure limit.
    HuronVidalOriginal = 27,
    /// Huron-Vidal simplified.
    HuronVidalSimplified = 28,
    /// Modified Huron-Vidal first order (MHV1). Zero-pressure reference.
    MHV1 = 29,
    /// Modified Huron-Vidal second order (MHV2). Improved zero-pressure reference.
    MHV2 = 30,
    /// Improved van der Waals one-fluid. a_mix = ΣΣ xᵢxⱼ(1-kij)√(aᵢaⱼ),
    /// b_mix = ΣΣ xᵢxⱼ(bᵢ+bⱼ)/2.
    IVDW = 31,
    /// Improved van der Waals two-fluid. Asymmetric variant of IVDW.
    IIVDW = 32,
    /// Classical one-fluid (quadratic). a_mix = ΣΣ xᵢxⱼaij, b_mix = Σ xᵢbᵢ.
    Classical = 33,

    // --- C-parameter mixing (from Pascal, Ref (4): Da Silva & Báez, 1989) ---
    /// Patel-Teja C-parameter: simple mole-fraction weighted average.
    /// c_mix = Σ xᵢ·cᵢ. Used with [`CubicEos::PatelTeja`].
    /// legacy/pascal/TERMOII.PAS.
    PatelTejaC = 34,
    /// Patel-Teja USB C-parameter: √B-weighted average.
    /// c_mix = Σ(xᵢ√Bᵢ·cᵢ) / Σ(xᵢ√Bᵢ). Used with [`CubicEos::PatelTejaUSB`].
    /// legacy/pascal/TERMOII.PAS.
    PatelTejaUSBC = 35,
    /// Schmidt-Wenzel C-parameter: √A-weighted average using acentric factor ω.
    /// c_mix = Σ(xᵢ√Aᵢ·ωᵢ) / Σ(xᵢ√Aᵢ). Used with [`CubicEos::SchmidtWenzel`].
    /// legacy/pascal/TERMOII.PAS.
    SchmidtWenzelC = 36,
}

#[cfg(test)]
mod tests {
    // `super` refers to the parent module (the code above this `mod tests`
    // block). The `*` means "import everything public from it". Together,
    // `use super::*` brings all the types defined in this file (like
    // MixingRule) into scope so the tests can use them directly.
    use super::*;

    // This test verifies that the integer codes for mixing rules match
    // the values used in the legacy VB6 program. In VB6, mixing rules
    // were numbered starting at 26 (WongSandler) through 33 (Classical).
    // If someone accidentally changes the enum numbering, this test will
    // fail, preventing a silent mismatch with legacy data or config files.
    #[test]
    fn vb6_mixing_rules_start_at_26() {
        assert_eq!(MixingRule::WongSandler as i32, 26);
        assert_eq!(MixingRule::Classical as i32, 33);
    }
}
