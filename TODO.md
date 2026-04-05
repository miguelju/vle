# TODO

Actionable tasks with rough time estimates. Grouped by [ROADMAP.md](ROADMAP.md) milestone.
Check off items as they're completed. Time estimates assume working with Claude Code.

---

## Milestone 1: Documentation & Translation

- [ ] **Translate Chapter I — Introduction** (~1h) — shortest chapter, mostly context
- [ ] **Translate Chapter II — VLE Theory** (~4–6h) — longest chapter, heavy equations (2.1–2.49), tables 2.1–2.3, figures 2.1–2.7. Equations need LaTeX or careful markdown formatting
- [ ] **Translate Chapter III — Architecture** (~2–3h) — class descriptions, figures 3.1–3.15
- [ ] **Translate Chapter IV — Validation** (~2–3h) — tables 4.1–4.12 with numerical data
- [ ] **Translate Chapter V — Conclusions** (~0.5h) — short, already partially done
- [ ] **Translate Appendix A — Analyst Manual** (~4–6h) — detailed class/module descriptions from `docs/es/research-paper/programdocs/Analista.md`
- [ ] **Translate Appendix B — User Manual** (~2–3h) — library usage guide from `docs/es/research-paper/programdocs/dllManual.md`
- [ ] **Create parameter reference** (~3–4h) — compile all parameters from Table in MODERNIZATION_PLAN.md into `docs/en/parameters/parameter_reference.md`, cross-reference with source code
- [ ] **Write developer setup guide** (~1–2h) — `docs/en/SETUP.md`: Rust toolchain, Python venv, maturin, how to build/test

## Milestone 2: Dev Environment & Scaffolding

- [ ] **Install Rust toolchain** (~0.5h) — `rustup`, verify `cargo --version`
- [ ] **Set up Python venv** (~0.5h) — `python -m venv .venv`, install maturin
- [ ] **Create `engine/Cargo.toml`** (~1h) — deps: nalgebra, pyo3, ndarray, approx (for tests)
- [ ] **Create `engine/src/lib.rs`** (~0.5h) — crate root with module declarations
- [ ] **Define Rust enums** (~2–3h) — `CubicEos` (22+ variants), `ActivityModel` (5), `MixingRule` (8+), `SatPressureModel` (6). Map from VB6 `Enum` and Pascal `case` statements
- [ ] **Define core structs** (~2–3h) — `Component`, `Mixture`, `Flow`, `Tolerances`, `ReferenceState`. Union of VB6 and Pascal fields
- [ ] **Create `python/pyproject.toml`** (~0.5h) — maturin build backend, package metadata
- [ ] **Create `python/src/vle/__init__.py`** (~0.5h) — empty public API skeleton
- [ ] **Verify end-to-end build** (~1h) — `cargo build` → `maturin develop` → `python -c "import vle"` works
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
| 1. Documentation & Translation | ~20–28h | Not started |
| 2. Dev Environment & Scaffolding | ~9–12h | Not started |
| 3. Numerics | ~12–15h | Not started |
| 4. Pure Component Models | ~24–32h | Not started |
| 5. Mixture Models | ~21–30h | Not started |
| 6. Flash & Regression | ~26–37h | Not started |
| 7. Python Bindings & Wrapper | ~15–22h | Not started |
| 8. Jupyter Notebooks | ~19–26h | Not started |
| **Total** | **~146–202h** | |
