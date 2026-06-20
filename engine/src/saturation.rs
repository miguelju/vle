//! Saturation pressure models.
//!
//! Saturation (vapor) pressure Psat(T) is the pressure at which a pure component
//! boils at temperature T. These correlations are essential for:
//!
//! 1. **Initial K-value estimates** in flash calculations — Ki ≈ Psat_i(T) / P_system
//!    gives a starting point for iterative convergence.
//! 2. **Bubble/dew point initialization** — correlating Psat across components
//!    gives initial temperature or pressure guesses.
//! 3. **Validation** — comparing EOS-predicted saturation with correlation values
//!    confirms the EOS parameterization is correct.
//!
//! The models range from simple empirical (Antoine, 3 parameters) to thermodynamically
//! consistent (Maxwell equal-area, derived directly from the EOS).
//!
//! # References
//! - (4) Da Silva & Báez (1989) — Antoine correlation

/// Saturation (vapor) pressure correlation model.
///
/// Used to estimate pure-component saturation pressure Psat(T) in **kPa**
/// from temperature in **K**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum SatPressureModel {
    /// Antoine equation: ln(P/Pc) = a₁ - a₂/(a₃ + T).
    /// Simple 3-parameter correlation, widely tabulated. Accurate over limited
    /// temperature ranges. Ref (4), legacy/pascal/TERMOI.PAS.
    Antoine = 0,
    /// Riedel correlation. Extended corresponding-states method using Tc, Pc, ω.
    /// Better extrapolation than Antoine over wider temperature ranges.
    Riedel = 1,
    /// Müller correlation. Alternative reduced-property correlation.
    Muller = 2,
    /// Reduced-pressure model (RPM). Corresponding-states correlation using
    /// reduced properties (Tr, Pr).
    RPM = 3,
    /// Database polynomial. Coefficients from external property database (e.g., DIPPR).
    /// P = exp(A + B/T + C·ln(T) + D·T^E) or similar fitted form.
    Polynomial = 4,
    /// Maxwell equal-area construction. Thermodynamically exact — finds the pressure
    /// where the integral of (V dP) over the van der Waals loop equals zero.
    /// Requires a cubic EOS and iterative solution. Slowest but most consistent.
    Maxwell = 5,
}

// ===========================================================================
// Saturation-pressure correlations.
//
// Antoine shipped in M7.1; the corresponding-states models (Riedel, Müller,
// RPM), the DIPPR-style polynomial, the Maxwell equal-area construction, and
// the boiling-point / Poynting / pseudo-Antoine helpers shipped in M7.4.
// `psat` dispatches the non-Maxwell models; Maxwell needs a cubic EOS
// (`psat_maxwell`). Ref (4): Da Silva & Báez (1989).
// ===========================================================================

use crate::eos::{CubicEos, PhaseId, ln_phi_pure};
use crate::types::Component;
use thiserror::Error;

/// Errors raised by the saturation-pressure layer.
#[derive(Debug, Error, PartialEq)]
pub enum SatError {
    /// The component is missing the `psat_coeffs` vector or it has the
    /// wrong length for the selected model (Antoine requires exactly 3).
    #[error("component {name:?}: expected {expected} Antoine coefficients, got {got}")]
    BadCoefficients {
        name: String,
        expected: usize,
        got: usize,
    },
    /// The model is not reachable through this entry point (e.g. `Maxwell`
    /// via the EOS-free `psat` / `d_psat_dt` dispatch — use `psat_maxwell`).
    #[error("saturation model {0:?} not available through this entry point")]
    NotImplemented(SatPressureModel),
    /// The Maxwell equal-area construction failed (supercritical T, or no
    /// convergence / no two-phase roots).
    #[error("Maxwell construction failed: {0}")]
    Maxwell(String),
    /// Temperature is outside a physically meaningful range (≤ 0 K, or
    /// the Antoine denominator a3 + T is non-positive).
    #[error("temperature {0} K out of range for saturation correlation")]
    OutOfRange(f64),
}

