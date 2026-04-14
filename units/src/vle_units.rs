//! Compile-time typed quantity aliases over `uom`.
//!
//! These types let the engine work with dimensioned values that the Rust
//! compiler will reject if mixed (e.g. `temperature + pressure` won't compile).
//! Conversions to/from the canonical VLE units (K, kPa, kJ/kmol, kJ/(kmol·K),
//! cm³/mol, kmol) happen at the API boundary; internal math uses raw `f64`
//! in canonical units for performance.
//!
//! See `docs/en/units/dimensional-analysis.md` §3.3, §3.4, §6.1.

use uom::si::f64::{
    AmountOfSubstance, MolarEnergy, MolarHeatCapacity, MolarVolume, Pressure, TemperatureInterval,
    ThermodynamicTemperature,
};

/// Absolute temperature (state variable).
///
/// Canonical unit: **K** (kelvin). Conversion from °C/°F includes an offset.
/// **Cannot** be added to another absolute temperature (use `VleTemperatureDiff`
/// for differences).
///
/// Compile-time safety — the following must NOT compile:
///
/// ```compile_fail
/// use vle_units::{VleTemperature, VlePressure};
/// use uom::si::thermodynamic_temperature::kelvin;
/// use uom::si::pressure::kilopascal;
/// let t = VleTemperature::new::<kelvin>(300.0);
/// let p = VlePressure::new::<kilopascal>(101.325);
/// let _bad = t + p; // dimension mismatch — rejected by uom
/// ```
///
/// Adding two absolute temperatures must also fail:
///
/// ```compile_fail
/// use vle_units::VleTemperature;
/// use uom::si::thermodynamic_temperature::kelvin;
/// let t1 = VleTemperature::new::<kelvin>(300.0);
/// let t2 = VleTemperature::new::<kelvin>(400.0);
/// let _bad = t1 + t2; // affine + affine is meaningless
/// ```
///
pub type VleTemperature = ThermodynamicTemperature;

/// Temperature difference / interval / gradient (ΔT).
///
/// Canonical unit: **K** (= ΔK). Conversion from Δ°C/Δ°F is **scale-only** —
/// the offset cancels (see dimensional-analysis.md §3.3.2). A separate type
/// from `VleTemperature` so the compiler enforces correct semantics.
pub type VleTemperatureDiff = TemperatureInterval;

/// Absolute pressure. Canonical unit: **kPa**.
///
/// Note: `uom`'s typed pressure has no built-in concept of "gauge" — gauge
/// conversions live in the runtime [`crate::UnitRegistry`] (which carries a
/// configurable atmospheric pressure offset, see §3.4).
pub type VlePressure = Pressure;

/// Molar energy (H, G). Canonical unit: **kJ/kmol** (= J/mol numerically).
pub type VleMolarEnergy = MolarEnergy;

/// Molar entropy (S) and heat capacity (Cp). Canonical unit: **kJ/(kmol·K)**.
/// The `K` here is a *temperature difference*, not absolute (see §3.3).
pub type VleMolarEntropy = MolarHeatCapacity;

/// Molar volume. Canonical unit: **cm³/mol** (= m³/kmol numerically).
pub type VleMolarVolume = MolarVolume;

/// Amount of substance. Canonical unit: **kmol**.
pub type VleAmount = AmountOfSubstance;

/// Standard atmospheric pressure in **kPa** (1 standard atm).
///
/// This constant is provided for convenience — it is **never** used as a
/// hidden default. All gauge ↔ absolute conversions in the
/// [`crate::UnitRegistry`] read P_atm from a configurable field
/// (`registry.atmospheric_pressure_kpa()`).
pub const P_ATM_STANDARD_KPA: f64 = 101.325;
