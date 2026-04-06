# TODO

Actionable tasks with rough time estimates. Grouped by [ROADMAP.md](ROADMAP.md) milestone.
Check off items as they're completed. Time estimates assume working with Claude Code.

---

## Milestone 1: Documentation & Translation
*Executed by Claude Code using Claude Opus 4.6 (1M context)*

- [x] **Translate Chapter I — Introduction** (~1h) — shortest chapter, mostly context
- [x] **Translate Chapter II — VLE Theory** (~4–6h) — longest chapter, heavy equations (2.1–2.49), tables 2.1–2.3, figures 2.1–2.7
- [x] **Translate Chapter III — Architecture** (~2–3h) — class descriptions, figures 3.1–3.15
- [x] **Translate Chapter IV — Validation** (~2–3h) — tables 4.1–4.12 with numerical data
- [x] **Translate Chapter V — Conclusions** (~0.5h) — short
- [x] **Translate Appendix A — Analyst Manual** (~4–6h) — detailed class/module descriptions (1118 lines)
- [x] **Translate Appendix B — User Manual** (~2–3h) — library usage guide (204 lines)
- [x] **Create parameter reference** (~3–4h) — `docs/en/parameters/parameter_reference.md` (167 lines)
- [x] **Write developer setup guide** (~1–2h) — `docs/en/SETUP.md`: Rust toolchain, conda env, maturin, how to build/test

## Milestone 1.5: Units of Measurement Library

Independent add-on (~12–15h total). Uses dimensional analysis via the 7 SI base dimensions. Rust: `uom` crate (compile-time checks, phantom types). Python: `pint` library (runtime checks).

- [ ] **Scaffold units crate** (~1h) — `units/Cargo.toml` with `uom` dependency, `units/src/lib.rs`
- [ ] **Define VLE quantity types** (~2h) — Temperature (absolute), TemperatureDiff (gradient / interval), Pressure, MolarEnergy, MolarEntropy, MolarVolume, Amount as aliases for `uom`'s SI types. Temperature and TemperatureDiff are *separate* types even though both canonicalize to K (see `docs/en/units/dimensional-analysis.md` §3.3)
- [ ] **Implement gauge pressure units** (~1–2h) — Built-in barg, psig, kPag with affine conversion (P_abs = P_gauge × scale + P_atm). P_atm is a **runtime-configurable parameter** in the registry (never hardcoded): `registry.set_atmospheric_pressure()` / `get_atmospheric_pressure()`. Default 101.325 kPa. Reject non-positive absolute results. Support `define_gauge()` for user-added gauge units. See `docs/en/units/dimensional-analysis.md` §3.4
- [ ] **Implement runtime UnitRegistry** (~3–4h) — extensible runtime registry alongside compile-time typed API; supports `define(name, dimension, scale, offset)` for user-added units
- [ ] **Implement unit string parser** (~2–3h) — `parse_unit_string("kPa")` → typed quantity; supports K/°C/°F/°R, Pa/kPa/bar/atm/psi/mmHg/torr, kJ/kmol, J/mol, cal/mol, etc.
- [ ] **Implement canonical conversion** (~1–2h) — `to_canonical()` / `from_canonical()` for each quantity (canonical: K, kPa, kJ/kmol, kJ/(kmol·K), cm³/mol, kmol)
- [ ] **Implement TOML unit file loader** (~2h) — `registry.load_from_toml("custom_units.toml")` for bulk user-defined units, shared by Rust and Python
- [ ] **Write Rust conversion tests** (~2h) — all 7 quantities × 3–4 alt units, round-trip identity, compile-time dimension check (`temperature + pressure` must fail to compile), **absolute-vs-difference temperature parity** (`T_abs + T_abs` must fail; `T_abs - T_abs → TemperatureDiff`; `Δ°C → ΔK` must be scale-only with no offset)
- [ ] **Test custom unit extension** (~1h) — round-trip test adding `mmH2O` to Pressure dimension and a new `heat_transfer_coefficient` dimension
- [ ] **Create Python units wrapper** (~2h) — `python/src/vle/units.py` using `pint`, same unit strings as Rust side, exposes `ureg` for user extensions
- [ ] **Write Python conversion tests** (~1–2h) — verify parity with Rust side via golden values, test user-added units via `ureg.define()`
- [ ] **Document units API + extension guide** (~1–2h) — update `docs/en/units/dimensional-analysis.md` with working code examples; add `docs/en/units/README.md` quickstart

## Milestone 2.5: Component Property Database

SQLite-based property database with CLI and Jupyter notebook interface (~8–10h total).

