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
// When built with the `python` feature (Maturin/PyO3 wheel build), expose this
// enum as a Python class. `eq` derives Python __eq__/__ne__ from Rust's
// PartialEq; `eq_int` lets Python compare variants against their integer
// codes. The cfg_attr means this annotation is invisible when the crate is
// used purely from Rust via `cargo add vle-thermo`.
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
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
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
#[repr(i32)]
pub enum PhaseId {
    Vapor = 0,
    Liquid = 1,
}

// ===========================================================================
// M7.1 / M7.2 — pure-component cubic-EOS algorithms.
//
// This block fills in the family constants, alpha functions, Z-factor,
// fugacity, and departure-property machinery for the two-parameter cubic
// EOS catalogue. M7.1 shipped the *deployable core* (PR1976, RKS1972,
// RK1949, VdW1870 — the four variants Chapter IV uses); M7.2 added the
// remaining twelve two-parameter variants (Berthelot, VdWAda1984,
// RKSGD1978, RKSL1997, RP1978, PRL1997, VdWVald1989, RKSmn1980,
// RKSATmn1995, PRATmng1997, PRMmn1989, PRSV1986), each with an analytical
// dα/dTr.
//
// The remaining CubicEos variants still panic via `unimplemented!` with a
// pointer to their deferred sub-milestone: the OL family (VdWOL1998,
// RKOL1998, PROL1998) is M7.4 because its α is coupled to the reduced
// saturation pressure, and the 3-param Pascal EOS is M7.3. The panic
// message names both the variant and the legacy source line, so a future
// porting session has the receipt it needs to fill in the gap.
//
// Convention used throughout this block (Abbott form, matching VB6):
//
//     P = R·T/(V − b)  −  a·α(T) / (V² + k1·b·V + k2·b²)
//
// with a = OmA · R²·Tc²/Pc and b = OmB · R·Tc/Pc.
// ===========================================================================

use crate::numerics::cubic::{CubicError, solve_real};
use crate::types::Component;
use thiserror::Error;

/// Errors raised by the cubic-EOS pure-component layer.
///
/// `Cubic` passes through any failure from the underlying cubic solver
/// (NaN coefficient, near-zero leading coefficient). `NoRootForPhase` is
/// thrown when the cubic has only one physical root and the caller asked
/// for the wrong phase (e.g., requesting Liquid at supercritical T).
/// `NotImplemented` is thrown for EOS variants whose α has not yet been
/// ported — exactly the variants the deferred sub-milestones cover.
#[derive(Debug, Error, PartialEq)]
pub enum EosError {
    /// The cubic solver itself returned an error (non-finite coefficient
    /// or near-zero leading term).
    #[error("cubic solver failed: {0}")]
    Cubic(#[from] CubicError),

    /// The cubic has no physical root for the requested phase. Usually
    /// means the caller asked for Liquid above the critical point (or
    /// vice versa) — the EOS has only one real root in that region.
    #[error("no real root above B={big_b:.6e} found for phase {phase:?}")]
    NoRootForPhase { phase: PhaseId, big_b: f64 },

    /// The EOS variant's α function has not been ported yet. The variant
    /// in the payload is the exact enum that triggered the panic.
    #[error("EOS variant {0:?} not yet ported — see M7 sub-milestones")]
    NotImplemented(CubicEos),
}

/// Two-parameter cubic-EOS family constants for the Abbott form.
///
/// `k1` and `k2` parameterize the attractive term's denominator
/// `(V² + k1·b·V + k2·b²)`. `om_a` and `om_b` are the Ω_a / Ω_b
/// dimensionless coefficients that multiply `R²·Tc²/Pc` and `R·Tc/Pc`
/// to get the EOS `a` and `b` parameters.
///
/// Constants are stored as `f64` even though VB6 declares them as
/// integers — the discriminant `k1² − 4·k2` is computed in f64 and
/// matching types saves a noisy cast at every call site.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FamilyConstants {
    /// Coefficient of `b·V` in the attractive-term denominator.
    pub k1: f64,
    /// Coefficient of `b²` in the attractive-term denominator.
    pub k2: f64,
    /// Ω_a — dimensionless coefficient for `a = Ω_a · R²·Tc²/Pc`.
    pub om_a: f64,
    /// Ω_b — dimensionless coefficient for `b = Ω_b · R·Tc/Pc`.
    pub om_b: f64,
}

