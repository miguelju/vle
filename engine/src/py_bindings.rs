//! Python bindings for the VLE engine.
//!
//! Builds a Python extension module `vle._engine` when this crate is
//! compiled with the `python` feature. End-users import it as
//! `from vle import _engine` (or, more commonly, via the higher-level
//! `vle` Python wrapper that re-exports the pieces they need).
//!
//! ## What lives here in Milestone 5
//!
//! This is the first PyO3 module the crate ships. The Milestone 5 goal is
//! to prove the boundary end-to-end (wheel builds, installs, imports,
//! tests pass on every platform in CI). To that end this module exposes
//! a small but useful surface:
//!
//! - [`version()`] — a free function returning the crate's semver string.
//!   Lets calling Python code introspect which version of the engine is
//!   loaded.
//! - Four enum classes — [`CubicEos`], [`ActivityModel`], [`MixingRule`],
//!   [`SatPressureModel`]. These are the model-selection types Python
//!   callers will pass to every subsequent calculation in M6+. Exposing
//!   them now means every later milestone can wire new functions in
//!   without first having to set up enum bindings.
//!
//! ## Adding more bindings in M6+
//!
//! Per CLAUDE.md's "PyO3 Bindings Rule (M5+)", every milestone after M5
//! that adds public Rust functionality must also expose it via PyO3 in
//! the same commit series. The convention:
//!
//! - **Free functions**: `#[pyfunction]` in this file or a co-located
//!   module file (e.g., `engine/src/py_bindings/numerics.rs`); register
//!   in the `#[pymodule]` block below via `wrap_pyfunction!`.
//! - **Types**: add `#[cfg_attr(feature = "python", pyo3::pyclass(...))]`
//!   to the Rust struct/enum definition where it already lives, then
//!   register here via `m.add_class::<T>()`.
//! - **Tests**: every new binding gets at least one round-trip test in
//!   `python/tests/test_engine.py` (or a sibling file).
//!
//! The CI matrix exercises every wheel via `pytest`, so a missing binding
//! is a hard failure — not a code-review oversight.

use std::cell::RefCell;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use crate::activity::{
    ActivityModel, excess_enthalpy as excess_enthalpy_rs, excess_entropy as excess_entropy_rs,
    excess_gibbs as excess_gibbs_rs, ln_gamma as ln_gamma_rs,
};
use crate::eos::{
    ChaoSeaderSpecies, CubicEos, EosError, PhaseId, alpha as eos_alpha_rs,
    chao_seader_ln_phi as chao_seader_ln_phi_rs, d_alpha_d_tr as eos_d_alpha_rs, family_constants,
    h_departure_rt, ln_phi_pure, s_departure_r, z_factor,
};
use crate::liquid_volume::{VolumeModel, liquid_molar_volume as liquid_molar_volume_rs};
use crate::saturation::{
    SatError, SatPressureModel, boiling_temperature as boiling_temperature_rs,
    d_psat_dt as d_psat_dt_rs, d_psat_dt_antoine, poynting_factor as poynting_factor_rs, psat as psat_rs,
    psat_antoine, psat_maxwell as psat_maxwell_rs, reduced_psat as reduced_psat_rs,
};
use crate::types::Component;
use crate::virial::{
    b_mix as virial_b_mix, h_departure_rt_virial, ln_phi_mix_virial, ln_phi_pure_virial, pitzer_b,
    pitzer_b0, pitzer_b1, pitzer_d_b_d_t, s_departure_r_virial, z_factor_virial,
};

/// Return the engine crate's version string (matches `Cargo.toml`).
///
/// Useful for diagnostics: `print(vle._engine.version())` confirms which
/// wheel is loaded when troubleshooting installs.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Return the raw TOML text of the canonical unit catalog.
///
/// Same bytes that seed `UnitRegistry::with_vle_defaults()` on the Rust
/// side. The Python `vle.units` module parses this to register gauge units
/// (barg/psig/kPag), `lbmol`, and `BTU/lbmol` with `pint`, so the two
/// halves of the stack share a single source of truth for unit constants.
#[pyfunction]
fn default_units_toml() -> &'static str {
    vle_units::default_units_toml()
}

// ──────────────────────────────────────────────────────────────────────
// M6.1 — numerics bindings.
//
// All four solvers + the small utility surface are exposed under flat
// names (no `numerics.` prefix on the Python side) so user code reads
// like `from vle._engine import brent, solve_cubic, halley`. The
// higher-level `vle` Python wrapper can re-export these into more
// pythonic submodules later (`vle.numerics.brent` etc.) without
// changing the binding shape.
//
// The three callback-taking functions (brent, illinois, halley) follow
// the same pattern: cache any Python error raised inside the callback
// in a RefCell, return NaN to the Rust solver (which handles NaN
// gracefully via its `NanEvaluation` / `NonFiniteEvaluation` error
// variants), then re-raise the original Python error after the solver
// returns. This preserves Python tracebacks across the FFI boundary
// instead of collapsing them into a generic Rust error message.
// ──────────────────────────────────────────────────────────────────────

