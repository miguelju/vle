//! The persistent mixture-VLE handle exposed to Swift.
//!
//! [`VleSystem`] is the FFI analog of the Python `vle.System` /
//! `vle._engine.System` pyclass: build it **once** with the component list
//! and model selections, then call flash methods on it repeatedly. UniFFI's
//! `#[derive(uniffi::Object)]` surfaces it in Swift as a **class** backed
//! by a Rust `Arc` — Swift holds a reference; Rust frees the memory when
//! the last reference drops. (Records, by contrast, are copied values.)
//!
//! The model-selection enums below *mirror* the engine's enums rather than
//! reusing them: the engine types carry non-FFI baggage (explicit
//! discriminants matching the VB6 constants, data-carrying variants), and
//! keeping the FFI mirror separate means the engine API can evolve without
//! breaking the Swift ABI. The `From` impls are exhaustive `match`es, so
//! the compiler flags any new engine variant that lacks an FFI mapping.
//!
//! ## Units
//!
//! Everything is **canonical engine units**: temperature **K**, pressure
//! **kPa absolute**, compositions as mole fractions summing to 1.

use std::sync::Arc;

use crate::component::ComponentData;
use crate::error::VleFfiError;
use vle_thermo::activity as act;
use vle_thermo::eos;
use vle_thermo::flash::bubble::{bubble_pressure, bubble_temperature};
use vle_thermo::flash::dew::{dew_pressure, dew_temperature};
use vle_thermo::flash::isothermal::flash_isothermal_warm;
use vle_thermo::flash::{SystemSpec, k_values};
use vle_thermo::mixing;
use vle_thermo::types::Component;

// Solver defaults, matching the Python bindings' keyword defaults
// (py_system.rs). v1 keeps them fixed; expose as arguments if an app ever
// needs looser tolerances.
const SAT_TOL: f64 = 1e-9;
const FLASH_TOL: f64 = 1e-10;
const MAX_ITER: usize = 200;

/// Cubic equation-of-state selector (mirrors `vle_thermo::CubicEos`).
/// See the engine docs for the literature reference behind each variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum CubicEosKind {
    /// Peng-Robinson (1976).
    Pr1976,
    /// Redlich-Kwong (1949).
    Rk1949,
    /// Soave-Redlich-Kwong (1972).
    Rks1972,
    /// van der Waals (1870).
    Vdw1870,
    /// Peng-Robinson-Lim (1997).
    Prl1997,
    /// RKS-Lim (1997).
    Rksl1997,
    /// RKS-Graboski-Daubert (1978).
    Rksgd1978,
    /// Redlich-Prausnitz (1978).
    Rp1978,
    /// Berthelot (1899).
    Berth1899,
    /// van der Waals-Adachi (1984).
    VdwAda1984,
    /// van der Waals-Valderrama (1989).
    VdwVald1989,
    /// RKS-Mathias-Naumann (1980).
    RksMn1980,
    /// RKS-Adachi-Tagawa-Mathias-Naumann (1995).
    RksAtMn1995,
    /// PR-Adachi-Tagawa-Mathias-Naumann-Gasem (1997).
    PrAtMnG1997,
    /// PR-Mathias-Massih-Naumann (1989).
    PrMMn1989,
    /// Peng-Robinson-Stryjek-Vera (1986).
    Prsv1986,
    /// van der Waals-OL (1998).
    VdwOl1998,
    /// Redlich-Kwong-OL (1998).
    RkOl1998,
    /// Peng-Robinson-OL (1998).
    PrOl1998,
    /// Schmidt-Wenzel 3-parameter. Ref (4), TERMOII.PAS.
    SchmidtWenzel,
    /// Patel-Teja 3-parameter. Ref (4), TERMOII.PAS.
    PatelTeja,
    /// Patel-Teja USB (√B-weighted C mixing). Ref (4), TERMOII.PAS.
    PatelTejaUsb,
}

