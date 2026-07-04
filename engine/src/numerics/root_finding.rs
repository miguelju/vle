//! Bracketed scalar root finders.
//!
//! Both algorithms here solve the same problem — find x in [a, b] such
//! that f(x) = 0, given that f changes sign across the interval — but
//! they trade off convergence speed vs. per-iteration cost:
//!
//! - [`brent`] is the recommended default. Combines bisection, secant,
//!   and inverse-quadratic interpolation to deliver super-linear
//!   convergence with the safety net of bisection. ~5-10 function
//!   evaluations for a typical engineering problem. Replaces the legacy
//!   Regula Falsi in `clsSatPressureSolver.cls`.
//! - [`illinois`] is a modified Regula Falsi with weight halving on
//!   stalled iterations. Lighter-weight than Brent (no IQI machinery),
//!   slightly slower, but predictable cost per iteration. Useful when
//!   f is expensive enough that you'd like to bound the per-iter work.
//!
//! Both return [`RootError::NoBracket`] if `f(a) * f(b) > 0` (no sign
//! change) — the caller is responsible for bracketing first. Neither
//! algorithm changes a's sign mid-iteration, so the returned root is
//! guaranteed to be inside the original interval.
//!
//! Reference: Brent, R. P., *"An algorithm with guaranteed convergence
//! for finding a zero of a function"* (1971); Dahlquist & Björck,
//! *Numerical Methods*, §6.4.

use thiserror::Error;

/// Errors returned by the root finders in this module.
#[derive(Debug, Error, PartialEq)]
pub enum RootError {
    /// `f(a)` and `f(b)` have the same sign — no guaranteed root in
    /// the interval. The caller must widen the bracket (or use an
    /// unbracketed Newton-style solver) before retrying.
    #[error("interval [{a}, {b}] does not bracket a root: f(a)={fa}, f(b)={fb}")]
    NoBracket { a: f64, b: f64, fa: f64, fb: f64 },
    /// The iteration limit was hit before the convergence criterion
    /// was met. The caller can raise `max_iter`, loosen tolerances, or
    /// switch to a different algorithm.
    #[error("did not converge in {max_iter} iterations (last residual {last_residual})")]
    NoConvergence { max_iter: usize, last_residual: f64 },
    /// A function evaluation returned NaN. Most often a domain error
    /// in `f` (e.g., taking `ln` of a negative number) — the caller
    /// should reshape `f` or narrow the bracket before retrying.
    #[error("f returned NaN at x = {x}")]
    NanEvaluation { x: f64 },
}

