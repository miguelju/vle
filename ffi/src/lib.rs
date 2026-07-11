//! # vle-ffi — Swift bindings for the vle engine (UniFFI)
//!
//! This crate is the **FFI adapter** between the `vle-thermo` /`vle-steam`
//! Rust engine and Swift (iOS + macOS apps). It exposes a deliberately
//! small, flat API — plain records, owned `Vec`s, fieldless-or-simple
//! enums — because that is what crosses a language boundary cleanly.
//! All thermodynamics lives in the engine crates; nothing here computes.
//!
//! ## How it works (the 30-second version)
//!
//! - `uniffi::setup_scaffolding!()` plus the `#[uniffi::export]` /
//!   `#[derive(uniffi::Record/Enum/Error/Object)]` attributes make the
//!   compiler emit a **C ABI layer** (`extern "C"` functions + interface
//!   metadata) into the compiled library.
//! - `ffi/uniffi-bindgen` then reads that metadata out of `libvle_ffi.a`
//!   and generates the matching **Swift wrapper** — classes, structs,
//!   enums, and `throws` functions that call the C layer for you.
//! - `scripts/build-ios.sh` packages the static libraries for each Apple
//!   target into `VleFFI.xcframework`, consumed by `swift/VleThermo`.
//!
//! See `docs/en/ios/README.md` for the full walkthrough (and the theory).
//!
//! ## Units (canonical engine units, per CLAUDE.md)
//!
//! The FFI layer speaks **repo-canonical units only** — no unit strings:
//! temperature **K**, pressure **kPa absolute**, molar energy **kJ/kmol**,
//! molar entropy **kJ/(kmol·K)**, molar volume **cm³/mol**. Steam-table
//! quantities are **mass-basis**: kJ/kg, kJ/(kg·K), m³/kg (the classic
//! printed-table units). Unit conversion belongs on the Swift side.
//!
//! ## Educational notes (Rust idioms used here)
//!
//! - Each `pub mod` below is one bounded slice of the API; `pub use`
//!   re-exports flatten them so the generated Swift sees a single module.
//! - Conversions between engine types and FFI records are written as
//!   `From`/`TryFrom` impls — the standard Rust way to say "this type can
//!   be built from that one", and the compiler forces the `match` arms to
//!   stay exhaustive when the engine gains a new enum variant.

// This single macro call generates the crate-level FFI plumbing (the
// "namespace" in UniFFI terms — it defaults to the crate name, vle_ffi).
uniffi::setup_scaffolding!();

pub mod component;
pub mod error;
pub mod steam;
pub mod system;

pub use component::*;
pub use error::*;
pub use steam::*;
pub use system::*;

/// Version of the vle engine compiled into this library.
///
/// The workspace shares one version number across `vle-thermo`,
/// `vle-steam`, and this wrapper, so this string identifies all of them.
/// Use it as the first smoke test from Swift: if `version()` returns,
/// the whole Rust→C→Swift pipeline is alive.
#[uniffi::export]
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
