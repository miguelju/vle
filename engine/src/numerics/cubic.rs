//! Cardano cubic-equation solver.
//!
//! Solves `a·x³ + b·x² + c·x + d = 0` over the reals and returns every
//! real root in ascending order. The solver applies the standard
//! depressed-cubic substitution `x = y − b/(3a)` and then picks one of
//! three branches based on the discriminant:
//!
//! - **Δ > 0** — one real root (and two complex conjugates we discard).
//!   Computed via Cardano's formula with sign-preserving `cbrt`.
//! - **Δ < 0** — three distinct real roots. Computed via the
//!   trigonometric form, which is numerically stable when there *are*
//!   three real roots.
//! - **Δ ≈ 0** — repeated roots. Caught via [`utils::is_near_zero`]
//!   rather than `== 0.0` so floating-point noise doesn't push us into
//!   the wrong branch right at the boundary. This is the (12) Poling &
//!   Prausnitz robustness recommendation for cubic-EOS work.
//!
//! Ported from `legacy/vb6/McommonFunctions.bas:324` (the inline Cardano
//! in the original codebase), with the discriminant test tightened per
//! Reference (12): Poling, Prausnitz & O'Connell, *"The Properties of
//! Gases and Liquids"* (5th ed.), §4-6.
//!
//! ## Why this matters for VLE
//!
//! Cubic equations of state (Peng-Robinson, RKS, Schmidt-Wenzel, …) are
//! literally cubics in the compressibility factor Z (or molar volume V).
//! At a given (T, P, composition) the EOS yields one or three real
//! roots: one root corresponds to the vapor phase (largest Z) and one to
//! the liquid (smallest Z); a middle root, if present, is
//! thermodynamically unstable. This solver is the workhorse that turns
//! the EOS into actual numbers. The (12) Poling robustness keeps it
//! stable at the critical point, where the three roots merge.

use thiserror::Error;

use crate::numerics::utils::{is_near_zero, is_near_zero_eps};

/// Errors returned by [`solve_real`].
#[derive(Debug, Error, PartialEq)]
pub enum CubicError {
    /// The leading coefficient `a` is (near-)zero — the equation isn't
    /// actually cubic. The caller should fall back to a quadratic
    /// solver. We refuse rather than silently divide-by-zero.
    #[error("leading coefficient is near zero ({0}); not a cubic equation")]
    NotCubic(f64),
    /// A coefficient was not finite (NaN, ±∞). Almost always a bug in
    /// upstream code that should be diagnosed at the call site.
    #[error("non-finite coefficient: a={a}, b={b}, c={c}, d={d}")]
    NotFinite { a: f64, b: f64, c: f64, d: f64 },
}

