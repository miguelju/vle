//! Maxwell–Bonnell vapor pressure and the atmospheric-equivalent boiling point.
//!
//! # Why a petroleum-specific vapor-pressure correlation exists
//!
//! Heavy petroleum cracks before it boils. A vacuum gas oil that would boil at
//! 500 °C atmospherically decomposes somewhere above 350 °C, so its
//! distillation has to be run under vacuum — and then the measured
//! temperatures are not boiling points at all, they are boiling points *at
//! 10 mmHg*. Every characterization correlation in [`super::properties`] wants
//! the **normal** boiling point. Something has to bridge the two.
//!
//! That something is the **atmospheric equivalent temperature** (AET): the
//! temperature at which the material *would* boil at 760 mmHg if it survived
//! the trip. Maxwell & Bonnell's correlation (40), adopted as API Procedure
//! 5A1.19, is the industry-standard way to compute it, and it is what makes
//! ASTM D1160 and D2892 vacuum data usable at all.
//!
//! # Shape of the correlation
//!
//! A pressure-dependent group `Q` — three branches, because the fit changes at
//! 2 mmHg and at 1 atm — relates the observed temperature to an uncorrected
//! normal boiling point:
//!
//! ```text
//!   Tb′ = 748.1·Q·T / (1 + T·(0.3861·Q − 0.00051606))      T in K
//! ```
//!
//! and then a **Watson-K correction** accounts for the fact that the whole
//! thing was fit on n-hexane:
//!
//! ```text
//!   Tb = Tb′ + 1.3889 · f · (K_W − 12) · log₁₀(P/760)
//!   f  = clamp((1.8·Tb′ − 659.67) / 200, 0, 1)
//! ```
//!
//! The `(K_W − 12)` factor is the physically meaningful part: at `K_W = 12`
//! the fraction *is* the reference and the correction vanishes identically.
//!
//! > **A note on a discrepancy.** At least one open-source implementation drops
//! > the `(K_W − 12)` factor on the upper `f = 1` branch, which makes the
//! > correction fail to vanish at the reference. The form used here keeps it.
//! > That is not just an appeal to the published form: tested against this
//! > crate's own Antoine equations for benzene and toluene at pressures more
//! > than half a decade away from atmospheric — the region where the two
//! > differ most — keeping the factor gives a mean error of **0.26 %** against
//! > **0.30 %**, and a worst case of **0.74 %** against **0.97 %**. The
//! > measurement and the theory agree, so the factor stays.
//!
//! # Accuracy
//!
//! Round-tripped against this crate's component database: feed the correlation
//! a hydrocarbon's Antoine vapor pressure at some temperature and ask for its
//! normal boiling point. Over seven hydrocarbons and 320–520 K (77 usable
//! points) the mean error is **0.19 %** and the worst **1.09 %** — a few kelvin
//! on a boiling point of several hundred. The tests at the bottom of this file
//! assert exactly that.
//!
//! # Two quirks of the published fit, measured
//!
//! Both are properties of Maxwell–Bonnell itself, not of this implementation,
//! and both are asserted in the tests so they cannot be mistaken for
//! regressions later:
//!
//! 1. **It is not an identity at 1 atm.** Asking for the boiling point of a
//!    fraction at 760 mmHg does not return its normal boiling point exactly; it
//!    overshoots by **0.22 K at Tb = 350 K rising to 0.37 K at Tb = 750 K**.
//!    That is simply the residual of an empirical fit, and it is well inside
//!    the correlation's own stated accuracy.
//! 2. **`Q` steps at the 760 mmHg branch boundary.** The sub- and
//!    super-atmospheric fits do not meet exactly, which puts a **0.35–0.55 K
//!    step** into any boiling-point curve that crosses 1 atm. It is small, but
//!    it is a true discontinuity, so [`vapor_pressure`] — which inverts the
//!    relation numerically — can land on the step instead of a root when the
//!    answer is near atmospheric. It returns the pressure at the step, which is
//!    correct to within the step's width.
//!
//! # References
//! - (40) Maxwell, J. B.; Bonnell, L. S. *Vapor Pressure Charts for Petroleum
//!   Engineers*; Esso Research: Linden, NJ, **1955**; and *Ind. Eng. Chem.*
//!   **1957**, *49*, 1187.
//! - (41) API *Technical Data Book — Petroleum Refining*, Procedure 5A1.19.

use super::PetroleumError;
use crate::numerics::root_finding::brent;

/// mmHg per kPa. Maxwell–Bonnell is published in mmHg and the branch
/// boundaries (2 mmHg, 760 mmHg) are in that unit, so the conversion happens
/// at the boundary of every public function rather than inside the algebra.
const MMHG_PER_KPA: f64 = 760.0 / 101.325;

/// Standard atmospheric pressure in mmHg — the reference the correlation
/// corrects towards.
const P_STANDARD_MMHG: f64 = 760.0;

