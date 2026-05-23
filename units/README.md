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

- **Compile-time typed quantities** via [`uom`](https://crates.io/crates/uom) —
  zero runtime cost, full dimension safety. Use `uom::si::f64::*` directly
  (`ThermodynamicTemperature`, `Pressure`, `MolarEnergy`, `TemperatureInterval`, …);
  the crate's canonical units (K, kPa, kJ/kmol, K) match VLE conventions.
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

## How it works

`vle-units` is two thin layers stacked on top of [`uom`](https://crates.io/crates/uom):

```
                                ┌────────────────────────────────────────┐
   compile-time                 │  uom::si::f64::*                       │
   (zero runtime cost)          │  Pressure, ThermodynamicTemperature,   │
                                │  MolarEnergy, TemperatureInterval, …   │
                                │  — dimension mismatches caught by rustc│
                                └────────────────────────────────────────┘
                                                ▲
                                                │  values enter in canonical
                                                │  units (K, kPa, kJ/kmol, …)
                                                │
                                ┌────────────────────────────────────────┐
   runtime                      │  vle_units::UnitRegistry               │
   (string-driven, extensible)  │  • parses "3.5 barg", "25 degC", …     │
                                │  • runtime-configurable P_atm for      │
                                │    gauge ↔ absolute pressure           │
                                │  • define() new units / load TOML      │
                                │  • exposed to Python via PyO3 so       │
                                │    `pint` shares the same source       │
                                └────────────────────────────────────────┘
```

**Why `uom` underneath.** `uom` is the most mature Rust units library: ~50
quantity types, ~500 unit definitions, dimension checking via the type system
(`typenum`-encoded SI exponent vectors), and zero runtime overhead — a
`Pressure<f64>` compiles to the same code as a bare `f64`. We get all of that
"for free" and use `uom`'s types directly in engine code, without wrapping
them in our own aliases.

**Why a layer on top.** `uom` is purely compile-time. It can't:

- Parse a string like `"3.5 barg"` that arrived from a Jupyter notebook, a
  TOML config, or a CLI flag.
- Carry a *runtime-configurable* offset — gauge pressure depends on
  atmospheric pressure, which the operator must be able to override (84.5 kPa
  at 1500 m elevation, etc.).
- Let downstream users register a brand-new unit without recompiling the
  library.

The `UnitRegistry` provides exactly those three things, and nothing more.
Once a value crosses into the engine it's a plain `f64` in canonical units,
ready to be wrapped in a `uom` quantity for type-safe algebra.

## Defining custom dimensions and units

The defaults cover the VLE quantities (temperature, pressure, molar
energy/entropy/volume, amount). To work with any other dimension — length,
mass, time, current, luminous intensity, or anything derived — define it on
the registry at runtime. **Length** is a good walkthrough because it isn't
in the defaults:

```rust
use vle_units::{DimensionVector, UnitRegistry};

let mut reg = UnitRegistry::with_vle_defaults();

// SI exponent vector is (L, M, T, I, Θ, N, J). Length is L^1.
reg.define_dimension("length", DimensionVector::new([1, 0, 0, 0, 0, 0, 0]))?;

// `scale` is meters per 1 of this unit. `offset = 0.0` for everything that
// isn't affine (°C/°F have a non-zero offset, length units don't).
for (name, meters_per_unit) in [
    ("m",     1.0),
    ("mm",    1.0e-3),
    ("cm",    1.0e-2),
    ("km",    1.0e3),
    ("in",    0.0254),     // exact, by 1959 international agreement
    ("ft",    0.3048),     // 12 in
    ("yd",    0.9144),     // 3 ft
    ("mi",    1609.344),   // 5280 ft
    ("nmi",   1852.0),     // international nautical mile
] {
    reg.define_with_dimension(name, "length", meters_per_unit, 0.0)?;
}

// Round-trips are now possible across every registered unit.
let one_mile  = reg.parse("1 mi")?;
let in_meters = reg.from_canonical(one_mile.canonical, "m")?;
let in_yards  = reg.from_canonical(one_mile.canonical, "yd")?;
assert!((in_meters - 1609.344).abs() < 1e-9);
assert!((in_yards  - 1760.0   ).abs() < 1e-9);
# Ok::<(), vle_units::RegistryError>(())
```

A few things worth noting:

- The `DimensionVector` is the SI 7-tuple — you can build any derived
  dimension by combining exponents (e.g. velocity = `[1, 0, -1, 0, 0, 0, 0]`,
  force = `[1, 1, -2, 0, 0, 0, 0]`).
- Aliasing a unit is just registering it twice under different names with
  the same scale (e.g. `"mi"` and `"mile"`, `"in"` and `"inch"`).
- The same data can be loaded from a TOML file via
  [`load_from_toml`](https://docs.rs/vle-units/latest/vle_units/struct.UnitRegistry.html#method.load_from_toml)
  if you'd rather keep the catalog out of source code — see
  [`src/data/defaults.toml`](https://github.com/miguelju/vle/blob/main/units/src/data/defaults.toml)
  for the format.

## Mini CLIs

Two ready-to-run demos ship in [`examples/`](examples/). Both follow the same
~30-line shape — build a `UnitRegistry`, register the dimension and units, then
call `reg.parse(&argv[1])?` and `reg.from_canonical(q.canonical, &argv[2])?` —
so reading both side-by-side is the fastest way to see how the pattern
generalizes.

### `length_convert` — a base-dimension example

[`examples/length_convert.rs`](examples/length_convert.rs) covers the simplest
case: a single SI base dimension (length, L¹).

```sh
$ cargo run -p vle-units --example length_convert -- "1 mile" m
1 mile = 1609.344 m

$ cargo run -p vle-units --example length_convert -- "100 yd" ft
100 yd = 300 ft

$ cargo run -p vle-units --example length_convert -- "2.5 km" mi
2.5 km = 1.5534279805933349 mi

$ cargo run -p vle-units --example length_convert -- "12 in" cm
12 in = 30.479999999999997 cm
```

### `mass_flow_convert` — a derived-dimension example

[`examples/mass_flow_convert.rs`](examples/mass_flow_convert.rs) shows the same
flow for a *derived* dimension — mass ÷ time (M¹·T⁻¹) — with a realistic
catalog covering SI metric (`kg/s`, `kg/h`, `g/s`, `t/h`, …), US customary
(`lb/s`, `lb/h`, `lb/day`, `oz/s`, `ston/h`, …), and the engineering shorthand
you see on heat-balance sheets and refinery PFDs (`klb/h`, `MMlb/day`).

```sh
$ cargo run -p vle-units --example mass_flow_convert -- "10 kg/s" lb/h
10 kg/s = 79366.41438655593 lb/h

$ cargo run -p vle-units --example mass_flow_convert -- "1 MMlb/day" t/h
1 MMlb/day = 18.89968208333333 t/h

$ cargo run -p vle-units --example mass_flow_convert -- "500 klb/h" kg/s
500 klb/h = 62.99894027777778 kg/s

$ cargo run -p vle-units --example mass_flow_convert -- "60 kg/min" kg/h
60 kg/min = 3600 kg/h
```

The two examples together demonstrate the full pattern — base or derived
dimension, SI or imperial, simple scale-only units or affine. The same shape
applies to anything else you'd add (volumetric flow, heat flux, viscosity,
…): pick an SI exponent vector, register it, then register your units.

## Why a separate crate?

Thermodynamics code needs a few things on top of a general units library:

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
