# Project Roadmap

High-level milestones for the VLE modernization project. For actionable tasks with time estimates, see [TODO.md](TODO.md). For full technical details, see [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md).

---

## Milestone 0: Project Foundation ✓
**Goal**: Repository, documentation structure, and analysis complete.

- [x] Analyze legacy VB6 codebase (~15,000 lines)
- [x] Analyze legacy Pascal codebase (~2,500 lines)
- [x] Create Pascal vs VB6 comparison document
- [x] Create modernization plan with 24 implementation phases *(originally 17; Phase 11 — Performance Foundation — added 2026-07-01; Phase 19 — Downstream Derivative & Database Release — added 2026-07-05; Phase 20 — Steam Tables (IAPWS-IF97) — added 2026-07-07; Phase 21 — NRTL Activity Model + Ammonia — added 2026-07-08; Phase 22 — iOS/macOS FFI via UniFFI — added 2026-07-11; Phase 23 — Android/Kotlin FFI via UniFFI — added 2026-07-12; Phase 24 — Web/JavaScript FFI via wasm-bindgen — added 2026-07-12)*
- [x] Map algorithms to 30 academic references (ACS format) *(originally 22; (23)–(29) added 2026-07-01 with PERFORMANCE_PROPOSAL.md; (30) added 2026-07-05 with DERIVATIVE_RELEASE_PLAN.md)*
- [x] Propose 8 algorithm performance improvements (A–H) *(extended to §A–§M + Performance Engineering tracks 2026-07-01 — see [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md))*
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

## Milestone 4: Component Property Database ✓
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
- [x] Deploy notebook to JupyterHub — deployed to both rocky (primary) and Oracle (standby) via hot-standby architecture; notebook verified end-to-end (Components=15, kij=0.1357, A12=0.5853, A21=0.3458, P-x-y plot renders)

## Milestone 5: CI/CD + Auto-Deploy ✓
**Goal**: Hybrid CI/CD pipeline (build/test wheels on every push, publish to PyPI + crates.io on tag), first PyO3 bindings shipping in the wheel, and automatic sandbox redeploy on release.
*Phase 5 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Opus 4.7 (1M context)*

