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
- [x] **Push to GitHub** (~0.5h) — create remote, initial push, verify README renders

## Milestone 2: Dev Environment & Scaffolding
*Executed by Claude Code using Claude Opus 4.6 (1M context)*

- [x] **Install Rust toolchain** (~0.5h) — `rustup`, verify `cargo --version`
- [x] **Set up conda environment** (~0.5h) — `conda create -n vle python=3.11`, install maturin via pip inside the conda env
- [x] **Create `engine/Cargo.toml`** (~1h) — deps: nalgebra, pyo3, ndarray, approx (for tests)
- [x] **Create `engine/src/lib.rs`** (~0.5h) — crate root with module declarations
- [x] **Define Rust enums** (~2–3h) — `CubicEos` (22+ variants), `ActivityModel` (5), `MixingRule` (11), `SatPressureModel` (6). Map from VB6 `Enum` and Pascal `case` statements
- [x] **Define core structs** (~2–3h) — `Component`, `Mixture`, `Flow`, `Tolerances`, `ReferenceState`. Union of VB6 and Pascal fields
- [x] **Create `python/pyproject.toml`** (~0.5h) — maturin build backend, package metadata
- [x] **Create `python/src/vle/__init__.py`** (~0.5h) — empty public API skeleton
- [x] **Verify end-to-end build** (~1h) — `conda activate vle` → `cargo build` → `maturin develop` → `python -c "import vle"` works

## Milestone 3: Units of Measurement Library
*Executed by Claude Code using Claude Opus 4.6 (1M context)*

Independent add-on (~12–15h total). Uses dimensional analysis via the 7 SI base dimensions. Rust: `uom` crate (compile-time checks, phantom types). Python: `pint` library (runtime checks).

- [x] **Scaffold units crate** (~1h) — `units/Cargo.toml` with `uom` dependency, `units/src/lib.rs`
- [x] **Define VLE quantity types** (~2h) — Temperature (absolute), TemperatureDiff (gradient / interval), Pressure, MolarEnergy, MolarEntropy, MolarVolume, Amount as aliases for `uom`'s SI types
- [x] **Implement gauge pressure units** (~1–2h) — Built-in barg, psig, kPag with affine conversion; P_atm is a runtime-configurable parameter in the registry (never hardcoded); default 101.325 kPa; rejects non-positive absolute results
- [x] **Implement runtime UnitRegistry** (~3–4h) — extensible runtime registry; supports `define()`, `define_gauge()`, `define_dimension()`, `define_with_dimension()`
- [x] **Implement unit string parser** (~2–3h) — `parse("25 degC")` → canonical Quantity; supports all VLE units
- [x] **Implement canonical conversion** (~1–2h) — `to_canonical()` / `from_canonical()` per unit
- [x] **Implement TOML unit file loader** (~2h) — `registry.load_from_toml()` for bulk user-defined units and dimensions
- [x] **Write Rust conversion tests** (~2h) — 18 integration tests + 3 parser unit tests + 2 compile-fail doctests covering all 7 quantities, gauge offset behavior, and absolute-vs-difference temperature semantics
- [x] **Test custom unit extension** (~1h) — round-trips for `mmH2O`, gauge `mmH2Og` that tracks P_atm, and a new `heat_transfer_coefficient` dimension
- [x] **Create Python units wrapper** (~2h) — `python/src/vle/units.py` around `pint` with the same canonical units and configurable P_atm
- [x] **Write Python conversion tests** (~1–2h) — 40 wrapper tests + 14 parity tests against Rust golden values
- [x] **Document units API + extension guide** (~1–2h) — added `docs/en/units/README.md` quickstart (existing `dimensional-analysis.md` already has the design)

## Milestone 4: Component Property Database
*Executed by Claude Code using Claude Opus 4.7 (1M context)*

SQLite-based property database with CLI, Jupyter notebook, and first deploy to the hub (~12–15h total).

- [x] **Define SQLite schema** (~0.5h) — `python/src/vle/db/sql/schema.sql` (bundled with the wheel) with 4 tables: components, kij_params, activity_params, experimental_vle
- [x] **Implement Python db package** (~3h) — `python/src/vle/db/` with connection.py, queries.py, models.py, seed.py
- [x] **Extract and seed Chapter IV data** (~1h) — 15 compounds from `thermo`/DIPPR, binary params from thesis tables, experimental VLE data
- [x] **Implement CLI tool** (~1.5h) — `python/src/vle/cli/main.py` with init, seed, validate, show, list, export commands; wired as `vle-db` console script via `[project.scripts]`
- [x] **Implement optional thermo seeding** (~1h) — `vle-db seed --source thermo` for ~70K compounds (optional dependency)
- [x] **Write validation tests** (~0.5h) — `vle-db validate chapter4` passes; 16 pytest cases in `python/tests/test_db.py` cover CRUD + kij round-trip (+ pair-order normalization) + seed artifact
- [x] **Create milestone notebook** (~2h) — `notebooks/00_component_database.ipynb` per CLAUDE.md *Notebook Conventions*: Chapter IV §4.1 / §4.3 / §4.7 blockquotes, worked example over all 4 tables, 2 user exercises with collapsed solutions, References section; generated deterministically by `scripts/build_notebook_00.py`
- [x] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example` updated for `vle-db init/seed`, optional `thermo` dep; `Dockerfile.notebook` now bakes `/opt/vle/notebooks/` and a pre-seeded `components.db`, with a first-start hook (`seed-user-home.sh`) that copies them into the user's `~/work/` on login
- [x] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-04.md` with Miguel's host-specific rebuild/restart steps
- [ ] **Deploy notebook to JupyterHub** (~1h) — run `deploy/scripts/deploy.sh` on the VPS to rebuild the notebook image and bounce the stack; then work through the smoke-test checklist in `deploy/local/deploy-notes/milestone-04.md`

