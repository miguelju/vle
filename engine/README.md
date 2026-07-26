# vle-thermo

Vapor-liquid equilibrium (VLE) thermodynamic calculator in Rust.

[![Crates.io](https://img.shields.io/crates/v/vle-thermo.svg)](https://crates.io/crates/vle-thermo)
[![Docs.rs](https://docs.rs/vle-thermo/badge.svg)](https://docs.rs/vle-thermo)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A modern Rust port of two legacy thermodynamic codebases (VB6 ~15,000 lines + Pascal ~2,500 lines) supporting:

- **22+ cubic equations of state** — Peng-Robinson, RKS, van der Waals, Schmidt-Wenzel, Patel-Teja, and more
- **6 activity coefficient models** — NRTL, Wilson, van Laar, Margules, Scatchard-Hildebrand, Ideal (NRTL with analytic ∂lnγ/∂T and excess enthalpy via dual-number AD)
- **11 mixing rules** — Classical (IVDW, IIVDW), Wong-Sandler, Huron-Vidal, MHV1/MHV2
- **6 saturation pressure correlations** — Antoine, Riedel, Müller, RPM, polynomial, Maxwell
- **6 flash calculation types** — bubble/dew point (T/P), isothermal, adiabatic (PH)
- **Parameter regression** — kij (binary interaction) and Aij (activity model)

## Status

**The model surface is complete and validated.** 22 cubic equations of state,
6 activity models, 11 mixing rules, the Cardano solver, Rachford-Rice via
Halley inside the Leibovici–Neoschil window, the full flash suite (isothermal,
bubble/dew *T* and *P*, adiabatic *PH*, phase-envelope continuation through the
critical point), tangent-plane stability analysis, the mixture critical point by
Heidemann's method, and kij/Aij regression — all with **exact derivatives**
(hand-derived analytic for the classical paths, `num-dual` dual-number AD for
the GE-based mixing rules; finite differences survive only as test oracles).
291 Rust tests back it, and the results are checked against the published
Chapter IV tables of the thesis this engine derives from, not merely against
themselves.

**What `0.x` means here.** It is a statement about the **API**, not the
numerics. This release itself adds a `FlashError` variant, which is a breaking
change for a downstream exhaustive `match` — that is the kind of latitude the
`0.` still buys. The numerical core is settled; the surface is not yet frozen.

Known gaps, stated rather than implied: `k_values_with_derivs` returns
`Unsupported` for a virial vapor and for a Chao-Seader liquid, and the
isothermal flash's terminal Newton polish on ln K (the analytic-Jacobian finish
described in the modernization plan §J) is **not** implemented — it converges by
GDEM-accelerated successive substitution, which reaches the Chapter IV cases in
7–14 iterations.

`0.12.0` is a **performance release**: the flash layer gained **allocation-free `*_into`
kernels** — `mixture::ln_phi_mix_into`, `mixture::ln_phi_mix_min_gibbs_into`
(both cubic roots from one shared mixture state), and
`flash::ln_k_values_into` (equilibrium ratios in **log** form, which is what
the models produce natively). The value-returning `ln_phi_mix` / `k_values`
are unchanged and are now defined *in terms of* those, so there is exactly one
implementation of the thermodynamics. Also new: `saturation::ln_poynting_factor`,
`flash::init::wilson_ln_k`, and a `FlashError::InvalidInput` variant for inputs
that are correctly shaped but numerically unusable (non-finite or negative
`zᵢ`, non-positive `Kᵢ`, non-positive tolerance). Measured effect on the
isothermal flash: −24…−28 %; on tangent-plane stability: −44…−51 % (cumulative
across both parts of the audit response). Details in
[`OPTIMIZATION_PLAN_PART1.md`](https://github.com/miguelju/vle/blob/main/docs/plans/engine/OPTIMIZATION_PLAN_PART1.md).

The mixture core followed: **`mixture::TpCache`** holds the composition-independent
half of an evaluation (every component's α, Aᵢ, Bᵢ and their roots) so a caller
sweeping composition at fixed `(T, P)` — any flash — builds it once and calls
`ln_phi_mix_cached_into` / `ln_phi_mix_min_gibbs_cached_into` per composition.
**`activity::ActivityTpCache`** does the same for Wilson's Λᵢⱼ and NRTL's
τᵢⱼ/Gᵢⱼ (NRTL's `ln_gamma_all` drops from O(N³) `exp` calls to O(N²)), and the
virial path gains flat row-major `b_mix_matrix_flat` +
`ln_phi_mix_virial_flat_into`. Cumulative measured effect: isothermal flash
−24…−28 %, tangent-plane stability −44…−51 %. Details in
[`OPTIMIZATION_PLAN_PART2.md`](https://github.com/miguelju/vle/blob/main/docs/plans/engine/OPTIMIZATION_PLAN_PART2.md).

`0.10.0` adds **IAPWS-IF97 steam
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
(`mixture::mixture_params` and friends now take `t: D, p: D`). As set out under
**Status**, the `0.` is about API stability: semver guarantees begin at 1.0, so
pin a minor version if a breaking `match` or signature change would cost you.

See the [roadmap](https://github.com/miguelju/vle/blob/main/ROADMAP.md) for
what's shipped vs. planned, and the
[modernization plan](https://github.com/miguelju/vle/blob/main/docs/plans/MODERNIZATION_PLAN.md)
for the phase-by-phase technical detail.

## Install

```toml
[dependencies]
vle-thermo = "0.12"
```

Or with `cargo add`:

```sh
cargo add vle-thermo
```

The crate is `no-pyo3` by default — PyO3 bindings are gated behind the optional
`python` feature and are only needed when maturin builds the Python wheel.

Opt into the bundled 25-compound property database with the `component-db`
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
