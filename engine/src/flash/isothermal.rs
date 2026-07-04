//! Isothermal (PT) flash — Milestone 9, §J + §F.
//!
//! Given a feed composition `z` at fixed `(T, P)`, find the vapor fraction
//! `β`, the liquid composition `x`, and the vapor composition `y` at
//! equilibrium. Two nested pieces:
//!
//! 1. **Rachford-Rice** (§F) — the inner scalar solve for `β` at fixed
//!    K-values, via Halley's method (cubic convergence) inside the
//!    Leibovici–Neoschil window with a bisection safeguard. Guaranteed to
//!    converge; supports negative flash.
//! 2. **Outer K-loop** (§J) — Wilson-initialized successive substitution
//!    accelerated by the General Dominant Eigenvalue Method (GDEM) every
//!    few iterations. Each SS step: solve RR for `β`, split into `x`/`y`,
//!    recompute K from the fugacity models, repeat to composition
//!    tolerance.
//!
//! The full Michelsen scheme finishes with a Newton step on `ln K` once the
//! residual is small; GDEM-accelerated SS already converges the Chapter IV
//! cases, and the Newton finish is layered on top (`flash_isothermal` uses
//! SS+GDEM; the analytic-Jacobian Newton polish is a follow-on refinement
//! tracked in the milestone).
//!
//! # References
//! - (19) Michelsen (1982) Part II — phase-split framework
//! - (23) Leibovici & Neoschil (1992) — the Rachford-Rice window
//! - (25) Crowe & Nishio (1975) — GDEM acceleration

// Parallel-array flash math (zᵢ, Kᵢ, xᵢ, yᵢ, ln-K residuals indexed in
// lockstep) — index loops mirror the equations; allow the range-loop lint.
#![allow(clippy::needless_range_loop)]

use super::FlashError;
use super::init::wilson_k_values;
use super::system::{SystemSpec, k_values};

/// Result of an isothermal flash.
#[derive(Debug, Clone, PartialEq)]
pub struct FlashResult {
    /// Vapor fraction β = V/F, **dimensionless**. In `[0, 1]` for a genuine
    /// two-phase split; the driver clamps single-phase feeds to 0 or 1.
    pub beta: f64,
    /// Liquid mole fractions xᵢ (length N, sum to 1).
    pub x: Vec<f64>,
    /// Vapor mole fractions yᵢ (length N, sum to 1).
    pub y: Vec<f64>,
    /// Converged equilibrium ratios Kᵢ = yᵢ/xᵢ.
    pub k: Vec<f64>,
    /// Number of outer (SS/GDEM) iterations taken.
    pub iterations: usize,
    /// `true` if the feed split into two phases; `false` if the flash
    /// resolved to a single phase (β clamped to 0 or 1).
    pub two_phase: bool,
}

/// Rachford-Rice residual `f(β) = Σ zᵢ(Kᵢ−1)/(1+β(Kᵢ−1))` and its first two
/// derivatives, all from one summation pass (§F):
/// ```text
///   f   =  Σ zᵢ·cᵢ/(1+β·cᵢ),      cᵢ = Kᵢ − 1
///   f'  = −Σ zᵢ·cᵢ²/(1+β·cᵢ)²
///   f'' =  2·Σ zᵢ·cᵢ³/(1+β·cᵢ)³
/// ```
fn rr_fdd(z: &[f64], k: &[f64], beta: f64) -> (f64, f64, f64) {
    let mut f = 0.0;
    let mut df = 0.0;
    let mut ddf = 0.0;
    for (&zi, &ki) in z.iter().zip(k) {
        let c = ki - 1.0;
        let d = 1.0 + beta * c;
        let zc = zi * c;
        let inv = 1.0 / d;
        f += zc * inv;
        df -= zc * c * inv * inv;
        ddf += 2.0 * zc * c * c * inv * inv * inv;
    }
    (f, df, ddf)
}

