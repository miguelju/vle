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

use crate::activity::ln_gamma_all;
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
            ln_gamma_all(model, x, spec.aij, spec.vl, spec.delta, t, &mut ln_gamma);
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
}
