//! VLE Units of Measurement (Milestone 3)
//!
//! Two parallel APIs:
//!
//! 1. **Compile-time typed API** (`vle_units` module) — re-exports of `uom`'s SI
//!    quantities tailored to VLE canonical units (K, kPa, kJ/kmol, etc.). Use
//!    inside the engine for zero-cost dimension safety.
//!
//! 2. **Runtime registry API** (`registry`, `parser`, `toml_loader`) — used at
//!    the FFI boundary to parse user-supplied unit strings (e.g. `"3.5 barg"`,
//!    `"25 degC"`). Extensible at runtime: users can `define()` new units and
//!    even new derived dimensions without recompiling.
//!
//! See [`docs/en/units/dimensional-analysis.md`] for the full design rationale.

pub mod vle_units;
pub mod registry;
pub mod parser;
pub mod toml_loader;

pub use registry::{Dimension, DimensionVector, RegistryError, UnitDef, UnitRegistry};
pub use vle_units::{
    P_ATM_STANDARD_KPA, VleAmount, VleMolarEnergy, VleMolarEntropy, VleMolarVolume, VlePressure,
    VleTemperature, VleTemperatureDiff,
};