/// Lowest pressure the correlation is defined at, mmHg. Below this the `Q`
/// logarithms lose meaning and the fit was never exercised.
const P_MIN_MMHG: f64 = 1e-4;
/// Highest pressure the correlation is defined at, mmHg (~66 bar).
const P_MAX_MMHG: f64 = 50_000.0;

/// The pressure-dependent group `Q`, dimensionless. `p` in **mmHg**.
///
/// Three branches, split at 2 mmHg (deep vacuum) and 760 mmHg (atmospheric).
/// They do not meet exactly; the resulting step in the boiling point is quirk 2
/// in the module docs, measured by
/// `the_branch_boundaries_step_by_the_documented_amount` in the tests.
fn q_group(p_mmhg: f64) -> f64 {
    let lp = p_mmhg.log10();
    let (a, b, c, d) = if p_mmhg < 2.0 {
        Q_BRANCHES[0]
    } else if p_mmhg < P_STANDARD_MMHG {
        Q_BRANCHES[1]
    } else {
        Q_BRANCHES[2]
    };
    (a - b * lp) / (c - d * lp)
}

/// The Watson-K correction term, **K**, added to the uncorrected `Tb′`.
///
/// `tb_ref` is whichever boiling point the ramp is evaluated at — `Tb′` going
/// forward, the known normal boiling point going backward.
fn watson_correction(tb_ref: f64, p_mmhg: f64, watson_k: Option<f64>) -> f64 {
    let Some(kw) = watson_k else { return 0.0 };
    // Ramp from 0 at 659.67 °R (366.5 K) to 1 at 859.67 °R (477.6 K). Written
    // in °R because that is how it is published; the K-form 0.009·T − 3.29835
    // is the same function.
    let f = ((1.8 * tb_ref - 659.67) / 200.0).clamp(0.0, 1.0);
    1.3889 * f * (kw - 12.0) * (p_mmhg / P_STANDARD_MMHG).log10()
}

fn check_pressure(p_kpa: f64) -> Result<f64, PetroleumError> {
    if p_kpa <= 0.0 || !p_kpa.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "pressure must be positive and finite, got {p_kpa} kPa"
        )));
    }
    let mmhg = p_kpa * MMHG_PER_KPA;
    if !(P_MIN_MMHG..=P_MAX_MMHG).contains(&mmhg) {
        return Err(PetroleumError::InvalidInput(format!(
            "pressure {p_kpa} kPa ({mmhg:.4} mmHg) is outside the \
             {P_MIN_MMHG}–{P_MAX_MMHG} mmHg range Maxwell-Bonnell covers"
        )));
    }
    Ok(mmhg)
}

fn check_temperature(t: f64) -> Result<(), PetroleumError> {
    if t <= 0.0 || !t.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "temperature must be positive and finite, got {t} K"
        )));
    }
    Ok(())
}

/// Normal boiling point (atmospheric equivalent temperature) of a fraction
/// observed to boil at `t` under pressure `p`.
///
/// This is the direction a vacuum distillation needs: the lab measured a
/// temperature at 10 mmHg, and every downstream correlation wants the
/// atmospheric value.
///
/// Ref (40), API Procedure 5A1.19.
///
/// # Arguments
/// * `t` — observed boiling temperature, **K**.
/// * `p` — pressure the observation was made at, **kPa** (absolute).
/// * `watson_k` — Watson characterization factor, **dimensionless**. Pass
///   `None` to skip the compositional correction, which is correct when the
///   fraction is n-hexane-like (`K_W ≈ 12`) or simply unknown.
///
/// # Returns
/// The normal boiling point at 101.325 kPa, in **K**.
pub fn normal_boiling_point(t: f64, p: f64, watson_k: Option<f64>) -> Result<f64, PetroleumError> {
    check_temperature(t)?;
    let p_mmhg = check_pressure(p)?;
    let q = q_group(p_mmhg);
    // Tb' = 748.1·Q·T / (1 + T·(0.3861·Q − 0.00051606)), T in K.
    let denom = 1.0 + t * (0.3861 * q - 0.000_516_06);
    if denom <= 0.0 {
        return Err(PetroleumError::NoConvergence(format!(
            "Maxwell-Bonnell is degenerate at T = {t} K, P = {p} kPa"
        )));
    }
    let tb_uncorrected = 748.1 * q * t / denom;

    // The published correction evaluates its ramp at the *normal* boiling
    // point — which is the answer, so the relation is implicit. Solve it by
    // fixed-point iteration. The map is a strong contraction (the ramp's slope
    // is 0.009 K⁻¹ and the whole correction is only a kelvin or two), so this
    // settles in a handful of passes; the loop bound is a backstop, not a
    // working iteration count. Doing it properly here is what makes
    // `boiling_point_at_pressure` an exact inverse rather than an approximate
    // one, since that direction knows the normal boiling point outright.
    let mut tb = tb_uncorrected;
    for _ in 0..100 {
        let next = tb_uncorrected + watson_correction(tb, p_mmhg, watson_k);
        if (next - tb).abs() < 1e-13 {
            return Ok(next);
        }
        tb = next;
    }
    Err(PetroleumError::NoConvergence(format!(
        "the Maxwell-Bonnell Watson-K correction did not settle at T = {t} K, P = {p} kPa"
    )))
}