## Milestone 5: Numerics

- [ ] **Implement Cardano cubic solver** (~2–3h) — from `McommonFunctions.bas:324`, add (12) robustness for near-degenerate discriminant, (13) volume root selection
- [ ] **Implement Brent's method** (~2h) — default bracketed root finder, from VB6 `clsLVE.cls` (Numerical Recipes reference)
- [ ] **Implement Illinois algorithm** (~1h) — lightweight modified Regula Falsi
- [ ] **Implement Broyden quasi-Newton** (~3–4h) — rank-1 Jacobian update, periodic full refresh every K=5 steps, stall detection fallback
- [ ] **Implement Halley's method** (~1h) — for scalar equations (used in Rachford-Rice)
- [ ] **Implement utility functions** (~1h) — SumFrac, Norm, convergence checks, parabolic interpolation
- [ ] **Write numerical method tests** (~2–3h) — test against known roots, convergence rates, edge cases — validation tests pass
- [ ] **Create milestone notebook** (~2–3h) — `notebooks/m05_numerics.ipynb` per CLAUDE.md *Notebook Conventions*: MODERNIZATION_PLAN §A–§H snippets, convergence plots comparing Brent / Illinois / Broyden / Halley vs. legacy Regula Falsi, ≥2 user exercises (e.g. "solve a custom cubic with Cardano")
- [ ] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`
- [ ] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-05.md`
- [ ] **Deploy notebook to JupyterHub** (~1h) — rebuild, restart, verify via `${DOMAIN}`

## Milestone 6: Pure Component Models

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
- [ ] **Write pure component tests** (~3–4h) — compare against known values, cross-validate EOS variants — validation tests pass
- [ ] **Create milestone notebook** (~3–4h) — `notebooks/02_pure_component.ipynb` per CLAUDE.md *Notebook Conventions*: Chapter II §2.3 snippets (cubic EOS forms), PVT isotherms, EOS variant comparison, saturation curves, ≥2 user exercises
- [ ] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`
- [ ] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-06.md`
- [ ] **Deploy notebook to JupyterHub** (~1h) — rebuild, restart, verify via `${DOMAIN}`

## Milestone 7: Mixture Models

- [ ] **Implement 5 activity models** (~4–6h) — Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand, each with analytical excess enthalpy
- [ ] **Implement liquid volume models** (~1–2h) — Rackett, Thomson/COSTALD (18)
- [ ] **Implement 8+ mixing rules** (~6–8h) — IVDW, IIVDW, WS (21), HOV, HVS, MHV1, MHV2, Clasica_I, plus Schmidt-Wenzel/Patel-Teja C-parameter mixing (4)
- [ ] **Implement multicomponent fugacity** (~4–6h) — partial fugacity coefficients for all mixing rules (9), 3-param EOS (4), Chao-Seader multicomp (4)
- [ ] **Implement enthalpy/entropy** (~3–4h) — ideal Cp integration, departure functions (9), condensation enthalpy (4), reference state handling
- [ ] **Write mixture model tests** (~3–4h) — compare against VB6/Pascal outputs — validation tests pass
- [ ] **Create milestone notebook** (~2–3h) — `notebooks/03_activity_models.ipynb` per CLAUDE.md *Notebook Conventions*: Chapter II §2.4–2.5 snippets, gamma vs. composition plots, excess Gibbs energy, mixing-rule comparison, ≥2 user exercises
- [ ] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`
- [ ] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-07.md`
- [ ] **Deploy notebook to JupyterHub** (~1h) — rebuild, restart, verify via `${DOMAIN}`

## Milestone 8: Flash & Regression

