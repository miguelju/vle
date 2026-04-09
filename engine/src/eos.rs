//! Cubic equation of state variants.
//!
//! A cubic EOS expresses pressure as a function of temperature and molar volume
//! using a cubic polynomial in V. The general two-parameter form (Ref (5), Abbott)
//! is: P = RT/(V-b) - a·α(T)/((V+k₁b)(V+k₂b)), where k₁, k₂ are family constants
//! and α(T) is a temperature-dependent function specific to each variant.
//!
//! This module merges all 19 VB6 alpha-function variants (TADiPEDC enum) with
//! 3 additional Pascal models: Schmidt-Wenzel, Patel-Teja, and Patel-Teja USB.
//! The Pascal models are three-parameter EOS that add a third constant (c) to
//! better represent polar and heavy molecules — they require special C-parameter
//! mixing rules defined in the `mixing` module.
//!
//! Numeric discriminants preserve the legacy integer codes used in both VB6
//! `Select Case` and Pascal `case` dispatch. The 3-parameter models are assigned
//! codes 19–21 (continuing from VB6's 0–18 range) since the Pascal codes (6–8)
//! would collide with existing VB6 variants.
//!
//! # References
//! - (5) Abbott — general cubic EOS form with family constants
//! - (4) Da Silva & Báez (1989) — Schmidt-Wenzel, Patel-Teja, Chao-Seader

/// Cubic equation of state model selection.
///
/// Each variant corresponds to a specific α(Tr) function and EOS
/// parameterization. Two-parameter EOS (variants 0–18) use the Abbott
/// generalized form with k₁, k₂ family constants. Three-parameter EOS
/// (variants 19–21) add a c parameter for improved representation of
/// polar/asymmetric molecules.
// Automatically implement common traits for this enum:
//   Debug  — allows printing with {:?} for debugging
//   Clone  — allows creating a deep copy via .clone()
//   Copy   — allows implicit bitwise copies (no need to call .clone())
//   PartialEq, Eq — allows comparing values with == and !=
//   Hash   — allows using this enum as a key in HashMap/HashSet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// Store this enum in memory as a 32-bit integer (i32), matching the
// integer variant codes used by the legacy VB6 code (0..21). This also
// makes it safe to pass across the FFI boundary to C/Python.
#[repr(i32)]
pub enum CubicEos {
    // --- Two-parameter cubic EOS (from VB6 TADiPEDC) ---
    /// Peng-Robinson (1976). The most widely used cubic EOS for hydrocarbon systems.
    /// α(Tr) = [1 + κ(1 - √Tr)]² where κ = f(ω). Ref (5).
    PR1976 = 0,
    /// Redlich-Kwong (1949). Original temperature-dependent cubic EOS.
    /// α(Tr) = 1/√Tr.
    RK1949 = 1,
    /// Soave-Redlich-Kwong (1972). Improved RK with acentric factor correlation.
    /// α(Tr) = [1 + m(1 - √Tr)]² where m = f(ω).
    RKS1972 = 2,
    /// van der Waals (1870). The first cubic EOS. α(Tr) = 1 (constant).
    VdW1870 = 3,
    /// Peng-Robinson-Lim (1997). Modified PR for improved vapor pressure prediction.
    PRL1997 = 4,
    /// RKS-Lim (1997). Modified RKS for improved vapor pressure prediction.
    RKSL1997 = 5,
    /// RKS-Graboski-Daubert (1978). Modified m(ω) correlation for RKS.
    RKSGD1978 = 6,
    /// Redlich-Prausnitz (1978).
    RP1978 = 7,
    /// Berthelot (1899). Temperature-modified van der Waals.
    Berth1899 = 8,
    /// van der Waals-Adachi (1984). Improved α function for van der Waals family.
    VdWAda1984 = 9,
    /// van der Waals-Valderrama (1989). Modified VdW for polar compounds.
    VdWVald1989 = 10,
    /// RKS-Mathias-Naumann (1980). Polar extension with m, n parameters.
    RKSmn1980 = 11,
    /// RKS-Adachi-Tagawa-Mathias-Naumann (1995). Three-constant α function.
    RKSATmn1995 = 12,
    /// PR-Adachi-Tagawa-Mathias-Naumann-Gasem (1997). Three-constant α for PR family.
    PRATmng1997 = 13,
    /// PR-Mathias-Massih-Naumann (1989). Polar extension with m, n parameters.
    PRMmn1989 = 14,
    /// Peng-Robinson-Stryjek-Vera (1986). Adds component-specific K₁ parameter.
    PRSV1986 = 15,
    /// van der Waals-OL (1998).
    VdWOL1998 = 16,
    /// Redlich-Kwong-OL (1998).
    RKOL1998 = 17,
    /// Peng-Robinson-OL (1998).
    PROL1998 = 18,