/// Antoine vapor pressure: `ln(P_sat/Pc) = a1 − a2/(a3 + T)`.
///
/// # Arguments
/// * `comp` — Component (uses `pc` and `psat_coeffs`).
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Saturation pressure in **kPa absolute**.
///
/// # Errors
/// `BadCoefficients` if `psat_coeffs` does not have exactly 3 entries.
/// `OutOfRange` if `a3 + T ≤ 0` (would produce inf/NaN in the exp).
///
/// # Source
/// Reference (4): Da Silva & Báez (1989), `legacy/pascal/TERMOI.PAS`. The
/// form `ln(P/Pc) = a1 − a2/(a3 + T)` is the "reduced" Antoine the
/// Pascal program uses — coefficients are tabulated against the
/// component's own Pc, not against 1 atm. Some external Antoine tables
/// use `log10(P) = A − B/(C + T)` with a different sign on `C`; convert
/// before calling this function.
pub fn psat_antoine(comp: &Component, t: f64) -> Result<f64, SatError> {
    if comp.psat_coeffs.len() != 3 {
        return Err(SatError::BadCoefficients {
            name: comp.name.clone(),
            expected: 3,
            got: comp.psat_coeffs.len(),
        });
    }
    let a1 = comp.psat_coeffs[0];
    let a2 = comp.psat_coeffs[1];
    let a3 = comp.psat_coeffs[2];
    let denom = a3 + t;
    if denom <= 0.0 {
        return Err(SatError::OutOfRange(t));
    }
    Ok(comp.pc * (a1 - a2 / denom).exp())
}

/// Analytical derivative `dPsat/dT` for the Antoine form.
///
/// `Psat = Pc · exp(a1 − a2/(a3 + T))`
/// `dPsat/dT = Psat · a2 / (a3 + T)²`
///
/// Returns `dPsat/dT` in **kPa/K**.
pub fn d_psat_dt_antoine(comp: &Component, t: f64) -> Result<f64, SatError> {
    let psat = psat_antoine(comp, t)?;
    let a2 = comp.psat_coeffs[1];
    let a3 = comp.psat_coeffs[2];
    let denom = a3 + t;
    Ok(psat * a2 / (denom * denom))
}

/// One standard atmosphere in **kPa** — the reference pressure baked into the
/// corresponding-states saturation correlations (the legacy used Pc/1.0135 bar
/// = Pc in atm; we use the exact 101.325 kPa).
const ATM_KPA: f64 = 101.325;

/// Riedel corresponding-states saturation pressure. Ref (4), TERMOI.PAS:161.
///
/// `ln(Psat/Pc)` from the Riedel criterion using `Tc`, `Pc`, and the normal
/// boiling point `Tb`. Returns Psat in **kPa**. Requires `comp.tb > 0`.
pub fn psat_riedel(comp: &Component, t: f64) -> Result<f64, SatError> {
    if comp.tb <= 0.0 || comp.tc <= 0.0 || comp.pc <= 0.0 || t <= 0.0 {
        return Err(SatError::OutOfRange(t));
    }
    let trb = comp.tb / comp.tc;
    let aux = -35.0 + 36.0 / trb + 42.0 * trb.ln() - trb.powi(6);
    // ln(Pc/1 atm) = ln(Pc in atm).
    let q = (0.315 * aux + (comp.pc / ATM_KPA).ln()) / (0.0838 * aux - trb.ln());
    let c1 = 0.0838 * (3.758 - q);
    let tr = t / comp.tc;
    let ln_pr = (-35.0 + 36.0 / tr) * c1 + (42.0 * c1 + q) * tr.ln() - c1 * tr.powi(6);
    Ok(comp.pc * ln_pr.exp())
}