/// Solve the Rachford-Rice equation for the vapor fraction β at fixed
/// K-values (§F).
///
/// Halley's method inside the Leibovici–Neoschil window
/// `β ∈ (1/(1−Kmax), 1/(1−Kmin))`, where f is monotone decreasing and
/// pole-free, with a bisection safeguard — so the iteration cannot diverge.
/// The returned β may lie outside `[0, 1]` (negative flash) when the feed
/// is single-phase; callers clamp as needed.
///
/// # Arguments
/// * `z` — feed mole fractions (length N, sum to 1).
/// * `k` — equilibrium ratios Kᵢ (length N).
/// * `tol` — convergence tolerance on |f(β)|.
/// * `max_iter` — iteration cap.
///
/// # Errors
/// [`FlashError::Dimension`] on length mismatch;
/// [`FlashError::NoRachfordRiceRoot`] if there is no interior root
/// (`Kmax ≤ 1` or `Kmin ≥ 1` — the mixture cannot be two-phase at these K);
/// [`FlashError::NoConvergence`] if the cap is hit.
pub fn rachford_rice(z: &[f64], k: &[f64], tol: f64, max_iter: usize) -> Result<f64, FlashError> {
    let n = z.len();
    if k.len() != n {
        return Err(FlashError::Dimension(format!("z={n}, k={}", k.len())));
    }
    let kmax = k.iter().cloned().fold(f64::MIN, f64::max);
    let kmin = k.iter().cloned().fold(f64::MAX, f64::min);
    // A bracketable interior root needs Kmax > 1 > Kmin.
    if kmax <= 1.0 || kmin >= 1.0 {
        return Err(FlashError::NoRachfordRiceRoot { kmax, kmin });
    }
    // Leibovici–Neoschil window (open interval between the two bounding
    // poles). f(β_lo⁺) = +∞, f(β_hi⁻) = −∞.
    let beta_lo = 1.0 / (1.0 - kmax); // < 0
    let beta_hi = 1.0 / (1.0 - kmin); // > 1
    // Maintain a sign bracket [lo, hi] with f(lo) > 0 > f(hi); nudge off the
    // poles so the first evaluation is finite.
    let span = beta_hi - beta_lo;
    let mut lo = beta_lo + 1e-10 * span;
    let mut hi = beta_hi - 1e-10 * span;
    let mut beta = 0.5 * (lo + hi);

    for iter in 0..max_iter {
        let (f, df, ddf) = rr_fdd(z, k, beta);
        if f.abs() <= tol {
            return Ok(beta);
        }
        // Tighten the bracket using the sign of f (f decreasing ⇒ f>0 is the
        // lower side).
        if f > 0.0 {
            lo = beta;
        } else {
            hi = beta;
        }
        // Halley step: β − 2·f·f' / (2·f'² − f·f'').
        let denom = 2.0 * df * df - f * ddf;
        let next = if denom.abs() > 0.0 {
            beta - 2.0 * f * df / denom
        } else {
            f64::NAN
        };
        // Accept Halley only if it stays strictly inside the current
        // bracket; otherwise bisect (guaranteed progress).
        beta = if next.is_finite() && next > lo && next < hi {
            next
        } else {
            0.5 * (lo + hi)
        };
        if iter + 1 == max_iter {
            let (f, _, _) = rr_fdd(z, k, beta);
            return Err(FlashError::NoConvergence {
                what: "Rachford-Rice",
                iters: max_iter,
                residual: f.abs(),
            });
        }
    }
    unreachable!("loop returns via convergence or NoConvergence")
}

/// Split a feed `z` into liquid `x` and vapor `y` at vapor fraction `β`
/// and K-values `k`: `xᵢ = zᵢ/(1+β(Kᵢ−1))`, `yᵢ = Kᵢ·xᵢ`.
fn split(z: &[f64], k: &[f64], beta: f64) -> (Vec<f64>, Vec<f64>) {
    let n = z.len();
    let mut x = vec![0.0; n];
    let mut y = vec![0.0; n];
    for i in 0..n {
        x[i] = z[i] / (1.0 + beta * (k[i] - 1.0));
        y[i] = k[i] * x[i];
    }
    (x, y)
}

