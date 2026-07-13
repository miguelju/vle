//! The persistent mixture-VLE handle exposed to JavaScript.
//!
//! [`VleSystem`] is the wasm analog of the Python `vle.System` and the
//! UniFFI `VleSystem` (Swift/Kotlin): build it **once** with the component
//! list and model selections, then call flash methods on it repeatedly.
//! `#[wasm_bindgen]` surfaces it in JS as a **class** backed by wasm
//! memory — the generated glue frees the Rust side automatically when the
//! JS object is garbage-collected (via `FinalizationRegistry`), and a
//! `.free()` method exists for deterministic cleanup.
//!
//! ## Model selection from JavaScript
//!
//! Model choices arrive as **strings or plain objects**, closest to the
//! Python API's ergonomics:
//!
//! ```js
//! // Strings: an EOS name means a cubic phase; activity names work for
//! // the liquid; "idealGas" / "virial" / "idealSolution" / "chaoSeader".
//! const sys = VleSystem.fromDb(["n-heptane", "n-butane"], "RKS1972", "RKS1972");
//!
//! // Objects: the explicit form, mirroring the Swift/Kotlin enums.
//! const sys2 = VleSystem.fromDb(
//!   ["methanol", "water"],
//!   { kind: "idealGas" },
//!   { kind: "activity", model: "vanLaar" },
//!   "classical",
//!   { aij: [[0, 0.847], [0.522, 0]] },
//! );
//! ```
//!
//! Names are case-insensitive and separator-blind (`"van-laar"` ==
//! `"vanLaar"` == `"VANLAAR"`). The full catalogue is in the parse
//! functions below — one `match` arm per engine variant, so the compiler
//! flags any new engine model that lacks a JS name.
//!
//! ## Units
//!
//! Everything is **canonical engine units**: temperature **K**, pressure
//! **kPa absolute**, compositions as mole fractions summing to 1.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::component::ComponentData;
use crate::error::VleWasmError;
use vle_thermo::activity as act;
use vle_thermo::eos;
use vle_thermo::flash::bubble::{bubble_pressure, bubble_temperature};
use vle_thermo::flash::dew::{dew_pressure, dew_temperature};
use vle_thermo::flash::isothermal::flash_isothermal_warm;
use vle_thermo::flash::{SystemSpec, k_values};
use vle_thermo::mixing;
use vle_thermo::types::Component;

// Solver defaults, matching the Python bindings' keyword defaults and the
// UniFFI wrapper. v1 keeps them fixed; expose as arguments if an app ever
// needs looser tolerances.
const SAT_TOL: f64 = 1e-9;
const FLASH_TOL: f64 = 1e-10;
const MAX_ITER: usize = 200;

/// Lowercase a model name and drop `-`, `_`, and spaces, so users can
/// write `"van-laar"`, `"van_laar"`, or `"vanLaar"` interchangeably.
fn normalize(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '-' | '_' | ' '))
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Parse a cubic-EOS name (mirrors `vle_thermo::CubicEos`).
///
/// Accepts the engine variant names (`"PR1976"`, `"RKS1972"`,
/// `"SchmidtWenzel"`, …) plus the obvious short aliases (`"PR"`, `"RKS"`,
/// `"vdW"`, `"PRSV"`). See the engine docs for the literature reference
/// behind each variant.
pub(crate) fn parse_eos(name: &str) -> Result<eos::CubicEos, VleWasmError> {
    use eos::CubicEos::*;
    Ok(match normalize(name).as_str() {
        "pr1976" | "pr" => PR1976,
        "rk1949" | "rk" => RK1949,
        "rks1972" | "rks" | "srk" => RKS1972,
        "vdw1870" | "vdw" => VdW1870,
        "prl1997" => PRL1997,
        "rksl1997" => RKSL1997,
        "rksgd1978" => RKSGD1978,
        "rp1978" => RP1978,
        "berth1899" | "berthelot" => Berth1899,
        "vdwada1984" => VdWAda1984,
        "vdwvald1989" => VdWVald1989,
        "rksmn1980" => RKSmn1980,
        "rksatmn1995" => RKSATmn1995,
        "pratmng1997" => PRATmng1997,
        "prmmn1989" => PRMmn1989,
        "prsv1986" | "prsv" => PRSV1986,
        "vdwol1998" => VdWOL1998,
        "rkol1998" => RKOL1998,
        "prol1998" => PROL1998,
        "schmidtwenzel" | "sw" => SchmidtWenzel,
        "patelteja" | "pt" => PatelTeja,
        "pateltejausb" | "ptusb" => PatelTejaUSB,
        _ => {
            return Err(VleWasmError::InvalidInput {
                message: format!("unknown cubic EOS {name:?} (try \"PR1976\", \"RKS1972\", …)"),
            });
        }
    })
}

