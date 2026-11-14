//! System specification + K-value computation — the bridge from the model
//! layers (EOS, activity, virial) to the flash drivers.
//!
//! A [`SystemSpec`] captures *everything* about a mixture's thermodynamic
//! model: the components, the vapor- and liquid-phase model choices, the
//! mixing rule + kij for EOS phases, and the activity parameters for the
//! γ-φ path. [`k_values`] turns that plus a `(T, P, x, y)` state into the
//! equilibrium ratios `Kᵢ = yᵢ/xᵢ` that every flash iterates on.
//!
//! See the module docs in [`super`] for the two thermodynamic paths.

use crate::activity::{ActivityModel, ln_gamma_all};
use crate::eos::{LiquidModel, PhaseId, VaporModel, ln_phi_pure};
use crate::mixing::MixingRule;
use crate::mixture::{
    GeSpec, MixtureSpec, ln_phi_mix_cached_into, ln_phi_mix_into, ln_phi_mix_min_gibbs_cached_into,
    ln_phi_mix_min_gibbs_into,
};
use crate::saturation::{SatPressureModel, ln_poynting_factor, psat};
use crate::types::Component;
use crate::virial::{ln_phi_mix_virial, ln_phi_pure_virial};

use super::FlashError;

/// Full thermodynamic-model specification of a mixture for flash work.
///
/// Borrows its data so a driver can build one per call cheaply. The empty
/// slice is the "not used / all-zero" sentinel for `kij`, `aij`, `vl`,
/// `delta`, and `sat_models`.
#[derive(Debug, Clone, Copy)]
pub struct SystemSpec<'a> {
    /// Component list.
    pub components: &'a [Component],
    /// Vapor-phase model (IdealGas / Virial / Cubic(eos)).
    pub vapor: VaporModel,
    /// Liquid-phase model (IdealSolution / Cubic(eos) / Activity(model) /
    /// ChaoSeader).
    pub liquid: LiquidModel,
    /// Mixing rule for any cubic phase.
    pub mixing_rule: MixingRule,
    /// kij matrix (N×N) for cubic phases; empty ⇒ all-zero.
    pub kij: &'a [Vec<f64>],
    /// Activity binary-parameter matrix (N×N) — used by the γ-φ liquid and
    /// by GE-based mixing rules; empty ⇒ all-zero.
    pub aij: &'a [Vec<f64>],
    /// NRTL non-randomness matrix αᵢⱼ (N×N, symmetric) — used **only** by the
    /// NRTL activity model; empty ⇒ ignored (every other model reads none).
    pub alpha: &'a [Vec<f64>],
    /// Liquid molar volumes Vᵢᴸ in **cm³/mol** (Wilson/Scatchard activity,
    /// Poynting correction); empty ⇒ Poynting disabled.
    pub vl: &'a [f64],
    /// Solubility parameters δᵢ in **(cal/cm³)^0.5** (Scatchard only).
    pub delta: &'a [f64],
    /// Per-component saturation model for the γ-φ Psat; empty ⇒ each
    /// component's own `sat_model` field.
    pub sat_models: &'a [SatPressureModel],
    /// Activity model coupled into a GE-based cubic mixing rule (WS, HV,
    /// MHV). `None` for classical mixing.
    pub ge_model: Option<crate::activity::ActivityModel>,
}

impl<'a> SystemSpec<'a> {
    /// Number of components.
    pub fn n(&self) -> usize {
        self.components.len()
    }

    /// Saturation model for component `i` (explicit override or the
    /// component's own field).
    fn sat_model(&self, i: usize) -> SatPressureModel {
        self.sat_models
            .get(i)
            .copied()
            .unwrap_or(self.components[i].sat_model)
    }

    /// Build the `GeSpec` for a GE-based cubic mixing rule, if configured.
    fn ge_spec(&self) -> Option<GeSpec<'a>> {
        self.ge_model.map(|model| GeSpec {
            model,
            aij: self.aij,
            alpha: self.alpha,
            vl: self.vl,
            delta: self.delta,
        })
    }

    /// `MixtureSpec` for a cubic phase using the given EOS. Exposed to the
    /// energy-based flash drivers (adiabatic, critical point) that need the
    /// mixture layer directly.
    pub(crate) fn mixture_spec(&self, eos: crate::eos::CubicEos) -> MixtureSpec<'a> {
        MixtureSpec {
            eos,
            rule: self.mixing_rule,
            components: self.components,
            kij: self.kij,
            ge: self.ge_spec(),
        }
    }
}

// ===========================================================================
// Per-(T, P) cache for a whole System — audit Part 2 §1, flash level.
// ===========================================================================

/// Everything about a [`SystemSpec`] at one `(T, P)` that does **not** depend
/// on composition.
///
/// The isothermal flash iterates composition at fixed `(T, P)`, so all of this
/// was being rebuilt on every outer iteration:
///
/// - both phases' pure-component EOS parameters (via [`crate::mixture::TpCache`]),
/// - on the γ-φ path, every component's `Psatᵢ(T)`, `φᵢˢᵃᵗ(T, Psatᵢ)` and
///   Poynting factor — three correlation evaluations per component per
///   iteration for numbers that change only with `(T, P)`,
/// - on the γ-φ path, the activity model's temperature-dependent matrices
///   (Wilson Λᵢⱼ, NRTL τᵢⱼ/Gᵢⱼ — audit Part 2 §5).
///
/// Crate-internal on purpose, exactly like `FlashWorkspace`: it is an
/// orchestration detail of the drivers, not something a caller should name.
/// The reusable piece with standalone value — [`crate::mixture::TpCache`] — is
/// public.
pub(crate) struct SystemTpCache {
    t: f64,
    p: f64,
    /// Pure-component EOS state for a cubic **liquid**, if the liquid is cubic.
    liquid: Option<crate::mixture::TpCache>,
    /// Pure-component EOS state for a cubic **vapor**, if the vapor is cubic.
    /// Separate from `liquid` because the two phases may use different EOS.
    vapor: Option<crate::mixture::TpCache>,
    /// γ-φ only: the entire composition-independent part of `ln Kᵢ`, i.e.
    /// `ln Psatᵢ + ln φᵢˢᵃᵗ + ln POYᵢ − ln P`. What is left at iteration time is
    /// `ln Kᵢ = ln γᵢ(x) + const_i − ln φ̂ᵢⱽ(y)`.
    gamma_phi_const: smallvec::SmallVec<[f64; 8]>,
    /// γ-φ only: the activity model's T-dependent matrices.
    activity: Option<crate::activity::ActivityTpCache>,
    /// Virial vapor only: the flat Bᵢⱼ matrix, which depends on T alone
    /// (audit Part 2 §9).
    virial_b: Option<Vec<f64>>,
    /// Refinery K-value methods (M20), the composition-independent part of
    /// `ln Kᵢ`: Grayson–Streed's `ln νᵢ`, or Braun K10's `ln Pᵢᴹᴮ − ln P`.
    /// Empty for every other liquid model.
    refinery_const: smallvec::SmallVec<[f64; 8]>,
    /// Grayson–Streed only: the Scatchard–Hildebrand inputs `(Vᵢᴸ, δᵢ)`
    /// resolved once (spec overrides, else the components' own fields), or
    /// `None` when they are unavailable and γ ≡ 1.
    scatchard: Option<ScatchardInputs>,
}

/// `(Vᵢᴸ [cm³/mol], δᵢ [(cal/cm³)^½])` per component for the regular-solution γ.
type ScatchardInputs = (smallvec::SmallVec<[f64; 8]>, smallvec::SmallVec<[f64; 8]>);

impl SystemTpCache {
    /// Build the cache for `spec` at `t` (**K**) and `p` (**kPa absolute**).
    ///
    /// # Errors
    /// [`FlashError::Thermo`] if a saturation or fugacity evaluation needed for
    /// the γ-φ constants fails, or the mixture layer rejects the spec.
    pub(crate) fn new(spec: &SystemSpec, t: f64, p: f64) -> Result<Self, FlashError> {
        let n = spec.n();
        let build = |eos| {
            crate::mixture::TpCache::new(&spec.mixture_spec(eos), t, p)
                .map_err(|e| FlashError::Thermo(e.to_string()))
        };
        let liquid = match spec.liquid {
            LiquidModel::Cubic(eos) => Some(build(eos)?),
            _ => None,
        };
        let vapor = match spec.vapor {
            VaporModel::Cubic(eos) => Some(build(eos)?),
            _ => None,
        };

        // γ-φ constants + activity matrices, only when the liquid is γ-based.
        let mut gamma_phi_const = smallvec::SmallVec::new();
        let mut activity = None;
        if let LiquidModel::Activity(_) | LiquidModel::IdealSolution = spec.liquid {
            let have_vl = spec.vl.len() == n;
            let ln_p = p.ln();
            gamma_phi_const.reserve(n);
            for i in 0..n {
                let psat_i = psat(spec.sat_model(i), &spec.components[i], t)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?;
                let ln_phi_sat = pure_sat_ln_phi(spec, i, t, psat_i);
                let ln_poy = if have_vl {
                    ln_poynting_factor(&spec.components[i], p, psat_i, t)
                } else {
                    0.0
                };
                gamma_phi_const.push(psat_i.ln() + ln_phi_sat + ln_poy - ln_p);
            }
            if let LiquidModel::Activity(model) = spec.liquid {
                activity = Some(crate::activity::ActivityTpCache::new(
                    model, spec.aij, spec.alpha, spec.vl, t,
                ));
            }
        }

        let virial_b = match spec.vapor {
            VaporModel::Virial => Some(crate::virial::b_mix_matrix_flat(spec.components, t)),
            _ => None,
        };

        // Refinery methods (M20): everything but the vapor φ̂ and (for
        // Grayson-Streed) the regular-solution γ is a per-(T, P) constant.
        let mut refinery_const = smallvec::SmallVec::new();
        let mut scatchard = None;
        match spec.liquid {
            LiquidModel::GraysonStreed => {
                refinery_const.reserve(n);
                for c in spec.components {
                    refinery_const.push(grayson_streed_ln_nu(c, t, p));
                }
                scatchard = scatchard_inputs(spec);
            }
            LiquidModel::BraunK10 => {
                refinery_const.reserve(n);
                for c in spec.components {
                    refinery_const.push(bk10_ln_k_ideal(c, t, p)?);
                }
            }
            _ => {}
        }

        Ok(Self {
            t,
            p,
            liquid,
            vapor,
            gamma_phi_const,
            activity,
            virial_b,
            refinery_const,
            scatchard,
        })
    }
}

// ===========================================================================
// Refinery K-value methods (M20): Grayson-Streed and Braun K10 building blocks.
// ===========================================================================

/// Grayson–Streed `ln νᵢ` for one component at `(t, p)`, species picked by name.
fn grayson_streed_ln_nu(c: &Component, t: f64, p: f64) -> f64 {
    crate::eos::regular_solution_ln_nu(
        crate::eos::RegularSolutionSet::GraysonStreed1963,
        t,
        p,
        c,
        crate::eos::ChaoSeaderSpecies::for_component(c),
    )
}

/// Braun K10 ideal-vapor `ln Kᵢ = ln Pᵢᴹᴮ(T; Tb,ᵢ, K_W,ᵢ) − ln P` for one
/// component. `Component::watson_k == 0` means "unknown" → no Watson correction.
fn bk10_ln_k_ideal(c: &Component, t: f64, p: f64) -> Result<f64, FlashError> {
    if c.tb <= 0.0 {
        return Err(FlashError::Thermo(format!(
            "Braun K10 needs a normal boiling point for '{}' (Component::tb is {})",
            c.name, c.tb
        )));
    }
    let kw = (c.watson_k > 0.0).then_some(c.watson_k);
    crate::petroleum::vapor_pressure::ln_vapor_pressure(t, c.tb, kw)
        .map(|ln_psat| ln_psat - p.ln())
        .map_err(|e| FlashError::Thermo(format!("Braun K10 for '{}': {e}", c.name)))
}

/// Resolve the Scatchard–Hildebrand inputs for a Grayson–Streed system:
/// the `SystemSpec` `vl` / `delta` overrides when both are full-length,
/// otherwise each component's `liquid_volume` / `solubility_param`. `None`
/// (γ ≡ 1) if any volume or solubility parameter is missing — a documented
/// degradation, not an error, because a hydrogen or light-gas component in a
/// database often has no δ and the νᵢ term still carries the method.
fn scatchard_inputs(spec: &SystemSpec) -> Option<ScatchardInputs> {
    let n = spec.n();
    let mut vl: smallvec::SmallVec<[f64; 8]> = smallvec::SmallVec::with_capacity(n);
    let mut delta: smallvec::SmallVec<[f64; 8]> = smallvec::SmallVec::with_capacity(n);
    let overrides = spec.vl.len() == n && spec.delta.len() == n;
    for (i, c) in spec.components.iter().enumerate() {
        let (v, d) = if overrides {
            (spec.vl[i], spec.delta[i])
        } else {
            (c.liquid_volume, c.solubility_param)
        };
        if !(v > 0.0 && v.is_finite() && d > 0.0 && d.is_finite()) {
            return None;
        }
        vl.push(v);
        delta.push(d);
    }
    Some((vl, delta))
}

