//! Small reusable numerical helpers shared across the engine.
//!
//! Nothing here is novel — these are the routines that appear over and
//! over in cubic-EOS, flash, and root-finding code. Extracting them once
//! keeps the rest of the engine readable and means there's exactly one
//! place to fix a bug (or change a tolerance).
//!
//! Ported / generalized from the inline VB6 + Pascal equivalents:
//! - `SumFrac` — Pascal `TERMOIV.PAS` Rachford-Rice helpers.
//! - `Norm` — VB6 `clsLVE.cls` flash convergence test.
//! - `is_near_zero` — used throughout the cubic solver and elsewhere.

/// Default tolerance for comparing a value against zero. Loose enough to
/// catch floating-point noise from a few arithmetic operations, tight
/// enough that legitimate small values aren't accidentally collapsed.
/// Callers that need a different scale (e.g. dimensioned values where
/// "near zero" depends on the unit) should use [`is_near_zero_eps`]
/// directly.
pub const DEFAULT_NEAR_ZERO_EPS: f64 = 1e-12;

/// True iff `|x|` is below the default near-zero epsilon.
///
/// # Arguments
/// * `x` — Value to test (dimensionless or scaled to a known reference).
///
/// # Returns
/// `true` if `x` is close enough to zero that it should be treated as
/// such for branching purposes (e.g. cubic-discriminant-is-zero tests).
#[inline]
pub fn is_near_zero(x: f64) -> bool {
    is_near_zero_eps(x, DEFAULT_NEAR_ZERO_EPS)
}

/// True iff `|x|` is below the caller-supplied epsilon. NaN-safe: any
/// NaN input returns `false`.
#[inline]
pub fn is_near_zero_eps(x: f64, eps: f64) -> bool {
    x.is_finite() && x.abs() <= eps
}

/// Sum-of-fractions residual `1 − Σxᵢ`.
///
/// Used in flash and bubble/dew calculations as a constraint check: the
/// liquid (or vapor) mole fractions of a mixture must sum to 1. Returns
/// the **signed** residual so callers can drive it to zero with a
/// Newton/Halley step.
///
/// # Arguments
/// * `xs` — Slice of mole fractions (or any quantity that must sum to 1).
///
/// # Returns
/// `1.0 − Σxᵢ` (dimensionless).
#[inline]
pub fn sum_frac_residual(xs: &[f64]) -> f64 {
    1.0 - xs.iter().sum::<f64>()
}

/// L₁ norm of a vector: `Σ|xᵢ|`. Cheap, used as a convergence test on
/// composition vectors where the absolute sum captures total drift.
#[inline]
pub fn norm_l1(xs: &[f64]) -> f64 {
    xs.iter().map(|x| x.abs()).sum()
}

