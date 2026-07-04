//! Adiabatic (PH) flash — Milestone 9, §M.
//!
//! Given a feed of composition `z` at pressure `P` with a known **specific
//! enthalpy** `H_feed`, find the temperature `T` (and the resulting phase
//! split) at which the flashed stream's enthalpy equals `H_feed`. This is
//! the energy-balance flash behind an adiabatic throttle or heater: the
//! pressure and enthalpy are fixed, temperature is the unknown.
//!
//! ## Algorithm (§M)
//!
//! A warm-started nested loop:
//! - **Outer** — bisection on `T` to drive the enthalpy residual
//!   `g(T) = H_mix(T) − H_feed → 0`. `H_mix` increases monotonically with
//!   `T` (positive heat capacity), so a bracketed bisection converges
//!   reliably.
//! - **Inner** — an isothermal flash at each trial `T`, **warm-started**
//!   with the previous temperature's converged K-values (§M's improvement
//!   over flashing from cold every time).
//!
//! The stream enthalpy is the phase-fraction-weighted sum of the liquid and
//! vapor molar enthalpies from the energy layer (ideal + EOS departure).
//! Restricted to the φ-φ (cubic both phases) path — the γ-φ liquid enthalpy
//! goes through the excess/condensation route, which is a later addition.
//!
//! # References
//! - (28) Michelsen — state-function-based flash (the optional simultaneous
//!   upgrade; this module keeps the robust nested form)

use super::FlashError;
use super::isothermal::{FlashResult, flash_isothermal_warm};
use super::system::SystemSpec;
use crate::energy::phase_enthalpy_entropy;
use crate::eos::{CubicEos, LiquidModel, PhaseId, VaporModel};

/// Result of an adiabatic flash: the found temperature plus the phase split.
#[derive(Debug, Clone, PartialEq)]
pub struct AdiabaticResult {
    /// Flash temperature in **K**.
    pub t: f64,
    /// The isothermal-flash phase split at `t`.
    pub flash: FlashResult,
    /// The stream enthalpy at the solution in **kJ/kmol** (≈ `H_feed`).
    pub enthalpy: f64,
}

/// Extract the (liquid EOS, vapor EOS) for the φ-φ cubic path, or error.
fn cubic_pair(spec: &SystemSpec) -> Result<(CubicEos, CubicEos), FlashError> {
    match (spec.liquid, spec.vapor) {
        (LiquidModel::Cubic(le), VaporModel::Cubic(ve)) => Ok((le, ve)),
        _ => Err(FlashError::Unsupported(
            "adiabatic flash currently requires a cubic EOS for both phases (φ-φ)".into(),
        )),
    }
}

/// Stream molar enthalpy at `(t, p)` for a converged flash split, in
/// **kJ/kmol**: `β·H_vapor(y) + (1−β)·H_liquid(x)` (ideal + EOS departure).
fn stream_enthalpy(
    spec: &SystemSpec,
    t: f64,
    p: f64,
    res: &FlashResult,
    t_ref: f64,
    p_ref: f64,
) -> Result<f64, FlashError> {
    let (le, ve) = cubic_pair(spec)?;
    let to_thermo = |e: crate::mixture::MixError| FlashError::Thermo(e.to_string());
    let (h_liq, _) = phase_enthalpy_entropy(
        &spec.mixture_spec(le),
        t,
        p,
        &res.x,
        PhaseId::Liquid,
        t_ref,
        p_ref,
        &[],
        &[],
    )
    .map_err(to_thermo)?;
    let (h_vap, _) = phase_enthalpy_entropy(
        &spec.mixture_spec(ve),
        t,
        p,
        &res.y,
        PhaseId::Vapor,
        t_ref,
        p_ref,
        &[],
        &[],
    )
    .map_err(to_thermo)?;
    Ok(res.beta * h_vap + (1.0 - res.beta) * h_liq)
}

