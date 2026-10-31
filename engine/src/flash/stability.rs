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
use super::system::{
    SystemSpec, SystemTpCache, min_gibbs_ln_phi_cached_into, pure_ln_phi_min_gibbs_cached,
};
use crate::eos::{EosState, LiquidModel, PhaseId};

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
    /// for the flash: `wᵢ/zᵢ` from the normalized trial composition. Which
    /// side of a flash the incipient phase belongs on is not known from the
    /// test alone; the flash's own retry tries this K and its inverse.
    /// `tpd` is the (negative) tangent-plane distance found.
    Unstable { trial_k: Vec<f64>, tpd: f64 },
}

/// Convergence + triviality thresholds for the trial-phase SS iteration.
const TPD_TOL: f64 = 1e-9;
const TRIVIAL_TOL: f64 = 1e-4;
const NEG_TPD_TOL: f64 = 1e-8;

/// Run one trial-phase successive substitution from an initial `w0`
/// (unnormalized mole numbers), reusing `ws`. Returns `(tm, converged,
/// last_step)`; the final normalized composition is left in `ws.wn`.
fn run_trial(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    d: &[f64],
    w0: &[f64],
    max_iter: usize,
    ws: &mut TrialWorkspace,
) -> Result<(f64, bool, f64), FlashError> {
    let n = spec.n();
    ws.w.copy_from_slice(w0);
    let mut converged = false;
    let mut last_step = f64::INFINITY;
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
        last_step = max_step;
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
    Ok((tm, converged, last_step))
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

    // One (T, P) cache for every trial and every one of their iterations
    // (audit Part 2 §1) — the trial loop is the engine's most
    // fugacity-intensive inner loop.
    let cache = SystemTpCache::new(spec, t, p)?;
    let v = stability_cached(spec, &cache, t, p, z, max_iter, Seeds::WilsonThenPure)?;
    if !v.conclusive {
        // A trial that ran out of iterations proves nothing either way. Until
        // 2026-10-31 this returned `Stable`, which near the critical locus
        // was wrong at least twice on the §10 grid (OPTIMIZATION_PLAN_PART1
        // §12–§13): give the caller the truth instead.
        return Err(FlashError::NoConvergence {
            what: "stability analysis",
            iters: max_iter,
            residual: v.residual,
        });
    }
    Ok(v.verdict)
}

/// Which trial phases a stability test starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seeds {
    /// The two Wilson trials only (vapor-like `zᵢ·Kᵢ`, liquid-like `zᵢ/Kᵢ`).
    /// Michelsen's classic pair; enough for every hydrocarbon point measured
    /// (OPTIMIZATION_PLAN_PART1 §10, §12).
    #[allow(dead_code)]
    Wilson,
    /// The Wilson pair, then — only if neither found a split — one near-pure
    /// trial per component. Liquid–liquid splits with an almost-pure phase
    /// (water in a hydrocarbon) are invisible to both Wilson seeds; on PR
    /// water/propane/n-heptane the pair missed 250 splits the flash itself
    /// found, the pure seeds none (§13). A stable verdict therefore costs
    /// n + 2 trials instead of 2.
    WilsonThenPure,
}

/// A stability verdict plus what is needed to know whether to trust it.
#[derive(Debug, Clone)]
pub(crate) struct Verdict {
    pub verdict: Stability,
    /// `true` when the verdict was earned: either a trial *proved* instability
    /// (any trial composition with tm < 0 does, converged or not), or every
    /// trial reached its fixed point. A `Stable` with `conclusive = false`
    /// means "no trial found a split before running out of iterations".
    pub conclusive: bool,
    /// Largest final step of any unconverged trial (0 if all converged).
    pub residual: f64,
}