/// GDEM acceleration factor from the last two ln-K residual vectors
/// (Crowe & Nishio, Ref (25)). Returns the scalar λ such that the
/// extrapolated step is `Δ·λ/(λ−1)` — here we apply the simple one-mode
/// GDEM: `μ = (r·r_prev)/(r_prev·r_prev)`, and the accelerated update
/// multiplies the SS step by `μ/(1−μ)` when `μ ∈ (0, 1)`.
fn gdem_lambda(r: &[f64], r_prev: &[f64]) -> Option<f64> {
    let mut num = 0.0;
    let mut den = 0.0;
    for (&ri, &rp) in r.iter().zip(r_prev) {
        num += ri * rp;
        den += rp * rp;
    }
    if den <= 0.0 {
        return None;
    }
    let mu = num / den;
    if mu > 0.0 && mu < 1.0 {
        Some(mu / (1.0 - mu))
    } else {
        None
    }
}

/// Isothermal (PT) flash by Wilson-initialized, GDEM-accelerated successive
/// substitution (§J).
///
/// # Arguments
/// * `spec` — the mixture's thermodynamic model.
/// * `t` — Temperature in **K**; `p` — Pressure in **kPa absolute**.
/// * `z` — feed mole fractions (length N, sum to 1).
/// * `tol` — composition-residual tolerance (‖Δ ln K‖∞).
/// * `max_iter` — outer iteration cap.
///
/// # Returns
/// A [`FlashResult`]. If the feed is single-phase at `(T, P)` the result has
/// `two_phase = false` and `β` clamped to 0 (liquid) or 1 (vapor) with
/// `x = y = z`.
///
/// # Errors
/// [`FlashError`] on dimension mismatch, thermodynamic failure, or
/// non-convergence.
pub fn flash_isothermal(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    tol: f64,
    max_iter: usize,
) -> Result<FlashResult, FlashError> {
    flash_isothermal_warm(spec, t, p, z, None, tol, max_iter)
}

/// Isothermal flash with an optional **warm-start** K-value vector (§J, §M).
///
/// Identical to [`flash_isothermal`] but seeds the K-loop from `k_init` when
/// given (e.g. the previous temperature's converged K in the adiabatic
/// flash's nested loop), falling back to Wilson otherwise. `k_init`, when
/// present, must have length N.
pub fn flash_isothermal_warm(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    z: &[f64],
    k_init: Option<&[f64]>,
    tol: f64,
    max_iter: usize,
) -> Result<FlashResult, FlashError> {
    let n = spec.n();
    if z.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, z={}",
            z.len()
        )));
    }

    // Warm-start K if supplied and shaped right, else Wilson.
    let mut k = match k_init {
        Some(k0) if k0.len() == n => k0.to_vec(),
        _ => wilson_k_values(spec.components, t, p),
    };

    let mut r_prev: Option<Vec<f64>> = None;
    let mut last_beta = 0.5;

    for iter in 0..max_iter {
        // Inner Rachford-Rice; if there's no interior root the feed is
        // single-phase at the current K — decide which side and return.
        let beta = match rachford_rice(z, &k, 1e-12, 200) {
            Ok(b) => b,
            Err(FlashError::NoRachfordRiceRoot { .. }) => {
                return Ok(single_phase(z, &k));
            }
            Err(e) => return Err(e),
        };
        last_beta = beta;
        let (x, y) = split(z, &k, beta);

        // Recompute K from the fugacity models.
        let k_new = k_values(spec, t, p, &x, &y)?;

        // ln-K residual.
        let r: Vec<f64> = (0..n).map(|i| (k_new[i] / k[i]).ln()).collect();
        let resid = r.iter().cloned().fold(0.0_f64, |m, v| m.max(v.abs()));

        if resid <= tol {
            // Converged. If β landed outside [0,1] the feed is single-phase.
            if !(0.0..=1.0).contains(&beta) {
                return Ok(single_phase(z, &k_new));
            }
            let k_final = k_new;
            return Ok(FlashResult {
                beta,
                x,
                y,
                k: k_final,
                iterations: iter + 1,
                two_phase: true,
            });
        }

        // GDEM acceleration on ln K every few iterations.
        let mut ln_k: Vec<f64> = k.iter().map(|v| v.ln()).collect();
        if let Some(r_prev_vec) = &r_prev {
            if iter % 5 == 4 {
                if let Some(lambda) = gdem_lambda(&r, r_prev_vec) {
                    for i in 0..n {
                        ln_k[i] += (1.0 + lambda) * r[i];
                    }
                    k = ln_k.iter().map(|v| v.exp()).collect();
                    r_prev = Some(r);
                    continue;
                }
            }
        }
        // Plain SS step: ln K ← ln K + r  (i.e. K ← K_new).
        k = k_new;
        r_prev = Some(r);

        if iter + 1 == max_iter {
            return Err(FlashError::NoConvergence {
                what: "isothermal flash",
                iters: max_iter,
                residual: resid,
            });
        }
    }
    let _ = last_beta;
    unreachable!("loop returns via convergence, single-phase, or NoConvergence")
}