/// Müller corresponding-states saturation pressure. Ref (4), TERMOI.PAS:177.
///
/// `ln(Psat/Pc)` from `Tc`, `Pc`, `Tb`, and ω. The legacy's `0.0134 − ln(Pc_bar)`
/// is exactly `ln(1 atm / Pc)`, written here unit-cleanly. Returns Psat in **kPa**.
pub fn psat_muller(comp: &Component, t: f64) -> Result<f64, SatError> {
    if comp.tb <= 0.0 || comp.tc <= 0.0 || comp.pc <= 0.0 || t <= 0.0 {
        return Err(SatError::OutOfRange(t));
    }
    let trb = comp.tb / comp.tc;
    let mut a = 5.37273 * (1.0 + comp.omega);
    let b = ((ATM_KPA / comp.pc).ln() - a * (1.0 - 1.0 / trb))
        / (trb.ln() - 0.832223 * (1.0 - 1.0 / trb));
    a -= 0.832223 * b;
    let tr = t / comp.tc;
    let ln_pr = a * (1.0 - 1.0 / tr) + b * tr.ln();
    Ok(comp.pc * ln_pr.exp())
}

/// Riedel-Plank-Miller (RPM) saturation pressure. Ref (4), TERMOI.PAS:169.
/// Returns Psat in **kPa**.
pub fn psat_rpm(comp: &Component, t: f64) -> Result<f64, SatError> {
    if comp.tb <= 0.0 || comp.tc <= 0.0 || comp.pc <= 0.0 || t <= 0.0 {
        return Err(SatError::OutOfRange(t));
    }
    let trb = comp.tb / comp.tc;
    let x = (comp.pc / ATM_KPA).ln() * trb / (1.0 - trb);
    let c1 = 0.4835 + 0.4605 * x;
    let g = (x / c1 - (1.0 + trb)) / ((3.0 + trb) * (1.0 - trb).powi(2));
    let tr = t / comp.tc;
    let ln_pr = -c1 * (g * (3.0 + tr) * (1.0 - tr).powi(3) - tr * tr + 1.0) / tr;
    Ok(comp.pc * ln_pr.exp())
}

/// Generic fitted (DIPPR-101-style) saturation polynomial:
/// `ln(Psat[kPa]) = c0 + c1/T + c2·ln(T) + c3·T^c4`, with `psat_coeffs =
/// [c0, c1, c2, c3, c4]`. A flexible stand-in for database correlations
/// (`SatPressureModel::Polynomial`); not a specific legacy formula.
pub fn psat_polynomial(comp: &Component, t: f64) -> Result<f64, SatError> {
    if comp.psat_coeffs.len() != 5 {
        return Err(SatError::BadCoefficients {
            name: comp.name.clone(),
            expected: 5,
            got: comp.psat_coeffs.len(),
        });
    }
    if t <= 0.0 {
        return Err(SatError::OutOfRange(t));
    }
    let c = &comp.psat_coeffs;
    let ln_p = c[0] + c[1] / t + c[2] * t.ln() + c[3] * t.powf(c[4]);
    Ok(ln_p.exp())
}