/// [`stability_analysis`] against a caller's (T, P) cache, with a choice of
/// trial seeds and a verdict that says whether it can be trusted.
///
/// `t` (**K**) and `p` (**kPa absolute**) must be the state `cache` was built
/// at; they seed the Wilson trial phases. The caller must already have
/// validated `z` (length N, every zᵢ > 0).
pub(crate) fn stability_cached(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    t: f64,
    p: f64,
    z: &[f64],
    max_iter: usize,
    seeds: Seeds,
) -> Result<Verdict, FlashError> {
    let n = spec.n();
    let mut ws = TrialWorkspace::new(n);
    // Feed reference dᵢ = ln zᵢ + ln φᵢ(z).
    let mut d: WorkVec = smallvec::smallvec![0.0; n];
    min_gibbs_ln_phi_cached_into(spec, cache, z, &mut d)?;
    for i in 0..n {
        d[i] += z[i].ln();
    }

    let kw = wilson_k_values(spec.components, t, p);
    let n_seeds = match seeds {
        Seeds::Wilson => 2,
        Seeds::WilsonThenPure => 2 + n,
    };
    let mut seed: WorkVec = smallvec::smallvec![0.0; n];
    let mut best: Option<(f64, Vec<f64>)> = None;
    let mut all_converged = true;
    let mut residual = 0.0_f64;
    for s_idx in 0..n_seeds {
        // The pure seeds are a second line: only consulted when the Wilson
        // pair found nothing, and abandoned at the first split they find.
        if s_idx >= 2 && best.is_some() {
            break;
        }
        for i in 0..n {
            seed[i] = match s_idx {
                // Vapor-like trial: Wᵢ = zᵢ·Kw ; liquid-like: Wᵢ = zᵢ/Kw.
                0 => z[i] * kw[i],
                1 => z[i] / kw[i],
                // Near-pure component k = s_idx − 2, the rest a trace.
                k if i == k - 2 => 1.0,
                _ => PURE_SEED_TRACE / n as f64,
            };
        }
        let (tm, converged, step) = run_trial(spec, cache, &d, &seed, max_iter, &mut ws)?;
        if !converged {
            all_converged = false;
            residual = residual.max(step);
        }
        let wn = &ws.wn;
        // Trivial solution: the trial collapsed back to the feed.
        let trivial = (0..n).all(|i| (wn[i] - z[i]).abs() < TRIVIAL_TOL);
        // tm(W) < 0 at *any* W proves the feed unstable, so an unconverged
        // trial that already reached negative tm counts too.
        if !trivial && tm < -NEG_TPD_TOL {
            // Non-trivial K estimate from this trial phase: Kᵢ = wᵢ/zᵢ.
            // (The textbook unnormalized Wᵢ/zᵢ, inverted for the liquid-like
            // trial, was tried on 2026-10-31 and changed no flash answer on
            // the §13 sweep once the retry tries both orientations — so the
            // public value is left as it was. OPTIMIZATION_PLAN_PART1 §13.)
            let trial_k: Vec<f64> = (0..n).map(|i| wn[i] / z[i]).collect();
            if best.as_ref().is_none_or(|(btm, _)| tm < *btm) {
                best = Some((tm, trial_k));
            }
        }
    }

    Ok(match best {
        Some((tpd, trial_k)) => Verdict {
            verdict: Stability::Unstable { trial_k, tpd },
            conclusive: true,
            residual,
        },
        None => Verdict {
            verdict: Stability::Stable,
            conclusive: all_converged,
            residual,
        },
    })
}

/// Mole fraction shared by the other components in a near-pure trial seed.
const PURE_SEED_TRACE: f64 = 1e-3;