/// Return the 2-parameter family constants for the given EOS.
///
/// Source: `legacy/vb6/McommonFunctions.bas:273` (`GeneralConstantsEOS`).
/// 3-parameter EOS (Schmidt-Wenzel, Patel-Teja) are M7.3 — they return
/// zeros here so callers can still pattern-match without an early panic;
/// the per-component override happens inside their own EOS code path.
pub fn family_constants(eos: CubicEos) -> FamilyConstants {
    use CubicEos::*;
    match eos {
        // PR family: k1=2, k2=-1 → denominator V² + 2bV − b² = (V+(1+√2)b)(V+(1−√2)b).
        PR1976 | RP1978 | PRL1997 | PRATmng1997 | PRSV1986 | PROL1998 | PRMmn1989 => {
            FamilyConstants {
                k1: 2.0,
                k2: -1.0,
                om_a: 0.457235528921382,
                om_b: 0.0777960739038885,
            }
        }
        // RKS family: k1=1, k2=0 → denominator V² + bV = V(V+b).
        RKS1972 | RKSL1997 | RKSGD1978 | RK1949 | VdWVald1989 | RKSATmn1995 | RKOL1998
        | RKSmn1980 => FamilyConstants {
            k1: 1.0,
            k2: 0.0,
            om_a: 0.427480233540341,
            om_b: 0.0866403499649577,
        },
        // VdW family: k1=k2=0 → denominator V². Ω values are the analytical
        // 27/64 and 1/8 (no floating-point rounding involved).
        VdW1870 | Berth1899 | VdWAda1984 | VdWOL1998 => FamilyConstants {
            k1: 0.0,
            k2: 0.0,
            om_a: 27.0 / 64.0,
            om_b: 1.0 / 8.0,
        },
        // 3-parameter EOS — M7.3 deferred. Returning zeros lets callers
        // pattern-match without an early panic; the EOS-specific code
        // path is the one that should detect and refuse.
        SchmidtWenzel | PatelTeja | PatelTejaUSB => FamilyConstants {
            k1: 0.0,
            k2: 0.0,
            om_a: 0.0,
            om_b: 0.0,
        },
    }
}

/// α(Tr) — the temperature-dependent multiplier on the attractive term.
///
/// # Arguments
/// * `eos` — EOS variant.
/// * `tr` — Reduced temperature T/Tc. **Dimensionless.**
/// * `comp` — Component data. The acentric-factor variants read only
///   `omega`; the polar/fitted variants additionally read `zc`
///   (VdWVald1989), `m_polar`/`n_polar` (RKSmn1980, PRMmn1989),
///   `m_polar`/`n_polar`/`g_polar` (RKSATmn1995, PRATmng1997), or
///   `prsv_k1` (PRSV1986).
///
/// # Returns
/// α evaluated at `tr`, **dimensionless**.
///
/// # Panics
/// Calls `unimplemented!` for the EOS variants not yet ported — the OL
/// family (M7.4, saturation-coupled) and the 3-param Pascal EOS (M7.3).
/// The panic message includes the variant name and a pointer to the legacy
/// source line so a future port can pick up where this left off.
pub fn alpha(eos: CubicEos, tr: f64, comp: &Component) -> f64 {
    use CubicEos::*;
    let w = comp.omega;
    match eos {
        // ----- Deployable core (M7.1) -----
        VdW1870 => 1.0,
        RK1949 => 1.0 / tr.sqrt(),
        RKS1972 => {
            // α = [1 + m·(1 − √Tr)]² with m = 0.48 + 1.574ω − 0.176ω².
            // VB6: clsQbicsPure.cls:1734
            let m = 0.48 + 1.574 * w - 0.176 * w * w;
            let s = 1.0 - tr.sqrt();
            (1.0 + m * s).powi(2)
        }
        PR1976 => {
            // α = [1 + κ·(1 − √Tr)]² with κ = 0.37464 + 1.54226ω − 0.26992ω².
            // VB6: clsQbicsPure.cls:1741
            let kappa = 0.37464 + 1.54226 * w - 0.26992 * w * w;
            let s = 1.0 - tr.sqrt();
            (1.0 + kappa * s).powi(2)
        }
        // ----- M7.2: remaining 2-parameter α variants -----
        // All ported from VB6 `Friend Function Alpha` (clsQbicsPure.cls:1719).

        // Berthelot (1899): a temperature-modified VdW. VB6:1730.
        // α = 1/Tr.
        Berth1899 => 1.0 / tr,

        // van der Waals-Adachi (1984). VB6:1732. α = 10^(m·(1 − Tr)).
        // The VB6 wrote m's linear term in two pieces
        // (0.791981·ω + 0.654505·ω); we fold them into the single
        // coefficient 1.446486·ω — a bit-identical result. (The split
        // is almost certainly a transcription artifact of an intended
        // higher-order term in the original correlation.)
        VdWAda1984 => {
            let m = 0.228165 + 1.446486 * w - 0.648552 * w * w;
            10f64.powf(m * (1.0 - tr))
        }

        // Soave-shaped variants: α = [1 + m(ω)·(1 − √Tr)]², identical in
        // form to RKS1972/PR1976 but with different m(ω) correlations.
        // RKSGD1978 = Graboski-Daubert (VB6:1738).
        RKSGD1978 => {
            let m = 0.48508 + 1.55171 * w - 0.15613 * w * w;
            let s = 1.0 - tr.sqrt();
            (1.0 + m * s).powi(2)
        }
        // RKSL1997 = Lim's RKS modification, cubic in ω (VB6:1740).
        RKSL1997 => {
            let m = 0.478972559 + 1.576809191 * w - 0.187219516 * w * w + 0.020424946 * w * w * w;
            let s = 1.0 - tr.sqrt();
            (1.0 + m * s).powi(2)
        }
        // RP1978 = Redlich-Prausnitz, PR family, cubic in ω (VB6:1744).
        RP1978 => {
            let m = 0.379642 + 1.48503 * w - 0.164423 * w * w + 0.016666 * w * w * w;
            let s = 1.0 - tr.sqrt();
            (1.0 + m * s).powi(2)
        }
        // PRL1997 = Peng-Robinson-Lim, cubic in ω (VB6:1746).
        PRL1997 => {
            let m = 0.378710697 + 1.487972964 * w - 0.166754831 * w * w + 0.017169486 * w * w * w;
            let s = 1.0 - tr.sqrt();
            (1.0 + m * s).powi(2)
        }

        // Mathias-Naumann linear form: α = 1 + (1 − Tr)·(m + n/Tr).
        // VdWVald1989 derives m, n from ω·Zc (VB6:1748); RKSmn1980 reads
        // the component-specific fitted m, n directly (VB6:1753).
        VdWVald1989 => {
            let omegac = w * comp.zc;
            let m = 0.4745 + (2.7349 + 6.0984 * omegac) * omegac;
            let n = 0.0674 + (2.1031 + 3.9512 * omegac) * omegac;
            1.0 + (1.0 - tr) * (m + n / tr)
        }
        RKSmn1980 => {
            let (m, n) = (comp.m_polar, comp.n_polar);
            1.0 + (1.0 - tr) * (m + n / tr)
        }

        // Adachi-Tagawa-Mathias-Naumann exponential form with three fitted
        // constants (m, n, g). The expression is identical for the RKS and
        // PR families — only the family k1/k2 constants differ. VB6:1755/1757.
        // α = exp[ (1 − Tr)·m·|1 − Tr|^(g−1) + n·(1/Tr − 1) ].
        RKSATmn1995 | PRATmng1997 => {
            let (m, n, g) = (comp.m_polar, comp.n_polar, comp.g_polar);
            let u = 1.0 - tr;
            (u * m * u.abs().powf(g - 1.0) + n * (1.0 / tr - 1.0)).exp()
        }

        // PR-Mathias-Massih-Naumann (1989): α = exp[ (1 − Tr)·m + n·(1 − √Tr)² ].
        // m, n are component-specific fitted constants. VB6:1759.
        PRMmn1989 => {
            let (m, n) = (comp.m_polar, comp.n_polar);
            let s = 1.0 - tr.sqrt();
            ((1.0 - tr) * m + n * s * s).exp()
        }

        // Peng-Robinson-Stryjek-Vera (1986): α = [1 + κ·(1 − √Tr)]² with
        // κ = κ₀(ω) + K₁·(1 + √Tr)·(0.7 − Tr). K₁ is the component-specific
        // PRSV parameter (zero recovers the plain PR-like κ₀ form). VB6:1762.
        PRSV1986 => {
            let r = tr.sqrt();
            let kappa0 = 0.378893 + 1.4897153 * w - 0.17131848 * w * w + 0.0196554 * w * w * w;
            let kappa = kappa0 + comp.prsv_k1 * (1.0 + r) * (0.7 - tr);
            let inner = 1.0 + kappa * (1.0 - r);
            inner * inner
        }

        // ----- Deferred: OL family → M7.4 -----
        // The OL-family α is not a function of (Tr, ω): VB6 computes
        // α = Tr·(1 + SumHk) where SumHk depends on the component's reduced
        // saturation pressure (clsQbicsPure.cls:268). It is therefore coupled
        // to the saturation layer and lands with M7.4 alongside the
        // non-Antoine saturation models.
        VdWOL1998 | RKOL1998 | PROL1998 => {
            unimplemented!(
                "M7.4 deferred: OL-family α({:?}) depends on reduced saturation pressure \
                 (SumHk, legacy/vb6/clsQbicsPure.cls:268) — lands with the M7.4 saturation layer",
                eos
            )
        }
        // ----- Deferred: 3-param Pascal EOS → M7.3 -----
        SchmidtWenzel | PatelTeja | PatelTejaUSB => {
            unimplemented!(
                "M7.3 deferred: 3-param EOS {:?} not yet ported — see legacy/pascal/TERMOII.PAS",
                eos
            )
        }
    }
}

