//! IF97 Region 3 — near-critical, 623.15–863.15 K above the B23 line.
//!
//! Region 3 is the one region IF97 gives as a **Helmholtz** free energy
//! `f(ρ,T)` rather than a Gibbs `g(p,T)`: `φ(δ,τ) = n₁ ln δ + Σ nᵢ δ^Iᵢ τ^Jᵢ`
//! (R7-97(2012) Eq. 28), with `δ = ρ/ρc`, `τ = Tc/T`. Because it is explicit in
//! density, a `(T, p)` query must **iterate on ρ** until the implied pressure
//! matches — done here with a **safeguarded Newton** on the analytic
//! `∂p/∂ρ`, bisecting whenever a step would leave the bracket (see
//! [`density_tp`]). Property derivatives are analytic throughout, never finite
//! differences.
//!
//! Verification: R7-97 Table 33, which gives properties at `(ρ, T)` inputs
//! (e.g. `ρ=500 kg/m³, T=650 K` → `p=0.255837018×10² MPa`,
//! `h=0.186343019×10⁴ kJ/kg`, `s=0.405427273×10¹ kJ/(kg·K)`).
//!
//! Internal units: `ρ` in **kg/m³**, `T` in **K**, `p` in **kPa**.

use crate::coefficients::{REGION3, REGION3_N1};
use crate::props::{Helmholtz, Props, helmholtz_props};
use crate::series::powers;
use crate::solve::{bracket, brent};
use crate::{R, RHO_C, SteamError, T_C};

/// Upper density bound for the region-3 solves, **kg/m³**.
///
/// Region 3 never actually reaches this: its densest physical state is the
/// corner at 623.15 K and 100 MPa, about **715 kg/m³**. The bound matters
/// because the IF97 region-3 series is a *fit*, valid only inside the region,
/// and it misbehaves when extrapolated past it — measured on the shipped
/// coefficients, `p(ρ)` along the 650 K isotherm rises to 312 MPa at
/// 900 kg/m³ and then **collapses to 17 MPa at 1000**, and the 863.15 K
/// isotherm goes outright **negative** there.
///
/// The earlier `1000.0` therefore put a turning point inside the search
/// interval: `p(ρ)` was no longer monotone on the dense branch, the endpoints
/// stopped bracketing the root, and a bracketing scan could in principle
/// return a spurious root off the descending limb. At 760 kg/m³ the isotherm
/// is still monotone at every region-3 temperature and already exceeds
/// 130 MPa — comfortably past IF97's own 100 MPa ceiling — so nothing
/// physical is excluded.
const RHO_MAX: f64 = 760.0;

// Exponent windows for the power tables (see [`crate::series`]). Table 30 has
// `I ∈ [0, 11]` and `J ∈ [0, 26]`; the second derivatives reach two below each.
const D_LO: i32 = -2;
const D_N: usize = 14;
const T_LO: i32 = -2;
const T_N: usize = 29;

/// The five dimensionless Helmholtz derivatives at reduced `(δ, τ)`.
fn phi(delta: f64, tau: f64) -> Helmholtz {
    let pd = powers::<D_N>(delta, D_LO);
    let pt = powers::<T_N>(tau, T_LO);
    // Leading n₁·ln δ term (and its δ-derivatives).
    let mut d = Helmholtz {
        f: REGION3_N1 * delta.ln(),
        fd: REGION3_N1 / delta,
        fdd: -REGION3_N1 / (delta * delta),
        ft: 0.0,
        ftt: 0.0,
        fdt: 0.0,
    };
    for term in &REGION3 {
        let (i, j, n) = (term.i, term.j, term.n);
        let (fi, fj) = (i as f64, j as f64);
        let id = (i - D_LO) as usize;
        let jt = (j - T_LO) as usize;
        let (d0, d1, d2) = (pd[id], pd[id - 1], pd[id - 2]);
        let (t0, t1, t2) = (pt[jt], pt[jt - 1], pt[jt - 2]);
        d.f += n * d0 * t0;
        d.fd += n * fi * d1 * t0;
        d.fdd += n * fi * (fi - 1.0) * d2 * t0;
        d.ft += n * d0 * fj * t1;
        d.ftt += n * d0 * fj * (fj - 1.0) * t2;
        d.fdt += n * fi * d1 * fj * t1;
    }
    d
}

