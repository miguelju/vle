# vle-steam

IAPWS-IF97 industrial water/steam thermodynamic properties — steam tables as a
dependency-free Rust crate.

[![Crates.io](https://img.shields.io/crates/v/vle-steam.svg)](https://crates.io/crates/vle-steam)
[![Docs.rs](https://docs.rs/vle-steam/badge.svg)](https://docs.rs/vle-steam)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A from-scratch implementation of the **IAPWS Industrial Formulation 1997**
(*Revised Release* R7-97, 2012) for the thermodynamic properties of water and
steam — the open standard every printed steam table and process simulator is
computed from. This is, in effect, "VLE for water only": a companion to the
multicomponent [`vle-thermo`](https://crates.io/crates/vle-thermo) engine, kept
separate because IF97 is self-contained and **dependency-free** (pure `f64`
math — not even nalgebra), so it drops cleanly into a static-library / FFI /
embedded build.

## Features

- **All five IF97 regions** — compressed liquid (1), superheated vapor (2),
  near-critical Helmholtz (3, with density iteration), the saturation line (4),
  and high-temperature steam (5). Validity: 273.15–2273.15 K, up to 100 MPa.
- **The saturation line both ways, closed-form** — `psat(T)` and `tsat(P)`
  (no iteration), plus the analytic `dPsat/dT`.
- **The state a practitioner reaches for** — `SteamState::tp / tx / px / ph / ps`
  and the printed saturation-table row via `sat_t` / `sat_p`, with two-phase
  quality logic.
- **Backward equations** `T(p,h)` / `T(p,s)` for non-iterative PH/PS flash
  (throttling valves, isentropic turbines).
- **Analytic derivatives throughout** (never finite differences) — enthalpy,
  entropy, `cp`, `cv`, and speed of sound from the exact Gibbs/Helmholtz
  derivatives.
- **Tested to the standard** — the R7-97(2012) computer-program verification
  tables are asserted to their full published precision (9 significant figures).

## Units

Public API takes **T in K** and **P in kPa absolute** (matching `vle-thermo`)
and returns **mass-basis** properties — enthalpy/internal energy in **kJ/kg**,
entropy and heat capacities in **kJ/(kg·K)**, specific volume in **m³/kg**,
speed of sound in **m/s**. A `.molar()` view converts to the engine's molar
canon via `M_water = 18.015268 kg/kmol`.

## Install

```toml
[dependencies]
vle-steam = "0.12"
```

## Example

```rust
use vle_steam::{SteamState, sat_p};

// Superheated steam at 500 °C, 4 MPa (773.15 K, 4000 kPa).
let st = SteamState::tp(773.15, 4000.0).unwrap();
println!("h = {:.1} kJ/kg, s = {:.4} kJ/(kg·K)", st.h, st.s);

// Saturation-table row at 1 bar (100 kPa): latent heat ≈ 2257 kJ/kg.
let row = sat_p(100.0).unwrap();
assert!((row.h_fg - 2257.5).abs() < 1.0);

// Isenthalpic throttle: what fraction of 10-bar condensate flashes at 1 bar?
let condensate = SteamState::px(1000.0, 0.0).unwrap();       // saturated liquid
let flashed = SteamState::ph(100.0, condensate.h).unwrap();  // constant enthalpy
println!("flashed fraction = {:.1}%", 100.0 * flashed.x.unwrap());
```

## Python

The same surface is available from Python as `vle.steam` (built into the
[`vle-thermo`](https://pypi.org/project/vle-thermo/) wheel), with unit-aware
inputs (pint quantities, gauge pressure) and a batch numpy API:

```python
from vle import steam
st = steam.Water(T="500 degC", P="40 bar")      # absolute; "40 barg" for gauge
sat = steam.saturation(P="10 bar")
props = steam.properties(T_array, P_array)       # batch numpy, GIL released
```

## Status

**Complete and verified for the properties it covers.** All five IAPWS-IF97
regions, the region-4 saturation line, and the region-1 backward equations,
checked against the official R7-97(2012) verification tables to 9 significant
figures — the standard's own acceptance criterion, not a self-consistency check.

Not included, by scope rather than by omission: transport properties (viscosity,
thermal conductivity, surface tension) and IAPWS-95 as a high-accuracy oracle.

## References

- IAPWS. *Revised Release on the IAPWS Industrial Formulation 1997 for the
  Thermodynamic Properties of Water and Steam*; IAPWS R7-97(2012).
- Wagner, W.; Kretzschmar, H.-J. *International Steam Tables*, 3rd ed.;
  Springer, 2019.

## License

MIT — part of the [vle](https://github.com/miguelju/vle) project.
