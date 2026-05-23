//! VLE Units of Measurement
//!
//! Two layers:
//!
//! 1. **Compile-time typed quantities** — use `uom::si::f64::*` directly
//!    (e.g. `ThermodynamicTemperature`, `Pressure`, `MolarEnergy`,
//!    `TemperatureInterval`). `uom` enforces dimensional safety at compile
//!    time at zero runtime cost; we no longer wrap its types in `Vle*`
//!    aliases.
//!
//! 2. **Runtime registry API** ([`registry`], [`parser`], [`toml_loader`]) —
//!    used at the FFI boundary to parse user-supplied unit strings like
//!    `"3.5 barg"` or `"25 degC"`. Extensible at runtime: define new units
//!    or whole derived dimensions without recompiling.
//!
//! See [`docs/en/units/dimensional-analysis.md`] for the full design rationale.

pub mod parser;
pub mod registry;
pub mod toml_loader;

pub use registry::{Dimension, DimensionVector, RegistryError, UnitDef, UnitRegistry};

/// Standard atmospheric pressure in **kPa** (1 standard atm).
///
/// Provided as a convenience constant — it is **never** used as a hidden
/// default anywhere in the engine. All gauge ↔ absolute conversions in the
/// [`UnitRegistry`] read P_atm from a configurable field
/// ([`UnitRegistry::atmospheric_pressure_kpa`]).
pub const P_ATM_STANDARD_KPA: f64 = 101.325;

/// The raw TOML text used by [`UnitRegistry::with_vle_defaults`].
///
/// Baked into the binary at compile time via `include_str!`. Exposed so the
/// Python wrapper (and any other FFI consumer) can derive its own unit
/// definitions from the same source of truth as the Rust engine, instead of
/// duplicating scale/offset constants.
pub fn default_units_toml() -> &'static str {
    include_str!("data/defaults.toml")
}
