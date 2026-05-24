//! Broyden's "good" quasi-Newton method for multi-variable systems.
//!
//! Solves `F(x) = 0` where `F: ℝⁿ → ℝⁿ` is a vector residual function,
//! given an initial guess `x₀`. Each iteration:
//!
//! 1. Solve the linear system `J · Δx = −F(x)` for the step `Δx`.
//! 2. Update `x ← x + Δx` and evaluate `F(x)`.
//! 3. Update the Jacobian approximation by a **rank-1 secant update**
//!    (the "good Broyden" formula):
//!
//!    ```text
//!    J ← J + ((Δy − J·Δx) · Δxᵀ) / (Δxᵀ · Δx)
//!    ```
//!
//!    where `Δy = F(x_new) − F(x_old)`. The update preserves the secant
//!    equation `J_new · Δx = Δy` and costs `O(n²)` instead of `O(n³)`
//!    (full re-factorization) per step.
//!
//! After a configurable number of rank-1 updates (`refresh_every` —
//! default 5), the approximation drifts from the true Jacobian and the
//! solver recomputes `J` from scratch via finite differences (one extra
//! function evaluation per column). This keeps long-run convergence
//! Newton-like without paying the per-step `O(n³)` cost.
//!
//! ## When to use this
//!
//! Use Broyden when:
//! - The residual function is **expensive** (so the `n` extra calls per
//!   full-Jacobian step actually matter).
//! - The system is **medium-sized** (n ≲ ~100; for larger n, switch to
//!   a Krylov method like GMRES or a sparse-Jacobian variant).
//! - Each `F` evaluation is **smooth**, so the secant approximation
//!   stays valid across the step.
//!
//! In the VLE engine this is the workhorse for M9's flash and bubble/
//! dew-point iterations, where each `F` evaluation walks the EOS state
//! across all components — easily 100s of arithmetic ops, so saving the
//! `n × (cost of F)` per Jacobian recompute is worth the rank-1 cost.
//!
//! ## Why "good" Broyden
//!
//! There are two textbook variants: "good" (this one, updates `J`) and
//! "bad" (updates `J⁻¹` directly). Good Broyden's rank-1 update is
//! cheaper per iteration when paired with an in-place LU re-solve, and
//! it has slightly better numerical stability. The engine uses "good"
//! everywhere.
//!
//! Reference: Broyden, C. G. (1965), *"A class of methods for solving
//! nonlinear simultaneous equations"*; Press et al., *Numerical Recipes*,
//! §9.7; Dennis & Schnabel, *Numerical Methods for Unconstrained
//! Optimization*, §8.

use nalgebra::{DMatrix, DVector};
use thiserror::Error;

use crate::numerics::utils::{is_near_zero, norm_l2};

/// Errors returned by [`broyden`].
#[derive(Debug, Error, PartialEq)]
pub enum BroydenError {
    /// The iteration limit was hit before either `xtol` or `ftol` was
    /// satisfied. Caller should raise `max_iter`, loosen tolerances,
    /// improve the initial guess, or switch algorithms.
    #[error(
        "did not converge in {max_iter} iterations (||F||={last_f_norm}, ||Δx||={last_dx_norm})"
    )]
    NoConvergence {
        max_iter: usize,
        last_f_norm: f64,
        last_dx_norm: f64,
    },
    /// The Jacobian became singular (LU solve failed). Often means a
    /// bad initial guess for an underdetermined system, or `F` is flat
    /// in some direction. A finite-difference Jacobian refresh may
    /// recover — but if `J` is singular at step 0, no refresh has yet
    /// happened and this returns immediately.
    #[error("Jacobian became singular at iteration {iter}; cannot solve for step")]
    SingularJacobian { iter: usize },
    /// `F(x)` returned a vector containing NaN or ±∞. Usually a domain
    /// error in the residual function (e.g., taking a log of a
    /// negative value mid-iteration).
    #[error("F returned non-finite value at x = {x:?}")]
    NonFiniteEvaluation { x: Vec<f64> },
    /// `F(x)` returned a vector of the wrong length. Should be caught
    /// at the call site, but we re-check defensively because the cost
    /// is one comparison per iteration and the failure mode is
    /// otherwise a silent dimension mismatch deep inside the linear
    /// algebra.
    #[error("F returned a vector of length {got}, expected {expected}")]
    DimensionMismatch { expected: usize, got: usize },
}

