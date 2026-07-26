//! Power-table evaluation of the IF97 series.
//!
//! Every IF97 fundamental equation is a sum of terms `nᵢ · x^Iᵢ · y^Jᵢ`, and
//! every property needs not just the term itself but its first and second
//! derivatives in **both** variables — six sums sharing the same term. Written
//! literally, that is what the region files used to do:
//!
//! ```text
//! d.g   += n *              x.powi(i)   * y.powi(j);
//! d.gp  += n * fi         * x.powi(i-1) * y.powi(j);
//! d.gpp += n * fi*(fi-1.) * x.powi(i-2) * y.powi(j);
//! d.gt  += n *              x.powi(i)   * fj * y.powi(j-1);
//! …
//! ```
//!
//! — **twelve** `powi` calls per term, where `x^I`, `x^(I−1)` and `x^(I−2)` are
//! three independent multiply chains for the same base. Region 2's `J` runs to
//! 58, so `y.powi(j)` alone is ~6 multiplications and the series pays for it
//! three times over on every term.
//!
//! [`powers`] instead builds the whole contiguous window of integer powers
//! once, by repeated multiplication, and the term loop then reads `x^(I−k)`
//! out of an array. For region 2's residual that is 85 multiplications to fill
//! two tables, replacing ~470 spent inside the term loop.
//!
//! ## Why this is safe for accuracy
//!
//! `f64::powi` uses exponentiation by squaring — `O(log n)` roundings — while
//! the multiply chain here accumulates `O(n)`, and region 1 reaches `J = −41`.
//! Measured against the `powi` formulation over a grid of state points, the
//! two agree to **1.7e-11** (region 1), **1.3e-12** (region 3) and **7.7e-14**
//! (region 2) — three or more orders below the 1e-8 at which IF97 states its
//! own acceptance tables, which both formulations satisfy. The region test
//! modules pin this: each keeps the original `powi` formulation as a
//! `#[cfg(test)]` oracle and asserts agreement over a grid.
//!
//! **Rejected refinement — re-anchoring the chain on `powi`.** Recomputing
//! every 8th power exactly, to cap the chain at seven multiplications, was
//! implemented and measured: region 1 went 1.728e-11 → **1.749e-11** and
//! region 3 went 1.270e-12 → **5.008e-12**. No improvement, because the
//! divergence is not accumulated chain error at all — it is cancellation
//! amplification of last-ulp differences inside sums whose terms are orders of
//! magnitude larger than their total (region 1's `γ_π` most of all). Anchoring
//! moves which ulp differs without shrinking the amplification. Reverted; do
//! not re-propose without a measurement showing otherwise.

/// Build the contiguous window of integer powers `x^k` for
/// `k ∈ [lo, lo + N - 1]`, returned as an array indexed by `(k - lo)`.
///
/// # Arguments
/// * `x` — The base. Must be **strictly positive** (every IF97 series
///   variable is: `7.1 − π ≥ 1.05`, `τ − 0.5 > 0`, `δ > 0`, …), because the
///   negative half of the window is built by repeated division.
/// * `lo` — The lowest exponent in the window (typically `Iₘᵢₙ − 2`, since the
///   second derivative needs `x^(I−2)`).
///
/// # Returns
/// `[f64; N]` where element `k - lo` is `x^k` (dimensionless).
#[inline(always)]
pub(crate) fn powers<const N: usize>(x: f64, lo: i32) -> [f64; N] {
    let mut out = [0.0; N];
    // Anchor at x⁰ = 1 and walk outwards, so each power costs one operation.
    let zero = (-lo) as usize;
    out[zero] = 1.0;
    for k in (zero + 1)..N {
        out[k] = out[k - 1] * x;
    }
    let inv = 1.0 / x;
    for k in (0..zero).rev() {
        out[k] = out[k + 1] * inv;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Every entry must reproduce `powi` to near machine precision.
    #[test]
    fn matches_powi_over_the_window() {
        for x in [1.05, 1.7, 2.5, 0.0031, 0.5, 12.0, 300.0] {
            let p = powers::<61>(x, -43);
            for k in -43..=17i32 {
                assert_relative_eq!(p[(k + 43) as usize], x.powi(k), max_relative = 1e-13);
            }
        }
    }

    /// The anchor and its immediate neighbours must be exact, not merely close:
    /// `x⁰ = 1`, `x¹ = x`, `x⁻¹ = 1/x`.
    #[test]
    fn anchor_is_exact() {
        let x = 3.7;
        let p = powers::<9>(x, -4);
        assert_eq!(p[4], 1.0);
        assert_eq!(p[5], x);
        assert_eq!(p[3], 1.0 / x);
    }
}