- [x] Insert M5 in ROADMAP / TODO / MODERNIZATION_PLAN; renumber M5–M10 → M6–M11 (this milestone's first commit)
- [x] `.github/workflows/_build.yml` — reusable cibuildwheel matrix (Linux x64 self-hosted ephemeral, Linux arm64 hosted, macOS arm64 self-hosted, Windows hosted), CPython 3.10+ abi3 wheels
- [x] `.github/workflows/ci.yml` — push/PR/dispatch: cargo fmt + clippy + cargo test + wheel matrix as artifact
- [x] `.github/workflows/release.yml` — `v*` tag: PyPI Trusted Publishing, crates.io publish (1Password-loaded token), GitHub Release *(an M5 auto-deploy job was later removed when the deployment moved to a separate private operator repository — see MODERNIZATION_PLAN.md)*
- [x] `[tool.cibuildwheel]` block in `python/pyproject.toml` (abi3, manylinux_2_28, pytest against the wheel)
- [x] First `#[pymodule]` in `engine/` — `vle._engine` exposes `version()` + the four enum types (`CubicEos`, `ActivityModel`, `MixingRule`, `SatPressureModel`); `python/tests/test_engine.py` exercises the boundary
- [x] `docs/ci.md` — developer overview, ephemerality table, fork-PR guard, retry flow
- [x] `docs/runners/linux-setup.md` — Proxmox LXC + Docker + `myoung34/github-runner` ephemeral
- [x] `docs/runners/macos-setup.md` — Mac mini M1 persistent runner via launchd, toolchain bootstrap
- [x] Private installer (`deploy/local/auto-deploy/{vle-deploy,install-rocky.sh,install-oracle.sh}`) — restricted `command=`-locked SSH wrapper for tag-pinned deploys
- [x] Drop `git pull` from `deploy/scripts/deploy.sh` (the deploy wrapper handles tag checkout)
- [x] Rewrite the CI-driven release section in `PUBLISHING.md`; add "Cutting a release" subsection
- [x] Add PyO3 Bindings Rule to `CLAUDE.md` (mandatory bindings from M5+)
- [x] Verify end-to-end: `git tag v0.1.0 && git push origin v0.1.0` → PyPI + crates.io + GitHub Release + sandbox redeploy on both hosts (shipped via v0.1.0 → v0.1.1 → v0.1.2 progression; v0.1.0 published packages, v0.1.1 fixed Dockerfile arch + workspace-root, v0.1.2 fixed wheel-glob mismatch and finally landed a fully-green deploy)

## Milestone 6: Numerics
**Goal**: Core numerical utilities tested and benchmarked, with PyO3 bindings exposed in the wheel.
*Phase 6 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Opus 4.7 (1M context)*

Executed in three slices: **M6.1** = scalar solvers + utils + their PyO3 bindings (foundation); **M6.2** = Broyden quasi-Newton (multi-variable); **M6.3** = milestone notebook + deploy docs + `v0.2.0` tag.

- [x] Cardano cubic solver with (12) Poling & Prausnitz robustness *(M6.1)*
- [x] Brent's method root finder (default bracketed solver) *(M6.1)*
- [x] Illinois algorithm root finder (lightweight alternative) *(M6.1)*
- [x] Broyden quasi-Newton solver with periodic Jacobian refresh *(M6.2)*
- [x] Halley's method for scalar equations *(M6.1)*
- [x] Utility functions: SumFrac, Norm, convergence checks *(M6.1)*
- [x] PyO3 bindings for every new public function/type added in this milestone (per CLAUDE.md PyO3 Bindings Rule) *(M6.1 + Broyden in M6.2)*
- [x] Unit tests for all numerical methods — validation test passes *(M6.1+M6.2; engine numerics tests: 40 Rust + 27 Python)*
- [x] Create milestone notebook (`notebooks/m06_numerics.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: research-paper snippets (§A–§H algorithm choices), worked convergence comparisons, ≥2 user exercises *(M6.3; 23 cells, 7 code cells, generated by `scripts/build_notebook_m06.py`, executes top-to-bottom in a fresh kernel)*
- [x] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`) — generic install deltas only *(M6.3; no new env vars or services — pure Rust + Python additions, deploy/NOTEBOOKS.md catalogue already listed `m06_numerics.ipynb`)*
- [x] Update private deploy notes (`deploy/local/deploy-notes/milestone-06.md`) *(M6.3)*
- [x] Tag a release (`v0.2.0`) — CI published to PyPI + crates.io *(M6.3; shipped, followed by v0.3.0 and v0.4.0 on the same pipeline)*

## Milestone 7: Pure Component Models
**Goal**: All pure component EOS, saturation pressure, and virial working.
*Phases 7–9 of MODERNIZATION_PLAN.md*

Split into four sub-milestones so the deployable core can ship without
blocking on the long tail of legacy α-function variants and the
Pascal-origin three-parameter EOS. M7.1 covers the variants Chapter IV's
validation cases actually use; the rest land in subsequent releases.

### Milestone 7.1 — Deployable Core (shipped in v0.3.0)
*Executed by Claude Code using Claude Opus 4.7 (1M context)*

- [x] EOS family constants — full table for all 22 variants (5)
- [x] α(Tr) + analytical dα/dTr for **PR1976 / RKS1972 / RK1949 / VdW1870** (§D)
- [x] Z-factor for 2-parameter cubic EOS (cubic solver integration)
- [x] Pure-component fugacity coefficient ln(φ) + departure H^R/RT, S^R/R
- [x] Antoine saturation pressure with analytical dPsat/dT (4)
- [x] Virial equation — pure + multicomponent (Pitzer B⁰/B¹)
- [x] PyO3 bindings for every new public function (M5+ rule)
- [x] Rust + Python tests (50 Rust + 40 Python pass)
- [x] Functional milestone notebook `notebooks/02_pure_component.ipynb`
- [x] Three placeholder notebooks for the deferred sub-milestones below
      (`02b_alpha_zoo.ipynb`, `02c_three_param_eos.ipynb`, `02d_advanced_saturation.ipynb`)
- [x] Update public deploy docs (`deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`)
- [x] Update private deploy notes (`deploy/local/deploy-notes/milestone-07.md`)
- [x] Deploy notebook to JupyterHub — full image rebuild, verify via `${DOMAIN}`

### Milestone 7.2 — Remaining α-Function Zoo (shipped in v0.4.0)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*
*Exercised by `notebooks/02b_alpha_zoo.ipynb` (now live).*

- [x] Port the 12 self-contained 2-parameter α variants from `legacy/vb6/clsQbicsPure.cls:1719`
      (Berthelot, VdWAda1984, RKSGD1978, RKSL1997, RP1978, PRL1997, VdWVald1989,
       RKSmn1980, RKSATmn1995, PRATmng1997, PRMmn1989, PRSV1986)
- [x] Analytical dα/dTr for each (CLAUDE.md *Algorithm Choices* rule)
- [x] Extended PyO3 bindings `eos_alpha_ex` / `eos_d_alpha_d_tr_ex` (M5+ rule) — thread
      the per-component `Zc` / `m` / `n` / `g` / `K₁` parameters across the FFI
- [x] Per-variant numerical-derivative oracle tests (Rust + Python through the wheel)
- [x] Rename `02b_alpha_zoo.ipynb` placeholder → live: 16-variant α(Tr) comparison plots, PRSV K₁ demo
- **OL family (VdWOL1998, RKOL1998, PROL1998) re-scoped to M7.4.** Their α is
  `Tr·(1 + Σ hₖ·…)` where the sum depends on the component's reduced saturation
  pressure (`clsQbicsPure.cls:268`), so it is coupled to the saturation layer and
  lands alongside the non-Antoine saturation models in M7.4.

### Milestone 7.3 — Three-Parameter EOS + Chao-Seader (shipped in v0.5.0)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] Schmidt-Wenzel 3-parameter EOS (β(ω) third parameter, piecewise m(Tr), guarded Tr=1 kink) (4)
- [x] Patel-Teja and Patel-Teja USB (fitted ξc(ω), Ωa/Ωb/Ωc; USB differs only in mixture C-rule) (4)
- [x] C-parameter mixing rules for 3-param EOS (mole-fraction + √B / √A-weighted) — `mixing::c_mix`
- [x] Chao-Seader liquid fugacity correlation with H₂ / methane special cases (4)
- [x] Z-factor + fugacity + departure across all three 3-param EOS — unified (U, W) form, verified vs the legacy cubics
- [x] `02c_three_param_eos.ipynb` now live (Schmidt-Wenzel/Patel-Teja/Chao-Seader, executes top-to-bottom)

### Milestone 7.4 — Advanced Saturation + Maxwell (shipped in v0.6.0)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] OL-family α (VdWOL1998, RKOL1998, PROL1998) — `Tr·(1 + SumHk)` with the
      per-family h-tables; reads the reduced saturation pressure via the new
      `Component.sat_model`, with an **analytical** dα/dTr (chain rule through dPsat/dT)
- [x] Riedel, Müller, RPM, polynomial saturation correlations (4) — unit-normalized to kPa
- [x] PseudoAntoine helper + generic `d_psat_dt` (analytical Antoine, numerical otherwise)
- [x] Maxwell equal-area construction (successive substitution on equal fugacity over the cubic isotherm)
- [x] Boiling-point calculation (closed form for Antoine, Brent for the others)
- [x] Poynting correction factor `exp[V_L·(P − Psat)/(R·T)]` (canonical kPa units)
- [x] `02d_advanced_saturation.ipynb` now live (executes top-to-bottom)

## Milestone 8: Mixture Models + Performance Foundation
**Goal**: Activity models, mixing rules with an exact-derivative core, multicomponent EOS, and a measured, allocation-free engine.
*Phases 10–14 of MODERNIZATION_PLAN.md*

Split into sub-milestones (8.1–8.4) mirroring Milestone 7, each independently
shippable with its own tests and version bump. Sub-milestones 8.2–8.4 were
restructured 2026-07-01 per [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md):
benchmarks + engine mechanics land first (8.2), then the mixing rules arrive
together with the analytic/AD derivative architecture (8.3) that Milestone 9's
Newton loops depend on.

### Milestone 8.1 — Activity Models + Liquid Volume (shipped in v0.7.0)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] 5 activity coefficient models (Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand) with analytical excess enthalpy (§E)
- [x] Rackett and Thomson/COSTALD (18) liquid molar volume
- [x] PyO3 bindings + Rust/Python tests (closed-form vs Table 2.3, numerical-oracle for analytical Wilson Hᴱ)
- [x] Create milestone notebook (`notebooks/03_activity_models.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: Chapter II §2.2 (activity models) snippets, gamma-vs-composition plots, excess Gibbs energy, ≥2 user exercises
- [x] Update the notebook catalogue (`deploy/NOTEBOOKS.md`); touch `deploy/README.md` only if a distribution channel changed

### Milestone 8.2 — Performance Foundation (shipped in v0.8.0)
*Tracks C + E of PERFORMANCE_PROPOSAL.md — measure first, then the free wins. No thermodynamic behavior change.*
*Executed by Claude Code using Claude Fable 5*

- [x] criterion benchmark suite (`engine/benches/engine_bench.rs`) + Python-side FFI boundary benchmark (`scripts/bench_ffi_boundary.py`) — the baseline every later claim is measured against
- [x] Informational CI bench job (reports deltas, non-blocking) — `bench-rust` in `.github/workflows/ci.yml`
- [x] `[profile.release]` (`lto = "fat"`, `codegen-units = 1`); dropped unused `ndarray` dep
- [x] Allocation-free cubic solver / Z-factor path (`([f64; 3], usize)` instead of `Vec`)
- [x] `EosState` caching struct (α, dα/dTr, A, B, U, W computed once per state; Wilson Λ via `WilsonCache` + virial B matrix reuse)
- [x] Stack-allocated composition arrays (n ≤ 8, `smallvec`); Broyden in-place Sherman–Morrison inverse update

### Milestone 8.3 — Mixing Rules + Multicomponent Fugacity + Derivative Core (shipped in v0.8.0)
*Executed by Claude Code using Claude Fable 5*

- [x] 8 a/b mixing rules (Classical, IVDW, IIVDW, Wong-Sandler (21), Huron-Vidal original/simplified, MHV1, MHV2) + 3 C-parameter rules, written once against the generalized (A, B, U, W) mixture core (26) in `engine/src/mixture.rs`
- [x] Multicomponent fugacity coefficients (9) — one closed form for every EOS/rule; Chao-Seader multicomponent (4)
- [x] Analytic ∂ln φ̂ᵢ/∂nⱼ for classical mixing; `num-dual` dual-number AD (27) for Wong-Sandler/MHV1/MHV2 and 3-parameter EOS (§L) — cross-validated against FD oracles. **Lands before any Milestone 9 flash code**
- [x] 3-parameter EOS mixture fugacity (4) — Schmidt-Wenzel + Patel-Teja (linear & √B-weighted) C-mixing

### Milestone 8.4 — Mixture Energy Properties + Validation (shipped in v0.8.0)
*Executed by Claude Code using Claude Fable 5*

- [x] Enthalpy and entropy (ideal + departure + excess) — `engine/src/energy.rs`, analytic `T·dA_mix/dT` for every rule (no FD)
- [x] Unit tests for all mixture calculations — Rust (`mixture.rs`, `energy.rs`) + Python (`test_m8_mixture.py`) validation, golden values + Gibbs-Duhem/Lewis-Randall/Euler invariants — validation tests pass

## Milestone 9: Flash & Regression (shipped in v0.8.0)
**Goal**: All flash calculations pass Chapter IV validation, with guaranteed-convergence modern algorithms.
*Phase 15 of MODERNIZATION_PLAN.md*

Algorithm suite modernized 2026-07-01 (Track A of [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md)); all Newton loops consume the Milestone 8.3 analytic/AD Jacobians.

*Complete — every flash algorithm, its bindings/tests, the Chapter IV validation, and notebooks 04–08 shipped (Claude Code using Claude Fable 5).*

- [x] Wilson K-value initialization (29) + tangent-plane-distance stability analysis (7) (§I) — `flash/init.rs`, `flash/stability.rs`
- [x] Isothermal flash — GDEM-accelerated SS (§J); Rachford-Rice via Halley inside the Leibovici–Neoschil window (§F, guaranteed convergence + negative flash) — `flash/isothermal.rs` (analytic-Jacobian Newton polish is a follow-on refinement)
- [x] Bubble point (T and P) — Wilson-seeded SS with multiplicative-P / bisection-T outer solve (§K); φ-φ and γ-φ paths — `flash/bubble.rs`
- [x] Dew point (T and P) — same structure (§K) — `flash/dew.rs`
- [x] Phase-envelope continuation through the critical point (24) (§K) — `flash/envelope.rs`; Michelsen unified incipient-phase formulation with min-Gibbs root selection through the critical point + tangent-predictor/Newton-corrector continuation
- [x] Adiabatic flash — warm-started nested loop (§M) — `flash/adiabatic.rs`
- [x] Critical point — Heidemann with dual-number Helmholtz derivatives (§G) — `flash/critical.rs` (2-D Newton on {λ_min, cubic form})
- [x] kij regression via Brent's method (§B) — `flash/kij_regression.rs` (`brent_minimize` in numerics)
- [x] Aij regression — Levenberg-Marquardt (4) — `flash/aij_regression.rs`
- [x] Extend criterion benches with RR / flash — `engine/benches/engine_bench.rs` (`bench_flash`)
- [x] Validate Chapter IV cases — isothermal flash (Table 4.10) reproduced against published numbers; kij fit validated on the sub-critical subset (`engine/tests/chapter_iv_validation.rs`). The near-critical kij points and remaining table groups land with the phase-envelope solver + notebooks.
- [x] Create milestone notebooks — professional structure per CLAUDE.md *Notebook Conventions*; each reproduces the referenced Chapter IV table(s), executes top-to-bottom, and has ≥2 exercises with solutions (build scripts in `scripts/build_notebook_m9_*.py`):
  - `notebooks/04_bubble_dew_point.ipynb` — Table 4.6 van Laar bubble P reproduced (y within 1%, P within the Psat-correlation band) + dew/bubble-T demos
  - `notebooks/05_flash_calculations.ipynb` — Table 4.10 isothermal flash reproduced exactly + adiabatic (Table 4.4) energy-balance round-trip
  - `notebooks/06_critical_points.ipynb` — Tables 4.1–4.2 mixtures reproduced within the thesis band (Tc < 2%, Pc < 6%)
  - `notebooks/07_kij_regression.ipynb` — Tables 4.11–4.12 CO₂/butane fit on the sub-critical subset (~0.14, near the literature 0.1357)
  - `notebooks/08_aij_regression.ipynb` — Levenberg-Marquardt recovers the Table 4.5 van Laar parameters from the Table 4.6 data
  - `notebooks/09_3d_phase_surfaces.ipynb` — 3-D showcase: methane/ethane phase-envelope dome + critical locus and the methanol/water P–x–y sail (pre-computed CSVs in `notebooks/data/`), plus the README hero image (`docs/assets/phase_surfaces_hero.png`)
- [x] Update the notebook catalogue (`deploy/NOTEBOOKS.md`) + rebuilt the landing index (`notebooks/index.ipynb`)

## Milestone 10: Python Bindings, Wrapper & Batch API (shipped in v0.8.0)
**Goal**: Python package installable, high-level API usable, batch numpy API makes it "numpy for thermo".
*Phases 16–17 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

> Note: The first `#[pymodule]` shipped in M5. This milestone is the
> end-user `vle.System` high-level API, plotting helpers, and the
> introduction notebook — i.e., the *ergonomics* layer over bindings
> that have been accumulating since M5 under the PyO3 Bindings Rule —
> plus the Track-D batch layer (PERFORMANCE_PROPOSAL.md).

- [x] PyO3 bindings for core types and calculation functions (M5–M9 free-function surface + the M10 `System` pyclass in `engine/src/py_system.rs`)
- [x] Python `System` class (high-level API) backed by a persistent `#[pyclass]` handle — `python/src/vle/system.py`: name-based construction from the bundled DB, friendly EOS/activity/mixing aliases, unit-aware T/P inputs, dataclass results
- [x] Batch numpy API via rust-numpy — array-in/array-out for every property and flash, zero-copy (`engine/src/py_system.rs` `*_batch` methods)
- [x] GIL release (`allow_threads`) + rayon parallelism in batch kernels; warm-start plumbing across batch points (chunked warm-start chains; ~10× flash speedup, 20% fewer iterations on smooth sweeps)
- [x] Boundary benchmark rerun (`scripts/bench_batch_api.py`: z_factor 16×, flash_pt 10× parallel vs scalar loop). *External comparison vs `thermo` / CoolProp deferred — those libs aren't in the env; tracked as a follow-up.*
- [x] Result dataclasses (FlashResult, BubbleResult, DewResult, CriticalResult) + batch result arrays — `python/src/vle/results.py`
- [x] Component database (JSON) — `scripts/build_components_json.py` → `vle/data/components.json` (15 compounds), read via `vle.components`
- [x] Plotting helpers (Pxy, Txy, phase-envelope via matplotlib) — `python/src/vle/plots.py`
- [x] Python test suite (reproduce Chapter IV validation) — `test_system.py`, `test_components.py`, `test_batch.py`, `test_validation.py` (Tables 4.1–4.2, 4.10, 4.11–4.12); 337 tests pass
- [x] Write end-user installation guide — README Quickstart (`vle.System` + batch tour) + `deploy/NOTEBOOKS.md` corrections
- [x] Create milestone notebook (`notebooks/01_introduction.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: Chapter I + Appendix B snippets, `vle.System` tour, unit-aware inputs, batch API + plot, 2 exercises; executes top-to-bottom
- [x] Update the notebook catalogue (`deploy/NOTEBOOKS.md`) + rebuilt the landing index (`notebooks/index.ipynb`)

> **Fixed (this milestone):** `bubble_temperature`/`dew_temperature` for *close-boiling* φ-φ systems (e.g. benzene/cyclohexane, α≈1.02) used to fail mid-bracket (`g(mid) failed`) because the real bubble T sits inside the K≈1 band the old `S(T)=1` objective filtered out as "trivial". `solve_temperature` now **inverts the robust, monotone saturation-pressure solver** (`engine/src/flash/incipient.rs`) — no trivial-K filter, works for φ-φ and γ-φ alike. Regression tests: `flash::bubble::bubble_temperature_close_boiling_phi_phi` + `test_system.py::test_bubble_temperature_close_boiling_txy`.

## Milestone 11: Chapter IV Walkthrough (shipped in v0.8.0)
**Goal**: One cohesive walkthrough of [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md), tying every milestone notebook together.
*Phase 18 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

> Notebooks 01–08 ship incrementally through Milestones 4–10 (each milestone produces the notebook for the feature it built). This milestone is the capstone: it adds the Chapter IV walkthrough and re-verifies every notebook runs top-to-bottom in a fresh kernel.

- [x] Re-run every existing milestone notebook top-to-bottom in a fresh kernel — all 15 pass (the lone `09` "failure" under batch execution is a relative-`data/` CWD artifact; it runs cleanly from `notebooks/`)
- [x] Create `notebooks/10_chapter4_validation_walkthrough.ipynb` — per CLAUDE.md *Notebook Conventions*; reproduces all seven Chapter IV cases (Tables 4.1–4.12) through the high-level `vle.System`, quotes the paper tables, reports % error vs. published values (critical <0.25%, van Laar bubble-P <0.5%, Raoult bubble-T 0.08%, isothermal flash ~1%; §4.4 Wilson dew a solver demo since the thesis's exact Wilson constants aren't bundled; §4.7 kij lands in the sub-critical neighborhood of 0.1357), ≥2 exercises. Executes top-to-bottom.
- [x] Update the notebook catalogue (`deploy/NOTEBOOKS.md`) — marked the 15-notebook collection complete; rebuilt `notebooks/index.ipynb`

## Milestone 12: Downstream Derivative & Database Release (vle-thermo 0.9.x) — **done**
**Goal**: Close the five upstream gaps identified by `stages-thermo` (the planned staged-separation library — the first downstream consumer of the published crate/wheel): expanded component database with ideal-gas Cp coefficients, a Rust-side component database, analytic/dual T and P derivatives of fugacity and K-values, real-mixture heat capacity + partial molar enthalpy, and a packaged γ-φ phase enthalpy. **All five closed** — M12.1 shipped in v0.8.2; M12.2–12.5 shipped in v0.9.0 (both tagged 2026-07-06). **v0.9.1** (2026-07-06) is a follow-up patch fixing the Wong-Sandler departure-enthalpy `db/dT` bug the M12.3 Gibbs–Helmholtz invariant surfaced (DERIVATIVE_RELEASE_PLAN.md §7).
*Phase 19 of MODERNIZATION_PLAN.md*

Full technical detail, current-state audit, and design decisions live in
[DERIVATIVE_RELEASE_PLAN.md](DERIVATIVE_RELEASE_PLAN.md). Ships in **two
releases**: v0.8.2 (M12.1 — data-only fast-track that unblocks the downstream
McCabe–Thiele milestone) and v0.9.0 (M12.2–12.5 — Rust DB + the derivative
core). Execution order 12.1 → 12.5; 12.4 depends on 12.3.

### Milestone 12.1 — Component DB Expansion + Cp Coefficients (→ v0.8.2) — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] Add 9 compounds to the bundled DB (toluene, ethanol, acetone, chloroform, isobutane, isopentane, n-octane, n-nonane, n-decane) via `scripts/build_components_json.py` — 15 → 24 compounds
- [x] Add `cp_coeffs` (Cp°/R polynomial), `cp_t_range`, `cp_source` to the JSON schema for **all 24** compounds (engine convention: `Cp°(T) = R·Σ aₖ·Tᵏ`, R = 8.31451; degree-4 fits of Poling POLING_POLY (30) via CalebBell/chemicals, machine-exact)
- [x] Thread `cp_coeffs` Python → engine: `vle.components.Component` dataclass + `vle.System` → `_engine.System(cp_coeffs=...)` (fixes the silent-zero ideal-Cp bug for DB-built systems)
- [x] Extend the `vle-db` static SQLite seed to the same 24 compounds (`seed_chapter4.sql`)
- [x] Tests (`python/tests/test_components_cp.py`): `psat(tb) ≈ 101.325 kPa` per compound (1% new / 5% legacy); pinned literature Cp°(298.15 K) per compound (±1%); benzene–toluene bubble-T smoke; nonzero-ideal-Cp regression test — full suite 413 passed, 1 skipped
- [x] Docs (parameter reference Cp°/R section, README/SETUP/package-README compound counts) + bump to **0.8.2** (shipped 2026-07-06)

### Milestone 12.2 — Rust-Side Component Database — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] `engine/data/components.json` becomes the canonical generated copy (`include_str!`), script emits all three copies; byte-identical drift test (`python/tests/test_rust_db.py`)
- [x] `engine/src/db.rs` behind a new `component-db` feature (optional serde/serde_json; `python` feature enables it): `component(name) -> Option<Component>`, `available()`, OnceLock-cached, name normalization matching `vle.components` (trim + lowercase, no invented aliases); JSON-absent fields documented as `Component::default()`
- [x] PyO3 bindings (`db_component` → canonical-unit dict, `db_available`) + wheel tests (M5+ rule)
- [x] Rust tests: lookup hit/miss/case+whitespace, all 24 parse, spot-checks of benzene (legacy) & toluene (new) vs JSON literals — 170 engine tests pass; full pytest 418 passed / 1 skipped

### Milestone 12.3 — T/P Derivatives of Fugacity & K-Values — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] Generalized the §L core in T and P: `mixture_params<D>` / `ln_phi_all_generic<D>` / `three_param_uw<D>` / `pure_params<D>` and the activity `ge_terms` (`ln_gamma_all_generic` + `wilson_lambda`) take `t: D, p: D`; new `eos::alpha_generic<D>` + `eos_dimensionless_generic<D>` propagate duals through α(Tr) *(breaking signature change — 0.9.0)*; scalar path proven unchanged (equivalence tests)
- [x] `d_ln_phi_d_t` / `d_ln_phi_d_p` in `mixture.rs` — **dual-universal** (exact for every EOS × rule, ≈2× a scalar call); the analytic 2-param fast-path is a deferred optimization (DERIVATIVE_RELEASE_PLAN.md §7)
- [x] `k_values_with_derivs` in `flash/system.rs` (`KValueDerivs {k, d_ln_k_d_t, d_ln_k_d_p}`) for φ-φ and γ-φ (term-by-term with `gamma_phi_k`: ∂lnγ/∂T dual, `d_psat_dt`, φˢᵃᵗ(T) chain, Poynting T/P, −1/P); ideal-gas + cubic vapor
- [x] Invariant tests: FD oracles (fugacity + K, φ-φ and γ-φ); Gibbs–Helmholtz `Σxᵢ·∂lnφ̂ᵢ/∂T = −H^R/(RT²)` (now machine-precision for **all** rules incl. Wong-Sandler); volumetric `Σxᵢ·∂lnφ̂ᵢ/∂P = (Z−1)/P` (all rules); K-parity — **surfaced + pinned a pre-existing Wong-Sandler departure-enthalpy bug** (fixed post-12.5: `h_departure_rt_mix` dropped WS's T-dependent co-volume `db/dT` term — see DERIVATIVE_RELEASE_PLAN.md §7)
- [x] PyO3 bindings (`System.d_ln_phi_d_t/_d_p/k_values_with_derivs`) + high-level `vle.System` wrappers + wheel tests (`test_m12_derivatives.py`) — 180 engine tests, 422 pytest pass

### Milestone 12.4 — Real Cp, Partial Molar Enthalpy & γ-φ Phase Enthalpy — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] `partial_molar_enthalpy` via `H̄ᵢ = h°ᵢ(T) − RT²·∂lnφ̂ᵢ/∂T` (identity over M12.3's exact `d_ln_phi_d_t`, no new differentiation) — in `energy.rs`
- [x] `phase_cp` = Σxᵢ·Cp°ᵢ + Cp^R via **second-order duals** (`num_dual::Dual2_64` through `ln_phi_all_generic`, `mixture::residual_cp`); `Cp^R = −R(2T·g′ + T²·g″)`, g = Σxᵢlnφ̂ᵢ — no FD needed (num-dual 0.11 Dual2 works under rust 1.85)
- [x] Packaged `SystemSpec`-level `phase_enthalpy_entropy` (`flash/system.rs`) — φ-φ / vapor delegate to `energy::`; γ-φ liquid assembles ideal − Clausius–Clapeyron condensation `ΔH_vap,ᵢ = RT²·(dPsatᵢ/dT)/Psatᵢ` (Ref (4), `TERMOIII.PAS:283/294` — the Phase 14 deferred path) + excess Hᴱ/Sᴱ
- [x] Tests: Euler `Σxᵢ·H̄ᵢ = H` (classical + MHV1); Cp FD oracle + ideal-gas limit; hand-assembled γ-φ methanol/water liquid enthalpy
- [x] PyO3: `System.phase_cp`, `System.partial_molar_enthalpy` + high-level wrappers; **routed `System.enthalpy_entropy` through the new dispatch** — γ-φ systems now return a value instead of erroring for a missing cubic liquid EOS (behavior change, documented). Wheel tests in `test_m12_energy.py`

### Milestone 12.5 — Notebook, Benches & v0.9.0 Release — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] Created milestone notebook `notebooks/11_derivatives_and_database.ipynb` (build script `scripts/build_notebook_m12.py`) per CLAUDE.md *Notebook Conventions* — DB tour, K(T) tangent plot, mixture Cp, partial-molar Euler assertion, 2 exercises with collapsed solutions; executes top-to-bottom (21 cells)
- [x] Extended criterion benches: `k_values` vs `k_values_with_derivs` (measured ~3.5×, computing k + dT + dP) and `phase_cp` in `engine/benches/engine_bench.rs`
- [x] Full doc sync (README, package READMEs, `deploy/NOTEBOOKS.md`, parameter reference) + bump to **0.9.0** (shipped 2026-07-06)

## Milestone 13: Steam Tables — `vle-steam` (IAPWS-IF97) — **shipped (v0.10.0)**
**Goal**: Add an industrial steam-tables capability ("VLE for water only") as a new dependency-free workspace crate `vle-steam` implementing the IAPWS Industrial Formulation 1997 (IF97, R7-97 rev. 2012) — regions 1–5, the saturation line, and backward equations — surfaced through the wheel as `vle.steam` with pint/gauge units and a batch numpy API. Ships as **v0.10.0** (new public API surface = minor bump). Full design record: [STEAM_TABLES_PLAN.md](STEAM_TABLES_PLAN.md).
*Phase 20 of MODERNIZATION_PLAN.md*

Separate crate (not an `engine/` module) because IF97 is self-contained with
zero coupling to the mixture-EOS machinery and is dependency-free (pure `f64`),
keeping it portable to the planned iOS FFI build ([IOS_FFI_PLAN.md](IOS_FFI_PLAN.md)) —
a steam-table iPhone app is the natural first FFI consumer. Public API is
**mass-basis** (kJ/kg, m³/kg) with a `.molar()` view; inputs are **T [K],
P [kPa absolute]** (repo canon). Correctness ground truth: the R7-97(2012)
computer-program verification tables, asserted to full published precision.

### Milestone 13.1 — Crate scaffold + region 4 + region detection — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] `steam/` workspace member (crate `vle-steam`, zero mandatory deps; `approx` dev-only)
- [x] Region 4 saturation line — `psat(T)`, `tsat(P)` (both closed-form), analytic `dPsat/dT`; verified vs R7-97 Tables 35/36 to 9 sig figs
- [x] B23 boundary (`b23_p`/`b23_t`, Table 25) + `region_of(T,P)` region map with saturation + out-of-range handling
- [x] Hand-rolled `SteamError` + kPa↔MPa boundary helpers; 12 tests pass

### Milestone 13.2 — Regions 1 & 2 (Gibbs + properties) — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] IF97 coefficient tables (regions 1/2/3/5) transcribed from R7-97(2012)
- [x] Region 1 Gibbs `γ(π,τ)` + all properties (v,u,h,s,cp,cv,w) — verified vs Table 5
- [x] Region 2 ideal+residual Gibbs + properties — verified vs Table 15 (subagent web cross-check caught an `e-16`/`e-6` typo in the last residual coeff, −45% in cp)
- [x] Shared Gibbs/Helmholtz → property maps with mass-basis unit bookkeeping

### Milestone 13.3 — Region 3 (Helmholtz + ρ-iteration) + region 5 — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] Region 3 Helmholtz `φ(δ,τ)` + density iteration (Brent) for `(T,P)` queries; verified vs Table 33
- [x] Region 5 high-T Gibbs; verified vs Table 42
- [x] Region-boundary continuity tests (1/3, 2/3, 2/5 seams within 0.1%)

### Milestone 13.4 — State API + backward equations + consistency — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] State constructors `tp/tx/px/ph/ps` + `sat_t/sat_p`; quality logic; latent heat; `.molar()` view
- [x] Backward `T(p,h)` / `T(p,s)` region 1 (verified vs Tables 7/9) as seed + Newton polish; region 2+ via bracketed forward solve
- [x] Thermodynamic-consistency tests: `h=u+pv`, Clausius–Clapeyron, ph/ps round-trips

### Milestone 13.5 — PyO3 bindings + `vle.steam` wrapper + batch numpy — **done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] `engine` `steam` feature (`dep:vle-steam`); `py_steam.rs` (`SteamState`/`SatState` pyclasses + module fns + rayon batch kernels)
- [x] `vle.steam` Python wrapper (`Water(...)`, `saturation`, `properties`/`ph_flash`/`sat_table`) with pint/gauge units; `test_steam.py` (18 tests, 443 pytest pass)

### Milestone 13.6 — Notebook, README, docs & v0.10.0 release — **shipped (v0.10.0)**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] Milestone notebook `notebooks/12_steam_tables.ipynb` (T–s dome, turbine expansion worked example, flash-steam + reboiler-duty exercises) per Notebook Conventions
- [x] `steam/README.md` (crates.io page); criterion benches (`steam/benches/steam_bench.rs`, one point per region + PH flash)
- [x] CLAUDE.md release-rule entry (#12) + architecture tree; full doc sync (README, package READMEs, NOTEBOOKS); version bumped workspace-wide to **v0.10.0**; `vle-steam` wired into `publish-crate.sh` + `release.yml`
- [x] **Operator step:** signed `v0.10.0` tag pushed + published (vle-units → vle-steam → vle-thermo); GitHub Release is Latest (2026-07-08)

---

## Milestone 14: NRTL Activity Model + Ammonia (vle-thermo 0.11.0) — **shipped (v0.11.0)**

*Phase 21 of MODERNIZATION_PLAN.md*

*Executed by Claude Code using Claude Opus 4.8 (1M context)*

Upstream enabler for the downstream `stages-thermo` library's Ponchon–Savarit
milestone (heat-of-mixing on the ammonia–water enthalpy–composition method). Adds
the **NRTL** activity model (general multicomponent form; analytic ∂lnγ/∂T and Hᴱ
via `num-dual`) and **ammonia** to the bundled component database. Design record:
[NRTL_AMMONIA_PLAN.md](NRTL_AMMONIA_PLAN.md).

- [x] `ActivityModel::Nrtl` (project ID 37) — general multicomponent γ + column-sum
      form, generic over the scalar type; f64 + dual paths; analytic Hᴱ via a
      T-seeded dual; binary closed-form + ternary generic-vs-f64 + Hᴱ-vs-oracle tests
- [x] Symmetric `alpha` non-randomness matrix threaded through `SystemSpec`,
      `GeSpec`, the `System` pyclass, and the activity / energy / mixture layers
      (option B — correct for ternary, serves the later extractive systems)
- [x] PyO3: `alpha=` on the four `activity_*` free functions, the `System`
      constructor, and `fit_aij` (NRTL energies fitted with α fixed)
- [x] Python: `"nrtl"` alias + `alpha=` kwarg on `vle.System`
- [x] Ammonia in the component DB (generator + all three JSON copies + Cp°/R
      quartic); Rust `all_25_compounds_parse` + Python DB tests updated
- [x] Milestone notebook (`notebooks/13_nrtl_ammonia.ipynb`) + NOTEBOOKS catalogue — NH₃–H₂O γ, exothermic Hᴱ, bubble-P curve (α = 0.2), with α-sensitivity and Aij-regression exercises; executes top-to-bottom
- [~] NH₃–H₂O NRTL parameters — qualitatively correct behavior demonstrated (α = 0.2 + illustrative energies: negative deviation, exothermic mixing, ammonia-rich vapor). Regression against a published bubble-P–x dataset is deferred; the definitive ammonia–water chart is reproduced from reference data downstream in `stages-thermo`, per the plan's accuracy bar
- [x] **Operator step:** version bumped → v0.11.0; signed `v0.11.0` tag pushed; `release.yml` publishes vle-thermo to crates.io + PyPI — YubiKey-gated

---

## Milestone 15: iOS/macOS FFI — `vle-ffi` (Rust → Swift via UniFFI) — **done (unreleased; local-build artifact)**

*Phase 22 of MODERNIZATION_PLAN.md*

*Executed by Claude Code using Claude Fable 5*

Compiles the engine into a Swift package for native Apple apps — steam
tables, the bundled component DB, and mixture flash callable from SwiftUI
on iOS **and** macOS (the XCFramework carries a native slice per platform,
so one Multiplatform app serves both). **All builds are local to a Mac by
design**: no CI involvement, no published or committed binaries — the repo
ships source + one build script, and `docs/en/ios/README.md` teaches anyone
to reproduce the artifact. No release needed: nothing published to
crates.io/PyPI changed. No milestone notebook: the artifact is a Swift
package, which Jupyter cannot execute — its teaching role is filled by the
learning doc + the XCTest suite. Design record:
[IOS_FFI_PLAN.md](IOS_FFI_PLAN.md) (drafted as "M14", renumbered on
adoption since NRTL landed first).

- [x] `ffi/` crate (`vle-ffi`, `publish = false`, staticlib+lib) — UniFFI
      proc-macro wrapper: `version()`, component DB (`db_available`,
      `db_component` → `ComponentData`), steam tables (`steam_tp/tx/px/ph/ps`,
      `sat_t/sat_p`, latent heat), mixture `VleSystem` object
      (explicit-components + `from_db` constructors; 22 cubic EOS, 6 activity
      models, 11 mixing rules as mirrored enums; `flash_tp`, `bubble_p/t`,
      `dew_p/t`, `k_values`), one `VleFfiError` → Swift `throws`; engine built
      **without** the `python` feature (no pyo3 in the Apple graph); 15 Rust
      wrapper tests
- [x] `ffi/uniffi-bindgen/` bin crate (library-mode `uniffi-bindgen-swift`,
      version-locked to the scaffolding) + `ffi/uniffi.toml`
      (`ffi_module_name = "VleFFI"`)
- [x] `scripts/build-ios.sh` — idempotent: 3 targets (`aarch64-apple-ios`,
      `-ios-sim`, `-darwin`) → bindings → `VleFFI.xcframework` →
      `swift test`; deployment targets iOS 16 / macOS 13
- [x] `swift/VleThermo/` Swift package — `binaryTarget` + generated wrapper
      (gitignored) + hand-written `Extensions.swift` + 10 XCTests through the
      real FFI boundary (IAPWS-IF97 verification point, Chapter IV flash
      configuration, error mapping) — green on the macOS slice
- [x] Learning doc `docs/en/ios/README.md` (C ABI, static libs vs
      XCFramework, UniFFI lift/lower, device-vs-simulator arm64, the
      `module.modulemap` gotcha, troubleshooting) + README/`.gitignore` sync
- [ ] The actual iOS/macOS app (separate repo, out of scope here) — consumes
      `swift/VleThermo` as a local package

---

## Milestone 16: Android/Kotlin FFI — `vle-ffi` → Kotlin via UniFFI — **code complete (first Android Studio run pending)**

*Phase 23 of MODERNIZATION_PLAN.md*

*Executed by Claude Code using Claude Fable 5*

Second consumer language for the M15 wrapper crate: the engine becomes a
Kotlin library for a native **Android app** (Jetpack Compose) and a
**Windows desktop app** (Compose Multiplatform — same Compose UI on the
desktop JVM). Same hard constraints as M15: **all builds local, no CI, no
committed or published binaries** (even the Gradle wrapper stays out — it
contains a jar). No release: nothing on crates.io/PyPI changed. No
milestone notebook (Kotlin isn't executable from Jupyter) — the learning
doc + smoke tests fill that role. Design record + framework decision log
(Kotlin/Compose chosen over MAUI/Avalonia; WSA is dead; C#/.NET
version-blocked): [ANDROID_FFI_PLAN.md](ANDROID_FFI_PLAN.md).

- [x] `ffi/` gains `"cdylib"` crate-type (JNA loads a shared library:
      `.so`/`.dylib`/`.dll`); zero new FFI surface — the whole M15 API
      (component DB, steam, `VleSystem`) comes along for free
- [x] `ffi/uniffi-bindgen/` second bin `uniffi-bindgen` (uniffi's general
      CLI, `generate --language kotlin`, library mode, version-locked) +
      `ffi/uniffi.toml` `[bindings.kotlin]`
      (`package_name = "dev.migueljackson.vle.ffi"`, plain-JVM flavor so
      one binding serves Android *and* desktop JVM)
- [x] `scripts/build-android.sh` — idempotent: cargo-ndk `.so`s
      (arm64-v8a + x86_64 default, `ABIS=` override) + host lib → Kotlin
      bindgen → drop into the Gradle module → host-JVM tests when a
      Gradle exists
- [x] `kotlin/VleThermo/` Android library module (AGP 8 / Kotlin 2 /
      minSdk 24; JNA `@aar` + desktop-jar for tests; `jna.library.path`
      wired to `target/release/`) + 5 committed smoke tests (version,
      water lookup, IF97 1-atm boiling, Ch. IV heptane/butane flash,
      error mapping)
- [x] Docs: learning guide `docs/en/android/README.md`, parked C#/.NET
      route `docs/en/dotnet/README.md` (version-blocked as of 2026-07-12),
      README/CLAUDE/deploy/`.gitignore` sync
- [ ] Verify in Android Studio on the dev machine (open `kotlin/`, run the
      smoke tests, emulator run from the app repo)
- [ ] The actual Compose app — Android + Windows desktop (separate repo,
      out of scope here) — consumes `kotlin/VleThermo` by path

---

## Milestone 17: Web/JavaScript FFI — `vle-wasm` → the browser via wasm-bindgen — **complete**

*Phase 24 of MODERNIZATION_PLAN.md*

*Executed by Claude Code using Claude Fable 5*

Third consumer language: the engine compiles to **WebAssembly** and
becomes an npm package for JavaScript/TypeScript apps — a **pure-React
website** (thermodynamics runs client-side, no compute server) and the
same bundle wrapped as **Windows/Android apps** via the webview shells
(Tauri 2, Electron, Capacitor; shell choice deferred to the app repo).
Same hard constraints as M15/M16: **all builds local, no CI, no committed
or published binaries** (nothing on npm). No release: nothing on
crates.io/PyPI changed. No milestone notebook (JS isn't executable from
Jupyter) — the learning doc + smoke tests fill that role. Design record +
framework decision log (React+wasm chosen over Flutter/React Native; the
verified feasibility spike; the single-threaded/rayon decomposition):
[WEB_UI_PLAN.md](WEB_UI_PLAN.md).

- [x] `wasm/` wrapper crate `vle-wasm` (`publish = false`, cdylib+rlib) —
      a **sibling** of `ffi/`, not an extension (UniFFI has no JS backend
      at our 0.32 pin; wasm-bindgen is the standard): engine with
      `component-db` + `steam`, never `python`
- [x] The M15/M16 API surface in JS form: `version()`, component DB
      (`dbAvailable`/`dbComponent` + custom components from object
      literals), steam tables (`steamTp/Tx/Px/Ph/Ps`, `steamSatT/P`,
      `steamPsat/Tsat`, `steamLatentHeat`), `VleSystem`
      (`flashTp`, `bubbleP/T`, `dewP/T`, `kValues`) — records as plain
      camelCase JS objects (serde-wasm-bindgen), compositions as
      `Float64Array`, model names as forgiving strings or tagged objects,
      Rust errors thrown as JS `Error`s with the family's message prefixes
- [x] `scripts/build-wasm.sh` — idempotent: Node smoke tests
      (`wasm-pack test --node`) → `wasm-pack build --target web --release`
      → `wasm/pkg/` (~360 KB wasm, ~150 KB gzipped, full engine + DB)
- [x] Verification ladder all green: 19 host-side unit tests
      (`cargo test -p vle-wasm`), 5 smoke tests through the real JS↔wasm
      boundary in Node (version, water lookup, IF97 1-atm boiling,
      Ch. IV Table 4.10 heptane/butane flash β within the thesis band,
      error mapping), plus a package-level sanity run of `wasm/pkg`
- [x] Docs: learning guide `docs/en/web/README.md` (theory, React
      quickstart, Web Worker pattern, plotly.js 3-D surfaces, shells,
      troubleshooting), README/CLAUDE/deploy/`.gitignore` sync,
      WEB_UI_PLAN.md adopted
- [ ] The actual React app — website + Tauri/Electron/Capacitor wrapping
      (separate repo, out of scope here) — consumes `wasm/pkg` by path

---

**Status key**: `[✓]` complete · `[ ]` not started · `[~]` in progress