/// The cheapest rigorous instability test: the tangent-plane distance of each
/// **pure** component as a trial phase,
///
/// ```text
///   tmᵢ = ln φᵢ(pure; T, P) − ln zᵢ − ln φᵢ(z)
/// ```
///
/// A stable feed has tm ≥ 0 at every trial composition, so a negative tmᵢ
/// *proves* the feed unstable — a near-pure phase of component i would form —
/// and the screen can never fire on a stable feed. It does not prove
/// stability when it stays quiet. Cost: one mixture ln φ at the feed plus one
/// pure-component cubic per component, O(n) on top of the feed evaluation.
///
/// Returns the index with the most negative tmᵢ, or `None`.
///
/// `ln_phi_feed` is ln φᵢ(z) on the min-Gibbs root, already evaluated by the
/// caller.
pub(crate) fn pure_component_screen(
    spec: &SystemSpec,
    cache: &SystemTpCache,
    t: f64,
    p: f64,
    z: &[f64],
    ln_phi_feed: &[f64],
) -> Option<usize> {
    let LiquidModel::Cubic(eos) = spec.liquid else {
        return None;
    };
    let mut worst: Option<(usize, f64)> = None;
    for (i, comp) in spec.components.iter().enumerate() {
        if z[i] <= 0.0 {
            continue;
        }
        // Pure component on its min-Gibbs root, read from the flash's (T, P)
        // cache when it has one (no α re-evaluation); a fresh pure EOS state
        // otherwise. For a pure fluid G^R/RT is ln φ itself, so the lower of
        // the two roots' ln φ is the stable one.
        let ln_phi_pure = pure_ln_phi_min_gibbs_cached(cache, i).unwrap_or_else(|| {
            let st = EosState::new(eos, t, p, comp);
            [PhaseId::Liquid, PhaseId::Vapor]
                .into_iter()
                .filter_map(|ph| st.ln_phi(ph).ok())
                .fold(f64::INFINITY, f64::min)
        });
        if !ln_phi_pure.is_finite() {
            continue;
        }
        let tm = ln_phi_pure - z[i].ln() - ln_phi_feed[i];
        if tm < -NEG_TPD_TOL && worst.is_none_or(|(_, w)| tm < w) {
            worst = Some((i, tm));
        }
    }
    worst.map(|(i, _)| i)
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

    // -----------------------------------------------------------------
    // Multi-seed trials, the pure-component screen, and honest verdicts
    // (OPTIMIZATION_PLAN_PART1 §13).
    // -----------------------------------------------------------------

    fn comp(name: &str, tc: f64, pc: f64, omega: f64) -> Component {
        Component {
            name: name.into(),
            tc,
            pc,
            omega,
            ..Component::default()
        }
    }

    /// Water/propane/n-heptane on PR with kij(water–HC) = 0.5, the bundled
    /// database's constants.
    fn water_propane_heptane() -> Vec<Component> {
        vec![
            comp("water", 647.096, 22064.0, 0.3443),
            comp("propane", 369.89, 4251.2, 0.1521),
            comp("n-heptane", 540.2, 2735.73, 0.349),
        ]
    }

    fn water_kij(n: usize, k: f64) -> Vec<Vec<f64>> {
        (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| if (i == 0) != (j == 0) { k } else { 0.0 })
                    .collect()
            })
            .collect()
    }

    fn pr<'a>(components: &'a [Component], kij: &'a [Vec<f64>]) -> SystemSpec<'a> {
        SystemSpec {
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
            kij,
            ..rks(components)
        }
    }

    #[test]
    fn pure_seeds_find_a_split_the_wilson_pair_misses() {
        // One of the 250 §12 points where the flash finds a split and the
        // two Wilson trials both converge to "stable".
        let comps = water_propane_heptane();
        let kij = water_kij(3, 0.5);
        let spec = pr(&comps, &kij);
        let (t, p, z) = (233.898305, 10.0, [0.3, 0.35, 0.35]);
        let cache = SystemTpCache::new(&spec, t, p).unwrap();
        let wilson = stability_cached(&spec, &cache, t, p, &z, 2000, Seeds::Wilson).unwrap();
        assert_eq!(wilson.verdict, Stability::Stable, "premise");
        assert!(wilson.conclusive, "premise: both Wilson trials converged");
        let all = stability_cached(&spec, &cache, t, p, &z, 2000, Seeds::WilsonThenPure).unwrap();
        assert!(matches!(all.verdict, Stability::Unstable { .. }));
        // The public entry point uses the full seed set.
        assert!(matches!(
            stability_analysis(&spec, t, p, &z, 2000).unwrap(),
            Stability::Unstable { .. }
        ));
    }

    #[test]
    fn unconverged_trials_are_judged_on_the_evidence() {
        // Methane/n-pentane 409.32 K / 11 043 kPa: a genuine near-critical
        // split whose trials need > 200 iterations. Until §13 the 200-
        // iteration answer was `Stable`. A trial that has already reached
        // tm < 0 proves instability whether or not it converged, so the
        // short run now says `Unstable` too.
        let comps = [
            comp("methane", 190.564, 4599.2, 0.01142),
            comp("n-pentane", 469.7, 3367.5, 0.251),
        ];
        let kij: Vec<Vec<f64>> = vec![];
        let spec = pr(&comps, &kij);
        let (t, p, z) = (409.3220, 11043.0009, [0.5, 0.5]);
        for iters in [200, 2000] {
            assert!(matches!(
                stability_analysis(&spec, t, p, &z, iters).unwrap(),
                Stability::Unstable { .. }
            ));
        }
        // A stable feed whose trials are cut off before converging has
        // proved nothing either way: that is an error, not `Stable`.
        let comps = [n_butane(), n_decane()];
        let spec = rks(&comps);
        assert_eq!(
            stability_analysis(&spec, 350.0, 30000.0, &[0.5, 0.5], 100).unwrap(),
            Stability::Stable,
            "premise"
        );
        assert!(matches!(
            stability_analysis(&spec, 350.0, 30000.0, &[0.5, 0.5], 1),
            Err(FlashError::NoConvergence {
                what: "stability analysis",
                ..
            })
        ));
    }

    #[test]
    fn cached_pure_ln_phi_is_the_mixture_pure_limit() {
        // The screen's pure ln φ comes from the cache's (A, B, U, W); it must
        // equal the mixture model evaluated at xᵢ = 1, for 2- and
        // 3-parameter EOS, across liquid, vapor and supercritical states.
        let comps = water_propane_heptane();
        let kij = water_kij(3, 0.5);
        for eos in [CubicEos::PR1976, CubicEos::RKS1972, CubicEos::PatelTeja] {
            let mut spec = pr(&comps, &kij);
            spec.vapor = VaporModel::Cubic(eos);
            spec.liquid = LiquidModel::Cubic(eos);
            for (t, p) in [(300.0, 100.0), (450.0, 5000.0), (600.0, 30_000.0)] {
                let cache = SystemTpCache::new(&spec, t, p).unwrap();
                for i in 0..3 {
                    let mut e = [0.0; 3];
                    e[i] = 1.0;
                    let mut lp = [0.0; 3];
                    min_gibbs_ln_phi_cached_into(&spec, &cache, &e, &mut lp).unwrap();
                    let cached = pure_ln_phi_min_gibbs_cached(&cache, i).unwrap();
                    assert!(
                        (cached - lp[i]).abs() < 1e-10,
                        "{eos:?} comp {i} at ({t}, {p}): {cached} vs {}",
                        lp[i]
                    );
                }
            }
        }
    }

    #[test]
    fn pure_screen_fires_on_an_immiscible_feed_and_never_on_a_stable_one() {
        // Fires: water/n-hexane at 300 K / 100 kPa is two liquids.
        let wh = [
            comp("water", 647.096, 22064.0, 0.3443),
            comp("n-hexane", 507.82, 3044.1, 0.3),
        ];
        let kij = water_kij(2, 0.5);
        let spec = pr(&wh, &kij);
        let z = [0.5, 0.5];
        let cache = SystemTpCache::new(&spec, 300.0, 100.0).unwrap();
        let mut d = [0.0; 2];
        min_gibbs_ln_phi_cached_into(&spec, &cache, &z, &mut d).unwrap();
        assert!(pure_component_screen(&spec, &cache, 300.0, 100.0, &z, &d).is_some());
        // Never fires where a converged stability test says stable: a
        // negative pure-component TPD would contradict it.
        let comps = [n_butane(), n_decane()];
        let spec = rks(&comps);
        let z = [0.5, 0.5];
        let mut stable_points = 0;
        for t in [250.0, 350.0, 450.0, 550.0, 650.0] {
            for p in [10.0, 100.0, 1000.0, 10_000.0, 30_000.0] {
                if stability_analysis(&spec, t, p, &z, 2000) != Ok(Stability::Stable) {
                    continue;
                }
                stable_points += 1;
                let cache = SystemTpCache::new(&spec, t, p).unwrap();
                let mut d = [0.0; 2];
                min_gibbs_ln_phi_cached_into(&spec, &cache, &z, &mut d).unwrap();
                assert_eq!(pure_component_screen(&spec, &cache, t, p, &z, &d), None);
            }
        }
        assert!(stable_points >= 10, "only {stable_points} stable points");
    }
}
