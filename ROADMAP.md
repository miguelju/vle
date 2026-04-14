# Project Roadmap

High-level milestones for the VLE modernization project. For actionable tasks with time estimates, see [TODO.md](TODO.md). For full technical details, see [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md).

---

## Milestone 0: Project Foundation ✓
**Goal**: Repository, documentation structure, and analysis complete.

- [x] Analyze legacy VB6 codebase (~15,000 lines)
- [x] Analyze legacy Pascal codebase (~2,500 lines)
- [x] Create Pascal vs VB6 comparison document
- [x] Create modernization plan with 16 implementation phases
- [x] Map algorithms to 22 academic references (ACS format)
- [x] Propose 8 algorithm performance improvements (A–H)
- [x] Initialize git repository
- [x] Create README, LICENSE (MIT), .gitignore
- [x] Reorganize repo structure (legacy/, docs/en/, docs/es/)
- [x] Create navigatable English research paper skeleton (13 interlinked files)
- [x] Convert all citations to ACS format

## Milestone 1: Documentation & Translation ✓
**Goal**: Complete English research paper, parameter reference, setup guide.
*Phase 1 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Opus 4.6 (1M context)*

- [x] Complete English translation of Chapter I (Introduction)
- [x] Complete English translation of Chapter II (VLE Theory)
- [x] Complete English translation of Chapter III (Architecture)
- [x] Complete English translation of Chapter IV (Validation)
- [x] Complete English translation of Chapter V (Conclusions)
- [x] Translate Appendix A (Analyst Manual — class/module descriptions)
- [x] Translate Appendix B (User Manual — library usage)
- [x] Create parameter reference document (`docs/en/parameters/parameter_reference.md`)
- [x] Write developer setup guide (`docs/en/SETUP.md`)

## Milestone 2: Dev Environment & Scaffolding ✓
**Goal**: Rust crate compiles, Python package builds, empty module importable.
*Phase 2 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Opus 4.6 (1M context)*

- [x] Install and verify Rust toolchain
- [x] Set up conda environment and install maturin
- [x] Scaffold Rust crate (`engine/Cargo.toml` — nalgebra, PyO3 deps)
- [x] Scaffold Python package (`python/pyproject.toml` — maturin backend)
- [x] Define all Rust enums (CubicEos, ActivityModel, MixingRule, SatPressureModel)
- [x] Define core structs (Component, Mixture, Flow, Tolerances, ReferenceState)
- [x] Verify end-to-end: `cargo build` → `maturin develop` → `python -c "import vle"`
- [x] Push to GitHub

## Milestone 3: Units of Measurement Library ✓
**Goal**: Independent Rust crate + Python wrapper for unit conversion using dimensional analysis.
*Phase 3 of MODERNIZATION_PLAN.md — add-on sub-project, works independently of VLE engine*
*Executed by Claude Code using Claude Opus 4.6 (1M context)*

- [x] Scaffold `units/` Rust crate with `uom` dependency
- [x] Define VLE-specific quantity types (Temperature, TemperatureDiff, Pressure, MolarEnergy, MolarEntropy, MolarVolume, Amount)
- [x] Implement built-in gauge pressure units (barg, psig, kPag) with configurable atmospheric pressure offset
- [x] Implement extensible runtime `UnitRegistry` (allows user-added units alongside the compile-time typed API)
- [x] Implement unit string parser (`parse_unit_string("kPa")` → typed quantity)
- [x] Implement `to_canonical()` / `from_canonical()` conversion functions
- [x] Implement TOML unit file loader (shared by Rust and Python for user-defined units)
- [x] Write Rust conversion test suite (7 quantities × 3+ alt units, round-trip; include absolute-vs-difference temperature parity)
- [x] Test custom unit extension (add `mmH2O` and a new dimension at runtime)
- [x] Create `python/src/vle/units.py` wrapper around `pint`, exposing `ureg` for user extensions
- [x] Write Python conversion tests (parity with Rust + custom user-added units)
- [x] Document units API and extension guide in `docs/en/units/`

## Milestone 4: Component Property Database
**Goal**: SQLite database with Chapter IV validation data, CLI tool, and interactive Jupyter notebook.
*Phase 4 of MODERNIZATION_PLAN.md*

