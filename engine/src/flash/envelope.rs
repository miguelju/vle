//! Phase-envelope continuation — Milestone 9, §K.
//!
//! Traces the full pressure–temperature saturation boundary of a mixture at
//! fixed overall composition `z` — the closed loop of bubble and dew points
//! that meets at the mixture critical point. This is Michelsen's
//! phase-envelope construction (Ref (24)): a predictor–corrector
//! continuation that walks *through* the critical point smoothly, where the
//! thesis's differential dP/dT stepping (and the separate bubble/dew solvers
//! in [`super::bubble`] / [`super::dew`]) break down.
//!
//! ## Formulation
//!
//! At every envelope point one phase is the bulk (composition `z`) and the
//! other is incipient (composition `w`, vanishing amount), with
//! `wᵢ = Kᵢ·zᵢ`. The unknowns are `X = (ln K₁, …, ln Kₙ, ln T, ln P)` and
//! the equations are
//!
//! ```text
//!   gᵢ   = ln Kᵢ + ln φ̂ᵢ(w̄) − ln φ̂ᵢ(z) = 0     (i = 1..n, equal fugacity)
//!   g_{n+1} = Σᵢ zᵢ·(Kᵢ − 1) = 0                (incipient phase sums to 1)
//!   g_{n+2} = X_S − S = 0                        (continuation specification)
//! ```
//!
//! where `w̄ = w/Σw`. The fugacity coefficients use the **minimum-Gibbs
//! root** at each composition (via `system::min_gibbs_ln_phi_root_into`), so
//! the bulk and incipient phases automatically pick liquid- vs vapor-like
//! roots away from the critical point and merge onto the single root at it —
//! the key to passing through the critical point without special-casing.
//!
//! ## Continuation
//!
//! From a converged point the tangent `dX/dS` is found from the Jacobian;
//! the next specification variable is the fastest-changing component of the
//! tangent (keeps the parameterization well-conditioned near the
//! cricondentherm/cricondenbar where P or T alone would fold back). A
//! first-order tangent predictor seeds the Newton corrector. The
//! `(n+2)×(n+2)` Jacobian (corrector and tangent) is **exact**: the
//! incipient phase's ∂ln φ̂/∂n plus one two-lane T/P dual pass per phase, on
//! the roots the residual's min-Gibbs selection chose (see [`jacobian`]).
//! Until 2026-11 it was a forward difference costing n+2 extra residuals per
//! iteration (`OPTIMIZATION_PLAN_PART1.md` §15).
//!
//! Restricted to the φ-φ (cubic both phases) path.
//!
//! # References
//! - (24) Michelsen (1980) — calculation of phase envelopes

use nalgebra::{DMatrix, DVector};

use super::FlashError;
use super::bubble::bubble_temperature;
use super::init::wilson_k_values;
use super::system::{SystemSpec, min_gibbs_ln_phi_root_into};
use crate::eos::PhaseId;
use crate::mixture::{d_ln_phi_d_n, d_ln_phi_d_tp};

/// One converged point on the phase envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvelopePoint {
    /// Temperature in **K**.
    pub t: f64,
    /// Pressure in **kPa**.
    pub p: f64,
    /// Equilibrium ratios Kᵢ at this point.
    pub k: Vec<f64>,
    /// The incipient-phase composition `w̄` (normalized).
    pub incipient: Vec<f64>,
}

/// Which cubic root the min-Gibbs selection chose for the incipient phase
/// (`w`) and the bulk phase (`z`) at one evaluation of [`residual`]. The
/// analytic [`jacobian`] differentiates on exactly these roots.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Roots {
    w: PhaseId,
    z: PhaseId,
}

/// The incipient composition `w̄ = (K∘z)/Σ(K∘z)` and `Σ(K∘z)` from the
/// unknowns `X` (its first n entries are ln K).
fn incipient(z: &[f64], x: &DVector<f64>) -> (Vec<f64>, f64) {
    let n = z.len();
    let w_un: Vec<f64> = (0..n).map(|i| x[i].exp() * z[i]).collect();
    let sw: f64 = w_un.iter().sum();
    (w_un.iter().map(|wi| wi / sw).collect(), sw)
}