/// Adiabatic (PH) flash: find `T` such that the flashed stream enthalpy at
/// `(T, p)` equals `h_feed`.
///
/// # Arguments
/// * `spec` — the mixture model (φ-φ cubic both phases).
/// * `p` — Pressure in **kPa absolute**.
/// * `z` — feed mole fractions (length N, sum to 1).
/// * `h_feed` — target specific enthalpy in **kJ/kmol** (same reference as
///   `t_ref`/`p_ref`).
/// * `t_ref`, `p_ref` — the ideal-gas enthalpy reference (**K**, **kPa**).
/// * `t_lo`, `t_hi` — temperature bracket in **K** to search within.
/// * `tol` — tolerance on `|H_mix − H_feed|` in **kJ/kmol**.
/// * `max_iter` — outer bisection cap.
///
/// # Returns
/// [`AdiabaticResult`] with the flash temperature and the phase split.
///
/// # Errors
/// [`FlashError::Unsupported`] for a non-cubic system;
/// [`FlashError::NoConvergence`] if `h_feed` is not bracketed by
/// `[H(t_lo), H(t_hi)]` or the cap is hit; [`FlashError::Thermo`] on a
/// thermodynamic failure.
#[allow(clippy::too_many_arguments)]
pub fn flash_adiabatic(
    spec: &SystemSpec,
    p: f64,
    z: &[f64],
    h_feed: f64,
    t_ref: f64,
    p_ref: f64,
    t_lo: f64,
    t_hi: f64,
    tol: f64,
    max_iter: usize,
) -> Result<AdiabaticResult, FlashError> {
    let n = spec.n();
    if z.len() != n {
        return Err(FlashError::Dimension(format!(
            "components={n}, z={}",
            z.len()
        )));
    }
    cubic_pair(spec)?; // fail fast on an unsupported model

    // Warm-start K carried across temperatures.
    let mut warm: Option<Vec<f64>> = None;
    // Enthalpy residual + flash at a trial T (updates the warm start).
    let eval = |t: f64, warm: &mut Option<Vec<f64>>| -> Result<(f64, FlashResult), FlashError> {
        let res = flash_isothermal_warm(spec, t, p, z, warm.as_deref(), 1e-10, 200)?;
        *warm = Some(res.k.clone());
        let h = stream_enthalpy(spec, t, p, &res, t_ref, p_ref)?;
        Ok((h - h_feed, res))
    };

    let (mut g_lo, _) = eval(t_lo, &mut warm)?;
    let (mut g_hi, _) = eval(t_hi, &mut warm)?;
    if g_lo * g_hi > 0.0 {
        return Err(FlashError::NoConvergence {
            what: "adiabatic flash bracket (H_feed not within [H(t_lo), H(t_hi)])",
            iters: 0,
            residual: g_lo.abs().min(g_hi.abs()),
        });
    }
    let _ = &mut g_hi;

    let (mut lo, mut hi) = (t_lo, t_hi);
    for iter in 0..max_iter {
        let mid = 0.5 * (lo + hi);
        let (g_mid, res) = eval(mid, &mut warm)?;
        if g_mid.abs() < tol || (hi - lo) < 1e-8 {
            let h = g_mid + h_feed;
            return Ok(AdiabaticResult {
                t: mid,
                flash: res,
                enthalpy: h,
            });
        }
        if g_mid * g_lo > 0.0 {
            lo = mid;
            g_lo = g_mid;
        } else {
            hi = mid;
        }
        if iter + 1 == max_iter {
            return Err(FlashError::NoConvergence {
                what: "adiabatic flash",
                iters: max_iter,
                residual: g_mid.abs(),
            });
        }
    }
    unreachable!("loop returns via convergence or NoConvergence")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flash::isothermal::flash_isothermal;
    use crate::mixing::MixingRule;
    use crate::types::Component;

    // n-pentane / n-decane — a wide-boiling pair with a broad two-phase
    // band (a close-boiling pair like benzene/cyclohexane would give a
    // sub-degree band, useless for an energy-balance test). Plausible
    // ideal-Cp/R polynomials.
    fn n_pentane() -> Component {
        Component {
            name: "n-pentane".into(),
            tc: 469.7,
            pc: 3370.0,
            omega: 0.252,
            cp_coeffs: [1.5, 4.0e-2, -1.2e-5, 0.0, 0.0],
            ..Component::default()
        }
    }

    fn n_decane() -> Component {
        Component {
            name: "n-decane".into(),
            tc: 617.7,
            pc: 2110.0,
            omega: 0.4884,
            cp_coeffs: [2.0, 8.0e-2, -2.4e-5, 0.0, 0.0],
            ..Component::default()
        }
    }

    fn pr(components: &[Component]) -> SystemSpec<'_> {
        SystemSpec {
            components,
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
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
    fn adiabatic_recovers_the_temperature_of_a_known_enthalpy() {
        // Round-trip: compute the stream enthalpy at a known T*, then ask
        // the adiabatic flash to recover T* from that enthalpy. T* = 450 K
        // at 500 kPa is mid-band (β ≈ 0.5).
        let comps = [n_pentane(), n_decane()];
        let spec = pr(&comps);
        let z = [0.5, 0.5];
        let (p, t_star) = (500.0, 450.0);
        let res_star = flash_isothermal(&spec, t_star, p, &z, 1e-10, 200).unwrap();
        assert!(res_star.two_phase, "reference point should be two-phase");
        let h_star = stream_enthalpy(&spec, t_star, p, &res_star, 298.15, 101.325).unwrap();

        let out = flash_adiabatic(
            &spec, p, &z, h_star, 298.15, 101.325, 400.0, 500.0, 1e-4, 200,
        )
        .unwrap();
        assert!(
            (out.t - t_star).abs() < 0.05,
            "recovered T={} vs true {t_star}",
            out.t
        );
        assert!((out.enthalpy - h_star).abs() < 1e-3);
    }

    #[test]
    fn adiabatic_enthalpy_matches_target() {
        let comps = [n_pentane(), n_decane()];
        let spec = pr(&comps);
        let z = [0.4, 0.6];
        // Pick a target H between the bracket endpoints' enthalpies.
        let lo = flash_isothermal(&spec, 420.0, 500.0, &z, 1e-10, 200).unwrap();
        let hi = flash_isothermal(&spec, 480.0, 500.0, &z, 1e-10, 200).unwrap();
        let h_lo = stream_enthalpy(&spec, 420.0, 500.0, &lo, 298.15, 101.325).unwrap();
        let h_hi = stream_enthalpy(&spec, 480.0, 500.0, &hi, 298.15, 101.325).unwrap();
        let target = 0.5 * (h_lo + h_hi);
        let out = flash_adiabatic(
            &spec, 500.0, &z, target, 298.15, 101.325, 420.0, 480.0, 1e-4, 200,
        )
        .unwrap();
        assert!((out.enthalpy - target).abs() < 1e-3);
        assert!(out.t > 420.0 && out.t < 480.0);
    }

    #[test]
    fn adiabatic_rejects_non_cubic_system() {
        let comps = [n_pentane(), n_decane()];
        let mut spec = pr(&comps);
        spec.liquid = LiquidModel::IdealSolution;
        assert!(matches!(
            flash_adiabatic(
                &spec,
                300.0,
                &[0.5, 0.5],
                0.0,
                298.15,
                101.325,
                300.0,
                400.0,
                1e-4,
                100
            ),
            Err(FlashError::Unsupported(_))
        ));
    }
}
