//! Mixture critical point — Heidemann–Khalil (Milestone 9, §G).
//!
//! The critical point of a mixture at fixed composition `z` is the `(T, V)`
//! where the second and third variations of the total Helmholtz energy
//! vanish along the same composition direction — Heidemann & Khalil (Ref
//! (16)). Written on the tangent-plane function
//!
//! ```text
//!   F(T, V, n) = Σ nᵢ·ln nᵢ + A_res(T, V, n)/(R·T)
//! ```
//!
//! the two conditions are:
//!
//! 1. **Spinodal-like** — the Hessian `Q_ij = ∂²F/∂nᵢ∂nⱼ` has a zero
//!    eigenvalue; let `s` be its eigenvector.
//! 2. **Critical** — the cubic form `C = Σ_ijk sᵢsⱼsₖ ∂³F/∂nᵢ∂nⱼ∂nₖ = 0`.
//!
//! ## Exact derivatives (§G, §L)
//!
//! All of `Q` and `C` come from **dual-number automatic differentiation**
//! of `F` (the `num-dual` crate, Ref (27)) — no finite differences. `F` is
//! written once generic over the scalar type; seeding a `Dual3` with a
//! composition *direction* yields the first, second, and third directional
//! derivatives in one evaluation. The Hessian is assembled from directional
//! second derivatives along the basis and its pairwise sums; the cubic form
//! is the third directional derivative along the null eigenvector.
//!
//! ## Solve structure
//!
//! Both residuals are defined at *every* `(T, V)` — `λ_min` is the smallest
//! Hessian eigenvalue and `C` is the cubic form along its eigenvector — so
//! the two conditions are solved **simultaneously** by a 2-D Newton on
//! `(T, V)`, seeded from the mole-fraction-average Tc and the PR
//! `Zc ≈ 0.307` critical volume. The 2×2 Jacobian is a numerical
//! finite-difference of the two residuals (an outer-loop Jacobian, not a
//! thermodynamic composition derivative), and the step is damped to keep
//! `T > 0` and `V > b_mix`. (A nested bisection can't cross Tc, because
//! below Tc the instability region is a narrow dip in `λ_min(V)` that
//! vanishes at the critical point — the simultaneous form avoids that.)
//!
//! Restricted to two-parameter cubic EOS with classical mixing — the case
//! Chapter IV validates (PR). The critical pressure is recovered from the
//! EOS at the converged `(Tc, Vc)`.
//!
//! # References
//! - (16) Heidemann & Khalil (1980) — mixture critical points
//! - (27) num-dual — dual-number AD

// Dense parallel-array numerical code (nᵢ, bᵢ, a_ij, basis directions
// indexed in lockstep) — index loops mirror the Hessian/cubic-form math.
#![allow(clippy::needless_range_loop)]

use nalgebra::DMatrix;
use num_dual::{Dual3, Dual3_64, DualNum};

use super::FlashError;
use super::system::SystemSpec;
use crate::eos::{CubicEos, LiquidModel, family_constants};
use crate::types::{Component, R_GAS};

/// A converged mixture critical point.
#[derive(Debug, Clone, PartialEq)]
pub struct CriticalPoint {
    /// Critical temperature in **K**.
    pub tc: f64,
    /// Critical pressure in **kPa**.
    pub pc: f64,
    /// Critical molar volume in **m³/kmol** (= L/mol).
    pub vc: f64,
}

/// Per-component dimensional EOS parameters at temperature `t`.
/// `a_i` in **kPa·m⁶/kmol²**, `b_i` in **m³/kmol** (R in kPa·m³/(kmol·K)).
struct PureAb {
    a: Vec<f64>,
    b: Vec<f64>,
}

fn pure_ab(eos: CubicEos, t: f64, comps: &[Component]) -> PureAb {
    let fc = family_constants(eos);
    let mut a = Vec::with_capacity(comps.len());
    let mut b = Vec::with_capacity(comps.len());
    for c in comps {
        let tr = t / c.tc;
        let alpha = crate::eos::alpha(eos, tr, c);
        a.push(fc.om_a * alpha * R_GAS * R_GAS * c.tc * c.tc / c.pc);
        b.push(fc.om_b * R_GAS * c.tc / c.pc);
    }
    PureAb { a, b }
}

