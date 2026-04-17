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
*Executed by Claude Code using Claude Opus 4.7 (1M context)*

- [x] Define SQLite schema (`python/src/vle/db/sql/schema.sql`, bundled with the wheel)
- [x] Implement Python `vle.db` package (connection, queries, models, seed)
- [x] Extract and seed Chapter IV compound properties (15 compounds from DIPPR)
- [x] Seed binary params (van Laar methanol/water, kij CO2/n-butane) and experimental VLE data
- [x] Implement CLI tool (`vle-db init`, `seed`, `validate`, `show`, `list`, `export`) — wired as `vle-db` console script
- [x] Implement optional `thermo` library seeding for ~70K compounds
- [x] Write Chapter IV validation test (`vle-db validate chapter4`) — validation passes; 16 pytest cases in `python/tests/test_db.py` cover CRUD + kij round-trip + seed
- [x] Create milestone notebook (`notebooks/00_component_database.ipynb`) — 24 cells, executed top-to-bottom; Chapter IV §4.1 / §4.3 / §4.7 snippets, worked example over all four tables, 2 user exercises with collapsed solutions
- [x] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`) — added `vle-db init / seed` step, optional `thermo` dep, first-start user-home seed hook
- [x] Update private deploy notes (`deploy/local/deploy-notes/milestone-04.md`)
- [ ] Deploy notebook to JupyterHub — rebuild notebook image, restart hub, verify via `${DOMAIN}` *(run `deploy/scripts/deploy.sh` on the VPS)*

## Milestone 5: Numerics
**Goal**: Core numerical utilities tested and benchmarked.
*Phase 5 of MODERNIZATION_PLAN.md*

- [ ] Cardano cubic solver with (12) Poling & Prausnitz robustness
- [ ] Brent's method root finder (default bracketed solver)
- [ ] Illinois algorithm root finder (lightweight alternative)
- [ ] Broyden quasi-Newton solver with periodic Jacobian refresh
- [ ] Halley's method for scalar equations
- [ ] Utility functions: SumFrac, Norm, convergence checks
- [ ] Unit tests for all numerical methods — validation test passes
- [ ] Create milestone notebook (`notebooks/m05_numerics.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: research-paper snippets (§A–§H algorithm choices), worked convergence comparisons, ≥2 user exercises
- [ ] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`) — generic install deltas only
- [ ] Update private deploy notes (`deploy/local/deploy-notes/milestone-05.md`)
- [ ] Deploy notebook to JupyterHub — rebuild notebook image, restart hub, verify via `${DOMAIN}`

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
- [ ] Unit tests for all pure component calculations — validation test passes
- [ ] Create milestone notebook (`notebooks/02_pure_component.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: Chapter II §2.3 (cubic EOS) snippets, PVT diagrams comparing EOS variants, saturation curves, ≥2 user exercises
- [ ] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`) — generic install deltas only
- [ ] Update private deploy notes (`deploy/local/deploy-notes/milestone-06.md`)
- [ ] Deploy notebook to JupyterHub — rebuild notebook image, restart hub, verify via `${DOMAIN}`

## Milestone 7: Mixture Models
**Goal**: Activity models, mixing rules, and multicomponent EOS working.
*Phases 9–12 of MODERNIZATION_PLAN.md*

- [ ] 5 activity coefficient models with analytical dGE/dT (§E)
- [ ] Rackett and Thomson (18) liquid molar volume
- [ ] 8+ mixing rules including Wong-Sandler (21)
- [ ] Multicomponent fugacity coefficients (9)
- [ ] 3-parameter EOS mixture fugacity (4)
- [ ] Enthalpy and entropy (ideal + departure + excess)
- [ ] Unit tests for all mixture calculations — validation test passes
- [ ] Create milestone notebook (`notebooks/03_activity_models.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: Chapter II §2.4–2.5 (activity / mixing rules) snippets, gamma-vs-composition plots, excess Gibbs energy, ≥2 user exercises
- [ ] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`) — generic install deltas only
- [ ] Update private deploy notes (`deploy/local/deploy-notes/milestone-07.md`)
- [ ] Deploy notebook to JupyterHub — rebuild notebook image, restart hub, verify via `${DOMAIN}`

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
- [ ] Validate all 8 Chapter IV test cases (<1–5% error) — validation tests pass
- [ ] Create milestone notebooks — professional structure per CLAUDE.md *Notebook Conventions*; one notebook per thesis table group, each reproducing the referenced Chapter IV table(s) with snippets from the research paper and ≥2 user exercises:
  - `notebooks/04_bubble_dew_point.ipynb` — Tables 4.6–4.9
  - `notebooks/05_flash_calculations.ipynb` — Tables 4.3–4.4, 4.10
  - `notebooks/06_critical_points.ipynb` — Tables 4.1–4.2
  - `notebooks/07_kij_regression.ipynb` — Tables 4.11–4.12
  - `notebooks/08_aij_regression.ipynb` — Aij fitting (Pascal-origin)
- [ ] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`) — generic install deltas only
- [ ] Update private deploy notes (`deploy/local/deploy-notes/milestone-08.md`)
- [ ] Deploy notebooks to JupyterHub — rebuild notebook image, restart hub, verify each new notebook opens via `${DOMAIN}`

