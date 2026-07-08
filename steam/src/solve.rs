//! Small self-contained bracketing root finder (Brent's method).
//!
//! The crate is dependency-free, so it cannot reuse `vle-thermo`'s
//! `numerics::brent`. Both the region-3 density solve and the region-2/5
//! `T(p,h)` / `T(p,s)` inversions are one-dimensional monotonic-ish root
//! problems, so a compact Brent with a scanning bracketer covers every case.

use crate::SteamError;

/// Scan `[lo, hi]` in `n` steps for the first sub-interval where `f` changes
/// sign, returning that bracket. `None` if no sign change is found.
pub(crate) fn bracket(f: &impl Fn(f64) -> f64, lo: f64, hi: f64, n: usize) -> Option<(f64, f64)> {
    let mut prev_x = lo;
    let mut prev_f = f(lo);
    for k in 1..=n {
        let x = lo + (hi - lo) * (k as f64) / (n as f64);
        let fx = f(x);
        if prev_f * fx <= 0.0 && prev_f.is_finite() && fx.is_finite() {
            return Some((prev_x, x));
        }
        prev_x = x;
        prev_f = fx;
    }
    None
}

/// Brent's method on a bracketed root of `f` in `[a, b]` (must change sign).
///
/// Converges to ~1e-11 relative in the abscissa; returns
/// [`SteamError::NoConvergence`] if the endpoints do not bracket a root.
pub(crate) fn brent(
    f: &impl Fn(f64) -> f64,
    mut a: f64,
    mut b: f64,
    what: &'static str,
) -> Result<f64, SteamError> {
    let mut fa = f(a);
    let mut fb = f(b);
    if fa * fb > 0.0 {
        return Err(SteamError::NoConvergence(what));
    }
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    let mut c = a;
    let mut fc = fa;
    let mut d = a;
    let mut mflag = true;
    for _ in 0..200 {
        if fb == 0.0 || (b - a).abs() < 1e-12 * b.abs().max(1.0) {
            return Ok(b);
        }
        let mut s = if fa != fc && fb != fc {
            a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb))
        } else {
            b - fb * (b - a) / (fb - fa)
        };

        let lo = (3.0 * a + b) / 4.0;
        let bisect = !(s > lo.min(b) && s < lo.max(b))
            || (mflag && (s - b).abs() >= (b - c).abs() / 2.0)
            || (!mflag && (s - b).abs() >= (c - d).abs() / 2.0)
            || (mflag && (b - c).abs() < 1e-12)
            || (!mflag && (c - d).abs() < 1e-12);
        if bisect {
            s = (a + b) / 2.0;
            mflag = true;
        } else {
            mflag = false;
        }

        let fs = f(s);
        d = c;
        c = b;
        fc = fb;
        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }
        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }
    Ok(b)
}