    // --- Three-parameter cubic EOS (from Pascal, Ref (4): Da Silva & Báez, 1989) ---
    /// Schmidt-Wenzel (3-parameter). Uses acentric factor ω as the third parameter
    /// to set the c constant. Beta parameter: 0.25989 - 0.02143ω + 0.00337ω².
    /// Ref (4), legacy/pascal/TERMOII.PAS.
    SchmidtWenzel = 19,
    /// Patel-Teja (3-parameter). Uses a fitted Zc correlation as the third parameter.
    /// C-parameter mixing: simple mole-fraction average c_mix = Σ xᵢcᵢ.
    /// Ref (4), legacy/pascal/TERMOII.PAS.
    PatelTeja = 20,
    /// Patel-Teja with Universal Saturation Behavior. Same EOS form as PatelTeja
    /// but uses √B-weighted C-parameter mixing: c_mix = Σ(xᵢ√Bᵢ·cᵢ)/Σ(xᵢ√Bᵢ).
    /// Ref (4), legacy/pascal/TERMOII.PAS.
    PatelTejaUSB = 21,
}

impl CubicEos {
    /// Returns `true` if this is a three-parameter EOS requiring a C-parameter
    /// mixing rule in addition to the standard a, b mixing.
    pub fn is_three_parameter(&self) -> bool {
        matches!(
            self,
            CubicEos::SchmidtWenzel | CubicEos::PatelTeja | CubicEos::PatelTejaUSB
        )
    }
}

/// Vapor-phase model selection.
///
/// In VLE calculations, the vapor phase fugacity can be computed from a cubic EOS,
/// the virial equation (truncated at second coefficient), or the ideal gas assumption.
/// This enum wraps [`CubicEos`] with those additional options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VaporModel {
    /// Ideal gas assumption (fugacity coefficient φᵢ = 1). Valid only at low pressures.
    IdealGas,
    /// Second virial equation using Pitzer B⁰/B¹ correlations.
    /// Good for moderate pressures where cubic EOS is overkill.
    Virial,
    /// Cubic EOS — use the associated [`CubicEos`] variant for fugacity calculation.
    Cubic(CubicEos),
}

/// Liquid-phase model selection.
///
/// The liquid phase can be modeled with the same EOS as the vapor (φ-φ approach),
/// with an activity coefficient model (γ-φ approach), or with the Chao-Seader
/// correlation for specific compound classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiquidModel {
    /// Ideal solution (Raoult's law: γᵢ = 1). Valid for chemically similar components.
    IdealSolution,
    /// Cubic EOS for liquid fugacity (φ-φ approach). Same EOS is used for both phases.
    Cubic(CubicEos),
    /// Activity coefficient model (γ-φ approach). Better for polar/non-ideal liquid mixtures.
    Activity(super::ActivityModel),
    /// Chao-Seader liquid fugacity correlation. Semi-empirical method with special
    /// handling for hydrogen and methane. Ref (4), legacy/pascal/TERMOII.PAS.
    ChaoSeader,
}

/// Phase identifier used to select liquid or vapor root from the cubic solver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseId {
    Vapor,
    Liquid,
}

// #[cfg(test)] is a conditional compilation attribute. It tells the Rust
// compiler: "only compile the code inside this module when running tests"
// (i.e., when you run `cargo test`). In a normal build (`cargo build`),
// everything inside this block is completely ignored — it won't end up
// in your final binary. This is Rust's built-in way of keeping test code
// next to the code it tests, without bloating the production build.
#[cfg(test)]
mod tests {
    // `use super::*` imports everything from the parent module (the file
    // above this block) into the test module, so we can use CubicEos
    // and its methods directly without fully qualifying them.
    use super::*;

    // #[test] marks this function as a test case. `cargo test` will
    // discover and run every function with this attribute. If the
    // function completes without panicking, the test passes. If any
    // assert! macro fails, it panics and the test is reported as failed.
    #[test]
    fn discriminant_values_match_legacy() {
        // `as i32` casts the enum variant to its integer value (set by
        // #[repr(i32)] above). These assertions verify that our Rust enum
        // values match the exact integer codes used in the legacy VB6
        // program — if someone accidentally reorders the enum or changes
        // a value, this test will catch it.
        assert_eq!(CubicEos::PR1976 as i32, 0);
        assert_eq!(CubicEos::RKS1972 as i32, 2);
        assert_eq!(CubicEos::PRSV1986 as i32, 15);
        assert_eq!(CubicEos::PROL1998 as i32, 18);
        assert_eq!(CubicEos::SchmidtWenzel as i32, 19);
        assert_eq!(CubicEos::PatelTeja as i32, 20);
        assert_eq!(CubicEos::PatelTejaUSB as i32, 21);
    }

    #[test]
    fn three_parameter_detection() {
        // assert!(condition) passes if condition is true, panics if false.
        // The `!` before the call negates the result, so these first two
        // lines check that PR1976 and RKS1972 are NOT three-parameter EOS.
        assert!(!CubicEos::PR1976.is_three_parameter());
        assert!(!CubicEos::RKS1972.is_three_parameter());
        // These three lines verify that the three-parameter EOS variants
        // are correctly identified by the is_three_parameter() method.
        assert!(CubicEos::SchmidtWenzel.is_three_parameter());
        assert!(CubicEos::PatelTeja.is_three_parameter());
        assert!(CubicEos::PatelTejaUSB.is_three_parameter());
    }
}