/// Residual vector `g(X)` of the envelope system (length n+2), given the
/// unknowns `X = (ln K…, ln T, ln P)` and the continuation spec `(s_idx, s_val)`,
/// plus the roots the two min-Gibbs evaluations chose.
fn residual(
    spec: &SystemSpec,
    z: &[f64],
    x: &DVector<f64>,
    s_idx: usize,
    s_val: f64,
) -> Result<(DVector<f64>, Roots), FlashError> {
    let n = z.len();
    let t = x[n].exp();
    let p = x[n + 1].exp();
    let (w, _) = incipient(z, x);

    let mut ln_phi_w = vec![0.0; n];
    let mut ln_phi_z = vec![0.0; n];
    let roots = Roots {
        w: min_gibbs_ln_phi_root_into(spec, t, p, &w, &mut ln_phi_w)?,
        z: min_gibbs_ln_phi_root_into(spec, t, p, z, &mut ln_phi_z)?,
    };

    let mut g = DVector::zeros(n + 2);
    for i in 0..n {
        g[i] = x[i] + ln_phi_w[i] - ln_phi_z[i];
    }
    // Σ zᵢ(Kᵢ − 1) = 0.
    g[n] = (0..n).map(|i| z[i] * (x[i].exp() - 1.0)).sum();
    // Continuation: X_s − S = 0.
    g[n + 1] = x[s_idx] - s_val;
    Ok((g, roots))
}

/// Exact `(n+2)×(n+2)` Jacobian of [`residual`] with respect to
/// `X = (ln K…, ln T, ln P)`, on the roots the residual chose.
///
/// With `Gᵢⱼ = ∂ln φ̂ᵢ/∂nⱼ` of the incipient phase at `w̄` (ln φ̂ is
/// intensive, so the mole-number derivative at total `Σw = Σ Kz` is `G/Σw`,
/// and `∂wⱼ/∂ln Kⱼ = wⱼ`) — the same identity as the saturation solver's full
/// Newton step (`incipient::full_newton_step`):
/// ```text
///   ∂gᵢ/∂ln Kⱼ = δᵢⱼ + Gᵢⱼ·w̄ⱼ
///   ∂gᵢ/∂ln T  = T·(∂ln φ̂ᵢʷ/∂T − ∂ln φ̂ᵢᶻ/∂T)
///   ∂gᵢ/∂ln P  = P·(∂ln φ̂ᵢʷ/∂P − ∂ln φ̂ᵢᶻ/∂P)
///   ∂gₙ₊₁/∂ln Kⱼ = zⱼKⱼ,   ∂gₙ₊₂/∂Xₖ = δₖ,ₛ
/// ```
/// The bulk phase `z` is fixed, so it contributes only T and P columns. G
/// comes from [`d_ln_phi_d_n`] (analytic for classical mixing, fixed-width
/// duals for GE rules); the T and P columns from one two-lane dual pass per
/// phase ([`d_ln_phi_d_tp`]). This replaced a forward-difference Jacobian
/// that cost n+2 further residuals — `2(n+2)` min-Gibbs evaluations — per
/// Newton iteration (`OPTIMIZATION_PLAN_PART1.md` §15).
fn jacobian(
    spec: &SystemSpec,
    z: &[f64],
    x: &DVector<f64>,
    s_idx: usize,
    roots: Roots,
) -> Result<DMatrix<f64>, FlashError> {
    let n = z.len();
    let t = x[n].exp();
    let p = x[n + 1].exp();
    let (w, _) = incipient(z, x);
    let eos = match spec.liquid {
        crate::eos::LiquidModel::Cubic(eos) => eos,
        _ => {
            return Err(FlashError::Unsupported(
                "phase envelope requires a cubic (φ-φ) system".into(),
            ));
        }
    };
    let ms = spec.mixture_spec(eos);
    let thermo = |e: crate::mixture::MixError| FlashError::Thermo(e.to_string());
    let g_w = d_ln_phi_d_n(&ms, t, p, &w, roots.w).map_err(thermo)?;
    let (dt_w, dp_w) = d_ln_phi_d_tp(&ms, t, p, &w, roots.w).map_err(thermo)?;
    let (dt_z, dp_z) = d_ln_phi_d_tp(&ms, t, p, z, roots.z).map_err(thermo)?;

    let mut j = DMatrix::zeros(n + 2, n + 2);
    for i in 0..n {
        for c in 0..n {
            j[(i, c)] = g_w[i][c] * w[c];
        }
        j[(i, i)] += 1.0;
        j[(i, n)] = t * (dt_w[i] - dt_z[i]);
        j[(i, n + 1)] = p * (dp_w[i] - dp_z[i]);
    }
    for c in 0..n {
        j[(n, c)] = z[c] * x[c].exp();
    }
    j[(n + 1, s_idx)] = 1.0;
    Ok(j)
}

