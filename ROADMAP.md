# Project Roadmap

High-level milestones for the VLE modernization project. For actionable tasks with time estimates, see [TODO.md](TODO.md). For full technical details, see [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md).

---

## Milestone 0: Project Foundation ✓
**Goal**: Repository, documentation structure, and analysis complete.

- [x] Analyze legacy VB6 codebase (~15,000 lines)
- [x] Analyze legacy Pascal codebase (~2,500 lines)
- [x] Create Pascal vs VB6 comparison document
- [x] Create modernization plan with 14 implementation phases
- [x] Map algorithms to 22 academic references (ACS format)
- [x] Propose 8 algorithm performance improvements (A–H)
- [x] Initialize git repository
- [x] Create README, LICENSE (MIT), .gitignore
- [x] Reorganize repo structure (legacy/, docs/en/, docs/es/)
- [x] Create navigatable English research paper skeleton (13 interlinked files)
- [x] Convert all citations to ACS format

## Milestone 1: Documentation & Translation ✓
**Goal**: Complete English research paper, parameter reference, setup guide.
*Phase 14 of MODERNIZATION_PLAN.md*

- [x] Complete English translation of Chapter I (Introduction)
- [x] Complete English translation of Chapter II (VLE Theory)
- [x] Complete English translation of Chapter III (Architecture)
- [x] Complete English translation of Chapter IV (Validation)
- [x] Complete English translation of Chapter V (Conclusions)
- [x] Translate Appendix A (Analyst Manual — class/module descriptions)
- [x] Translate Appendix B (User Manual — library usage)
- [x] Create parameter reference document (`docs/en/parameters/parameter_reference.md`)
- [x] Write developer setup guide (`docs/en/SETUP.md`)

## Milestone 2: Dev Environment & Scaffolding
**Goal**: Rust crate compiles, Python package builds, empty module importable.
*Phases 1 of MODERNIZATION_PLAN.md*

- [ ] Install and verify Rust toolchain
- [ ] Set up conda environment and install maturin
- [ ] Scaffold Rust crate (`engine/Cargo.toml` — nalgebra, PyO3 deps)
- [ ] Scaffold Python package (`python/pyproject.toml` — maturin backend)
- [ ] Define all Rust enums (CubicEos, ActivityModel, MixingRule, SatPressureModel)
- [ ] Define core structs (Component, Mixture, Flow, Tolerances, ReferenceState)
- [ ] Verify end-to-end: `cargo build` → `maturin develop` → `python -c "import vle"`
- [ ] Push to GitHub

## Milestone 3: Numerics
**Goal**: Core numerical utilities tested and benchmarked.
*Phase 2 of MODERNIZATION_PLAN.md*

- [ ] Cardano cubic solver with (12) Poling & Prausnitz robustness
- [ ] Brent's method root finder (default bracketed solver)
- [ ] Illinois algorithm root finder (lightweight alternative)
- [ ] Broyden quasi-Newton solver with periodic Jacobian refresh
- [ ] Halley's method for scalar equations
- [ ] Utility functions: SumFrac, Norm, convergence checks
- [ ] Unit tests for all numerical methods

## Milestone 4: Pure Component Models
**Goal**: All pure component EOS, saturation pressure, and virial working.
*Phases 3–5 of MODERNIZATION_PLAN.md*

- [ ] EOS family constants — k1, k2, k3 parameterization (5)
- [ ] All 22+ alpha(Tr) functions with analytical dα/dTr (§D)
- [ ] 3-parameter EOS: Schmidt-Wenzel, Patel-Teja (4)
- [ ] Chao-Seader liquid fugacity correlation (4)
- [ ] Z-factor, fugacity coefficient, departure H/S
- [ ] Maxwell equal-area test for saturation
- [ ] Saturation pressure: Antoine (4), Riedel, Muller, RPM, polynomial, Maxwell
- [ ] Virial equation — pure + multicomponent (Pitzer/Tsonopoulos)
- [ ] Unit tests for all pure component calculations

## Milestone 5: Mixture Models
**Goal**: Activity models, mixing rules, and multicomponent EOS working.
*Phases 6–9 of MODERNIZATION_PLAN.md*

- [ ] 5 activity coefficient models with analytical dGE/dT (§E)
- [ ] Rackett and Thomson (18) liquid molar volume
- [ ] 8+ mixing rules including Wong-Sandler (21)
- [ ] Multicomponent fugacity coefficients (9)
- [ ] 3-parameter EOS mixture fugacity (4)
- [ ] Enthalpy and entropy (ideal + departure + excess)
- [ ] Unit tests for all mixture calculations

## Milestone 6: Flash & Regression
**Goal**: All flash calculations pass Chapter IV validation.
*Phase 10 of MODERNIZATION_PLAN.md*

- [ ] Bubble point (T and P) with Broyden NR (§A)
- [ ] Dew point (T and P)
- [ ] Isothermal flash with Halley's Rachford-Rice (§F)
- [ ] Adiabatic flash
- [ ] Critical point — Heidemann with analytical Helmholtz derivatives (§G)
- [ ] kij regression via Brent's method (§B)
- [ ] Aij regression with analytical Jacobian (4)
- [ ] Validate all 8 Chapter IV test cases (<1–5% error)

## Milestone 7: Python Bindings & Wrapper
**Goal**: Python package installable, high-level API usable.
*Phases 11–12 of MODERNIZATION_PLAN.md*

- [ ] PyO3 bindings for core types and calculation functions
- [ ] Python `System` class (high-level API)
- [ ] Result dataclasses (FlashResult, BubbleResult, DewResult)
- [ ] Component database (JSON)
- [ ] Plotting helpers (Pxy, Txy diagrams via matplotlib)
- [ ] Python test suite (reproduce Chapter IV validation)
- [ ] Write end-user installation guide

## Milestone 8: Jupyter Notebooks
**Goal**: Interactive notebooks reproducing all thesis results.
*Phase 13 of MODERNIZATION_PLAN.md*

- [ ] 01_introduction — Overview, installation, basic API
- [ ] 02_pure_component — PVT, EOS comparison, saturation curves
- [ ] 03_activity_models — Gamma plots, excess Gibbs energy
- [ ] 04_bubble_dew_point — Reproduce Tables 4.6–4.9
- [ ] 05_flash_calculations — Reproduce Tables 4.3–4.4, 4.10
- [ ] 06_critical_points — Reproduce Tables 4.1–4.2
- [ ] 07_kij_regression — Reproduce Tables 4.11–4.12
- [ ] 08_aij_regression — Activity model Aij fitting

---

**Status key**: `[x]` complete · `[ ]` not started · `[~]` in progress