/// Solve `a·x³ + b·x² + c·x + d = 0` over the reals.
///
/// Returns a list of every real root in ascending order — 1, 2, or 3
/// entries depending on the discriminant. Raises `ValueError` if `a`
/// is near zero (not actually a cubic) or any coefficient is non-finite.
#[pyfunction]
fn solve_cubic(a: f64, b: f64, c: f64, d: f64) -> PyResult<Vec<f64>> {
    crate::numerics::cubic::solve_real(a, b, c, d).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Brent's method scalar root finder on a bracketed interval.
///
/// `f(a)` and `f(b)` must have opposite signs. Returns the root in
/// `[a, b]`. Raises `RuntimeError` if the bracket is invalid, the
/// iteration limit is hit, or `f` returns NaN. Any exception raised
/// inside `f` is re-raised through the Rust solver.
#[pyfunction]
#[pyo3(signature = (f, a, b, xtol = 1e-9, max_iter = 100))]
fn brent(py: Python<'_>, f: PyObject, a: f64, b: f64, xtol: f64, max_iter: usize) -> PyResult<f64> {
    let err_cache: RefCell<Option<PyErr>> = RefCell::new(None);
    let result = crate::numerics::root_finding::brent(
        |x| call_scalar_callback(py, &f, x, &err_cache),
        a,
        b,
        xtol,
        max_iter,
    );
    if let Some(e) = err_cache.into_inner() {
        return Err(e);
    }
    result.map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Illinois algorithm (modified Regula Falsi) scalar root finder.
///
/// Lighter-weight alternative to [`brent`]: predictable per-iteration
/// cost, slightly slower convergence. Same bracketing requirement and
/// error semantics.
#[pyfunction]
#[pyo3(signature = (f, a, b, xtol = 1e-9, max_iter = 100))]
fn illinois(
    py: Python<'_>,
    f: PyObject,
    a: f64,
    b: f64,
    xtol: f64,
    max_iter: usize,
) -> PyResult<f64> {
    let err_cache: RefCell<Option<PyErr>> = RefCell::new(None);
    let result = crate::numerics::root_finding::illinois(
        |x| call_scalar_callback(py, &f, x, &err_cache),
        a,
        b,
        xtol,
        max_iter,
    );
    if let Some(e) = err_cache.into_inner() {
        return Err(e);
    }
    result.map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Halley's method scalar root finder (cubic convergence).
///
/// `f_and_derivs(x)` must return a 3-tuple `(f, f', f'')`. Not
/// bracketed — caller is responsible for a good initial guess `x0`.
/// Raises `RuntimeError` on non-convergence, singular step, or
/// non-finite evaluation.
#[pyfunction]
#[pyo3(signature = (f_and_derivs, x0, xtol = 1e-12, max_iter = 50))]
fn halley(
    py: Python<'_>,
    f_and_derivs: PyObject,
    x0: f64,
    xtol: f64,
    max_iter: usize,
) -> PyResult<f64> {
    let err_cache: RefCell<Option<PyErr>> = RefCell::new(None);
    let result = crate::numerics::halley::halley(
        |x| match f_and_derivs
            .call1(py, (x,))
            .and_then(|r| r.extract::<(f64, f64, f64)>(py))
        {
            Ok(triple) => triple,
            Err(e) => {
                if err_cache.borrow().is_none() {
                    *err_cache.borrow_mut() = Some(e);
                }
                (f64::NAN, f64::NAN, f64::NAN)
            }
        },
        x0,
        xtol,
        max_iter,
    );
    if let Some(e) = err_cache.into_inner() {
        return Err(e);
    }
    result.map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Broyden's "good" quasi-Newton method for `F(x) = 0` with `F: ℝⁿ → ℝⁿ`.
///
/// `f(x)` must accept a sequence of `n` floats and return a sequence of
/// `n` floats. `x0` sets the problem size. Tolerances and refresh
/// cadence default to engineering values (xtol=ftol=1e-8, max_iter=100,
/// refresh_every=5, fd_step=1e-7); tighten via the keyword args when
/// you need it.
///
/// Raises `ValueError` for input-shape problems and `RuntimeError` for
/// convergence/singular-Jacobian failures. Python exceptions raised
/// inside `f` re-raise verbatim through the Rust solver.
#[pyfunction]
#[pyo3(signature = (f, x0, xtol = 1e-8, ftol = 1e-8, max_iter = 100, refresh_every = 5, fd_step = 1e-7))]
#[allow(clippy::too_many_arguments)]
fn broyden(
    py: Python<'_>,
    f: PyObject,
    x0: Vec<f64>,
    xtol: f64,
    ftol: f64,
    max_iter: usize,
    refresh_every: usize,
    fd_step: f64,
) -> PyResult<Vec<f64>> {
    let cfg = crate::numerics::broyden::BroydenConfig {
        xtol,
        ftol,
        max_iter,
        refresh_every,
        fd_step,
    };
    // Cache the first Python exception so the Rust solver can keep
    // running on a "NaN cycle" of empty residuals, then we re-raise
    // the original traceback after the solver returns. Matches the
    // pattern used by brent / illinois / halley above.
    let err_cache: RefCell<Option<PyErr>> = RefCell::new(None);
    let result = crate::numerics::broyden::broyden(
        |x| match f
            .call1(py, (x.to_vec(),))
            .and_then(|r| r.extract::<Vec<f64>>(py))
        {
            Ok(v) => v,
            Err(e) => {
                if err_cache.borrow().is_none() {
                    *err_cache.borrow_mut() = Some(e);
                }
                // Return a NaN vector of the right length so the Rust
                // solver hits NonFiniteEvaluation on the very next
                // check and exits cleanly.
                vec![f64::NAN; x.len()]
            }
        },
        &x0,
        cfg,
    );
    if let Some(e) = err_cache.into_inner() {
        return Err(e);
    }
    result.map_err(|e| match e {
        crate::numerics::broyden::BroydenError::DimensionMismatch { .. } => {
            PyValueError::new_err(e.to_string())
        }
        _ => PyRuntimeError::new_err(e.to_string()),
    })
}

/// Helper for the scalar-callback solvers: call into Python, extract
/// f64, cache the first error and return NaN so the Rust solver fails
/// fast with `NanEvaluation`.
fn call_scalar_callback(
    py: Python<'_>,
    f: &PyObject,
    x: f64,
    err_cache: &RefCell<Option<PyErr>>,
) -> f64 {
    // Short-circuit subsequent calls once an error is pending — saves
    // wasted Python-call work while we wait for the Rust loop to exit.
    if err_cache.borrow().is_some() {
        return f64::NAN;
    }
    match f.call1(py, (x,)).and_then(|r| r.extract::<f64>(py)) {
        Ok(v) => v,
        Err(e) => {
            *err_cache.borrow_mut() = Some(e);
            f64::NAN
        }
    }
}

// ---- Utility functions ----------------------------------------------

/// Compute `1 - sum(xs)`. Used as the Rachford-Rice / bubble-point
/// composition-closure residual.
#[pyfunction]
fn sum_frac_residual(xs: Vec<f64>) -> f64 {
    crate::numerics::utils::sum_frac_residual(&xs)
}

/// L1 (sum-of-absolutes) norm of a vector.
#[pyfunction]
fn norm_l1(xs: Vec<f64>) -> f64 {
    crate::numerics::utils::norm_l1(&xs)
}

/// L2 (Euclidean) norm of a vector.
#[pyfunction]
fn norm_l2(xs: Vec<f64>) -> f64 {
    crate::numerics::utils::norm_l2(&xs)
}

/// L∞ (max-of-absolutes) norm of a vector.
#[pyfunction]
fn norm_linf(xs: Vec<f64>) -> f64 {
    crate::numerics::utils::norm_linf(&xs)
}

// ──────────────────────────────────────────────────────────────────────
// M7.1 — pure-component EOS bindings.
//
// Cubic EOS expressions take scalar floats for the component data
// (tc, pc, omega) rather than a `Component` pyclass — same convention
// as the M6.1 numerics bindings. The higher-level `vle` Python wrapper
// can pull these into a class-shaped API once we have a richer property
// database; in M7 keeping the C ABI scalar-only keeps the FFI surface
// minimal and the test matrix tractable.
// ──────────────────────────────────────────────────────────────────────

/// Build a minimal Component for the cubic-EOS layer. Internal helper —
/// not exposed to Python. Only the four fields used by the core PR/RKS/
/// RK/VdW path are populated; everything else stays at its `Default`.
fn comp_for_eos(tc: f64, pc: f64, omega: f64) -> Component {
    Component {
        tc,
        pc,
        omega,
        ..Component::default()
    }
}

/// Build a Component carrying the parameters the M7.2 α functions read.
/// Internal helper for the `_ex` α bindings. Tc/Pc are left at 1.0 because
/// α(Tr) is a pure function of the *reduced* temperature and these polar
/// parameters — it never touches the dimensional critical constants.
fn comp_for_alpha(omega: f64, zc: f64, m: f64, n: f64, g: f64, prsv_k1: f64) -> Component {
    Component {
        tc: 1.0,
        pc: 1.0,
        omega,
        zc,
        m_polar: m,
        n_polar: n,
        g_polar: g,
        prsv_k1,
        ..Component::default()
    }
}

fn phase_from_str(s: &str) -> PyResult<PhaseId> {
    match s.to_ascii_lowercase().as_str() {
        "vapor" | "v" | "gas" => Ok(PhaseId::Vapor),
        "liquid" | "l" => Ok(PhaseId::Liquid),
        other => Err(PyValueError::new_err(format!(
            "phase must be 'vapor' or 'liquid' (got {other:?})"
        ))),
    }
}

/// Map an `EosError` to a Python exception. `NotImplemented` becomes a
/// `NotImplementedError`; the rest become `RuntimeError` so the
/// originating message survives the FFI hop.
fn map_eos_err(e: EosError) -> PyErr {
    match e {
        EosError::NotImplemented(_) => {
            pyo3::exceptions::PyNotImplementedError::new_err(e.to_string())
        }
        _ => PyRuntimeError::new_err(e.to_string()),
    }
}

/// α(Tr) for the requested cubic EOS.
///
/// This is the acentric-factor-only entry point: it covers every variant
/// whose α depends solely on ω (PR1976, RKS1972, RK1949, VdW1870,
/// Berth1899, VdWAda1984, RKSGD1978, RKSL1997, RP1978, PRL1997). For the
/// variants that additionally read `Zc` or a fitted polar parameter
/// (VdWVald1989, RKSmn1980, RKSATmn1995, PRATmng1997, PRMmn1989, PRSV1986)
/// use [`eos_alpha_ex`], which accepts them.
///
/// Panics (surfaced as a Python `PanicException`) for the OL family
/// (M7.4 — saturation-coupled) and the 3-param Pascal EOS (M7.3).
#[pyfunction]
fn eos_alpha(eos: CubicEos, tr: f64, omega: f64) -> f64 {
    eos_alpha_rs(eos, tr, &comp_for_eos(1.0, 1.0, omega))
}

/// dα/dTr for the requested cubic EOS (analytical). ω-only entry point;
/// see [`eos_d_alpha_d_tr_ex`] for the parameterized variants.
#[pyfunction]
fn eos_d_alpha_d_tr(eos: CubicEos, tr: f64, omega: f64) -> f64 {
    eos_d_alpha_rs(eos, tr, &comp_for_eos(1.0, 1.0, omega))
}

/// α(Tr) for the requested cubic EOS, with the full parameter set.
///
/// Extends [`eos_alpha`] with the fields the M7.2 polar/fitted variants
/// read: `zc` (VdWVald1989), `m`/`n` (RKSmn1980, PRMmn1989), `m`/`n`/`g`
/// (RKSATmn1995, PRATmng1997), and `prsv_k1` (PRSV1986). The ω-only
/// variants ignore the extra arguments, so this is a strict superset of
/// `eos_alpha` — passing the defaults reproduces it exactly.
///
/// All extra parameters default to 0, so callers only specify what their
/// chosen variant needs.
#[pyfunction]
#[pyo3(signature = (eos, tr, omega, zc=0.0, m=0.0, n=0.0, g=0.0, prsv_k1=0.0))]
#[allow(clippy::too_many_arguments)]
fn eos_alpha_ex(
    eos: CubicEos,
    tr: f64,
    omega: f64,
    zc: f64,
    m: f64,
    n: f64,
    g: f64,
    prsv_k1: f64,
) -> f64 {
    eos_alpha_rs(eos, tr, &comp_for_alpha(omega, zc, m, n, g, prsv_k1))
}

/// dα/dTr for the requested cubic EOS (analytical), with the full
/// parameter set. The `_ex` counterpart of [`eos_d_alpha_d_tr`] — see
/// [`eos_alpha_ex`] for the parameter meanings.
#[pyfunction]
#[pyo3(signature = (eos, tr, omega, zc=0.0, m=0.0, n=0.0, g=0.0, prsv_k1=0.0))]
#[allow(clippy::too_many_arguments)]
fn eos_d_alpha_d_tr_ex(
    eos: CubicEos,
    tr: f64,
    omega: f64,
    zc: f64,
    m: f64,
    n: f64,
    g: f64,
    prsv_k1: f64,
) -> f64 {
    eos_d_alpha_rs(eos, tr, &comp_for_alpha(omega, zc, m, n, g, prsv_k1))
}

/// Family constants (k1, k2, OmA, OmB) for the EOS, as a 4-tuple.
#[pyfunction]
fn eos_family_constants(eos: CubicEos) -> (f64, f64, f64, f64) {
    let fc = family_constants(eos);
    (fc.k1, fc.k2, fc.om_a, fc.om_b)
}

/// Z = P·V/(R·T) for the requested phase.
///
/// `t` in K, `p` in kPa absolute. `phase` is "vapor" or "liquid".
#[pyfunction]
fn eos_z_factor(
    eos: CubicEos,
    t: f64,
    p: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    phase: &str,
) -> PyResult<f64> {
    let comp = comp_for_eos(tc, pc, omega);
    let phase = phase_from_str(phase)?;
    z_factor(eos, t, p, &comp, phase).map_err(map_eos_err)
}

/// Pure-component ln(φ) for the requested phase.
#[pyfunction]
fn eos_ln_phi_pure(
    eos: CubicEos,
    t: f64,
    p: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    phase: &str,
) -> PyResult<f64> {
    let comp = comp_for_eos(tc, pc, omega);
    let phase = phase_from_str(phase)?;
    ln_phi_pure(eos, t, p, &comp, phase).map_err(map_eos_err)
}

/// Departure enthalpy H^R/(R·T), dimensionless.
#[pyfunction]
fn eos_h_departure_rt(
    eos: CubicEos,
    t: f64,
    p: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    phase: &str,
) -> PyResult<f64> {
    let comp = comp_for_eos(tc, pc, omega);
    let phase = phase_from_str(phase)?;
    h_departure_rt(eos, t, p, &comp, phase).map_err(map_eos_err)
}

/// Departure entropy S^R/R, dimensionless.
#[pyfunction]
fn eos_s_departure_r(
    eos: CubicEos,
    t: f64,
    p: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    phase: &str,
) -> PyResult<f64> {
    let comp = comp_for_eos(tc, pc, omega);
    let phase = phase_from_str(phase)?;
    s_departure_r(eos, t, p, &comp, phase).map_err(map_eos_err)
}

/// Chao-Seader pure-liquid fugacity coefficient as **ln(ν)** (M7.3).
///
/// `species` selects the coefficient set (`ChaoSeaderSpecies.Normal`,
/// `.Hydrogen`, or `.Methane`). T in **K**, P in **kPa absolute**.
/// Ref (4): Da Silva & Báez (1989), legacy/pascal/TERMOII.PAS.
#[pyfunction]
fn chao_seader_ln_phi(
    t: f64,
    p: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    species: ChaoSeaderSpecies,
) -> f64 {
    let comp = comp_for_eos(tc, pc, omega);
    chao_seader_ln_phi_rs(t, p, &comp, species)
}

// ──────────────────────────────────────────────────────────────────────
// M7.5 — Antoine saturation bindings.
// ──────────────────────────────────────────────────────────────────────

fn map_sat_err(e: SatError) -> PyErr {
    match e {
        SatError::NotImplemented(_) => {
            pyo3::exceptions::PyNotImplementedError::new_err(e.to_string())
        }
        SatError::BadCoefficients { .. } => PyValueError::new_err(e.to_string()),
        SatError::OutOfRange(_) => PyValueError::new_err(e.to_string()),
        SatError::Maxwell(_) => PyRuntimeError::new_err(e.to_string()),
    }
}

/// Build a Component carrying the fields the saturation layer reads.
fn comp_for_sat(tc: f64, pc: f64, omega: f64, tb: f64, coeffs: Vec<f64>) -> Component {
    Component {
        tc,
        pc,
        omega,
        tb,
        psat_coeffs: coeffs,
        ..Component::default()
    }
}

/// Antoine saturation pressure `ln(Psat/Pc) = a1 − a2/(a3 + T)`.
///
/// Returns Psat in **kPa**. `coeffs` is `[a1, a2, a3]`.
#[pyfunction]
fn antoine_psat(t: f64, pc: f64, coeffs: Vec<f64>) -> PyResult<f64> {
    let comp = Component {
        pc,
        psat_coeffs: coeffs,
        ..Component::default()
    };
    psat_antoine(&comp, t).map_err(map_sat_err)
}

/// Analytical dPsat/dT for the Antoine form. Returns kPa/K.
#[pyfunction]
fn antoine_d_psat_dt(t: f64, pc: f64, coeffs: Vec<f64>) -> PyResult<f64> {
    let comp = Component {
        pc,
        psat_coeffs: coeffs,
        ..Component::default()
    };
    d_psat_dt_antoine(&comp, t).map_err(map_sat_err)
}

// ──────────────────────────────────────────────────────────────────────
// M7.4 — advanced saturation models + OL-family α.
// ──────────────────────────────────────────────────────────────────────

/// Saturation pressure for any non-Maxwell model. Returns Psat in **kPa**.
/// `coeffs` are the Antoine `[a1,a2,a3]` (or DIPPR `[c0..c4]` for `Polynomial`);
/// the corresponding-states models (Riedel/Müller/RPM) read `tc`, `pc`, `omega`, `tb`.
#[pyfunction]
#[pyo3(signature = (model, t, tc, pc, omega=0.0, tb=0.0, coeffs=vec![]))]
fn sat_psat(
    model: SatPressureModel,
    t: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    tb: f64,
    coeffs: Vec<f64>,
) -> PyResult<f64> {
    psat_rs(model, &comp_for_sat(tc, pc, omega, tb, coeffs), t).map_err(map_sat_err)
}

/// `dPsat/dT` (kPa/K) for any non-Maxwell model — analytical for Antoine,
/// central-difference otherwise.
#[pyfunction]
#[pyo3(signature = (model, t, tc, pc, omega=0.0, tb=0.0, coeffs=vec![]))]
fn sat_d_psat_dt(
    model: SatPressureModel,
    t: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    tb: f64,
    coeffs: Vec<f64>,
) -> PyResult<f64> {
    d_psat_dt_rs(model, &comp_for_sat(tc, pc, omega, tb, coeffs), t).map_err(map_sat_err)
}

/// Reduced saturation pressure `Psat/Pc` (dimensionless) for a non-Maxwell model.
#[pyfunction]
#[pyo3(signature = (model, t, tc, pc, omega=0.0, tb=0.0, coeffs=vec![]))]
fn sat_reduced_psat(
    model: SatPressureModel,
    t: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    tb: f64,
    coeffs: Vec<f64>,
) -> PyResult<f64> {
    reduced_psat_rs(model, &comp_for_sat(tc, pc, omega, tb, coeffs), t).map_err(map_sat_err)
}

/// Maxwell equal-area saturation pressure for a cubic EOS at temperature `t`.
/// Returns Psat in **kPa**. `coeffs` seed the initial guess (Antoine if length 3).
#[pyfunction]
#[pyo3(signature = (eos, t, tc, pc, omega=0.0, coeffs=vec![]))]
fn sat_maxwell(
    eos: CubicEos,
    t: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    coeffs: Vec<f64>,
) -> PyResult<f64> {
    psat_maxwell_rs(eos, &comp_for_sat(tc, pc, omega, 0.0, coeffs), t).map_err(map_sat_err)
}

/// Boiling temperature (**K**): invert `Psat(T)=P` for the given model.
#[pyfunction]
#[pyo3(signature = (model, p, tc, pc, omega=0.0, tb=0.0, coeffs=vec![]))]
fn boiling_temperature(
    model: SatPressureModel,
    p: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    tb: f64,
    coeffs: Vec<f64>,
) -> PyResult<f64> {
    boiling_temperature_rs(model, &comp_for_sat(tc, pc, omega, tb, coeffs), p).map_err(map_sat_err)
}

/// Poynting factor `exp[V_L·(P − Psat)/(R·T)]` (dimensionless). `liquid_volume`
/// in **cm³/mol**; `p`, `psat` in **kPa**; `t` in **K**.
#[pyfunction]
fn poynting_factor(p: f64, psat: f64, t: f64, liquid_volume: f64) -> f64 {
    let comp = Component {
        liquid_volume,
        ..Component::default()
    };
    poynting_factor_rs(&comp, p, psat, t)
}

/// OL-family α(Tr) — needs the component's saturation data. `sat_model` selects
/// the reduced-saturation-pressure source; pass Antoine `coeffs=[a1,a2,a3]` (or
/// `tb` for the corresponding-states models).
#[pyfunction]
#[pyo3(signature = (eos, tr, tc, pc, omega, sat_model, tb=0.0, coeffs=vec![]))]
#[allow(clippy::too_many_arguments)]
fn eos_alpha_ol(
    eos: CubicEos,
    tr: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    sat_model: SatPressureModel,
    tb: f64,
    coeffs: Vec<f64>,
) -> f64 {
    let mut comp = comp_for_sat(tc, pc, omega, tb, coeffs);
    comp.sat_model = sat_model;
    eos_alpha_rs(eos, tr, &comp)
}

/// Analytical dα/dTr for the OL family (companion to [`eos_alpha_ol`]).
#[pyfunction]
#[pyo3(signature = (eos, tr, tc, pc, omega, sat_model, tb=0.0, coeffs=vec![]))]
#[allow(clippy::too_many_arguments)]
fn eos_d_alpha_d_tr_ol(
    eos: CubicEos,
    tr: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    sat_model: SatPressureModel,
    tb: f64,
    coeffs: Vec<f64>,
) -> f64 {
    let mut comp = comp_for_sat(tc, pc, omega, tb, coeffs);
    comp.sat_model = sat_model;
    eos_d_alpha_rs(eos, tr, &comp)
}

// ──────────────────────────────────────────────────────────────────────
// M7.6 — Virial bindings.
// ──────────────────────────────────────────────────────────────────────

/// Pitzer B⁰(Tr) — simple-fluid reduced second virial coefficient.
#[pyfunction]
fn virial_pitzer_b0(tr: f64) -> f64 {
    pitzer_b0(tr)
}

/// Pitzer B¹(Tr) — acentric correction to the reduced second virial coefficient.
#[pyfunction]
fn virial_pitzer_b1(tr: f64) -> f64 {
    pitzer_b1(tr)
}

/// Second virial coefficient B(T) for a pure component. Returns cm³/mol.
#[pyfunction]
fn virial_b_pure(tc: f64, pc: f64, omega: f64, t: f64) -> f64 {
    pitzer_b(&comp_for_eos(tc, pc, omega), t)
}

/// dB/dT for a pure component. Returns cm³/(mol·K).
#[pyfunction]
fn virial_d_b_d_t_pure(tc: f64, pc: f64, omega: f64, t: f64) -> f64 {
    pitzer_d_b_d_t(&comp_for_eos(tc, pc, omega), t)
}

/// Z = 1 + B·P/(R·T) — pure-component truncated virial.
#[pyfunction]
fn virial_z(tc: f64, pc: f64, omega: f64, t: f64, p: f64) -> f64 {
    z_factor_virial(&comp_for_eos(tc, pc, omega), t, p)
}

/// ln(φ) from the truncated virial — pure.
#[pyfunction]
fn virial_ln_phi(tc: f64, pc: f64, omega: f64, t: f64, p: f64) -> f64 {
    ln_phi_pure_virial(&comp_for_eos(tc, pc, omega), t, p)
}

/// Departure enthalpy H^R/(R·T) from the truncated virial — pure.
#[pyfunction]
fn virial_h_dep_rt(tc: f64, pc: f64, omega: f64, t: f64, p: f64) -> f64 {
    h_departure_rt_virial(&comp_for_eos(tc, pc, omega), t, p)
}

/// Departure entropy S^R/R from the truncated virial — pure.
#[pyfunction]
fn virial_s_dep_r(tc: f64, pc: f64, omega: f64, t: f64, p: f64) -> f64 {
    s_departure_r_virial(&comp_for_eos(tc, pc, omega), t, p)
}

/// Mixture B(T, x) — quadratic Lewis-Randall mixing with Pitzer cross-terms.
///
/// Parallel input vectors `tcs`, `pcs`, `omegas`, `mole_fractions` all
/// must have the same length. Returns B_mix in cm³/mol.
#[pyfunction]
fn virial_b_mix_py(
    tcs: Vec<f64>,
    pcs: Vec<f64>,
    omegas: Vec<f64>,
    mole_fractions: Vec<f64>,
    t: f64,
) -> PyResult<f64> {
    let n = tcs.len();
    if pcs.len() != n || omegas.len() != n || mole_fractions.len() != n {
        return Err(PyValueError::new_err(
            "tcs, pcs, omegas, mole_fractions must all have the same length",
        ));
    }
    let comps: Vec<Component> = (0..n)
        .map(|i| comp_for_eos(tcs[i], pcs[i], omegas[i]))
        .collect();
    virial_b_mix(&comps, &mole_fractions, t).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Mixture partial fugacity coefficients ln(φᵢ) from the truncated virial.
///
/// Returns one ln(φᵢ) per component in input order.
#[pyfunction]
fn virial_ln_phi_mix(
    tcs: Vec<f64>,
    pcs: Vec<f64>,
    omegas: Vec<f64>,
    mole_fractions: Vec<f64>,
    t: f64,
    p: f64,
) -> PyResult<Vec<f64>> {
    let n = tcs.len();
    if pcs.len() != n || omegas.len() != n || mole_fractions.len() != n {
        return Err(PyValueError::new_err(
            "tcs, pcs, omegas, mole_fractions must all have the same length",
        ));
    }
    let comps: Vec<Component> = (0..n)
        .map(|i| comp_for_eos(tcs[i], pcs[i], omegas[i]))
        .collect();
    ln_phi_mix_virial(&comps, &mole_fractions, t, p)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

// ──────────────────────────────────────────────────────────────────────
// M8.1 — liquid molar volume + activity-coefficient models.
// ──────────────────────────────────────────────────────────────────────

/// Saturated-liquid molar volume Vᴸ for one component.
///
/// `model` is [`VolumeModel::Rackett`] (uses `zra`) or
/// [`VolumeModel::Thomson`] (uses `vstar` = V* in cm³/mol and `omega_srk`).
/// `tc` in K, `pc` in kPa, `t` in K. Returns cm³/mol.
#[pyfunction]
#[pyo3(signature = (model, tc, pc, t, zra=0.0, vstar=0.0, omega_srk=0.0))]
fn liquid_molar_volume(
    model: VolumeModel,
    tc: f64,
    pc: f64,
    t: f64,
    zra: f64,
    vstar: f64,
    omega_srk: f64,
) -> f64 {
    let comp = Component {
        tc,
        pc,
        zra,
        liquid_volume: vstar,
        omega_srk,
        ..Component::default()
    };
    liquid_molar_volume_rs(model, &comp, t)
}

/// ln(γᵢ) for component `i`. See the Rust [`crate::activity::ln_gamma`] for the
/// per-model `aij` convention; `vl` (cm³/mol) and `delta` ((cal/cm³)^0.5) may be
/// empty lists for models that don't use them. `t` in K.
#[pyfunction]
#[pyo3(signature = (model, i, x, aij, vl=vec![], delta=vec![], t=298.15))]
fn activity_ln_gamma(
    model: ActivityModel,
    i: usize,
    x: Vec<f64>,
    aij: Vec<Vec<f64>>,
    vl: Vec<f64>,
    delta: Vec<f64>,
    t: f64,
) -> f64 {
    ln_gamma_rs(model, i, &x, &aij, &vl, &delta, t)
}

/// Excess Gibbs energy Gᴱ = RT Σ xᵢ ln γᵢ, in kJ/kmol. Args as in
/// [`activity_ln_gamma`] (minus the component index).
#[pyfunction]
#[pyo3(signature = (model, x, aij, vl=vec![], delta=vec![], t=298.15))]
fn activity_excess_gibbs(
    model: ActivityModel,
    x: Vec<f64>,
    aij: Vec<Vec<f64>>,
    vl: Vec<f64>,
    delta: Vec<f64>,
    t: f64,
) -> f64 {
    excess_gibbs_rs(model, &x, &aij, &vl, &delta, t)
}

/// Excess enthalpy Hᴱ (analytical), in kJ/kmol. Args as in [`activity_excess_gibbs`].
#[pyfunction]
#[pyo3(signature = (model, x, aij, vl=vec![], delta=vec![], t=298.15))]
fn activity_excess_enthalpy(
    model: ActivityModel,
    x: Vec<f64>,
    aij: Vec<Vec<f64>>,
    vl: Vec<f64>,
    delta: Vec<f64>,
    t: f64,
) -> f64 {
    excess_enthalpy_rs(model, &x, &aij, &vl, &delta, t)
}

/// Excess entropy Sᴱ = (Hᴱ − Gᴱ)/T, in kJ/(kmol·K). Args as in
/// [`activity_excess_gibbs`].
#[pyfunction]
#[pyo3(signature = (model, x, aij, vl=vec![], delta=vec![], t=298.15))]
fn activity_excess_entropy(
    model: ActivityModel,
    x: Vec<f64>,
    aij: Vec<Vec<f64>>,
    vl: Vec<f64>,
    delta: Vec<f64>,
    t: f64,
) -> f64 {
    excess_entropy_rs(model, &x, &aij, &vl, &delta, t)
}

/// PyO3 module entry point.
///
/// Maturin builds this into `vle/_engine.<platform>.<ext>` and Python
/// imports it as `vle._engine`. The function name (`_engine`) must match
/// the `module-name` set in `python/pyproject.toml`'s `[tool.maturin]`.
#[pymodule]
fn _engine(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(default_units_toml, m)?)?;

    // M6.1 numerics — solvers + utility functions.
    m.add_function(wrap_pyfunction!(solve_cubic, m)?)?;
    m.add_function(wrap_pyfunction!(brent, m)?)?;
    m.add_function(wrap_pyfunction!(illinois, m)?)?;
    m.add_function(wrap_pyfunction!(halley, m)?)?;
    m.add_function(wrap_pyfunction!(broyden, m)?)?;
    m.add_function(wrap_pyfunction!(sum_frac_residual, m)?)?;
    m.add_function(wrap_pyfunction!(norm_l1, m)?)?;
    m.add_function(wrap_pyfunction!(norm_l2, m)?)?;
    m.add_function(wrap_pyfunction!(norm_linf, m)?)?;

    // The four model-selection enums. Each is `#[pyclass(eq, eq_int)]`
    // at its definition site; here we just register the class with the
    // module so `vle._engine.CubicEos` etc. resolve from Python.
    m.add_class::<crate::eos::CubicEos>()?;
    m.add_class::<crate::activity::ActivityModel>()?;
    m.add_class::<crate::mixing::MixingRule>()?;
    m.add_class::<crate::saturation::SatPressureModel>()?;
    m.add_class::<crate::eos::PhaseId>()?;
    m.add_class::<crate::eos::ChaoSeaderSpecies>()?;

    // M7.1 cubic-EOS bindings.
    m.add_function(wrap_pyfunction!(eos_alpha, m)?)?;
    m.add_function(wrap_pyfunction!(eos_d_alpha_d_tr, m)?)?;
    // M7.2 — α variants taking the full polar/fitted parameter set.
    m.add_function(wrap_pyfunction!(eos_alpha_ex, m)?)?;
    m.add_function(wrap_pyfunction!(eos_d_alpha_d_tr_ex, m)?)?;
    m.add_function(wrap_pyfunction!(eos_family_constants, m)?)?;
    m.add_function(wrap_pyfunction!(eos_z_factor, m)?)?;
    m.add_function(wrap_pyfunction!(eos_ln_phi_pure, m)?)?;
    m.add_function(wrap_pyfunction!(eos_h_departure_rt, m)?)?;
    m.add_function(wrap_pyfunction!(eos_s_departure_r, m)?)?;
    m.add_function(wrap_pyfunction!(chao_seader_ln_phi, m)?)?;

    // M7.5 Antoine saturation bindings.
    m.add_function(wrap_pyfunction!(antoine_psat, m)?)?;
    m.add_function(wrap_pyfunction!(antoine_d_psat_dt, m)?)?;

    // M7.4 advanced saturation models + OL-family α.
    m.add_function(wrap_pyfunction!(sat_psat, m)?)?;
    m.add_function(wrap_pyfunction!(sat_d_psat_dt, m)?)?;
    m.add_function(wrap_pyfunction!(sat_reduced_psat, m)?)?;
    m.add_function(wrap_pyfunction!(sat_maxwell, m)?)?;
    m.add_function(wrap_pyfunction!(boiling_temperature, m)?)?;
    m.add_function(wrap_pyfunction!(poynting_factor, m)?)?;
    m.add_function(wrap_pyfunction!(eos_alpha_ol, m)?)?;
    m.add_function(wrap_pyfunction!(eos_d_alpha_d_tr_ol, m)?)?;

    // M7.6 Virial bindings.
    m.add_function(wrap_pyfunction!(virial_pitzer_b0, m)?)?;
    m.add_function(wrap_pyfunction!(virial_pitzer_b1, m)?)?;
    m.add_function(wrap_pyfunction!(virial_b_pure, m)?)?;
    m.add_function(wrap_pyfunction!(virial_d_b_d_t_pure, m)?)?;
    m.add_function(wrap_pyfunction!(virial_z, m)?)?;
    m.add_function(wrap_pyfunction!(virial_ln_phi, m)?)?;
    m.add_function(wrap_pyfunction!(virial_h_dep_rt, m)?)?;
    m.add_function(wrap_pyfunction!(virial_s_dep_r, m)?)?;
    m.add_function(wrap_pyfunction!(virial_b_mix_py, m)?)?;
    m.add_function(wrap_pyfunction!(virial_ln_phi_mix, m)?)?;

    // M8.1 liquid molar volume + activity-coefficient models.
    m.add_class::<crate::liquid_volume::VolumeModel>()?;
    m.add_function(wrap_pyfunction!(liquid_molar_volume, m)?)?;
    m.add_function(wrap_pyfunction!(activity_ln_gamma, m)?)?;
    m.add_function(wrap_pyfunction!(activity_excess_gibbs, m)?)?;
    m.add_function(wrap_pyfunction!(activity_excess_enthalpy, m)?)?;
    m.add_function(wrap_pyfunction!(activity_excess_entropy, m)?)?;

    Ok(())
}