- [ ] **Define SQLite schema** (~0.5h) — `data/schema.sql` with 4 tables: components, kij_params, activity_params, experimental_vle
- [ ] **Implement Python db package** (~3h) — `python/src/vle/db/` with connection.py, queries.py, models.py, seed.py
- [ ] **Extract and seed Chapter IV data** (~1h) — 15 compounds from `thermo`/DIPPR, binary params from thesis tables, experimental VLE data
- [ ] **Implement CLI tool** (~1.5h) — `python/src/vle/cli/main.py` with init, seed, validate, show, list, export commands
- [ ] **Create Jupyter notebook** (~1.5h) — `notebooks/00_component_database.ipynb` with interactive browsing, search, add/edit
- [ ] **Implement optional thermo seeding** (~1h) — `vle-db seed --source thermo` for ~70K compounds (optional dependency)
- [ ] **Write validation tests** (~0.5h) — `vle-db validate chapter4` verifies all 15 compounds + binary params + experimental data

## Milestone 2: Dev Environment & Scaffolding

- [ ] **Install Rust toolchain** (~0.5h) — `rustup`, verify `cargo --version`
- [ ] **Set up conda environment** (~0.5h) — `conda create -n vle python=3.11`, install maturin via pip inside the conda env
- [ ] **Create `engine/Cargo.toml`** (~1h) — deps: nalgebra, pyo3, ndarray, approx (for tests)
- [ ] **Create `engine/src/lib.rs`** (~0.5h) — crate root with module declarations
- [ ] **Define Rust enums** (~2–3h) — `CubicEos` (22+ variants), `ActivityModel` (5), `MixingRule` (8+), `SatPressureModel` (6). Map from VB6 `Enum` and Pascal `case` statements
- [ ] **Define core structs** (~2–3h) — `Component`, `Mixture`, `Flow`, `Tolerances`, `ReferenceState`. Union of VB6 and Pascal fields
- [ ] **Create `python/pyproject.toml`** (~0.5h) — maturin build backend, package metadata
- [ ] **Create `python/src/vle/__init__.py`** (~0.5h) — empty public API skeleton
- [ ] **Verify end-to-end build** (~1h) — `conda activate vle` → `cargo build` → `maturin develop` → `python -c "import vle"` works
- [ ] **Push to GitHub** (~0.5h) — create remote, initial push, verify README renders

## Milestone 3: Numerics

- [ ] **Implement Cardano cubic solver** (~2–3h) — from `McommonFunctions.bas:324`, add (12) robustness for near-degenerate discriminant, (13) volume root selection
- [ ] **Implement Brent's method** (~2h) — default bracketed root finder, from VB6 `clsLVE.cls` (Numerical Recipes reference)
- [ ] **Implement Illinois algorithm** (~1h) — lightweight modified Regula Falsi
- [ ] **Implement Broyden quasi-Newton** (~3–4h) — rank-1 Jacobian update, periodic full refresh every K=5 steps, stall detection fallback
- [ ] **Implement Halley's method** (~1h) — for scalar equations (used in Rachford-Rice)
- [ ] **Implement utility functions** (~1h) — SumFrac, Norm, convergence checks, parabolic interpolation
- [ ] **Write numerical method tests** (~2–3h) — test against known roots, convergence rates, edge cases

## Milestone 4: Pure Component Models

- [ ] **Implement EOS family constants** (~1–2h) — k1, k2, k3, OmA, OmB lookup table per (5)
- [ ] **Implement 22+ alpha functions** (~4–6h) — all VB6 + Pascal variants, each with `alpha(tr)` and `d_alpha_d_tr(tr)`
- [ ] **Implement Schmidt-Wenzel EOS** (~2–3h) — 3-parameter, Beta parameter, from Pascal `TERMOII.PAS` (4)
- [ ] **Implement Patel-Teja EOS** (~2–3h) — 2 mixing variants (PatelT, PatelTUSB), from Pascal (4)
- [ ] **Implement Chao-Seader correlation** (~2–3h) — liquid fugacity with H2/methane special cases, from Pascal (4)
- [ ] **Implement Z-factor calculation** (~1–2h) — cubic solver integration, 2-param and 3-param
- [ ] **Implement fugacity coefficient** (~2h) — pure component, departure H/S
- [ ] **Implement Maxwell equal-area test** (~2h) — saturation pressure from EOS
- [ ] **Implement saturation pressure models** (~3–4h) — Antoine (4), Riedel, Muller, RPM, polynomial, Maxwell, temperature derivatives
- [ ] **Implement virial equation** (~2–3h) — Pitzer B0/B1, pure + multicomp Z/fugacity/HR/SR
- [ ] **Write pure component tests** (~3–4h) — compare against known values, cross-validate EOS variants

## Milestone 5: Mixture Models