impl From<CubicEosKind> for eos::CubicEos {
    fn from(k: CubicEosKind) -> Self {
        use CubicEosKind::*;
        match k {
            Pr1976 => eos::CubicEos::PR1976,
            Rk1949 => eos::CubicEos::RK1949,
            Rks1972 => eos::CubicEos::RKS1972,
            Vdw1870 => eos::CubicEos::VdW1870,
            Prl1997 => eos::CubicEos::PRL1997,
            Rksl1997 => eos::CubicEos::RKSL1997,
            Rksgd1978 => eos::CubicEos::RKSGD1978,
            Rp1978 => eos::CubicEos::RP1978,
            Berth1899 => eos::CubicEos::Berth1899,
            VdwAda1984 => eos::CubicEos::VdWAda1984,
            VdwVald1989 => eos::CubicEos::VdWVald1989,
            RksMn1980 => eos::CubicEos::RKSmn1980,
            RksAtMn1995 => eos::CubicEos::RKSATmn1995,
            PrAtMnG1997 => eos::CubicEos::PRATmng1997,
            PrMMn1989 => eos::CubicEos::PRMmn1989,
            Prsv1986 => eos::CubicEos::PRSV1986,
            VdwOl1998 => eos::CubicEos::VdWOL1998,
            RkOl1998 => eos::CubicEos::RKOL1998,
            PrOl1998 => eos::CubicEos::PROL1998,
            SchmidtWenzel => eos::CubicEos::SchmidtWenzel,
            PatelTeja => eos::CubicEos::PatelTeja,
            PatelTejaUsb => eos::CubicEos::PatelTejaUSB,
        }
    }
}

/// Activity-coefficient model selector (mirrors `vle_thermo::ActivityModel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ActivityModelKind {
    /// Ideal solution (γᵢ = 1) — the Raoult's-law baseline.
    IdealSolution,
    /// van Laar (A₁₂, A₂₁ in `aij`).
    VanLaar,
    /// Wilson (needs liquid molar volumes `vl`).
    Wilson,
    /// Scatchard-Hildebrand regular solutions (needs `vl` + `delta`).
    ScatchardHildebrand,
    /// Margules two-suffix (binary only; A₁₂, A₂₁ in `aij`).
    Margules,
    /// NRTL (Renon & Prausnitz; needs `aij` energies + `alpha` matrix).
    Nrtl,
}

impl From<ActivityModelKind> for act::ActivityModel {
    fn from(k: ActivityModelKind) -> Self {
        match k {
            ActivityModelKind::IdealSolution => act::ActivityModel::IdealSolution,
            ActivityModelKind::VanLaar => act::ActivityModel::VanLaar,
            ActivityModelKind::Wilson => act::ActivityModel::Wilson,
            ActivityModelKind::ScatchardHildebrand => act::ActivityModel::ScatchardHildebrand,
            ActivityModelKind::Margules => act::ActivityModel::Margules,
            ActivityModelKind::Nrtl => act::ActivityModel::Nrtl,
        }
    }
}

/// Mixing-rule selector for cubic phases (mirrors `vle_thermo::MixingRule`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum MixingRuleKind {
    /// Classical one-fluid quadratic (the default).
    Classical,
    /// Wong-Sandler GE-based rule (needs `ge_model`). Ref (21).
    WongSandler,
    /// Huron-Vidal original (needs `ge_model`).
    HuronVidalOriginal,
    /// Huron-Vidal simplified (needs `ge_model`).
    HuronVidalSimplified,
    /// Modified Huron-Vidal, first order (needs `ge_model`).
    Mhv1,
    /// Modified Huron-Vidal, second order (needs `ge_model`).
    Mhv2,
    /// Improved van der Waals one-fluid.
    Ivdw,
    /// Improved van der Waals two-fluid.
    Iivdw,
    /// Patel-Teja C-parameter mixing. Ref (4), TERMOII.PAS.
    PatelTejaC,
    /// Patel-Teja USB C-parameter mixing. Ref (4), TERMOII.PAS.
    PatelTejaUsbC,
    /// Schmidt-Wenzel C-parameter mixing. Ref (4), TERMOII.PAS.
    SchmidtWenzelC,
}

impl From<MixingRuleKind> for mixing::MixingRule {
    fn from(k: MixingRuleKind) -> Self {
        use MixingRuleKind::*;
        match k {
            Classical => mixing::MixingRule::Classical,
            WongSandler => mixing::MixingRule::WongSandler,
            HuronVidalOriginal => mixing::MixingRule::HuronVidalOriginal,
            HuronVidalSimplified => mixing::MixingRule::HuronVidalSimplified,
            Mhv1 => mixing::MixingRule::MHV1,
            Mhv2 => mixing::MixingRule::MHV2,
            Ivdw => mixing::MixingRule::IVDW,
            Iivdw => mixing::MixingRule::IIVDW,
            PatelTejaC => mixing::MixingRule::PatelTejaC,
            PatelTejaUsbC => mixing::MixingRule::PatelTejaUSBC,
            SchmidtWenzelC => mixing::MixingRule::SchmidtWenzelC,
        }
    }
}

