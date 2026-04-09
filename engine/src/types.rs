//! Core data structures shared across all VLE engine modules.
//!
//! These structs represent the fundamental objects in a VLE calculation:
//!
//! - **Component**: A single pure chemical substance with its thermodynamic properties
//!   (critical point, acentric factor, heat capacity coefficients, etc.). Properties
//!   come from databases like DIPPR or are fit to experimental data.
//!
//! - **Mixture**: A collection of components with their mole fractions, plus the
//!   model selections (which EOS, activity model, mixing rule to use).
//!
//! - **Flow**: A process stream at specific T, P conditions with known phase
//!   compositions. Represents the liquid, vapor, or feed stream in a flash calculation.
//!
//! - **Tolerances**: Convergence criteria for iterative calculations. Flash calculations
//!   are solved by Newton-Raphson or successive substitution, and these thresholds
//!   determine when the iteration has converged.
//!
//! - **ReferenceState**: The thermodynamic reference point for enthalpy and entropy.
//!   H and S are always relative to a chosen reference — typically ideal gas at 298 K
//!   and 101.325 kPa, or saturated liquid/vapor at a reference temperature.
//!
//! ## Field naming conventions
//!
//! Fields use standard thermodynamic notation where possible:
//! - `tc`, `pc`, `vc`, `zc` — critical temperature/pressure/volume/compressibility
//! - `omega` — acentric factor (ω), measures molecular non-sphericity
//! - `tb` — normal boiling point temperature
//! - `mw` — molecular weight (g/mol)
//! - `cp_coeffs` — ideal gas heat capacity polynomial coefficients
//!
//! ## Units
//!
//! All fields use the engine's canonical units (see lib.rs):
//! - Temperature: K | Pressure: kPa | Volume: cm³/mol | Energy: kJ/kmol

// `use` imports types from other modules so we can refer to them by
// their short name (e.g., `MixingRule`) instead of the full path
// (e.g., `crate::mixing::MixingRule`).
//
// `crate::` means "start from the root of this crate (library)".
// The curly braces `{LiquidModel, VaporModel}` import multiple
// types from the same module in a single line.
use crate::eos::{LiquidModel, VaporModel};
use crate::mixing::MixingRule;
use crate::saturation::SatPressureModel;

/// Universal gas constant in **kJ/(kmol·K)**.
///
/// This value matches both legacy codebases and is used consistently
/// throughout all thermodynamic calculations.
pub const R_GAS: f64 = 8.31451;

/// Pure component thermodynamic properties.
///
/// Union of all fields from both legacy codebases:
/// - VB6: `clsCriticalProps.cls` + `clsOtherProps.cls` + `clsQbicsPure.cls`
/// - Pascal: `Registro` record type in `TERMOI.PAS`
///
/// Not all fields are needed for every calculation. For example, `dipole_moment`
/// and `solubility_param` are only used by polar-capable EOS variants and the
/// Scatchard-Hildebrand activity model, respectively.
// Auto-implement Debug and Clone for this struct:
//   Debug — allows printing with {:?} for debugging (e.g., println!("{:?}", comp))
//   Clone — allows creating a deep copy via .clone()
// Note: unlike the enums (which also derive Copy), structs containing
// heap-allocated data like String cannot derive Copy — they must use
// .clone() explicitly to make a copy.
#[derive(Debug, Clone)]
pub struct Component {
    /// Component name (e.g., "methane", "water").
    pub name: String,

    // --- Critical properties (required for all EOS calculations) ---
    /// Critical temperature in **K**.
    pub tc: f64,
    /// Critical pressure in **kPa**.
    pub pc: f64,
    /// Critical molar volume in **cm³/mol**.
    pub vc: f64,
    /// Critical compressibility factor Zc = Pc·Vc/(R·Tc). Dimensionless.
    pub zc: f64,
    /// Acentric factor ω. Dimensionless. Measures deviation from spherical
    /// molecular shape; used in most α(Tr) functions.
    pub omega: f64,

    // --- Boiling point and molecular weight ---
    /// Normal boiling temperature in **K** (at 101.325 kPa).
    pub tb: f64,
    /// Molecular weight in **g/mol**.
    pub mw: f64,

    // --- Heat capacity coefficients ---
    /// Ideal gas heat capacity polynomial coefficients [a₀, a₁, a₂, a₃, a₄].
    /// Cp°/R = a₀ + a₁T + a₂T² + a₃T³ + a₄T⁴, with T in **K** and Cp° in **kJ/(kmol·K)**.
    pub cp_coeffs: [f64; 5],

    // --- Saturation pressure coefficients ---
    /// Antoine (or other correlation) coefficients [a₁, a₂, a₃].
    /// Antoine form: ln(P/Pc) = a₁ - a₂/(a₃ + T), T in **K**, P in **kPa**.
    /// Ref (4): Da Silva & Báez (1989), legacy/pascal/TERMOI.PAS.
    pub psat_coeffs: Vec<f64>,