/// All real roots of `a·x³ + b·x² + c·x + d = 0`, in **ascending order**.
///
/// # Arguments
/// * `a`, `b`, `c`, `d` — Coefficients of the cubic in standard form.
///   `a` must be far enough from zero to be safely inverted (use
///   [`is_near_zero`] for the boundary).
///
/// # Returns
/// * `Ok(vec)` containing 1, 2, or 3 real roots:
///   - 1 root: discriminant > 0, only one real solution.
///   - 2 roots: a double root and a single root (discriminant ≈ 0).
///   - 3 roots: three distinct real solutions (discriminant < 0).
/// * `Err(CubicError::NotCubic)` if `a` is near zero.
/// * `Err(CubicError::NotFinite)` if any coefficient is NaN or ±∞.
///
/// Duplicates are collapsed via [`f64::partial_cmp`] + adjacent dedup,
/// so a triple root returns a 1-element vector.
pub fn solve_real(a: f64, b: f64, c: f64, d: f64) -> Result<Vec<f64>, CubicError> {
    if !(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()) {
        return Err(CubicError::NotFinite { a, b, c, d });
    }
    if is_near_zero(a) {
        return Err(CubicError::NotCubic(a));
    }

    // Normalize to monic form: x³ + b1·x² + c1·x + d1 = 0
    let b1 = b / a;
    let c1 = c / a;
    let d1 = d / a;

    // Depressed cubic: substitute x = y − b1/3, giving y³ + p·y + q = 0
    // with p and q below. The substitution is what lets us solve in
    // closed form; we shift back at the end.
    let shift = b1 / 3.0;
    let p = c1 - b1 * b1 / 3.0;
    let q = 2.0 * b1.powi(3) / 27.0 - b1 * c1 / 3.0 + d1;

    // Cardano discriminant: Δ = (q/2)² + (p/3)³
    //   Δ > 0  → one real root + two complex conjugates
    //   Δ < 0  → three distinct real roots (use trigonometric form)
    //   Δ = 0  → repeated roots
    let half_q = q / 2.0;
    let third_p = p / 3.0;
    let delta = half_q * half_q + third_p * third_p * third_p;

    // (12) Poling & Prausnitz robustness — at the critical point of a
    // cubic EOS the discriminant ≈ 0 from rounding even when the
    // mathematical Δ is exactly zero. Use a tolerance scaled to the
    // problem size so we route through the `repeated_roots` branch
    // instead of getting tripped up by tiny negative deltas slipping
    // into the trigonometric branch (which would then take √ of a
    // near-zero negative).
    let scale = (half_q * half_q).max(third_p.abs().powi(3)).max(1.0);
    let delta_eps = 1e-14 * scale;

    let mut ys: Vec<f64> = if is_near_zero_eps(delta, delta_eps) {
        repeated_roots(p, q)
    } else if delta > 0.0 {
        // One real root: y = ∛(−q/2 + √Δ) + ∛(−q/2 − √Δ)
        let sqrt_delta = delta.sqrt();
        let u = (-half_q + sqrt_delta).cbrt();
        let v = (-half_q - sqrt_delta).cbrt();
        vec![u + v]
    } else {
        // Three real roots via trigonometric form.
        //   y_k = 2·√(−p/3) · cos((φ − 2πk)/3),  k = 0, 1, 2
        // where φ = acos(−q/2 / √(−(p/3)³)).
        let neg_third_p = -third_p; // p < 0 here, so this is > 0
        let r = (-third_p.powi(3)).sqrt();
        // Clamp arccos argument to [-1, 1] — floating-point noise near
        // the boundary (a triple root has |arg| → 1) can put it
        // marginally outside the domain and produce NaN.
        let arg = (-half_q / r).clamp(-1.0, 1.0);
        let phi = arg.acos();
        let amp = 2.0 * neg_third_p.sqrt();
        let two_pi = 2.0 * std::f64::consts::PI;
        vec![
            amp * (phi / 3.0).cos(),
            amp * ((phi - two_pi) / 3.0).cos(),
            amp * ((phi + two_pi) / 3.0).cos(),
        ]
    };

    // Shift back: x = y − b1/3
    for y in ys.iter_mut() {
        *y -= shift;
    }

    // Sort ascending and collapse duplicates that arose from the
    // repeated-roots branch (or from floating-point noise putting two
    // computed roots within an ULP of each other).
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    dedup_close(&mut ys, 1e-12);
    Ok(ys)
}

/// Handle the discriminant-≈-0 case. Returns the depressed-cubic roots
/// (still in `y`-space; caller shifts back).
///
/// Three sub-cases:
/// - `p ≈ 0` and `q ≈ 0`: triple root at 0 → returns `[0.0]`.
/// - `p ≈ 0` and `q ≠ 0`: y³ = −q → single real cube root.
/// - General: double root at `y = −q/(2·(−p/3))`, single root at the
///   complementary location.
fn repeated_roots(p: f64, q: f64) -> Vec<f64> {
    if is_near_zero(p) {
        if is_near_zero(q) {
            vec![0.0]
        } else {
            vec![(-q).cbrt()]
        }
    } else {
        // From the depressed cubic with Δ = 0:
        //   single root  y_s = 3·q / p      (multiplicity 1)
        //   double root  y_d = −3·q / (2·p) (multiplicity 2)
        let y_double = -3.0 * q / (2.0 * p);
        let y_single = 3.0 * q / p;
        vec![y_double, y_single]
    }
}

/// In-place collapse of values within `eps` of each other (sorted input).
fn dedup_close(vs: &mut Vec<f64>, eps: f64) {
    vs.dedup_by(|a, b| (*a - *b).abs() <= eps);
}

/// Smallest real root — typical use: liquid molar volume / liquid Z.
///
/// # Returns
/// `Some(min)` when `roots` is non-empty, `None` otherwise.
#[inline]
pub fn select_smallest(roots: &[f64]) -> Option<f64> {
    roots.first().copied()
}