/// Grayson–Streed `ln Kᵢ` assembly: `slot` arrives holding the vapor `ln φ̂ᵢⱽ`
/// and leaves holding `ln νᵢ + ln γᵢ(x) − ln φ̂ᵢⱽ`.
fn grayson_streed_ln_k_into(
    ln_nu: &[f64],
    scatchard: Option<&ScatchardInputs>,
    x: &[f64],
    t: f64,
    slot: &mut [f64],
) {
    match scatchard {
        Some((vl, delta)) => {
            let mut ln_gamma: smallvec::SmallVec<[f64; 8]> = smallvec::smallvec![0.0; x.len()];
            ln_gamma_all(
                crate::activity::ActivityModel::ScatchardHildebrand,
                x,
                &[],
                &[],
                vl,
                delta,
                t,
                &mut ln_gamma,
            );
            for i in 0..slot.len() {
                slot[i] = ln_nu[i] + ln_gamma[i] - slot[i];
            }
        }
        None => {
            for i in 0..slot.len() {
                slot[i] = ln_nu[i] - slot[i];
            }
        }
    }
}

/// ln φ̂ᵢ of every component in the **vapor** phase of composition `y`, written
/// into a caller-owned slice (no allocation on the cubic and ideal-gas paths).
fn vapor_ln_phi_into(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    y: &[f64],
    out: &mut [f64],
) -> Result<(), FlashError> {
    match spec.vapor {
        VaporModel::IdealGas => {
            out.fill(0.0);
            Ok(())
        }
        // The virial path still builds its own Bᵢⱼ matrix internally; flattening
        // that is Part 2 §9 of the performance audit.
        VaporModel::Virial => {
            let v = ln_phi_mix_virial(spec.components, y, t, p)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
            out.copy_from_slice(&v);
            Ok(())
        }
        VaporModel::Cubic(eos) => {
            ln_phi_mix_into(&spec.mixture_spec(eos), t, p, y, PhaseId::Vapor, out)
                .map_err(|e| FlashError::Thermo(e.to_string()))
        }
    }
}

/// [`vapor_ln_phi_into`] reusing the cubic vapor's prebuilt pure-component
/// state. Non-cubic vapors have nothing cached and fall through unchanged.
fn vapor_ln_phi_cached_into(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    y: &[f64],
    out: &mut [f64],
) -> Result<(), FlashError> {
    match (spec.vapor, &cache.vapor, &cache.virial_b) {
        (VaporModel::Cubic(eos), Some(tp), _) => {
            ln_phi_mix_cached_into(&spec.mixture_spec(eos), tp, y, PhaseId::Vapor, out)
                .map_err(|e| FlashError::Thermo(e.to_string()))
        }
        (VaporModel::Virial, _, Some(mat)) => {
            let mut row_dot: smallvec::SmallVec<[f64; 8]> = smallvec::smallvec![0.0; y.len()];
            crate::virial::ln_phi_mix_virial_flat_into(mat, y, cache.t, cache.p, &mut row_dot, out);
            Ok(())
        }
        _ => vapor_ln_phi_into(spec, cache.t, cache.p, y, out),
    }
}

/// **ln** of the pure-component saturated-vapor fugacity coefficient φᵢˢᵃᵗ at
/// (T, Psat,ᵢ) — the reference state the γ-φ Poynting correction hangs off.
/// Zero for an ideal vapor.
fn pure_sat_ln_phi(spec: &SystemSpec, i: usize, t: f64, psat_i: f64) -> f64 {
    let comp = &spec.components[i];
    match spec.vapor {
        VaporModel::IdealGas => 0.0,
        VaporModel::Virial => ln_phi_pure_virial(comp, t, psat_i),
        VaporModel::Cubic(eos) => ln_phi_pure(eos, t, psat_i, comp, PhaseId::Vapor).unwrap_or(0.0),
    }
}

/// Equilibrium ratios `Kᵢ = yᵢ/xᵢ` for the mixture at `(t, p)` given trial
/// phase compositions `x` (liquid) and `y` (vapor).
///
/// Dispatches on the liquid model:
/// - **φ-φ** (`Cubic`): `Kᵢ = exp(ln φ̂ᵢᴸ(x) − ln φ̂ᵢⱽ(y))`.
/// - **γ-φ** (`Activity` / `IdealSolution`): modified Raoult
///   `Kᵢ = γᵢ(x)·Psat,ᵢ·φᵢˢᵃᵗ·POYᵢ / (φ̂ᵢⱽ(y)·P)`; γ = 1 for the ideal
///   solution.
/// - `ChaoSeader`: `Kᵢ = νᵢᴸ·P / (φ̂ᵢⱽ(y)·P)` using the Chao-Seader liquid
///   fugacity coefficient.
///
/// `t` in **K**, `p` in **kPa absolute**.
///
/// # Errors
/// [`FlashError::Dimension`] on length mismatch; [`FlashError::Thermo`] if a
/// fugacity evaluation fails.
pub fn k_values(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    x: &[f64],
    y: &[f64],
) -> Result<Vec<f64>, FlashError> {
    let mut k = vec![0.0; spec.n()];
    ln_k_values_into(spec, t, p, x, y, &mut k)?;
    // Only here — at the API boundary — do the logs become ratios.
    for ki in k.iter_mut() {
        *ki = ki.exp();
    }
    Ok(k)
}

/// [`ln_k_values_into`] reusing a prebuilt [`SystemTpCache`].
///
/// Identical result. What it skips per call: both phases' per-component α and
/// dimensionless-parameter pass (audit Part 2 §1), and — on the γ-φ path — the
/// per-component `Psat`, `φˢᵃᵗ` and Poynting evaluations plus the activity
/// model's temperature-dependent matrix rebuild (Part 2 §5). All of it is
/// composition-independent, and the flash calls this once per outer iteration.
///
/// # Errors
/// As [`ln_k_values_into`]; additionally [`FlashError::Dimension`] if the cache
/// was not built for the same `(t, p)`.
pub(crate) fn ln_k_values_cached_into(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    x: &[f64],
    y: &[f64],
    out: &mut [f64],
) -> Result<(), FlashError> {
    let n = spec.n();
    if x.len() != n || y.len() != n || out.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, x={}, y={}, out={}",
            x.len(),
            y.len(),
            out.len()
        )));
    }
    type Scratch = smallvec::SmallVec<[f64; 8]>;
    let (t, p) = (cache.t, cache.p);
    vapor_ln_phi_cached_into(spec, cache, y, out)?;

    match spec.liquid {
        LiquidModel::Cubic(eos) => {
            let mut liq: Scratch = smallvec::smallvec![0.0; n];
            match &cache.liquid {
                Some(tp) => ln_phi_mix_cached_into(
                    &spec.mixture_spec(eos),
                    tp,
                    x,
                    PhaseId::Liquid,
                    &mut liq,
                )
                .map_err(|e| FlashError::Thermo(e.to_string()))?,
                None => {
                    ln_phi_mix_into(&spec.mixture_spec(eos), t, p, x, PhaseId::Liquid, &mut liq)
                        .map_err(|e| FlashError::Thermo(e.to_string()))?
                }
            }
            for i in 0..n {
                out[i] = liq[i] - out[i];
            }
            Ok(())
        }

        // γ-φ: everything except ln γ and the vapor ln φ̂ is precomputed, so
        // this collapses to `ln Kᵢ = ln γᵢ + constᵢ − ln φ̂ᵢⱽ`.
        LiquidModel::Activity(model) => {
            let mut ln_gamma: Scratch = smallvec::smallvec![0.0; n];
            match &cache.activity {
                Some(act) => act.ln_gamma_all(
                    model,
                    x,
                    spec.aij,
                    spec.alpha,
                    spec.vl,
                    spec.delta,
                    t,
                    &mut ln_gamma,
                ),
                None => ln_gamma_all(
                    model,
                    x,
                    spec.aij,
                    spec.alpha,
                    spec.vl,
                    spec.delta,
                    t,
                    &mut ln_gamma,
                ),
            }
            gamma_phi_ln_k_cached_into(cache, &ln_gamma, out)
        }
        LiquidModel::IdealSolution => {
            let ln_gamma: Scratch = smallvec::smallvec![0.0; n]; // γ = 1
            gamma_phi_ln_k_cached_into(cache, &ln_gamma, out)
        }

        LiquidModel::ChaoSeader => {
            for (slot, comp) in out.iter_mut().zip(spec.components) {
                let ln_nu = crate::eos::chao_seader_ln_phi(
                    t,
                    p,
                    comp,
                    crate::eos::ChaoSeaderSpecies::Normal,
                );
                *slot = ln_nu - *slot;
            }
            Ok(())
        }

        // Refinery methods (M20): the constant part is cached; only the
        // regular-solution γ (Grayson-Streed) is composition work.
        LiquidModel::GraysonStreed => {
            if cache.refinery_const.len() != n {
                return Err(FlashError::Dimension("Grayson-Streed cache size".into()));
            }
            grayson_streed_ln_k_into(&cache.refinery_const, cache.scatchard.as_ref(), x, t, out);
            Ok(())
        }
        LiquidModel::BraunK10 => {
            if cache.refinery_const.len() != n {
                return Err(FlashError::Dimension("Braun K10 cache size".into()));
            }
            for (o, c) in out.iter_mut().zip(&cache.refinery_const) {
                *o = c - *o;
            }
            Ok(())
        }
    }
}

/// Modified-Raoult **ln** K from a ln γ vector and the cache's precomputed
/// `ln Psat + ln φˢᵃᵗ + ln POY − ln P`. `slot` arrives holding the vapor
/// `ln φ̂ᵢⱽ` and leaves holding `ln Kᵢ`.
fn gamma_phi_ln_k_cached_into(
    cache: &SystemTpCache,
    ln_gamma: &[f64],
    slot: &mut [f64],
) -> Result<(), FlashError> {
    if cache.gamma_phi_const.len() != slot.len() {
        return Err(FlashError::Dimension(format!(
            "γ-φ cache has {} entries, need {}",
            cache.gamma_phi_const.len(),
            slot.len()
        )));
    }
    for i in 0..slot.len() {
        slot[i] = ln_gamma[i] + cache.gamma_phi_const[i] - slot[i];
    }
    Ok(())
}

/// **ln** of the equilibrium ratios, written into a caller-owned slice.
///
/// This is the primitive [`k_values`] is built from, and the one every flash
/// driver should call: equilibrium models produce ln φ̂, ln γ and ln Psat
/// natively, so exponentiating each term only to divide the products — and
/// then taking a logarithm again to form the flash's ln-K residual — is pure
/// added cost and added numerical range (Part 1 §2 of the performance audit).
/// The flash exponentiates exactly once per iteration, where Rachford-Rice
/// genuinely needs `Kᵢ` rather than `ln Kᵢ`.
///
/// Model dispatch is identical to [`k_values`]; the assembly is additive:
/// - **φ-φ**: `ln Kᵢ = ln φ̂ᵢᴸ(x) − ln φ̂ᵢⱽ(y)`
/// - **γ-φ**: `ln Kᵢ = ln γᵢ + ln Psatᵢ + ln φᵢˢᵃᵗ + ln POYᵢ − ln φ̂ᵢⱽ − ln P`
/// - **Chao-Seader**: `ln Kᵢ = ln νᵢᴸ − ln φ̂ᵢⱽ`
///
/// `t` in **K**, `p` in **kPa absolute**; `out` receives one **dimensionless**
/// `ln Kᵢ` per component.
///
/// # Errors
/// [`FlashError::Dimension`] on any length mismatch (including `out`);
/// [`FlashError::Thermo`] if a fugacity or saturation evaluation fails.
pub fn ln_k_values_into(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    x: &[f64],
    y: &[f64],
    out: &mut [f64],
) -> Result<(), FlashError> {
    let n = spec.n();
    if x.len() != n || y.len() != n || out.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, x={}, y={}, out={}",
            x.len(),
            y.len(),
            out.len()
        )));
    }
    // Vapor ln φ̂ lands in `out` first, then each branch subtracts it in place
    // — one N-wide scratch buffer instead of two owned vectors per call.
    // `Scratch` stays inline (no heap) for the mixture sizes this engine
    // targets; larger mixtures spill transparently.
    type Scratch = smallvec::SmallVec<[f64; 8]>;
    vapor_ln_phi_into(spec, t, p, y, out)?;

    match spec.liquid {
        // --- φ-φ: EOS both phases ---
        LiquidModel::Cubic(eos) => {
            let mut liq: Scratch = smallvec::smallvec![0.0; n];
            ln_phi_mix_into(&spec.mixture_spec(eos), t, p, x, PhaseId::Liquid, &mut liq)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
            for i in 0..n {
                out[i] = liq[i] - out[i];
            }
            Ok(())
        }

        // --- γ-φ: activity-model liquid ---
        LiquidModel::Activity(model) => {
            let mut ln_gamma: Scratch = smallvec::smallvec![0.0; n];
            ln_gamma_all(
                model,
                x,
                spec.aij,
                spec.alpha,
                spec.vl,
                spec.delta,
                t,
                &mut ln_gamma,
            );
            gamma_phi_ln_k_into(spec, t, p, &ln_gamma, out)
        }
        LiquidModel::IdealSolution => {
            let ln_gamma: Scratch = smallvec::smallvec![0.0; n]; // γ = 1
            gamma_phi_ln_k_into(spec, t, p, &ln_gamma, out)
        }

        // --- Chao-Seader liquid fugacity coefficient νᵢ ---
        LiquidModel::ChaoSeader => {
            // νᵢ = fᵢᴸ/(xᵢP); with the vapor φ̂ᵢⱽ, Kᵢ = νᵢ/φ̂ᵢⱽ. Species set
            // defaults to Normal (H₂/methane special-casing is a caller
            // concern handled through the pure binding).
            for (slot, comp) in out.iter_mut().zip(spec.components) {
                let ln_nu = crate::eos::chao_seader_ln_phi(
                    t,
                    p,
                    comp,
                    crate::eos::ChaoSeaderSpecies::Normal,
                );
                *slot = ln_nu - *slot;
            }
            Ok(())
        }

        // --- Grayson-Streed: ln Kᵢ = ln νᵢ + ln γᵢ − ln φ̂ᵢⱽ (M20) ---
        LiquidModel::GraysonStreed => {
            let mut ln_nu: Scratch = smallvec::smallvec![0.0; n];
            for (slot, c) in ln_nu.iter_mut().zip(spec.components) {
                *slot = grayson_streed_ln_nu(c, t, p);
            }
            let scatchard = scatchard_inputs(spec);
            grayson_streed_ln_k_into(&ln_nu, scatchard.as_ref(), x, t, out);
            Ok(())
        }

        // --- Braun K10: ln Kᵢ = ln Pᵢᴹᴮ − ln P − ln φ̂ᵢⱽ (M20) ---
        LiquidModel::BraunK10 => {
            for (slot, c) in out.iter_mut().zip(spec.components) {
                *slot = bk10_ln_k_ideal(c, t, p)? - *slot;
            }
            Ok(())
        }
    }
}