/// Vapor-phase model selection. A UniFFI enum **with fields** becomes a
/// Swift enum with associated values: `.cubic(eos: .pr1976)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum VaporSpec {
    /// Ideal gas (φᵢ = 1). Low pressures only.
    IdealGas,
    /// Second virial equation (Pitzer B⁰/B¹). Moderate pressures.
    Virial,
    /// Cubic EOS (φ-φ when the liquid is also cubic).
    Cubic { eos: CubicEosKind },
}

impl From<VaporSpec> for eos::VaporModel {
    fn from(v: VaporSpec) -> Self {
        match v {
            VaporSpec::IdealGas => eos::VaporModel::IdealGas,
            VaporSpec::Virial => eos::VaporModel::Virial,
            VaporSpec::Cubic { eos: e } => eos::VaporModel::Cubic(e.into()),
        }
    }
}

/// Liquid-phase model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum LiquidSpec {
    /// Ideal solution — Raoult's law (γᵢ = 1).
    IdealSolution,
    /// Cubic EOS liquid (φ-φ approach).
    Cubic { eos: CubicEosKind },
    /// Activity-coefficient liquid (γ-φ approach).
    Activity { model: ActivityModelKind },
    /// Chao-Seader liquid fugacity correlation. Ref (4), TERMOII.PAS.
    ChaoSeader,
}

impl From<LiquidSpec> for eos::LiquidModel {
    fn from(l: LiquidSpec) -> Self {
        match l {
            LiquidSpec::IdealSolution => eos::LiquidModel::IdealSolution,
            LiquidSpec::Cubic { eos: e } => eos::LiquidModel::Cubic(e.into()),
            LiquidSpec::Activity { model } => eos::LiquidModel::Activity(model.into()),
            LiquidSpec::ChaoSeader => eos::LiquidModel::ChaoSeader,
        }
    }
}

/// Optional binary-interaction data and GE coupling for a [`VleSystem`].
///
/// The **empty vector is the "not used" sentinel** throughout, mirroring
/// the engine convention: an empty `kij` means all-zero interactions, an
/// empty `vl` disables the Poynting correction, and so on. Start from
/// [`default_system_options`] and set only what your model needs.
///
/// Units: `aij` follows the selected activity model's convention (see the
/// engine's `activity` docs; e.g. NRTL energies in **kJ/kmol**), `vl` in
/// **cm³/mol**, `delta` in **(cal/cm³)^0.5**; `kij` and `alpha` are
/// dimensionless.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SystemOptions {
    /// kij matrix (N×N) for cubic phases; empty ⇒ all-zero.
    pub kij: Vec<Vec<f64>>,
    /// Activity binary-parameter matrix (N×N); empty ⇒ all-zero.
    pub aij: Vec<Vec<f64>>,
    /// NRTL non-randomness matrix αᵢⱼ (N×N, symmetric); empty ⇒ ignored.
    pub alpha: Vec<Vec<f64>>,
    /// Liquid molar volumes override, **cm³/mol** (Wilson / Scatchard /
    /// Poynting). Empty ⇒ taken from each component's `liquid_volume`.
    pub vl: Vec<f64>,
    /// Solubility-parameter override, **(cal/cm³)^0.5** (Scatchard).
    /// Empty ⇒ taken from each component's `solubility_param`.
    pub delta: Vec<f64>,
    /// Activity model coupled into a GE-based mixing rule (Wong-Sandler,
    /// Huron-Vidal, MHV1/2). `nil` for classical mixing.
    pub ge_model: Option<ActivityModelKind>,
}

/// A [`SystemOptions`] with every field empty — the sensible baseline for
/// classical-mixing systems. (Swift: `defaultSystemOptions()`.)
#[uniffi::export]
pub fn default_system_options() -> SystemOptions {
    SystemOptions {
        kij: vec![],
        aij: vec![],
        alpha: vec![],
        vl: vec![],
        delta: vec![],
        ge_model: None,
    }
}

