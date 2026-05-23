//! Runtime extensible unit registry.
//!
//! Each unit is stored with:
//! - a [`DimensionVector`] (7 SI base-dimension exponents),
//! - a `scale` factor (canonical units per 1 of this unit),
//! - either a constant `offset` or a marker that the offset should be
//!   resolved from the registry's atmospheric pressure (gauge units).
//!
//! Canonical units per dimension (kept aligned with the typed `vle_units`
//! aliases and the legacy VB6/Pascal codebases):
//!
//! | Dimension | Canonical |
//! |-----------|-----------|
//! | Temperature (absolute) | K |
//! | TemperatureDiff (Δ) | K |
//! | Pressure | kPa |
//! | MolarEnergy | kJ/kmol |
//! | MolarEntropy | kJ/(kmol·K) |
//! | MolarVolume | cm³/mol |
//! | Amount | kmol |

use std::collections::HashMap;

use thiserror::Error;

use crate::P_ATM_STANDARD_KPA;

/// Errors returned by registry operations.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("unknown unit: {0}")]
    UnknownUnit(String),
    #[error("unknown dimension: {0}")]
    UnknownDimension(String),
    #[error("unit `{0}` already defined (pass overwrite=true to replace)")]
    AlreadyDefined(String),
    #[error("dimension `{0}` already defined")]
    DimensionAlreadyDefined(String),
    #[error("dimension mismatch: expected {expected:?}, found {found:?}")]
    DimensionMismatch {
        expected: DimensionVector,
        found: DimensionVector,
    },
    #[error("non-positive absolute pressure ({0} kPa) — gauge value is below vacuum")]
    NonPositivePressure(f64),
    #[error("invalid unit string: {0}")]
    ParseError(String),
    #[error("atmospheric pressure must be > 0; got {0} kPa")]
    BadAtmosphericPressure(f64),
}

/// 7-tuple of SI base-dimension exponents `(L, M, T, I, Θ, N, J)`.
///
/// See `docs/en/units/dimensional-analysis.md` §2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DimensionVector(pub [i8; 7]);

impl DimensionVector {
    pub const fn new(exps: [i8; 7]) -> Self {
        DimensionVector(exps)
    }
}

/// Built-in VLE dimensions. Custom dimensions are stored separately in the
/// registry's [`UnitRegistry::dimensions`] table by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    Temperature,          // (0,0,0,0,1,0,0) — absolute
    TemperatureDiff,      // (0,0,0,0,1,0,0) — interval (separate type)
    Pressure,             // (-1,1,-2,0,0,0,0)
    MolarEnergy,          // (2,1,-2,0,0,-1,0)
    MolarEntropy,         // (2,1,-2,0,-1,-1,0)
    MolarVolume,          // (3,0,0,0,0,-1,0)
    Amount,               // (0,0,0,0,0,1,0)
    Custom(&'static str), // not used by built-ins; future hook
}

impl Dimension {
    pub fn name(&self) -> &str {
        match self {
            Dimension::Temperature => "temperature",
            Dimension::TemperatureDiff => "temperature_diff",
            Dimension::Pressure => "pressure",
            Dimension::MolarEnergy => "molar_energy",
            Dimension::MolarEntropy => "molar_entropy",
            Dimension::MolarVolume => "molar_volume",
            Dimension::Amount => "amount",
            Dimension::Custom(s) => s,
        }
    }

    pub fn vector(&self) -> DimensionVector {
        match self {
            Dimension::Temperature | Dimension::TemperatureDiff => {
                DimensionVector::new([0, 0, 0, 0, 1, 0, 0])
            }
            Dimension::Pressure => DimensionVector::new([-1, 1, -2, 0, 0, 0, 0]),
            Dimension::MolarEnergy => DimensionVector::new([2, 1, -2, 0, 0, -1, 0]),
            Dimension::MolarEntropy => DimensionVector::new([2, 1, -2, 0, -1, -1, 0]),
            Dimension::MolarVolume => DimensionVector::new([3, 0, 0, 0, 0, -1, 0]),
            Dimension::Amount => DimensionVector::new([0, 0, 0, 0, 0, 1, 0]),
            Dimension::Custom(_) => DimensionVector::new([0; 7]),
        }
    }

    /// Lookup by string name. Returns `None` for unknown names — callers should
    /// also check the custom dimensions table on the registry.
    pub fn from_name(name: &str) -> Option<Dimension> {
        Some(match name {
            "temperature" => Dimension::Temperature,
            "temperature_diff" => Dimension::TemperatureDiff,
            "pressure" => Dimension::Pressure,
            "molar_energy" => Dimension::MolarEnergy,
            "molar_entropy" => Dimension::MolarEntropy,
            "molar_volume" => Dimension::MolarVolume,
            "amount" => Dimension::Amount,
            _ => return None,
        })
    }
}

/// Source of the affine offset in the conversion `canonical = value*scale + offset`.
#[derive(Debug, Clone, Copy)]
enum OffsetSource {
    /// Constant offset baked into the unit definition (e.g. 273.15 for °C).
    Constant(f64),
    /// Resolve from `registry.atmospheric_pressure_kpa()` at conversion time
    /// (gauge pressure units: barg, psig, kPag, …).
    GaugePAtm,
}