/// L₂ (Euclidean) norm: `√(Σxᵢ²)`. The "default" vector norm; used for
/// residual vectors in multi-variable Newton-Raphson and Broyden.
#[inline]
pub fn norm_l2(xs: &[f64]) -> f64 {
    xs.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// L_∞ (max) norm: `maxᵢ |xᵢ|`. Tightest convergence test — every
/// component must be within tolerance, not just the average. Returns
/// `0.0` for an empty slice (convention matches L₁/L₂).
#[inline]
pub fn norm_linf(xs: &[f64]) -> f64 {
    xs.iter().map(|x| x.abs()).fold(0.0_f64, f64::max)
}

/// Standard scalar convergence check: `|new − old| ≤ atol + rtol·|new|`.
///
/// This is the combined-tolerance form used in SciPy, GSL, and most
/// modern iterative solvers. It degrades gracefully across the dynamic
/// range:
/// - When `new` is small (≈ 0), the **absolute** tolerance `atol`
///   dominates and avoids dividing by ~0.
/// - When `new` is large, the **relative** tolerance `rtol` dominates
///   and gives the same number of significant figures everywhere.
///
/// # Arguments
/// * `new` — Latest iterate.
/// * `old` — Previous iterate.
/// * `atol` — Absolute tolerance (>= 0).
/// * `rtol` — Relative tolerance (>= 0).
///
/// # Returns
/// `true` if the change between iterates is below tolerance.
#[inline]
pub fn converged_scalar(new: f64, old: f64, atol: f64, rtol: f64) -> bool {
    (new - old).abs() <= atol + rtol * new.abs()
}

/// Parabolic interpolation through three (x, y) points.
///
/// Given three function values, returns the x-coordinate of the vertex
/// of the parabola through them. Useful as a refinement step in
/// bracketed root finders (Brent uses inverse quadratic interpolation,
/// which is the same idea expressed differently) and in line searches.
///
/// Returns `None` when the three points are collinear (denominator is
/// zero) — caller should fall back to bisection or secant.
///
/// # Arguments
/// * `(x0, y0)`, `(x1, y1)`, `(x2, y2)` — three points, x-coordinates
///   must be distinct.
///
/// # Returns
/// `Some(x_vertex)` of the fitted parabola, or `None` if degenerate.
pub fn parabolic_vertex(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
) -> Option<f64> {
    let (x0, y0) = p0;
    let (x1, y1) = p1;
    let (x2, y2) = p2;

    let d01 = (y0 - y1) * (x1 - x2);
    let d21 = (y2 - y1) * (x1 - x0);
    let denom = d01 - d21;
    if is_near_zero(denom) {
        return None;
    }
    let num = (x1 - x2) * d01 - (x1 - x0) * d21;
    Some(x1 - 0.5 * num / denom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_zero_basic_and_nan() {
        assert!(is_near_zero(0.0));
        assert!(is_near_zero(1e-15));
        assert!(!is_near_zero(1e-9));
        assert!(!is_near_zero(f64::NAN));
        assert!(!is_near_zero(f64::INFINITY));
    }

    #[test]
    fn sum_frac_residual_works() {
        assert!((sum_frac_residual(&[0.4, 0.6]) - 0.0).abs() < 1e-15);
        assert!((sum_frac_residual(&[0.3, 0.3, 0.3]) - 0.1).abs() < 1e-15);
        assert_eq!(sum_frac_residual(&[]), 1.0);
    }

    #[test]
    fn norms_basic() {
        let v = [3.0, -4.0];
        assert!((norm_l1(&v) - 7.0).abs() < 1e-15);
        assert!((norm_l2(&v) - 5.0).abs() < 1e-15); // classic 3-4-5
        assert!((norm_linf(&v) - 4.0).abs() < 1e-15);
        // Empty slice degenerates to 0 for all norms
        assert_eq!(norm_l1(&[]), 0.0);
        assert_eq!(norm_l2(&[]), 0.0);
        assert_eq!(norm_linf(&[]), 0.0);
    }

    #[test]
    fn converged_scalar_handles_both_regimes() {
        // Absolute regime: small magnitudes, atol dominates.
        assert!(converged_scalar(1e-10, 0.0, 1e-9, 1e-6));
        assert!(!converged_scalar(1e-8, 0.0, 1e-9, 1e-6));
        // Relative regime: large magnitudes, rtol dominates.
        assert!(converged_scalar(1.0 + 1e-7, 1.0, 1e-12, 1e-6));
        assert!(!converged_scalar(1.0 + 1e-5, 1.0, 1e-12, 1e-6));
    }

    #[test]
    fn parabolic_vertex_finds_known_minimum() {
        // y = (x - 2)^2 + 1 — vertex at x = 2.
        let f = |x: f64| (x - 2.0).powi(2) + 1.0;
        let v = parabolic_vertex(
            (0.0, f(0.0)),
            (1.5, f(1.5)),
            (5.0, f(5.0)),
        )
        .expect("non-degenerate parabola");
        assert!((v - 2.0).abs() < 1e-12, "vertex = {v}, expected 2.0");
    }

    #[test]
    fn parabolic_vertex_degenerate_returns_none() {
        // Three collinear points -> no parabola.
        assert!(parabolic_vertex((0.0, 0.0), (1.0, 1.0), (2.0, 2.0)).is_none());
    }
}
