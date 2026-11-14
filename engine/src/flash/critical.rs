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
//! ## Not every root is a critical point
//!
//! The two conditions are local, and they have roots no fluid can sit at:
//! at absurdly low temperature with negative pressure (85 % methane / 15 %
//! n-pentane from the default seed: 10.9 K, −142 543 kPa), and inside the
//! two-phase region, where the feed has already split. [`critical_point`]
//! therefore vets what the Newton returns — positive pressure, a temperature
//! above half the lightest component's Tc, and no conclusive "unstable" from
//! the tangent-plane test at that (T, P) — and on a rejection or a failure
//! re-seeds from Heidemann & Khalil's own starting point, `V = 4·b_mix` at
//! 1.5, 1.2, 1.0 and 2.0 times the mole-fraction-average Tc. If no seed
//! gives a root that passes, the result is `NoConvergence`: a mixture whose
//! every root is rejected (methane-rich methane/n-pentane above ≈ 95 %
//! methane, on this model) has no answer here, and saying so is the answer.
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
use super::stability::{Stability, stability_analysis};
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
/// [`FlashError::NoConvergence`] if no seed converges to a root that is a
/// physical critical point (see the module docs: positive pressure, a
/// plausible temperature, and not inside the two-phase region).
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

    // Seed ladder. The first seed is the caller's (or the mole-fraction
    // average Tc) with the PR `Zc ≈ 0.307` volume — the seed every earlier
    // release used, so an answer that was right stays bit-identical. The
    // rest are Heidemann & Khalil's own recommendation, `V = 4·b_mix` at a
    // multiple of the average Tc: approached from the hot, dilute side the
    // Newton lands on the highest-temperature critical point.
    let t_avg: f64 = z.iter().zip(comps).map(|(zi, c)| zi * c.tc).sum();
    let p_avg: f64 = z.iter().zip(comps).map(|(zi, c)| zi * c.pc).sum();
    let t0 = if t_init > 0.0 { t_init } else { t_avg };
    let bm0 = {
        let PureAb { b, .. } = pure_ab(eos, t_avg, comps);
        b_mix(z, &b)
    };
    let mut seeds = vec![(t0, 0.307 * R_GAS * t0 / p_avg)];
    for f in HK_SEED_FACTORS {
        seeds.push((f * t_avg, 4.0 * bm0));
    }

    // The two Heidemann–Khalil conditions have roots that are not critical
    // points of a real fluid: for 85 % methane / 15 % n-pentane the first
    // seed converges — to 1e-8, a genuine root — at 10.9 K and −142 543 kPa.
    // A root is accepted only if it passes `is_physical`; otherwise the next
    // seed is tried. The first error is kept for the case where none works.
    let t_floor = PHYSICAL_T_FRACTION * comps.iter().map(|c| c.tc).fold(f64::MAX, f64::min);
    let mut first_err = None;
    for (ts, vs) in seeds {
        match solve_from(z, ts, vs, eos, comps, kij, max_iter) {
            Ok(cp) if is_physical(&cp, t_floor) && !inside_two_phase_region(spec, z, &cp) => {
                return Ok(cp);
            }
            Ok(cp) => {
                first_err.get_or_insert(FlashError::NoConvergence {
                    what: "critical point (only a non-physical root found)",
                    iters: max_iter,
                    residual: cp.pc,
                });
            }
            Err(e) => {
                first_err.get_or_insert(e);
            }
        }
    }
    Err(first_err.expect("the seed ladder is never empty"))
}

/// Multiples of the mole-fraction-average Tc tried, at `V = 4·b_mix`, after
/// the first seed fails or returns a non-physical root.
const HK_SEED_FACTORS: [f64; 4] = [1.5, 1.2, 1.0, 2.0];

/// A critical temperature below this fraction of the lightest component's Tc
/// is rejected as a non-physical root of the criticality conditions.
const PHYSICAL_T_FRACTION: f64 = 0.5;

/// Is this root of the criticality conditions a critical point a fluid can
/// have? Positive finite pressure, and a temperature (**K**) above `t_floor`.
fn is_physical(cp: &CriticalPoint, t_floor: f64) -> bool {
    cp.pc.is_finite() && cp.pc > 0.0 && cp.tc.is_finite() && cp.tc > t_floor && cp.vc > 0.0
}

/// TPD below this at a candidate critical point means the feed splits there.
const UNSTABLE_TPD: f64 = -1e-6;

/// Trial-phase iteration cap for the vetting stability test. Deliberately
/// small: a root inside the two-phase region shows a negative TPD within a
/// few iterations, while at a true critical point the trials never converge
/// and every extra iteration is wasted. Measured on 545 compositions each of
/// methane/n-pentane and methane/n-heptane: caps of 50, 200 and 2 000 reject
/// exactly the same roots, and the 2-component solve costs 33 µs, 113 µs and
/// 1 039 µs (5.4 µs with no check). Do not raise it.
const VET_STABILITY_ITERS: usize = 50;

