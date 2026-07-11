//! Swift bindings generator for `vle-ffi` (the standard UniFFI pattern).
//!
//! UniFFI runs in "library mode": this binary reads the *compiled*
//! `libvle_ffi.a`, finds the interface metadata the proc-macros embedded in
//! it, and emits the Swift wrapper (`vle_ffi.swift`), the C header
//! (`vle_ffiFFI.h`), and the `module.modulemap`. Because it reads the built
//! artifact, the bindings can never be out of sync with the code.

fn main() {
    uniffi::uniffi_bindgen_swift()
}