/// A single registered unit.
#[derive(Debug, Clone)]
pub struct UnitDef {
    pub name: String,
    pub dimension_name: String,
    pub dimension_vector: DimensionVector,
    /// Canonical units per 1 of this unit (e.g. for °C → K, scale=1).
    pub scale: f64,
    offset: OffsetSource,
}

impl UnitDef {
    pub fn is_gauge(&self) -> bool {
        matches!(self.offset, OffsetSource::GaugePAtm)
    }

    pub fn constant_offset(&self) -> Option<f64> {
        match self.offset {
            OffsetSource::Constant(o) => Some(o),
            OffsetSource::GaugePAtm => None,
        }
    }
}

/// Runtime, extensible unit registry. Created via [`UnitRegistry::with_vle_defaults`].
#[derive(Debug, Clone)]
pub struct UnitRegistry {
    units: HashMap<String, UnitDef>,
    /// Custom dimensions registered via [`define_dimension`](Self::define_dimension).
    /// Built-in dimensions are recognized via [`Dimension::from_name`].
    dimensions: HashMap<String, DimensionVector>,
    p_atm_kpa: f64,
}

impl UnitRegistry {
    /// Create an empty registry (no units pre-registered). Mostly for testing —
    /// most callers want [`with_vle_defaults`](Self::with_vle_defaults).
    pub fn new() -> Self {
        UnitRegistry {
            units: HashMap::new(),
            dimensions: HashMap::new(),
            p_atm_kpa: P_ATM_STANDARD_KPA,
        }
    }

    /// Create a registry pre-populated with VLE-relevant units (temperature,
    /// pressure including gauge units, molar energy, etc.).
    pub fn with_vle_defaults() -> Self {
        let mut r = Self::new();
        r.install_defaults();
        r
    }

    fn install_defaults(&mut self) {
        // The default unit catalog lives in `data/defaults.toml` and is baked
        // into the binary via `include_str!` — no runtime file I/O, no missing-
        // file failure mode. To add or tweak a built-in unit, edit that file.
        const DEFAULTS_TOML: &str = include_str!("data/defaults.toml");
        self.load_from_toml_str(DEFAULTS_TOML)
            .expect("built-in defaults.toml must parse and apply cleanly");
    }

    fn put(&mut self, u: UnitDef) {
        self.units.insert(u.name.clone(), u);
    }

    // ── Atmospheric pressure (configurable, never hardcoded) ────────────────

    /// Current atmospheric pressure used for gauge ↔ absolute conversions.
    ///
    /// # Returns
    /// Atmospheric pressure in **kPa**.
    pub fn atmospheric_pressure_kpa(&self) -> f64 {
        self.p_atm_kpa
    }

    /// Override atmospheric pressure for all subsequent gauge conversions.
    ///
    /// Default is 101.325 kPa (1 standard atm). Override for non-standard
    /// altitude or weather (e.g. 84.5 kPa at ~1500 m elevation).
    ///
    /// # Arguments
    /// * `p_atm_kpa` — Local atmospheric pressure in **kPa** (must be > 0)
    pub fn set_atmospheric_pressure(&mut self, p_atm_kpa: f64) -> Result<(), RegistryError> {
        // Use `is_finite() && > 0.0` so NaN and ±inf are also rejected. (The
        // legacy `!(x > 0.0)` form trips clippy::neg_cmp_op_on_partial_ord.)
        if !(p_atm_kpa.is_finite() && p_atm_kpa > 0.0) {
            return Err(RegistryError::BadAtmosphericPressure(p_atm_kpa));
        }
        self.p_atm_kpa = p_atm_kpa;
        Ok(())
    }

    // ── User extensions ─────────────────────────────────────────────────────

    /// Define a new unit with a constant scale (and optional constant offset).
    ///
    /// # Arguments
    /// * `name` — Unit symbol (no spaces, must be valid identifier-ish)
    /// * `dimension` — One of the built-in dimensions
    /// * `scale` — Canonical units per 1 of this unit
    /// * `offset` — Constant affine offset (0.0 for pure scale)
    pub fn define(
        &mut self,
        name: &str,
        dimension: Dimension,
        scale: f64,
        offset: f64,
    ) -> Result<(), RegistryError> {
        self.define_with_dimension_name(name, dimension.name(), dimension.vector(), scale, offset)
    }

    /// Define a gauge unit whose offset is resolved from the registry's
    /// atmospheric pressure at conversion time. The unit must be a pressure unit.
    pub fn define_gauge(
        &mut self,
        name: &str,
        scale_kpa_per_unit: f64,
    ) -> Result<(), RegistryError> {
        if self.units.contains_key(name) {
            return Err(RegistryError::AlreadyDefined(name.into()));
        }
        self.put(UnitDef {
            name: name.into(),
            dimension_name: "pressure".into(),
            dimension_vector: Dimension::Pressure.vector(),
            scale: scale_kpa_per_unit,
            offset: OffsetSource::GaugePAtm,
        });
        Ok(())
    }