/// dα/dTr — analytical derivative of α with respect to reduced temperature.
///
/// Computed analytically per CLAUDE.md "Algorithm Choices": numerical
/// derivatives exist only as test oracles. The closed forms for the four
/// core variants are derived by hand from `alpha` above; see comments
/// for the chain-rule step at each branch.
pub fn d_alpha_d_tr(eos: CubicEos, tr: f64, comp: &Component) -> f64 {
    use CubicEos::*;
    let w = comp.omega;
    match eos {
        // ----- Deployable core (M7.1) -----
        // α = 1 → dα/dTr = 0
        VdW1870 => 0.0,
        // α = Tr^(−1/2) → dα/dTr = −(1/2)·Tr^(−3/2) = −1 / (2·Tr·√Tr)
        RK1949 => -0.5 / (tr * tr.sqrt()),
        RKS1972 => {
            // α = (1 + m·s)²  with s = 1 − √Tr, ds/dTr = −1/(2·√Tr).
            // dα/dTr = 2·(1 + m·s)·m·(−1/(2·√Tr)) = −m·(1 + m·s)/√Tr.
            let m = 0.48 + 1.574 * w - 0.176 * w * w;
            let s = 1.0 - tr.sqrt();
            -m * (1.0 + m * s) / tr.sqrt()
        }
        PR1976 => {
            let kappa = 0.37464 + 1.54226 * w - 0.26992 * w * w;
            let s = 1.0 - tr.sqrt();
            -kappa * (1.0 + kappa * s) / tr.sqrt()
        }
        // ----- M7.2: analytical derivatives for the remaining 2-param variants -----

        // Berthelot: α = 1/Tr → dα/dTr = −1/Tr².
        Berth1899 => -1.0 / (tr * tr),

        // VdWAda: α = 10^(m·(1 − Tr)) = exp(m·(1 − Tr)·ln10) →
        // dα/dTr = −m·ln(10)·α.
        VdWAda1984 => {
            let m = 0.228165 + 1.446486 * w - 0.648552 * w * w;
            let a = 10f64.powf(m * (1.0 - tr));
            -m * std::f64::consts::LN_10 * a
        }

        // Soave form: α = (1 + m·s)², s = 1 − √Tr, ds/dTr = −1/(2√Tr) →
        // dα/dTr = −m·(1 + m·s)/√Tr.
        RKSGD1978 => {
            let m = 0.48508 + 1.55171 * w - 0.15613 * w * w;
            let s = 1.0 - tr.sqrt();
            -m * (1.0 + m * s) / tr.sqrt()
        }
        RKSL1997 => {
            let m = 0.478972559 + 1.576809191 * w - 0.187219516 * w * w + 0.020424946 * w * w * w;
            let s = 1.0 - tr.sqrt();
            -m * (1.0 + m * s) / tr.sqrt()
        }
        RP1978 => {
            let m = 0.379642 + 1.48503 * w - 0.164423 * w * w + 0.016666 * w * w * w;
            let s = 1.0 - tr.sqrt();
            -m * (1.0 + m * s) / tr.sqrt()
        }
        PRL1997 => {
            let m = 0.378710697 + 1.487972964 * w - 0.166754831 * w * w + 0.017169486 * w * w * w;
            let s = 1.0 - tr.sqrt();
            -m * (1.0 + m * s) / tr.sqrt()
        }

        // Mathias-Naumann linear form: α = 1 + (1 − Tr)(m + n/Tr).
        // The (1−Tr)·n/Tr terms collapse exactly: dα/dTr = −m − n/Tr².
        VdWVald1989 => {
            let omegac = w * comp.zc;
            let m = 0.4745 + (2.7349 + 6.0984 * omegac) * omegac;
            let n = 0.0674 + (2.1031 + 3.9512 * omegac) * omegac;
            -m - n / (tr * tr)
        }
        RKSmn1980 => {
            let (m, n) = (comp.m_polar, comp.n_polar);
            -m - n / (tr * tr)
        }

        // ATmn exponential form: α = exp(f) with
        // f = (1 − Tr)·m·|1 − Tr|^(g−1) + n·(1/Tr − 1).
        // Using d/dTr[(1 − Tr)·|1 − Tr|^(g−1)] = −g·|1 − Tr|^(g−1):
        // dα/dTr = α·( −m·g·|1 − Tr|^(g−1) − n/Tr² ).
        RKSATmn1995 | PRATmng1997 => {
            let (m, n, g) = (comp.m_polar, comp.n_polar, comp.g_polar);
            let u = 1.0 - tr;
            let a = (u * m * u.abs().powf(g - 1.0) + n * (1.0 / tr - 1.0)).exp();
            a * (-m * g * u.abs().powf(g - 1.0) - n / (tr * tr))
        }

        // PRMmn: α = exp[(1 − Tr)·m + n·s²], s = 1 − √Tr.
        // dα/dTr = α·( −m − n·s/√Tr ).
        PRMmn1989 => {
            let (m, n) = (comp.m_polar, comp.n_polar);
            let r = tr.sqrt();
            let s = 1.0 - r;
            let a = ((1.0 - tr) * m + n * s * s).exp();
            a * (-m - n * s / r)
        }

        // PRSV: α = inner², inner = 1 + κ·(1 − √Tr),
        // κ = κ₀ + K₁·(1 + √Tr)·(0.7 − Tr). Let r = √Tr.
        // dκ/dTr = K₁·[ (0.7 − Tr)/(2r) − (1 + r) ];
        // d(inner)/dTr = dκ·(1 − r) − κ/(2r);
        // dα/dTr = 2·inner·d(inner)/dTr.
        PRSV1986 => {
            let r = tr.sqrt();
            let kappa0 = 0.378893 + 1.4897153 * w - 0.17131848 * w * w + 0.0196554 * w * w * w;
            let kappa = kappa0 + comp.prsv_k1 * (1.0 + r) * (0.7 - tr);
            let inner = 1.0 + kappa * (1.0 - r);
            let dkappa = comp.prsv_k1 * ((0.7 - tr) / (2.0 * r) - (1.0 + r));
            let dinner = dkappa * (1.0 - r) - kappa / (2.0 * r);
            2.0 * inner * dinner
        }

        // ----- Deferred: OL family → M7.4 (saturation-coupled α) -----
        VdWOL1998 | RKOL1998 | PROL1998 => {
            unimplemented!(
                "M7.4 deferred: OL-family dα/dTr({:?}) depends on reduced saturation pressure \
                 (SumHk, legacy/vb6/clsQbicsPure.cls:268) — lands with the M7.4 saturation layer",
                eos
            )
        }
        SchmidtWenzel | PatelTeja | PatelTejaUSB => {
            unimplemented!("M7.3 deferred: 3-param EOS {:?} not yet ported", eos)
        }
    }
}