/// ln φ̂ᵢ of a composition `w` treated as a **single phase**, using the root
/// that minimizes the reduced Gibbs energy `g = Σ wᵢ(ln wᵢ + ln φ̂ᵢ)`.
///
/// This is the fugacity the tangent-plane stability test (§I) needs: at a
/// candidate single-phase composition, the physically realized phase is the
/// lower-Gibbs cubic root. Only the cubic (φ-φ) path is supported —
/// activity-model liquids don't exhibit the trivial-solution instability
/// the test targets, so [`super::stability`] restricts to cubic systems.
///
/// # Errors
/// [`FlashError::Unsupported`] for non-cubic liquid models;
/// [`FlashError::Thermo`] if the fugacity evaluation fails.
///
/// Test-only since the phase envelope moved to the root-reporting
/// [`min_gibbs_ln_phi_root_into`]; production callers use the `_into` forms.
#[cfg(test)]
pub fn min_gibbs_ln_phi(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    w: &[f64],
) -> Result<Vec<f64>, FlashError> {
    let mut out = vec![0.0; w.len()];
    min_gibbs_ln_phi_into(spec, t, p, w, &mut out)?;
    Ok(out)
}

/// The owned-return `min_gibbs_ln_phi` written into a caller-owned slice —
/// **no allocation**.
///
/// The stability test calls this once per trial-phase iteration, so the owned
/// return of `min_gibbs_ln_phi` was one allocation per iteration per trial.
/// Both cubic roots are now evaluated against a single shared mixture state
/// (see [`ln_phi_mix_min_gibbs_into`]) instead of walking the whole mixture
/// path twice.
///
/// # Errors
/// [`FlashError::Unsupported`] for non-cubic liquid models;
/// [`FlashError::Thermo`] if neither root is physical at this composition.
/// [`min_gibbs_ln_phi_into`] reusing a prebuilt [`SystemTpCache`].
///
/// The stability test's trial-phase loop calls this every iteration at fixed
/// `(T, P)`, so this is where Part 1 §8 (one mixture state, both roots) and
/// Part 2 §1 (one pure-component pass per state point) compound.
/// Pure-component min-Gibbs ln φᵢ of the cubic **liquid** model at the cache's
/// (T, P), or `None` if the liquid is not cubic or the cubic has no root.
pub(crate) fn pure_ln_phi_min_gibbs_cached(cache: &SystemTpCache, i: usize) -> Option<f64> {
    cache.liquid.as_ref()?.pure_ln_phi_min_gibbs(i)
}

pub(crate) fn min_gibbs_ln_phi_cached_into(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    w: &[f64],
    out: &mut [f64],
) -> Result<(), FlashError> {
    let eos = match spec.liquid {
        LiquidModel::Cubic(eos) => eos,
        _ => {
            return Err(FlashError::Unsupported(
                "min-Gibbs ln φ is defined only for a cubic (φ-φ) system".into(),
            ));
        }
    };
    match &cache.liquid {
        Some(tp) => ln_phi_mix_min_gibbs_cached_into(&spec.mixture_spec(eos), tp, w, out)
            .map_err(|e| FlashError::Thermo(e.to_string())),
        None => min_gibbs_ln_phi_into(spec, cache.t, cache.p, w, out),
    }
}

pub fn min_gibbs_ln_phi_into(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    w: &[f64],
    out: &mut [f64],
) -> Result<(), FlashError> {
    let eos = match spec.liquid {
        LiquidModel::Cubic(eos) => eos,
        _ => {
            return Err(FlashError::Unsupported(
                "min-Gibbs ln φ is defined only for a cubic (φ-φ) system".into(),
            ));
        }
    };
    ln_phi_mix_min_gibbs_into(&spec.mixture_spec(eos), t, p, w, out)
        .map_err(|e| FlashError::Thermo(e.to_string()))
}

/// [`min_gibbs_ln_phi_into`] that also reports which cubic root won
/// (`PhaseId::Liquid` or `PhaseId::Vapor`). `t` in **K**, `p` in **kPa
/// absolute**; same errors. Used by the phase-envelope corrector, whose
/// analytic Jacobian must differentiate the same root the residual used.
pub(crate) fn min_gibbs_ln_phi_root_into(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    w: &[f64],
    out: &mut [f64],
) -> Result<PhaseId, FlashError> {
    let eos = match spec.liquid {
        LiquidModel::Cubic(eos) => eos,
        _ => {
            return Err(FlashError::Unsupported(
                "min-Gibbs ln φ is defined only for a cubic (φ-φ) system".into(),
            ));
        }
    };
    crate::mixture::ln_phi_mix_min_gibbs_root_into(&spec.mixture_spec(eos), t, p, w, out)
        .map_err(|e| FlashError::Thermo(e.to_string()))
}

/// Modified-Raoult **ln** K from a ln γ vector, assembled additively in place.
///
/// `slot` arrives holding the vapor `ln φ̂ᵢⱽ` and leaves holding `ln Kᵢ` —
/// every term enters as a logarithm, so the whole modified-Raoult expression
/// `Kᵢ = γᵢ·Psatᵢ·φᵢˢᵃᵗ·POYᵢ / (φ̂ᵢⱽ·P)` becomes a sum with no intermediate
/// `exp` (Part 1 §2 of the performance audit).
fn gamma_phi_ln_k_into(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    ln_gamma: &[f64],
    slot: &mut [f64],
) -> Result<(), FlashError> {
    let n = spec.n();
    let have_vl = spec.vl.len() == n;
    let ln_p = p.ln();
    for i in 0..n {
        let psat_i = psat(spec.sat_model(i), &spec.components[i], t)
            .map_err(|e| FlashError::Thermo(e.to_string()))?;
        let ln_phi_sat = pure_sat_ln_phi(spec, i, t, psat_i);
        let ln_poy = if have_vl {
            ln_poynting_factor(&spec.components[i], p, psat_i, t)
        } else {
            0.0
        };
        // ln Kᵢ = ln γᵢ + ln Psat + ln φˢᵃᵗ + ln POY − ln φ̂ᵢⱽ − ln P.
        slot[i] = ln_gamma[i] + psat_i.ln() + ln_phi_sat + ln_poy - slot[i] - ln_p;
    }
    Ok(())
}

// ===========================================================================
// K-value temperature / pressure derivatives (§L step 3, M12.3).
// ===========================================================================

/// Equilibrium ratios and their exact T- and P-derivatives at one state.
///
/// Units: `k` dimensionless; `d_ln_k_d_t` in **1/K**; `d_ln_k_d_p` in **1/kPa**.
/// The `k` field is bit-identical to [`k_values`] on the same inputs.
///
/// Composition derivatives of K are intentionally not included: they follow
/// from the per-phase `mixture::d_ln_phi_d_n` (⌈n/8⌉ dual passes each) as
/// `∂lnKᵢ/∂nⱼ = ∂lnφ̂ᵢᴸ/∂nⱼ|x − ∂lnφ̂ᵢⱽ/∂nⱼ|y`; callers that need the full
/// Jacobian block assemble it from those.
#[derive(Debug, Clone)]
pub struct KValueDerivs {
    /// Kᵢ = yᵢ/xᵢ. **Dimensionless.**
    pub k: Vec<f64>,
    /// ∂ln Kᵢ/∂T at constant P, x, y. **1/K.**
    pub d_ln_k_d_t: Vec<f64>,
    /// ∂ln Kᵢ/∂P at constant T, x, y. **1/kPa.**
    pub d_ln_k_d_p: Vec<f64>,
}

/// Vapor-side ∂ln φ̂ᵢⱽ/∂T and ∂ln φ̂ᵢⱽ/∂P for the mixture at `(t, p, y)`.
///
/// Ideal-gas vapor ⇒ both are zero; cubic vapor ⇒ exact dual derivatives.
/// Virial vapor is not yet supported by the derivative API.
fn vapor_lnphi_derivs(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    y: &[f64],
    want_t: bool,
    want_p: bool,
) -> Result<(Vec<f64>, Vec<f64>), FlashError> {
    let n = spec.n();
    match spec.vapor {
        VaporModel::IdealGas => Ok((vec![0.0; n], vec![0.0; n])),
        VaporModel::Cubic(eos) => {
            let ms = spec.mixture_spec(eos);
            // Each derivative is a separate dual evaluation; a caller that
            // needs only one (`k_values_with_t_derivs`, or the saturation-
            // pressure solver's `k_values_with_p_derivs`) skips the other.
            let dt = if want_t {
                crate::mixture::d_ln_phi_d_t(&ms, t, p, y, PhaseId::Vapor)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?
            } else {
                vec![0.0; n]
            };
            let dp = if want_p {
                crate::mixture::d_ln_phi_d_p(&ms, t, p, y, PhaseId::Vapor)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?
            } else {
                vec![0.0; n]
            };
            Ok((dt, dp))
        }
        VaporModel::Virial => Err(FlashError::Unsupported(
            "k_values_with_derivs: virial vapor T/P derivatives not implemented".into(),
        )),
    }
}

/// d(ln φᵢˢᵃᵗ)/dT for the pure saturated-vapor reference at `(T, Psatᵢ(T))`.
///
/// φᵢˢᵃᵗ = φ_pure(T, Psatᵢ(T)), so the total T-derivative carries the Psat(T)
/// chain: `d lnφˢᵃᵗ/dT = ∂lnφ_pure/∂T|_P + ∂lnφ_pure/∂P|_T · dPsatᵢ/dT`. Both
/// partials come from the exact mixture dual path applied to the single
/// component (an n=1 mixture). Ideal-gas vapor ⇒ 0.
fn dln_phi_sat_dt(
    spec: &SystemSpec,
    i: usize,
    t: f64,
    psat_i: f64,
    dpsat_dt: f64,
) -> Result<f64, FlashError> {
    match spec.vapor {
        VaporModel::IdealGas => Ok(0.0),
        VaporModel::Cubic(eos) => {
            // Single-component mixture spec for component i (classical rule,
            // no kij / GE) — its ln φ̂ equals the pure ln φ.
            let comp = std::slice::from_ref(&spec.components[i]);
            let ms = MixtureSpec {
                eos,
                rule: MixingRule::Classical,
                components: comp,
                kij: &[],
                ge: None,
            };
            let one = [1.0];
            let dt = crate::mixture::d_ln_phi_d_t(&ms, t, psat_i, &one, PhaseId::Vapor)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
            let dp = crate::mixture::d_ln_phi_d_p(&ms, t, psat_i, &one, PhaseId::Vapor)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
            Ok(dt[0] + dp[0] * dpsat_dt)
        }
        VaporModel::Virial => Err(FlashError::Unsupported(
            "k_values_with_derivs: virial φˢᵃᵗ T derivative not implemented".into(),
        )),
    }
}

/// ∂ln γᵢ/∂T at constant composition, via one dual evaluation of the
/// T-generic activity path (M12.3). Result in **1/K**.
fn dln_gamma_dt(spec: &SystemSpec, model: ActivityModel, t: f64, x: &[f64]) -> Vec<f64> {
    use num_dual::Dual64;
    let n = spec.n();
    let xd: Vec<Dual64> = x.iter().map(|&xi| Dual64::from(xi)).collect();
    let td = Dual64::new(t, 1.0);
    let mut lng = vec![Dual64::from(0.0); n];
    crate::activity::ln_gamma_all_generic(
        model, &xd, spec.aij, spec.alpha, spec.vl, spec.delta, td, &mut lng,
    );
    lng.iter().map(|v| v.eps).collect()
}