/// Result of an isothermal (PT) flash.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FlashSplit {
    /// Vapor fraction β = V/F, dimensionless, in `[0, 1]`.
    pub beta: f64,
    /// Liquid mole fractions xᵢ (length N, sums to 1).
    pub x: Vec<f64>,
    /// Vapor mole fractions yᵢ (length N, sums to 1).
    pub y: Vec<f64>,
    /// Converged equilibrium ratios Kᵢ = yᵢ/xᵢ.
    pub k: Vec<f64>,
    /// Outer-loop iterations taken.
    pub iterations: u32,
    /// `true` if the feed split into two phases; `false` if it resolved to
    /// a single phase (β clamped to 0 or 1).
    pub two_phase: bool,
}

/// Result of a bubble- or dew-point solve.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SaturationPoint {
    /// The solved-for variable: pressure in **kPa absolute** for the
    /// `*_pressure` methods, temperature in **K** for `*_temperature`.
    pub value: f64,
    /// Incipient-phase composition (vapor `y` at a bubble point, liquid
    /// `x` at a dew point); length N, sums to 1.
    pub incipient: Vec<f64>,
    /// Converged equilibrium ratios Kᵢ.
    pub k: Vec<f64>,
}

/// Persistent VLE system: components + model selections, built once.
///
/// In Swift this is a class; construct it with `VleSystem(components:…)`
/// or `VleSystem.fromDb(names:…)`, then call the flash methods. The object
/// is immutable after construction and safe to share across threads
/// (UniFFI requires `Send + Sync`, which this satisfies by owning plain
/// data and never mutating it).
#[derive(uniffi::Object)]
pub struct VleSystem {
    components: Vec<Component>,
    vapor: eos::VaporModel,
    liquid: eos::LiquidModel,
    mixing_rule: mixing::MixingRule,
    kij: Vec<Vec<f64>>,
    aij: Vec<Vec<f64>>,
    alpha: Vec<Vec<f64>>,
    /// Effective liquid molar volumes (cm³/mol): the options override or
    /// the per-component values; empty ⇒ Poynting off.
    vl: Vec<f64>,
    /// Effective solubility parameters ((cal/cm³)^0.5); empty ⇒ unused.
    delta: Vec<f64>,
    ge_model: Option<act::ActivityModel>,
}

impl VleSystem {
    /// Borrow the owned state into the slice-based [`SystemSpec`] the flash
    /// drivers consume — same pattern as the Python `System` pyclass.
    fn spec(&self) -> SystemSpec<'_> {
        SystemSpec {
            components: &self.components,
            vapor: self.vapor,
            liquid: self.liquid,
            mixing_rule: self.mixing_rule,
            kij: &self.kij,
            aij: &self.aij,
            alpha: &self.alpha,
            vl: &self.vl,
            delta: &self.delta,
            sat_models: &[],
            ge_model: self.ge_model,
        }
    }

    fn check_width(&self, what: &str, len: usize) -> Result<(), VleFfiError> {
        if len != self.components.len() {
            return Err(VleFfiError::InvalidInput {
                message: format!(
                    "{what} has {len} entries but the system has {} components",
                    self.components.len()
                ),
            });
        }
        Ok(())
    }

    /// Shared construction path for both exported constructors.
    fn build(
        components: Vec<Component>,
        vapor: VaporSpec,
        liquid: LiquidSpec,
        mixing_rule: MixingRuleKind,
        options: Option<SystemOptions>,
    ) -> Result<Arc<Self>, VleFfiError> {
        let n = components.len();
        if n == 0 {
            return Err(VleFfiError::InvalidInput {
                message: "a system needs at least one component".into(),
            });
        }
        let opts = options.unwrap_or_else(default_system_options);

        // Matrices must be empty (the "unused" sentinel) or square N×N.
        for (label, m) in [
            ("kij", &opts.kij),
            ("aij", &opts.aij),
            ("alpha", &opts.alpha),
        ] {
            if !m.is_empty() && (m.len() != n || m.iter().any(|row| row.len() != n)) {
                return Err(VleFfiError::InvalidInput {
                    message: format!("{label} must be empty or an {n}x{n} matrix"),
                });
            }
        }
        for (label, v) in [("vl", &opts.vl), ("delta", &opts.delta)] {
            if !v.is_empty() && v.len() != n {
                return Err(VleFfiError::InvalidInput {
                    message: format!("{label} must be empty or have {n} entries"),
                });
            }
        }

        // Effective vl/delta: explicit override wins; otherwise collect the
        // per-component fields, treating all-zero as "not available" so the
        // Poynting correction stays off for bare-bones components.
        let vl = if opts.vl.is_empty() {
            let from_components: Vec<f64> = components.iter().map(|c| c.liquid_volume).collect();
            if from_components.iter().all(|&v| v == 0.0) {
                vec![]
            } else {
                from_components
            }
        } else {
            opts.vl
        };
        let delta = if opts.delta.is_empty() {
            let from_components: Vec<f64> = components.iter().map(|c| c.solubility_param).collect();
            if from_components.iter().all(|&d| d == 0.0) {
                vec![]
            } else {
                from_components
            }
        } else {
            opts.delta
        };

        Ok(Arc::new(VleSystem {
            components,
            vapor: vapor.into(),
            liquid: liquid.into(),
            mixing_rule: mixing_rule.into(),
            kij: opts.kij,
            aij: opts.aij,
            alpha: opts.alpha,
            vl,
            delta,
            ge_model: opts.ge_model.map(Into::into),
        }))
    }
}

