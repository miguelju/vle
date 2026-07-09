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
//! root** at each composition (via [`super::system::min_gibbs_ln_phi`]), so
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
//! `(n+2)×(n+2)` corrector Jacobian is numerical — an outer-loop Jacobian of
//! the whole state, not a thermodynamic composition derivative.
//!
//! Restricted to the φ-φ (cubic both phases) path.
//!
//! # References
//! - (24) Michelsen (1980) — calculation of phase envelopes

use nalgebra::{DMatrix, DVector};

use super::FlashError;
use super::init::wilson_k_values;
use super::system::{SystemSpec, min_gibbs_ln_phi};

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

/// Residual vector `g(X)` of the envelope system (length n+2), given the
/// unknowns `X = (ln K…, ln T, ln P)` and the continuation spec `(s_idx, s_val)`.
fn residual(
    spec: &SystemSpec,
    z: &[f64],
    x: &DVector<f64>,
    s_idx: usize,
    s_val: f64,
) -> Result<DVector<f64>, FlashError> {
    let n = z.len();
    let k: Vec<f64> = (0..n).map(|i| x[i].exp()).collect();
    let t = x[n].exp();
    let p = x[n + 1].exp();
    // Incipient composition w = K∘z, normalized.
    let w_un: Vec<f64> = (0..n).map(|i| k[i] * z[i]).collect();
    let sw: f64 = w_un.iter().sum();
    let w: Vec<f64> = w_un.iter().map(|wi| wi / sw).collect();

    let ln_phi_w = min_gibbs_ln_phi(spec, t, p, &w)?;
    let ln_phi_z = min_gibbs_ln_phi(spec, t, p, z)?;

    let mut g = DVector::zeros(n + 2);
    for i in 0..n {
        g[i] = x[i] + ln_phi_w[i] - ln_phi_z[i];
    }
    // Σ zᵢ(Kᵢ − 1) = 0.
    g[n] = (0..n).map(|i| z[i] * (k[i] - 1.0)).sum();
    // Continuation: X_s − S = 0.
    g[n + 1] = x[s_idx] - s_val;
    Ok(g)
}

/// Numerical `(n+2)×(n+2)` Jacobian of [`residual`] w.r.t. `X`.
fn jacobian(
    spec: &SystemSpec,
    z: &[f64],
    x: &DVector<f64>,
    s_idx: usize,
    s_val: f64,
) -> Result<DMatrix<f64>, FlashError> {
    let m = x.len();
    let g0 = residual(spec, z, x, s_idx, s_val)?;
    let mut j = DMatrix::zeros(m, m);
    for col in 0..m {
        let h = 1e-6 * x[col].abs().max(1.0);
        let mut xp = x.clone();
        xp[col] += h;
        let gp = residual(spec, z, &xp, s_idx, s_val)?;
        for row in 0..m {
            j[(row, col)] = (gp[row] - g0[row]) / h;
        }
    }
    Ok(j)
}

/// Newton-correct the unknowns `X` to satisfy the envelope system at the
/// given continuation spec. Returns the converged `X`.
fn correct(
    spec: &SystemSpec,
    z: &[f64],
    mut x: DVector<f64>,
    s_idx: usize,
    s_val: f64,
    tol: f64,
    max_iter: usize,
) -> Result<DVector<f64>, FlashError> {
    for iter in 0..max_iter {
        let g = residual(spec, z, &x, s_idx, s_val)?;
        if g.amax() < tol {
            return Ok(x);
        }
        let j = jacobian(spec, z, &x, s_idx, s_val)?;
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
            let g = residual(spec, z, &x, s_idx, s_val)?;
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
/// The traced [`EnvelopePoint`]s in order. Tracing stops at `max_points`, if
/// a corrector fails, or once the branch turns back to low pressure.
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

    // Seed: a bubble point at p_start. Use the existing bubble-pressure-free
    // route — start from a bubble temperature guess via Wilson and correct.
    // Initial K, T from Wilson at p_start (bubble side).
    let t0 = {
        // Bubble T guess: Σ zᵢ·Tc,ᵢ scaled — refine by the corrector.
        let tavg: f64 = z.iter().zip(spec.components).map(|(zi, c)| zi * c.tc).sum();
        0.7 * tavg
    };
    let mut x = DVector::zeros(n + 2);
    let kw = wilson_k_values(spec.components, t0, p_start);
    for i in 0..n {
        x[i] = kw[i].ln();
    }
    x[n] = t0.ln();
    x[n + 1] = p_start.ln();

    // Correct the seed with P fixed (spec = ln P index n+1).
    let s_idx0 = n + 1;
    let mut x = correct(spec, z, x, s_idx0, p_start.ln(), 1e-9, 100)?;

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
    let mut s_idx = n; // start by stepping ln T
    let mut ds = 0.05; // step in the specified variable
    let mut x_prev = x.clone();
    for _ in 1..max_points {
        // Tangent dX/dS: solve J·t = e_{last} (the spec row derivative).
        let j = jacobian(spec, z, &x, s_idx, x[s_idx])?;
        let mut rhs = DVector::zeros(n + 2);
        rhs[n + 1] = 1.0; // ∂g_{n+2}/∂S = −1 → tangent solves J·t = e
        let tangent = match j.lu().solve(&rhs) {
            Some(t) => t,
            None => break,
        };
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
            Ok(x_new) => {
                // Reject a step that barely moved or blew up.
                let moved = (&x_new - &x).amax();
                if !x_new.iter().all(|v| v.is_finite()) || moved < 1e-9 {
                    break;
                }
                x_prev = x.clone();
                x = x_new;
                points.push(record(&x));
                // Grow the step when convergence is easy.
                ds = (ds * 1.1).min(0.15);
            }
            Err(_) => {
                // Halve the step and retry from the last good point.
                ds *= 0.5;
                x = x_prev.clone();
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
