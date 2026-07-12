//! General (multi-language) bindings generator for `vle-ffi`.
//!
//! Same pattern as the Swift runner in `main.rs`, but exposing uniffi's
//! *general* CLI, whose `generate --language kotlin` subcommand emits the
//! Kotlin wrapper used by the Android / Compose Desktop library
//! (`kotlin/VleThermo`). Library mode again: it reads the interface
//! metadata out of the *compiled* `libvle_ffi.dylib` (or `.so`), so the
//! bindings can never be out of sync with the code.
//!
//! Invoked by `scripts/build-android.sh`; see `docs/en/android/README.md`.

fn main() {
    uniffi::uniffi_bindgen_main()
}
