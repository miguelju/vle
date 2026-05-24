//! Halley's method — cubic-convergence scalar root finder.
//!
//! Where Newton-Raphson uses `f` and `f'` to iterate
//!     x ← x − f / f'
//! Halley adds `f''` for a tighter update:
//!     x ← x − 2·f·f' / (2·(f')² − f·f'')
//! and gets **cubic** convergence (≈ tripling the number of correct
//! digits per iteration on smooth f) at the cost of one extra
//! derivative evaluation per step.
//!
//! Used in M9's Rachford-Rice solver, where the residual equation is a
//! sum of fractions whose first and second derivatives are essentially
//! free to compute alongside f itself. For the cost of bundling them
//! into one callback we save 2-3× the iteration count compared to
//! Newton-Raphson on the same problem.
//!
//! Unlike the bracketed solvers in [`root_finding`](super::root_finding),
//! Halley is **not safeguarded** — it can diverge if started far from a
//! root or if f' is near zero. Callers should bracket first if there's
//! any doubt, or use Brent.
//!
//! Reference: Halley, Edmund (1694), *"A new, exact, and easy method of
//! finding the roots of any equations generally, and that without any
//! previous reduction"*. Restated in modern form in any numerical-
//! analysis text; see e.g. Press et al., *Numerical Recipes*, §9.4.

use thiserror::Error;

use crate::numerics::utils::is_near_zero;

/// Errors returned by [`halley`].
#[derive(Debug, Error, PartialEq)]
pub enum HalleyError {
    /// The iteration limit was hit without meeting the convergence
    /// criterion. The caller should either widen tolerances, raise
    /// `max_iter`, switch to a bracketed solver, or pick a better
    /// initial guess.
    #[error("did not converge in {max_iter} iterations (last residual {last_residual})")]
    NoConvergence { max_iter: usize, last_residual: f64 },
    /// The denominator `2·(f')² − f·f''` evaluated to (near-)zero,
    /// which would produce a divide-by-zero step. Often means the
    /// initial guess sits at an inflection point of `f`; perturb the
    /// guess slightly or switch algorithms.
    #[error("Halley denominator is near zero at x = {x}; iteration cannot proceed")]
    SingularStep { x: f64 },
    /// `f`, `f'`, or `f''` returned NaN. Usually a domain error in the
    /// callback (e.g., taking `ln` of a negative value).
    #[error("non-finite evaluation at x = {x}")]
    NonFiniteEvaluation { x: f64 },
}

/// Halley's method.
///
/// # Arguments
/// * `f_and_derivs` — Callback returning the triple `(f, f', f'')` at
///   the current iterate. Bundled into one function because callers
///   typically share intermediate computations between the three.
/// * `x0` — Initial guess. Halley is not safeguarded — start near the
///   root if possible.
/// * `xtol` — Absolute tolerance on the iterate. Convergence when
///   `|x_new − x_old| ≤ xtol + |x_new|·ε`.
/// * `max_iter` — Hard iteration cap. 50 is generous for cubic
///   convergence; if you need more you probably have the wrong solver.
///
/// # Returns
/// The converged root.
pub fn halley<F>(
    mut f_and_derivs: F,
    x0: f64,
    xtol: f64,
    max_iter: usize,
) -> Result<f64, HalleyError>
where
    F: FnMut(f64) -> (f64, f64, f64),
{
    let mut x = x0;

    for iter in 0..max_iter {
        let (f, df, ddf) = f_and_derivs(x);
        if !(f.is_finite() && df.is_finite() && ddf.is_finite()) {
            return Err(HalleyError::NonFiniteEvaluation { x });
        }
        if f == 0.0 {
            return Ok(x);
        }

        // Halley's update: Δx = 2·f·f' / (2·(f')² − f·f'').
        // Equivalent to Newton's −f/f' multiplied by 1 / (1 − f·f''/(2(f')²)).
        let denom = 2.0 * df * df - f * ddf;
        if is_near_zero(denom) {
            return Err(HalleyError::SingularStep { x });
        }
        let delta = 2.0 * f * df / denom;
        let x_new = x - delta;

        if delta.abs() <= xtol + x_new.abs() * f64::EPSILON {
            return Ok(x_new);
        }

        x = x_new;

        if iter + 1 == max_iter {
            return Err(HalleyError::NoConvergence {
                max_iter,
                last_residual: f,
            });
        }
    }

    unreachable!("loop exits via convergence return or NoConvergence")
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

    #[test]
    fn finds_sqrt_via_halley() {
        // x² − 2 = 0 → x = √2.
        let r = halley(|x: f64| (x * x - 2.0, 2.0 * x, 2.0), 1.0, 1e-15, 50).unwrap();
        assert_close(r, std::f64::consts::SQRT_2, 1e-15);
    }

    #[test]
    fn finds_dottie_number() {
        // cos(x) − x = 0 → Dottie number ≈ 0.7390851332151607.
        // f'  = −sin(x) − 1
        // f'' = −cos(x)
        let r = halley(
            |x: f64| (x.cos() - x, -x.sin() - 1.0, -x.cos()),
            0.5,
            1e-15,
            50,
        )
        .unwrap();
        assert_close(r, 0.739_085_133_215_160_7, 1e-15);
    }

    #[test]
    fn cubic_convergence_beats_newton_iteration_count() {
        // x² − 612 = 0 starting from x = 10. Halley's reaches f64
        // precision in just a few iterations — count them and confirm.
        let mut iters = 0;
        let target = 612.0_f64.sqrt();
        let _r = halley(
            |x: f64| {
                iters += 1;
                (x * x - 612.0, 2.0 * x, 2.0)
            },
            10.0,
            1e-15,
            50,
        )
        .unwrap();
        // Halley should land in ≤ 5 iterations from x=10 (Newton typically
        // needs 6-7 on this problem). The exact count is implementation-
        // dependent; we mostly care that it didn't take dozens.
        assert!(
            iters <= 6,
            "Halley took {iters} iterations on √612 (expected ≤ 6); approx target {target}"
        );
    }

    #[test]
    fn detects_singular_step() {
        // Construct values so the Halley denominator 2·(f')² − f·f'' is
        // exactly zero while f itself is non-zero (the f==0 short-circuit
        // would otherwise return early as "found a root"):
        //   f = 2, f' = 1, f'' = 2  →  2·1 − 2·2 = -2  ✗ not zero
        //   f = 2, f' = 1, f'' = 1  →  2·1 − 2·1 = 0   ✓ singular
        // The triple is constant in x, so the iteration immediately hits
        // SingularStep on the very first step.
        let err = halley(|_x: f64| (2.0, 1.0, 1.0), 0.5, 1e-12, 50).unwrap_err();
        assert!(matches!(err, HalleyError::SingularStep { .. }));
    }

    #[test]
    fn reports_non_finite_evaluation() {
        let err = halley(|_x: f64| (f64::NAN, 1.0, 1.0), 0.0, 1e-12, 50).unwrap_err();
        assert!(matches!(err, HalleyError::NonFiniteEvaluation { .. }));
    }
}
