# vle-thermo

Vapor-liquid equilibrium (VLE) thermodynamic calculator in Rust.

[![Crates.io](https://img.shields.io/crates/v/vle-thermo.svg)](https://crates.io/crates/vle-thermo)
[![Docs.rs](https://docs.rs/vle-thermo/badge.svg)](https://docs.rs/vle-thermo)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A modern Rust port of two legacy thermodynamic codebases (VB6 ~15,000 lines + Pascal ~2,500 lines) supporting:

- **22+ cubic equations of state** — Peng-Robinson, RKS, van der Waals, Schmidt-Wenzel, Patel-Teja, and more
- **5 activity coefficient models** — Wilson, van Laar, Margules, Scatchard-Hildebrand, Ideal
- **11 mixing rules** — Classical (IVDW, IIVDW), Wong-Sandler, Huron-Vidal, MHV1/MHV2
- **6 saturation pressure correlations** — Antoine, Riedel, Müller, RPM, polynomial, Maxwell
- **6 flash calculation types** — bubble/dew point (T/P), isothermal, adiabatic (WIP)
- **Parameter regression** — kij (binary interaction) and Aij (activity model) (WIP)

## Status

`0.1.x` — types, enums, and data structures are stable. Numerical kernels (Cardano
solver, Newton-Raphson, Broyden, Rachford-Rice, flash algorithms) are under active
development across Milestones 5–7 of the modernization plan. Semver promises do
**not** apply until 1.0; treat `0.1.x` as a pre-release.

See the [roadmap](https://github.com/miguelju/vle/blob/main/ROADMAP.md) for
what's shipped vs. planned, and the
[modernization plan](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md)
for the phase-by-phase technical detail.

## Install

```toml
[dependencies]
vle-thermo = "0.1"
```

Or with `cargo add`:

```sh
cargo add vle-thermo
```

The crate is `no-pyo3` by default — PyO3 bindings are gated behind the optional
`python` feature and are only needed when maturin builds the Python wheel.

## Quick look

```rust
use vle_thermo::{CubicEos, ActivityModel, MixingRule, SatPressureModel};

let eos = CubicEos::PengRobinson;
let activity = ActivityModel::Wilson;
let mixing = MixingRule::WongSandler;
let psat = SatPressureModel::Antoine;

println!("using {eos:?} + {activity:?} + {mixing:?} + {psat:?}");
```

Full API docs: <https://docs.rs/vle-thermo>.

## Units

Numerical kernels use the thesis's canonical internal units:

| Quantity      | Unit             |
|---------------|------------------|
| Temperature   | K (absolute)     |
| Pressure      | kPa (absolute)   |
| Molar energy  | kJ/kmol          |
| Molar entropy | kJ/(kmol·K)      |
| Molar volume  | cm³/mol          |
| Amount        | kmol             |

For user-facing input/output in arbitrary units (including gauge pressure, °C,
°F, psi, barg, mmHg, etc.) see the companion crate
[`vle-units`](https://crates.io/crates/vle-units).

## Origin

Based on the thesis *"Desarrollo de un Programa Computacional para el Cálculo
del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente
Windows"* (Jackson & Mendible, Universidad Simón Bolívar, 1999), with
additional models from Da Silva & Báez (1989). See the
[research paper](https://github.com/miguelju/vle/tree/main/docs/en/research-paper)
(English translation) for the algorithms and their academic references.

## License

MIT. See [LICENSE](https://github.com/miguelju/vle/blob/main/LICENSE).