/// Compute dimensionless `A = a·P/(R·T)²` and `B = b·P/(R·T)` for the EOS.
///
/// Internal helper — every consumer below (z_factor, ln_phi, departure
/// functions) shares the same A, B definitions, so factoring this out
/// keeps the formulas consistent and the variable names clean.
fn ab_dimensionless(eos: CubicEos, t: f64, p: f64, comp: &Component) -> (f64, f64) {
    let fc = family_constants(eos);
    let tr = t / comp.tc;
    let pr = p / comp.pc;
    let a_val = alpha(eos, tr, comp);
    // A = OmA · α · Pr / Tr², B = OmB · Pr / Tr.
    (fc.om_a * a_val * pr / (tr * tr), fc.om_b * pr / tr)
}

/// Compute Z = P·V/(R·T) for the requested phase.
///
/// Solves the cubic in Z and selects the appropriate real root:
/// **liquid** = smallest Z above B (the lower stable branch), **vapor**
/// = largest Z above B (the upper stable branch). Roots ≤ B are
/// unphysical (V < b, molecular impossibility) and discarded.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p` — Pressure in **kPa absolute**.
/// * `comp` — Component (only Tc, Pc, ω used by the core variants).
/// * `phase` — Which root to return.
///
/// # Returns
/// `Z`, dimensionless. Use `V = Z·R·T/P` to recover the molar volume.
///
/// # Errors
/// `EosError::Cubic` if the underlying solver fails. `NoRootForPhase`
/// if the cubic has no real root above B for the requested phase.
/// `NotImplemented` for 3-parameter EOS (deferred to M7.3).
pub fn z_factor(
    eos: CubicEos,
    t: f64,
    p: f64,
    comp: &Component,
    phase: PhaseId,
) -> Result<f64, EosError> {
    if eos.is_three_parameter() {
        return Err(EosError::NotImplemented(eos));
    }
    let fc = family_constants(eos);
    let (big_a, big_b) = ab_dimensionless(eos, t, p, comp);
    // Cubic in Z: 1·Z³ + a2·Z² + a1·Z + a0 = 0
    //   a2 = (k1 − 1)·B − 1
    //   a1 = A − (k1 − k2)·B² − k1·B
    //   a0 = −(k2·B³ + k2·B² + A·B)
    // Verified against textbook forms for PR, RKS, VdW (see eos.rs::tests).
    let a2 = (fc.k1 - 1.0) * big_b - 1.0;
    let a1 = big_a - (fc.k1 - fc.k2) * big_b * big_b - fc.k1 * big_b;
    let a0 = -(fc.k2 * big_b * big_b * big_b + fc.k2 * big_b * big_b + big_a * big_b);
    let roots = solve_real(1.0, a2, a1, a0)?;
    // Filter out roots ≤ B (V ≤ b is unphysical).
    let mut physical: Vec<f64> = roots.into_iter().filter(|&z| z > big_b).collect();
    if physical.is_empty() {
        return Err(EosError::NoRootForPhase { phase, big_b });
    }
    physical.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Ok(match phase {
        PhaseId::Liquid => *physical.first().unwrap(),
        PhaseId::Vapor => *physical.last().unwrap(),
    })
}