/// The forward-difference Jacobian the analytic [`jacobian`] replaced — kept
/// only as a test oracle (CLAUDE.md: numerical derivatives are test oracles).
#[cfg(test)]
fn jacobian_fd(
    spec: &SystemSpec,
    z: &[f64],
    x: &DVector<f64>,
    s_idx: usize,
    s_val: f64,
) -> Result<DMatrix<f64>, FlashError> {
    let m = x.len();
    let (g0, _) = residual(spec, z, x, s_idx, s_val)?;
    let mut j = DMatrix::zeros(m, m);
    for col in 0..m {
        // Central differences: the oracle should be well below the
        // tolerance the comparison test asserts.
        let h = 1e-6 * x[col].abs().max(1.0);
        let mut xp = x.clone();
        let mut xm = x.clone();
        xp[col] += h;
        xm[col] -= h;
        let (gp, _) = residual(spec, z, &xp, s_idx, s_val)?;
        let (gm, _) = residual(spec, z, &xm, s_idx, s_val)?;
        for row in 0..m {
            j[(row, col)] = (gp[row] - gm[row]) / (2.0 * h);
        }
    }
    let _ = g0;
    Ok(j)
}

/// Newton-correct the unknowns `X` to satisfy the envelope system at the
/// given continuation spec. Returns the converged `X` and the roots its
/// final residual chose (which the tangent's Jacobian reuses).
fn correct(
    spec: &SystemSpec,
    z: &[f64],
    mut x: DVector<f64>,
    s_idx: usize,
    s_val: f64,
    tol: f64,
    max_iter: usize,
) -> Result<(DVector<f64>, Roots), FlashError> {
    for iter in 0..max_iter {
        let (g, roots) = residual(spec, z, &x, s_idx, s_val)?;
        if g.amax() < tol {
            return Ok((x, roots));
        }
        let j = jacobian(spec, z, &x, s_idx, roots)?;
        let dx = j.lu().solve(&(-&g)).ok_or(FlashError::NoConvergence {
            what: "envelope corrector (singular Jacobian)",
            iters: iter,
            residual: g.amax(),
        })?;
        // Damp the Newton step to keep ln T / ln P moves bounded.
        let scale = {
            let max_step = dx.amax();
            if max_step > 0.3 { 0.3 / max_step } else { 1.0 }
        };
        x += scale * dx;
        if iter + 1 == max_iter {
            let (g, _) = residual(spec, z, &x, s_idx, s_val)?;
            return Err(FlashError::NoConvergence {
                what: "envelope corrector",
                iters: max_iter,
                residual: g.amax(),
            });
        }
    }
    unreachable!("loop returns via convergence or NoConvergence")
}

