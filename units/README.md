# vle-units

Dimensional analysis and unit conversion for vapor-liquid equilibrium (VLE) and
thermodynamic calculations.

[![Crates.io](https://img.shields.io/crates/v/vle-units.svg)](https://crates.io/crates/vle-units)
[![Docs.rs](https://docs.rs/vle-units/badge.svg)](https://docs.rs/vle-units)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Works standalone. Designed as the unit layer for
[`vle-thermo`](https://crates.io/crates/vle-thermo) but useful in any Rust
project that needs thermodynamic-flavored units (gauge pressure, °C / °F,
kJ/kmol, barg, psig, mmHg, etc.).

## Features

- **Compile-time typed quantities** via `uom` — zero runtime cost, full
  dimension safety. Canonical quantity aliases match VLE conventions
  (`VleTemperature`, `VlePressure`, `VleMolarEnergy`, …).
- **Runtime unit registry** — parse user-supplied strings like `"25 degC"`,
  `"3.5 barg"`, `"1 atm"` at the FFI boundary.
- **Gauge ↔ absolute conversion** with a **runtime-configurable atmospheric
  pressure** (never hardcoded). Default: 101.325 kPa.
- **Extensible** — define custom units and derived dimensions at runtime (TOML
  or direct API) without recompiling.

## Install

```toml
[dependencies]
vle-units = "0.1"
```

## Example

```rust
use vle_units::UnitRegistry;

let mut reg = UnitRegistry::with_vle_defaults();

// Parse user input in arbitrary units.
let t = reg.parse("25 degC")?;
let p = reg.parse("3.5 barg")?;

assert!((t.value_kelvin() - 298.15).abs() < 1e-9);
assert!((p.value_kpa()    - 451.325).abs() < 1e-9); // gauge + P_atm

// Configure atmospheric pressure for a different location / altitude.
reg.set_atmospheric_pressure(95.0)?;
# Ok::<(), vle_units::RegistryError>(())
```

Full API docs: <https://docs.rs/vle-units>.

## Why a separate crate?

Thermodynamics code needs a few things that generic unit libraries don't handle
cleanly:

1. **Gauge pressure** (`barg`, `psig`, `kPag`) where absolute = gauge + P_atm, and
   P_atm is a **runtime** parameter the operator can set.
2. **Temperature differences vs. absolute temperatures** (`ΔT in K vs. T in °C`
   — `uom` models these as distinct dimensions to prevent misuse).
3. **Molar units** (`kJ/kmol`, `cm³/mol`) that map to the canonical internal
   units used by cubic-EOS codebases going back to the 1980s.

See [`docs/en/units/dimensional-analysis.md`](https://github.com/miguelju/vle/blob/main/docs/en/units/dimensional-analysis.md)
for the full design rationale.

## License

MIT. See [LICENSE](https://github.com/miguelju/vle/blob/main/LICENSE).
