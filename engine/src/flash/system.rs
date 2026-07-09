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
use crate::mixture::{GeSpec, MixtureSpec, ln_phi_mix};
use crate::saturation::{SatPressureModel, poynting_factor, psat};
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

/// ln φ̂ᵢ of every component in the **vapor** phase of composition `y`.
fn vapor_ln_phi(spec: &SystemSpec, t: f64, p: f64, y: &[f64]) -> Result<Vec<f64>, FlashError> {
    match spec.vapor {
        VaporModel::IdealGas => Ok(vec![0.0; spec.n()]),
        VaporModel::Virial => ln_phi_mix_virial(spec.components, y, t, p)
            .map_err(|e| FlashError::Thermo(e.to_string())),
        VaporModel::Cubic(eos) => ln_phi_mix(&spec.mixture_spec(eos), t, p, y, PhaseId::Vapor)
            .map_err(|e| FlashError::Thermo(e.to_string())),
    }
}

/// Pure-component saturated-vapor fugacity coefficient φᵢˢᵃᵗ at (T, Psat,ᵢ),
/// the Poynting-reference correction for the γ-φ path. Returns 1 (ln = 0)
/// for an ideal vapor.
fn pure_sat_phi(spec: &SystemSpec, i: usize, t: f64, psat_i: f64) -> f64 {
    let comp = &spec.components[i];
    let ln_phi = match spec.vapor {
        VaporModel::IdealGas => 0.0,
        VaporModel::Virial => ln_phi_pure_virial(comp, t, psat_i),
        VaporModel::Cubic(eos) => ln_phi_pure(eos, t, psat_i, comp, PhaseId::Vapor).unwrap_or(0.0),
    };
    ln_phi.exp()
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
    let n = spec.n();
    if x.len() != n || y.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, x={}, y={}",
            x.len(),
            y.len()
        )));
    }
    let vap = vapor_ln_phi(spec, t, p, y)?;

    match spec.liquid {
        // --- φ-φ: EOS both phases ---
        LiquidModel::Cubic(eos) => {
            let liq = ln_phi_mix(&spec.mixture_spec(eos), t, p, x, PhaseId::Liquid)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
            Ok((0..n).map(|i| (liq[i] - vap[i]).exp()).collect())
        }

        // --- γ-φ: activity-model liquid ---
        LiquidModel::Activity(model) => {
            let mut ln_gamma = vec![0.0; n];
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
            gamma_phi_k(spec, t, p, &ln_gamma, &vap)
        }
        LiquidModel::IdealSolution => {
            let ln_gamma = vec![0.0; n]; // γ = 1
            gamma_phi_k(spec, t, p, &ln_gamma, &vap)
        }

        // --- Chao-Seader liquid fugacity coefficient νᵢ ---
        LiquidModel::ChaoSeader => {
            // νᵢ = fᵢᴸ/(xᵢP); with the vapor φ̂ᵢⱽ, Kᵢ = νᵢ/φ̂ᵢⱽ. Species set
            // defaults to Normal (H₂/methane special-casing is a caller
            // concern handled through the pure binding).
            let ks: Vec<f64> = (0..n)
                .map(|i| {
                    let ln_nu = crate::eos::chao_seader_ln_phi(
                        t,
                        p,
                        &spec.components[i],
                        crate::eos::ChaoSeaderSpecies::Normal,
                    );
                    (ln_nu - vap[i]).exp()
                })
                .collect();
            Ok(ks)
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
pub fn min_gibbs_ln_phi(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    w: &[f64],
) -> Result<Vec<f64>, FlashError> {
    let eos = match spec.liquid {
        LiquidModel::Cubic(eos) => eos,
        _ => {
            return Err(FlashError::Unsupported(
                "min-Gibbs ln φ is defined only for a cubic (φ-φ) system".into(),
            ));
        }
    };
    let ms = spec.mixture_spec(eos);
    // Try both roots; keep whichever gives the lower reduced Gibbs energy.
    let mut best: Option<(f64, Vec<f64>)> = None;
    for phase in [PhaseId::Liquid, PhaseId::Vapor] {
        if let Ok(lnphi) = ln_phi_mix(&ms, t, p, w, phase) {
            let g: f64 = (0..w.len())
                .filter(|&i| w[i] > 0.0)
                .map(|i| w[i] * (w[i].ln() + lnphi[i]))
                .sum();
            if best.as_ref().is_none_or(|(bg, _)| g < *bg) {
                best = Some((g, lnphi));
            }
        }
    }
    best.map(|(_, lnphi)| lnphi)
        .ok_or_else(|| FlashError::Thermo("no physical root at composition".into()))
}

/// Modified-Raoult K-values from a ln γ vector and the vapor ln φ̂.
fn gamma_phi_k(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    ln_gamma: &[f64],
    vap: &[f64],
) -> Result<Vec<f64>, FlashError> {
    let n = spec.n();
    let have_vl = spec.vl.len() == n;
    let mut k = Vec::with_capacity(n);
    for i in 0..n {
        let psat_i = psat(spec.sat_model(i), &spec.components[i], t)
            .map_err(|e| FlashError::Thermo(e.to_string()))?;
        let phi_sat = pure_sat_phi(spec, i, t, psat_i);
        let poy = if have_vl {
            poynting_factor(&spec.components[i], p, psat_i, t)
        } else {
            1.0
        };
        // Kᵢ = γᵢ·Psat·φˢᵃᵗ·POY / (φ̂ᵢⱽ·P).
        let numer = ln_gamma[i].exp() * psat_i * phi_sat * poy;
        k.push(numer / (vap[i].exp() * p));
    }
    Ok(k)
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
/// from the per-phase `mixture::d_ln_phi_d_n` (an O(n) dual sweep each) as
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
) -> Result<(Vec<f64>, Vec<f64>), FlashError> {
    let n = spec.n();
    match spec.vapor {
        VaporModel::IdealGas => Ok((vec![0.0; n], vec![0.0; n])),
        VaporModel::Cubic(eos) => {
            let ms = spec.mixture_spec(eos);
            let dt = crate::mixture::d_ln_phi_d_t(&ms, t, p, y, PhaseId::Vapor)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
            let dp = crate::mixture::d_ln_phi_d_p(&ms, t, p, y, PhaseId::Vapor)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
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

/// [`k_values`] plus exact ∂ln Kᵢ/∂T and ∂ln Kᵢ/∂P (§L, M12.3).
///
/// `t` in **K**, `p` in **kPa absolute**. Supports the φ-φ (cubic liquid) and
/// γ-φ (activity / ideal-solution liquid) paths with an ideal-gas or cubic
/// vapor. Virial vapor and Chao-Seader liquid derivatives are not yet
/// implemented (they return [`FlashError::Unsupported`]).
///
/// The γ-φ derivative is assembled **term-for-term** from the same pieces
/// [`gamma_phi_k`] multiplies, so the two never drift:
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
    let n = spec.n();
    let k = k_values(spec, t, p, x, y)?;
    let (vap_dt, vap_dp) = vapor_lnphi_derivs(spec, t, p, y)?;

    match spec.liquid {
        // --- φ-φ: ∂lnKᵢ = ∂lnφ̂ᵢᴸ − ∂lnφ̂ᵢⱽ ---
        LiquidModel::Cubic(eos) => {
            let ms = spec.mixture_spec(eos);
            let liq_dt = crate::mixture::d_ln_phi_d_t(&ms, t, p, x, PhaseId::Liquid)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
            let liq_dp = crate::mixture::d_ln_phi_d_p(&ms, t, p, x, PhaseId::Liquid)
                .map_err(|e| FlashError::Thermo(e.to_string()))?;
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
                LiquidModel::Activity(model) => dln_gamma_dt(spec, model, t, x),
                _ => vec![0.0; n],
            };
            let have_vl = spec.vl.len() == n;
            const R: f64 = 8.31451; // J/(mol·K); matches poynting_factor
            let mut d_ln_k_d_t = vec![0.0; n];
            let mut d_ln_k_d_p = vec![0.0; n];
            for i in 0..n {
                let comp = &spec.components[i];
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

        LiquidModel::ChaoSeader => Err(FlashError::Unsupported(
            "k_values_with_derivs: Chao-Seader liquid derivatives not implemented".into(),
        )),
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
            let n = spec.n();
            // Ideal-gas mixture baseline (each component as an ideal gas).
            let h_ideal = ideal_enthalpy_mix(spec.components, comp, t, t_ref, h_ref);
            let s_ideal = ideal_entropy_mix(spec.components, comp, t, p, t_ref, p_ref, s_ref);
            // Condensation (Clausius–Clapeyron, Ref (4) TERMOIII.PAS:283/294):
            // ΔH_vap,ᵢ = R·T²·dln(Psatᵢ)/dT; the liquid sits ΔH_vap below the gas.
            let mut h_cond = 0.0;
            let mut s_cond = 0.0;
            for i in 0..n {
                let c = &spec.components[i];
                let psat_i =
                    psat(spec.sat_model(i), c, t).map_err(|e| FlashError::Thermo(e.to_string()))?;
                let dpsat_dt = crate::saturation::d_psat_dt(spec.sat_model(i), c, t)
                    .map_err(|e| FlashError::Thermo(e.to_string()))?;
                let dh_vap = R * t * t * dpsat_dt / psat_i;
                h_cond += comp[i] * dh_vap;
                s_cond += comp[i] * (dh_vap / t); // ΔS_vap = ΔH_vap/T
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
        for i in 0..k_ref.len() {
            assert_eq!(kv.k[i], k_ref[i], "{label}: K[{i}] not bit-identical");
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
}