/// Tunable parameters for [`broyden`]. Sensible defaults are exposed
/// through [`BroydenConfig::default`]; tighten or loosen at the call
/// site as the problem requires.
#[derive(Debug, Clone, Copy)]
pub struct BroydenConfig {
    /// Convergence tolerance on `‖Δx‖₂` (the most recent step).
    pub xtol: f64,
    /// Convergence tolerance on `‖F(x)‖₂` (the residual).
    pub ftol: f64,
    /// Hard iteration cap.
    pub max_iter: usize,
    /// Recompute `J` from scratch via finite differences every this
    /// many rank-1 updates. `0` means *never* refresh (rank-1 only —
    /// dangerous for long runs). The default `5` is a textbook
    /// compromise: rank-1 enough to amortize the FD cost, frequent
    /// enough that approximation drift doesn't kill convergence.
    pub refresh_every: usize,
    /// Step size for the finite-difference Jacobian (initial + refresh).
    /// Should be ≈ √ε ≈ 1.5e-8 for problems scaled to O(1); larger for
    /// noisy `F`, smaller if you're confident in `F`'s smoothness.
    pub fd_step: f64,
}

impl Default for BroydenConfig {
    fn default() -> Self {
        Self {
            xtol: 1e-8,
            ftol: 1e-8,
            max_iter: 100,
            refresh_every: 5,
            fd_step: 1e-7,
        }
    }
}

/// Solve `F(x) = 0` using Broyden's "good" quasi-Newton iteration.
///
/// # Arguments
/// * `f` — Residual function. Takes `&[f64]` of length `n`, returns
///   `Vec<f64>` of length `n`. Called once per iteration plus `n`
///   times each time the Jacobian is refreshed.
/// * `x0` — Initial guess, length `n`. Determines the problem size.
/// * `cfg` — Tolerances + refresh cadence + FD step. Use
///   [`BroydenConfig::default`] for sensible engineering defaults.
///
/// # Returns
/// The converged `x` such that `‖F(x)‖₂ ≤ ftol` or `‖Δx‖₂ ≤ xtol`.
pub fn broyden<F>(mut f: F, x0: &[f64], cfg: BroydenConfig) -> Result<Vec<f64>, BroydenError>
where
    F: FnMut(&[f64]) -> Vec<f64>,
{
    let n = x0.len();
    let mut x = DVector::from_column_slice(x0);

    // Initial residual and Jacobian. The FD Jacobian costs n+1
    // function evaluations (one at x₀, n at perturbations).
    let mut fx = eval(&mut f, &x, n)?;
    if norm_l2(fx.as_slice()) <= cfg.ftol {
        return Ok(x.iter().copied().collect());
    }
    let mut j = finite_diff_jacobian(&mut f, &x, &fx, n, cfg.fd_step)?;
    let mut updates_since_refresh: usize = 0;

    for iter in 0..cfg.max_iter {
        // Solve J · Δx = -F for the step. nalgebra's LU re-factorizes
        // every step (we updated J via rank-1 since the last solve);
        // this is the O(n³) work per iter.
        let neg_fx = -fx.clone();
        let dx = j
            .clone()
            .lu()
            .solve(&neg_fx)
            .ok_or(BroydenError::SingularJacobian { iter })?;

        // Take the step.
        let x_new = &x + &dx;
        let fx_new = eval(&mut f, &x_new, n)?;

        let dx_norm = norm_l2(dx.as_slice());
        let f_norm = norm_l2(fx_new.as_slice());

        // Convergence: either the residual is small or the step is.
        // Both checks are reasonable; we satisfy on either.
        if f_norm <= cfg.ftol || dx_norm <= cfg.xtol {
            return Ok(x_new.iter().copied().collect());
        }

        // Decide whether to refresh J from scratch this step. The
        // refresh_every == 0 sentinel means "never refresh".
        let should_refresh =
            cfg.refresh_every != 0 && updates_since_refresh + 1 >= cfg.refresh_every;

        if should_refresh {
            j = finite_diff_jacobian(&mut f, &x_new, &fx_new, n, cfg.fd_step)?;
            updates_since_refresh = 0;
        } else {
            // Rank-1 "good Broyden" update.
            //   J ← J + ((Δy − J·Δx) · Δxᵀ) / (Δxᵀ · Δx)
            let dy = &fx_new - &fx;
            let j_dx = &j * &dx;
            let denom = dx.dot(&dx);
            // Defensive: skip update if Δx is effectively zero (would
            // divide by 0). The next iteration will pick up via the
            // unchanged J — convergence check above already covered the
            // "we're done" case, so a tiny Δx with non-converged F
            // signals a stall; do an emergency refresh instead.
            if is_near_zero(denom) {
                j = finite_diff_jacobian(&mut f, &x_new, &fx_new, n, cfg.fd_step)?;
                updates_since_refresh = 0;
            } else {
                let numerator = &dy - &j_dx;
                // Outer product: numerator · dxᵀ → n×n matrix, scaled
                // by 1/denom. nalgebra spells this with .iter() loops
                // (no built-in outer-product helper in DMatrix/DVector
                // that we can call on borrowed refs without cloning).
                for i in 0..n {
                    for k in 0..n {
                        j[(i, k)] += numerator[i] * dx[k] / denom;
                    }
                }
                updates_since_refresh += 1;
            }
        }

        x = x_new;
        fx = fx_new;

        if iter + 1 == cfg.max_iter {
            return Err(BroydenError::NoConvergence {
                max_iter: cfg.max_iter,
                last_f_norm: f_norm,
                last_dx_norm: dx_norm,
            });
        }
    }
    unreachable!("loop exits via convergence return or NoConvergence")
}