    /// Register a brand-new derived dimension (not in the built-in list).
    pub fn define_dimension(
        &mut self,
        name: &str,
        vector: DimensionVector,
    ) -> Result<(), RegistryError> {
        if Dimension::from_name(name).is_some() || self.dimensions.contains_key(name) {
            return Err(RegistryError::DimensionAlreadyDefined(name.into()));
        }
        self.dimensions.insert(name.into(), vector);
        Ok(())
    }

    /// Define a unit attached to a previously-registered custom dimension.
    pub fn define_with_dimension(
        &mut self,
        name: &str,
        dimension_name: &str,
        scale: f64,
        offset: f64,
    ) -> Result<(), RegistryError> {
        let vector = self
            .lookup_dimension(dimension_name)
            .ok_or_else(|| RegistryError::UnknownDimension(dimension_name.into()))?;
        self.define_with_dimension_name(name, dimension_name, vector, scale, offset)
    }

    fn define_with_dimension_name(
        &mut self,
        name: &str,
        dim_name: &str,
        vector: DimensionVector,
        scale: f64,
        offset: f64,
    ) -> Result<(), RegistryError> {
        if self.units.contains_key(name) {
            return Err(RegistryError::AlreadyDefined(name.into()));
        }
        self.put(UnitDef {
            name: name.into(),
            dimension_name: dim_name.into(),
            dimension_vector: vector,
            scale,
            offset: OffsetSource::Constant(offset),
        });
        Ok(())
    }

    fn lookup_dimension(&self, name: &str) -> Option<DimensionVector> {
        Dimension::from_name(name)
            .map(|d| d.vector())
            .or_else(|| self.dimensions.get(name).copied())
    }

    // ── Lookups ─────────────────────────────────────────────────────────────

    pub fn get(&self, name: &str) -> Result<&UnitDef, RegistryError> {
        self.units
            .get(name)
            .ok_or_else(|| RegistryError::UnknownUnit(name.into()))
    }

    /// Iterate all registered unit names. Order is unspecified.
    pub fn unit_names(&self) -> impl Iterator<Item = &str> {
        self.units.keys().map(|s| s.as_str())
    }

    // ── Conversion ──────────────────────────────────────────────────────────

    fn offset_value(&self, u: &UnitDef) -> f64 {
        match u.offset {
            OffsetSource::Constant(o) => o,
            OffsetSource::GaugePAtm => self.p_atm_kpa,
        }
    }

    /// Convert `value` (in `unit_name`) into canonical units for that unit's
    /// dimension. For gauge pressure units, the result is **absolute kPa**.
    pub fn to_canonical(&self, value: f64, unit_name: &str) -> Result<f64, RegistryError> {
        let u = self.get(unit_name)?;
        let result = value * u.scale + self.offset_value(u);
        // Reject non-physical absolute pressures from gauge inputs
        if u.is_gauge() && result <= 0.0 {
            return Err(RegistryError::NonPositivePressure(result));
        }
        Ok(result)
    }

    /// Inverse of [`to_canonical`](Self::to_canonical): convert a canonical
    /// value into `unit_name`'s scale.
    pub fn from_canonical(
        &self,
        value_canonical: f64,
        unit_name: &str,
    ) -> Result<f64, RegistryError> {
        let u = self.get(unit_name)?;
        Ok((value_canonical - self.offset_value(u)) / u.scale)
    }

    /// Parse `"<value> <unit>"` and convert to canonical units.
    /// See [`crate::parser::split_value_unit`].
    pub fn parse(&self, s: &str) -> Result<Quantity, RegistryError> {
        let (value, unit) = crate::parser::split_value_unit(s)?;
        let canonical = self.to_canonical(value, unit)?;
        let u = self.get(unit)?;
        Ok(Quantity {
            canonical,
            dimension: u.dimension_name.clone(),
        })
    }

    /// Format a canonical value as `"<value> <unit>"` (no rounding).
    pub fn format(&self, q: &Quantity, unit_name: &str) -> Result<String, RegistryError> {
        let u = self.get(unit_name)?;
        if u.dimension_name != q.dimension {
            return Err(RegistryError::DimensionMismatch {
                expected: self
                    .lookup_dimension(&q.dimension)
                    .unwrap_or(DimensionVector::new([0; 7])),
                found: u.dimension_vector,
            });
        }
        let v = self.from_canonical(q.canonical, unit_name)?;
        Ok(format!("{v} {unit_name}"))
    }
}

impl Default for UnitRegistry {
    fn default() -> Self {
        Self::with_vle_defaults()
    }
}

/// A dimensioned value carried across the FFI boundary in canonical units.
#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    pub canonical: f64,
    pub dimension: String,
}

impl Quantity {
    pub fn value_kpa(&self) -> f64 {
        debug_assert_eq!(self.dimension, "pressure");
        self.canonical
    }
    pub fn value_kelvin(&self) -> f64 {
        debug_assert_eq!(self.dimension, "temperature");
        self.canonical
    }
}