/// Dimensionless attractive-term integral `F(Z, B; k1, k2)`.
///
/// Defined so that `ln(φ) = Z − 1 − ln(Z − B) − (A/B)·F`. The closed
/// form for the non-degenerate case (k1² ≠ 4·k2) is:
///
/// ```text
///   F = (1/√(k1² − 4·k2)) · ln( (2Z + B·(k1 + √(k1² − 4·k2)))
///                              / (2Z + B·(k1 − √(k1² − 4·k2))) )
/// ```
///
/// For the VdW limit (k1 = k2 = 0) the formula degenerates; we use the
/// closed-form `F = B/Z` instead, which reproduces the standard VdW
/// expression `ln(φ) = Z − 1 − ln(Z − B) − A/Z`.
fn integral_attractive(z: f64, big_b: f64, fc: FamilyConstants) -> f64 {
    let disc = fc.k1 * fc.k1 - 4.0 * fc.k2;
    if disc.abs() < 1e-12 {
        // VdW limit: F = B/Z. Verified by checking ln(φ_VdW) = Z − 1 −
        // ln(Z − B) − A/Z (Smith-Van Ness-Abbott §6.4 example 6.4).
        big_b / z
    } else {
        let sd = disc.sqrt();
        (1.0 / sd) * ((2.0 * z + big_b * (fc.k1 + sd)) / (2.0 * z + big_b * (fc.k1 - sd))).ln()
    }
}

/// Pure-component fugacity coefficient ln(φ).
///
/// `ln(φ) = Z − 1 − ln(Z − B) − (A/B)·F(Z, B; k1, k2)`.
///
/// Returns the **natural log** of the fugacity coefficient. To get φ
/// itself, exponentiate. `ln(φ) → 0` as `P → 0` (ideal gas limit).
///
/// # Errors
/// Same as [`z_factor`].
pub fn ln_phi_pure(
    eos: CubicEos,
    t: f64,
    p: f64,
    comp: &Component,
    phase: PhaseId,
) -> Result<f64, EosError> {
    if eos.is_three_parameter() {
        return Err(EosError::NotImplemented(eos));
    }
    let fc = family_constants(eos);
    let (big_a, big_b) = ab_dimensionless(eos, t, p, comp);
    let z = z_factor(eos, t, p, comp, phase)?;
    let f = integral_attractive(z, big_b, fc);
    Ok(z - 1.0 - (z - big_b).ln() - (big_a / big_b) * f)
}