/// ∂ln γᵢ/∂nⱼ of an activity-model liquid at mole fractions `x` and total
/// moles 1 — the composition Jacobian a Newton solver needs for a γ-φ liquid
/// (`OPTIMIZATION_PLAN_PART1.md` §16). **Dimensionless**; `t` in **K**.
///
/// Exact, by dual numbers: one pass of the generic activity code per
/// component gives the unconstrained `Dᵢⱼ = ∂ln γᵢ/∂xⱼ`, and because the mole
/// fractions are `xₖ = nₖ/N`, the mole-number derivative at `N = 1` is
/// `Gᵢⱼ = Dᵢⱼ − Σₖ xₖ·Dᵢₖ`. Rows are `i`, columns `j`. Each row satisfies
/// `Σⱼ xⱼ·Gᵢⱼ = 0` (ln γ is intensive), and `Σᵢ xᵢ·Gᵢⱼ = 0` (Gibbs–Duhem).
fn d_ln_gamma_d_n(spec: &SystemSpec, model: ActivityModel, t: f64, x: &[f64]) -> Vec<Vec<f64>> {
    use num_dual::Dual64;
    let n = x.len();
    let td = Dual64::from(t);
    let mut xd: Vec<Dual64> = x.iter().map(|&xi| Dual64::from(xi)).collect();
    let mut lng = vec![Dual64::from(0.0); n];
    let mut g = vec![vec![0.0; n]; n];
    for j in 0..n {
        // Seed component j: its derivative part is 1, every other one 0.
        xd[j] = Dual64::new(x[j], 1.0);
        crate::activity::ln_gamma_all_generic(
            model, &xd, spec.aij, spec.alpha, spec.vl, spec.delta, td, &mut lng,
        );
        xd[j] = Dual64::from(x[j]);
        for (row, v) in g.iter_mut().zip(&lng) {
            row[j] = v.eps;
        }
    }
    // Project the unconstrained ∂/∂x onto ∂/∂n at fixed total.
    for row in g.iter_mut() {
        let shift: f64 = row.iter().zip(x).map(|(d, xk)| d * xk).sum();
        row.iter_mut().for_each(|v| *v -= shift);
    }
    g
}

/// Composition Jacobian of one phase's fugacity coefficients under the
/// system's model pair: `Gᵢⱼ = ∂ln φ̂ᵢ/∂nⱼ` at mole fractions `w` and total
/// moles 1 (**dimensionless**; rows `i`, columns `j`). `t` in **K**, `p` in
/// **kPa absolute**. For `PhaseId::Vapor` it is the vapor model's, for
/// `PhaseId::Liquid` the liquid model's.
///
/// - Ideal gas / ideal solution ⇒ zero (nothing depends on composition).
/// - Cubic ⇒ the mixture's exact Jacobian on that phase's root
///   ([`crate::mixture::d_ln_phi_d_n`]).
/// - Activity-model liquid ⇒ `∂ln γᵢ/∂nⱼ` ([`d_ln_gamma_d_n`]): Psat, φˢᵃᵗ
///   and the Poynting factor are composition-free.
/// - Virial vapor and the Chao–Seader-family liquids ⇒ `None`; a Newton
///   solver that needs the Jacobian stays on successive substitution there.
pub(crate) fn phase_ln_phi_jacobian(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    w: &[f64],
    phase: PhaseId,
) -> Option<Vec<Vec<f64>>> {
    let n = w.len();
    let zeros = || Some(vec![vec![0.0; n]; n]);
    match phase {
        PhaseId::Vapor => match spec.vapor {
            VaporModel::IdealGas => zeros(),
            VaporModel::Cubic(eos) => {
                crate::mixture::d_ln_phi_d_n(&spec.mixture_spec(eos), t, p, w, PhaseId::Vapor).ok()
            }
            VaporModel::Virial => None,
        },
        PhaseId::Liquid => match spec.liquid {
            LiquidModel::IdealSolution => zeros(),
            LiquidModel::Cubic(eos) => {
                crate::mixture::d_ln_phi_d_n(&spec.mixture_spec(eos), t, p, w, PhaseId::Liquid).ok()
            }
            LiquidModel::Activity(model) => Some(d_ln_gamma_d_n(spec, model, t, w)),
            LiquidModel::ChaoSeader | LiquidModel::GraysonStreed | LiquidModel::BraunK10 => None,
        },
    }
}

/// [`k_values`] plus exact ∂ln Kᵢ/∂T and ∂ln Kᵢ/∂P (§L, M12.3).
///
/// `t` in **K**, `p` in **kPa absolute**. Supports the φ-φ (cubic liquid) and
/// γ-φ (activity / ideal-solution liquid) paths with an ideal-gas or cubic
/// vapor. Virial vapor and Chao-Seader-family (Chao-Seader, Grayson-Streed,
/// Braun K10) liquid derivatives are not yet
/// implemented (they return [`FlashError::Unsupported`]).
///
/// The γ-φ derivative is assembled **term-for-term** from the same pieces
/// `gamma_phi_ln_k_into` adds up, so the two never drift:
/// `ln Kᵢ = ln γᵢ + ln Psatᵢ + ln φᵢˢᵃᵗ + ln POYᵢ − ln φ̂ᵢⱽ − ln P`, giving
/// `∂/∂T = ∂lnγᵢ/∂T + (dPsat/dT)/Psat + dlnφˢᵃᵗ/dT + ∂lnPOY/∂T − ∂lnφ̂ᵢⱽ/∂T`
/// and `∂/∂P = ∂lnPOY/∂P − ∂lnφ̂ᵢⱽ/∂P − 1/P`.
pub fn k_values_with_derivs(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    x: &[f64],
    y: &[f64],
) -> Result<KValueDerivs, FlashError> {
    k_values_with_derivs_impl(spec, t, p, x, y, true, true)
}

/// [`k_values_with_derivs`] without the pressure derivative: the returned
/// `d_ln_k_d_p` is all zeros and must not be read. `k` and `d_ln_k_d_t` are
/// identical to the full call. For callers that never use ∂/∂P — the
/// saturation-temperature solver here, and a column solver fitting a
/// temperature-only K model downstream. Skipping it removes one dual
/// evaluation per cubic phase (≈ ⅓ of the call at n = 10,
/// `OPTIMIZATION_PLAN_PART1.md` §14).
///
/// `t` in **K**, `p` in **kPa absolute**; `d_ln_k_d_t` in **1/K**.
pub fn k_values_with_t_derivs(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    x: &[f64],
    y: &[f64],
) -> Result<KValueDerivs, FlashError> {
    k_values_with_derivs_impl(spec, t, p, x, y, true, false)
}

/// [`k_values_with_derivs`] without the temperature derivative: the returned
/// `d_ln_k_d_t` is all zeros and must not be read. `k` and `d_ln_k_d_p` are
/// identical to the full call. For the saturation-pressure Newton solver,
/// which never uses ∂/∂T.
///
/// `t` in **K**, `p` in **kPa absolute**; `d_ln_k_d_p` in **1/kPa**.
pub(crate) fn k_values_with_p_derivs(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    x: &[f64],
    y: &[f64],
) -> Result<KValueDerivs, FlashError> {
    k_values_with_derivs_impl(spec, t, p, x, y, false, true)
}

fn k_values_with_derivs_impl(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    x: &[f64],
    y: &[f64],
    want_t: bool,
    want_p: bool,
) -> Result<KValueDerivs, FlashError> {
    let n = spec.n();
    let k = k_values(spec, t, p, x, y)?;
    let (vap_dt, vap_dp) = vapor_lnphi_derivs(spec, t, p, y, want_t, want_p)?;

    match spec.liquid {
        // --- φ-φ: ∂lnKᵢ = ∂lnφ̂ᵢᴸ − ∂lnφ̂ᵢⱽ ---
        LiquidModel::Cubic(eos) => {
            let ms = spec.mixture_spec(eos);
            let liq_dt = if want_t {
                crate::mixture::d_ln_phi_d_t(&ms, t, p, x, PhaseId::Liquid)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?
            } else {
                vec![0.0; n]
            };
            let liq_dp = if want_p {
                crate::mixture::d_ln_phi_d_p(&ms, t, p, x, PhaseId::Liquid)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?
            } else {
                vec![0.0; n]
            };
            let d_ln_k_d_t = (0..n).map(|i| liq_dt[i] - vap_dt[i]).collect();
            let d_ln_k_d_p = (0..n).map(|i| liq_dp[i] - vap_dp[i]).collect();
            Ok(KValueDerivs {
                k,
                d_ln_k_d_t,
                d_ln_k_d_p,
            })
        }

        // --- γ-φ: term-by-term over the modified-Raoult assembly ---
        LiquidModel::Activity(_) | LiquidModel::IdealSolution => {
            // ∂lnγ/∂T (γ = 1 ⇒ 0 for the ideal solution).
            let dgamma_dt = match spec.liquid {
                LiquidModel::Activity(model) if want_t => dln_gamma_dt(spec, model, t, x),
                _ => vec![0.0; n],
            };
            let have_vl = spec.vl.len() == n;
            const R: f64 = 8.31451; // J/(mol·K); matches poynting_factor
            let mut d_ln_k_d_t = vec![0.0; n];
            let mut d_ln_k_d_p = vec![0.0; n];
            for i in 0..n {
                let comp = &spec.components[i];
                if !want_t {
                    // Pressure only: γ, Psat and φˢᵃᵗ are P-independent, so
                    // none of the saturation-curve work below is needed.
                    let dpoy_dp = if have_vl {
                        comp.liquid_volume * 1e-3 / R / t
                    } else {
                        0.0
                    };
                    d_ln_k_d_p[i] = dpoy_dp - vap_dp[i] - 1.0 / p;
                    continue;
                }
                let psat_i = psat(spec.sat_model(i), comp, t)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?;
                let dpsat_dt = crate::saturation::d_psat_dt(spec.sat_model(i), comp, t)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?;
                let dln_psat_dt = dpsat_dt / psat_i;
                let dln_phisat_dt = dln_phi_sat_dt(spec, i, t, psat_i, dpsat_dt)?;
                // Poynting: ln POY = k_poy·(P − Psat)/T, k_poy = V_L·1e-3/R.
                let (dpoy_dt, dpoy_dp) = if have_vl {
                    let k_poy = comp.liquid_volume * 1e-3 / R;
                    let dt = k_poy * (-dpsat_dt / t - (p - psat_i) / (t * t));
                    let dp = k_poy / t;
                    (dt, dp)
                } else {
                    (0.0, 0.0)
                };
                d_ln_k_d_t[i] = dgamma_dt[i] + dln_psat_dt + dln_phisat_dt + dpoy_dt - vap_dt[i];
                // γ, Psat, φˢᵃᵗ are P-independent; the −ln P term gives −1/P.
                d_ln_k_d_p[i] = dpoy_dp - vap_dp[i] - 1.0 / p;
            }
            Ok(KValueDerivs {
                k,
                d_ln_k_d_t,
                d_ln_k_d_p,
            })
        }

        LiquidModel::ChaoSeader | LiquidModel::GraysonStreed | LiquidModel::BraunK10 => {
            Err(FlashError::Unsupported(
                "k_values_with_derivs: Chao-Seader / Grayson-Streed / Braun K10 liquid \
                 derivatives not implemented"
                    .into(),
            ))
        }
    }
}

// ===========================================================================
// Packaged phase enthalpy / entropy under the system's model pair (M12.4).
// ===========================================================================