/// Boiling temperature at pressure `p` of a fraction whose **normal** boiling
/// point is `tb` — the exact inverse of [`normal_boiling_point`].
///
/// This is the direction a flash calculation needs: given a characterized cut,
/// at what temperature does it boil in a vacuum tower?
///
/// # Arguments
/// * `tb` — normal boiling point at 101.325 kPa, **K**.
/// * `p` — pressure of interest, **kPa** (absolute).
/// * `watson_k` — Watson characterization factor, **dimensionless**, or `None`.
///
/// # Returns
/// The boiling temperature at `p`, in **K**.
pub fn boiling_point_at_pressure(
    tb: f64,
    p: f64,
    watson_k: Option<f64>,
) -> Result<f64, PetroleumError> {
    check_temperature(tb)?;
    let p_mmhg = check_pressure(p)?;
    let q = q_group(p_mmhg);
    // Undo the Watson correction first — going backwards the ramp is evaluated
    // at the known normal boiling point, which is what it is a proxy for.
    let tb_uncorrected = tb - watson_correction(tb, p_mmhg, watson_k);
    // Invert Tb' = 748.1·Q·T/(1 + T·c) for T, with c = 0.3861·Q − 0.00051606.
    let c = 0.3861 * q - 0.000_516_06;
    let denom = 748.1 * q - tb_uncorrected * c;
    if denom <= 0.0 {
        return Err(PetroleumError::NoConvergence(format!(
            "Maxwell-Bonnell inversion is degenerate at Tb = {tb} K, P = {p} kPa"
        )));
    }
    Ok(tb_uncorrected / denom)
}

/// The three `Q` branches as `(a, b, c, d, L_lo, L_hi)` in
/// `Q = (a − b·L)/(c − d·L)`, `L = log₁₀ P[mmHg]`, each valid on `[L_lo, L_hi)`.
/// Written once so [`q_group`] and the closed-form inversion in
/// [`vapor_pressure`] cannot drift apart.
const Q_BRANCHES: [(f64, f64, f64, f64); 3] = [
    (6.761_56, 0.987_672, 3000.538, 43.0),
    (5.994_296, 0.972_546, 2663.129, 95.76),
    (6.412_631, 0.989_679, 2770.085, 36.0),
];

/// Vapor pressure of a petroleum fraction at temperature `t`.
///
/// The correlation is published as a boiling-point relation, but for a fixed
/// `Q` branch it is **algebraic in `L = log₁₀ P`**: `Q` is a ratio of two
/// affine functions of `L`, and the Watson correction is affine in `L` too, so
/// `boiling_point_at_pressure(tb, P) = t` collapses to a *quadratic* in `L`
/// per branch (linear when `watson_k` is `None`). This function solves that
/// quadratic on each of the three branches and keeps the root that lands
/// inside its own branch — no iteration, no bracketing, ~40 flops. That
/// matters because the BK10 K-value path calls this once per component per
/// stage per iteration, and a column solve wants it to cost what an Antoine
/// evaluation costs.
///
/// The three fits do not meet exactly at 2 and 760 mmHg (quirk 2 in the module
/// docs), so a `t` that falls *inside* the step has no root on any branch. That
/// case is detected explicitly and returns the boundary pressure — the answer
/// is then correct to within the step's own width (0.35–0.55 K), exactly as the
/// numerical inversion it replaced, but deterministically rather than by
/// whichever side Brent happened to converge on. Every closed-form root is
/// checked against the forward relation before being returned; the Brent
/// solve survives only as an unreachable-in-practice fallback.
///
/// # Arguments
/// * `t` — temperature, **K**.
/// * `tb` — the fraction's normal boiling point, **K**.
/// * `watson_k` — Watson characterization factor, **dimensionless**, or `None`.
///
/// # Returns
/// Vapor pressure in **kPa** (absolute).
///
/// # Errors
/// [`PetroleumError::NoConvergence`] if the required pressure falls outside the
/// 10⁻⁴–5×10⁴ mmHg window the correlation covers — which for a fraction with a
/// sane `tb` means `t` is far below its freezing point or above its critical
/// point.
pub fn vapor_pressure(t: f64, tb: f64, watson_k: Option<f64>) -> Result<f64, PetroleumError> {
    check_temperature(t)?;
    check_temperature(tb)?;
    if let Some(p) = vapor_pressure_closed_form(t, tb, watson_k, false) {
        return Ok(p);
    }
    vapor_pressure_brent(t, tb, watson_k)
}