/// Maxwell equal-area construction: the thermodynamically exact saturation
/// pressure for a cubic EOS at temperature `t` — the pressure where the
/// liquid and vapor roots have equal fugacity. Ref (4), clsQbicsPure.cls:631.
///
/// Solved by successive substitution `P ← P·exp(ln φ_liq − ln φ_vap)` from an
/// initial guess (Antoine if the component carries coefficients, else a
/// Clausius-Clapeyron-style estimate). Returns Psat in **kPa**.
///
/// # Errors
/// `Maxwell` if the iteration fails to find a pressure with both a liquid and
/// a vapor root (e.g. supercritical `t`) within the iteration budget.
pub fn psat_maxwell(eos: CubicEos, comp: &Component, t: f64) -> Result<f64, SatError> {
    if comp.tc <= 0.0 || comp.pc <= 0.0 || t <= 0.0 {
        return Err(SatError::OutOfRange(t));
    }
    // Initial pressure estimate.
    let mut p = if comp.psat_coeffs.len() == 3 {
        psat_antoine(comp, t).unwrap_or(comp.pc)
    } else {
        // Clausius-Clapeyron-ish: ln(Pr) ≈ 5.37(1+ω)(1 − Tc/T).
        comp.pc * (5.37 * (1.0 + comp.omega) * (1.0 - comp.tc / t)).exp()
    };
    for _ in 0..100 {
        let lnphi_l = ln_phi_pure(eos, t, p, comp, PhaseId::Liquid)
            .map_err(|e| SatError::Maxwell(e.to_string()))?;
        let lnphi_v = ln_phi_pure(eos, t, p, comp, PhaseId::Vapor)
            .map_err(|e| SatError::Maxwell(e.to_string()))?;
        let step = lnphi_l - lnphi_v;
        let p_new = p * step.exp();
        if !(p_new.is_finite() && p_new > 0.0) {
            return Err(SatError::Maxwell(format!("non-finite P at T={t}")));
        }
        if ((p_new - p) / p_new).abs() < 1e-9 {
            return Ok(p_new);
        }
        p = p_new;
    }
    Err(SatError::Maxwell(format!("no convergence at T={t}")))
}

/// Reduced saturation pressure `Psat(T)/Pc` for the given model. Used by the
/// OL-family α (which reads the component's `sat_model`); also handy directly.
/// Dimensionless. For `Maxwell`, requires an EOS — call [`psat_maxwell`] instead.
pub fn reduced_psat(model: SatPressureModel, comp: &Component, t: f64) -> Result<f64, SatError> {
    Ok(psat(model, comp, t)? / comp.pc)
}

/// Generic `dPsat/dT` in **kPa/K**. Analytical for Antoine; central-difference
/// for the corresponding-states correlations (matching the legacy
/// `DPrVapor_DT`, TERMOI.PAS:236). `Maxwell` is not supported here.
pub fn d_psat_dt(model: SatPressureModel, comp: &Component, t: f64) -> Result<f64, SatError> {
    match model {
        SatPressureModel::Antoine => d_psat_dt_antoine(comp, t),
        SatPressureModel::Maxwell => Err(SatError::NotImplemented(SatPressureModel::Maxwell)),
        other => {
            let h = 1e-3 * t.max(1.0);
            let hi = psat(other, comp, t + h)?;
            let lo = psat(other, comp, t - h)?;
            Ok((hi - lo) / (2.0 * h))
        }
    }
}

/// Boiling temperature in **K**: invert `Psat(T) = P`. Closed form for Antoine;
/// Brent's method on `psat(T) − P` over `[0.3·Tc, Tc]` for the others.
/// Ref (4), TERMOI.PAS:208 (`TEbullicion`).
pub fn boiling_temperature(
    model: SatPressureModel,
    comp: &Component,
    p: f64,
) -> Result<f64, SatError> {
    if comp.tc <= 0.0 || comp.pc <= 0.0 || p <= 0.0 {
        return Err(SatError::OutOfRange(p));
    }
    if model == SatPressureModel::Antoine {
        // T = a2/(a1 − ln(P/Pc)) − a3.
        if comp.psat_coeffs.len() != 3 {
            return Err(SatError::BadCoefficients {
                name: comp.name.clone(),
                expected: 3,
                got: comp.psat_coeffs.len(),
            });
        }
        let (a1, a2, a3) = (
            comp.psat_coeffs[0],
            comp.psat_coeffs[1],
            comp.psat_coeffs[2],
        );
        let denom = a1 - (p / comp.pc).ln();
        if denom.abs() < 1e-300 {
            return Err(SatError::OutOfRange(p));
        }
        return Ok(a2 / denom - a3);
    }
    // Bracketed solve for the corresponding-states correlations.
    let f = |tt: f64| psat(model, comp, tt).map(|ps| ps - p).unwrap_or(f64::NAN);
    crate::numerics::root_finding::brent(f, 0.3 * comp.tc, comp.tc, 1e-6, 200)
        .map_err(|e| SatError::Maxwell(format!("boiling-point solve: {e}")))
}