/// Does the stability test prove the feed splits at this candidate? The
/// criticality conditions are local — they also hold at points buried inside
/// the two-phase region (methane-rich methane/n-pentane has one near 193 K),
/// where no critical point can be observed. Only a conclusive "unstable"
/// rejects: at a true critical point the trials converge slowly and usually
/// return `NoConvergence`, which proves nothing and is not held against it.
fn inside_two_phase_region(spec: &SystemSpec, z: &[f64], cp: &CriticalPoint) -> bool {
    matches!(
        stability_analysis(spec, cp.tc, cp.pc, z, VET_STABILITY_ITERS),
        Ok(Stability::Unstable { tpd, .. }) if tpd < UNSTABLE_TPD
    )
}

/// One 2-D Newton solve of `{λ_min, C} = 0` from the seed `(t, v)` —
/// `t` in **K**, `v` in **m³/kmol**. Returns whatever root it converges to,
/// physical or not; [`critical_point`] does the vetting.
fn solve_from(
    z: &[f64],
    mut t: f64,
    mut v: f64,
    eos: CubicEos,
    comps: &[Component],
    kij: &[Vec<f64>],
    max_iter: usize,
) -> Result<CriticalPoint, FlashError> {
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
            alpha: &[],
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

    fn n_pentane() -> Component {
        Component {
            name: "n-pentane".into(),
            tc: 469.7,
            pc: 3370.0,
            omega: 0.252,
            ..Component::default()
        }
    }

    /// 85 % methane / 15 % n-pentane: the default seed converges on a root
    /// of the criticality conditions at ≈ 11 K and a large negative
    /// pressure, and until 2026-11-14 that was returned as `Ok`.
    #[test]
    fn a_non_physical_root_is_not_returned() {
        let comps = [methane(), n_pentane()];
        let spec = pr(&comps);
        let z = [0.85, 0.15];
        // The root is real: the bare Newton from the old seed still finds it.
        let t_avg = 0.85 * 190.564 + 0.15 * 469.7;
        let p_avg = 0.85 * 4599.0 + 0.15 * 3370.0;
        let raw = solve_from(
            &z,
            t_avg,
            0.307 * R_GAS * t_avg / p_avg,
            CubicEos::PR1976,
            &comps,
            &[],
            200,
        )
        .unwrap();
        assert!(raw.pc < 0.0 && raw.tc < 50.0, "raw root {raw:?}");
        // The public solver rejects it and finds the critical point.
        let cp = critical_point(&spec, &z, 0.0, 200).unwrap();
        assert!(cp.tc > 190.564 && cp.tc < 469.7, "Tc={}", cp.tc);
        assert!(cp.pc > 4599.0 && cp.pc < 25000.0, "Pc={}", cp.pc);
    }

    /// Across the composition range every answer is either a physical
    /// critical point on a smooth locus or an error — never a silent wrong
    /// root. On `main`, of 545 evenly spaced compositions 11 came back with
    /// negative pressure and 51 failed; now none and 25. This test walks
    /// every fifth one to stay quick in a debug build.
    #[test]
    fn methane_n_pentane_locus_is_physical_or_an_error() {
        let comps = [methane(), n_pentane()];
        let spec = pr(&comps);
        let (mut ok, mut failed) = (0, 0);
        let mut prev: Option<(usize, f64)> = None;
        for i in (5..546).step_by(5) {
            let z1 = i as f64 / 546.0;
            match critical_point(&spec, &[z1, 1.0 - z1], 0.0, 200) {
                Ok(cp) => {
                    ok += 1;
                    assert!(
                        cp.pc > 0.0 && cp.tc > 190.0 && cp.tc < 470.0,
                        "z1={z1} {cp:?}"
                    );
                    if let Some((i_prev, t_prev)) = prev {
                        // Adjacent samples: the locus moves well under 10 K.
                        if i - i_prev == 5 {
                            assert!((cp.tc - t_prev).abs() < 10.0, "jump at z1={z1}");
                        }
                    }
                    prev = Some((i, cp.tc));
                }
                Err(FlashError::NoConvergence { .. }) => failed += 1,
                Err(e) => panic!("z1={z1}: {e}"),
            }
        }
        assert!(ok >= 100, "only {ok} critical points found");
        assert!(failed <= 8, "{failed} failures");
    }

    /// A root buried in the two-phase region is refused. Methane-rich
    /// methane/n-pentane has such roots near 193 K; the stability test
    /// proves the feed splits there.
    #[test]
    fn a_root_inside_the_two_phase_region_is_refused() {
        let comps = [methane(), n_pentane()];
        let spec = pr(&comps);
        for z1 in [0.963, 0.967, 0.969] {
            match critical_point(&spec, &[z1, 1.0 - z1], 0.0, 200) {
                Err(FlashError::NoConvergence { .. }) => {}
                Ok(cp) => assert!(
                    !inside_two_phase_region(&spec, &[z1, 1.0 - z1], &cp),
                    "z1={z1}: returned {cp:?}, which the stability test rejects"
                ),
                Err(e) => panic!("z1={z1}: {e}"),
            }
        }
    }
}