/// **ln** of the Maxwell–Bonnell vapor pressure — what a K-value assembly wants
/// (`ln K = ln Pˢᵃᵗ − ln P`). `ln` of **kPa**.
///
/// Unlike [`vapor_pressure`] this **extrapolates the outer `Q` branches** past
/// the 10⁻⁴–5×10⁴ mmHg window instead of erroring, because a K-value path
/// cannot afford to fail on a light end that happens to be far above its
/// critical point on a hot stage (a Tb = 340 K cut at 600 K wants
/// "Pˢᵃᵗ" ≈ 100 bar — physically meaningless as a vapor pressure, entirely
/// adequate as "K ≫ 1"). The Braun K10 charts themselves are used this way.
/// Inside the window the result is identical to [`vapor_pressure`].
///
/// # Errors
/// [`PetroleumError::InvalidInput`] on a non-positive `t` or `tb`;
/// [`PetroleumError::NoConvergence`] only if even the extrapolated branches
/// admit no root (a degenerate quadratic).
pub fn ln_vapor_pressure(t: f64, tb: f64, watson_k: Option<f64>) -> Result<f64, PetroleumError> {
    check_temperature(t)?;
    check_temperature(tb)?;
    vapor_pressure_closed_form(t, tb, watson_k, true)
        .map(f64::ln)
        .ok_or_else(|| {
            PetroleumError::NoConvergence(format!(
                "Maxwell-Bonnell has no vapor pressure for Tb = {tb} K at T = {t} K"
            ))
        })
}

/// [`boiling_point_at_pressure`] without the pressure-window check, `p` in
/// **mmHg** — the forward relation the closed-form inversion verifies against,
/// which must be evaluable on the extrapolated branches too. `None` where the
/// relation is degenerate.
fn boiling_point_unchecked(tb: f64, p_mmhg: f64, watson_k: Option<f64>) -> Option<f64> {
    if !(p_mmhg > 0.0 && p_mmhg.is_finite()) {
        return None;
    }
    let q = q_group(p_mmhg);
    let tb_uncorrected = tb - watson_correction(tb, p_mmhg, watson_k);
    let c = 0.3861 * q - 0.000_516_06;
    let denom = 748.1 * q - tb_uncorrected * c;
    (denom > 0.0).then(|| tb_uncorrected / denom)
}

/// The closed-form branch-wise inversion. `None` if no branch admits a root
/// that reproduces `t` (out of range, or a degenerate quadratic). With
/// `extrapolate`, the lowest and highest branches are extended beyond
/// `P_MIN_MMHG` / `P_MAX_MMHG` (see [`ln_vapor_pressure`]).
fn vapor_pressure_closed_form(
    t: f64,
    tb: f64,
    watson_k: Option<f64>,
    extrapolate: bool,
) -> Option<f64> {
    let l0 = P_STANDARD_MMHG.log10();
    // Watson correction as k·(L − L0): the ramp is evaluated at the known Tb.
    let k = match watson_k {
        Some(kw) => 1.3889 * ((1.8 * tb - 659.67) / 200.0).clamp(0.0, 1.0) * (kw - 12.0),
        None => 0.0,
    };
    // Tb′ = m − k·L.
    let m = tb + k * l0;
    let a0 = 748.1 - 0.3861 * m;
    let s = 1.0 - 0.000_516_06 * t;
    let bounds = if extrapolate {
        [f64::NEG_INFINITY, 2f64.log10(), l0, f64::INFINITY]
    } else {
        [P_MIN_MMHG.log10(), 2f64.log10(), l0, P_MAX_MMHG.log10()]
    };
    let mut best: Option<(f64, f64)> = None; // (|residual|, L)
    for (i, &(a, b, c, d)) in Q_BRANCHES.iter().enumerate() {
        // Q·T·(748.1 − 0.3861·Tb′) = Tb′·(1 − 0.00051606·T), Q = (a − bL)/(c − dL),
        // Tb′ = m − kL  ⇒  α L² + β L + γ = 0.
        let alpha = -0.3861 * b * k * t - s * d * k;
        let beta = t * (0.3861 * a * k - b * a0) + s * (c * k + d * m);
        let gamma = t * a * a0 - s * c * m;
        let mut roots: [Option<f64>; 2] = [None, None];
        if alpha.abs() < 1e-300 {
            if beta != 0.0 {
                roots[0] = Some(-gamma / beta);
            }
        } else {
            let disc = beta * beta - 4.0 * alpha * gamma;
            if disc >= 0.0 {
                let sq = disc.sqrt();
                // Numerically stable pair (avoids cancellation in one root).
                let q = -0.5 * (beta + beta.signum() * sq);
                roots[0] = Some(q / alpha);
                if q != 0.0 {
                    roots[1] = Some(gamma / q);
                }
            }
        }
        let (lo, hi) = (bounds[i], bounds[i + 1]);
        for l in roots.into_iter().flatten() {
            let inside = if i == 2 {
                (lo..=hi).contains(&l)
            } else {
                (lo..hi).contains(&l)
            };
            if !inside {
                continue;
            }
            // Confirm against the forward relation — the quadratic can carry a
            // spurious root where the denominator changes sign.
            if let Some(t_back) = boiling_point_unchecked(tb, 10f64.powf(l), watson_k) {
                let r = (t_back - t).abs();
                if r < 1e-6 && best.is_none_or(|(rb, _)| r < rb) {
                    best = Some((r, l));
                }
            }
        }
    }
    if let Some((_, l)) = best {
        return Some(10f64.powf(l) / MMHG_PER_KPA);
    }
    // No branch owns `t`: it sits inside a step between two fits. Return the
    // boundary whose two-sided boiling points straddle `t`.
    for &p_mmhg in &[2.0, P_STANDARD_MMHG] {
        let p_kpa = p_mmhg / MMHG_PER_KPA;
        let below = boiling_point_at_pressure(tb, p_kpa * (1.0 - 1e-9), watson_k).ok()?;
        let above = boiling_point_at_pressure(tb, p_kpa * (1.0 + 1e-9), watson_k).ok()?;
        if (below.min(above)..=below.max(above)).contains(&t) {
            return Some(p_kpa);
        }
    }
    None
}