/// Molar enthalpy and entropy of one phase under the System's model pair,
/// relative to the ideal-gas reference at `(t_ref, p_ref)` (M12.4).
///
/// Dispatches on the phase model, so a γ-φ System no longer silently falls
/// back to (or errors on) the φ-φ EOS liquid path:
/// - **Vapor / cubic (φ-φ) liquid** → the EOS departure route
///   ([`crate::energy::phase_enthalpy_entropy`]).
/// - **Ideal-gas vapor** → the pure ideal-gas mixture terms.
/// - **γ-φ liquid** (activity / ideal solution) → ideal-gas enthalpy **minus
///   the Clausius–Clapeyron condensation enthalpy** `ΔH_vap,ᵢ = R·T²·
///   (dPsatᵢ/dT)/Psatᵢ` per component, **plus** the excess Hᴱ/Sᴱ. This is the
///   Ref (4) `TERMOIII.PAS:283/294` path Phase 14 deferred —
///   `// Ref (4): Da Silva & Báez (1989), legacy/pascal/TERMOIII.PAS`. The
///   entropy assembles in parallel with `ΔS_vap,ᵢ = ΔH_vap,ᵢ/T`.
///
/// Returns `(H [kJ/kmol], S [kJ/(kmol·K)])`.
#[allow(clippy::too_many_arguments)]
pub fn phase_enthalpy_entropy(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    comp: &[f64],
    phase: PhaseId,
    t_ref: f64,
    p_ref: f64,
    h_ref: &[f64],
    s_ref: &[f64],
) -> Result<(f64, f64), FlashError> {
    use crate::energy::{
        excess_h_s, ideal_enthalpy_mix, ideal_entropy_mix, phase_enthalpy_entropy as eos_hs,
    };
    const R: f64 = 8.31451; // kJ/(kmol·K)

    // Which phase model is active for this call?
    let cubic_eos = match phase {
        PhaseId::Vapor => match spec.vapor {
            VaporModel::Cubic(eos) => Some(eos),
            _ => None,
        },
        PhaseId::Liquid => match spec.liquid {
            LiquidModel::Cubic(eos) => Some(eos),
            _ => None,
        },
    };

    // φ-φ (cubic) phase: delegate to the EOS departure route.
    if let Some(eos) = cubic_eos {
        return eos_hs(
            &spec.mixture_spec(eos),
            t,
            p,
            comp,
            phase,
            t_ref,
            p_ref,
            h_ref,
            s_ref,
        )
        .map_err(|e| FlashError::Thermo(e.to_string()));
    }

    match phase {
        // Ideal-gas vapor: pure ideal-gas mixture terms (no residual).
        PhaseId::Vapor => match spec.vapor {
            VaporModel::IdealGas => Ok((
                ideal_enthalpy_mix(spec.components, comp, t, t_ref, h_ref),
                ideal_entropy_mix(spec.components, comp, t, p, t_ref, p_ref, s_ref),
            )),
            VaporModel::Virial => Err(FlashError::Unsupported(
                "phase_enthalpy_entropy: virial vapor enthalpy not implemented".into(),
            )),
            VaporModel::Cubic(_) => unreachable!("handled above"),
        },

        // γ-φ liquid: ideal − condensation + excess.
        PhaseId::Liquid => {
            // Ideal-gas mixture baseline (each component as an ideal gas).
            let h_ideal = ideal_enthalpy_mix(spec.components, comp, t, t_ref, h_ref);
            let s_ideal = ideal_entropy_mix(spec.components, comp, t, p, t_ref, p_ref, s_ref);
            // Condensation (Clausius–Clapeyron, Ref (4) TERMOIII.PAS:283/294):
            // ΔH_vap,ᵢ = R·T²·dln(Psatᵢ)/dT; the liquid sits ΔH_vap below the gas.
            let mut h_cond = 0.0;
            let mut s_cond = 0.0;
            for (i, &z_i) in comp.iter().enumerate() {
                let c = &spec.components[i];
                let psat_i =
                    psat(spec.sat_model(i), c, t).map_err(|e| FlashError::Thermo(e.to_string()))?;
                let dpsat_dt = crate::saturation::d_psat_dt(spec.sat_model(i), c, t)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?;
                let dh_vap = R * t * t * dpsat_dt / psat_i;
                h_cond += z_i * dh_vap;
                s_cond += z_i * (dh_vap / t); // ΔS_vap = ΔH_vap/T
            }
            // Excess (0 for the ideal solution).
            let (he, se) = match spec.liquid {
                LiquidModel::Activity(model) => {
                    excess_h_s(model, comp, spec.aij, spec.alpha, spec.vl, spec.delta, t)
                }
                _ => (0.0, 0.0),
            };
            Ok((h_ideal - h_cond + he, s_ideal - s_cond + se))
        }
    }
}

// ===========================================================================
// One stage's thermodynamics in a single call (OPTIMIZATION_PLAN_PART2 §10).
// ===========================================================================

/// Enthalpy and heat capacity of both phases at one `(T, P, x, y)` — the
/// scalar half of [`stage_thermo_into`]'s answer.
///
/// `#[derive(Clone, Copy)]` makes this a plain value type: four `f64`s that
/// are copied, never borrowed.
///
/// Python receives these as fields of `vle.StageThermoResult`, built from the
/// tuple `System.stage_thermo` returns (the same pattern as
/// [`KValueDerivs`]), so this struct carries no `pyclass` attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageThermo {
    /// Liquid molar enthalpy, **kJ/kmol**, relative to the ideal gas at `t_ref`.
    pub h_liquid: f64,
    /// Liquid isobaric heat capacity `(∂H/∂T)_{P,x}`, **kJ/(kmol·K)**.
    pub cp_liquid: f64,
    /// Vapor molar enthalpy, **kJ/kmol**, relative to the ideal gas at `t_ref`.
    pub h_vapor: f64,
    /// Vapor isobaric heat capacity `(∂H/∂T)_{P,y}`, **kJ/(kmol·K)**.
    pub cp_vapor: f64,
}

/// Everything a staged-separation solver asks of one stage, in one call:
/// `Kᵢ`, `∂ln Kᵢ/∂T`, and the enthalpy and heat capacity of both phases.
///
/// The same numbers come from five separate calls —
/// [`k_values_with_t_derivs`], [`phase_enthalpy_entropy`] twice and
/// [`phase_cp`] twice — and this function agrees with them to round-off
/// (asserted by test). It exists because those five calls each rebuild the
/// mixture at the same `(T, P)` and composition.
///
/// **Cubic liquid and cubic vapor (φ-φ):** one second-order dual pass per
/// phase ([`crate::mixture::ln_phi_t2_into`]) yields `ln φ̂ᵢ`, `∂ln φ̂ᵢ/∂T`
/// and the two derivatives of `G^R/(RT)` that are `H^R` and `Cp^R`. Measured
/// against the five calls on RKS alkanes: 544 ns vs 2.60 µs at n = 4 (4.8×)
/// and 786 ns vs 3.54 µs at n = 8 (4.5×) (`OPTIMIZATION_PLAN_PART2.md` §10).
///
/// **Every other model pair** (γ-φ, ideal-gas vapor, …): assembled from the
/// five calls above. Same answer, no speed-up; what those calls do not
/// support (virial vapor, the Chao-Seader family) is `Unsupported` here too.
///
/// # Arguments
/// * `t` — Temperature in **K**; `p` — pressure in **kPa absolute**.
/// * `x`, `y` — liquid and vapor mole fractions (length N, each sums to 1).
/// * `t_ref` — ideal-gas enthalpy reference temperature in **K**.
/// * `h_ref` — per-component reference enthalpy Hᵢ° in **kJ/kmol** (pass
///   `&[]` for the all-zero convention).
/// * `k` — receives Kᵢ, **dimensionless** (length N).
/// * `d_ln_k_d_t` — receives ∂ln Kᵢ/∂T in **1/K** (length N).
///
/// # Errors
/// [`FlashError::Dimension`] on a length mismatch; [`FlashError::Thermo`] if
/// a fugacity evaluation fails; [`FlashError::Unsupported`] as noted above.
#[allow(clippy::too_many_arguments)]
pub fn stage_thermo_into(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    x: &[f64],
    y: &[f64],
    t_ref: f64,
    h_ref: &[f64],
    k: &mut [f64],
    d_ln_k_d_t: &mut [f64],
) -> Result<StageThermo, FlashError> {
    use crate::energy::{ideal_cp, ideal_enthalpy_mix};
    const R: f64 = 8.31451; // kJ/(kmol·K)

    let n = spec.n();
    if x.len() != n || y.len() != n || k.len() != n || d_ln_k_d_t.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, x={}, y={}, k={}, d_ln_k_d_t={}",
            x.len(),
            y.len(),
            k.len(),
            d_ln_k_d_t.len()
        )));
    }

    // `if let … else` handles the one fast case and falls through otherwise.
    let (LiquidModel::Cubic(liq_eos), VaporModel::Cubic(vap_eos)) = (spec.liquid, spec.vapor)
    else {
        // Not φ-φ: assemble from the public calls. `p` stands in for the
        // entropy reference pressure, which the enthalpy does not depend on.
        let d = k_values_with_t_derivs(spec, t, p, x, y)?;
        k.copy_from_slice(&d.k);
        d_ln_k_d_t.copy_from_slice(&d.d_ln_k_d_t);
        let h = |comp: &[f64], phase| {
            phase_enthalpy_entropy(spec, t, p, comp, phase, t_ref, p, h_ref, &[]).map(|hs| hs.0)
        };
        return Ok(StageThermo {
            h_liquid: h(x, PhaseId::Liquid)?,
            cp_liquid: phase_cp(spec, t, p, x, PhaseId::Liquid)?,
            h_vapor: h(y, PhaseId::Vapor)?,
            cp_vapor: phase_cp(spec, t, p, y, PhaseId::Vapor)?,
        });
    };

    let mix_err = |e: crate::mixture::MixError| FlashError::Thermo(e.to_string());
    // The liquid pass writes straight into the caller's slices; the vapor
    // pass needs two scratch buffers. `SmallVec` keeps them on the stack up
    // to 8 components and moves to the heap above that.
    type Scratch = smallvec::SmallVec<[f64; 8]>;
    let mut vap_ln_phi: Scratch = smallvec::smallvec![0.0; n];
    let mut vap_dt: Scratch = smallvec::smallvec![0.0; n];
    let (gl1, gl2) = crate::mixture::ln_phi_t2_into(
        &spec.mixture_spec(liq_eos),
        t,
        p,
        x,
        PhaseId::Liquid,
        k,
        d_ln_k_d_t,
    )
    .map_err(mix_err)?;
    let (gv1, gv2) = crate::mixture::ln_phi_t2_into(
        &spec.mixture_spec(vap_eos),
        t,
        p,
        y,
        PhaseId::Vapor,
        &mut vap_ln_phi,
        &mut vap_dt,
    )
    .map_err(mix_err)?;
    // ln Kᵢ = ln φ̂ᵢᴸ − ln φ̂ᵢⱽ, and the same difference for the T-derivative.
    for i in 0..n {
        k[i] = (k[i] - vap_ln_phi[i]).exp();
        d_ln_k_d_t[i] -= vap_dt[i];
    }

    // Ideal-gas mixture Cp at a composition: Σ zᵢ·Cp°ᵢ(T).
    let cp_ideal = |z: &[f64]| -> f64 {
        z.iter()
            .zip(spec.components)
            .map(|(zi, c)| zi * ideal_cp(c, t))
            .sum()
    };
    // H = H_ideal + H^R with H^R = −R·T²·g′; Cp = Cp_ideal + Cp^R with
    // Cp^R = −R·(2·T·g′ + T²·g″)  — see `ln_phi_t2_into`.
    Ok(StageThermo {
        h_liquid: ideal_enthalpy_mix(spec.components, x, t, t_ref, h_ref) - R * t * t * gl1,
        cp_liquid: cp_ideal(x) - R * (2.0 * t * gl1 + t * t * gl2),
        h_vapor: ideal_enthalpy_mix(spec.components, y, t, t_ref, h_ref) - R * t * t * gv1,
        cp_vapor: cp_ideal(y) - R * (2.0 * t * gv1 + t * t * gv2),
    })
}

// ===========================================================================
// Packaged phase heat capacity under the system's model pair (M12.6).
// ===========================================================================

