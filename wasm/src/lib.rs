//! # vle-wasm — JavaScript/TypeScript bindings for the vle engine (wasm-bindgen)
//!
//! This crate is the **FFI adapter** between the `vle-thermo` / `vle-steam`
//! Rust engine and JavaScript — browsers, Node.js, and the webview shells
//! (Tauri, Electron, Capacitor). It is the third sibling of `ffi/` (Swift +
//! Kotlin via UniFFI) and exposes the same deliberately small, flat API.
//! All thermodynamics lives in the engine crates; nothing here computes.
//!
//! ## How it works (the 30-second version)
//!
//! - `#[wasm_bindgen]` attributes make the compiler emit **wasm exports**
//!   (plain functions over wasm's four numeric types) plus a metadata
//!   section describing the richer signatures.
//! - `wasm-pack` (driving the wasm-bindgen CLI) reads that metadata out of
//!   the compiled `.wasm` and generates the matching **JS glue + TypeScript
//!   declarations** — `wasm/pkg/`, a ready-to-`npm install` package.
//! - Records cross the boundary as **plain JS objects** (via
//!   `serde-wasm-bindgen`), compositions as **`Float64Array`s**, and Rust
//!   `Err`s are **thrown as JS `Error`s**.
//!
//! See `docs/en/web/README.md` for the full walkthrough (and the theory).
//!
//! ## Units (canonical engine units, per CLAUDE.md)
//!
//! The wasm layer speaks **repo-canonical units only** — no unit strings:
//! temperature **K**, pressure **kPa absolute**, molar energy **kJ/kmol**,
//! molar volume **cm³/mol**. Steam-table quantities are **mass-basis**:
//! kJ/kg, kJ/(kg·K), m³/kg (the classic printed-table units). Unit
//! conversion belongs on the JavaScript side.
//!
//! ## Educational notes (Rust idioms used here)
//!
//! - The exported shims are the *only* place `JsValue` appears; the
//!   underlying logic (`system::SystemCore`, the parse/convert helpers) is
//!   plain Rust, so `cargo test -p vle-wasm` exercises it on the host
//!   without any JS runtime.
//! - Conversions between engine types and boundary records are `From`/
//!   `TryFrom` impls, exactly as in `ffi/` — the compiler keeps the `match`
//!   arms exhaustive when the engine gains a new enum variant.

use wasm_bindgen::prelude::*;

pub mod component;
pub mod error;
pub mod steam;
pub mod system;

pub use component::*;
pub use error::*;
pub use steam::*;
pub use system::*;

/// Module-init hook: runs once when the wasm module is instantiated.
///
/// Installs the panic hook so a Rust panic prints its message and location
/// to the browser/Node console instead of a bare `unreachable` trap. (All
/// *expected* failures are `Result`s that surface as thrown JS `Error`s —
/// a panic here is always a bug worth a readable report.)
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Version of the vle engine compiled into this module.
///
/// The workspace shares one version number across `vle-thermo`,
/// `vle-steam`, and this wrapper, so this string identifies all of them.
/// Use it as the first smoke test from JavaScript: if `version()` returns,
/// the whole Rust→wasm→JS pipeline is alive.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_matches_workspace() {
        // The wrapper inherits the workspace version, so this doubles as a
        // "did the workspace wiring work" check.
        assert_eq!(super::version(), env!("CARGO_PKG_VERSION"));
    }
}
