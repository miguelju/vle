//! Numerical primitives used throughout the VLE engine.
//!
//! The legacy VB6 + Pascal codebases hand-rolled their own solvers (Regula
//! Falsi, golden-section search, naive Newton-Raphson) inline with each
//! thermodynamic routine. This module extracts every numerical method into
//! one well-tested place so the rest of the engine can call into algorithms
//! by name instead of copy-pasting the legacy implementations.
//!
//! ## What lives here
//!
//! - [`utils`] — small reusable helpers (`SumFrac`, vector norms, convergence
//!   checks, `is_near_zero`). Used by every other algorithm in this module.
//! - [`cubic`] — Cardano's formula for `a·x³ + b·x² + c·x + d = 0` with the
//!   (12) Poling & Prausnitz robustness for near-degenerate discriminants.
//!   The workhorse for cubic-EOS Z-factor and volume-root calculations
//!   (M7+).
//! - [`root_finding`] — bracketed scalar root finders. [`root_finding::brent`]
//!   is the recommended default (combines bisection + secant + inverse-
//!   quadratic interpolation, super-linear convergence with the safety of
//!   bisection). [`root_finding::illinois`] is a lighter alternative when
//!   you want predictable per-iter cost. Both replace the legacy Regula
//!   Falsi from `clsSatPressureSolver.cls`.
//! - [`halley`] — Halley's method (cubic convergence) for scalar equations
//!   where you have analytical first and second derivatives. Used in
//!   Rachford-Rice (M9).
//!
//! ## Conventions
//!
//! - **Errors as enums**, not panics. Every fallible algorithm returns
//!   `Result<T, SpecificError>` where `SpecificError` tells the caller
//!   *what* went wrong (no bracket, no convergence, malformed input, …).
//!   Callers up the stack collapse them into engine-level errors.
//! - **Pure-function API.** Functions take their callback as a `Fn(f64) ->
//!   f64`-style argument and return their result. No globals, no mutable
//!   shared state. This keeps the algorithms trivially parallel-safe and
//!   easy to test in isolation.
//! - **Tolerances are required, not defaulted.** Every iterative method
//!   asks the caller for `xtol` / `max_iter`. Sensible defaults belong at
//!   the call site, where the caller knows the physical scale of the
//!   variable being solved for.

pub mod cubic;
pub mod halley;
pub mod root_finding;
pub mod utils;
