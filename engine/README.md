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
- **6 flash calculation types** — bubble/dew point (T/P), isothermal, adiabatic (PH)
- **Parameter regression** — kij (binary interaction) and Aij (activity model)

## Status

`0.10.0` — pre-1.0, but the numerical core is live. The Cardano solver,
Newton-Raphson / Broyden drivers, Rachford-Rice, the flash algorithms
(isothermal, bubble/dew *T* and *P*, adiabatic *PH*), the mixture critical
point, and kij/Aij regression all run. `0.10.0` adds **IAPWS-IF97 steam
tables** via the new sibling crate [`vle-steam`](https://crates.io/crates/vle-steam),
re-exported here as `vle_thermo::steam` behind the optional `steam` feature
(on by default in the Python wheel). `0.9.1` fixed a ~1% Wong-Sandler
departure-enthalpy inconsistency (`energy::h_departure_rt_mix` dropped the
T-dependence of the WS co-volume; the Gibbs–Helmholtz identity now holds to
machine precision for every mixing rule). `0.9.0` added **exact temperature /
pressure derivatives** of fugacity and K-values (`mixture::d_ln_phi_d_t` /
`d_ln_phi_d_p`, `flash::k_values_with_derivs`, dual-number AD), **real-mixture
`energy::phase_cp`** and **`partial_molar_enthalpy`**, and an optional
bundled component database (the `component-db` feature — see below), with
one deliberate breaking change: the T/P-generic core signatures
(`mixture::mixture_params` and friends now take `t: D, p: D`). Semver promises
do **not** apply until 1.0; treat `0.x` as a pre-release.

See the [roadmap](https://github.com/miguelju/vle/blob/main/ROADMAP.md) for
what's shipped vs. planned, and the
[modernization plan](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md)
for the phase-by-phase technical detail.

## Install

```toml
[dependencies]
vle-thermo = "0.10"
```

Or with `cargo add`:

```sh
cargo add vle-thermo
```

The crate is `no-pyo3` by default — PyO3 bindings are gated behind the optional
`python` feature and are only needed when maturin builds the Python wheel.

Opt into the bundled 24-compound property database with the `component-db`
feature (`cargo add vle-thermo --features component-db`), which adds
`vle_thermo::db::component(name)` / `available()` for name-based lookups
(pulls only `serde` + `serde_json`; off by default).

## Quick look

Compute the vapour-phase compressibility factor of methane with Peng-Robinson.
All numerical kernels use the canonical internal units below (K, kPa):

```rust
use vle_thermo::eos::{z_factor, PhaseId};
use vle_thermo::{Component, CubicEos};

let methane = Component {
    name: "methane".into(),
    tc: 190.564, // K
    pc: 4599.0,  // kPa
    omega: 0.0115,
    ..Component::default()
};

// Vapour root at 200 K, 5000 kPa
let z = z_factor(CubicEos::PR1976, 200.0, 5000.0, &methane, PhaseId::Vapor)
    .expect("Z-factor converges");
println!("Z_vapor = {z:.4}"); // ≈ 0.5234
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