/// Parse an activity-model name (mirrors `vle_thermo::ActivityModel`).
pub(crate) fn parse_activity(name: &str) -> Result<act::ActivityModel, VleWasmError> {
    use act::ActivityModel::*;
    Ok(match normalize(name).as_str() {
        "idealsolution" | "ideal" => IdealSolution,
        "vanlaar" => VanLaar,
        "wilson" => Wilson,
        "scatchardhildebrand" | "scatchard" => ScatchardHildebrand,
        "margules" => Margules,
        "nrtl" => Nrtl,
        _ => {
            return Err(VleWasmError::InvalidInput {
                message: format!(
                    "unknown activity model {name:?} (try \"NRTL\", \"Wilson\", \"vanLaar\", …)"
                ),
            });
        }
    })
}

/// Parse a mixing-rule name (mirrors `vle_thermo::MixingRule`).
pub(crate) fn parse_mixing(name: &str) -> Result<mixing::MixingRule, VleWasmError> {
    use mixing::MixingRule::*;
    Ok(match normalize(name).as_str() {
        "classical" => Classical,
        "wongsandler" | "ws" => WongSandler,
        "huronvidaloriginal" | "huronvidal" | "hv" => HuronVidalOriginal,
        "huronvidalsimplified" | "hvs" => HuronVidalSimplified,
        "mhv1" => MHV1,
        "mhv2" => MHV2,
        "ivdw" => IVDW,
        "iivdw" => IIVDW,
        "pateltejac" => PatelTejaC,
        "pateltejausbc" => PatelTejaUSBC,
        "schmidtwenzelc" => SchmidtWenzelC,
        _ => {
            return Err(VleWasmError::InvalidInput {
                message: format!(
                    "unknown mixing rule {name:?} (try \"classical\", \"wongSandler\", \"MHV1\", …)"
                ),
            });
        }
    })
}

/// Vapor-phase model selection, as it arrives from JavaScript: either a
/// bare string (`"idealGas"`, `"virial"`, or a cubic-EOS name) or the
/// explicit tagged object (`{ kind: "cubic", eos: "PR1976" }`).
///
/// `#[serde(untagged)]` tries each representation in order — the standard
/// serde idiom for "this JSON value has two accepted shapes".
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VaporIn {
    /// Bare string form.
    Name(String),
    /// Explicit object form.
    Spec(VaporSpecIn),
}

/// The explicit vapor object form (mirrors the UniFFI `VaporSpec` enum).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum VaporSpecIn {
    /// Ideal gas (φᵢ = 1). Low pressures only.
    IdealGas,
    /// Second virial equation (Pitzer B⁰/B¹). Moderate pressures.
    Virial,
    /// Cubic EOS (φ-φ when the liquid is also cubic).
    Cubic { eos: String },
}

impl VaporIn {
    /// Resolve to the engine's `VaporModel`.
    pub fn resolve(&self) -> Result<eos::VaporModel, VleWasmError> {
        Ok(match self {
            VaporIn::Name(s) => match normalize(s).as_str() {
                "idealgas" | "ideal" => eos::VaporModel::IdealGas,
                "virial" => eos::VaporModel::Virial,
                _ => eos::VaporModel::Cubic(parse_eos(s)?),
            },
            VaporIn::Spec(VaporSpecIn::IdealGas) => eos::VaporModel::IdealGas,
            VaporIn::Spec(VaporSpecIn::Virial) => eos::VaporModel::Virial,
            VaporIn::Spec(VaporSpecIn::Cubic { eos: e }) => eos::VaporModel::Cubic(parse_eos(e)?),
        })
    }
}