/// Brent's method for scalar root finding on a bracketed interval.
///
/// This is the recommended default for bracketed problems: super-linear
/// convergence (≈1.6 order on smooth f, sometimes near quadratic) with
/// bisection as a fallback so it never diverges. It's the algorithm
/// behind `scipy.optimize.brentq` and most "industrial-strength" zero
/// finders.
///
/// # Arguments
/// * `f` — Function f(x) to find a root of. Called once per iteration
///   plus twice for the initial bracket check.
/// * `a`, `b` — Bracket endpoints. Required: `f(a) * f(b) < 0`.
/// * `xtol` — Absolute tolerance on x. Convergence when the bracket
///   width drops below `xtol + 4·ε·|x|` (ε = `f64::EPSILON`).
/// * `max_iter` — Hard iteration cap. 100 is a generous default for
///   engineering problems.
///
/// # Returns
/// The root, somewhere in `[a, b]`, converged to within `xtol`.
pub fn brent<F>(mut f: F, a: f64, b: f64, xtol: f64, max_iter: usize) -> Result<f64, RootError>
where
    F: FnMut(f64) -> f64,
{
    let mut fa = f(a);
    let mut fb = f(b);
    if fa.is_nan() {
        return Err(RootError::NanEvaluation { x: a });
    }
    if fb.is_nan() {
        return Err(RootError::NanEvaluation { x: b });
    }
    if fa * fb > 0.0 {
        return Err(RootError::NoBracket { a, b, fa, fb });
    }

    // Brent's invariant: |f(b)| ≤ |f(a)| (so b is always the "best"
    // estimate). Swap if necessary.
    let (mut a, mut b) = (a, b);
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }

    // c is the contrapoint — the previous value of a, used for IQI.
    let mut c = a;
    let mut fc = fa;
    // d is two iterations back, used to detect repeated bisections that
    // force a fallback (the "five consecutive bisections" rule).
    let mut d = c;
    // `mflag` tracks whether the previous step was bisection.
    let mut mflag = true;

    for iter in 0..max_iter {
        // Convergence check on bracket width. Uses the standard form
        // tol = 2·ε·|b| + xtol/2; bracket closed when |b - a| < tol.
        let tol = 2.0 * f64::EPSILON * b.abs() + 0.5 * xtol;
        let mid = 0.5 * (a - b);
        if fb == 0.0 || mid.abs() < tol {
            return Ok(b);
        }

        // Try inverse-quadratic interpolation (if we have three distinct
        // function values) or secant (if we only have two). Bisection is
        // the fallback if the interpolated point fails any safeguard.
        let s: f64 = if fa != fc && fb != fc {
            // IQI: x_new = a·f(b)·f(c) / ((f(a)-f(b))·(f(a)-f(c))) + ...
            let denom_a = (fa - fb) * (fa - fc);
            let denom_b = (fb - fa) * (fb - fc);
            let denom_c = (fc - fa) * (fc - fb);
            a * fb * fc / denom_a + b * fa * fc / denom_b + c * fa * fb / denom_c
        } else {
            // Secant
            b - fb * (b - a) / (fb - fa)
        };

        // Safeguards — if `s` violates any of these, fall back to
        // bisection. This is what makes Brent's robust.
        let cond1 = {
            let lo = (3.0 * a + b) / 4.0;
            let (low, high) = if lo < b { (lo, b) } else { (b, lo) };
            s < low || s > high
        };
        let cond2 = mflag && (s - b).abs() >= (b - c).abs() / 2.0;
        let cond3 = !mflag && (s - b).abs() >= (c - d).abs() / 2.0;
        let cond4 = mflag && (b - c).abs() < tol;
        let cond5 = !mflag && (c - d).abs() < tol;

        let s = if cond1 || cond2 || cond3 || cond4 || cond5 {
            // Bisection
            mflag = true;
            0.5 * (a + b)
        } else {
            mflag = false;
            s
        };

        let fs = f(s);
        if fs.is_nan() {
            return Err(RootError::NanEvaluation { x: s });
        }

        // Shuffle: d ← c, c ← b. Then update b ← s if it tightens the
        // bracket on the same side as a, else a ← s.
        d = c;
        c = b;
        fc = fb;
        if fa * fs < 0.0 {
            // Sign change between a and s: new b = s.
            b = s;
            fb = fs;
        } else {
            // Sign change between s and b: new a = s.
            a = s;
            fa = fs;
        }
        // Re-establish |f(b)| ≤ |f(a)|.
        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }

        // Last-iteration check: return the best estimate.
        if iter + 1 == max_iter {
            return Err(RootError::NoConvergence {
                max_iter,
                last_residual: fb,
            });
        }
    }

    unreachable!("loop exits via convergence return or NoConvergence")
}