/// Largest real root — typical use: vapor molar volume / vapor Z.
///
/// # Returns
/// `Some(max)` when `roots` is non-empty, `None` otherwise.
#[inline]
pub fn select_largest(roots: &[f64]) -> Option<f64> {
    roots.last().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience: evaluate the cubic at `x` to check residual.
    fn cubic_eval(a: f64, b: f64, c: f64, d: f64, x: f64) -> f64 {
        ((a * x + b) * x + c) * x + d
    }

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() < tol,
            "expected {expected}, got {actual} (diff {})",
            (actual - expected).abs()
        );
    }

    #[test]
    fn rejects_non_cubic() {
        let r = solve_real(0.0, 1.0, 2.0, 3.0).unwrap_err();
        assert!(matches!(r, CubicError::NotCubic(_)));
    }

    #[test]
    fn rejects_nan_coefficient() {
        let r = solve_real(1.0, f64::NAN, 0.0, 0.0).unwrap_err();
        assert!(matches!(r, CubicError::NotFinite { .. }));
    }

    #[test]
    fn one_real_root_via_cardano() {
        // x³ + x + 1 = 0 — discriminant > 0, exactly one real root.
        // Numerical reference value (Wolfram Alpha): x ≈ −0.6823278038...
        let roots = solve_real(1.0, 0.0, 1.0, 1.0).unwrap();
        assert_eq!(roots.len(), 1);
        assert_close(roots[0], -0.682_327_803_828_019_3, 1e-12);
        // Residual sanity check
        assert!(cubic_eval(1.0, 0.0, 1.0, 1.0, roots[0]).abs() < 1e-10);
    }

    #[test]
    fn three_real_roots_via_trigonometric_form() {
        // (x − 1)(x − 2)(x − 3) = x³ − 6x² + 11x − 6
        let roots = solve_real(1.0, -6.0, 11.0, -6.0).unwrap();
        assert_eq!(roots.len(), 3);
        assert_close(roots[0], 1.0, 1e-10);
        assert_close(roots[1], 2.0, 1e-10);
        assert_close(roots[2], 3.0, 1e-10);
    }

    #[test]
    fn double_plus_single_root() {
        // (x − 2)²·(x + 1) = x³ − 3x² + 4
        // Roots: 2 (double), −1 (single). After dedup the double appears once,
        // so the returned vector has 2 entries.
        let roots = solve_real(1.0, -3.0, 0.0, 4.0).unwrap();
        assert_eq!(roots.len(), 2);
        assert_close(roots[0], -1.0, 1e-9);
        assert_close(roots[1], 2.0, 1e-9);
    }

    #[test]
    fn triple_root() {
        // (x − 5)³ = x³ − 15x² + 75x − 125
        let roots = solve_real(1.0, -15.0, 75.0, -125.0).unwrap();
        assert_eq!(roots.len(), 1, "triple root collapses after dedup");
        assert_close(roots[0], 5.0, 1e-6);
    }

    #[test]
    fn vle_z_factor_style_cubic() {
        // Toy PR-like cubic in Z designed to mimic a three-root EOS regime
        // (liquid / middle / vapor). Constructed from known roots so we
        // can check exact-arithmetic correctness:
        //   (Z − 0.10)(Z − 0.15)(Z − 0.80)
        //     = Z³ − 1.05·Z² + 0.215·Z − 0.012
        let roots = solve_real(1.0, -1.05, 0.215, -0.012).unwrap();
        assert_eq!(roots.len(), 3);
        assert_close(roots[0], 0.10, 1e-10);
        assert_close(roots[1], 0.15, 1e-10);
        assert_close(roots[2], 0.80, 1e-10);
        // Selectors pick the right extreme — liquid Z and vapor Z.
        assert_close(select_smallest(&roots).unwrap(), 0.10, 1e-10);
        assert_close(select_largest(&roots).unwrap(), 0.80, 1e-10);
    }

    #[test]
    fn near_degenerate_discriminant_does_not_panic() {
        // Δ very close to zero — the (12) Poling robustness routes us
        // through repeated_roots() instead of trying to take √(tiny
        // negative) or producing NaN from arccos overflow. Construct a
        // case where Δ is mathematically zero (triple root at 0):
        //   x³ + 1e-16·x² = 0 (with rounding, Δ ≈ 0).
        let roots = solve_real(1.0, 1e-16, 0.0, 0.0).unwrap();
        assert!(!roots.is_empty());
        for r in &roots {
            assert!(r.is_finite(), "root must be finite, got {r}");
        }
    }
}