/// Liquid-phase model selection from JavaScript: a bare string
/// (`"idealSolution"`, `"chaoSeader"`, an activity-model name, or a
/// cubic-EOS name) or the explicit tagged object.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LiquidIn {
    /// Bare string form.
    Name(String),
    /// Explicit object form.
    Spec(LiquidSpecIn),
}

/// The explicit liquid object form (mirrors the UniFFI `LiquidSpec` enum).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LiquidSpecIn {
    /// Ideal solution — Raoult's law (γᵢ = 1).
    IdealSolution,
    /// Cubic EOS liquid (φ-φ approach).
    Cubic { eos: String },
    /// Activity-coefficient liquid (γ-φ approach).
    Activity { model: String },
    /// Chao-Seader liquid fugacity correlation. Ref (4), TERMOII.PAS.
    ChaoSeader,
}

impl LiquidIn {
    /// Resolve to the engine's `LiquidModel`. For the bare-string form the
    /// name is tried as ideal / Chao-Seader, then as an activity model,
    /// then as a cubic EOS — the three namespaces don't overlap.
    pub fn resolve(&self) -> Result<eos::LiquidModel, VleWasmError> {
        Ok(match self {
            LiquidIn::Name(s) => match normalize(s).as_str() {
                "idealsolution" | "ideal" => eos::LiquidModel::IdealSolution,
                "chaoseader" => eos::LiquidModel::ChaoSeader,
                _ => {
                    if let Ok(m) = parse_activity(s) {
                        eos::LiquidModel::Activity(m)
                    } else if let Ok(e) = parse_eos(s) {
                        eos::LiquidModel::Cubic(e)
                    } else {
                        return Err(VleWasmError::InvalidInput {
                            message: format!(
                                "unknown liquid model {s:?} — not an activity model or cubic EOS"
                            ),
                        });
                    }
                }
            },
            LiquidIn::Spec(LiquidSpecIn::IdealSolution) => eos::LiquidModel::IdealSolution,
            LiquidIn::Spec(LiquidSpecIn::Cubic { eos: e }) => {
                eos::LiquidModel::Cubic(parse_eos(e)?)
            }
            LiquidIn::Spec(LiquidSpecIn::Activity { model }) => {
                eos::LiquidModel::Activity(parse_activity(model)?)
            }
            LiquidIn::Spec(LiquidSpecIn::ChaoSeader) => eos::LiquidModel::ChaoSeader,
        })
    }
}

/// Optional binary-interaction data and GE coupling for a [`VleSystem`],
/// as a plain JS object — every field may be omitted.
///
/// The **empty array is the "not used" sentinel** throughout, mirroring
/// the engine convention: an empty `kij` means all-zero interactions, an
/// empty `vl` disables the Poynting correction, and so on.
///
/// Units: `aij` follows the selected activity model's convention (see the
/// engine's `activity` docs; e.g. NRTL energies in **kJ/kmol**), `vl` in
/// **cm³/mol**, `delta` in **(cal/cm³)^0.5**; `kij` and `alpha` are
/// dimensionless.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SystemOptionsIn {
    /// kij matrix (N×N) for cubic phases; empty ⇒ all-zero.
    pub kij: Vec<Vec<f64>>,
    /// Activity binary-parameter matrix (N×N); empty ⇒ all-zero.
    pub aij: Vec<Vec<f64>>,
    /// NRTL non-randomness matrix αᵢⱼ (N×N, symmetric); empty ⇒ ignored.
    pub alpha: Vec<Vec<f64>>,
    /// Liquid molar volumes override, **cm³/mol** (Wilson / Scatchard /
    /// Poynting). Empty ⇒ taken from each component's `liquidVolume`.
    pub vl: Vec<f64>,
    /// Solubility-parameter override, **(cal/cm³)^0.5** (Scatchard).
    /// Empty ⇒ taken from each component's `solubilityParam`.
    pub delta: Vec<f64>,
    /// Activity model coupled into a GE-based mixing rule (Wong-Sandler,
    /// Huron-Vidal, MHV1/2), by name. Omit for classical mixing.
    pub ge_model: Option<String>,
}

