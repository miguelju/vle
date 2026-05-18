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

use pyo3::prelude::*;

/// Return the engine crate's version string (matches `Cargo.toml`).
///
/// Useful for diagnostics: `print(vle._engine.version())` confirms which
/// wheel is loaded when troubleshooting installs.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// PyO3 module entry point.
///
/// Maturin builds this into `vle/_engine.<platform>.<ext>` and Python
/// imports it as `vle._engine`. The function name (`_engine`) must match
/// the `module-name` set in `python/pyproject.toml`'s `[tool.maturin]`.
#[pymodule]
fn _engine(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;

    // The four model-selection enums. Each is `#[pyclass(eq, eq_int)]`
    // at its definition site; here we just register the class with the
    // module so `vle._engine.CubicEos` etc. resolve from Python.
    m.add_class::<crate::eos::CubicEos>()?;
    m.add_class::<crate::activity::ActivityModel>()?;
    m.add_class::<crate::mixing::MixingRule>()?;
    m.add_class::<crate::saturation::SatPressureModel>()?;

    Ok(())
}