    // --- Polar/special parameters (from Pascal `Registro` and VB6 `clsOtherProps`) ---
    /// Dipole moment in **Debye**. Used by polar-capable EOS variants.
    /// Field `momentoDip` in Pascal. Zero for non-polar molecules.
    pub dipole_moment: f64,
    /// Solubility parameter δ in **(cal/cm³)^0.5**. Used by the Scatchard-Hildebrand
    /// activity model. Field `delta` in Pascal and VB6.
    pub solubility_param: f64,
    /// Liquid molar volume at 25°C in **cm³/mol**. Used by Wilson and
    /// Scatchard-Hildebrand activity models. Field `vl` in Pascal.
    pub liquid_volume: f64,

    // --- EOS-specific fitting parameters (from VB6 `clsOtherProps`) ---
    /// Rackett compressibility factor ZRA. Used in the Rackett equation for
    /// saturated liquid volume. Different from Zc for most compounds.
    pub zra: f64,
    /// SRK-specific acentric factor wSRK. Some compounds have a separate ω
    /// optimized for the SRK EOS.
    pub omega_srk: f64,
    /// Polar extension parameter m. Used by RKSmn1980, PRMmn1989, and similar
    /// α(Tr) functions that add polar correction terms.
    pub m_polar: f64,
    /// Polar extension parameter n. Used with `m_polar` for polar α functions.
    pub n_polar: f64,
    /// Polar extension parameter g. Used by some three-constant α functions.
    pub g_polar: f64,
    /// PRSV K₁ parameter. Component-specific constant for the PRSV1986 EOS variant.
    pub prsv_k1: f64,
}

// `impl Default for Component` provides a default constructor, so you
// can create a Component with all fields set to zero/empty by calling
// `Component::default()`. This is Rust's standard trait for "give me a
// blank instance". We implement it manually here (instead of using
// #[derive(Default)]) because the result is the same but being explicit
// makes the initial values clear. `Self` is shorthand for `Component`.
impl Default for Component {
    fn default() -> Self {
        Self {
            name: String::new(),
            tc: 0.0,
            pc: 0.0,
            vc: 0.0,
            zc: 0.0,
            omega: 0.0,
            tb: 0.0,
            mw: 0.0,
            cp_coeffs: [0.0; 5],
            psat_coeffs: Vec::new(),
            dipole_moment: 0.0,
            solubility_param: 0.0,
            liquid_volume: 0.0,
            zra: 0.0,
            omega_srk: 0.0,
            m_polar: 0.0,
            n_polar: 0.0,
            g_polar: 0.0,
            prsv_k1: 0.0,
        }
    }
}

/// A multicomponent mixture with model selections.
///
/// Groups a set of components with their mole fractions and the user's choice
/// of thermodynamic models. Also stores binary interaction parameters (kij for
/// EOS mixing rules, Aij for activity models).
///
/// Maps to VB6 `clsAllProps.cls` + model selection fields from `clsLVE.cls`.
pub struct Mixture {
    /// Component list with properties.
    pub components: Vec<Component>,
    /// Overall mole fractions (must sum to 1.0). Length = components.len().
    pub mole_fractions: Vec<f64>,

    // --- Model selections ---
    /// Liquid-phase thermodynamic model.
    pub liquid_model: LiquidModel,
    /// Vapor-phase thermodynamic model.
    pub vapor_model: VaporModel,
    /// Mixing rule for EOS parameter combination.
    pub mixing_rule: MixingRule,
    /// Saturation pressure correlation for initial K-value estimates.
    pub sat_pressure_model: SatPressureModel,

    // --- Binary interaction parameters ---
    /// kij matrix (symmetric, N×N). Used by EOS mixing rules.
    /// kij[i][j] = kij[j][i], kij[i][i] = 0. Dimensionless.
    pub kij: Vec<Vec<f64>>,
    /// Aij matrix (asymmetric, N×N). Used by activity coefficient models.
    /// Aij[i][j] ≠ Aij[j][i] in general. Units depend on the activity model.
    pub aij: Vec<Vec<f64>>,
}

/// A process stream at specified conditions.
///
/// Represents a liquid, vapor, or feed stream in a flash calculation.
/// After a flash converges, the liquid and vapor Flow objects contain the
/// equilibrium compositions and thermodynamic properties of each phase.
///
/// Maps to VB6 `clsFlow.cls` (LiquidFlow, VaporFlow, FeedFlow in `clsLVE.cls`).
pub struct Flow {
    /// Temperature in **K**.
    pub temperature: f64,
    /// Pressure in **kPa** (absolute).
    pub pressure: f64,
    /// Mole fractions per component. Length = number of components.
    /// For feed: overall composition (zi). For liquid: xi. For vapor: yi.
    pub mole_fractions: Vec<f64>,
    /// Total molar flow rate in **kmol** (or **kmol/h** for continuous processes).
    pub total_flow: f64,
    /// Molar enthalpy in **kJ/kmol** (relative to reference state).
    pub enthalpy: f64,
    /// Molar entropy in **kJ/(kmol·K)** (relative to reference state).
    pub entropy: f64,
    /// Compressibility factor Z = PV/(nRT). Dimensionless.
    pub z_factor: f64,
}