/// The tangent-plane function `F(T, V, n)`, generic over the scalar type of
/// the mole numbers `n` (T, V are constants). `a_ij` and `b` are the
/// composition-independent dimensional parameters at `T`; `delta1`, `delta2`
/// are the roots of the attractive denominator (`x² + k1·x + k2`).
fn f_tpd<D: DualNum<f64> + Copy>(
    t: f64,
    v: f64,
    n: &[D],
    a_ij: &[Vec<f64>],
    b: &[f64],
    delta1: f64,
    delta2: f64,
) -> D {
    let ncomp = n.len();
    let mut n_tot = D::from(0.0);
    let mut n_b = D::from(0.0);
    let mut ideal = D::from(0.0);
    for i in 0..ncomp {
        n_tot += n[i];
        n_b += n[i] * b[i];
        ideal += n[i] * n[i].ln();
    }
    let mut n_a = D::from(0.0);
    for i in 0..ncomp {
        for j in 0..ncomp {
            n_a += n[i] * n[j] * a_ij[i][j];
        }
    }
    let diff = delta1 - delta2;
    // Repulsive term: −n·ln((V − n_b)/V).
    let rep = -n_tot * ((-n_b + v) * (1.0 / v)).ln();
    // Attractive term: −(n_a/(n_b·R·T·(δ1−δ2)))·ln((V+δ1·n_b)/(V+δ2·n_b)).
    let rt_diff = R_GAS * t * diff;
    let coeff = n_a / (n_b * rt_diff);
    let ln_term = ((n_b * delta1 + v) / (n_b * delta2 + v)).ln();
    ideal + rep - coeff * ln_term
}

/// Second directional derivative of F along `dir` at composition `z` (the
/// `v2` part of a Dual3 seeded with the direction).
#[allow(clippy::too_many_arguments)]
fn dir_second(
    z: &[f64],
    dir: &[f64],
    t: f64,
    v: f64,
    a_ij: &[Vec<f64>],
    b: &[f64],
    d1: f64,
    d2: f64,
) -> f64 {
    let n: Vec<Dual3_64> = (0..z.len())
        .map(|i| Dual3::new(z[i], dir[i], 0.0, 0.0))
        .collect();
    f_tpd(t, v, &n, a_ij, b, d1, d2).v2
}

/// Third directional derivative of F along `dir` (the `v3` part).
#[allow(clippy::too_many_arguments)]
fn dir_third(
    z: &[f64],
    dir: &[f64],
    t: f64,
    v: f64,
    a_ij: &[Vec<f64>],
    b: &[f64],
    d1: f64,
    d2: f64,
) -> f64 {
    let n: Vec<Dual3_64> = (0..z.len())
        .map(|i| Dual3::new(z[i], dir[i], 0.0, 0.0))
        .collect();
    f_tpd(t, v, &n, a_ij, b, d1, d2).v3
}

/// Assemble the Hessian `Q` and return its smallest eigenvalue + eigenvector.
fn smallest_eigen(
    z: &[f64],
    t: f64,
    v: f64,
    a_ij: &[Vec<f64>],
    b: &[f64],
    d1: f64,
    d2: f64,
) -> (f64, Vec<f64>) {
    let ncomp = z.len();
    let mut basis = vec![0.0; ncomp];
    // Diagonal Q_ii from the pure basis directions.
    let mut diag = vec![0.0; ncomp];
    for i in 0..ncomp {
        basis[i] = 1.0;
        diag[i] = dir_second(z, &basis, t, v, a_ij, b, d1, d2);
        basis[i] = 0.0;
    }
    let mut q = DMatrix::<f64>::zeros(ncomp, ncomp);
    for i in 0..ncomp {
        q[(i, i)] = diag[i];
        for j in (i + 1)..ncomp {
            // Q_ij = (F''[e_i+e_j] − Q_ii − Q_jj)/2.
            basis[i] = 1.0;
            basis[j] = 1.0;
            let mixed = dir_second(z, &basis, t, v, a_ij, b, d1, d2);
            basis[i] = 0.0;
            basis[j] = 0.0;
            let val = 0.5 * (mixed - diag[i] - diag[j]);
            q[(i, j)] = val;
            q[(j, i)] = val;
        }
    }
    let eig = q.symmetric_eigen();
    // Smallest eigenvalue + its eigenvector column.
    let mut min_idx = 0;
    let mut min_val = f64::MAX;
    for (idx, &lam) in eig.eigenvalues.iter().enumerate() {
        if lam < min_val {
            min_val = lam;
            min_idx = idx;
        }
    }
    let mut vec: Vec<f64> = eig.eigenvectors.column(min_idx).iter().copied().collect();
    // Fix the eigenvector sign (eigenvectors are defined up to ±): make the
    // largest-magnitude component positive, so the cubic form varies
    // continuously across (T, V) instead of flipping with an arbitrary sign.
    let lead = vec
        .iter()
        .cloned()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);
    if vec[lead] < 0.0 {
        for c in vec.iter_mut() {
            *c = -*c;
        }
    }
    (min_val, vec)
}