/// Isobaric heat capacity `Cp = (∂H/∂T)_{P,x}` of one phase under the
/// System's model pair, in **kJ/(kmol·K)** (M12.6) — the term-by-term
/// temperature derivative of [`phase_enthalpy_entropy`]'s `H`, so the two
/// agree to round-off for every model pair (asserted by test).
///
/// - **Cubic (φ-φ) phase** → [`crate::energy::phase_cp`] (ideal-gas mixture
///   Cp + the residual Cp from a second-order dual through the EOS).
/// - **Ideal-gas vapor** → `Σᵢ yᵢ Cp°ᵢ(T)`.
/// - **γ-φ liquid** (activity / ideal solution) → the derivative of the
///   shipped `H_L = H_ig − Σxᵢ ΔH_vap,ᵢ + Hᴱ`:
///   `Σxᵢ Cp°ᵢ − Σxᵢ d(ΔH_vap,ᵢ)/dT + Cpᴱ`, with the condensation term from
///   [`crate::saturation::condensation_cp`] (one second-order dual through the
///   saturation correlation) and `Cpᴱ` from [`crate::activity::excess_cp`]
///   (each model's own Hᴱ convention differentiated — see there).
/// - Virial vapor and the Chao–Seader-family liquids → `Unsupported`, as for
///   the enthalpy.
///
/// `t` in **K**, `p` in **kPa absolute**, `comp` mole fractions of the phase.
pub fn phase_cp(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    comp: &[f64],
    phase: PhaseId,
) -> Result<f64, FlashError> {
    use crate::activity::excess_cp;
    use crate::energy::{ideal_cp, phase_cp as eos_cp};
    use crate::saturation::condensation_cp;

    let n = spec.n();
    if comp.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, comp={}",
            comp.len()
        )));
    }
    let cubic_eos = match phase {
        PhaseId::Vapor => match spec.vapor {
            VaporModel::Cubic(eos) => Some(eos),
            _ => None,
        },
        PhaseId::Liquid => match spec.liquid {
            LiquidModel::Cubic(eos) => Some(eos),
            _ => None,
        },
    };
    if let Some(eos) = cubic_eos {
        return eos_cp(&spec.mixture_spec(eos), t, p, comp, phase)
            .map_err(|e| FlashError::Thermo(e.to_string()));
    }
    // Ideal-gas mixture heat capacity — the baseline of both remaining routes.
    let cp_ideal: f64 = comp
        .iter()
        .zip(spec.components)
        .map(|(z, c)| z * ideal_cp(c, t))
        .sum();
    match phase {
        PhaseId::Vapor => match spec.vapor {
            VaporModel::IdealGas => Ok(cp_ideal),
            VaporModel::Virial => Err(FlashError::Unsupported(
                "phase_cp: virial vapor heat capacity not implemented".into(),
            )),
            VaporModel::Cubic(_) => unreachable!("handled above"),
        },
        PhaseId::Liquid => match spec.liquid {
            LiquidModel::Activity(_) | LiquidModel::IdealSolution => {
                // −Σxᵢ d(ΔH_vap,ᵢ)/dT — the liquid sits ΔH_vap below the gas.
                let mut cp_cond = 0.0;
                for (i, &z_i) in comp.iter().enumerate() {
                    cp_cond += z_i
                        * condensation_cp(spec.sat_model(i), &spec.components[i], t)
                            .map_err(|e| FlashError::Thermo(e.to_string()))?;
                }
                let cp_e = match spec.liquid {
                    LiquidModel::Activity(model) => {
                        excess_cp(model, comp, spec.aij, spec.alpha, spec.vl, spec.delta, t)
                    }
                    _ => 0.0,
                };
                Ok(cp_ideal - cp_cond + cp_e)
            }
            LiquidModel::ChaoSeader | LiquidModel::GraysonStreed | LiquidModel::BraunK10 => {
                Err(FlashError::Unsupported(
                    "phase_cp: Chao-Seader / Grayson-Streed / Braun K10 liquid heat capacity \
                     not implemented"
                        .into(),
                ))
            }
            LiquidModel::Cubic(_) => unreachable!("handled above"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::ActivityModel;
    use crate::eos::CubicEos;

    fn n_butane() -> Component {
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            // Reduced Antoine ln(P/Pc)=a1−a2/(a3+T) fit (kPa, K).
            psat_coeffs: vec![4.35, 2277.0, -30.0],
            ..Component::default()
        }
    }

    fn n_heptane() -> Component {
        Component {
            name: "n-heptane".into(),
            tc: 540.2,
            pc: 2740.0,
            omega: 0.350,
            psat_coeffs: vec![4.02, 2911.0, -56.0],
            ..Component::default()
        }
    }

    fn classical<'a>(components: &'a [Component], kij: &'a [Vec<f64>]) -> SystemSpec<'a> {
        SystemSpec {
            components,
            vapor: VaporModel::Cubic(CubicEos::RKS1972),
            liquid: LiquidModel::Cubic(CubicEos::RKS1972),
            mixing_rule: MixingRule::Classical,
            kij,
            aij: &[],
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        }
    }

    #[test]
    fn phi_phi_k_values_finite_and_ordered() {
        // n-butane(1)/n-heptane(2) RKS both phases at 400 K, 500 kPa.
        // The lighter butane must have the larger K (more volatile).
        let comps = [n_butane(), n_heptane()];
        let spec = classical(&comps, &[]);
        let x = [0.3, 0.7];
        let y = [0.6, 0.4];
        let k = k_values(&spec, 400.0, 500.0, &x, &y).unwrap();
        assert_eq!(k.len(), 2);
        assert!(k.iter().all(|v| v.is_finite() && *v > 0.0));
        assert!(
            k[0] > k[1],
            "butane K={} should exceed heptane K={}",
            k[0],
            k[1]
        );
    }

    #[test]
    fn gamma_phi_ideal_solution_is_raoult() {
        // Ideal solution + ideal vapor ⇒ Kᵢ = Psat,ᵢ/P exactly (γ=1,
        // φ̂ⱽ=1, φˢᵃᵗ=1, no Poynting without vl).
        let comps = [n_butane(), n_heptane()];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::IdealSolution,
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &[],
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let x = [0.5, 0.5];
        let y = [0.5, 0.5];
        let k = k_values(&spec, 380.0, 300.0, &x, &y).unwrap();
        for (i, c) in comps.iter().enumerate() {
            let expect = psat(c.sat_model, c, 380.0).unwrap() / 300.0;
            assert!(
                (k[i] - expect).abs() < 1e-12,
                "comp {i}: {} vs {}",
                k[i],
                expect
            );
        }
    }

    #[test]
    fn gamma_phi_wilson_deviates_from_raoult() {
        // A non-ideal Wilson liquid must move K away from the ideal Raoult
        // value (γ ≠ 1). Use a methanol/water-like pair.
        let a = Component {
            name: "a".into(),
            tc: 512.6,
            pc: 8097.0,
            omega: 0.564,
            liquid_volume: 40.7,
            psat_coeffs: vec![5.20, 3200.0, -35.0],
            ..Component::default()
        };
        let b = Component {
            name: "b".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            liquid_volume: 18.07,
            psat_coeffs: vec![5.11, 3800.0, -46.0],
            ..Component::default()
        };
        let comps = [a, b];
        let aij = vec![vec![0.0, 1200.0], vec![-300.0, 0.0]];
        let vl = [40.7, 18.07];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::Wilson),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let x = [0.4, 0.6];
        let y = [0.5, 0.5];
        let k = k_values(&spec, 340.0, 100.0, &x, &y).unwrap();
        // Raoult reference (γ=1) — Poynting is on (vl provided), so compare
        // to γ·(Raoult·POY); the point is γ shifts it measurably.
        for (i, c) in comps.iter().enumerate() {
            let raoult = psat(c.sat_model, c, 340.0).unwrap() / 100.0;
            assert!(k[i].is_finite() && k[i] > 0.0);
            assert!(
                (k[i] / raoult - 1.0).abs() > 1e-3,
                "comp {i}: Wilson K {} too close to Raoult {}",
                k[i],
                raoult
            );
        }
    }

    /// `k_values` is now a thin `exp` over [`ln_k_values_into`], so the two
    /// must agree exactly — on both thermodynamic paths.
    #[test]
    fn ln_k_values_into_matches_k_values() {
        let comps = [n_butane(), n_heptane()];
        let x = [0.3, 0.7];
        let y = [0.6, 0.4];

        // φ-φ.
        let phi_phi = classical(&comps, &[]);
        // γ-φ with a real activity model, Poynting on, cubic vapor — the path
        // whose K assembly moved from products of exponentials to a log sum.
        let aij = vec![vec![0.0, 1200.0], vec![-300.0, 0.0]];
        let vl = [100.4, 147.5];
        let mut a = n_butane();
        a.liquid_volume = 100.4;
        let mut b = n_heptane();
        b.liquid_volume = 147.5;
        let gp_comps = [a, b];
        let gamma_phi = SystemSpec {
            components: &gp_comps,
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Activity(ActivityModel::Wilson),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };

        for (label, spec) in [("φ-φ", phi_phi), ("γ-φ", gamma_phi)] {
            let k = k_values(&spec, 400.0, 500.0, &x, &y).unwrap();
            let mut ln_k = vec![0.0; 2];
            ln_k_values_into(&spec, 400.0, 500.0, &x, &y, &mut ln_k).unwrap();
            for i in 0..2 {
                assert_eq!(ln_k[i].exp(), k[i], "{label}: K[{i}] disagrees");
            }
        }
    }

    /// `ln_k_values_into` must reject a wrongly-sized output slice rather than
    /// writing past the caller's buffer or silently filling part of it.
    #[test]
    fn ln_k_values_into_checks_output_length() {
        let comps = [n_butane(), n_heptane()];
        let spec = classical(&comps, &[]);
        let mut too_small = vec![0.0; 1];
        assert!(matches!(
            ln_k_values_into(
                &spec,
                400.0,
                500.0,
                &[0.3, 0.7],
                &[0.6, 0.4],
                &mut too_small
            ),
            Err(FlashError::Dimension(_))
        ));
    }

    /// The min-Gibbs root selection now builds one shared mixture state and
    /// evaluates both roots against it. That must give bit-identical answers to
    /// walking the whole mixture path once per root.
    #[test]
    fn min_gibbs_matches_independent_two_root_evaluation() {
        use crate::mixture::ln_phi_mix;
        let comps = [n_butane(), n_heptane()];
        let spec = classical(&comps, &[]);
        let ms = spec.mixture_spec(CubicEos::RKS1972);
        // A state with two distinct physical roots, so the choice is real.
        let (t, p) = (400.0, 1500.0);
        for w in [[0.5, 0.5], [0.9, 0.1], [0.15, 0.85]] {
            let got = min_gibbs_ln_phi(&spec, t, p, &w).unwrap();
            // Reference: evaluate each root independently and pick the lower g.
            let mut best: Option<(f64, Vec<f64>)> = None;
            for phase in [PhaseId::Liquid, PhaseId::Vapor] {
                if let Ok(lnphi) = ln_phi_mix(&ms, t, p, &w, phase) {
                    let g: f64 = (0..w.len())
                        .filter(|&i| w[i] > 0.0)
                        .map(|i| w[i] * (w[i].ln() + lnphi[i]))
                        .sum();
                    if best.as_ref().is_none_or(|(bg, _)| g < *bg) {
                        best = Some((g, lnphi));
                    }
                }
            }
            let expect = best.expect("a physical root exists here").1;
            for i in 0..w.len() {
                assert_eq!(got[i], expect[i], "w={w:?} comp {i}");
            }
        }
    }

    #[test]
    fn dimension_mismatch_errors() {
        let comps = [n_butane(), n_heptane()];
        let spec = classical(&comps, &[]);
        assert!(matches!(
            k_values(&spec, 400.0, 500.0, &[1.0], &[0.5, 0.5]),
            Err(FlashError::Dimension(_))
        ));
    }

    // -----------------------------------------------------------------
    // K-value T/P derivatives (§L step 3, M12.3).
    // -----------------------------------------------------------------

    /// Central-difference ∂ln Kᵢ/∂T oracle.
    fn dlnk_dt_fd(spec: &SystemSpec, t: f64, p: f64, x: &[f64], y: &[f64], h: f64) -> Vec<f64> {
        let hi = k_values(spec, t + h, p, x, y).unwrap();
        let lo = k_values(spec, t - h, p, x, y).unwrap();
        hi.iter()
            .zip(&lo)
            .map(|(a, b)| (a.ln() - b.ln()) / (2.0 * h))
            .collect()
    }

    /// Central-difference ∂ln Kᵢ/∂P oracle.
    fn dlnk_dp_fd(spec: &SystemSpec, t: f64, p: f64, x: &[f64], y: &[f64], h: f64) -> Vec<f64> {
        let hi = k_values(spec, t, p + h, x, y).unwrap();
        let lo = k_values(spec, t, p - h, x, y).unwrap();
        hi.iter()
            .zip(&lo)
            .map(|(a, b)| (a.ln() - b.ln()) / (2.0 * h))
            .collect()
    }

    fn assert_k_derivs_match_fd(
        spec: &SystemSpec,
        t: f64,
        p: f64,
        x: &[f64],
        y: &[f64],
        label: &str,
    ) {
        let kv = k_values_with_derivs(spec, t, p, x, y).unwrap();
        let k_ref = k_values(spec, t, p, x, y).unwrap();
        // K field bit-identical to k_values.
        for (i, &k_i) in k_ref.iter().enumerate() {
            assert_eq!(kv.k[i], k_i, "{label}: K[{i}] not bit-identical");
        }
        let fd_t = dlnk_dt_fd(spec, t, p, x, y, 1e-3);
        let fd_p = dlnk_dp_fd(spec, t, p, x, y, 1e-2);
        for i in 0..k_ref.len() {
            let tol_t = 1e-6 * kv.d_ln_k_d_t[i].abs().max(1e-6) + 1e-9;
            assert!(
                (kv.d_ln_k_d_t[i] - fd_t[i]).abs() <= tol_t,
                "{label}: ∂lnK{i}/∂T exact={} fd={}",
                kv.d_ln_k_d_t[i],
                fd_t[i]
            );
            let tol_p = 1e-6 * kv.d_ln_k_d_p[i].abs().max(1e-6) + 1e-12;
            assert!(
                (kv.d_ln_k_d_p[i] - fd_p[i]).abs() <= tol_p,
                "{label}: ∂lnK{i}/∂P exact={} fd={}",
                kv.d_ln_k_d_p[i],
                fd_p[i]
            );
        }
    }

    #[test]
    fn k_derivs_phi_phi_match_fd() {
        // Cubic both phases (RKS) — the isothermal-flash validation path.
        let comps = [n_butane(), n_heptane()];
        let spec = classical(&comps, &[]);
        assert_k_derivs_match_fd(&spec, 400.0, 500.0, &[0.3, 0.7], &[0.6, 0.4], "φ-φ RKS");
    }

    #[test]
    fn k_derivs_gamma_phi_wilson_ideal_vapor_match_fd() {
        // γ-φ with a Wilson liquid (real ∂lnγ/∂T) and ideal-gas vapor +
        // Poynting — the modified-Raoult term list end to end.
        let a = Component {
            name: "a".into(),
            tc: 512.6,
            pc: 8097.0,
            omega: 0.564,
            liquid_volume: 40.7,
            psat_coeffs: vec![5.20, 3200.0, -35.0],
            ..Component::default()
        };
        let b = Component {
            name: "b".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            liquid_volume: 18.07,
            psat_coeffs: vec![5.11, 3800.0, -46.0],
            ..Component::default()
        };
        let comps = [a, b];
        let aij = vec![vec![0.0, 1200.0], vec![-300.0, 0.0]];
        let vl = [40.7, 18.07];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::Wilson),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        assert_k_derivs_match_fd(
            &spec,
            340.0,
            100.0,
            &[0.4, 0.6],
            &[0.5, 0.5],
            "γ-φ Wilson/ideal",
        );
    }

    #[test]
    fn k_derivs_gamma_phi_cubic_vapor_match_fd() {
        // γ-φ with a CUBIC vapor exercises the φᵢˢᵃᵗ(T) chain term
        // (dln_phi_sat_dt) and the cubic vapor mixture derivative together.
        // Well-behaved hydrocarbon pair at a moderate state so both the vapor
        // cubic roots and the Poynting reference stay physical.
        let mut a = n_butane();
        a.liquid_volume = 100.4;
        let mut b = n_heptane();
        b.liquid_volume = 147.5;
        let comps = [a, b];
        let aij = vec![vec![0.0, 0.15], vec![0.12, 0.0]]; // mild van Laar
        let vl = [100.4, 147.5];
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        assert_k_derivs_match_fd(
            &spec,
            400.0,
            500.0,
            &[0.4, 0.6],
            &[0.55, 0.45],
            "γ-φ vanLaar/PR",
        );
    }

    // === Refinery K-value methods (M20) ==================================

    fn refinery_pair() -> [Component; 2] {
        // n-butane / n-heptane with regular-solution data (δ in (cal/cm³)^½,
        // Vᴸ in cm³/mol) and boiling points, so every M20 liquid model applies.
        let mut b = n_butane();
        b.solubility_param = 6.73;
        b.liquid_volume = 101.4;
        b.tb = 272.65;
        let mut h = n_heptane();
        h.solubility_param = 7.43;
        h.liquid_volume = 147.5;
        h.tb = 371.55;
        [b, h]
    }

    #[test]
    fn grayson_streed_k_is_nu_times_gamma_over_phi() {
        let comps = refinery_pair();
        let mut spec = classical(&comps, &[]);
        spec.liquid = LiquidModel::GraysonStreed;
        let (t, p) = (400.0, 800.0);
        let x = [0.4, 0.6];
        let y = [0.7, 0.3];
        let k = k_values(&spec, t, p, &x, &y).unwrap();
        // Assemble by hand from the public pieces.
        let mut vap = vec![0.0; 2];
        vapor_ln_phi_into(&spec, t, p, &y, &mut vap).unwrap();
        let vl = [101.4, 147.5];
        let delta = [6.73, 7.43];
        let mut lg = vec![0.0; 2];
        ln_gamma_all(
            crate::activity::ActivityModel::ScatchardHildebrand,
            &x,
            &[],
            &[],
            &vl,
            &delta,
            t,
            &mut lg,
        );
        for i in 0..2 {
            let ln_nu = crate::eos::regular_solution_ln_nu(
                crate::eos::RegularSolutionSet::GraysonStreed1963,
                t,
                p,
                &comps[i],
                crate::eos::ChaoSeaderSpecies::Normal,
            );
            let want = (ln_nu + lg[i] - vap[i]).exp();
            assert!(
                (k[i] - want).abs() < 1e-12 * want,
                "K[{i}] = {} vs {want}",
                k[i]
            );
        }
        assert!(
            k[0] > 1.0 && k[1] < 1.0,
            "butane volatile, heptane heavy: {k:?}"
        );
        // The regular-solution γ is a real contribution (not identically 1).
        assert!(lg.iter().any(|g| g.abs() > 1e-4), "ln γ = {lg:?}");
    }

    #[test]
    fn grayson_streed_cached_path_matches_the_direct_path_and_flashes() {
        let comps = refinery_pair();
        let mut spec = classical(&comps, &[]);
        spec.liquid = LiquidModel::GraysonStreed;
        let (t, p) = (380.0, 600.0);
        let cache = SystemTpCache::new(&spec, t, p).unwrap();
        let x = [0.3, 0.7];
        let y = [0.8, 0.2];
        let mut a = vec![0.0; 2];
        let mut b = vec![0.0; 2];
        ln_k_values_into(&spec, t, p, &x, &y, &mut a).unwrap();
        ln_k_values_cached_into(&spec, &cache, &x, &y, &mut b).unwrap();
        for i in 0..2 {
            assert!((a[i] - b[i]).abs() < 1e-14, "{a:?} vs {b:?}");
        }
        // And the isothermal flash converges on it.
        let r = crate::flash::isothermal::flash_isothermal(&spec, t, p, &[0.5, 0.5], 1e-10, 200)
            .unwrap();
        assert!(r.two_phase, "{r:?}");
        for i in 0..2 {
            let bal = r.beta * r.y[i] + (1.0 - r.beta) * r.x[i];
            assert!((bal - 0.5).abs() < 1e-9);
        }
    }

    #[test]
    fn grayson_streed_without_solubility_data_degrades_to_gamma_one() {
        // Components with no δ/Vᴸ: γ ≡ 1 and Grayson-Streed equals the legacy
        // Chao-Seader path (same ν table, no γ). Documented degradation.
        let comps = [n_butane(), n_heptane()];
        let mut gs = classical(&comps, &[]);
        gs.liquid = LiquidModel::GraysonStreed;
        let mut cs = classical(&comps, &[]);
        cs.liquid = LiquidModel::ChaoSeader;
        let (x, y) = ([0.4, 0.6], [0.7, 0.3]);
        let a = k_values(&gs, 400.0, 800.0, &x, &y).unwrap();
        let b = k_values(&cs, 400.0, 800.0, &x, &y).unwrap();
        for i in 0..2 {
            assert!((a[i] - b[i]).abs() < 1e-12 * b[i]);
        }
    }

    #[test]
    fn braun_k10_is_maxwell_bonnell_over_pressure_with_an_ideal_vapor() {
        let comps = refinery_pair();
        let mut spec = classical(&comps, &[]);
        spec.vapor = VaporModel::IdealGas;
        spec.liquid = LiquidModel::BraunK10;
        let (t, p) = (350.0, 120.0);
        let k = k_values(&spec, t, p, &[0.5, 0.5], &[0.5, 0.5]).unwrap();
        for i in 0..2 {
            let want = crate::petroleum::vapor_pressure(t, comps[i].tb, None).unwrap() / p;
            assert!(
                (k[i] - want).abs() < 1e-12 * want,
                "K[{i}] {} vs {want}",
                k[i]
            );
        }
        // Cached path agrees.
        let cache = SystemTpCache::new(&spec, t, p).unwrap();
        let mut b = vec![0.0; 2];
        ln_k_values_cached_into(&spec, &cache, &[0.5, 0.5], &[0.5, 0.5], &mut b).unwrap();
        for i in 0..2 {
            assert!((b[i].exp() - k[i]).abs() < 1e-12 * k[i]);
        }
        // A component's Watson K, when set, changes the answer (the correction
        // is applied), and a missing Tb is an error, not a silent zero.
        let mut with_kw = refinery_pair();
        with_kw[1].watson_k = 11.0; // heptane: Tb above the ramp's 366.5 K onset
        let mut s2 = classical(&with_kw, &[]);
        s2.vapor = VaporModel::IdealGas;
        s2.liquid = LiquidModel::BraunK10;
        let k2 = k_values(&s2, t, 50.0, &[0.5, 0.5], &[0.5, 0.5]).unwrap();
        let k1 = k_values(&spec, t, 50.0, &[0.5, 0.5], &[0.5, 0.5]).unwrap();
        assert!(
            (k2[1] - k1[1]).abs() > 1e-6 * k1[1],
            "Watson-K correction had no effect"
        );
        assert!((k2[0] - k1[0]).abs() < 1e-12 * k1[0]);
        let no_tb = [n_butane(), n_heptane()];
        let mut s3 = classical(&no_tb, &[]);
        s3.liquid = LiquidModel::BraunK10;
        assert!(k_values(&s3, t, p, &[0.5, 0.5], &[0.5, 0.5]).is_err());
    }

    // -----------------------------------------------------------------
    // M12.6 — packaged phase heat capacity
    // -----------------------------------------------------------------

    /// Butane / heptane with ideal-gas Cp polynomials, so the ideal term is
    /// live (the plain fixtures carry zeros).
    fn cp_pair() -> [Component; 2] {
        let mut a = n_butane();
        a.cp_coeffs = [1.935, 3.685e-2, -1.14e-5, 0.0, 0.0]; // Cp°/R shape
        a.liquid_volume = 100.4;
        let mut b = n_heptane();
        b.cp_coeffs = [3.15, 5.7e-2, -1.6e-5, 0.0, 0.0];
        b.liquid_volume = 147.5;
        [a, b]
    }

    /// `phase_cp` is the T-derivative of the shipped `phase_enthalpy_entropy`
    /// H for **every** model pair it supports: γ-φ liquid (van Laar / ideal
    /// gas; Wilson / cubic vapor; NRTL), ideal-gas vapor, and the cubic
    /// phases (unchanged: equals `energy::phase_cp`).
    #[test]
    fn phase_cp_matches_fd_of_phase_enthalpy_for_every_route() {
        let comps = cp_pair();
        let x = [0.4, 0.6];
        let (t, p) = (350.0, 300.0);
        let h = 0.05;
        let a_vl = vec![vec![0.0, 0.7], vec![1.1, 0.0]];
        let a_wilson = vec![vec![0.0, 1200.0], vec![-300.0, 0.0]];
        let a_nrtl = vec![vec![0.0, 2400.0], vec![-1100.0, 0.0]];
        let alpha = vec![vec![0.0, 0.3], vec![0.3, 0.0]];
        let vl = [100.4, 147.5];
        let mut van_laar = classical(&comps, &[]);
        van_laar.vapor = VaporModel::IdealGas;
        van_laar.liquid = LiquidModel::Activity(ActivityModel::VanLaar);
        van_laar.aij = &a_vl;
        let mut wilson_cubic = classical(&comps, &[]);
        wilson_cubic.vapor = VaporModel::Cubic(CubicEos::PR1976);
        wilson_cubic.liquid = LiquidModel::Activity(ActivityModel::Wilson);
        wilson_cubic.aij = &a_wilson;
        wilson_cubic.vl = &vl;
        let mut nrtl = classical(&comps, &[]);
        nrtl.vapor = VaporModel::IdealGas;
        nrtl.liquid = LiquidModel::Activity(ActivityModel::Nrtl);
        nrtl.aij = &a_nrtl;
        nrtl.alpha = &alpha;
        let mut ideal_sol = classical(&comps, &[]);
        ideal_sol.vapor = VaporModel::IdealGas;
        ideal_sol.liquid = LiquidModel::IdealSolution;
        let phi_phi = classical(&comps, &[]);
        for (label, spec) in [
            ("van Laar / ideal gas", &van_laar),
            ("Wilson / PR vapor", &wilson_cubic),
            ("NRTL / ideal gas", &nrtl),
            ("ideal solution / ideal gas", &ideal_sol),
            ("φ-φ RKS", &phi_phi),
        ] {
            for phase in [PhaseId::Liquid, PhaseId::Vapor] {
                let hh = |tt: f64| {
                    phase_enthalpy_entropy(spec, tt, p, &x, phase, 298.15, 101.325, &[], &[])
                        .unwrap()
                        .0
                };
                let fd = (hh(t + h) - hh(t - h)) / (2.0 * h);
                let cp = phase_cp(spec, t, p, &x, phase).unwrap();
                assert!(
                    (cp - fd).abs() < 1e-6 * fd.abs().max(1.0),
                    "{label} {phase:?}: Cp {cp} vs FD {fd}"
                );
                assert!(cp > 0.0, "{label} {phase:?}: Cp = {cp}");
            }
        }
        // The cubic route is the M12.4 function, untouched.
        let direct = crate::energy::phase_cp(
            &phi_phi.mixture_spec(CubicEos::RKS1972),
            t,
            p,
            &x,
            PhaseId::Liquid,
        )
        .unwrap();
        assert_eq!(
            phase_cp(&phi_phi, t, p, &x, PhaseId::Liquid).unwrap(),
            direct
        );
        // Ideal-gas vapor is exactly Σ y Cp°.
        let cp_ig: f64 = x
            .iter()
            .zip(&comps)
            .map(|(z, c)| z * crate::energy::ideal_cp(c, t))
            .sum();
        assert_eq!(
            phase_cp(&van_laar, t, p, &x, PhaseId::Vapor).unwrap(),
            cp_ig
        );
        // Unsupported routes say so.
        let mut cs = classical(&comps, &[]);
        cs.liquid = LiquidModel::ChaoSeader;
        assert!(matches!(
            phase_cp(&cs, t, p, &x, PhaseId::Liquid),
            Err(FlashError::Unsupported(_))
        ));
        assert!(matches!(
            phase_cp(&van_laar, t, p, &[1.0], PhaseId::Liquid),
            Err(FlashError::Dimension(_))
        ));
    }

    /// The five public calls `stage_thermo_into` replaces, assembled the way
    /// a consumer made them before it existed. This is the test oracle: it
    /// shares no code with the single-pass route above the mixture core.
    fn stage_thermo_by_separate_calls(
        spec: &SystemSpec,
        t: f64,
        p: f64,
        x: &[f64],
        y: &[f64],
        t_ref: f64,
        h_ref: &[f64],
    ) -> (Vec<f64>, Vec<f64>, StageThermo) {
        let d = k_values_with_derivs(spec, t, p, x, y).unwrap();
        let h = |c: &[f64], ph| {
            phase_enthalpy_entropy(spec, t, p, c, ph, t_ref, 101.325, h_ref, &[])
                .unwrap()
                .0
        };
        let st = StageThermo {
            h_liquid: h(x, PhaseId::Liquid),
            cp_liquid: phase_cp(spec, t, p, x, PhaseId::Liquid).unwrap(),
            h_vapor: h(y, PhaseId::Vapor),
            cp_vapor: phase_cp(spec, t, p, y, PhaseId::Vapor).unwrap(),
        };
        (d.k, d.d_ln_k_d_t, st)
    }

    /// Assert the single call and the separate calls agree to `tol`
    /// (relative, with an absolute floor of `tol` for values near zero).
    #[allow(clippy::too_many_arguments)]
    fn assert_stage_thermo_agrees(
        label: &str,
        spec: &SystemSpec,
        t: f64,
        p: f64,
        x: &[f64],
        y: &[f64],
        h_ref: &[f64],
        tol: f64,
    ) {
        let n = spec.n();
        let (k0, d0, s0) = stage_thermo_by_separate_calls(spec, t, p, x, y, 298.15, h_ref);
        let (mut k, mut d) = (vec![0.0; n], vec![0.0; n]);
        let s = stage_thermo_into(spec, t, p, x, y, 298.15, h_ref, &mut k, &mut d).unwrap();
        let close = |a: f64, b: f64| (a - b).abs() <= tol * b.abs().max(1.0);
        for i in 0..n {
            assert!(close(k[i], k0[i]), "{label}: K[{i}] {} vs {}", k[i], k0[i]);
            assert!(
                (d[i] - d0[i]).abs() <= tol * d0[i].abs().max(1e-3),
                "{label}: dlnK/dT[{i}] {} vs {}",
                d[i],
                d0[i]
            );
        }
        for (name, a, b) in [
            ("H_L", s.h_liquid, s0.h_liquid),
            ("Cp_L", s.cp_liquid, s0.cp_liquid),
            ("H_V", s.h_vapor, s0.h_vapor),
            ("Cp_V", s.cp_vapor, s0.cp_vapor),
        ] {
            assert!(close(a, b), "{label}: {name} {a} vs {b}");
        }
    }

    /// `stage_thermo_into` returns what the five separate public calls
    /// return, for the single-pass φ-φ route (classical with and without
    /// kij, two EOS, a GE mixing rule, a 3-parameter EOS, non-zero reference
    /// enthalpies) and for the assembled route (γ-φ, mixed).
    #[test]
    fn stage_thermo_matches_the_five_separate_calls() {
        let comps = cp_pair();
        let (x, y) = ([0.4, 0.6], [0.75, 0.25]);
        let (t, p) = (350.0, 300.0);
        let kij = vec![vec![0.0, 0.03], vec![0.03, 0.0]];
        let a_wilson = vec![vec![0.0, 1200.0], vec![-300.0, 0.0]];
        let a_vl = vec![vec![0.0, 0.7], vec![1.1, 0.0]];
        let vl = [100.4, 147.5];

        let rks = classical(&comps, &[]);
        let rks_kij = classical(&comps, &kij);
        let mut pr = classical(&comps, &kij);
        pr.vapor = VaporModel::Cubic(CubicEos::PR1976);
        pr.liquid = LiquidModel::Cubic(CubicEos::PR1976);
        let mut patel_teja = classical(&comps, &[]);
        patel_teja.vapor = VaporModel::Cubic(CubicEos::PatelTeja);
        patel_teja.liquid = LiquidModel::Cubic(CubicEos::PatelTeja);
        let mut wong_sandler = pr;
        wong_sandler.mixing_rule = MixingRule::WongSandler;
        wong_sandler.aij = &a_wilson;
        wong_sandler.vl = &vl;
        wong_sandler.ge_model = Some(ActivityModel::Wilson);
        // Assembled route: γ-φ, and an activity liquid under a cubic vapor.
        let mut van_laar = classical(&comps, &[]);
        van_laar.vapor = VaporModel::IdealGas;
        van_laar.liquid = LiquidModel::Activity(ActivityModel::VanLaar);
        van_laar.aij = &a_vl;
        let mut wilson_cubic = classical(&comps, &[]);
        wilson_cubic.vapor = VaporModel::Cubic(CubicEos::PR1976);
        wilson_cubic.liquid = LiquidModel::Activity(ActivityModel::Wilson);
        wilson_cubic.aij = &a_wilson;
        wilson_cubic.vl = &vl;

        for (label, spec) in [
            ("RKS", &rks),
            ("RKS + kij", &rks_kij),
            ("PR + kij", &pr),
            ("Patel-Teja", &patel_teja),
            ("PR Wong-Sandler", &wong_sandler),
            ("van Laar / ideal gas", &van_laar),
            ("Wilson / PR vapor", &wilson_cubic),
        ] {
            assert_stage_thermo_agrees(label, spec, t, p, &x, &y, &[], 1e-10);
        }
        // Reference enthalpies shift H and nothing else.
        assert_stage_thermo_agrees("RKS + h_ref", &rks, t, p, &x, &y, &[1500.0, -700.0], 1e-10);
        assert_stage_thermo_agrees(
            "van Laar + h_ref",
            &van_laar,
            t,
            p,
            &x,
            &y,
            &[1500.0, -700.0],
            1e-10,
        );
    }

    /// Eleven components: above the 8-wide inline buffers, so the scratch
    /// vectors live on the heap. Same agreement required.
    #[test]
    fn stage_thermo_matches_separate_calls_at_eleven_components() {
        let [a, b] = cp_pair();
        // Eleven pseudo-components interpolated between n-butane and
        // n-heptane, so each has distinct critical constants and Cp°.
        let comps: Vec<Component> = (0..11)
            .map(|i| {
                let f = i as f64 / 10.0;
                let mix = |u: f64, v: f64| u + f * (v - u);
                Component {
                    name: format!("pseudo-{i}"),
                    tc: mix(a.tc, b.tc),
                    pc: mix(a.pc, b.pc),
                    omega: mix(a.omega, b.omega),
                    cp_coeffs: [
                        mix(a.cp_coeffs[0], b.cp_coeffs[0]),
                        mix(a.cp_coeffs[1], b.cp_coeffs[1]),
                        mix(a.cp_coeffs[2], b.cp_coeffs[2]),
                        0.0,
                        0.0,
                    ],
                    ..Component::default()
                }
            })
            .collect();
        let norm = |v: Vec<f64>| -> Vec<f64> {
            let s: f64 = v.iter().sum();
            v.into_iter().map(|u| u / s).collect()
        };
        let x = norm((0..11).map(|i| 1.0 + i as f64).collect());
        let y = norm((0..11).map(|i| 11.0 - i as f64).collect());
        let spec = classical(&comps, &[]);
        assert_stage_thermo_agrees("RKS n=11", &spec, 400.0, 800.0, &x, &y, &[], 1e-10);
    }

    /// Wrong-length inputs and unsupported models are errors, not panics.
    #[test]
    fn stage_thermo_rejects_bad_lengths_and_unsupported_models() {
        let comps = cp_pair();
        let spec = classical(&comps, &[]);
        let (mut k, mut d) = (vec![0.0; 2], vec![0.0; 2]);
        let mut short = vec![0.0; 1];
        assert!(matches!(
            stage_thermo_into(
                &spec,
                350.0,
                300.0,
                &[1.0],
                &[0.5, 0.5],
                298.15,
                &[],
                &mut k,
                &mut d
            ),
            Err(FlashError::Dimension(_))
        ));
        assert!(matches!(
            stage_thermo_into(
                &spec,
                350.0,
                300.0,
                &[0.5, 0.5],
                &[0.5, 0.5],
                298.15,
                &[],
                &mut short,
                &mut d
            ),
            Err(FlashError::Dimension(_))
        ));
        let mut cs = classical(&comps, &[]);
        cs.liquid = LiquidModel::ChaoSeader;
        assert!(matches!(
            stage_thermo_into(
                &cs,
                350.0,
                300.0,
                &[0.5, 0.5],
                &[0.5, 0.5],
                298.15,
                &[],
                &mut k,
                &mut d
            ),
            Err(FlashError::Unsupported(_))
        ));
    }

    /// `k_values_with_t_derivs` is the full call minus the pressure column.
    #[test]
    fn t_only_derivs_equal_the_full_call() {
        let comps = cp_pair();
        let spec = classical(&comps, &[]);
        let (x, y) = ([0.4, 0.6], [0.75, 0.25]);
        let full = k_values_with_derivs(&spec, 350.0, 300.0, &x, &y).unwrap();
        let t_only = k_values_with_t_derivs(&spec, 350.0, 300.0, &x, &y).unwrap();
        assert_eq!(t_only.k, full.k);
        assert_eq!(t_only.d_ln_k_d_t, full.d_ln_k_d_t);
        assert!(full.d_ln_k_d_p.iter().all(|v| *v != 0.0));
        assert!(t_only.d_ln_k_d_p.iter().all(|v| *v == 0.0));
    }

    /// `d_ln_gamma_d_n` against central differences in the mole numbers, for
    /// every activity model with a composition dependence, and the two
    /// identities an exact Jacobian must satisfy: Σⱼ xⱼ·Gᵢⱼ = 0 (ln γ is
    /// intensive) and Σᵢ xᵢ·Gᵢⱼ = 0 (Gibbs–Duhem).
    #[test]
    fn d_ln_gamma_d_n_matches_finite_differences() {
        let comps = cp_pair();
        let a_vl = vec![vec![0.0, 0.7], vec![1.1, 0.0]];
        let a_wilson = vec![vec![0.0, 1200.0], vec![-300.0, 0.0]];
        let a_nrtl = vec![vec![0.0, 2400.0], vec![-1100.0, 0.0]];
        let alpha = vec![vec![0.0, 0.3], vec![0.3, 0.0]];
        let vl = [100.4, 147.5];
        let delta = [14.1, 15.3];
        let base = classical(&comps, &[]);
        let cases = [
            (ActivityModel::VanLaar, &a_vl),
            (ActivityModel::Margules, &a_vl),
            (ActivityModel::Wilson, &a_wilson),
            (ActivityModel::Nrtl, &a_nrtl),
            (ActivityModel::ScatchardHildebrand, &a_vl),
        ];
        let t = 350.0;
        for (model, aij) in cases {
            let spec = SystemSpec {
                liquid: LiquidModel::Activity(model),
                aij,
                alpha: &alpha,
                vl: &vl,
                delta: &delta,
                ..base
            };
            let ln_gamma_at = |n: &[f64]| -> Vec<f64> {
                let tot: f64 = n.iter().sum();
                let x: Vec<f64> = n.iter().map(|v| v / tot).collect();
                let mut out = vec![0.0; n.len()];
                crate::activity::ln_gamma_all(
                    model, &x, spec.aij, spec.alpha, spec.vl, spec.delta, t, &mut out,
                );
                out
            };
            for x in [[0.4, 0.6], [0.05, 0.95], [0.9, 0.1]] {
                let g = d_ln_gamma_d_n(&spec, model, t, &x);
                let h = 1e-6;
                for j in 0..2 {
                    let (mut hi, mut lo) = (x, x);
                    hi[j] += h;
                    lo[j] -= h;
                    let (fh, fl) = (ln_gamma_at(&hi), ln_gamma_at(&lo));
                    for i in 0..2 {
                        let fd = (fh[i] - fl[i]) / (2.0 * h);
                        assert!(
                            (g[i][j] - fd).abs() < 1e-6 * (1.0 + fd.abs()),
                            "{model:?} x={x:?} G[{i}][{j}] = {} vs FD {fd}",
                            g[i][j]
                        );
                    }
                    let gd: f64 = (0..2).map(|i| x[i] * g[i][j]).sum();
                    assert!(gd.abs() < 1e-10, "{model:?} Gibbs–Duhem column {j}: {gd}");
                }
                for (i, row) in g.iter().enumerate() {
                    let s: f64 = row.iter().zip(&x).map(|(a, b)| a * b).sum();
                    assert!(s.abs() < 1e-10, "{model:?} row {i} sum {s}");
                }
            }
        }
    }

    /// `k_values_with_p_derivs` returns the same K and ∂ln K/∂P as the full
    /// call, for a φ-φ and a γ-φ pair.
    #[test]
    fn p_only_derivatives_match_the_full_call() {
        let comps = cp_pair();
        let a_wilson = vec![vec![0.0, 1200.0], vec![-300.0, 0.0]];
        let vl = [100.4, 147.5];
        let mut wilson_cubic = classical(&comps, &[]);
        wilson_cubic.vapor = VaporModel::Cubic(CubicEos::PR1976);
        wilson_cubic.liquid = LiquidModel::Activity(ActivityModel::Wilson);
        wilson_cubic.aij = &a_wilson;
        wilson_cubic.vl = &vl;
        for spec in [classical(&comps, &[]), wilson_cubic] {
            let (x, y) = ([0.3, 0.7], [0.7, 0.3]);
            let full = k_values_with_derivs(&spec, 380.0, 600.0, &x, &y).unwrap();
            let p_only = k_values_with_p_derivs(&spec, 380.0, 600.0, &x, &y).unwrap();
            assert_eq!(full.k, p_only.k);
            assert_eq!(full.d_ln_k_d_p, p_only.d_ln_k_d_p);
        }
    }
}