/// Trace the phase envelope of a mixture at overall composition `z`,
/// starting from a low-pressure bubble point and stepping toward and through
/// the critical point (§K).
///
/// # Arguments
/// * `spec` — the mixture model (φ-φ cubic both phases).
/// * `z` — overall composition (length N, sum to 1).
/// * `p_start` — a low starting pressure in **kPa** (well below the
///   cricondenbar, where a bubble point is easy to seed).
/// * `max_points` — maximum number of envelope points to trace.
///
/// # Returns
/// The traced [`EnvelopePoint`]s in order: up the bubble branch, through the
/// critical point, and down the dew branch. Tracing stops at `max_points`,
/// when the step size cannot be reduced further (corrector failure), or once
/// the dew branch has come back down to `p_start` — the closed envelope.
///
/// # Errors
/// [`FlashError::Unsupported`] for a non-cubic system;
/// [`FlashError::NoConvergence`] if the initial point cannot be established.
pub fn trace_envelope(
    spec: &SystemSpec,
    z: &[f64],
    p_start: f64,
    max_points: usize,
) -> Result<Vec<EnvelopePoint>, FlashError> {
    let n = z.len();
    if z.len() != spec.n() {
        return Err(FlashError::Dimension(format!(
            "components={}, z={}",
            spec.n(),
            n
        )));
    }
    if !matches!(spec.liquid, crate::eos::LiquidModel::Cubic(_)) {
        return Err(FlashError::Unsupported(
            "phase envelope requires a cubic (φ-φ) system".into(),
        ));
    }

    // Seed: the bubble point at p_start, from the engine's own
    // bubble-temperature solver (Newton on 1/T since v0.17.0, a few µs). Its
    // converged T and K put the corrector inside its basin of attraction.
    // The previous seed — Wilson K at T = 0.7·Σzᵢ·Tc,ᵢ — could sit tens of
    // kelvin from the bubble point (≈ 338 K vs ≈ 290 K for equimolar
    // n-butane/n-heptane at 100 kPa), and the fixed-P corrector then failed
    // to converge from p_start 50, 100 and 1000 kPa on ordinary pairs
    // (OPTIMIZATION_PLAN_PART1.md §9). It is kept as the fallback for a
    // bubble solve that fails.
    let mut x = DVector::zeros(n + 2);
    match bubble_temperature(spec, p_start, z, 1e-10, 200) {
        Ok(b) => {
            for i in 0..n {
                x[i] = b.k[i].ln();
            }
            x[n] = b.value.ln();
        }
        Err(_) => {
            let tavg: f64 = z.iter().zip(spec.components).map(|(zi, c)| zi * c.tc).sum();
            let t0 = 0.7 * tavg;
            let kw = wilson_k_values(spec.components, t0, p_start);
            for i in 0..n {
                x[i] = kw[i].ln();
            }
            x[n] = t0.ln();
        }
    }
    x[n + 1] = p_start.ln();

    // Correct the seed with P fixed (spec = ln P index n+1).
    let s_idx0 = n + 1;
    let (mut x, mut roots) = correct(spec, z, x, s_idx0, p_start.ln(), 1e-9, 100)?;

    let mut points = Vec::with_capacity(max_points);
    let record = |x: &DVector<f64>| -> EnvelopePoint {
        let k: Vec<f64> = (0..n).map(|i| x[i].exp()).collect();
        let t = x[n].exp();
        let p = x[n + 1].exp();
        let w_un: Vec<f64> = (0..n).map(|i| k[i] * z[i]).collect();
        let sw: f64 = w_un.iter().sum();
        EnvelopePoint {
            t,
            p,
            k: k.clone(),
            incipient: w_un.iter().map(|wi| wi / sw).collect(),
        }
    };
    points.push(record(&x));

    // Continuation loop.
    let mut s_idx = n; // start by stepping ln T (up the bubble branch)
    let mut ds = 0.05; // step in the specified variable
    // Direction of travel, carried between steps. The tangent solve below
    // fixes t[s_idx] = +1, i.e. it always points toward *increasing* the
    // spec variable; without re-orienting it against the previous direction
    // the trace reverses whenever the spec switches to a variable that must
    // now decrease (ln Kᵢ falling through 0 at the critical point is the
    // common case) and oscillates there forever. That was the behaviour
    // before v0.17.0: hundreds of points bouncing around the critical point,
    // the dew branch never reached.
    let mut prev_dir: Option<DVector<f64>> = None;
    let mut p_max = p_start;
    for _ in 1..max_points {
        // Tangent dX/dS: solve J·t = e_{last} (the spec row derivative).
        let j = jacobian(spec, z, &x, s_idx, roots)?;
        let mut rhs = DVector::zeros(n + 2);
        rhs[n + 1] = 1.0; // ∂g_{n+2}/∂S = −1 → tangent solves J·t = e
        let mut tangent = match j.lu().solve(&rhs) {
            Some(t) => t,
            None => break,
        };
        // Keep moving the way we were moving (secant orientation).
        if let Some(d) = &prev_dir {
            if tangent.dot(d) < 0.0 {
                tangent = -tangent;
            }
        }
        // Choose the next spec = the fastest-moving unknown (best-conditioned).
        let mut best = n; // default ln T
        let mut best_mag = 0.0;
        for (idx, item) in tangent.iter().enumerate().take(n + 2) {
            if item.abs() > best_mag {
                best_mag = item.abs();
                best = idx;
            }
        }
        s_idx = best;
        // Predictor: step the spec variable and move along the tangent.
        let step_dir = tangent[s_idx].signum();
        let s_val = x[s_idx] + ds * step_dir;
        let scale = ds * step_dir / tangent[s_idx];
        let x_pred = &x + scale * &tangent;

        match correct(spec, z, x_pred, s_idx, s_val, 1e-9, 60) {
            Ok((x_new, roots_new)) => {
                // Reject a step that barely moved or blew up.
                let moved = (&x_new - &x).amax();
                if !x_new.iter().all(|v| v.is_finite()) || moved < 1e-9 {
                    break;
                }
                prev_dir = Some(&x_new - &x);
                x = x_new;
                roots = roots_new;
                let pt = record(&x);
                let p_now = pt.p;
                points.push(pt);
                // Grow the step when convergence is easy.
                ds = (ds * 1.1).min(0.15);
                // Closed the loop: past the cricondenbar and back down the
                // dew branch to the starting pressure.
                p_max = p_max.max(p_now);
                if p_max > p_start * 1.5 && p_now <= p_start {
                    break;
                }
            }
            Err(_) => {
                // Halve the step and retry from the same (last good) point.
                ds *= 0.5;
                if ds < 1e-3 {
                    break;
                }
            }
        }
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos::{CubicEos, LiquidModel, VaporModel};
    use crate::flash::system::min_gibbs_ln_phi;
    use crate::mixing::MixingRule;
    use crate::types::Component;

    fn methane() -> Component {
        Component {
            name: "methane".into(),
            tc: 190.564,
            pc: 4599.0,
            omega: 0.0115,
            ..Component::default()
        }
    }

    fn ethane() -> Component {
        Component {
            name: "ethane".into(),
            tc: 305.32,
            pc: 4872.0,
            omega: 0.0995,
            ..Component::default()
        }
    }

    fn pr(components: &[Component]) -> SystemSpec<'_> {
        SystemSpec {
            components,
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
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

    /// Unknowns `X = (ln K…, ln T, ln P)` of a traced envelope point.
    fn unknowns(pt: &EnvelopePoint) -> DVector<f64> {
        let n = pt.k.len();
        DVector::from_fn(n + 2, |r, _| match r {
            r if r < n => pt.k[r].ln(),
            r if r == n => pt.t.ln(),
            _ => pt.p.ln(),
        })
    }

    /// Largest entrywise disagreement between the analytic and the
    /// central-difference Jacobian, relative to the entry (floored at 1).
    fn jacobian_mismatch(spec: &SystemSpec, z: &[f64], x: &DVector<f64>, s_idx: usize) -> f64 {
        let (_, roots) = residual(spec, z, x, s_idx, x[s_idx]).unwrap();
        let got = jacobian(spec, z, x, s_idx, roots).unwrap();
        let fd = jacobian_fd(spec, z, x, s_idx, x[s_idx]).unwrap();
        got.iter()
            .zip(fd.iter())
            .map(|(a, b)| (a - b).abs() / b.abs().max(1.0))
            .fold(0.0, f64::max)
    }

    /// The analytic Jacobian against a central-difference oracle at every
    /// point of two traced envelopes — up the bubble branch, through the
    /// critical point (where the two roots merge) and down the dew branch —
    /// for every choice of continuation variable. A point where the
    /// min-Gibbs root flips inside the ±h stencil makes the oracle itself
    /// meaningless, so such points are skipped and counted, never silently.
    #[test]
    fn analytic_jacobian_matches_finite_differences_along_the_envelope() {
        for comps in [[methane(), ethane()], [n_butane(), n_heptane()]] {
            let spec = pr(&comps);
            let z = [0.5, 0.5];
            let env = trace_envelope(&spec, &z, 100.0, 400).unwrap();
            let mut checked = 0;
            for pt in &env {
                let x = unknowns(pt);
                let (_, r0) = residual(&spec, &z, &x, 3, x[3]).unwrap();
                let flips = (0..x.len()).any(|c| {
                    let h = 1e-6 * x[c].abs().max(1.0);
                    [h, -h].iter().any(|d| {
                        let mut xs = x.clone();
                        xs[c] += d;
                        residual(&spec, &z, &xs, 3, xs[3]).unwrap().1 != r0
                    })
                });
                if flips {
                    continue;
                }
                for s_idx in 0..x.len() {
                    let err = jacobian_mismatch(&spec, &z, &x, s_idx);
                    assert!(
                        err < 1e-6,
                        "{}: T={} P={} s={s_idx}: mismatch {err:e}",
                        comps[0].name,
                        pt.t,
                        pt.p
                    );
                }
                checked += 1;
            }
            assert!(
                checked * 10 >= env.len() * 9,
                "{}: compared only {checked} of {} points",
                comps[0].name,
                env.len()
            );
        }
    }

    /// Same oracle for the non-classical mixing rules, whose composition
    /// Jacobian comes from the fixed-width dual path rather than the
    /// analytic closed form, at a ternary (state taken off a classical
    /// trace; the Jacobian is defined at any X, converged or not).
    #[test]
    fn analytic_jacobian_matches_finite_differences_for_ge_mixing_rules() {
        use crate::activity::ActivityModel;
        let comps = [
            Component {
                liquid_volume: 52.0,
                ..methane()
            },
            Component {
                liquid_volume: 68.0,
                ..ethane()
            },
            Component {
                liquid_volume: 101.0,
                ..n_butane()
            },
        ];
        let z = [0.4, 0.35, 0.25];
        let classical = pr(&comps);
        let env = trace_envelope(&classical, &z, 500.0, 6).unwrap();
        let aij = vec![
            vec![0.0, 150.0, 400.0],
            vec![-120.0, 0.0, 200.0],
            vec![-300.0, -150.0, 0.0],
        ];
        let vl: Vec<f64> = comps.iter().map(|c| c.liquid_volume).collect();
        for rule in [
            MixingRule::WongSandler,
            MixingRule::MHV1,
            MixingRule::HuronVidalOriginal,
        ] {
            let spec = SystemSpec {
                mixing_rule: rule,
                aij: &aij,
                vl: &vl,
                ge_model: Some(ActivityModel::Wilson),
                ..pr(&comps)
            };
            for pt in &env {
                let x = unknowns(pt);
                for s_idx in [z.len(), z.len() + 1] {
                    let err = jacobian_mismatch(&spec, &z, &x, s_idx);
                    assert!(
                        err < 1e-6,
                        "{rule:?} T={} P={}: mismatch {err:e}",
                        pt.t,
                        pt.p
                    );
                }
            }
        }
    }

    #[test]
    fn traces_a_multi_point_envelope() {
        // Methane/ethane should trace a smooth multi-point envelope from a
        // low-pressure bubble point upward.
        let comps = [methane(), ethane()];
        let spec = pr(&comps);
        let env = trace_envelope(&spec, &[0.5, 0.5], 300.0, 40).unwrap();
        assert!(env.len() >= 5, "only traced {} points", env.len());
        // Every point is physical.
        for pt in &env {
            assert!(pt.t > 0.0 && pt.p > 0.0, "non-physical point {pt:?}");
            assert!(pt.k.iter().all(|k| k.is_finite() && *k > 0.0));
            assert!((pt.incipient.iter().sum::<f64>() - 1.0).abs() < 1e-8);
        }
    }

    #[test]
    fn envelope_points_satisfy_equal_fugacity() {
        // Each traced point must satisfy equal component fugacities:
        // ln Kᵢ + ln φ̂ᵢ(w) − ln φ̂ᵢ(z) = 0.
        let comps = [methane(), ethane()];
        let spec = pr(&comps);
        let z = [0.4, 0.6];
        let env = trace_envelope(&spec, &z, 300.0, 20).unwrap();
        for pt in &env {
            let ln_phi_z = min_gibbs_ln_phi(&spec, pt.t, pt.p, &z).unwrap();
            let ln_phi_w = min_gibbs_ln_phi(&spec, pt.t, pt.p, &pt.incipient).unwrap();
            for i in 0..2 {
                let g = pt.k[i].ln() + ln_phi_w[i] - ln_phi_z[i];
                assert!(g.abs() < 1e-6, "equal-fugacity residual {g} at {pt:?}");
            }
        }
    }

    #[test]
    fn envelope_pressure_climbs_from_the_seed() {
        // Tracing from a low seed pressure, the envelope must reach markedly
        // higher pressures (climbing toward the critical/cricondenbar).
        let comps = [methane(), ethane()];
        let spec = pr(&comps);
        let env = trace_envelope(&spec, &[0.5, 0.5], 300.0, 40).unwrap();
        let p_max = env.iter().map(|p| p.p).fold(0.0_f64, f64::max);
        assert!(
            p_max > 2.0 * 300.0,
            "envelope P_max {p_max} did not climb above the seed"
        );
    }

    fn n_butane() -> Component {
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            ..Component::default()
        }
    }

    fn n_heptane() -> Component {
        Component {
            name: "n-heptane".into(),
            tc: 540.2,
            pc: 2740.0,
            omega: 0.350,
            ..Component::default()
        }
    }

    /// Every answer the saturation solvers give must agree with the envelope:
    /// on the bubble side (incipient vapor, K of the light component > 1)
    /// `bubble_temperature` at the point's P, on the dew side
    /// `dew_temperature` — two independent solvers. Near the critical point
    /// they may decline (`Err`), never answer wrongly; before v0.17.1
    /// `bubble_temperature` returned a trivial-solution T up to 21 K off
    /// here. Returns how many points were checked.
    fn assert_matches_saturation_solvers(
        spec: &SystemSpec,
        z: &[f64],
        env: &[EnvelopePoint],
    ) -> usize {
        use crate::flash::dew::dew_temperature;
        let mut checked = 0;
        for pt in env {
            // Skip the cricondenbar/critical stretch, where one pressure can
            // carry two envelope temperatures on the same branch.
            if pt.k[0].ln().abs() < 0.1 {
                continue;
            }
            let solved = if pt.k[0] > 1.0 {
                bubble_temperature(spec, pt.p, z, 1e-10, 200)
            } else {
                dew_temperature(spec, pt.p, z, 1e-10, 200)
            };
            if let Ok(r) = solved {
                assert!(
                    (pt.t - r.value).abs() < 1e-3,
                    "envelope T {} vs saturation solver {} at P {}",
                    pt.t,
                    r.value,
                    pt.p
                );
                checked += 1;
            }
        }
        checked
    }

    #[test]
    fn closes_the_loop_through_the_critical_point_once() {
        // Regression (v0.17.0): the tangent was never oriented against the
        // direction of travel, so once the spec variable switched to an
        // ln Kᵢ the trace reversed at the critical point and oscillated
        // there for every remaining point; the dew branch was never reached.
        // Now it must go up the bubble branch, cross the critical point
        // exactly once (ln K₁ changes sign once), and come back down the dew
        // branch to the starting pressure.
        for comps in [[methane(), ethane()], [n_butane(), n_heptane()]] {
            let spec = pr(&comps);
            let z = [0.5, 0.5];
            let p_start = 100.0;
            let env = trace_envelope(&spec, &z, p_start, 400).unwrap();
            let last = env.last().unwrap();
            assert!(
                last.p <= p_start,
                "{}: did not close (stopped at P {}, {} points)",
                comps[0].name,
                last.p,
                env.len()
            );
            assert!(env.len() < 200, "{}: {} points", comps[0].name, env.len());
            let crossings = env
                .windows(2)
                .filter(|w| (w[0].k[0] > 1.0) != (w[1].k[0] > 1.0))
                .count();
            assert_eq!(crossings, 1, "{}: critical crossings", comps[0].name);
            // The dew point at p_start is hotter than the bubble point.
            assert!(last.t > env[0].t);
            let checked = assert_matches_saturation_solvers(&spec, &z, &env);
            assert!(
                checked * 4 >= env.len() * 3,
                "{}: solvers answered at only {checked} of {} points",
                comps[0].name,
                env.len()
            );
        }
    }

    #[test]
    fn traces_from_low_starting_pressures() {
        // Regression (v0.17.0): the seed was Wilson K at T = 0.7·Σzᵢ·Tc,ᵢ,
        // tens of kelvin from the bubble point, and the fixed-P corrector
        // failed for n-butane/n-heptane from p_start 50, 100 and 1000 kPa
        // (100 kPa is the Python default). The seed is now the converged
        // bubble point.
        let comps = [n_butane(), n_heptane()];
        let spec = pr(&comps);
        for p_start in [50.0, 100.0, 300.0, 1000.0] {
            let env = trace_envelope(&spec, &[0.5, 0.5], p_start, 400).unwrap();
            let p_max = env.iter().map(|p| p.p).fold(0.0_f64, f64::max);
            assert!(p_max > 3000.0, "p_start {p_start}: P_max {p_max}");
        }
    }

    #[test]
    fn rejects_non_cubic_system() {
        let comps = [methane(), ethane()];
        let mut spec = pr(&comps);
        spec.liquid = LiquidModel::IdealSolution;
        assert!(matches!(
            trace_envelope(&spec, &[0.5, 0.5], 300.0, 10),
            Err(FlashError::Unsupported(_))
        ));
    }
}