- [ ] **Implement bubble point (T and P)** (~4–6h) — parabolic interpolation (4), Asselineau high-pressure path (14), Anderson-Prausnitz 2nd stage (20)
- [ ] **Implement dew point (T and P)** (~3–4h) — same algorithm structure as bubble point
- [ ] **Implement isothermal flash** (~3–4h) — Halley's Rachford-Rice (19), K-value iteration
- [ ] **Implement adiabatic flash** (~3–4h) — nested T-loop with enthalpy balance, Brent's for temperature
- [ ] **Implement critical point** (~4–6h) — Heidemann (16) with analytical Helmholtz derivatives, ZCriticoMezcla quick estimate (4)
- [ ] **Implement kij regression** (~2–3h) — Brent's method replacing golden section (4)
- [ ] **Implement Aij regression** (~4–6h) — NR with analytical Jacobian for Margules/VanLaar/Wilson (4), experimental gamma calculation, correlation factor analysis
- [ ] **Validate Chapter IV cases** (~3–4h) — all 8 test cases, verify <1–5% error vs. published results — validation tests pass
- [ ] **Create milestone notebooks** (~10–13h total) — per CLAUDE.md *Notebook Conventions*, one notebook per Chapter IV table group with research-paper snippets, reproduction, and ≥2 exercises each:
  - `notebooks/04_bubble_dew_point.ipynb` (~2–3h) — Tables 4.6–4.9
  - `notebooks/05_flash_calculations.ipynb` (~2–3h) — Tables 4.3–4.4, 4.10
  - `notebooks/06_critical_points.ipynb` (~2–3h) — Tables 4.1–4.2
  - `notebooks/07_kij_regression.ipynb` (~2h) — Tables 4.11–4.12
  - `notebooks/08_aij_regression.ipynb` (~2–3h) — Aij fitting (Pascal-origin)
- [ ] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`
- [ ] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-08.md`
- [ ] **Deploy notebooks to JupyterHub** (~1h) — rebuild, restart, verify each new notebook via `${DOMAIN}`

## Milestone 9: Python Bindings & Wrapper

- [ ] **Create PyO3 bindings** (~4–6h) — expose core types as `#[pyclass]`, calculation functions as `#[pyfunction]`, `VleEngine` class
- [ ] **Build Python `System` class** (~3–4h) — high-level API: `system.bubble_point_T()`, `system.flash_isothermal()`, etc.
- [ ] **Create result dataclasses** (~1–2h) — FlashResult, BubbleResult, DewResult with fields matching thesis notation
- [ ] **Build component database** (~2–3h) — `notebooks/data/components.json` with common substances (Tc, Pc, w, Cp coefficients, etc.)
- [ ] **Build plotting helpers** (~2–3h) — Pxy, Txy, phase envelope diagrams via matplotlib
- [ ] **Write Python test suite** (~2–3h) — `test_validation.py` reproducing all Chapter IV results — validation tests pass
- [ ] **Write installation guide** (~1h) — end-user: `pip install`, basic usage example
- [ ] **Create milestone notebook** (~2–3h) — `notebooks/01_introduction.ipynb` per CLAUDE.md *Notebook Conventions*: Chapter I + Appendix B snippets, `vle.System` API tour, first flash calculation end-to-end, ≥2 user exercises
- [ ] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`
- [ ] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-09.md`
- [ ] **Deploy notebook to JupyterHub** (~1h) — rebuild, restart, verify via `${DOMAIN}`

## Milestone 10: Chapter IV Walkthrough & Final Deployment

Notebooks 01–08 ship incrementally through Milestones 4–9. This milestone is the capstone: one new walkthrough notebook covering all Chapter IV results, plus a final clean-state redeployment of every notebook.

- [ ] **Re-run all prior milestone notebooks** (~1–2h) — fresh kernel, Run All, verify no cell errors — validation pass
- [ ] **Create `notebooks/09_chapter4_validation_walkthrough.ipynb`** (~4–6h) — per CLAUDE.md *Notebook Conventions*: narrated end-to-end walkthrough of [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md) §4.1–§4.7, running the library against every Table 4.1–4.12 and reporting % error vs. published values, ≥2 user exercises
- [ ] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md` catalogue marked complete
- [ ] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-10.md` with final redeploy steps + smoke test checklist
- [ ] **Full clean-state redeploy** (~1–2h) — `docker compose down`, rebuild both hub and notebook images from scratch, bring stack back up, verify every notebook in the catalogue opens and Run-All succeeds via `${DOMAIN}`

---

## Summary

| Milestone | Est. Total | Status |
|-----------|-----------|--------|
| 0. Foundation | — | Done |
| 1. Documentation & Translation | ~20–28h | **Done** |
| 2. Dev Environment & Scaffolding | ~9–12h | **Done** |
| 3. Units Library | ~19–26h | **Done** |
| 4. Component Database | ~12–15h | **Done** (pending VPS deploy) |
| 5. Numerics | ~16–20h | Not started |
| 6. Pure Component Models | ~28–37h | Not started |
| 7. Mixture Models | ~25–35h | Not started |
| 8. Flash & Regression | ~38–52h | Not started |
| 9. Python Bindings & Wrapper | ~19–27h | Not started |
| 10. Ch. IV Walkthrough & Final Deploy | ~7–11h | Not started |
| **Total** | **~193–263h** | |

Each active milestone's total now includes: milestone notebook (~2–4h) + public deploy docs (~0.5h) + private deploy notes (~0.5h) + deploy to hub (~1h).
