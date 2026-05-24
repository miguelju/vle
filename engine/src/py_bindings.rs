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

    Ok(())
}