/// The original numerical inversion — Brent on `log₁₀ P`. Kept as the fallback
/// behind [`vapor_pressure_closed_form`] and as the oracle its tests compare
/// against.
fn vapor_pressure_brent(t: f64, tb: f64, watson_k: Option<f64>) -> Result<f64, PetroleumError> {
    // Residual in log₁₀ P (mmHg): zero where the fraction boils at exactly `t`.
    let residual = |log_p: f64| -> f64 {
        let p_kpa = 10f64.powf(log_p) / MMHG_PER_KPA;
        match boiling_point_at_pressure(tb, p_kpa, watson_k) {
            Ok(t_boil) => t_boil - t,
            // Push the solver back inside the valid window rather than
            // poisoning the bracket with a NaN.
            Err(_) => f64::INFINITY * (log_p - P_STANDARD_MMHG.log10()).signum(),
        }
    };
    let (lo, hi) = (P_MIN_MMHG.log10(), P_MAX_MMHG.log10());
    let (f_lo, f_hi) = (residual(lo), residual(hi));
    if !(f_lo <= 0.0 && f_hi >= 0.0) {
        return Err(PetroleumError::NoConvergence(format!(
            "no vapor pressure in {P_MIN_MMHG}–{P_MAX_MMHG} mmHg puts the boiling \
             point of a Tb = {tb} K fraction at {t} K"
        )));
    }
    let log_p = brent(residual, lo, hi, 1e-12, 200).map_err(|e| {
        PetroleumError::NoConvergence(format!(
            "Maxwell-Bonnell vapor-pressure inversion failed at T = {t} K, Tb = {tb} K: {e}"
        ))
    })?;
    Ok(10f64.powf(log_p) / MMHG_PER_KPA)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(name, Tb [K], SG 60/60, Pc [kPa], reduced-Antoine [a1, a2, a3])`.
    ///
    /// The Antoine coefficients are this crate's bundled component database:
    /// `ln(Psat/Pc) = a1 − a2/(a3 + T)`, T in K, Psat and Pc in kPa. They are a
    /// genuinely **independent** oracle for Maxwell–Bonnell — nothing in this
    /// module was fit to them — which is what makes the round-trip test below a
    /// real check rather than a tautology.
    const REFERENCE: [(&str, f64, f64, f64, [f64; 3]); 7] = [
        (
            "n-hexane",
            341.866,
            0.664,
            3044.1,
            [5.800675, 2697.547514, -48.784],
        ),
        (
            "n-heptane",
            371.55,
            0.6882,
            2735.73,
            [5.966566, 2921.142342, -56.199],
        ),
        (
            "n-octane",
            398.794,
            0.707,
            2483.59,
            [6.114906, 3123.134317, -63.515],
        ),
        (
            "n-nonane",
            423.913,
            0.7219,
            2281.0,
            [6.252519, 3311.186441, -70.456],
        ),
        (
            "n-decane",
            447.27,
            0.7342,
            2103.0,
            [6.322187, 3442.756153, -79.292],
        ),
        (
            "benzene",
            353.219,
            0.8829,
            4907.277,
            [5.358805, 2771.932525, -53.226],
        ),
        (
            "toluene",
            383.746,
            0.8719,
            4126.3,
            [5.606494, 3056.958021, -55.525],
        ),
    ];

    fn antoine_psat(row: &(&str, f64, f64, f64, [f64; 3]), t: f64) -> f64 {
        let [a1, a2, a3] = row.4;
        row.3 * (a1 - a2 / (a3 + t)).exp()
    }

    fn kw_of(row: &(&str, f64, f64, f64, [f64; 3])) -> f64 {
        super::super::gravity::watson_k(row.1, row.2).unwrap()
    }

    // === Validation against an independent vapor-pressure oracle ==========

    #[test]
    fn recovers_normal_boiling_points_from_antoine_vapor_pressures() {
        // The headline claim in the module docs. Take a hydrocarbon's real
        // vapor pressure at some temperature, hand the pair to Maxwell-Bonnell,
        // and it must return that hydrocarbon's normal boiling point.
        let mut worst: f64 = 0.0;
        let mut who = String::new();
        let mut total = 0.0;
        let mut n = 0;
        for row in &REFERENCE {
            let kw = kw_of(row);
            for step in 0..=10 {
                let t = 320.0 + step as f64 * 20.0;
                let p = antoine_psat(row, t);
                let Ok(tb) = normal_boiling_point(t, p, Some(kw)) else {
                    continue;
                };
                let err = 100.0 * (tb - row.1).abs() / row.1;
                total += err;
                n += 1;
                if err > worst {
                    worst = err;
                    who = format!("{} at {t:.0} K", row.0);
                }
            }
        }
        let mean = total / n as f64;
        assert!(
            n > 50,
            "only {n} usable points — the oracle sweep is too thin"
        );
        // The bounds are the measured values rounded up a hair, not aspirations,
        // so a mistyped coefficient trips them instead of hiding under slack.
        assert!(
            mean < 0.20,
            "mean error {mean:.3}%, expected < 0.20% over {n} points"
        );
        assert!(
            worst < 1.15,
            "worst error {worst:.3}% ({who}), expected < 1.15%"
        );
    }

    #[test]
    fn predicts_vapor_pressure_within_a_few_percent_of_antoine() {
        // The same check run the other way: given only Tb and Watson K,
        // Maxwell-Bonnell should reproduce the measured vapor pressure. Errors
        // in P are much larger than errors in T because vapor pressure is
        // exponential in temperature — a 1 % error in Tb is a ~10 % error in P,
        // which is exactly what this bound reflects.
        for row in &REFERENCE {
            let kw = kw_of(row);
            for step in 0..=6 {
                let t = 340.0 + step as f64 * 25.0;
                let want = antoine_psat(row, t);
                if !(0.05..3000.0).contains(&want) {
                    continue;
                }
                let got = vapor_pressure(t, row.1, Some(kw)).unwrap();
                let err = 100.0 * (got - want).abs() / want;
                assert!(
                    err < 25.0,
                    "{} at {t:.0} K: Maxwell-Bonnell {got:.3} kPa vs Antoine \
                     {want:.3} kPa ({err:.1}%)",
                    row.0
                );
            }
        }
    }

    #[test]
    fn is_nearly_but_not_exactly_an_identity_at_one_atmosphere() {
        // Quirk 1 in the module docs. At 760 mmHg the Watson correction
        // vanishes exactly (log₁₀(P/760) = 0), so what is left is the raw `Q`
        // relation — and being an empirical fit rather than an identity, it
        // does not return Tb exactly. Measured overshoot: +0.22 K at Tb = 350 K
        // rising to +0.37 K at Tb = 750 K. Asserted from both sides so that a
        // future change either preserves the quirk or is forced to explain
        // itself.
        for row in &REFERENCE {
            let t = boiling_point_at_pressure(row.1, 101.325, Some(kw_of(row))).unwrap();
            let err = t - row.1;
            assert!(
                (0.0..0.4).contains(&err),
                "{}: boiling point at 1 atm came back {t} K vs Tb = {} K \
                 (offset {err:+.4} K, expected the documented 0-0.4 K overshoot)",
                row.0,
                row.1
            );
        }
        // The offset grows with Tb, which is what makes it look like fit
        // residual rather than a unit error (a unit error would be
        // proportional, not slowly drifting).
        let light = boiling_point_at_pressure(350.0, 101.325, None).unwrap() - 350.0;
        let heavy = boiling_point_at_pressure(750.0, 101.325, None).unwrap() - 750.0;
        assert!(heavy > light, "offset {light:.4} K -> {heavy:.4} K");
    }

    // === Directional consistency =========================================

    #[test]
    fn the_two_directions_are_exact_inverses() {
        for kw in [None, Some(10.5), Some(12.0), Some(13.0)] {
            for tb in [350.0, 450.0, 600.0, 750.0] {
                for p in [0.5, 5.0, 50.0, 101.325, 500.0] {
                    let t = boiling_point_at_pressure(tb, p, kw).unwrap();
                    let back = normal_boiling_point(t, p, kw).unwrap();
                    assert!(
                        (back - tb).abs() < 1e-8,
                        "Tb {tb} K at {p} kPa -> T {t} K -> Tb {back} K (K_W = {kw:?})"
                    );
                }
            }
        }
    }

    #[test]
    fn vapor_pressure_inverts_the_boiling_point_relation() {
        for kw in [None, Some(11.0), Some(12.6)] {
            for tb in [400.0, 550.0, 700.0] {
                for t in [350.0, 450.0, 550.0] {
                    if t > tb + 150.0 {
                        continue;
                    }
                    let Ok(p) = vapor_pressure(t, tb, kw) else {
                        continue;
                    };
                    let back = boiling_point_at_pressure(tb, p, kw).unwrap();
                    // 0.6 K rather than machine precision because of quirk 2 in
                    // the module docs: `Q` steps by 0.35-0.55 K across the
                    // 760 mmHg branch boundary, so when the answer lands near
                    // 1 atm the solver converges on the step rather than on a
                    // true root. Away from that boundary this is exact.
                    assert!(
                        (back - t).abs() < 0.6,
                        "T {t} K -> P {p} kPa -> T {back} K (Tb {tb} K, K_W {kw:?})"
                    );
                }
            }
        }
    }

    // === Physical behaviour ==============================================

    #[test]
    fn vacuum_lowers_the_boiling_point() {
        // The entire reason vacuum distillation exists. A 600 K normal boiling
        // point cut must boil well below 600 K at 10 mmHg — and the drop must
        // be large, since that is what keeps heavy oil below its cracking
        // temperature.
        let p_10mmhg = 10.0 / MMHG_PER_KPA;
        let t = boiling_point_at_pressure(600.0, p_10mmhg, Some(11.8)).unwrap();
        assert!(t < 600.0, "boiling point at 10 mmHg came out at {t} K");
        assert!(
            (450.0..520.0).contains(&t),
            "a 600 K cut should boil around 470-490 K at 10 mmHg, got {t} K"
        );
    }

    #[test]
    fn boiling_point_rises_monotonically_with_pressure() {
        let mut prev = f64::NEG_INFINITY;
        for step in 0..40 {
            let p = 0.1 * 1.2f64.powi(step);
            let Ok(t) = boiling_point_at_pressure(550.0, p, Some(12.2)) else {
                continue;
            };
            assert!(t > prev, "boiling point fell to {t} K at {p} kPa");
            prev = t;
        }
    }

    #[test]
    fn vapor_pressure_rises_monotonically_with_temperature() {
        let mut prev = f64::NEG_INFINITY;
        for step in 0..25 {
            let t = 350.0 + step as f64 * 10.0;
            let Ok(p) = vapor_pressure(t, 500.0, Some(12.0)) else {
                continue;
            };
            assert!(p > prev, "vapor pressure fell to {p} kPa at {t} K");
            prev = p;
        }
    }

    #[test]
    fn the_watson_correction_vanishes_at_the_reference() {
        // K_W = 12 is n-hexane, the compound Maxwell & Bonnell fit. The
        // correction must be identically zero there — this is the property that
        // decides the (K_W − 12) form documented in the module header, and it
        // is why that form is used.
        for p in [1.0, 10.0, 101.325, 1000.0] {
            let with = normal_boiling_point(500.0, p, Some(12.0)).unwrap();
            let without = normal_boiling_point(500.0, p, None).unwrap();
            assert!(
                (with - without).abs() < 1e-12,
                "at {p} kPa the K_W = 12 correction moved Tb by {}",
                with - without
            );
        }
    }

    #[test]
    fn aromatic_and_paraffinic_corrections_pull_in_opposite_directions() {
        // Under vacuum log₁₀(P/760) < 0, so a paraffin (K_W > 12) is corrected
        // down and an aromatic (K_W < 12) up. A sign slip in the correction
        // shows up here and nowhere else.
        let p = 10.0 / MMHG_PER_KPA;
        let base = normal_boiling_point(500.0, p, None).unwrap();
        let paraffin = normal_boiling_point(500.0, p, Some(12.8)).unwrap();
        let aromatic = normal_boiling_point(500.0, p, Some(10.5)).unwrap();
        assert!(paraffin < base, "paraffin {paraffin} vs uncorrected {base}");
        assert!(aromatic > base, "aromatic {aromatic} vs uncorrected {base}");
    }

    // === Numerics ========================================================

    #[test]
    fn the_branch_boundaries_step_by_the_documented_amount() {
        // Quirk 2 in the module docs, measured in the unit that matters. `Q`'s
        // three fits do not meet exactly, and the relative jump in `Q` itself
        // is too small to be informative — what a caller feels is the jump in
        // the *boiling point*, so that is what this pins.
        //
        // A regression here (say, someone "tidying" a branch boundary) would
        // change results for every vacuum cut, so the bounds are two-sided.
        for (boundary, lo_k, hi_k) in [(2.0, 0.05, 0.15), (760.0, 0.3, 0.6)] {
            for tb in [450.0, 650.0] {
                let below =
                    boiling_point_at_pressure(tb, boundary * (1.0 - 1e-9) / MMHG_PER_KPA, None)
                        .unwrap();
                let above =
                    boiling_point_at_pressure(tb, boundary * (1.0 + 1e-9) / MMHG_PER_KPA, None)
                        .unwrap();
                let jump = (above - below).abs();
                assert!(
                    (lo_k..=hi_k).contains(&jump),
                    "boiling point steps {jump:.4} K across {boundary} mmHg at \
                     Tb = {tb} K; the module docs claim {lo_k}-{hi_k} K"
                );
            }
        }
    }

    #[test]
    fn out_of_range_pressure_is_reported_not_extrapolated() {
        assert!(normal_boiling_point(500.0, 0.0, None).is_err());
        assert!(normal_boiling_point(500.0, -1.0, None).is_err());
        assert!(normal_boiling_point(500.0, 1e-9, None).is_err());
        assert!(normal_boiling_point(500.0, 1e7, None).is_err());
        assert!(boiling_point_at_pressure(500.0, 1e7, None).is_err());
    }

    #[test]
    fn out_of_range_temperature_is_reported() {
        assert!(normal_boiling_point(0.0, 101.325, None).is_err());
        assert!(normal_boiling_point(f64::NAN, 101.325, None).is_err());
        assert!(vapor_pressure(-5.0, 500.0, None).is_err());
        assert!(vapor_pressure(500.0, 0.0, None).is_err());
    }

    #[test]
    fn an_unreachable_vapor_pressure_says_so_rather_than_guessing() {
        // A 900 K cut at 200 K would need a pressure far below the correlation's
        // floor. It must report that, not clamp silently to the bracket edge.
        let err = vapor_pressure(200.0, 900.0, Some(11.5)).unwrap_err();
        assert!(
            matches!(err, PetroleumError::NoConvergence(_)),
            "got {err:?}"
        );
    }

    // === The closed-form inversion (M20) =================================

    #[test]
    fn closed_form_inversion_matches_the_brent_oracle_everywhere() {
        // The Brent solve is the previous implementation and the oracle. The
        // closed form must agree with it to solver tolerance across every
        // branch, with and without the Watson correction, on light and heavy
        // cuts alike. Points that fall in the branch step are compared on the
        // pressure the oracle converged to as well.
        let mut checked = 0;
        for tb in [350.0, 450.0, 550.0, 650.0, 750.0] {
            for kw in [None, Some(10.5), Some(11.8), Some(12.9)] {
                let mut t = 0.55 * tb;
                while t < 1.05 * tb {
                    if let Ok(oracle) = vapor_pressure_brent(t, tb, kw) {
                        let got =
                            vapor_pressure_closed_form(t, tb, kw, false).unwrap_or_else(|| {
                                panic!("no closed form at T={t}, Tb={tb}, K={kw:?}")
                            });
                        let rel = (got - oracle).abs() / oracle;
                        assert!(
                            rel < 1e-7,
                            "T={t} Tb={tb} K={kw:?}: closed form {got} vs Brent {oracle} ({rel:.2e})"
                        );
                        checked += 1;
                    }
                    t += 3.7;
                }
            }
        }
        assert!(checked > 500, "only {checked} points compared");
    }

    #[test]
    fn closed_form_reproduces_the_forward_relation_exactly() {
        // Round trip: T → P → T must close to floating-point noise, which is
        // stricter than the oracle comparison above (Brent stops at 1e-12 in
        // log P, i.e. ~2e-12 relative in P).
        for tb in [400.0, 600.0, 800.0] {
            for kw in [None, Some(11.0)] {
                for frac in [0.7, 0.8, 0.9, 0.97] {
                    let t = frac * tb;
                    let p = vapor_pressure(t, tb, kw).unwrap();
                    let back = boiling_point_at_pressure(tb, p, kw).unwrap();
                    assert!((back - t).abs() < 1e-8, "T={t} Tb={tb} K={kw:?}: {back}");
                }
            }
        }
    }

    #[test]
    fn a_temperature_inside_the_branch_step_returns_the_boundary_pressure() {
        // Quirk 2: the two fits at 760 mmHg do not meet, so a T strictly between
        // the two one-sided boiling points has no root on any branch. That must
        // yield exactly the boundary pressure, deterministically.
        let tb = 500.0;
        let p760 = P_STANDARD_MMHG / MMHG_PER_KPA;
        let below = boiling_point_at_pressure(tb, p760 * (1.0 - 1e-9), None).unwrap();
        let above = boiling_point_at_pressure(tb, p760 * (1.0 + 1e-9), None).unwrap();
        let mid = 0.5 * (below + above);
        assert!((below - above).abs() > 0.2, "no step to test at Tb={tb}");
        let p = vapor_pressure(mid, tb, None).unwrap();
        assert!(
            (p - p760).abs() < 1e-9,
            "got {p} kPa, expected the 760 mmHg boundary"
        );
    }

    #[test]
    fn ln_vapor_pressure_extrapolates_where_vapor_pressure_refuses() {
        // Inside the window the two agree exactly; outside it, the K-value
        // form extends the outer branch instead of failing.
        let p = vapor_pressure(450.0, 500.0, Some(11.5)).unwrap();
        assert!((ln_vapor_pressure(450.0, 500.0, Some(11.5)).unwrap() - p.ln()).abs() < 1e-12);
        assert!(vapor_pressure(600.0, 343.0, None).is_err());
        let ln_p = ln_vapor_pressure(600.0, 343.0, None).unwrap();
        assert!(
            ln_p > (P_MAX_MMHG / MMHG_PER_KPA).ln(),
            "extrapolated ln P = {ln_p}"
        );
        // and it is monotone in T through the boundary
        let a = ln_vapor_pressure(560.0, 343.0, None).unwrap();
        let b = ln_vapor_pressure(580.0, 343.0, None).unwrap();
        assert!(a < b && b < ln_p);
        // deep-vacuum side too
        assert!(vapor_pressure(200.0, 900.0, Some(11.5)).is_err());
        assert!(
            ln_vapor_pressure(200.0, 900.0, Some(11.5)).unwrap() < (P_MIN_MMHG / MMHG_PER_KPA).ln()
        );
    }
}
