//! Tangent-plane-distance (TPD) phase-stability analysis — Milestone 9, §I.
//!
//! Before (or instead of) a flash, the question "is this single-phase feed
//! actually stable, or will it split?" is answered by Michelsen's
//! tangent-plane-distance test (Ref (7)). It is the *structural* fix for the
//! thesis's trivial-solution weakness: rather than guarding a successive-
//! substitution flash against collapsing to the feed composition, the
//! stability test decides up front whether a second phase exists and, if so,
//! hands the flash a **non-trivial** K-value estimate to start from.
//!
//! ## The test
//!
//! At the feed `z` with single-phase fugacity coefficients φᵢ(z), define the
//! tangent-plane reference `dᵢ = ln zᵢ + ln φᵢ(z)`. A trial phase of
//! (unnormalized) mole numbers `Wᵢ` has tangent-plane distance
//!
//! ```text
//!   tm(W) = 1 + Σ Wᵢ·(ln Wᵢ + ln φᵢ(w) − dᵢ − 1),   w = W/ΣW
//! ```
//!
//! minimized by the fixed point `Wᵢ = exp(dᵢ − ln φᵢ(w))`, reached by
//! successive substitution. The feed is **unstable** (splits) if any trial
//! converges to `tm < 0` at a non-trivial composition. Two trials are run —
//! a vapor-like seed (`Wᵢ = zᵢ·Kw`) and a liquid-like seed (`Wᵢ = zᵢ/Kw`),
//! with `Kw` the Wilson estimate — which between them find essentially every
//! split.
//!
//! Only the cubic (φ-φ) path is handled; activity-model liquids don't show
//! the trivial-solution instability this targets, and the isothermal flash's
//! Rachford-Rice window already resolves single-phase γ-φ feeds.
//!
//! # References
//! - (7) Michelsen (1982) Part I — stability analysis
//! - (29) Wilson — the K-value seed

use super::FlashError;
use super::init::wilson_k_values;
use super::system::{SystemSpec, min_gibbs_ln_phi};

/// Outcome of a stability analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum Stability {
    /// The feed is a stable single phase at these conditions.
    Stable,
    /// The feed will split. `trial_k` is a non-trivial K-value estimate
    /// (`yᵢ/xᵢ`-like) derived from the winning trial phase — a warm start
    /// for the flash. `tpd` is the (negative) tangent-plane distance found.
    Unstable { trial_k: Vec<f64>, tpd: f64 },
}

/// Convergence + triviality thresholds for the trial-phase SS iteration.
const TPD_TOL: f64 = 1e-9;
const TRIVIAL_TOL: f64 = 1e-4;
const NEG_TPD_TOL: f64 = 1e-8;

/// Run one trial-phase successive substitution from an initial `w0`
/// (unnormalized mole numbers). Returns `(tm, w_normalized, converged)`.
fn run_trial(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    d: &[f64],
    w0: &[f64],
    max_iter: usize,
) -> Result<(f64, Vec<f64>, bool), FlashError> {
    let n = spec.n();
    let mut w = w0.to_vec();
    let mut converged = false;
    for _ in 0..max_iter {
        let sum_w: f64 = w.iter().sum();
        let wn: Vec<f64> = w.iter().map(|wi| wi / sum_w).collect();
        let ln_phi = min_gibbs_ln_phi(spec, t, p, &wn)?;
        // Fixed-point update Wᵢ = exp(dᵢ − ln φᵢ(w)).
        let mut w_new = vec![0.0; n];
        let mut max_step = 0.0_f64;
        for i in 0..n {
            w_new[i] = (d[i] - ln_phi[i]).exp();
            max_step = max_step.max((w_new[i] - w[i]).abs());
        }
        w = w_new;
        if max_step < TPD_TOL {
            converged = true;
            break;
        }
    }
    let sum_w: f64 = w.iter().sum();
    let wn: Vec<f64> = w.iter().map(|wi| wi / sum_w).collect();
    let ln_phi = min_gibbs_ln_phi(spec, t, p, &wn)?;
    // tm = 1 + Σ Wᵢ(ln Wᵢ + ln φᵢ(w) − dᵢ − 1).
    let tm: f64 = 1.0
        + (0..n)
            .filter(|&i| w[i] > 0.0)
            .map(|i| w[i] * (w[i].ln() + ln_phi[i] - d[i] - 1.0))
            .sum::<f64>();
    Ok((tm, wn, converged))
}