#[uniffi::export]
impl VleSystem {
    /// Build a system from explicit component data.
    ///
    /// `options` may be `nil` for the no-interaction baseline (all kij =
    /// 0, no activity parameters). See [`SystemOptions`] for units.
    ///
    /// # Errors
    /// [`VleFfiError::InvalidInput`] on an empty component list, malformed
    /// `cp_coeffs`, or non-square option matrices.
    #[uniffi::constructor]
    pub fn new(
        components: Vec<ComponentData>,
        vapor: VaporSpec,
        liquid: LiquidSpec,
        mixing_rule: MixingRuleKind,
        options: Option<SystemOptions>,
    ) -> Result<Arc<Self>, VleFfiError> {
        let components: Vec<Component> = components
            .into_iter()
            .map(Component::try_from)
            .collect::<Result<_, _>>()?;
        Self::build(components, vapor, liquid, mixing_rule, options)
    }

    /// Build a system straight from the bundled component database.
    ///
    /// Convenience over `new`: each name is looked up with
    /// [`crate::component::db_component`] semantics (case-insensitive).
    ///
    /// # Errors
    /// [`VleFfiError::NotFound`] for an unknown name;
    /// [`VleFfiError::InvalidInput`] as in `new`.
    #[uniffi::constructor]
    pub fn from_db(
        names: Vec<String>,
        vapor: VaporSpec,
        liquid: LiquidSpec,
        mixing_rule: MixingRuleKind,
        options: Option<SystemOptions>,
    ) -> Result<Arc<Self>, VleFfiError> {
        let components: Vec<Component> = names
            .into_iter()
            .map(|name| vle_thermo::db::component(&name).ok_or(VleFfiError::NotFound { name }))
            .collect::<Result<_, _>>()?;
        Self::build(components, vapor, liquid, mixing_rule, options)
    }

    /// Number of components in the system.
    pub fn n_components(&self) -> u32 {
        self.components.len() as u32
    }

    /// Component names, in system order.
    pub fn names(&self) -> Vec<String> {
        self.components.iter().map(|c| c.name.clone()).collect()
    }

    /// Equilibrium ratios Kᵢ = yᵢ/xᵢ at a trial state.
    /// `t` **K**, `p` **kPa absolute**, `x`/`y` mole fractions (length N).
    ///
    /// # Errors
    /// [`VleFfiError::InvalidInput`] on length mismatch;
    /// [`VleFfiError::Flash`] if a fugacity evaluation fails.
    pub fn k_values(
        &self,
        t: f64,
        p: f64,
        x: Vec<f64>,
        y: Vec<f64>,
    ) -> Result<Vec<f64>, VleFfiError> {
        Ok(k_values(&self.spec(), t, p, &x, &y)?)
    }

    /// Isothermal (PT/TP) flash of feed `z` at `t` **K**, `p` **kPa abs**.
    ///
    /// Wilson-initialized, stability-checked, GDEM-accelerated — see
    /// MODERNIZATION_PLAN "Algorithm Choices". Single-phase feeds return
    /// β clamped to 0 or 1 with `two_phase = false`.
    ///
    /// # Errors
    /// [`VleFfiError::InvalidInput`] on a wrong-length `z`;
    /// [`VleFfiError::Flash`] on convergence failure.
    pub fn flash_tp(&self, t: f64, p: f64, z: Vec<f64>) -> Result<FlashSplit, VleFfiError> {
        self.check_width("z", z.len())?;
        let r = flash_isothermal_warm(&self.spec(), t, p, &z, None, FLASH_TOL, MAX_ITER)?;
        Ok(FlashSplit {
            beta: r.beta,
            x: r.x,
            y: r.y,
            k: r.k,
            iterations: r.iterations as u32,
            two_phase: r.two_phase,
        })
    }