/// Just `(φ_δ, φ_δδ)` — the two derivatives the density solve needs.
///
/// The `(T, p) → ρ` inversion iterates on pressure alone, and pressure depends
/// only on `φ_δ` (with `φ_δδ` supplying `∂p/∂ρ` for Newton). Evaluating the
/// full six-derivative [`phi`] there would compute four sums per term that the
/// solve then discards; skipping them makes each iteration roughly three times
/// cheaper. The converged point still goes through [`phi`] once, so the
/// returned properties come from the same series as ever.
#[inline]
fn phi_delta(delta: f64, tau: f64) -> (f64, f64) {
    let pd = powers::<D_N>(delta, D_LO);
    let pt = powers::<T_N>(tau, T_LO);
    let mut fd = REGION3_N1 / delta;
    let mut fdd = -REGION3_N1 / (delta * delta);
    for term in &REGION3 {
        let (i, j, n) = (term.i, term.j, term.n);
        let fi = i as f64;
        let id = (i - D_LO) as usize;
        let t0 = pt[(j - T_LO) as usize];
        fd += n * fi * pd[id - 1] * t0;
        fdd += n * fi * (fi - 1.0) * pd[id - 2] * t0;
    }
    (fd, fdd)
}

/// Pressure implied by a `(ρ, T)` state in region 3.
///
/// # Arguments
/// * `rho` — Density in **kg/m³**.
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Pressure in **kPa absolute**. (`p = ρ R T δ φ_δ`; the units work because
/// `kg/m³ · kJ/(kg·K) · K = kJ/m³ = kPa`.)
pub(crate) fn pressure(rho: f64, t: f64) -> f64 {
    let delta = rho / RHO_C;
    let tau = T_C / t;
    let (fd, _) = phi_delta(delta, tau);
    rho * R * t * delta * fd
}

/// Pressure **and** its density slope at `(ρ, T)` — the Newton pair for the
/// `(T, p) → ρ` inversion.
///
/// # Arguments
/// * `rho` — Density in **kg/m³**.
/// * `t` — Temperature in **K**.
///
/// # Returns
/// `(p, ∂p/∂ρ|_T)` with `p` in **kPa absolute** and the slope in
/// **kPa/(kg/m³)** — i.e. kJ/kg, since `∂p/∂ρ = R·T·(2δφ_δ + δ²φ_δδ)`
/// (differentiating `p = ρRTδφ_δ` at fixed `T`, with `δ = ρ/ρc`). Analytic,
/// never a difference quotient.
/// `(∂p/∂ρ)_T` alone, for callers that already know the density.
///
/// # Arguments
/// * `rho` — Density in **kg/m³**.
/// * `t` — Temperature in **K**.
///
/// # Returns
/// `(∂p/∂ρ)_T` in **kPa/(kg/m³)**. [`crate::transport`] inverts it to get the
/// `(∂ρ/∂p)_T` that the R15-11 critical enhancement needs.
pub(crate) fn dp_drho(rho: f64, t: f64) -> f64 {
    pressure_slope(rho, t).1
}

#[inline]
fn pressure_slope(rho: f64, t: f64) -> (f64, f64) {
    let delta = rho / RHO_C;
    let tau = T_C / t;
    let (fd, fdd) = phi_delta(delta, tau);
    (
        rho * R * t * delta * fd,
        R * t * (2.0 * delta * fd + delta * delta * fdd),
    )
}

/// Full property set for a region-3 state given by `(ρ, T)`.
///
/// # Arguments
/// * `rho` — Density in **kg/m³**.
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Mass-basis [`Props`] (see that struct for units).
pub(crate) fn props_rho_t(rho: f64, t: f64) -> Props {
    let delta = rho / RHO_C;
    let tau = T_C / t;
    helmholtz_props(phi(delta, tau), delta, tau, rho, t)
}

/// Full property set for a region-3 state given by `(T, p)` — solves for ρ.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Pressure in **kPa absolute**.
///
/// # Returns
/// Mass-basis [`Props`], or [`SteamError::NoConvergence`] if the density solve
/// fails to bracket/converge.
pub(crate) fn props_tp(t: f64, p_kpa: f64) -> Result<Props, SteamError> {
    let rho = density_tp(t, p_kpa)?;
    Ok(props_rho_t(rho, t))
}

/// Solve `pressure(ρ, T) = p` for density ρ in region 3.
///
/// # Arguments
/// * `t` — Temperature in **K**.
/// * `p_kpa` — Target pressure in **kPa absolute**.
///
/// # Returns
/// Density in **kg/m³**.
///
/// Strategy: the region-3 isotherm `p(ρ)` is monotonic for `T ≥ Tc`; below the
/// critical temperature it has a van-der-Waals loop, so we pick the liquid
/// (high-ρ) or vapor (low-ρ) branch by comparing `p` to `Psat(T)`.
///
/// **On each branch the residual has exactly one sign change**, which is what
/// makes the branch endpoints a valid bracket on their own. Below `Tc` the
/// dense branch runs from `ρc` — where the isotherm sits inside the loop at
/// roughly `Psat`, so `p(ρc) − p < 0` for a compressed liquid — up to
/// [`RHO_MAX`], where it has long since risen past any region-3 pressure; the
/// light branch mirrors it. So the solve here is a **safeguarded Newton**
/// (Newton on `∂p/∂ρ`, falling back to bisection whenever a step would leave
/// the bracket or fails to shrink it fast enough) seeded from the branch
/// midpoint — not a scan.
///
/// The previous formulation stepped a 64-point linear scan across the branch
/// looking for that sign change *before* Brent ever started, evaluating the
/// full 40-term six-derivative series at every step. That scan, not the root
/// finder, was the cost: region-3 `(T,p)` queries measured 4.5–5.3 µs against
/// region 1's 259 ns for the same-sized series. See `steam_audit.md`.
pub(crate) fn density_tp(t: f64, p_kpa: f64) -> Result<f64, SteamError> {
    // Region-3 density envelope: ~1 … ~1000 kg/m³ (ρc = 322 sits in the middle).
    let (lo, hi) = if t >= T_C {
        (1.0, RHO_MAX)
    } else {
        // Below Tc: choose branch by saturation pressure.
        let psat_kpa = crate::mpa_to_kpa(crate::region4::psat(t));
        if p_kpa >= psat_kpa {
            (RHO_C, RHO_MAX) // compressed liquid → dense branch
        } else {
            (1.0, RHO_C) // superheated vapor → light branch
        }
    };

    if let Some(rho) = newton_density(t, p_kpa, lo, hi) {
        return Ok(rho);
    }
    // Safety net: if the endpoints did not bracket (a state right on a branch
    // seam, say), fall back to the original scan-and-Brent path.
    let f = |rho: f64| pressure(rho, t) - p_kpa;
    let (a, b) =
        bracket(&f, lo, hi, 64).ok_or(SteamError::NoConvergence("region-3 density bracket"))?;
    brent(&f, a, b, "region-3 density")
}

/// Safeguarded Newton for `pressure(ρ, T) = p` on a bracketing interval.
///
/// Returns `None` if `[lo, hi]` does not bracket a root, leaving the caller to
/// choose a fallback. The safeguard is the standard one: take the Newton step
/// only when it lands inside the current bracket **and** at least halves the
/// previous step, otherwise bisect. That keeps bisection's guaranteed
/// convergence while getting Newton's quadratic rate wherever the isotherm is
/// well-behaved.
fn newton_density(t: f64, p_kpa: f64, lo: f64, hi: f64) -> Option<f64> {
    let f_lo = pressure(lo, t) - p_kpa;
    let f_hi = pressure(hi, t) - p_kpa;
    if f_lo * f_hi > 0.0 || !f_lo.is_finite() || !f_hi.is_finite() {
        return None;
    }
    // An endpoint that is exactly the root is returned as-is — both because it
    // is the answer and because the orientation below needs a strict sign.
    if f_lo == 0.0 {
        return Some(lo);
    }
    if f_hi == 0.0 {
        return Some(hi);
    }
    // Orient the bracket so `xl` is the end where the residual is negative.
    let (mut xl, mut xh) = if f_lo < 0.0 { (lo, hi) } else { (hi, lo) };
    let mut rho = 0.5 * (lo + hi);
    let mut step_prev = (hi - lo).abs();
    let mut step = step_prev;
    let (mut p, mut dp) = pressure_slope(rho, t);
    let mut res = p - p_kpa;
    for _ in 0..80 {
        let newton_out_of_range = ((rho - xh) * dp - res) * ((rho - xl) * dp - res) > 0.0;
        let newton_too_slow = (2.0 * res).abs() > (step_prev * dp).abs();
        step_prev = step;
        if dp == 0.0 || newton_out_of_range || newton_too_slow {
            step = 0.5 * (xh - xl);
            rho = xl + step;
        } else {
            step = res / dp;
            rho -= step;
        }
        if step.abs() < 1e-12 * rho.abs() {
            return Some(rho);
        }
        (p, dp) = pressure_slope(rho, t);
        res = p - p_kpa;
        if res < 0.0 {
            xl = rho;
        } else {
            xh = rho;
        }
    }
    Some(rho)
}

/// Saturated liquid and vapor densities in the near-critical band
/// (623.15 K < T ≤ 647.096 K), where the two-phase boundary lives inside
/// region 3 rather than between regions 1 and 2.
///
/// # Arguments
/// * `t` — Temperature in **K** (must be `623.15 < t ≤ 647.096`).
///
/// # Returns
/// `(ρ_f, ρ_g)` — saturated **liquid** and **vapor** densities in **kg/m³**.
///
/// Both are roots of `pressure(ρ, T) = Psat(T)`; the van-der-Waals loop gives
/// three crossings, and we take the outer (stable) two by scanning the dense
/// `[ρc, 1000]` and light `[1, ρc]` branches separately. This one keeps the
/// scan: unlike [`density_tp`] it is off every measured hot path (it is
/// reached only by saturation queries above 623.15 K), and near `Tc` the three
/// roots crowd together, where a fine scan picks the right pair by
/// construction.
pub(crate) fn saturated_densities(t: f64) -> Result<(f64, f64), SteamError> {
    let p = crate::mpa_to_kpa(crate::region4::psat(t));
    let f = |rho: f64| pressure(rho, t) - p;
    let (a1, b1) = bracket(&f, RHO_C, RHO_MAX, 128)
        .ok_or(SteamError::NoConvergence("region-3 sat liquid bracket"))?;
    let rho_f = brent(&f, a1, b1, "region-3 sat liquid")?;
    let (a2, b2) = bracket(&f, 1.0, RHO_C, 128)
        .ok_or(SteamError::NoConvergence("region-3 sat vapor bracket"))?;
    let rho_g = brent(&f, a2, b2, "region-3 sat vapor")?;
    Ok((rho_f, rho_g))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// R7-97(2012) Table 33 — three verification points, given as `(ρ, T)`.
    #[test]
    fn table_33_verification() {
        // Columns: rho[kg/m³], T[K], p[kPa], h, u, s, cp, w.
        let cases = [
            (
                500.0,
                650.0,
                0.255837018e5,
                0.186343019e4,
                0.181226279e4,
                0.405427273e1,
                0.138935717e2,
                0.502005554e3,
            ),
            (
                200.0,
                650.0,
                0.222930643e5,
                0.237512401e4,
                0.226365868e4,
                0.485438792e1,
                0.446579342e2,
                0.383444594e3,
            ),
            (
                500.0,
                750.0,
                0.783095639e5,
                0.225868845e4,
                0.210206932e4,
                0.446971906e1,
                0.634165359e1,
                0.760696041e3,
            ),
        ];
        for (rho, t, p, h, u, s, cp, w) in cases {
            assert_relative_eq!(pressure(rho, t), p, max_relative = 1e-8);
            let r = props_rho_t(rho, t);
            assert_relative_eq!(r.h, h, max_relative = 1e-8);
            assert_relative_eq!(r.u, u, max_relative = 1e-8);
            assert_relative_eq!(r.s, s, max_relative = 1e-8);
            assert_relative_eq!(r.cp, cp, max_relative = 1e-8);
            assert_relative_eq!(r.w, w, max_relative = 1e-8);
        }
    }

    /// The `(T,p) → ρ` inverse solve must recover the Table-33 density.
    #[test]
    fn density_solve_round_trip() {
        // T=750 K ≥ ... below Tc but far from the dome; ρ=500 → p=78.31 MPa.
        let p = pressure(500.0, 750.0);
        let rho = density_tp(750.0, p).unwrap();
        assert_relative_eq!(rho, 500.0, max_relative = 1e-7);

        // A supercritical point: T=700 K > Tc, ρ=300.
        let p2 = pressure(300.0, 700.0);
        let rho2 = density_tp(700.0, p2).unwrap();
        assert_relative_eq!(rho2, 300.0, max_relative = 1e-7);
    }

    /// Test oracle: the literal `powi`-per-derivative formulation the
    /// power-table [`phi`] replaced.
    fn phi_powi(delta: f64, tau: f64) -> Helmholtz {
        let mut d = Helmholtz {
            f: REGION3_N1 * delta.ln(),
            fd: REGION3_N1 / delta,
            fdd: -REGION3_N1 / (delta * delta),
            ft: 0.0,
            ftt: 0.0,
            fdt: 0.0,
        };
        for term in &REGION3 {
            let (i, j, n) = (term.i, term.j, term.n);
            let (fi, fj) = (i as f64, j as f64);
            d.f += n * delta.powi(i) * tau.powi(j);
            d.fd += n * fi * delta.powi(i - 1) * tau.powi(j);
            d.fdd += n * fi * (fi - 1.0) * delta.powi(i - 2) * tau.powi(j);
            d.ft += n * delta.powi(i) * fj * tau.powi(j - 1);
            d.ftt += n * delta.powi(i) * fj * (fj - 1.0) * tau.powi(j - 2);
            d.fdt += n * fi * delta.powi(i - 1) * fj * tau.powi(j - 1);
        }
        d
    }

    /// The power-table series — and the pressure-only [`phi_delta`] used by the
    /// density solve — must both agree with the `powi` oracle.
    ///
    /// Measured worst case over this grid: **1.27e-12**.
    #[test]
    fn power_table_matches_powi_oracle() {
        for t in [623.15, 650.0, 700.0, 800.0, 863.15] {
            for rho in [1.0, 50.0, 200.0, 322.0, 500.0, 800.0, 1000.0] {
                let (delta, tau) = (rho / RHO_C, T_C / t);
                let (fast, slow) = (phi(delta, tau), phi_powi(delta, tau));
                assert_relative_eq!(fast.f, slow.f, max_relative = 3e-12);
                assert_relative_eq!(fast.fd, slow.fd, max_relative = 3e-12);
                assert_relative_eq!(fast.fdd, slow.fdd, max_relative = 3e-12);
                assert_relative_eq!(fast.ft, slow.ft, max_relative = 3e-12);
                assert_relative_eq!(fast.ftt, slow.ftt, max_relative = 3e-12);
                assert_relative_eq!(fast.fdt, slow.fdt, max_relative = 3e-12);
                let (fd, fdd) = phi_delta(delta, tau);
                assert_relative_eq!(fd, slow.fd, max_relative = 3e-12);
                assert_relative_eq!(fdd, slow.fdd, max_relative = 3e-12);
            }
        }
    }

    /// `∂p/∂ρ|_T` must match a central difference of `pressure` — the standard
    /// house pattern of validating an analytic derivative against a numerical
    /// oracle that never ships in the production path.
    #[test]
    fn pressure_slope_matches_central_difference() {
        for t in [650.0, 700.0, 800.0] {
            for rho in [50.0, 200.0, 500.0, 800.0] {
                let h = rho * 1e-6;
                let fd = (pressure(rho + h, t) - pressure(rho - h, t)) / (2.0 * h);
                let (_, analytic) = pressure_slope(rho, t);
                assert_relative_eq!(analytic, fd, max_relative = 1e-6);
            }
        }
    }

    /// The safeguarded-Newton density solve must reproduce the scan-and-Brent
    /// result it replaced, across both branches and up to the critical point.
    ///
    /// Densities strictly between `ρ_g(T)` and `ρ_f(T)` are skipped below
    /// `Tc`: those sit inside the van-der-Waals loop, where `p(ρ) = p` has
    /// three roots and "the" density is not defined — a two-phase state, not a
    /// region-3 one. (Feeding them in is what makes the two solvers disagree,
    /// and legitimately so: they walk to different roots of the same equation.)
    #[test]
    fn newton_density_matches_scan_and_brent() {
        let mut compared = 0;
        for t in [623.16, 640.0, 647.0, 647.096, 650.0, 700.0, 800.0, 863.15] {
            let dome = (t < T_C).then(|| saturated_densities(t).unwrap());
            for rho_true in [1.0, 10.0, 100.0, 250.0, 322.0, 400.0, 600.0, 900.0] {
                if let Some((rho_f, rho_g)) = dome {
                    if rho_true > rho_g && rho_true < rho_f {
                        continue;
                    }
                }
                let p = pressure(rho_true, t);
                if p <= 0.0 {
                    continue;
                }
                let Ok(fast) = density_tp(t, p) else { continue };
                // The reference path, verbatim from the pre-Newton implementation.
                let f = |rho: f64| pressure(rho, t) - p;
                let (lo, hi) = if t >= T_C {
                    (1.0, RHO_MAX)
                } else {
                    let psat_kpa = crate::mpa_to_kpa(crate::region4::psat(t));
                    if p >= psat_kpa {
                        (RHO_C, RHO_MAX)
                    } else {
                        (1.0, RHO_C)
                    }
                };
                let Some((a, b)) = bracket(&f, lo, hi, 64) else {
                    continue;
                };
                let slow = brent(&f, a, b, "reference").unwrap();
                // The contract both solvers owe: a density that reproduces the
                // target pressure.
                assert_relative_eq!(pressure(fast, t), p, max_relative = 1e-9);
                assert_relative_eq!(pressure(slow, t), p, max_relative = 1e-9);
                // They must also agree on the density itself — except within a
                // whisker of the critical point, where `∂p/∂ρ` and `∂²p/∂ρ²`
                // both vanish and `p − p_target` behaves like `(ρ − ρc)³`. A
                // residual at the 1e-12 level then pins ρ only to ~1e-4, so
                // comparing densities there would assert on the conditioning of
                // the problem rather than on either solver.
                let (_, slope) = pressure_slope(fast, t);
                if slope.abs() > 1.0 {
                    compared += 1;
                    assert_relative_eq!(fast, slow, max_relative = 1e-9);
                }
            }
        }
        // Without this the skip clause above could quietly hollow the test out
        // — every comparison bypassed, and it would still report green.
        assert!(compared >= 30, "only {compared} density comparisons ran");
    }
}
