//! TOML loader for bulk user-defined units.
//!
//! Schema (see `docs/en/units/dimensional-analysis.md` §7.2):
//!
//! ```toml
//! [[unit]]
//! name = "mmH2O"
//! dimension = "pressure"
//! scale = 0.00980665
//! offset = 0.0          # optional, default 0.0
//! gauge = false         # optional, default false; if true, offset = registry P_atm
//!
//! [[dimension]]
//! name = "heat_transfer_coefficient"
//! exponents = [0, 1, -3, 0, -1, 0, 0]
//! ```

use serde::Deserialize;

use crate::registry::{Dimension, DimensionVector, RegistryError, UnitRegistry};

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    dimension: Vec<DimensionEntry>,
    #[serde(default)]
    unit: Vec<UnitEntry>,
}

#[derive(Debug, Deserialize)]
struct DimensionEntry {
    name: String,
    exponents: [i8; 7],
}

#[derive(Debug, Deserialize)]
struct UnitEntry {
    name: String,
    dimension: String,
    scale: f64,
    #[serde(default)]
    offset: f64,
    #[serde(default)]
    gauge: bool,
}

impl UnitRegistry {
    /// Parse a TOML string and apply its `[[dimension]]` and `[[unit]]` entries
    /// to this registry. Dimensions are added before units so units can reference
    /// freshly-defined custom dimensions.
    pub fn load_from_toml_str(&mut self, toml_text: &str) -> Result<(), RegistryError> {
        let file: File = toml::from_str(toml_text)
            .map_err(|e| RegistryError::ParseError(format!("TOML: {e}")))?;
        for d in file.dimension {
            self.define_dimension(&d.name, DimensionVector::new(d.exponents))?;
        }
        for u in file.unit {
            if u.gauge {
                // Gauge units only meaningful for pressure
                if u.dimension != "pressure" {
                    return Err(RegistryError::ParseError(format!(
                        "gauge=true is only valid for pressure (unit `{}` has dimension `{}`)",
                        u.name, u.dimension
                    )));
                }
                self.define_gauge(&u.name, u.scale)?;
            } else if let Some(d) = Dimension::from_name(&u.dimension) {
                self.define(&u.name, d, u.scale, u.offset)?;
            } else {
                self.define_with_dimension(&u.name, &u.dimension, u.scale, u.offset)?;
            }
        }
        Ok(())
    }

    /// Convenience: read TOML from `path` and apply.
    pub fn load_from_toml(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), RegistryError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| RegistryError::ParseError(format!("read: {e}")))?;
        self.load_from_toml_str(&text)
    }
}