/// Illinois algorithm — Regula Falsi with weight halving on stalled
/// iterations.
///
/// Cheaper than [`brent`] per iteration (no IQI, no contrapoint
/// bookkeeping) and easier to reason about, but slower to converge on
/// well-conditioned problems. Useful when you want predictable
/// per-iteration cost or when the function is so cheap that the savings
/// outweigh the extra iterations.
///
/// The "Illinois" modification: when one endpoint stays fixed across two
/// iterations (the classic Regula Falsi stall mode), halve the f-value
/// at that endpoint to bias the next secant step toward the other side.
/// Recovers superlinear-ish convergence.
///
/// # Arguments
/// * `f` — Function f(x) to find a root of.
/// * `a`, `b` — Bracket endpoints. Required: `f(a) * f(b) < 0`.
/// * `xtol` — Absolute tolerance on x.
/// * `max_iter` — Hard iteration cap.
///
/// # Returns
/// The root in `[a, b]`, converged to within `xtol`.
pub fn illinois<F>(mut f: F, a: f64, b: f64, xtol: f64, max_iter: usize) -> Result<f64, RootError>
where
    F: FnMut(f64) -> f64,
{
    let mut fa = f(a);
    let mut fb = f(b);
    if fa.is_nan() {
        return Err(RootError::NanEvaluation { x: a });
    }
    if fb.is_nan() {
        return Err(RootError::NanEvaluation { x: b });
    }
    if fa * fb > 0.0 {
        return Err(RootError::NoBracket { a, b, fa, fb });
    }

    let (mut a, mut b) = (a, b);
    // Side counter: +1 each time the upper endpoint sticks, -1 for the
    // lower. When |side| ≥ 2 we apply the Illinois weight-halving.
    let mut side: i32 = 0;

    for iter in 0..max_iter {
        // Secant estimate: x = (a·fb − b·fa) / (fb − fa).
        let denom = fb - fa;
        // Defensive: a perfectly flat secant means f is locally
        // constant — fall back to bisection to make progress.
        let c = if denom == 0.0 {
            0.5 * (a + b)
        } else {
            (a * fb - b * fa) / denom
        };
        let fc = f(c);
        if fc.is_nan() {
            return Err(RootError::NanEvaluation { x: c });
        }
        if fc == 0.0 || (b - a).abs() < xtol {
            return Ok(c);
        }

        if fc * fb < 0.0 {
            // Sign change on the [c, b] side: new bracket = [c, b].
            a = c;
            fa = fc;
            if side == -1 {
                fb *= 0.5; // Illinois weight halving
            }
            side = -1;
        } else {
            // Sign change on the [a, c] side: new bracket = [a, c].
            b = c;
            fb = fc;
            if side == 1 {
                fa *= 0.5;
            }
            side = 1;
        }

        if iter + 1 == max_iter {
            return Err(RootError::NoConvergence {
                max_iter,
                last_residual: fc,
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

    // ---------- shared problems used by both algorithms ----------

    // f(x) = cos(x) − x. Root ≈ 0.7390851332151607 (Dottie number).
    fn cos_minus_x(x: f64) -> f64 {
        x.cos() - x
    }

    // f(x) = x³ − 2x − 5. Root ≈ 2.0945514815423265 (Wallis's polynomial).
    fn wallis(x: f64) -> f64 {
        x.powi(3) - 2.0 * x - 5.0
    }

    // ---------- Brent ----------

    #[test]
    fn brent_finds_dottie_number() {
        let r = brent(cos_minus_x, 0.0, 1.0, 1e-12, 100).unwrap();
        assert_close(r, 0.739_085_133_215_160_7, 1e-12);
    }

    #[test]
    fn brent_finds_wallis_root() {
        let r = brent(wallis, 2.0, 3.0, 1e-12, 100).unwrap();
        assert_close(r, 2.094_551_481_542_326_5, 1e-12);
    }

    #[test]
    fn brent_rejects_unbracketed_interval() {
        // f(x) = x² has no sign change on [1, 2] — both endpoints positive.
        let err = brent(|x: f64| x * x, 1.0, 2.0, 1e-9, 50).unwrap_err();
        assert!(matches!(err, RootError::NoBracket { .. }));
    }

    #[test]
    fn brent_handles_root_on_endpoint() {
        // Root exactly at a — Brent should detect immediately.
        let r = brent(|x: f64| x - 3.0, 3.0, 4.0, 1e-12, 100).unwrap();
        assert_close(r, 3.0, 1e-12);
    }

    #[test]
    fn brent_reports_nan_evaluation() {
        // ln on negative argument gives NaN; bracketing implies one
        // endpoint is negative so the very first eval trips it.
        let err = brent(f64::ln, -1.0, 1.0, 1e-9, 50).unwrap_err();
        assert!(matches!(err, RootError::NanEvaluation { .. }));
    }

    // ---------- Illinois ----------

    #[test]
    fn illinois_finds_dottie_number() {
        let r = illinois(cos_minus_x, 0.0, 1.0, 1e-12, 100).unwrap();
        assert_close(r, 0.739_085_133_215_160_7, 1e-9);
    }

    #[test]
    fn illinois_finds_wallis_root() {
        let r = illinois(wallis, 2.0, 3.0, 1e-12, 100).unwrap();
        assert_close(r, 2.094_551_481_542_326_5, 1e-9);
    }

    #[test]
    fn illinois_rejects_unbracketed_interval() {
        let err = illinois(|x: f64| x * x + 1.0, -1.0, 1.0, 1e-9, 50).unwrap_err();
        assert!(matches!(err, RootError::NoBracket { .. }));
    }

    #[test]
    fn illinois_handles_steep_function() {
        // f(x) = x¹⁵ + 1 — classic Regula Falsi stall: function very
        // small over most of the bracket, then suddenly steep at the
        // root. Plain Regula Falsi gets stuck on the flat side; the
        // Illinois weight halving recovers.
        let r = illinois(|x: f64| x.powi(15) + 1.0, -2.0, 0.5, 1e-9, 100).unwrap();
        assert_close(r, -1.0, 1e-6);
    }
}

// ─────────────────────────────────────────────────────────────────────
// Brent's method for one-dimensional MINIMIZATION (parabolic interpolation
// + golden-section fallback). Distinct from the root finders above: this
// finds a local minimum of a scalar function on a bracket, not a zero.
// Used for kij parameter regression (§B) — it replaces the legacy
// golden-section search with the same guaranteed convergence but the
// super-linear speed of parabolic interpolation near the optimum.
//
// Reference: Brent, R. P., *Algorithms for Minimization without
// Derivatives* (1973), Ch. 5; Press et al., *Numerical Recipes*, §10.2.
// ─────────────────────────────────────────────────────────────────────

/// Golden ratio complement `(3 − √5)/2` used by the golden-section fallback.
const CGOLD: f64 = 0.381_966_011_250_105_2;

/// Minimize a scalar function on the bracket `[a, b]` by Brent's method.
///
/// Returns `(x_min, f(x_min))`. The function should be (at least locally)
/// unimodal on `[a, b]`; the algorithm combines inverse-parabolic
/// interpolation (fast near a smooth minimum) with a golden-section
/// safeguard (guaranteed linear progress otherwise), so it always
/// converges to a local minimum inside the bracket.
///
/// # Arguments
/// * `f` — objective, evaluated as `FnMut(f64) -> f64`.
/// * `a`, `b` — the bracket ends (any order).
/// * `tol` — relative x-tolerance; convergence when the bracket half-width
///   falls below `tol·|x| + 1e-12`.
/// * `max_iter` — hard iteration cap.
///
/// # Errors
/// [`RootError::NoConvergence`] if the cap is hit before the tolerance.
pub fn brent_minimize<F>(
    mut f: F,
    a: f64,
    b: f64,
    tol: f64,
    max_iter: usize,
) -> Result<(f64, f64), RootError>
where
    F: FnMut(f64) -> f64,
{
    let (mut lo, mut hi) = if a < b { (a, b) } else { (b, a) };
    // x = best point so far; w = second best; v = previous w.
    let mut x = lo + CGOLD * (hi - lo);
    let (mut w, mut v) = (x, x);
    let mut fx = f(x);
    let (mut fw, mut fv) = (fx, fx);
    let mut d = 0.0_f64;
    let mut e = 0.0_f64; // the step before last

    for iter in 0..max_iter {
        let xm = 0.5 * (lo + hi);
        let tol1 = tol * x.abs() + 1e-12;
        let tol2 = 2.0 * tol1;
        if (x - xm).abs() <= tol2 - 0.5 * (hi - lo) {
            return Ok((x, fx));
        }
        let mut use_golden = true;
        if e.abs() > tol1 {
            // Try inverse-parabolic interpolation through (x, w, v).
            let r = (x - w) * (fx - fv);
            let q0 = (x - v) * (fx - fw);
            let mut p = (x - v) * q0 - (x - w) * r;
            let mut q = 2.0 * (q0 - r);
            if q > 0.0 {
                p = -p;
            }
            q = q.abs();
            let e_temp = e;
            e = d;
            // Accept the parabolic step only if it stays in the bracket and
            // is smaller than half the step-before-last.
            if p.abs() < (0.5 * q * e_temp).abs() && p > q * (lo - x) && p < q * (hi - x) {
                d = p / q;
                let u = x + d;
                if (u - lo) < tol2 || (hi - u) < tol2 {
                    d = if xm - x >= 0.0 { tol1 } else { -tol1 };
                }
                use_golden = false;
            }
        }
        if use_golden {
            e = if x >= xm { lo - x } else { hi - x };
            d = CGOLD * e;
        }
        // Take a step at least tol1 away from x.
        let u = if d.abs() >= tol1 {
            x + d
        } else if d >= 0.0 {
            x + tol1
        } else {
            x - tol1
        };
        let fu = f(u);
        // Bookkeeping: update the bracket and the best/second-best points.
        if fu <= fx {
            if u >= x {
                lo = x;
            } else {
                hi = x;
            }
            v = w;
            fv = fw;
            w = x;
            fw = fx;
            x = u;
            fx = fu;
        } else {
            if u < x {
                lo = u;
            } else {
                hi = u;
            }
            if fu <= fw || w == x {
                v = w;
                fv = fw;
                w = u;
                fw = fu;
            } else if fu <= fv || v == x || v == w {
                v = u;
                fv = fu;
            }
        }
        if iter + 1 == max_iter {
            return Err(RootError::NoConvergence {
                max_iter,
                last_residual: hi - lo,
            });
        }
    }
    Ok((x, fx))
}

#[cfg(test)]
mod minimize_tests {
    use super::*;

    #[test]
    fn finds_parabola_minimum() {
        // (x − 3)² + 1 → min at x = 3, f = 1.
        let (xm, fm) =
            brent_minimize(|x| (x - 3.0).powi(2) + 1.0, -10.0, 10.0, 1e-10, 100).unwrap();
        assert!((xm - 3.0).abs() < 1e-6, "x_min={xm}");
        assert!((fm - 1.0).abs() < 1e-9, "f_min={fm}");
    }

    #[test]
    fn finds_minimum_of_quartic() {
        // A smooth quartic with a single interior minimum on [0, 2].
        let f = |x: f64| x.powi(4) - 4.0 * x.powi(2) + x; // min near x ≈ 1.347
        let (xm, _) = brent_minimize(f, 0.5, 2.0, 1e-10, 100).unwrap();
        // Derivative 4x³ − 8x + 1 = 0 has its interior minimum at x ≈ 1.34700.
        assert!((xm - 1.34700).abs() < 1e-3, "x_min={xm}");
    }
}
