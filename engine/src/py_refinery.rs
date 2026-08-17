//! PyO3 bindings for the refinery-thermodynamics layer (Milestone 20).
//!
//! The mixture-level entry points live on `vle._engine.System`
//! (`flash_free_water`, `lee_kesler_departure`, `enthalpy_entropy_lee_kesler`,
//! `peneloux_shifts`, `translated_molar_volume`, `translated_density`); this
//! module exposes the **pure / reduced-state** building blocks as free
//! functions so each correlation can be used and taught on its own — the same
//! split `py_petroleum` uses. Units are the crate's canonical ones (K, kPa
//! absolute, cm³/mol); `vle.refinery` is where `pint` quantities are handled.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use crate::eos::{
    ChaoSeaderSpecies, CubicEos, PhaseId, RegularSolutionSet,
    regular_solution_ln_nu as regular_solution_ln_nu_rs,
};
use crate::refinery::{
    RefineryError, lee_kesler_reduced as lee_kesler_reduced_rs,
    peneloux_shift as peneloux_shift_rs, refinery_error_is_input,
};
use crate::types::Component;

fn refinery_err(e: RefineryError) -> PyErr {
    if refinery_error_is_input(&e) {
        PyValueError::new_err(e.to_string())
    } else {
        PyRuntimeError::new_err(e.to_string())
    }
}

fn parse_phase(s: &str) -> PyResult<PhaseId> {
    match s.to_ascii_lowercase().as_str() {
        "vapor" | "v" | "gas" => Ok(PhaseId::Vapor),
        "liquid" | "l" => Ok(PhaseId::Liquid),
        other => Err(PyValueError::new_err(format!(
            "phase must be 'vapor' or 'liquid' (got {other:?})"
        ))),
    }
}

/// Lee–Kesler reduced departure functions at `(tr, pr, omega)` for `phase`
/// (`"vapor"` / `"liquid"`). Returns `(z, h_dep_rt, s_dep_r, ln_phi)`:
/// `Z`, `(H−H°)/(RT)`, `(S−S°)/R` (S° at the same T, P), `ln(f/P)`.
#[pyfunction]
pub fn refinery_lee_kesler_reduced(
    tr: f64,
    pr: f64,
    omega: f64,
    phase: &str,
) -> PyResult<(f64, f64, f64, f64)> {
    let d = lee_kesler_reduced_rs(tr, pr, omega, parse_phase(phase)?).map_err(refinery_err)?;
    Ok((d.z, d.h_dep_rt, d.s_dep_r, d.ln_phi))
}

/// Regular-solution pure-liquid `ln ν` (Chao–Seader / Grayson–Streed) at `t`
/// [K], `p` [kPa abs] for a component with `tc` [K], `pc` [kPa], `omega`.
/// `set` selects the coefficient table (`RegularSolutionSet.GraysonStreed1963`
/// or `.ChaoSeader1961`), `species` the fluid class.
#[pyfunction]
#[pyo3(signature = (t, p, tc, pc, omega, set=RegularSolutionSet::GraysonStreed1963,
    species=ChaoSeaderSpecies::Normal))]
pub fn regular_solution_ln_nu(
    t: f64,
    p: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    set: RegularSolutionSet,
    species: ChaoSeaderSpecies,
) -> f64 {
    let comp = Component {
        tc,
        pc,
        omega,
        ..Component::default()
    };
    regular_solution_ln_nu_rs(set, t, p, &comp, species)
}

/// Peneloux volume shift `c` in **cm³/mol** for a component with `tc` [K],
/// `pc` [kPa], `omega` under `eos` (SRK or PR family). `zra = 0` uses the
/// `0.29056 − 0.08775·ω` correlation.
#[pyfunction]
#[pyo3(signature = (eos, tc, pc, omega, zra=0.0))]
pub fn refinery_peneloux_shift(
    eos: CubicEos,
    tc: f64,
    pc: f64,
    omega: f64,
    zra: f64,
) -> PyResult<f64> {
    let comp = Component {
        tc,
        pc,
        omega,
        zra,
        ..Component::default()
    };
    peneloux_shift_rs(eos, &comp).map_err(refinery_err)
}

/// Register everything in this module into `vle._engine`.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RegularSolutionSet>()?;
    m.add_function(wrap_pyfunction!(refinery_lee_kesler_reduced, m)?)?;
    m.add_function(wrap_pyfunction!(regular_solution_ln_nu, m)?)?;
    m.add_function(wrap_pyfunction!(refinery_peneloux_shift, m)?)?;
    Ok(())
}