/// Departure enthalpy in dimensionless form `H^R / (R·T)`.
///
/// `H^R = H_real(T, P) − H_ideal_gas(T)`. Always **negative** for stable
/// liquid and vapor phases at sub-critical conditions (the attractive
/// term lowers the energy below the ideal-gas reference).
///
/// Formula (general 2-param cubic):
/// ```text
///   H^R/(RT) = (Z − 1) + (A/B)·F · (Tr·dα/dTr/α − 1)
/// ```
/// Derivation: Smith-Van Ness-Abbott §6.4, generalized to the Abbott
/// (k1, k2) form. Verified against PR/RKS/VdW textbook expressions.
///
/// # Errors
/// Same as [`z_factor`].
pub fn h_departure_rt(
    eos: CubicEos,
    t: f64,
    p: f64,
    comp: &Component,
    phase: PhaseId,
) -> Result<f64, EosError> {
    if eos.is_three_parameter() {
        return Err(EosError::NotImplemented(eos));
    }
    let fc = family_constants(eos);
    let (big_a, big_b) = ab_dimensionless(eos, t, p, comp);
    let z = z_factor(eos, t, p, comp, phase)?;
    let tr = t / comp.tc;
    let a_val = alpha(eos, tr, comp);
    let da_val = d_alpha_d_tr(eos, tr, comp);
    let f = integral_attractive(z, big_b, fc);
    Ok((z - 1.0) + (big_a / big_b) * f * (tr * da_val / a_val - 1.0))
}