## Milestone 9: Python Bindings & Wrapper
**Goal**: Python package installable, high-level API usable.
*Phases 14–15 of MODERNIZATION_PLAN.md*

- [ ] PyO3 bindings for core types and calculation functions
- [ ] Python `System` class (high-level API)
- [ ] Result dataclasses (FlashResult, BubbleResult, DewResult)
- [ ] Component database (JSON)
- [ ] Plotting helpers (Pxy, Txy diagrams via matplotlib)
- [ ] Python test suite (reproduce Chapter IV validation) — validation tests pass
- [ ] Write end-user installation guide
- [ ] Create milestone notebook (`notebooks/01_introduction.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: Chapter I + Appendix B (User Manual) snippets, installation walk-through, `vle.System` basic API tour, ≥2 user exercises
- [ ] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`) — generic install deltas only
- [ ] Update private deploy notes (`deploy/local/deploy-notes/milestone-09.md`)
- [ ] Deploy notebook to JupyterHub — rebuild notebook image, restart hub, verify via `${DOMAIN}`

## Milestone 10: Chapter IV Walkthrough & Final Deployment
**Goal**: One cohesive walkthrough of [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md) and a final full-stack redeploy of every milestone notebook.
*Phase 16 of MODERNIZATION_PLAN.md*

> Notebooks 01–08 ship incrementally through Milestones 4–9 (each milestone produces the notebook for the feature it built). This milestone is the capstone: it adds the Chapter IV walkthrough and verifies every notebook is still reachable after a fresh deploy.

- [ ] Re-run every existing milestone notebook top-to-bottom in a fresh kernel — validation pass
- [ ] Create `notebooks/09_chapter4_validation_walkthrough.ipynb` — professional structure per CLAUDE.md *Notebook Conventions*; walks a reader through all seven Chapter IV cases (Tables 4.1–4.12), pulls quoted snippets from [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md), runs the `vle` library against each table, reports % error vs. published values, and ends with ≥2 user exercises (e.g. "repeat the kij regression for a different binary").
- [ ] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`) — mark the full notebook catalogue as published
- [ ] Update private deploy notes (`deploy/local/deploy-notes/milestone-10.md`) — final redeploy steps, smoke test checklist
- [ ] Full redeploy — rebuild both hub and notebook images from a clean state, restart the stack, verify every notebook in `deploy/NOTEBOOKS.md`'s catalogue opens and Runs-All cleanly through `${DOMAIN}`

---

**Status key**: `[✓]` complete · `[ ]` not started · `[~]` in progress