/// Molar covolume `b_mix = Σ zᵢ bᵢ` (the lower bound on V).
fn b_mix(z: &[f64], b: &[f64]) -> f64 {
    z.iter().zip(b).map(|(zi, bi)| zi * bi).sum()
}

/// The two Heidemann–Khalil criticality residuals at `(T, V)`:
/// `(λ_min, C)` — the smallest Hessian eigenvalue and the cubic form along
/// its eigenvector. Both vanish at the critical point. Defined for every
/// `(T, V)` (no spinodal pre-solve needed), which is what lets a 2-D Newton
/// converge on both at once.
fn residuals(
    z: &[f64],
    t: f64,
    v: f64,
    eos: CubicEos,
    comps: &[Component],
    kij: &[Vec<f64>],
) -> (f64, f64) {
    let PureAb { a, b } = pure_ab(eos, t, comps);
    let a_ij = build_aij(&a, kij, comps.len());
    let fc = family_constants(eos);
    let diff = (fc.k1 * fc.k1 - 4.0 * fc.k2).sqrt();
    let (d1, d2) = ((fc.k1 + diff) / 2.0, (fc.k1 - diff) / 2.0);
    let (lam, evec) = smallest_eigen(z, t, v, &a_ij, &b, d1, d2);
    let cubic = dir_third(z, &evec, t, v, &a_ij, &b, d1, d2);
    (lam, cubic)
}

/// Classical cross-parameter matrix `a_ij = (1 − k_ij)·√(aᵢ·aⱼ)`.
fn build_aij(a: &[f64], kij: &[Vec<f64>], n: usize) -> Vec<Vec<f64>> {
    let kij_at = |i: usize, j: usize| {
        if kij.is_empty() { 0.0 } else { kij[i][j] }
    };
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| (1.0 - kij_at(i, j)) * (a[i] * a[j]).sqrt())
                .collect()
        })
        .collect()
}