/// Tangent-plane-distance stability analysis of the feed `z` at `(t, p)`.
///
/// # Arguments
/// * `spec` — the mixture model (cubic/φ-φ only).
/// * `t` — Temperature in **K**; `p` — Pressure in **kPa absolute**.
/// * `z` — feed mole fractions (length N, sum to 1).
/// * `max_iter` — SS iteration cap per trial.
///
/// # Returns
/// [`Stability::Stable`] or [`Stability::Unstable`] with a non-trivial
/// K-value warm start.
///
/// # Errors
/// [`FlashError::Unsupported`] for a non-cubic liquid model;
/// [`FlashError::Thermo`] on a fugacity failure.
pub fn stability_analysis(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    max_iter: usize,
) -> Result<Stability, FlashError> {
    let n = spec.n();
    if z.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, z={}",
            z.len()
        )));
    }
    // Feed reference dᵢ = ln zᵢ + ln φᵢ(z).
    let ln_phi_z = min_gibbs_ln_phi(spec, t, p, z)?;
    let d: Vec<f64> = (0..n).map(|i| z[i].ln() + ln_phi_z[i]).collect();

    let kw = wilson_k_values(spec.components, t, p);
    // Vapor-like trial: Wᵢ = zᵢ·Kw ; liquid-like: Wᵢ = zᵢ/Kw.
    let vapor_seed: Vec<f64> = (0..n).map(|i| z[i] * kw[i]).collect();
    let liquid_seed: Vec<f64> = (0..n).map(|i| z[i] / kw[i]).collect();

    let mut best: Option<(f64, Vec<f64>)> = None;
    for seed in [vapor_seed, liquid_seed] {
        let (tm, wn, converged) = run_trial(spec, t, p, &d, &seed, max_iter)?;
        // Trivial solution: the trial collapsed back to the feed.
        let trivial = (0..n).all(|i| (wn[i] - z[i]).abs() < TRIVIAL_TOL);
        if converged && !trivial && tm < -NEG_TPD_TOL {
            // Non-trivial K estimate from this trial phase: Kᵢ = wᵢ/zᵢ.
            let trial_k: Vec<f64> = (0..n).map(|i| wn[i] / z[i]).collect();
            if best.as_ref().is_none_or(|(btm, _)| tm < *btm) {
                best = Some((tm, trial_k));
            }
        }
    }

    Ok(match best {
        Some((tpd, trial_k)) => Stability::Unstable { trial_k, tpd },
        None => Stability::Stable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos::{CubicEos, LiquidModel, VaporModel};
    use crate::mixing::MixingRule;
    use crate::types::Component;

    fn n_butane() -> Component {
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            ..Component::default()
        }
    }

    fn n_decane() -> Component {
        Component {
            name: "n-decane".into(),
            tc: 617.7,
            pc: 2110.0,
            omega: 0.4884,
            ..Component::default()
        }
    }

    fn rks(components: &[Component]) -> SystemSpec<'_> {
        SystemSpec {
            components,
            vapor: VaporModel::Cubic(CubicEos::RKS1972),
            liquid: LiquidModel::Cubic(CubicEos::RKS1972),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &[],
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        }
    }

    #[test]
    fn unstable_feed_detected_and_gives_warm_start() {
        // Light/heavy mix in the two-phase region → unstable, with a K
        // estimate spreading the components between phases (450 K / 800 kPa
        // → β ≈ 0.45; 1500 kPa would compress it to a single liquid).
        let comps = [n_butane(), n_decane()];
        let spec = rks(&comps);
        let res = stability_analysis(&spec, 450.0, 800.0, &[0.5, 0.5], 100).unwrap();
        match res {
            Stability::Unstable { trial_k, tpd } => {
                assert!(tpd < 0.0, "TPD should be negative, got {tpd}");
                // Non-trivial: the light component prefers one phase.
                assert!(trial_k[0] != trial_k[1]);
                assert!(trial_k.iter().all(|k| k.is_finite() && *k > 0.0));
            }
            Stability::Stable => panic!("expected unstable two-phase feed"),
        }
    }

    #[test]
    fn stable_single_phase_high_pressure() {
        // At very high pressure the mixture is a stable single (liquid) phase.
        let comps = [n_butane(), n_decane()];
        let spec = rks(&comps);
        let res = stability_analysis(&spec, 350.0, 30000.0, &[0.5, 0.5], 100).unwrap();
        assert_eq!(res, Stability::Stable);
    }

    #[test]
    fn stability_agrees_with_flash_phase_count() {
        // Where stability says "unstable", the isothermal flash must find a
        // genuine two-phase split, and vice-versa.
        use crate::flash::isothermal::flash_isothermal;
        let comps = [n_butane(), n_decane()];
        let spec = rks(&comps);
        // One genuinely two-phase condition, one compressed single-phase.
        for (t, p) in [(450.0, 800.0), (350.0, 30000.0)] {
            let stab = stability_analysis(&spec, t, p, &[0.5, 0.5], 100).unwrap();
            let flash = flash_isothermal(&spec, t, p, &[0.5, 0.5], 1e-9, 200).unwrap();
            match stab {
                Stability::Unstable { .. } => assert!(
                    flash.two_phase,
                    "stability=unstable but flash single-phase at ({t},{p})"
                ),
                Stability::Stable => assert!(
                    !flash.two_phase,
                    "stability=stable but flash two-phase at ({t},{p})"
                ),
            }
        }
    }
}