/// Build the single-phase result: decide liquid vs vapor from the sign of
/// the Rachford-Rice residual at β = 0, and set `x = y = z`.
fn single_phase(z: &[f64], k: &[f64]) -> FlashResult {
    let (f0, _, _) = rr_fdd(z, k, 0.0);
    // f(0) = Σ zᵢ(Kᵢ−1). > 0 ⇒ would-be vapor fraction positive but no
    // interior root ⇒ superheated vapor (β=1); ≤ 0 ⇒ subcooled liquid (β=0).
    let beta = if f0 > 0.0 { 1.0 } else { 0.0 };
    FlashResult {
        beta,
        x: z.to_vec(),
        y: z.to_vec(),
        k: k.to_vec(),
        iterations: 0,
        two_phase: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::ActivityModel;
    use crate::eos::{CubicEos, LiquidModel, VaporModel};
    use crate::mixing::MixingRule;
    use crate::types::Component;

    // -----------------------------------------------------------------
    // Rachford-Rice — analytic + oracle checks.
    // -----------------------------------------------------------------

    #[test]
    fn rr_matches_hand_solution_binary() {
        // z = [0.5, 0.5], K = [2, 0.5]. f(β) = 0.5·1/(1+β) + 0.5·(−0.5)/(1−0.5β).
        // Solve: 0.5/(1+β) = 0.25/(1−0.5β) → 0.5(1−0.5β) = 0.25(1+β)
        // → 0.5 − 0.25β = 0.25 + 0.25β → 0.25 = 0.5β → β = 0.5.
        let beta = rachford_rice(&[0.5, 0.5], &[2.0, 0.5], 1e-12, 100).unwrap();
        assert!((beta - 0.5).abs() < 1e-10, "β={beta}");
    }

    #[test]
    fn rr_residual_is_zero_at_root() {
        let z = [0.3, 0.4, 0.3];
        let k = [3.0, 1.2, 0.4];
        let beta = rachford_rice(&z, &k, 1e-13, 100).unwrap();
        let (f, _, _) = rr_fdd(&z, &k, beta);
        assert!(f.abs() < 1e-10, "f(β*)={f}");
        assert!((0.0..=1.0).contains(&beta), "β={beta} should be two-phase");
    }

    #[test]
    fn rr_negative_flash_root_outside_unit_interval() {
        // Feed dominated by the light component with K slightly two-phase:
        // the root can exceed 1 (negative flash) — must still solve.
        let z = [0.98, 0.02];
        let k = [1.05, 0.2];
        let beta = rachford_rice(&z, &k, 1e-12, 100).unwrap();
        let (f, _, _) = rr_fdd(&z, &k, beta);
        assert!(f.abs() < 1e-9);
    }

    #[test]
    fn rr_rejects_single_phase_k() {
        // All K > 1 ⇒ no interior root.
        assert!(matches!(
            rachford_rice(&[0.5, 0.5], &[2.0, 1.5], 1e-12, 100),
            Err(FlashError::NoRachfordRiceRoot { .. })
        ));
        // All K < 1 ⇒ no interior root.
        assert!(matches!(
            rachford_rice(&[0.5, 0.5], &[0.9, 0.3], 1e-12, 100),
            Err(FlashError::NoRachfordRiceRoot { .. })
        ));
    }

    // -----------------------------------------------------------------
    // Full flash.
    // -----------------------------------------------------------------

    fn n_butane() -> Component {
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
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

    fn rks_system(components: &[Component]) -> SystemSpec<'_> {
        SystemSpec {
            components,
            vapor: VaporModel::Cubic(CubicEos::RKS1972),
            liquid: LiquidModel::Cubic(CubicEos::RKS1972),
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
    fn flash_two_phase_mass_balance_and_equilibrium() {
        // n-butane/n-heptane, RKS, at a T/P inside the two-phase region
        // (420 K / 1000 kPa → β ≈ 0.56; higher P compresses to a single
        // liquid, so the conditions matter).
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let z = [0.5, 0.5];
        let res = flash_isothermal(&spec, 420.0, 1000.0, &z, 1e-10, 200).unwrap();
        assert!(res.two_phase, "expected a two-phase split");
        assert!((0.0..=1.0).contains(&res.beta));
        // Overall mass balance: β·yᵢ + (1−β)·xᵢ = zᵢ.
        for i in 0..2 {
            let recombined = res.beta * res.y[i] + (1.0 - res.beta) * res.x[i];
            assert!((recombined - z[i]).abs() < 1e-8, "mass balance comp {i}");
        }
        // Compositions sum to 1.
        assert!((res.x.iter().sum::<f64>() - 1.0).abs() < 1e-8);
        assert!((res.y.iter().sum::<f64>() - 1.0).abs() < 1e-8);
        // Equilibrium: Kᵢ = yᵢ/xᵢ.
        for i in 0..2 {
            assert!((res.k[i] - res.y[i] / res.x[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn flash_isofugacity_at_convergence() {
        // At the converged split the K-values must reproduce equal
        // fugacities: recomputing K from (x, y) leaves it unchanged.
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let res = flash_isothermal(&spec, 420.0, 1000.0, &[0.5, 0.5], 1e-11, 200).unwrap();
        let k_check = k_values(&spec, 420.0, 1000.0, &res.x, &res.y).unwrap();
        for i in 0..2 {
            assert!(
                (k_check[i] / res.k[i] - 1.0).abs() < 1e-6,
                "K comp {i} drifted"
            );
        }
    }

    #[test]
    fn flash_single_phase_high_pressure_liquid() {
        // At high pressure the mixture is subcooled liquid — single phase,
        // β = 0.
        let comps = [n_butane(), n_heptane()];
        let spec = rks_system(&comps);
        let res = flash_isothermal(&spec, 350.0, 20000.0, &[0.5, 0.5], 1e-10, 200).unwrap();
        assert!(!res.two_phase);
        assert_eq!(res.beta, 0.0);
    }

    #[test]
    fn flash_gamma_phi_activity_liquid() {
        // γ-φ path: Wilson liquid + ideal vapor for a non-ideal binary.
        let a = Component {
            name: "a".into(),
            tc: 508.3,
            pc: 4762.0,
            omega: 0.665,
            liquid_volume: 76.8,
            psat_coeffs: vec![5.31, 3100.0, -60.0],
            ..Component::default()
        };
        let b = Component {
            name: "water".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            liquid_volume: 18.07,
            psat_coeffs: vec![5.11, 3800.0, -46.0],
            ..Component::default()
        };
        let comps = [a, b];
        let aij = vec![vec![0.0, 1100.0], vec![-250.0, 0.0]];
        let vl = [76.8, 18.07];
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
        let z = [0.5, 0.5];
        let res = flash_isothermal(&spec, 350.0, 80.0, &z, 1e-10, 300).unwrap();
        if res.two_phase {
            for i in 0..2 {
                let recombined = res.beta * res.y[i] + (1.0 - res.beta) * res.x[i];
                assert!((recombined - z[i]).abs() < 1e-8);
            }
        }
    }
}