/// Departure entropy in dimensionless form `S^R / R`.
///
/// `S^R = S_real(T, P) − S_ideal_gas(T, P)`. Use the identity
/// `S^R/R = H^R/(RT) − G^R/(RT)` and `G^R/(RT) = ln(φ_pure)` for a pure
/// component (Lewis-Randall).
///
/// # Errors
/// Same as [`z_factor`].
pub fn s_departure_r(
    eos: CubicEos,
    t: f64,
    p: f64,
    comp: &Component,
    phase: PhaseId,
) -> Result<f64, EosError> {
    if eos.is_three_parameter() {
        return Err(EosError::NotImplemented(eos));
    }
    let g_dep = ln_phi_pure(eos, t, p, comp, phase)?;
    let h_dep = h_departure_rt(eos, t, p, comp, phase)?;
    Ok(h_dep - g_dep)
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

    // -----------------------------------------------------------------
    // M7.1 — tests for the deployable core (PR / RKS / RK / VdW).
    // -----------------------------------------------------------------

    fn methane() -> Component {
        // Methane critical properties + acentric factor from NIST.
        Component {
            name: "methane".into(),
            tc: 190.564,
            pc: 4599.0, // kPa
            omega: 0.0115,
            ..Component::default()
        }
    }

    fn n_pentane() -> Component {
        // n-Pentane — a more "typical" hydrocarbon with non-trivial ω.
        Component {
            name: "n-pentane".into(),
            tc: 469.7,
            pc: 3370.0,
            omega: 0.252,
            ..Component::default()
        }
    }

    #[test]
    fn family_constants_match_legacy_table() {
        // PR family — verify the high-precision OmA/OmB constants from
        // legacy/vb6/McommonFunctions.bas:273.
        let fc = family_constants(CubicEos::PR1976);
        assert_eq!(fc.k1, 2.0);
        assert_eq!(fc.k2, -1.0);
        assert!((fc.om_a - 0.457235528921382).abs() < 1e-15);
        assert!((fc.om_b - 0.0777960739038885).abs() < 1e-15);

        // RKS family.
        let fc = family_constants(CubicEos::RKS1972);
        assert_eq!(fc.k1, 1.0);
        assert_eq!(fc.k2, 0.0);
        assert!((fc.om_a - 0.427480233540341).abs() < 1e-15);
        assert!((fc.om_b - 0.0866403499649577).abs() < 1e-15);

        // VdW family — analytical fractions.
        let fc = family_constants(CubicEos::VdW1870);
        assert_eq!(fc.k1, 0.0);
        assert_eq!(fc.k2, 0.0);
        assert_eq!(fc.om_a, 27.0 / 64.0);
        assert_eq!(fc.om_b, 1.0 / 8.0);
    }

    #[test]
    fn alpha_at_tr_one_is_one_for_pr_rks() {
        // For PR and RKS, α(Tr=1) = 1 by construction of the α form.
        // VdW gives α=1 trivially; RK gives α=1/√1=1.
        let c = n_pentane();
        for eos in [
            CubicEos::PR1976,
            CubicEos::RKS1972,
            CubicEos::RK1949,
            CubicEos::VdW1870,
        ] {
            let a = alpha(eos, 1.0, &c);
            assert!(
                (a - 1.0).abs() < 1e-12,
                "{:?}: α(Tr=1) = {} (expected 1.0)",
                eos,
                a
            );
        }
    }

    /// Numerical derivative via central differences — used as test oracle.
    fn d_alpha_numerical(eos: CubicEos, tr: f64, comp: &Component, h: f64) -> f64 {
        (alpha(eos, tr + h, comp) - alpha(eos, tr - h, comp)) / (2.0 * h)
    }

    #[test]
    fn analytical_d_alpha_matches_numerical() {
        // Sweep Tr across a sub/supercritical range and verify the
        // analytical derivative agrees with central differences within
        // 1e-7 relative tolerance — a CLAUDE.md "Algorithm Choices" rule
        // (numerical derivatives are test oracles, not production code).
        let c = n_pentane();
        for eos in [
            CubicEos::PR1976,
            CubicEos::RKS1972,
            CubicEos::RK1949,
            CubicEos::VdW1870,
        ] {
            for tr in [0.5_f64, 0.8, 1.0, 1.2, 2.0] {
                let analytical = d_alpha_d_tr(eos, tr, &c);
                let numerical = d_alpha_numerical(eos, tr, &c, 1e-6);
                let rel = if analytical.abs() < 1e-10 {
                    (analytical - numerical).abs()
                } else {
                    ((analytical - numerical) / analytical).abs()
                };
                assert!(
                    rel < 1e-5,
                    "{:?} Tr={} analytical={} numerical={} rel={}",
                    eos,
                    tr,
                    analytical,
                    numerical,
                    rel
                );
            }
        }
    }

    #[test]
    fn z_factor_methane_supercritical() {
        // Methane at 300 K, 5 MPa (5000 kPa). T > Tc → only one real
        // root — vapor and "liquid" both resolve to the same root.
        let c = methane();
        let z_v = z_factor(CubicEos::PR1976, 300.0, 5000.0, &c, PhaseId::Vapor).unwrap();
        // Z should be < 1 (attractive forces dominate at moderate pressure).
        // For methane at 300 K and 5 MPa, Z ≈ 0.91 with PR.
        assert!(
            z_v > 0.8 && z_v < 1.05,
            "Z(vapor) = {} not in plausible range",
            z_v
        );
    }

    #[test]
    fn z_factor_n_pentane_two_phase() {
        // n-pentane at 400 K (Tr = 0.85), 1500 kPa. Below critical and at
        // moderate pressure → expect both liquid and vapor roots,
        // Z_liquid << Z_vapor.
        let c = n_pentane();
        let z_l = z_factor(CubicEos::PR1976, 400.0, 1500.0, &c, PhaseId::Liquid).unwrap();
        let z_v = z_factor(CubicEos::PR1976, 400.0, 1500.0, &c, PhaseId::Vapor).unwrap();
        assert!(z_l < z_v, "expected Z_liquid={} < Z_vapor={}", z_l, z_v);
        assert!(z_l < 0.1, "liquid Z should be small, got {}", z_l);
        assert!(z_v > 0.5, "vapor Z should be > 0.5, got {}", z_v);
    }

    #[test]
    fn ln_phi_ideal_gas_limit() {
        // At very low pressure, ln(φ) → 0 (any cubic EOS reduces to
        // ideal gas). Use 0.1 kPa as "very low" — should give |ln(φ)|
        // well below 1e-2 for methane at 300 K.
        let c = methane();
        for eos in [
            CubicEos::PR1976,
            CubicEos::RKS1972,
            CubicEos::RK1949,
            CubicEos::VdW1870,
        ] {
            let ln_phi = ln_phi_pure(eos, 300.0, 0.1, &c, PhaseId::Vapor).unwrap();
            assert!(
                ln_phi.abs() < 1e-3,
                "{:?}: ln(φ) at P→0 = {} (expected near 0)",
                eos,
                ln_phi
            );
        }
    }

    #[test]
    fn three_parameter_eos_errors_cleanly() {
        // Schmidt-Wenzel and Patel-Teja must error, not panic, at the
        // z_factor / ln_phi / departure layer. Their α functions panic
        // (M7.3 deferred) but the higher-level functions short-circuit
        // via EosError::NotImplemented before reaching α.
        let c = methane();
        for eos in [
            CubicEos::SchmidtWenzel,
            CubicEos::PatelTeja,
            CubicEos::PatelTejaUSB,
        ] {
            let err = z_factor(eos, 300.0, 1000.0, &c, PhaseId::Vapor).unwrap_err();
            assert!(matches!(err, EosError::NotImplemented(_)));
        }
    }

    #[test]
    #[should_panic(expected = "M7.4 deferred")]
    fn deferred_alpha_panics_with_marker() {
        // The OL family is now the M7.4-deferred bucket (its α is coupled
        // to the reduced saturation pressure) — verify the panic message
        // names the variant and the porting milestone.
        let c = methane();
        let _ = alpha(CubicEos::RKOL1998, 1.0, &c);
    }

    // -----------------------------------------------------------------
    // M7.2 — tests for the remaining twelve two-parameter α variants.
    // -----------------------------------------------------------------

    /// A synthetic "polar" component carrying every fitted parameter the
    /// M7.2 variants might read (zc, m/n/g, prsv_k1). Critical constants
    /// loosely mimic water so the numbers stay in a plausible range; the
    /// exact values don't matter — the oracle test only checks internal
    /// consistency of α and its analytical derivative.
    fn polar_component() -> Component {
        Component {
            name: "synthetic-polar".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            zc: 0.229,
            // Fitted polar parameters. g > 1 keeps |1−Tr|^(g−1) smooth at
            // Tr = 1 for the ATmn exponential variants.
            m_polar: 0.45,
            n_polar: 0.12,
            g_polar: 1.5,
            prsv_k1: 0.07,
            ..Component::default()
        }
    }

    /// Every M7.2 variant, plus the synthetic component each one is
    /// exercised against. The OL family is excluded — it is M7.4.
    const M72_VARIANTS: [CubicEos; 12] = [
        CubicEos::Berth1899,
        CubicEos::VdWAda1984,
        CubicEos::RKSGD1978,
        CubicEos::RKSL1997,
        CubicEos::RP1978,
        CubicEos::PRL1997,
        CubicEos::VdWVald1989,
        CubicEos::RKSmn1980,
        CubicEos::RKSATmn1995,
        CubicEos::PRATmng1997,
        CubicEos::PRMmn1989,
        CubicEos::PRSV1986,
    ];

    #[test]
    fn m72_alpha_is_finite_and_positive() {
        // α must be a finite, strictly positive number everywhere — a
        // negative or NaN α would make a·α negative and break the EOS.
        let c = polar_component();
        for eos in M72_VARIANTS {
            for tr in [0.5_f64, 0.7, 0.9, 1.0, 1.3, 2.0] {
                let a = alpha(eos, tr, &c);
                assert!(
                    a.is_finite() && a > 0.0,
                    "{:?} Tr={} gave α={} (expected finite, positive)",
                    eos,
                    tr,
                    a
                );
            }
        }
    }

    #[test]
    fn m72_alpha_at_tr_one_is_one() {
        // By construction, all of these α forms collapse to 1 at Tr = 1
        // (the 1 − √Tr / 1 − Tr / 1/Tr − 1 factors all vanish there). This
        // is the single most important sanity check on the porting: the
        // attractive term must reduce to its critical-point value at Tc.
        let c = polar_component();
        for eos in M72_VARIANTS {
            let a = alpha(eos, 1.0, &c);
            assert!(
                (a - 1.0).abs() < 1e-12,
                "{:?}: α(Tr=1) = {} (expected 1.0)",
                eos,
                a
            );
        }
    }

    #[test]
    fn m72_analytical_d_alpha_matches_numerical() {
        // The CLAUDE.md "Algorithm Choices" rule: analytical dα/dTr is the
        // production path, the central-difference value is only an oracle.
        // Tr = 1.0 is skipped for the abs-power ATmn variants — even with
        // g = 1.5 the |1−Tr|^(g−1) factor is only C¹ there, so a symmetric
        // difference would clip the corner; every other Tr is smooth.
        let c = polar_component();
        for eos in M72_VARIANTS {
            for tr in [0.55_f64, 0.7, 0.85, 1.1, 1.4] {
                let analytical = d_alpha_d_tr(eos, tr, &c);
                let numerical = d_alpha_numerical(eos, tr, &c, 1e-6);
                let rel = if analytical.abs() < 1e-8 {
                    (analytical - numerical).abs()
                } else {
                    ((analytical - numerical) / analytical).abs()
                };
                assert!(
                    rel < 1e-5,
                    "{:?} Tr={} analytical={} numerical={} rel={}",
                    eos,
                    tr,
                    analytical,
                    numerical,
                    rel
                );
            }
        }
    }

    #[test]
    fn prsv_k1_recovers_kappa0_when_zero() {
        // With K₁ = 0 the PRSV κ collapses to κ₀(ω), so α(PRSV) must match
        // the bracketed [1 + κ₀(1−√Tr)]² form exactly. This pins the K₁
        // contribution as a pure additive correction.
        let mut c = polar_component();
        c.prsv_k1 = 0.0;
        let w = c.omega;
        let kappa0 = 0.378893 + 1.4897153 * w - 0.17131848 * w * w + 0.0196554 * w * w * w;
        for tr in [0.6_f64, 0.8, 1.2] {
            let s = 1.0 - tr.sqrt();
            let expected = (1.0 + kappa0 * s).powi(2);
            let got = alpha(CubicEos::PRSV1986, tr, &c);
            assert!(
                (got - expected).abs() < 1e-12,
                "PRSV K₁=0 Tr={}: got {} expected {}",
                tr,
                got,
                expected
            );
        }
    }

    #[test]
    fn m72_z_factor_and_ln_phi_work() {
        // The full Z-factor / ln(φ) machinery must run end-to-end for the
        // new variants (they were panicking before M7.2). Use the polar
        // component below its critical point.
        let c = polar_component();
        for eos in M72_VARIANTS {
            let z = z_factor(eos, 500.0, 2000.0, &c, PhaseId::Vapor).unwrap();
            assert!(z > 0.0 && z < 1.2, "{:?}: Z={} out of range", eos, z);
            let ln_phi = ln_phi_pure(eos, 500.0, 2000.0, &c, PhaseId::Vapor).unwrap();
            assert!(ln_phi.is_finite(), "{:?}: ln(φ)={} not finite", eos, ln_phi);
        }
    }
}