/// Poynting correction factor `exp[V_L·(P − Psat) / (R·T)]` — the pressure
/// correction on a liquid's fugacity above its saturation pressure.
/// Ref (4), TERMOI.PAS:149 (`CorrectPoynting`).
///
/// # Arguments
/// * `comp` — uses `liquid_volume` (V_L in **cm³/mol**).
/// * `p`, `psat` — pressures in **kPa**; `t` — temperature in **K**.
///
/// # Returns
/// Dimensionless Poynting factor. (Unit factor 1e-3 converts cm³·kPa to J;
/// R = 8.31451 J/(mol·K).)
pub fn poynting_factor(comp: &Component, p: f64, psat: f64, t: f64) -> f64 {
    const R: f64 = 8.31451; // J/(mol·K) = kJ/(kmol·K)
    (comp.liquid_volume * (p - psat) * 1e-3 / (R * t)).exp()
}

/// Local "pseudo-Antoine" fit: three Antoine coefficients `[a1, a2, a3]` that
/// reproduce a non-Antoine model's `ln(Psat/Pc)` at `t_ref` and `t_ref ± range`.
/// Ref (4), TERMOI.PAS:191 (`PseudoAntoine`) — used by the legacy boiling-point
/// inversion; exposed here for callers that want a cheap local Antoine surrogate.
pub fn pseudo_antoine(
    model: SatPressureModel,
    comp: &Component,
    t_ref: f64,
    range: f64,
) -> Result<[f64; 3], SatError> {
    let t1 = t_ref - range;
    let t3 = t_ref + range;
    let x1 = (psat(model, comp, t1)? / comp.pc).ln();
    let x2 = (psat(model, comp, t_ref)? / comp.pc).ln();
    let x3 = (psat(model, comp, t3)? / comp.pc).ln();
    let teta = (t1 - t_ref) / (t_ref - t3);
    let gama = (x1 - x2) / (x2 - x3);
    let a3 = -(teta * t3 - gama * t1) / (teta - gama);
    let a2 = (x2 - x3) / (1.0 / (t3 + a3) - 1.0 / (t_ref + a3));
    let a1 = x2 + a2 / (t_ref + a3);
    Ok([a1, a2, a3])
}