- [x] Define SQLite schema (`data/schema.sql`)
- [ ] Implement Python `vle.db` package (connection, queries, models, seed)
- [x] Extract and seed Chapter IV compound properties (15 compounds from DIPPR)
- [ ] Seed binary params (van Laar methanol/water, kij CO2/n-butane) and experimental VLE data
- [ ] Implement CLI tool (`vle-db init`, `seed`, `validate`, `show`, `list`, `export`)
- [ ] Create interactive Jupyter notebook (`notebooks/00_component_database.ipynb`)
- [ ] Implement optional `thermo` library seeding for ~70K compounds
- [ ] Write Chapter IV validation test (`vle-db validate chapter4`)

## Milestone 5: Numerics
**Goal**: Core numerical utilities tested and benchmarked.
*Phase 5 of MODERNIZATION_PLAN.md*

- [ ] Cardano cubic solver with (12) Poling & Prausnitz robustness
- [ ] Brent's method root finder (default bracketed solver)
- [ ] Illinois algorithm root finder (lightweight alternative)
- [ ] Broyden quasi-Newton solver with periodic Jacobian refresh
- [ ] Halley's method for scalar equations
- [ ] Utility functions: SumFrac, Norm, convergence checks
- [ ] Unit tests for all numerical methods

## Milestone 6: Pure Component Models
**Goal**: All pure component EOS, saturation pressure, and virial working.
*Phases 6–8 of MODERNIZATION_PLAN.md*

- [ ] EOS family constants — k1, k2, k3 parameterization (5)
- [ ] All 22+ alpha(Tr) functions with analytical dα/dTr (§D)
- [ ] 3-parameter EOS: Schmidt-Wenzel, Patel-Teja (4)
- [ ] Chao-Seader liquid fugacity correlation (4)
- [ ] Z-factor, fugacity coefficient, departure H/S
- [ ] Maxwell equal-area test for saturation
- [ ] Saturation pressure: Antoine (4), Riedel, Muller, RPM, polynomial, Maxwell
- [ ] Virial equation — pure + multicomponent (Pitzer/Tsonopoulos)
- [ ] Unit tests for all pure component calculations

## Milestone 7: Mixture Models
**Goal**: Activity models, mixing rules, and multicomponent EOS working.
*Phases 9–12 of MODERNIZATION_PLAN.md*

- [ ] 5 activity coefficient models with analytical dGE/dT (§E)
- [ ] Rackett and Thomson (18) liquid molar volume
- [ ] 8+ mixing rules including Wong-Sandler (21)
- [ ] Multicomponent fugacity coefficients (9)
- [ ] 3-parameter EOS mixture fugacity (4)
- [ ] Enthalpy and entropy (ideal + departure + excess)
- [ ] Unit tests for all mixture calculations

## Milestone 8: Flash & Regression
**Goal**: All flash calculations pass Chapter IV validation.
*Phase 13 of MODERNIZATION_PLAN.md*

- [ ] Bubble point (T and P) with Broyden NR (§A)
- [ ] Dew point (T and P)
- [ ] Isothermal flash with Halley's Rachford-Rice (§F)
- [ ] Adiabatic flash
- [ ] Critical point — Heidemann with analytical Helmholtz derivatives (§G)
- [ ] kij regression via Brent's method (§B)
- [ ] Aij regression with analytical Jacobian (4)
- [ ] Validate all 8 Chapter IV test cases (<1–5% error)

## Milestone 9: Python Bindings & Wrapper
**Goal**: Python package installable, high-level API usable.
*Phases 14–15 of MODERNIZATION_PLAN.md*

- [ ] PyO3 bindings for core types and calculation functions
- [ ] Python `System` class (high-level API)
- [ ] Result dataclasses (FlashResult, BubbleResult, DewResult)
- [ ] Component database (JSON)
- [ ] Plotting helpers (Pxy, Txy diagrams via matplotlib)
- [ ] Python test suite (reproduce Chapter IV validation)
- [ ] Write end-user installation guide

## Milestone 10: Jupyter Notebooks
**Goal**: Interactive notebooks reproducing all thesis results.
*Phase 16 of MODERNIZATION_PLAN.md*

- [ ] 01_introduction — Overview, installation, basic API
- [ ] 02_pure_component — PVT, EOS comparison, saturation curves
- [ ] 03_activity_models — Gamma plots, excess Gibbs energy
- [ ] 04_bubble_dew_point — Reproduce Tables 4.6–4.9
- [ ] 05_flash_calculations — Reproduce Tables 4.3–4.4, 4.10
- [ ] 06_critical_points — Reproduce Tables 4.1–4.2
- [ ] 07_kij_regression — Reproduce Tables 4.11–4.12
- [ ] 08_aij_regression — Activity model Aij fitting

---

**Status key**: `[✓]` complete · `[ ]` not started · `[~]` in progress