    /// Bubble-point **pressure** at `t` **K** for liquid composition `x`.
    /// Returns the pressure in **kPa absolute** (in `value`) plus the
    /// incipient vapor composition.
    ///
    /// # Errors
    /// [`VleFfiError::InvalidInput`] / [`VleFfiError::Flash`] as in
    /// [`Self::flash_tp`].
    pub fn bubble_p(&self, t: f64, x: Vec<f64>) -> Result<SaturationPoint, VleFfiError> {
        self.check_width("x", x.len())?;
        let r = bubble_pressure(&self.spec(), t, &x, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }

    /// Bubble-point **temperature** at `p` **kPa absolute** for liquid `x`.
    /// Returns the temperature in **K** (in `value`).
    ///
    /// # Errors
    /// As in [`Self::bubble_p`].
    pub fn bubble_t(&self, p: f64, x: Vec<f64>) -> Result<SaturationPoint, VleFfiError> {
        self.check_width("x", x.len())?;
        let r = bubble_temperature(&self.spec(), p, &x, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }

    /// Dew-point **pressure** at `t` **K** for vapor composition `y`.
    /// Returns the pressure in **kPa absolute** (in `value`) plus the
    /// incipient liquid composition.
    ///
    /// # Errors
    /// As in [`Self::bubble_p`].
    pub fn dew_p(&self, t: f64, y: Vec<f64>) -> Result<SaturationPoint, VleFfiError> {
        self.check_width("y", y.len())?;
        let r = dew_pressure(&self.spec(), t, &y, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }

    /// Dew-point **temperature** at `p` **kPa absolute** for vapor `y`.
    /// Returns the temperature in **K** (in `value`).
    ///
    /// # Errors
    /// As in [`Self::bubble_p`].
    pub fn dew_t(&self, p: f64, y: Vec<f64>) -> Result<SaturationPoint, VleFfiError> {
        self.check_width("y", y.len())?;
        let r = dew_temperature(&self.spec(), p, &y, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// n-heptane/n-butane RKS φ-φ system — the Chapter IV Table 4.10
    /// isothermal-flash configuration (300 K, 100 kPa).
    fn rks_heptane_butane() -> Arc<VleSystem> {
        VleSystem::from_db(
            vec!["n-heptane".into(), "n-butane".into()],
            VaporSpec::Cubic {
                eos: CubicEosKind::Rks1972,
            },
            LiquidSpec::Cubic {
                eos: CubicEosKind::Rks1972,
            },
            MixingRuleKind::Classical,
            None,
        )
        .expect("both components are bundled")
    }

    #[test]
    fn isothermal_flash_splits_heptane_butane() {
        // Chapter IV validation case 7: equimolar n-heptane/n-butane at
        // 300 K, 100 kPa must split two-phase, with butane enriching the
        // vapor (K_butane > 1 > K_heptane).
        let sys = rks_heptane_butane();
        let r = sys.flash_tp(300.0, 100.0, vec![0.5, 0.5]).unwrap();
        assert!(r.two_phase, "expected a two-phase split");
        assert!(r.beta > 0.0 && r.beta < 1.0, "beta = {}", r.beta);
        assert!(r.k[1] > 1.0 && r.k[0] < 1.0, "k = {:?}", r.k);
        // The wrapper must be bit-identical to calling the engine directly.
        let spec = sys.spec();
        let direct =
            flash_isothermal_warm(&spec, 300.0, 100.0, &[0.5, 0.5], None, FLASH_TOL, MAX_ITER)
                .unwrap();
        assert_eq!(r.beta, direct.beta);
        assert_eq!(r.x, direct.x);
    }

    #[test]
    fn bubble_and_dew_bracket_the_flash() {
        // At fixed T and composition, bubble P ≥ dew P, and any P between
        // them flashes two-phase — a physical-consistency sweep.
        let sys = rks_heptane_butane();
        let z = vec![0.5, 0.5];
        let bub = sys.bubble_p(300.0, z.clone()).unwrap();
        let dew = sys.dew_p(300.0, z.clone()).unwrap();
        assert!(
            bub.value > dew.value,
            "bubble P {} must exceed dew P {}",
            bub.value,
            dew.value
        );
        let mid = 0.5 * (bub.value + dew.value);
        let r = sys.flash_tp(300.0, mid, z).unwrap();
        assert!(r.two_phase);
    }

    #[test]
    fn bubble_t_and_dew_t_are_consistent() {
        let sys = rks_heptane_butane();
        let z = vec![0.5, 0.5];
        let bub = sys.bubble_t(100.0, z.clone()).unwrap();
        let dew = sys.dew_t(100.0, z).unwrap();
        assert!(
            dew.value > bub.value,
            "dew T {} must exceed bubble T {}",
            dew.value,
            bub.value
        );
    }

    /// A bare component: just Tc/Pc/ω + reduced-Antoine coefficients, no
    /// liquid volume — so the Poynting correction stays off and Raoult's
    /// law is exact. Same fits as the engine's flash::system tests.
    fn bare(name: &str, tc: f64, pc: f64, omega: f64, psat: Vec<f64>) -> ComponentData {
        ComponentData {
            name: name.into(),
            tc,
            pc,
            omega,
            psat_coeffs: psat,
            vc: 0.0,
            zc: 0.0,
            tb: 0.0,
            mw: 0.0,
            cp_coeffs: vec![],
            dipole_moment: 0.0,
            solubility_param: 0.0,
            liquid_volume: 0.0,
            zra: 0.0,
            omega_srk: 0.0,
            m_polar: 0.0,
            n_polar: 0.0,
            g_polar: 0.0,
            prsv_k1: 0.0,
            sat_model: crate::component::SatModel::Antoine,
        }
    }

    #[test]
    fn raoult_ideal_system_works_from_explicit_components() {
        // γ-φ ideal/ideal (pure Raoult): bubble P of an equimolar mixture
        // must sit between the two pure-component vapor pressures, and
        // with no liquid-volume data (Poynting off) it is exactly
        // Σ xᵢ·Psatᵢ.
        let comps = vec![
            bare("n-butane", 425.12, 3796.0, 0.200, vec![4.35, 2277.0, -30.0]),
            bare("n-heptane", 540.2, 2740.0, 0.350, vec![4.02, 2911.0, -56.0]),
        ];
        let sys = VleSystem::new(
            comps,
            VaporSpec::IdealGas,
            LiquidSpec::IdealSolution,
            MixingRuleKind::Classical,
            None,
        )
        .unwrap();
        let bub = sys.bubble_p(350.0, vec![0.5, 0.5]).unwrap();
        let p_butane = sys.bubble_p(350.0, vec![1.0, 0.0]).unwrap().value;
        let p_heptane = sys.bubble_p(350.0, vec![0.0, 1.0]).unwrap().value;
        assert!(
            p_heptane < bub.value && bub.value < p_butane,
            "Raoult: {} < {} < {} violated",
            p_heptane,
            bub.value,
            p_butane
        );
        // Raoult exactness: P_bubble = Σ xᵢ Psatᵢ.
        let expect = 0.5 * p_butane + 0.5 * p_heptane;
        assert!(
            (bub.value - expect).abs() / expect < 1e-6,
            "{} vs {}",
            bub.value,
            expect
        );
    }

    #[test]
    fn dimension_mismatch_is_invalid_input() {
        let sys = rks_heptane_butane();
        assert!(matches!(
            sys.flash_tp(300.0, 100.0, vec![1.0]),
            Err(VleFfiError::InvalidInput { .. })
        ));
    }

    #[test]
    fn unknown_db_name_is_not_found() {
        let r = VleSystem::from_db(
            vec!["water".into(), "unobtainium".into()],
            VaporSpec::IdealGas,
            LiquidSpec::IdealSolution,
            MixingRuleKind::Classical,
            None,
        );
        assert!(matches!(r, Err(VleFfiError::NotFound { .. })));
    }

    #[test]
    fn malformed_kij_is_rejected() {
        let opts = SystemOptions {
            kij: vec![vec![0.0]], // 1x1 for a 2-component system
            ..default_system_options()
        };
        let r = VleSystem::from_db(
            vec!["n-heptane".into(), "n-butane".into()],
            VaporSpec::IdealGas,
            LiquidSpec::IdealSolution,
            MixingRuleKind::Classical,
            Some(opts),
        );
        assert!(matches!(r, Err(VleFfiError::InvalidInput { .. })));
    }
}
