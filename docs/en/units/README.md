# Units of Measurement — Quickstart

The VLE Units add-on (Milestone 3) provides dimensional analysis and unit
conversion for the engine and the Python wrapper. It ships with the units
needed for VLE work and is **extensible at runtime** — users can add custom
units (or whole new derived dimensions) without recompiling.

For the full design rationale (7 SI base dimensions, phantom types in Rust,
absolute vs. difference temperature, gauge vs. absolute pressure with
configurable atmospheric offset, custom-unit extension), see
[`dimensional-analysis.md`](dimensional-analysis.md).

---

## Rust (`vle-units` crate)

```rust
use vle_units::{Dimension, UnitRegistry};

let mut registry = UnitRegistry::with_vle_defaults();

// 25 °C → 298.15 K (canonical)
let t = registry.parse("25 degC")?;
assert!((t.value_kelvin() - 298.15).abs() < 1e-9);

// 3.5 barg → 451.325 kPa (absolute, at default 101.325 kPa atmosphere)
let p = registry.parse("3.5 barg")?;
assert!((p.value_kpa() - 451.325).abs() < 1e-6);

// Plant at altitude — change atmospheric pressure
registry.set_atmospheric_pressure(84.5)?;
let p2 = registry.parse("3.5 barg")?;          // → 434.5 kPa absolute

// Add a custom unit within an existing dimension
registry.define("mmH2O", Dimension::Pressure, 0.009_806_65, 0.0)?;
let p3 = registry.parse("1000 mmH2O")?;        // → 9.80665 kPa
```

Compile-time dimension safety (zero runtime cost) is provided by `uom`:

```rust
use vle_units::{VlePressure, VleTemperature};
use uom::si::{thermodynamic_temperature::kelvin, pressure::kilopascal};

let t = VleTemperature::new::<kelvin>(300.0);
let p = VlePressure::new::<kilopascal>(101.325);
// let bad = t + p;  // compile error — dimension mismatch
```

`T + T` is also rejected at compile time (an absolute temperature plus an
absolute temperature is meaningless). `T - T` produces a `VleTemperatureDiff`,
which is the type used for ΔT, Cp denominators, and heat-transfer coefficients.

### TOML bulk import

```toml
# custom_units.toml
[[unit]]
name   = "mmH2O"
dimension = "pressure"
scale  = 0.00980665           # kPa per mmH2O

[[unit]]
name   = "atg"
dimension = "pressure"
scale  = 101.325
gauge  = true                 # offset from registry P_atm at runtime

[[dimension]]
name      = "heat_transfer_coefficient"
exponents = [0, 1, -3, 0, -1, 0, 0]   # L M T I Θ N J
```

```rust
registry.load_from_toml("custom_units.toml")?;
```

---

## Python (`vle.units` module)

```python
from vle.units import to_canonical, parse, set_atmospheric_pressure, ureg

to_canonical(25.0, "degC", "temperature")     # → 298.15
parse("3.5 barg", "pressure")                  # → 451.325 (kPa absolute)

# Plant at altitude
set_atmospheric_pressure(84.5)
parse("3.5 barg", "pressure")                  # → 434.5

# Add a custom unit (uses pint directly)
ureg.define("mmH2O = 9.80665 * pascal")
to_canonical(1000.0, "mmH2O", "pressure")      # → 9.80665

# Custom derived quantity — pint composes automatically
import vle.units as u
h = 150 * u.ureg.watt / (u.ureg.meter**2 * u.ureg.kelvin)
h.to("BTU / (hour * foot**2 * degR)")
```

Runtime dimension safety:

```python
from vle.units import Q_
t = Q_(300, "kelvin")
p = Q_(101.325, "kPa")
t + p   # raises pint.DimensionalityError
```

---

## Canonical units

The library always returns canonical values for the engine. These match the
legacy VB6/Pascal codebases:

| Quantity         | Canonical       |
|------------------|-----------------|
| Temperature      | **K**           |
| Temperature diff | **K** (= ΔK)    |
| Pressure         | **kPa** absolute|
| Molar energy     | **kJ/kmol**     |
| Molar entropy    | **kJ/(kmol·K)** |
| Molar volume     | **cm³/mol**    |
| Amount           | **kmol**        |

Default atmospheric pressure for gauge conversions: **101.325 kPa**
(1 standard atm). Configurable via `set_atmospheric_pressure()` /
`registry.set_atmospheric_pressure()`.

---

## Tests

- Rust: `cd units && cargo test` — 18 integration + 3 unit + 2 compile-fail
  doctests
- Python: `PYTHONPATH=python/src python -m pytest python/tests/test_units.py
  python/tests/test_units_parity.py` — 54 tests including a golden-value
  parity table against the Rust crate