/// Result of an isothermal (PT) flash, returned as a plain JS object.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
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

/// Result of a bubble- or dew-point solve, returned as a plain JS object.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaturationPoint {
    /// The solved-for variable: pressure in **kPa absolute** for the
    /// `*P` methods, temperature in **K** for `*T`.
    pub value: f64,
    /// Incipient-phase composition (vapor `y` at a bubble point, liquid
    /// `x` at a dew point); length N, sums to 1.
    pub incipient: Vec<f64>,
    /// Converged equilibrium ratios Kᵢ.
    pub k: Vec<f64>,
}

/// The JsValue-free core of [`VleSystem`]: plain Rust, so the host test
/// suite (`cargo test -p vle-wasm`) exercises construction, validation,
/// and every flash path without a JS runtime. The `#[wasm_bindgen]` shim
/// below is a thin JsValue⇄record conversion layer over this.
pub struct SystemCore {
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

impl SystemCore {
    /// Shared construction path for both constructors. Validation is
    /// identical to the UniFFI wrapper's `VleSystem::build`.
    pub fn build(
        components: Vec<Component>,
        vapor: &VaporIn,
        liquid: &LiquidIn,
        mixing_rule: Option<&str>,
        opts: SystemOptionsIn,
    ) -> Result<Self, VleWasmError> {
        let n = components.len();
        if n == 0 {
            return Err(VleWasmError::InvalidInput {
                message: "a system needs at least one component".into(),
            });
        }

        // Matrices must be empty (the "unused" sentinel) or square N×N.
        for (label, m) in [
            ("kij", &opts.kij),
            ("aij", &opts.aij),
            ("alpha", &opts.alpha),
        ] {
            if !m.is_empty() && (m.len() != n || m.iter().any(|row| row.len() != n)) {
                return Err(VleWasmError::InvalidInput {
                    message: format!("{label} must be empty or an {n}x{n} matrix"),
                });
            }
        }
        for (label, v) in [("vl", &opts.vl), ("delta", &opts.delta)] {
            if !v.is_empty() && v.len() != n {
                return Err(VleWasmError::InvalidInput {
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

        Ok(SystemCore {
            components,
            vapor: vapor.resolve()?,
            liquid: liquid.resolve()?,
            mixing_rule: parse_mixing(mixing_rule.unwrap_or("classical"))?,
            kij: opts.kij,
            aij: opts.aij,
            alpha: opts.alpha,
            vl,
            delta,
            ge_model: opts.ge_model.as_deref().map(parse_activity).transpose()?,
        })
    }

    /// Build from bundled-database names (case-insensitive lookup).
    pub fn from_db(
        names: &[String],
        vapor: &VaporIn,
        liquid: &LiquidIn,
        mixing_rule: Option<&str>,
        opts: SystemOptionsIn,
    ) -> Result<Self, VleWasmError> {
        let components: Vec<Component> = names
            .iter()
            .map(|name| {
                vle_thermo::db::component(name)
                    .ok_or_else(|| VleWasmError::NotFound { name: name.clone() })
            })
            .collect::<Result<_, _>>()?;
        Self::build(components, vapor, liquid, mixing_rule, opts)
    }

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

    fn check_width(&self, what: &str, len: usize) -> Result<(), VleWasmError> {
        if len != self.components.len() {
            return Err(VleWasmError::InvalidInput {
                message: format!(
                    "{what} has {len} entries but the system has {} components",
                    self.components.len()
                ),
            });
        }
        Ok(())
    }

    /// Component names, in system order.
    pub fn names(&self) -> Vec<String> {
        self.components.iter().map(|c| c.name.clone()).collect()
    }

    /// Number of components.
    pub fn n(&self) -> usize {
        self.components.len()
    }

    /// Equilibrium ratios Kᵢ = yᵢ/xᵢ at a trial state.
    /// `t` **K**, `p` **kPa absolute**, `x`/`y` mole fractions (length N).
    pub fn k_values(&self, t: f64, p: f64, x: &[f64], y: &[f64]) -> Result<Vec<f64>, VleWasmError> {
        self.check_width("x", x.len())?;
        self.check_width("y", y.len())?;
        Ok(k_values(&self.spec(), t, p, x, y)?)
    }

    /// Isothermal (PT/TP) flash of feed `z` at `t` **K**, `p` **kPa abs**.
    pub fn flash_tp(&self, t: f64, p: f64, z: &[f64]) -> Result<FlashSplit, VleWasmError> {
        self.check_width("z", z.len())?;
        let r = flash_isothermal_warm(&self.spec(), t, p, z, None, FLASH_TOL, MAX_ITER)?;
        Ok(FlashSplit {
            beta: r.beta,
            x: r.x,
            y: r.y,
            k: r.k,
            iterations: r.iterations as u32,
            two_phase: r.two_phase,
        })
    }

    /// Bubble-point **pressure** (kPa abs) at `t` **K** for liquid `x`.
    pub fn bubble_p(&self, t: f64, x: &[f64]) -> Result<SaturationPoint, VleWasmError> {
        self.check_width("x", x.len())?;
        let r = bubble_pressure(&self.spec(), t, x, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }

    /// Bubble-point **temperature** (K) at `p` **kPa abs** for liquid `x`.
    pub fn bubble_t(&self, p: f64, x: &[f64]) -> Result<SaturationPoint, VleWasmError> {
        self.check_width("x", x.len())?;
        let r = bubble_temperature(&self.spec(), p, x, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }

    /// Dew-point **pressure** (kPa abs) at `t` **K** for vapor `y`.
    pub fn dew_p(&self, t: f64, y: &[f64]) -> Result<SaturationPoint, VleWasmError> {
        self.check_width("y", y.len())?;
        let r = dew_pressure(&self.spec(), t, y, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }

    /// Dew-point **temperature** (K) at `p` **kPa abs** for vapor `y`.
    pub fn dew_t(&self, p: f64, y: &[f64]) -> Result<SaturationPoint, VleWasmError> {
        self.check_width("y", y.len())?;
        let r = dew_temperature(&self.spec(), p, y, SAT_TOL, MAX_ITER)?;
        Ok(SaturationPoint {
            value: r.value,
            incipient: r.incipient,
            k: r.k,
        })
    }
}

// ── The JavaScript-facing shim ─────────────────────────────────────────
// Everything below is JsValue conversion only; the logic lives in
// SystemCore. Shim helpers keep each exported method to a few lines.

/// Deserialize a JS value, mapping serde errors to `InvalidInput`.
fn from_js<T: for<'de> Deserialize<'de>>(v: JsValue, what: &str) -> Result<T, VleWasmError> {
    serde_wasm_bindgen::from_value(v).map_err(|e| VleWasmError::InvalidInput {
        message: format!("could not parse {what}: {e}"),
    })
}

/// Serialize a record into a plain JS object.
fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsError::new(&e.to_string()))
}

/// Parse the optional `options` argument (`undefined`/`null` ⇒ defaults).
fn opts_from_js(options: JsValue) -> Result<SystemOptionsIn, VleWasmError> {
    Ok(from_js::<Option<SystemOptionsIn>>(options, "options")?.unwrap_or_default())
}

/// Persistent VLE system: components + model selections, built once.
///
/// In JavaScript this is a class; construct it with
/// `new VleSystem(components, vapor, liquid, mixingRule?, options?)` or
/// `VleSystem.fromDb(names, vapor, liquid, mixingRule?, options?)`, then
/// call the flash methods. The object is immutable after construction.
#[wasm_bindgen]
pub struct VleSystem {
    core: SystemCore,
}

#[wasm_bindgen]
impl VleSystem {
    /// Build a system from explicit component objects (see
    /// [`ComponentData`] for fields, units, and defaults).
    ///
    /// `mixingRule` defaults to `"classical"`; `options` may be omitted
    /// for the no-interaction baseline (all kij = 0, no activity
    /// parameters).
    ///
    /// # Errors
    /// Throws `Error("invalid input: …")` on an empty component list,
    /// malformed `cpCoeffs`, unknown model names, or non-square option
    /// matrices.
    #[wasm_bindgen(constructor)]
    pub fn new(
        components: JsValue,
        vapor: JsValue,
        liquid: JsValue,
        mixing_rule: Option<String>,
        options: JsValue,
    ) -> Result<VleSystem, JsError> {
        let data: Vec<ComponentData> = from_js(components, "components")?;
        let components: Vec<Component> = data
            .into_iter()
            .map(Component::try_from)
            .collect::<Result<_, _>>()?;
        let core = SystemCore::build(
            components,
            &from_js::<VaporIn>(vapor, "vapor")?,
            &from_js::<LiquidIn>(liquid, "liquid")?,
            mixing_rule.as_deref(),
            opts_from_js(options)?,
        )?;
        Ok(VleSystem { core })
    }

    /// Build a system straight from the bundled component database.
    ///
    /// Convenience over the constructor: each name is looked up with
    /// [`crate::component::db_component`] semantics (case-insensitive).
    ///
    /// # Errors
    /// Throws `Error("component not found …")` for an unknown name, plus
    /// everything the constructor throws.
    #[wasm_bindgen(js_name = fromDb)]
    pub fn from_db(
        names: Vec<String>,
        vapor: JsValue,
        liquid: JsValue,
        mixing_rule: Option<String>,
        options: JsValue,
    ) -> Result<VleSystem, JsError> {
        let core = SystemCore::from_db(
            &names,
            &from_js::<VaporIn>(vapor, "vapor")?,
            &from_js::<LiquidIn>(liquid, "liquid")?,
            mixing_rule.as_deref(),
            opts_from_js(options)?,
        )?;
        Ok(VleSystem { core })
    }

    /// Number of components in the system.
    #[wasm_bindgen(js_name = nComponents)]
    pub fn n_components(&self) -> u32 {
        self.core.n() as u32
    }

    /// Component names, in system order.
    pub fn names(&self) -> Vec<String> {
        self.core.names()
    }

    /// Equilibrium ratios Kᵢ = yᵢ/xᵢ at a trial state, as a
    /// `Float64Array`. `t` **K**, `p` **kPa absolute**, `x`/`y` mole
    /// fractions (length N).
    ///
    /// # Errors
    /// Throws on length mismatch or if a fugacity evaluation fails.
    #[wasm_bindgen(js_name = kValues)]
    pub fn k_values(&self, t: f64, p: f64, x: &[f64], y: &[f64]) -> Result<Vec<f64>, JsError> {
        Ok(self.core.k_values(t, p, x, y)?)
    }

    /// Isothermal (PT/TP) flash of feed `z` at `t` **K**, `p` **kPa abs**.
    /// Returns a [`FlashSplit`] object.
    ///
    /// Wilson-initialized, stability-checked, GDEM-accelerated — see
    /// MODERNIZATION_PLAN "Algorithm Choices". Single-phase feeds return
    /// β clamped to 0 or 1 with `twoPhase: false`.
    ///
    /// # Errors
    /// Throws on a wrong-length `z` or convergence failure.
    #[wasm_bindgen(js_name = flashTp)]
    pub fn flash_tp(&self, t: f64, p: f64, z: &[f64]) -> Result<JsValue, JsError> {
        to_js(&self.core.flash_tp(t, p, z)?)
    }

    /// Bubble-point **pressure** at `t` **K** for liquid composition `x`.
    /// Returns a [`SaturationPoint`] with the pressure in **kPa absolute**
    /// (in `value`) plus the incipient vapor composition.
    ///
    /// # Errors
    /// Throws as in [`Self::flash_tp`].
    #[wasm_bindgen(js_name = bubbleP)]
    pub fn bubble_p(&self, t: f64, x: &[f64]) -> Result<JsValue, JsError> {
        to_js(&self.core.bubble_p(t, x)?)
    }

    /// Bubble-point **temperature** at `p` **kPa absolute** for liquid
    /// `x`. Returns the temperature in **K** (in `value`).
    ///
    /// # Errors
    /// Throws as in [`Self::bubble_p`].
    #[wasm_bindgen(js_name = bubbleT)]
    pub fn bubble_t(&self, p: f64, x: &[f64]) -> Result<JsValue, JsError> {
        to_js(&self.core.bubble_t(p, x)?)
    }

    /// Dew-point **pressure** at `t` **K** for vapor composition `y`.
    /// Returns the pressure in **kPa absolute** (in `value`) plus the
    /// incipient liquid composition.
    ///
    /// # Errors
    /// Throws as in [`Self::bubble_p`].
    #[wasm_bindgen(js_name = dewP)]
    pub fn dew_p(&self, t: f64, y: &[f64]) -> Result<JsValue, JsError> {
        to_js(&self.core.dew_p(t, y)?)
    }

    /// Dew-point **temperature** at `p` **kPa absolute** for vapor `y`.
    /// Returns the temperature in **K** (in `value`).
    ///
    /// # Errors
    /// Throws as in [`Self::bubble_p`].
    #[wasm_bindgen(js_name = dewT)]
    pub fn dew_t(&self, p: f64, y: &[f64]) -> Result<JsValue, JsError> {
        to_js(&self.core.dew_t(p, y)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// n-heptane/n-butane RKS φ-φ system — the Chapter IV Table 4.10
    /// isothermal-flash configuration (300 K, 100 kPa), built through the
    /// bare-string model form.
    fn rks_heptane_butane() -> SystemCore {
        SystemCore::from_db(
            &["n-heptane".into(), "n-butane".into()],
            &VaporIn::Name("RKS1972".into()),
            &LiquidIn::Name("RKS1972".into()),
            None,
            SystemOptionsIn::default(),
        )
        .expect("both components are bundled")
    }

    #[test]
    fn isothermal_flash_splits_heptane_butane() {
        // Chapter IV validation case 7: equimolar n-heptane/n-butane at
        // 300 K, 100 kPa must split two-phase, with butane enriching the
        // vapor (K_butane > 1 > K_heptane).
        let sys = rks_heptane_butane();
        let r = sys.flash_tp(300.0, 100.0, &[0.5, 0.5]).unwrap();
        assert!(r.two_phase, "expected a two-phase split");
        assert!(r.beta > 0.0 && r.beta < 1.0, "beta = {}", r.beta);
        assert!(r.k[1] > 1.0 && r.k[0] < 1.0, "k = {:?}", r.k);
        // The wrapper must be bit-identical to calling the engine directly.
        let direct = flash_isothermal_warm(
            &sys.spec(),
            300.0,
            100.0,
            &[0.5, 0.5],
            None,
            FLASH_TOL,
            MAX_ITER,
        )
        .unwrap();
        assert_eq!(r.beta, direct.beta);
        assert_eq!(r.x, direct.x);
    }

    #[test]
    fn bubble_and_dew_bracket_the_flash() {
        // At fixed T and composition, bubble P ≥ dew P, and any P between
        // them flashes two-phase — a physical-consistency sweep.
        let sys = rks_heptane_butane();
        let z = [0.5, 0.5];
        let bub = sys.bubble_p(300.0, &z).unwrap();
        let dew = sys.dew_p(300.0, &z).unwrap();
        assert!(
            bub.value > dew.value,
            "bubble P {} must exceed dew P {}",
            bub.value,
            dew.value
        );
        let mid = 0.5 * (bub.value + dew.value);
        let r = sys.flash_tp(300.0, mid, &z).unwrap();
        assert!(r.two_phase);
    }

    #[test]
    fn bubble_t_and_dew_t_are_consistent() {
        let sys = rks_heptane_butane();
        let z = [0.5, 0.5];
        let bub = sys.bubble_t(100.0, &z).unwrap();
        let dew = sys.dew_t(100.0, &z).unwrap();
        assert!(
            dew.value > bub.value,
            "dew T {} must exceed bubble T {}",
            dew.value,
            bub.value
        );
    }

    #[test]
    fn model_selection_accepts_both_shapes() {
        // The untagged serde enums must accept both the bare-string and
        // the tagged-object JSON shape (serde_json stands in for
        // serde-wasm-bindgen — same Deserialize impls).
        let s: VaporIn = serde_json::from_value(serde_json::json!("virial")).unwrap();
        assert!(matches!(s.resolve().unwrap(), eos::VaporModel::Virial));
        let o: VaporIn =
            serde_json::from_value(serde_json::json!({"kind": "cubic", "eos": "PR1976"})).unwrap();
        assert!(matches!(
            o.resolve().unwrap(),
            eos::VaporModel::Cubic(eos::CubicEos::PR1976)
        ));
        let l: LiquidIn =
            serde_json::from_value(serde_json::json!({"kind": "activity", "model": "van-laar"}))
                .unwrap();
        assert!(matches!(
            l.resolve().unwrap(),
            eos::LiquidModel::Activity(act::ActivityModel::VanLaar)
        ));
        // Bare liquid string resolves through the activity → EOS fallback.
        let l2: LiquidIn = serde_json::from_value(serde_json::json!("NRTL")).unwrap();
        assert!(matches!(
            l2.resolve().unwrap(),
            eos::LiquidModel::Activity(act::ActivityModel::Nrtl)
        ));
    }

    #[test]
    fn name_normalization_is_forgiving() {
        assert_eq!(
            parse_eos("rks-1972").unwrap(),
            parse_eos("RKS1972").unwrap()
        );
        assert_eq!(
            parse_activity("Van Laar").unwrap(),
            parse_activity("vanLaar").unwrap()
        );
        assert_eq!(
            parse_mixing("wong_sandler").unwrap(),
            parse_mixing("wongSandler").unwrap()
        );
    }

    #[test]
    fn dimension_mismatch_is_invalid_input() {
        let sys = rks_heptane_butane();
        assert!(matches!(
            sys.flash_tp(300.0, 100.0, &[1.0]),
            Err(VleWasmError::InvalidInput { .. })
        ));
    }

    #[test]
    fn unknown_db_name_is_not_found() {
        let r = SystemCore::from_db(
            &["water".into(), "unobtainium".into()],
            &VaporIn::Name("idealGas".into()),
            &LiquidIn::Name("ideal".into()),
            None,
            SystemOptionsIn::default(),
        );
        assert!(matches!(r, Err(VleWasmError::NotFound { .. })));
    }

    #[test]
    fn malformed_kij_is_rejected() {
        let opts = SystemOptionsIn {
            kij: vec![vec![0.0]], // 1x1 for a 2-component system
            ..Default::default()
        };
        let r = SystemCore::from_db(
            &["n-heptane".into(), "n-butane".into()],
            &VaporIn::Name("idealGas".into()),
            &LiquidIn::Name("ideal".into()),
            None,
            opts,
        );
        assert!(matches!(r, Err(VleWasmError::InvalidInput { .. })));
    }

    #[test]
    fn unknown_mixing_rule_is_rejected() {
        let r = SystemCore::from_db(
            &["water".into()],
            &VaporIn::Name("idealGas".into()),
            &LiquidIn::Name("ideal".into()),
            Some("geometric"),
            SystemOptionsIn::default(),
        );
        assert!(matches!(r, Err(VleWasmError::InvalidInput { .. })));
    }
}