/// Compute the mixture critical point at composition `z` (§G) by a 2-D
/// Newton on the two Heidemann–Khalil residuals `{λ_min(T,V), C(T,V)} = 0`.
///
/// # Arguments
/// * `spec` — the mixture model (two-parameter cubic + classical mixing).
/// * `z` — composition (length N, sum to 1).
/// * `t_init` — initial temperature guess in **K** (e.g. `Σ zᵢ·Tc,ᵢ`).
///   Pass `0.0` to use that mole-fraction average automatically.
/// * `max_iter` — Newton iteration cap.
///
/// # Returns
/// [`CriticalPoint`] with Tc (**K**), Pc (**kPa**), Vc (**m³/kmol**).
///
/// # Errors
/// [`FlashError::Unsupported`] for a non-cubic or 3-parameter EOS;
/// [`FlashError::NoConvergence`] if the Newton fails to converge.
pub fn critical_point(
    spec: &SystemSpec,
    z: &[f64],
    t_init: f64,
    max_iter: usize,
) -> Result<CriticalPoint, FlashError> {
    let eos = match spec.liquid {
        LiquidModel::Cubic(e) if !e.is_three_parameter() => e,
        _ => {
            return Err(FlashError::Unsupported(
                "critical point requires a two-parameter cubic EOS".into(),
            ));
        }
    };
    let comps = spec.components;
    if z.len() != comps.len() {
        return Err(FlashError::Dimension(format!(
            "components={}, z={}",
            comps.len(),
            z.len()
        )));
    }
    let kij = spec.kij;

    // Initial guesses: T from the mole-fraction-average Tc; V from the PR
    // critical compressibility Zc ≈ 0.307 and a pseudo-critical pressure.
    let t0 = if t_init > 0.0 {
        t_init
    } else {
        z.iter().zip(comps).map(|(zi, c)| zi * c.tc).sum()
    };
    let p0: f64 = z.iter().zip(comps).map(|(zi, c)| zi * c.pc).sum();
    let mut t = t0;
    let mut v = 0.307 * R_GAS * t0 / p0;

    for iter in 0..max_iter {
        let (f1, f2) = residuals(z, t, v, eos, comps, kij);
        if f1.abs() < 1e-8 && f2.abs() < 1e-7 {
            break;
        }
        // Numerical 2×2 Jacobian of (λ_min, C) w.r.t. (T, V) — an outer-loop
        // Jacobian (not a thermodynamic composition derivative), so a finite
        // difference here is fine.
        let dt = 1e-3 * t.max(1.0);
        let dv = 1e-4 * v.max(1e-4);
        let (f1t, f2t) = residuals(z, t + dt, v, eos, comps, kij);
        let (f1v, f2v) = residuals(z, t, v + dv, eos, comps, kij);
        let j11 = (f1t - f1) / dt;
        let j12 = (f1v - f1) / dv;
        let j21 = (f2t - f2) / dt;
        let j22 = (f2v - f2) / dv;
        let det = j11 * j22 - j12 * j21;
        if det.abs() < 1e-300 {
            return Err(FlashError::NoConvergence {
                what: "critical point (singular Jacobian)",
                iters: iter,
                residual: f1.abs().max(f2.abs()),
            });
        }
        // Solve J·Δ = −f; damp the step so it can't leave the physical region.
        let mut d_t = -(j22 * f1 - j12 * f2) / det;
        let mut d_v = -(-j21 * f1 + j11 * f2) / det;
        let bm = {
            let PureAb { b, .. } = pure_ab(eos, t, comps);
            b_mix(z, &b)
        };
        // Limit the step to keep T > 0 and V > b_mix.
        while t + d_t <= 0.0 || v + d_v <= bm * 1.01 {
            d_t *= 0.5;
            d_v *= 0.5;
            if d_t.abs() < 1e-12 && d_v.abs() < 1e-12 {
                break;
            }
        }
        t += d_t;
        v += d_v;
        if iter + 1 == max_iter {
            let (f1, f2) = residuals(z, t, v, eos, comps, kij);
            return Err(FlashError::NoConvergence {
                what: "critical point",
                iters: max_iter,
                residual: f1.abs().max(f2.abs()),
            });
        }
    }

    // Pc from the EOS at the converged (Tc, Vc).
    let PureAb { a, b } = pure_ab(eos, t, comps);
    let a_ij = build_aij(&a, kij, comps.len());
    let fc = family_constants(eos);
    let diff = (fc.k1 * fc.k1 - 4.0 * fc.k2).sqrt();
    let (d1, d2) = ((fc.k1 + diff) / 2.0, (fc.k1 - diff) / 2.0);
    let bm = b_mix(z, &b);
    let mut a_mix = 0.0;
    for i in 0..comps.len() {
        for j in 0..comps.len() {
            a_mix += z[i] * z[j] * a_ij[i][j];
        }
    }
    let pc = R_GAS * t / (v - bm) - a_mix / ((v + d1 * bm) * (v + d2 * bm));

    Ok(CriticalPoint { tc: t, pc, vc: v })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos::VaporModel;
    use crate::mixing::MixingRule;

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
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        }
    }

    #[test]
    fn pure_component_critical_recovers_tc_pc() {
        // For a pure component (z=[1]) the mixture critical point must return
        // the component's own Tc and Pc (PR reproduces them by construction).
        let comps = [ethane()];
        let spec = pr(&comps);
        let cp = critical_point(&spec, &[1.0], 305.0, 200).unwrap();
        assert!((cp.tc - 305.32).abs() < 1.0, "Tc={} vs 305.32", cp.tc);
        assert!(
            (cp.pc - 4872.0).abs() / 4872.0 < 0.02,
            "Pc={} vs 4872",
            cp.pc
        );
    }

    #[test]
    fn binary_critical_between_pure_criticals() {
        // A methane/ethane mixture critical T must lie between the two pure
        // Tc's (true for this near-ideal pair).
        let comps = [methane(), ethane()];
        let spec = pr(&comps);
        let cp = critical_point(&spec, &[0.5, 0.5], 250.0, 200).unwrap();
        assert!(
            cp.tc > 190.564 && cp.tc < 305.32,
            "mixture Tc={} not between pure Tc's",
            cp.tc
        );
        assert!(cp.pc > 0.0 && cp.vc > 0.0);
    }

    #[test]
    fn rejects_three_parameter_eos() {
        let comps = [methane(), ethane()];
        let mut spec = pr(&comps);
        spec.liquid = LiquidModel::Cubic(CubicEos::PatelTeja);
        assert!(matches!(
            critical_point(&spec, &[0.5, 0.5], 250.0, 100),
            Err(FlashError::Unsupported(_))
        ));
    }
}