/// Generic dispatch: compute saturation pressure for any (non-Maxwell) model.
///
/// `Maxwell` needs a cubic EOS, so it is not reachable through this
/// model-only entry point — call [`psat_maxwell`] directly.
pub fn psat(model: SatPressureModel, comp: &Component, t: f64) -> Result<f64, SatError> {
    match model {
        SatPressureModel::Antoine => psat_antoine(comp, t),
        SatPressureModel::Riedel => psat_riedel(comp, t),
        SatPressureModel::Muller => psat_muller(comp, t),
        SatPressureModel::RPM => psat_rpm(comp, t),
        SatPressureModel::Polynomial => psat_polynomial(comp, t),
        SatPressureModel::Maxwell => Err(SatError::NotImplemented(SatPressureModel::Maxwell)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos::CubicEos;

    fn pentane() -> Component {
        // n-pentane with the data the saturation layer needs: a reduced
        // Antoine fit (ln(P/Pc)=a1−a2/(a3+T)), normal boiling point, V_L.
        Component {
            name: "n-pentane".into(),
            tc: 469.7,
            pc: 3370.0,
            omega: 0.252,
            tb: 309.2,
            psat_coeffs: vec![6.738, 3165.0, 0.0],
            liquid_volume: 116.0,
            ..Component::default()
        }
    }

    const CORRELATIONS: [SatPressureModel; 3] = [
        SatPressureModel::Riedel,
        SatPressureModel::Muller,
        SatPressureModel::RPM,
    ];

    #[test]
    fn correlations_finite_and_subcritical() {
        let c = pentane();
        for model in CORRELATIONS {
            let ps = psat(model, &c, 350.0).unwrap();
            assert!(ps.is_finite() && ps > 0.0 && ps < c.pc, "{model:?} ps={ps}");
            assert!(reduced_psat(model, &c, 350.0).unwrap() < 1.0, "{model:?}");
        }
    }

    #[test]
    fn correlations_hit_one_atm_at_boiling_point() {
        // By construction every corresponding-states fit gives ~1 atm at Tb.
        let c = pentane();
        for model in CORRELATIONS {
            let ps = psat(model, &c, c.tb).unwrap();
            assert!(
                (ps - ATM_KPA).abs() / ATM_KPA < 0.05,
                "{model:?} ps@Tb={ps}"
            );
        }
    }

    #[test]
    fn d_psat_dt_matches_numerical() {
        let c = pentane();
        let t = 350.0;
        for model in [
            SatPressureModel::Antoine,
            SatPressureModel::Riedel,
            SatPressureModel::Muller,
            SatPressureModel::RPM,
        ] {
            let analytical = d_psat_dt(model, &c, t).unwrap();
            let h = 1e-2;
            let num =
                (psat(model, &c, t + h).unwrap() - psat(model, &c, t - h).unwrap()) / (2.0 * h);
            assert!(
                ((analytical - num) / analytical).abs() < 1e-3,
                "{model:?} a={analytical} n={num}"
            );
        }
    }

    #[test]
    fn polynomial_dippr_form() {
        let mut c = pentane();
        c.psat_coeffs = vec![10.0, -3000.0, 0.0, 0.0, 0.0]; // ln P = 10 − 3000/T
        let ps = psat_polynomial(&c, 350.0).unwrap();
        assert!((ps - (10.0 - 3000.0 / 350.0_f64).exp()).abs() < 1e-6);
    }

    #[test]
    fn maxwell_in_antoine_ballpark() {
        let c = pentane();
        let pm = psat_maxwell(CubicEos::PR1976, &c, 350.0).unwrap();
        let pa = psat_antoine(&c, 350.0).unwrap();
        assert!(pm.is_finite() && pm > 0.0, "maxwell={pm}");
        assert!((pm / pa).ln().abs() < 1.0, "maxwell={pm} antoine={pa}");
    }

    #[test]
    fn boiling_point_round_trips() {
        let c = pentane();
        let p = 200.0;
        let tb_a = boiling_temperature(SatPressureModel::Antoine, &c, p).unwrap();
        assert!((psat_antoine(&c, tb_a).unwrap() - p).abs() / p < 1e-6);
        let tb_r = boiling_temperature(SatPressureModel::Riedel, &c, p).unwrap();
        assert!((psat(SatPressureModel::Riedel, &c, tb_r).unwrap() - p).abs() / p < 1e-4);
    }

    #[test]
    fn poynting_unity_at_saturation_and_grows() {
        let c = pentane();
        assert!((poynting_factor(&c, 500.0, 500.0, 350.0) - 1.0).abs() < 1e-12);
        assert!(poynting_factor(&c, 2000.0, 500.0, 350.0) > 1.0);
    }

    #[test]
    fn pseudo_antoine_reproduces_model_at_ref() {
        let c = pentane();
        let [a1, a2, a3] = pseudo_antoine(SatPressureModel::Riedel, &c, 350.0, 5.0).unwrap();
        let lp_fit = a1 - a2 / (a3 + 350.0);
        let lp_true = (psat(SatPressureModel::Riedel, &c, 350.0).unwrap() / c.pc).ln();
        assert!((lp_fit - lp_true).abs() < 1e-6);
    }
}
