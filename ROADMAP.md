# Project Roadmap

High-level milestones for the VLE modernization project. For actionable tasks with time estimates, see [TODO.md](TODO.md). For full technical details, see [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md).

---

## Milestone 0: Project Foundation ✓
**Goal**: Repository, documentation structure, and analysis complete.

- [x] Analyze legacy VB6 codebase (~15,000 lines)
- [x] Analyze legacy Pascal codebase (~2,500 lines)
- [x] Create Pascal vs VB6 comparison document
- [x] Create modernization plan with 18 implementation phases *(originally 17; Phase 11 — Performance Foundation — added 2026-07-01)*
- [x] Map algorithms to 29 academic references (ACS format) *(originally 22; (23)–(29) added 2026-07-01 with PERFORMANCE_PROPOSAL.md)*
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
- [x] `.github/workflows/release.yml` — `v*` tag: PyPI Trusted Publishing, crates.io publish (1Password-loaded token), GitHub Release *(an M5 auto-deploy job was later removed when the deployment moved to `homelab-iac` — see MODERNIZATION_PLAN.md)*
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

### Milestone 8.1 — Activity Models + Liquid Volume (complete)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] 5 activity coefficient models (Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand) with analytical excess enthalpy (§E)
- [x] Rackett and Thomson/COSTALD (18) liquid molar volume
- [x] PyO3 bindings + Rust/Python tests (closed-form vs Table 2.3, numerical-oracle for analytical Wilson Hᴱ)
- [x] Create milestone notebook (`notebooks/03_activity_models.ipynb`) — professional structure per CLAUDE.md *Notebook Conventions*: Chapter II §2.2 (activity models) snippets, gamma-vs-composition plots, excess Gibbs energy, ≥2 user exercises
- [x] Update the notebook catalogue (`deploy/NOTEBOOKS.md`); touch `deploy/README.md` only if a distribution channel changed

### Milestone 8.2 — Performance Foundation (complete)
*Tracks C + E of PERFORMANCE_PROPOSAL.md — measure first, then the free wins. No thermodynamic behavior change.*
*Executed by Claude Code using Claude Fable 5*

- [x] criterion benchmark suite (`engine/benches/engine_bench.rs`) + Python-side FFI boundary benchmark (`scripts/bench_ffi_boundary.py`) — the baseline every later claim is measured against
- [x] Informational CI bench job (reports deltas, non-blocking) — `bench-rust` in `.github/workflows/ci.yml`
- [x] `[profile.release]` (`lto = "fat"`, `codegen-units = 1`); dropped unused `ndarray` dep
- [x] Allocation-free cubic solver / Z-factor path (`([f64; 3], usize)` instead of `Vec`)
- [x] `EosState` caching struct (α, dα/dTr, A, B, U, W computed once per state; Wilson Λ via `WilsonCache` + virial B matrix reuse)
- [x] Stack-allocated composition arrays (n ≤ 8, `smallvec`); Broyden in-place Sherman–Morrison inverse update

### Milestone 8.3 — Mixing Rules + Multicomponent Fugacity + Derivative Core (complete)
*Executed by Claude Code using Claude Fable 5*

- [x] 8 a/b mixing rules (Classical, IVDW, IIVDW, Wong-Sandler (21), Huron-Vidal original/simplified, MHV1, MHV2) + 3 C-parameter rules, written once against the generalized (A, B, U, W) mixture core (26) in `engine/src/mixture.rs`
- [x] Multicomponent fugacity coefficients (9) — one closed form for every EOS/rule; Chao-Seader multicomponent (4)
- [x] Analytic ∂ln φ̂ᵢ/∂nⱼ for classical mixing; `num-dual` dual-number AD (27) for Wong-Sandler/MHV1/MHV2 and 3-parameter EOS (§L) — cross-validated against FD oracles. **Lands before any Milestone 9 flash code**
- [x] 3-parameter EOS mixture fugacity (4) — Schmidt-Wenzel + Patel-Teja (linear & √B-weighted) C-mixing

### Milestone 8.4 — Mixture Energy Properties + Validation (complete)
*Executed by Claude Code using Claude Fable 5*

- [x] Enthalpy and entropy (ideal + departure + excess) — `engine/src/energy.rs`, analytic `T·dA_mix/dT` for every rule (no FD)
- [x] Unit tests for all mixture calculations — Rust (`mixture.rs`, `energy.rs`) + Python (`test_m8_mixture.py`) validation, golden values + Gibbs-Duhem/Lewis-Randall/Euler invariants — validation tests pass

## Milestone 9: Flash & Regression
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
- [x] Update the notebook catalogue (`deploy/NOTEBOOKS.md`) + rebuilt the landing index (`notebooks/index.ipynb`)

## Milestone 10: Python Bindings, Wrapper & Batch API
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

> **Known follow-up (pre-existing M9 engine gap, surfaced by the M10 Txy helper):** `bubble_temperature` for *close-boiling* φ-φ systems (e.g. benzene/cyclohexane, α≈1.02) fails mid-bracket (`g(mid) failed`) because the real bubble T sits inside the K≈1 trivial-solution band that `solve_temperature` filters out. `bubble_pressure` is robust. Fixing needs a trivial-band-aware bracketing/bisection in `engine/src/flash/incipient.rs`, re-validated against Chapter IV.

## Milestone 11: Chapter IV Walkthrough & Final Deployment
**Goal**: One cohesive walkthrough of [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md) and a final full-stack redeploy of every milestone notebook.
*Phase 18 of MODERNIZATION_PLAN.md*

> Notebooks 01–08 ship incrementally through Milestones 4–10 (each milestone produces the notebook for the feature it built). This milestone is the capstone: it adds the Chapter IV walkthrough and verifies every notebook is still reachable after a fresh deploy.

- [ ] Re-run every existing milestone notebook top-to-bottom in a fresh kernel — validation pass
- [ ] Create `notebooks/10_chapter4_validation_walkthrough.ipynb` — professional structure per CLAUDE.md *Notebook Conventions*; walks a reader through all seven Chapter IV cases (Tables 4.1–4.12), pulls quoted snippets from [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md), runs the `vle` library against each table, reports % error vs. published values, and ends with ≥2 user exercises (e.g. "repeat the kij regression for a different binary").
- [ ] Update the notebook catalogue (`deploy/NOTEBOOKS.md`) — mark the full catalogue as published
- [ ] Final hub refresh (operator-side) — run `deploy-vle` (mode=full) in `homelab-iac`, verify every notebook in the catalogue opens and Runs-All cleanly on the hub

---

**Status key**: `[✓]` complete · `[ ]` not started · `[~]` in progress