impl Default for Flow {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            pressure: 0.0,
            mole_fractions: Vec::new(),
            total_flow: 0.0,
            enthalpy: 0.0,
            entropy: 0.0,
            z_factor: 0.0,
        }
    }
}

/// Convergence tolerances for iterative VLE calculations.
///
/// Flash calculations use Newton-Raphson or successive substitution to solve
/// the equilibrium equations. These tolerances define when the iteration has
/// converged. Defaults match the VB6 `clsTolerances.cls` values.
pub struct Tolerances {
    /// Pressure convergence tolerance in **kPa**. Default: 1e-8.
    pub pressure: f64,
    /// Temperature convergence tolerance in **K**. Default: 1e-7.
    pub temperature: f64,
    /// Mole fraction convergence tolerance. Dimensionless. Default: 1e-5.
    pub mole_fraction: f64,
    /// Maximum number of iterations before declaring non-convergence. Default: 500.
    pub max_iterations: usize,
}

impl Default for Tolerances {
    fn default() -> Self {
        Self {
            pressure: 1e-8,
            temperature: 1e-7,
            mole_fraction: 1e-5,
            max_iterations: 500,
        }
    }
}

/// Thermodynamic reference state for enthalpy and entropy calculations.
///
/// Enthalpy (H) and entropy (S) are always relative to a chosen reference point.
/// The three standard choices are:
/// - **Saturated liquid** at reference T, P — common in chemical engineering
/// - **Saturated vapor** at reference T, P
/// - **Ideal gas** at reference T, P — the most fundamental reference
///
/// Maps to VB6 `clsReferencesSt.cls`.
pub struct ReferenceState {
    /// Reference temperature in **K**. Typically 298.15 K (25°C).
    pub temperature: f64,
    /// Reference pressure in **kPa** (absolute). Default: 101.325 kPa (1 atm).
    pub pressure: f64,
    /// Reference enthalpy in **kJ/kmol**. Default: 0.0.
    pub enthalpy: f64,
    /// Reference entropy in **kJ/(kmol·K)**. Default: 0.0.
    pub entropy: f64,
    /// Reference molar volume in **cm³/mol**. Computed from reference state EOS.
    pub molar_volume: f64,
    /// Reference compressibility factor. Computed from reference state EOS.
    pub z_factor: f64,
    /// Which phase defines the reference point.
    pub phase: ReferencePhase,
}

impl Default for ReferenceState {
    fn default() -> Self {
        Self {
            temperature: 298.15,
            pressure: 101.325,
            enthalpy: 0.0,
            entropy: 0.0,
            molar_volume: 0.0,
            z_factor: 0.0,
            phase: ReferencePhase::IdealGas,
        }
    }
}

/// Phase choice for the thermodynamic reference state.
///
/// Maps to VB6 `TADiPRefSt` enum in `clsReferencesSt.cls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ReferencePhase {
    /// Saturated liquid at reference T.
    SaturatedLiquid = 1,
    /// Saturated vapor at reference T.
    SaturatedVapor = 2,
    /// Ideal gas at reference T, P.
    IdealGas = 3,
}

/// Liquid molar volume model selection.
///
/// Used when the activity coefficient approach needs liquid volumes (e.g., Wilson
/// model, Scatchard-Hildebrand). Maps to VB6 `TADiPvlModel` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LiquidVolumeModel {
    /// Rackett equation. Simple corresponding-states method using ZRA.
    Rackett = 1,
    /// Thomson (COSTALD) correlation. Ref (18): Hankinson & Thomson.
    /// More accurate than Rackett for a wider range of compounds.
    Thomson = 2,
}

/// Type of VLE flash calculation.
///
/// Maps to VB6 `TypeCalculation` enum in `clsLVE.cls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalculationType {
    /// Dew point pressure: given T and vapor composition yi, find P and liquid xi.
    DewPressure,
    /// Dew point temperature: given P and vapor composition yi, find T and liquid xi.
    DewTemperature,
    /// Bubble point pressure: given T and liquid composition xi, find P and vapor yi.
    BubblePressure,
    /// Bubble point temperature: given P and liquid composition xi, find T and vapor yi.
    BubbleTemperature,
    /// Isothermal flash: given T, P, and overall composition zi, find vapor fraction
    /// and equilibrium compositions xi, yi.
    IsothermalFlash,
    /// Adiabatic flash: given P, overall composition zi, and feed enthalpy,
    /// find T, vapor fraction, and equilibrium compositions.
    AdiabaticFlash,
}

/// Dimensionless vs. dimensional flag for thermodynamic properties.
///
/// Some internal calculations work with dimensionless residual properties
/// (HR/RT, SR/R) while the final API returns dimensional values.
/// Maps to VB6 `TADiPdim` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionalFlag {
    /// Return dimensional quantities (kJ/kmol, kJ/(kmol·K)).
    Dimensional,
    /// Return dimensionless quantities (HR/RT, SR/R).
    Dimensionless,
}
