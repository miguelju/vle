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

use smallvec::SmallVec;

use super::FlashError;
use super::init::wilson_k_values;
use super::system::{SystemSpec, SystemTpCache, min_gibbs_ln_phi_cached_into};

/// Stack-resident per-component buffer, matching the mixture core's inline
/// width so the engine has one such policy rather than several.
type WorkVec = SmallVec<[f64; 8]>;

/// Scratch buffers reused across every iteration of one trial phase.
///
/// The trial loop previously allocated the normalized composition, the
/// fugacity vector, and the updated mole numbers on each pass — three
/// allocations per iteration per seed (Part 1 §7 of the performance audit).
struct TrialWorkspace {
    /// Unnormalized trial mole numbers Wᵢ.
    w: WorkVec,
    /// Normalized trial composition wᵢ = Wᵢ/ΣW.
    wn: WorkVec,
    /// ln φ̂ᵢ at the trial composition, on the min-Gibbs root.
    ln_phi: WorkVec,
}

impl TrialWorkspace {
    fn new(n: usize) -> Self {
        Self {
            w: smallvec::smallvec![0.0; n],
            wn: smallvec::smallvec![0.0; n],
            ln_phi: smallvec::smallvec![0.0; n],
        }
    }
}

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
/// (unnormalized mole numbers), reusing `ws`. Returns `(tm, converged)`; the
/// converged normalized composition is left in `ws.wn`.
fn run_trial(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    d: &[f64],
    w0: &[f64],
    max_iter: usize,
    ws: &mut TrialWorkspace,
) -> Result<(f64, bool), FlashError> {
    let n = spec.n();
    ws.w.copy_from_slice(w0);
    let mut converged = false;
    for _ in 0..max_iter {
        normalize_into(&ws.w, &mut ws.wn);
        min_gibbs_ln_phi_cached_into(spec, cache, &ws.wn, &mut ws.ln_phi)?;
        // Fixed-point update Wᵢ = exp(dᵢ − ln φᵢ(w)), in place.
        let mut max_step = 0.0_f64;
        for ((wi, &di), &lnphi) in ws.w.iter_mut().zip(d.iter()).zip(ws.ln_phi.iter()) {
            let w_new = (di - lnphi).exp();
            max_step = max_step.max((w_new - *wi).abs());
            *wi = w_new;
        }
        if max_step < TPD_TOL {
            converged = true;
            break;
        }
    }
    normalize_into(&ws.w, &mut ws.wn);
    min_gibbs_ln_phi_cached_into(spec, cache, &ws.wn, &mut ws.ln_phi)?;
    // tm = 1 + Σ Wᵢ(ln Wᵢ + ln φᵢ(w) − dᵢ − 1).
    let tm: f64 = 1.0
        + (0..n)
            .filter(|&i| ws.w[i] > 0.0)
            .map(|i| ws.w[i] * (ws.w[i].ln() + ws.ln_phi[i] - d[i] - 1.0))
            .sum::<f64>();
    Ok((tm, converged))
}

/// Normalize mole numbers to mole fractions, into a caller-owned buffer.
#[inline]
fn normalize_into(w: &[f64], out: &mut [f64]) {
    let inv = w.iter().sum::<f64>().recip();
    for (dst, &wi) in out.iter_mut().zip(w) {
        *dst = wi * inv;
    }
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
    // A component genuinely absent from the feed has dᵢ = ln 0 = −∞, which
    // poisons every downstream sum. Reject it explicitly rather than returning
    // a NaN-derived verdict.
    if let Some(i) = z.iter().position(|&zi| zi <= 0.0 || !zi.is_finite()) {
        return Err(FlashError::InvalidInput(format!(
            "stability analysis needs every zᵢ > 0; z[{i}]={}",
            z[i]
        )));
    }

    // One (T, P) cache for both trials and every one of their iterations
    // (audit Part 2 §1) — the trial loop is the engine's most
    // fugacity-intensive inner loop.
    let cache = SystemTpCache::new(spec, t, p)?;
    let mut ws = TrialWorkspace::new(n);
    // Feed reference dᵢ = ln zᵢ + ln φᵢ(z).
    let mut d: WorkVec = smallvec::smallvec![0.0; n];
    min_gibbs_ln_phi_cached_into(spec, &cache, z, &mut d)?;
    for i in 0..n {
        d[i] += z[i].ln();
    }

    let kw = wilson_k_values(spec.components, t, p);
    // Vapor-like trial: Wᵢ = zᵢ·Kw ; liquid-like: Wᵢ = zᵢ/Kw.
    let mut seed: WorkVec = smallvec::smallvec![0.0; n];
    let mut best: Option<(f64, Vec<f64>)> = None;
    for vapor_like in [true, false] {
        for i in 0..n {
            seed[i] = if vapor_like {
                z[i] * kw[i]
            } else {
                z[i] / kw[i]
            };
        }
        let (tm, converged) = run_trial(spec, &cache, &d, &seed, max_iter, &mut ws)?;
        let wn = &ws.wn;
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
