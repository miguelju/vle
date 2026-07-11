//! The single error type every exported function throws.
//!
//! UniFFI maps `Result<T, VleFfiError>` to a Swift `throws` function, and
//! `#[derive(uniffi::Error)]` turns this enum into a Swift `enum … : Error`
//! with one case per variant (fields become associated values). Swift code
//! can then `catch VleFfiError.NotFound(let name)` etc.
//!
//! Panics need no mapping here: UniFFI's generated scaffolding catches any
//! Rust panic at the FFI boundary and surfaces it as a Swift error instead
//! of aborting the app (which is why the workspace keeps the default
//! unwinding panic strategy — see IOS_FFI_PLAN.md §3).

use vle_thermo::flash::FlashError;

/// Everything that can go wrong across the FFI boundary.
///
/// `thiserror` writes the `Display` impls (the `#[error(...)]` strings);
/// UniFFI carries that text to Swift as the error's `message`.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum VleFfiError {
    /// A component name was not found in the bundled database.
    #[error("component not found in the bundled database: {name}")]
    NotFound { name: String },
    /// The caller passed structurally invalid input (wrong array length,
    /// unknown selection, malformed matrix, …). Fix the call site.
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

// `From` impls let wrapper code use `?` on engine Results directly —
// `engine_call().map_err(VleFfiError::from)?` collapses to just `?`.

impl From<FlashError> for VleFfiError {
    fn from(e: FlashError) -> Self {
        match e {
            // Same split as the Python bindings (py_system::flash_err):
            // caller mistakes surface as InvalidInput, numerical trouble
            // as Flash.
            FlashError::Dimension(_)
            | FlashError::Unsupported(_)
            | FlashError::NoRachfordRiceRoot { .. } => VleFfiError::InvalidInput {
                message: e.to_string(),
            },
            _ => VleFfiError::Flash {
                message: e.to_string(),
            },
        }
    }
}

impl From<vle_thermo::steam::SteamError> for VleFfiError {
    fn from(e: vle_thermo::steam::SteamError) -> Self {
        VleFfiError::Steam {
            message: e.to_string(),
        }
    }
}
