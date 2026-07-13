//! The single error type every exported function throws.
//!
//! wasm-bindgen maps `Result<T, JsError>` to a JS function that **throws**:
//! the `Err` becomes a JavaScript `Error` whose `message` is this enum's
//! `Display` text. The categories and message prefixes are identical to
//! `ffi/`'s `VleFfiError`, so error handling reads the same in Swift,
//! Kotlin, and JavaScript — match on the prefix:
//!
//! ```js
//! try { sys.flashTp(t, p, z) } catch (e) {
//!   if (e.message.startsWith("invalid input:")) { /* fix the call site */ }
//! }
//! ```
//!
//! Panics need no mapping here: they are always bugs, and the
//! `console_error_panic_hook` installed in `lib.rs` makes them print a
//! readable message before the wasm trap unwinds the JS call.

use vle_thermo::flash::FlashError;

/// Everything that can go wrong across the wasm boundary.
///
/// `thiserror` writes the `Display` impls (the `#[error(...)]` strings);
/// the `From<VleWasmError> for JsError` impl below carries that text to
/// JavaScript as the thrown `Error`'s `message`.
#[derive(Debug, thiserror::Error)]
pub enum VleWasmError {
    /// A component name was not found in the bundled database.
    #[error("component not found in the bundled database: {name}")]
    NotFound { name: String },
    /// The caller passed structurally invalid input (wrong array length,
    /// unknown model name, malformed matrix, …). Fix the call site.
    #[error("invalid input: {message}")]
    InvalidInput { message: String },
    /// A flash / saturation computation failed to converge or is not
    /// supported for the selected model combination.
    #[error("flash calculation failed: {message}")]
    Flash { message: String },
    /// A steam-tables query failed (out of the IF97 validity range,
    /// invalid quality, or a non-converged inner solve).
    #[error("steam tables error: {message}")]
    Steam { message: String },
}

// `From` impls let wrapper code use `?` on engine Results directly.

impl From<FlashError> for VleWasmError {
    fn from(e: FlashError) -> Self {
        match e {
            // Same split as the Python and UniFFI bindings: caller mistakes
            // surface as InvalidInput, numerical trouble as Flash.
            FlashError::Dimension(_)
            | FlashError::Unsupported(_)
            | FlashError::NoRachfordRiceRoot { .. } => VleWasmError::InvalidInput {
                message: e.to_string(),
            },
            _ => VleWasmError::Flash {
                message: e.to_string(),
            },
        }
    }
}

impl From<vle_thermo::steam::SteamError> for VleWasmError {
    fn from(e: vle_thermo::steam::SteamError) -> Self {
        VleWasmError::Steam {
            message: e.to_string(),
        }
    }
}

// The bridge to JavaScript needs no code of ours: wasm-bindgen ships a
// blanket `impl<E: std::error::Error> From<E> for JsError`, and thiserror
// derives `std::error::Error` above. A `?` in an exported
// `Result<_, JsError>` shim therefore throws a JS `Error` whose message is
// exactly the `Display` text above.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_errors_map_to_invalid_input() {
        // The message prefix is the JS-side dispatch key — pin it.
        let e: VleWasmError = FlashError::Dimension("z has 1 entry".into()).into();
        assert!(e.to_string().starts_with("invalid input:"), "{e}");
    }

    #[test]
    fn steam_errors_keep_their_prefix() {
        let bad = vle_thermo::steam::SteamState::tp(5000.0, 101.325).unwrap_err();
        let e: VleWasmError = bad.into();
        assert!(e.to_string().starts_with("steam tables error:"), "{e}");
    }
}