/// Evaluate `f` and convert the result into a DVector, validating
/// length and finiteness in one place.
fn eval<F>(f: &mut F, x: &DVector<f64>, n: usize) -> Result<DVector<f64>, BroydenError>
where
    F: FnMut(&[f64]) -> Vec<f64>,
{
    let raw = f(x.as_slice());
    if raw.len() != n {
        return Err(BroydenError::DimensionMismatch {
            expected: n,
            got: raw.len(),
        });
    }
    if !raw.iter().all(|v| v.is_finite()) {
        return Err(BroydenError::NonFiniteEvaluation {
            x: x.iter().copied().collect(),
        });
    }
    Ok(DVector::from_vec(raw))
}

/// One-sided finite-difference Jacobian at `x`. Reuses `fx = f(x)`
/// (already computed by the caller) so the cost is exactly `n`
/// additional function evaluations rather than `n + 1`.
fn finite_diff_jacobian<F>(
    f: &mut F,
    x: &DVector<f64>,
    fx: &DVector<f64>,
    n: usize,
    h: f64,
) -> Result<DMatrix<f64>, BroydenError>
where
    F: FnMut(&[f64]) -> Vec<f64>,
{
    let mut j = DMatrix::zeros(n, n);
    let mut x_pert = x.clone();
    for col in 0..n {
        // Scale the FD step by |x[col]| so the relative error stays
        // bounded for large-magnitude inputs.
        let h_col = h * (1.0 + x[col].abs());
        x_pert[col] = x[col] + h_col;
        let f_pert = eval(f, &x_pert, n)?;
        for row in 0..n {
            j[(row, col)] = (f_pert[row] - fx[row]) / h_col;
        }
        x_pert[col] = x[col]; // restore for the next column
    }
    Ok(j)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() < tol,
            "expected {expected}, got {actual}"
        );
    }

    /// Two-variable polynomial system with a known root:
    ///   x² + y² = 2
    ///   x·y    = 1
    /// Root: (x, y) = (1, 1)  (and the symmetric (−1, −1)).
    ///
    /// Uses tight ftol so the test can assert a tight tolerance on x.
    /// (Default ftol=1e-8 on this residual allows x to be ~5e-5 off the
    /// true root — the residual is small there because near (1, 1) it
    /// scales as ~2·Δ·max(|x|,|y|).)
    #[test]
    fn solves_simple_2x2_polynomial_system() {
        let f = |v: &[f64]| {
            let (x, y) = (v[0], v[1]);
            vec![x * x + y * y - 2.0, x * y - 1.0]
        };
        let cfg = BroydenConfig {
            ftol: 1e-14,
            xtol: 1e-14,
            ..Default::default()
        };
        let root = broyden(f, &[0.5, 1.5], cfg).unwrap();
        assert_close(root[0], 1.0, 1e-6);
        assert_close(root[1], 1.0, 1e-6);
    }

    /// Three-variable linear system — sanity check that Broyden
    /// reduces to a single Newton step when F is linear (the rank-1
    /// update produces an exact J after one iteration).
    #[test]
    fn solves_linear_system_essentially_immediately() {
        // A·x = b → F(x) = A·x − b
        // A = [[2, 1, 0], [1, 3, 1], [0, 1, 2]], b = [3, 5, 3]
        // Solution: x = [1, 1, 1]
        let f = |v: &[f64]| {
            vec![
                2.0 * v[0] + v[1] - 3.0,
                v[0] + 3.0 * v[1] + v[2] - 5.0,
                v[1] + 2.0 * v[2] - 3.0,
            ]
        };
        let root = broyden(f, &[0.0, 0.0, 0.0], BroydenConfig::default()).unwrap();
        for &r in &root {
            assert_close(r, 1.0, 1e-7);
        }
    }

    /// Rosenbrock-style non-quadratic system:
    ///   10·(y − x²) = 0
    ///   (1 − x)    = 0
    /// Root: (1, 1). Harder than the polynomial case because the
    /// first equation is curved.
    #[test]
    fn solves_rosenbrock_style_system() {
        let f = |v: &[f64]| {
            let (x, y) = (v[0], v[1]);
            vec![10.0 * (y - x * x), 1.0 - x]
        };
        let root = broyden(f, &[-1.2, 1.0], BroydenConfig::default()).unwrap();
        assert_close(root[0], 1.0, 1e-7);
        assert_close(root[1], 1.0, 1e-7);
    }

    /// Refresh cadence sanity check — confirm convergence at every
    /// `refresh_every` setting from 0 (never refresh) to 10. Uses tight
    /// tolerances so we're actually measuring "Broyden found the root"
    /// not "Broyden's residual is below loose threshold".
    #[test]
    fn refresh_cadence_does_not_break_convergence() {
        let f = |v: &[f64]| {
            let (x, y) = (v[0], v[1]);
            vec![x * x + y * y - 2.0, x * y - 1.0]
        };
        for refresh_every in [0_usize, 1, 2, 5, 10] {
            let cfg = BroydenConfig {
                refresh_every,
                ftol: 1e-13,
                xtol: 1e-13,
                ..Default::default()
            };
            let root = broyden(f, &[0.5, 1.5], cfg).unwrap_or_else(|e| {
                panic!("refresh_every={refresh_every} failed: {e}");
            });
            assert_close(root[0], 1.0, 1e-5);
            assert_close(root[1], 1.0, 1e-5);
        }
    }

    /// Singular initial Jacobian: F(x, y) = (x + y, 2x + 2y). The
    /// FD Jacobian is rank-1 (proportional columns) so the very first
    /// LU solve fails. The error variant must be SingularJacobian, not
    /// silently NaN-propagated.
    #[test]
    fn detects_singular_jacobian() {
        let f = |v: &[f64]| vec![v[0] + v[1] - 1.0, 2.0 * (v[0] + v[1]) - 2.0];
        let err = broyden(f, &[0.0, 0.0], BroydenConfig::default()).unwrap_err();
        assert!(matches!(err, BroydenError::SingularJacobian { .. }));
    }

    /// Dimension mismatch is caught at the validation layer rather
    /// than corrupting linear-algebra calls deeper down.
    #[test]
    fn detects_dimension_mismatch() {
        // x0 has length 2 but F returns length 3 — caught on the
        // initial residual evaluation.
        let f = |_v: &[f64]| vec![1.0, 2.0, 3.0];
        let err = broyden(f, &[0.0, 0.0], BroydenConfig::default()).unwrap_err();
        assert!(matches!(err, BroydenError::DimensionMismatch { .. }));
    }

    /// NaN propagation — F returning a NaN must surface as
    /// NonFiniteEvaluation, not silently corrupt the iterate.
    #[test]
    fn detects_non_finite_evaluation() {
        let f = |_v: &[f64]| vec![f64::NAN, 0.0];
        let err = broyden(f, &[0.0, 0.0], BroydenConfig::default()).unwrap_err();
        assert!(matches!(err, BroydenError::NonFiniteEvaluation { .. }));
    }

    /// Hard problem with a tight iteration cap → NoConvergence
    /// surfaces, with last residual norms reported so the caller can
    /// diagnose.
    #[test]
    fn reports_no_convergence_with_residuals() {
        // Two-variable system that would normally converge in ~20
        // iterations — cap at 3 to force NoConvergence.
        let f = |v: &[f64]| {
            let (x, y) = (v[0], v[1]);
            vec![x.exp() + y - 2.0, x + y.exp() - 2.0]
        };
        let cfg = BroydenConfig {
            max_iter: 3,
            ..Default::default()
        };
        let err = broyden(f, &[2.0, 2.0], cfg).unwrap_err();
        match err {
            BroydenError::NoConvergence {
                max_iter,
                last_f_norm,
                last_dx_norm,
            } => {
                assert_eq!(max_iter, 3);
                assert!(last_f_norm > 0.0, "residual norm should be positive");
                assert!(last_dx_norm >= 0.0, "step norm should be non-negative");
            }
            other => panic!("expected NoConvergence, got {other:?}"),
        }
    }
}