- [ ] **Implement 5 activity models** (~4–6h) — Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand, each with analytical excess enthalpy
- [ ] **Implement liquid volume models** (~1–2h) — Rackett, Thomson/COSTALD (18)
- [ ] **Implement 8+ mixing rules** (~6–8h) — IVDW, IIVDW, WS (21), HOV, HVS, MHV1, MHV2, Clasica_I, plus Schmidt-Wenzel/Patel-Teja C-parameter mixing (4)
- [ ] **Implement multicomponent fugacity** (~4–6h) — partial fugacity coefficients for all mixing rules (9), 3-param EOS (4), Chao-Seader multicomp (4)
- [ ] **Implement enthalpy/entropy** (~3–4h) — ideal Cp integration, departure functions (9), condensation enthalpy (4), reference state handling
- [ ] **Write mixture model tests** (~3–4h) — compare against VB6/Pascal outputs

## Milestone 6: Flash & Regression

- [ ] **Implement bubble point (T and P)** (~4–6h) — parabolic interpolation (4), Asselineau high-pressure path (14), Anderson-Prausnitz 2nd stage (20)
- [ ] **Implement dew point (T and P)** (~3–4h) — same algorithm structure as bubble point
- [ ] **Implement isothermal flash** (~3–4h) — Halley's Rachford-Rice (19), K-value iteration
- [ ] **Implement adiabatic flash** (~3–4h) — nested T-loop with enthalpy balance, Brent's for temperature
- [ ] **Implement critical point** (~4–6h) — Heidemann (16) with analytical Helmholtz derivatives, ZCriticoMezcla quick estimate (4)
- [ ] **Implement kij regression** (~2–3h) — Brent's method replacing golden section (4)
- [ ] **Implement Aij regression** (~4–6h) — NR with analytical Jacobian for Margules/VanLaar/Wilson (4), experimental gamma calculation, correlation factor analysis
- [ ] **Validate Chapter IV cases** (~3–4h) — all 8 test cases, verify <1–5% error vs. published results

## Milestone 7: Python Bindings & Wrapper

- [ ] **Create PyO3 bindings** (~4–6h) — expose core types as `#[pyclass]`, calculation functions as `#[pyfunction]`, `VleEngine` class
- [ ] **Build Python `System` class** (~3–4h) — high-level API: `system.bubble_point_T()`, `system.flash_isothermal()`, etc.
- [ ] **Create result dataclasses** (~1–2h) — FlashResult, BubbleResult, DewResult with fields matching thesis notation
- [ ] **Build component database** (~2–3h) — `notebooks/data/components.json` with common substances (Tc, Pc, w, Cp coefficients, etc.)
- [ ] **Build plotting helpers** (~2–3h) — Pxy, Txy, phase envelope diagrams via matplotlib
- [ ] **Write Python test suite** (~2–3h) — `test_validation.py` reproducing all Chapter IV results
- [ ] **Write installation guide** (~1h) — end-user: `pip install`, basic usage example

## Milestone 8: Jupyter Notebooks

- [ ] **01_introduction** (~2–3h) — overview, installation, basic API walkthrough
- [ ] **02_pure_component** (~3–4h) — PVT diagrams, compare 22+ EOS variants, saturation curves
- [ ] **03_activity_models** (~2–3h) — gamma vs composition, excess Gibbs plots
- [ ] **04_bubble_dew_point** (~2–3h) — reproduce Tables 4.6–4.9 (methanol/water, 2-propanol/water)
- [ ] **05_flash_calculations** (~2–3h) — reproduce Tables 4.3–4.4, 4.10 (adiabatic, isothermal)
- [ ] **06_critical_points** (~2–3h) — reproduce Tables 4.1–4.2 (4 mixture critical points)
- [ ] **07_kij_regression** (~2–3h) — reproduce Tables 4.11–4.12 (CO2/butane)
- [ ] **08_aij_regression** (~3–4h) — Aij fitting demo, compare analytical vs numerical Jacobian

---

## Summary

| Milestone | Est. Total | Status |
|-----------|-----------|--------|
| 0. Foundation | — | Done |
| 1. Documentation & Translation | ~20–28h | **Done** |
| 1.5. Units Library | ~19–26h | Not started |
| 2.5. Component Database | ~8–10h | Not started |
| 2. Dev Environment & Scaffolding | ~9–12h | Not started |
| 3. Numerics | ~12–15h | Not started |
| 4. Pure Component Models | ~24–32h | Not started |
| 5. Mixture Models | ~21–30h | Not started |
| 6. Flash & Regression | ~26–37h | Not started |
| 7. Python Bindings & Wrapper | ~15–22h | Not started |
| 8. Jupyter Notebooks | ~19–26h | Not started |
| **Total** | **~172–236h** | |
