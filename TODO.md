# TODO

Actionable tasks with rough time estimates. Grouped by [ROADMAP.md](ROADMAP.md) milestone.
Check off items as they're completed. Time estimates assume working with Claude Code.

---

## Milestone 0: Project Foundation
*Executed by Claude Code using Claude Opus 4.6 (1M context)*

Repository, documentation structure, and legacy analysis complete.

- [x] Analyze legacy VB6 codebase (~15,000 lines)
- [x] Analyze legacy Pascal codebase (~2,500 lines)
- [x] Create Pascal vs VB6 comparison document
- [x] Create modernization plan with 24 implementation phases *(originally 17; Phase 11 — Performance Foundation — added 2026-07-01; Phase 19 — Downstream Derivative & Database Release — added 2026-07-05; Phase 20 — Steam Tables (IAPWS-IF97) — added 2026-07-07; Phase 21 — NRTL Activity Model + Ammonia — added 2026-07-08; Phase 22 — iOS/macOS FFI via UniFFI — added 2026-07-11; Phase 23 — Android/Kotlin FFI via UniFFI — added 2026-07-12; Phase 24 — Web/JavaScript FFI via wasm-bindgen — added 2026-07-12)*
- [x] Map algorithms to 30 academic references (ACS format) *(originally 22; (23)–(29) added 2026-07-01 with PERFORMANCE_PROPOSAL.md; (30) added 2026-07-05 with DERIVATIVE_RELEASE_PLAN.md)*
- [x] Propose 8 algorithm performance improvements (A–H) *(extended to §A–§M + Performance Engineering tracks 2026-07-01 — see PERFORMANCE_PROPOSAL.md)*
- [x] Initialize git repository
- [x] Create README, LICENSE (MIT), .gitignore
- [x] Reorganize repo structure (legacy/, docs/en/, docs/es/)
- [x] Create navigatable English research paper skeleton (13 interlinked files)
- [x] Convert all citations to ACS format

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
- [x] **Deploy notebook to JupyterHub** (~1h) — image rebuilt and deployed to both rocky (primary) and Oracle (standby); notebook `00_component_database.ipynb` runs top-to-bottom against the bundled `components.db`, all five pinned assertions match Chapter IV (`Components = 15`, `kij = 0.1357`, `A12 = 0.5853`, `A21 = 0.3458`, P-x-y plot renders for CO₂/n-butane)

## Milestone 5: CI/CD + Auto-Deploy
*Executed by Claude Code using Claude Opus 4.7 (1M context)*

Hybrid CI/CD pipeline, first PyO3 bindings, and automatic sandbox redeploy on tag pushes (~16–22h total).

- [x] **Doc renumber + README/PUBLISHING refactor** (~1h) — insert new M5 in ROADMAP/TODO/MODERNIZATION_PLAN; shift M5–M10 → M6–M11; update Phase pointers; drop README's Docker subsection; rewrite PUBLISHING.md; add the PyO3 Bindings Rule to CLAUDE.md
- [x] **`.github/workflows/_build.yml`** (~2h) — reusable cibuildwheel matrix: Linux x64 (`ubuntu-latest`), Linux arm64 (`ubuntu-24.04-arm`), macOS arm64 (`macos-14`), Windows (`windows-latest`) — **all GitHub-hosted**, CPython 3.10+ abi3 *(originally Linux x64 + macOS arm64 on self-hosted runners; both moved to GitHub-hosted in v0.12.0 — see `OPTIMIZATION_PLAN_PART2.md` and `docs/ci.md`)*
- [x] **`.github/workflows/ci.yml`** (~1h) — push/PR/dispatch: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, call `_build.yml` (artifact-only) *(the per-job fork-PR guards went away with the self-hosted jobs in v0.12.0; only `bench-rust` remains on lab hardware and its `push || workflow_dispatch` condition is its own guard)*
- [x] **`[tool.cibuildwheel]` block in `python/pyproject.toml`** (~0.5h) — abi3, manylinux_2_28, `pytest {package}/tests`, skip pp + musllinux
- [x] **First `#[pymodule]` in engine/** (~3–4h) — add `pyo3` `abi3-py310` feature to `engine/Cargo.toml`; gate enums in `engine/src/types.rs` with `#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]`; create `engine/src/py_bindings.rs` with `#[pymodule] fn _engine(...)` exposing `version()` + the four enums; add `python/tests/test_engine.py`
- [x] **`docs/ci.md`** (~1h) — developer overview, ephemerality table, fork-PR guard, retry flow, badges
- [x] **`docs/runners/linux-setup.md`** (~1.5h) — Proxmox LXC + Docker + `myoung34/github-runner:latest` ephemeral container; PAT setup; verification; scaling
- [x] **`docs/runners/macos-setup.md`** (~1.5h) — Mac mini M1 launchd service; toolchain bootstrap (Xcode CLT, rustup, four Pythons via python.org or `uv`, maturin); periodic-cleanup checklist *(**RETIRED** in v0.12.0 — doc kept for reference; `vle-mac-01` stopped via `svc.sh` and deregistered)*
- [x] **`.github/workflows/release.yml`** (~3h) — `v*` tag: call `_build.yml`, then `publish-pypi` (Trusted Publishing OIDC), `publish-crates` (1Password-loaded token; `vle-units` then `vle-thermo`), `gh-release` (wheels + sdist attached). *(M5 also shipped an auto-deploy job; it was later removed when the deployment moved to a separate private operator repository — see MODERNIZATION_PLAN.md.)*
- [x] **Drop `git pull` from `deploy/scripts/deploy.sh`** (~0.3h) — tag-checkout happens in the deploy wrapper; deploy.sh becomes pure docker build + up
- [x] **Private auto-deploy installer** (`deploy/local/auto-deploy/{vle-deploy, install-rocky.sh, install-oracle.sh, README.md}`) (~2h) — `/usr/local/bin/vle-deploy` wrapper with tag-regex validation; one-shot installers add the `command="..."` restriction to `~/.ssh/authorized_keys`; fail2ban on rocky
- [x] **PUBLISHING.md rewrite** (~0.5h) — drop GHCR section; add "Cutting a release" subsection with the tag-push flow
- [x] **`deploy/FAILOVER.md`** (~0.3h) — replace `deploy.sh` references that assumed git-pull
- [x] **End-to-end smoke test** (~1h → actual ~4h across v0.1.0–v0.1.2) — pushed `v0.1.0` 2026-05-22; packages landed on all three registries; sandbox redeploy hit three latent Dockerfile bugs (arm64 hardcode, missing workspace root, wheel-glob mismatch) that required v0.1.1 + v0.1.2 patch tags to fully resolve. v0.1.2 ran fully green end-to-end: all 5 build jobs, publish-pypi (env approval), publish-crates, gh-release, deploy-sandbox (rocky + Oracle containers restarted). `pip install vle-thermo==0.1.2 && vle._engine.version() == "0.1.2"` verified.
- [x] **Manual config outside the repo** (~2h, one-time) — 1Password vault + Service Account; PyPI Trusted Publisher; rocky + Oracle SSH installers; runners (`vle-runner-01` Linux LXC + `vle-mac-01` Mac mini) registered and online *(v0.12.0: `vle-mac-01` decommissioned; `vle-runner-01` relabelled `self-hosted,Linux,X64,vle-runner` and now serves only the criterion bench)*

## Milestone 6: Numerics

Sliced into three execution batches: **M6.1** (scalar solvers + utils + bindings, ~10h), **M6.2** (Broyden, ~4h), **M6.3** (notebook + deploy + tag, ~3-4h).

### M6.1 — Scalar solvers + utils + bindings (done)

- [x] **Implement Cardano cubic solver** (~2–3h) — from `McommonFunctions.bas:324`, add (12) robustness for near-degenerate discriminant, (13) volume root selection
- [x] **Implement Brent's method** (~2h) — default bracketed root finder, from VB6 `clsLVE.cls` (Numerical Recipes reference)
- [x] **Implement Illinois algorithm** (~1h) — lightweight modified Regula Falsi
- [x] **Implement Halley's method** (~1h) — for scalar equations (used in Rachford-Rice)
- [x] **Implement utility functions** (~1h) — SumFrac, Norm, convergence checks, parabolic interpolation
- [x] **PyO3 bindings for the M6.1 surface** (~1–2h) — `solve_cubic`, `brent`, `illinois`, `halley`, `sum_frac_residual`, `norm_l1/l2/linf` exposed in `vle._engine`; Python-callback exceptions re-raised through the Rust solvers via a `RefCell<Option<PyErr>>` cache pattern
- [x] **Write numerical method tests** (~2–3h) — 28 Rust unit tests under `engine/src/numerics/` + 20 Python binding tests under `python/tests/test_numerics.py`

### M6.2 — Broyden quasi-Newton (done)

- [x] **Implement Broyden quasi-Newton** (~3–4h) — rank-1 Jacobian update, periodic full refresh every K=5 steps via finite differences, stall detection (emergency refresh on near-zero `Δx·Δx`); `engine/src/numerics/broyden.rs`. Uses `nalgebra::DMatrix` for J and LU re-factorization per step. Includes `BroydenConfig` for tunable tolerances/cadence/FD step. PyO3 binding (`vle._engine.broyden`) preserves Python tracebacks across the FFI boundary via the same `RefCell<Option<PyErr>>` cache pattern as M6.1's brent/illinois/halley. 8 Rust unit tests + 7 Python binding tests.

### M6.3 — Notebook + deploy + tag (done modulo the tag push)

- [x] **Create milestone notebook** (~2–3h) — `notebooks/m06_numerics.ipynb` (23 cells, 7 code cells). Generator at `scripts/build_notebook_m06.py`. Walks Cardano on a Z-factor-style cubic, Brent vs. Illinois iteration-count plot, Halley vs. Newton convergence comparison, Broyden on a 2-equation nonlinear system, plus 2 collapsible-solution exercises. Executes top-to-bottom in a fresh kernel.
- [x] **Update public deploy docs** (~0.5h) — No public-doc deltas required for M6 (no new env vars, no new services). `deploy/NOTEBOOKS.md` catalogue already listed `m06_numerics.ipynb`.
- [x] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-06.md` (gitignored) with the two-mode deploy steps for the v0.2.0 release.
- [x] **Bump GitHub Actions to Node-24 versions** *(side task, folded into M6.3)* — `actions/{checkout,cache,setup-python,upload-artifact,download-artifact}` bumped to their latest majors. Self-hosted runners may need a ≥ 2.327.1 update before the next CI run.
- [x] **Tag a release** (`v0.2.0`) — CI auto-published to PyPI + crates.io and ran the sandbox redeploy; the two-mode deploy split was exercised end-to-end. Subsequent releases `v0.3.0` (M7.1) and `v0.4.0` (M7.2) followed on the same pipeline.

## Milestone 7: Pure Component Models

Split into four sub-milestones; M7.1 ships in v0.3.0, the rest follow.

### Milestone 7.1 — Deployable Core (shipped in v0.3.0)
*Executed by Claude Code using Claude Opus 4.7 (1M context)*

- [x] **Implement EOS family constants** (~1h) — full table for all 22 variants per (5)
- [x] **Implement core α + dα/dTr** (~1h) — PR1976, RKS1972, RK1949, VdW1870 with analytical derivatives; other 18 variants stubbed with `unimplemented!` pointing at the deferred sub-milestone (12 since ported in M7.2; OL family → M7.4; 3-param Pascal → M7.3)
- [x] **Implement Z-factor** (~1h) — 2-parameter cubic EOS via the existing Cardano solver; 3-param EOS return `NotImplemented` cleanly
- [x] **Implement fugacity + H^R/RT, S^R/R** (~1h) — pure component, Abbott form
- [x] **Implement Antoine saturation** (~0.5h) — analytical dPsat/dT; other 5 models stubbed
- [x] **Implement virial equation** (~1h) — Pitzer B⁰/B¹ pure + multicomp Z, ln(φ), H^R/RT, S^R/R
- [x] **PyO3 bindings** (~1h) — every new public function exposed in `vle._engine`
- [x] **Rust + Python tests** (~1h) — 50 Rust + 40 Python, including analytical-vs-numerical derivative oracle and ideal-gas-limit checks
- [x] **Create v0.3.0 notebook** (~1h) — `notebooks/02_pure_component.ipynb`, generated by `scripts/build_notebook_m07.py`
- [x] **Create 3 placeholder notebooks** (~0.5h) — `02b_alpha_zoo.ipynb`, `02c_three_param_eos.ipynb`, `02d_advanced_saturation.ipynb` advertising M7.2/M7.3/M7.4
- [x] **Update public deploy docs** (~0.5h) — `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`
- [x] **Update private deploy notes** (~0.5h) — `deploy/local/deploy-notes/milestone-07.md`
- [x] **Deploy notebooks + full image rebuild** (~1h) — verify v0.3.0 hub via `${DOMAIN}`

### Milestone 7.2 — Remaining α-Function Zoo (shipped in v0.4.0)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Port 12 self-contained α functions** (~4–6h) — Berthelot, VdWAda1984, RKSGD1978,
      RKSL1997, RP1978, PRL1997, VdWVald1989, RKSmn1980, RKSATmn1995, PRATmng1997,
      PRMmn1989, PRSV1986 from `legacy/vb6/clsQbicsPure.cls:1719`
- [x] **Analytical dα/dTr for each** (~2–3h) — verified against a central-difference oracle
- [x] **Extended PyO3 bindings** (~0.5h) — `eos_alpha_ex` / `eos_d_alpha_d_tr_ex` carry the
      per-component `Zc`/`m`/`n`/`g`/`K₁` (M5+ binding rule)
- [x] **Numerical-oracle tests across the new variants** (~1h) — Rust (`eos.rs`) + Python
      (`test_m7_pure_component.py`) through the built wheel
- [x] **Rename `02b_alpha_zoo.ipynb` placeholder → live** (~1h) — 16-variant α(Tr) plots, PRSV K₁ demo
- **OL family deferred to M7.4** — `Tr·(1 + Σ hₖ)` depends on reduced saturation pressure
  (`clsQbicsPure.cls:268`); belongs with the saturation layer, not the pure-α zoo.

### Milestone 7.3 — Three-Parameter EOS + Chao-Seader (shipped in v0.5.0)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Schmidt-Wenzel** — β(ω) third parameter, piecewise m(Tr) with a guarded Tr=1 derivative kink (finite entropy, vs the legacy NaN); from Pascal `TERMOII.PAS` (4)
- [x] **Patel-Teja + Patel-Teja USB** — fitted ξc(ω), Ωa/Ωb, dimensionless C=(1−3ξc)Pr/Tr; USB shares the pure-component α (differs only in the M8 mixture C-rule) (4)
- [x] **Chao-Seader liquid fugacity** — ν⁰+ων¹ with the H₂ / methane special coefficient sets, `chao_seader_ln_phi` + `ChaoSeaderSpecies` (4)
- [x] **C-parameter mixing rules module** — `mixing::c_mix` (mole-fraction, √B-weighted, √A-weighted) ready for M8
- [x] **Z-factor + fugacity + departure for 3-param EOS** — unified general (U, W) cubic form, verified to reproduce the legacy Patel-Teja / Schmidt-Wenzel cubics coefficient-for-coefficient; routed through the existing `eos_*` bindings
- [x] **`02c_three_param_eos.ipynb` now live** — generated by `scripts/build_notebook_m73.py`, executes top-to-bottom (α plot, Z/lnφ table, Chao-Seader, 2 exercises). 61 Rust + 245 Python tests green; bumped **v0.5.0**.

### Milestone 7.4 — Advanced Saturation + Maxwell (shipped in v0.6.0)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **OL-family α** — VdWOL1998 / RKOL1998 / PROL1998, `Tr·(1 + SumHk)` with the per-family h-tables; reads the reduced saturation pressure via the new `Component.sat_model`; **analytical** dα/dTr through the chain rule (matches a numerical oracle)
- [x] **Riedel, Müller, RPM, polynomial correlations** — corresponding-states fits unit-normalized to kPa (the legacy's `ln(Pc/1.0135 bar)` reference becomes `ln(Pc/101.325 kPa)`); each gives ~1 atm at Tb
- [x] **PseudoAntoine helper + generic `d_psat_dt`** — analytical for Antoine, central-difference fallback for the others (per the legacy `DPrVapor_DT`)
- [x] **Maxwell equal-area construction** — successive substitution on equal fugacity over the cubic isotherm (`sat_maxwell`)
- [x] **Boiling-point calculation + Poynting correction** — closed-form/Brent boiling-T inversion; `exp[V_L·(P−Psat)/(R·T)]` in canonical kPa units
- [x] **`02d_advanced_saturation.ipynb` now live** — generated by `scripts/build_notebook_m74.py`, executes top-to-bottom. 75 Rust + 256 Python tests green; bumped **v0.6.0**. New `Component.sat_model` field + 8 new PyO3 bindings (`sat_psat`, `sat_d_psat_dt`, `sat_reduced_psat`, `sat_maxwell`, `boiling_temperature`, `poynting_factor`, `eos_alpha_ol`, `eos_d_alpha_d_tr_ol`).

## Milestone 8: Mixture Models + Performance Foundation

Like Milestone 7, this milestone is split into sub-milestones (8.1–8.4), each
independently shippable with its own tests, notebook contribution, and version
bump. The single planned notebook (`03_activity_models.ipynb`) ships with 8.1.
Sub-milestones 8.2–8.4 restructured 2026-07-01 per PERFORMANCE_PROPOSAL.md
(benchmarks + engine mechanics first, then mixing rules together with the
exact-derivative core that Milestone 9 depends on).

### Milestone 8.1 — Activity Models + Liquid Volume (complete)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Implement 5 activity models** (~4–6h) — Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand in `engine/src/activity.rs`, formulas from research-paper Table 2.3; each with analytical excess enthalpy (Wilson closed-form; the legacy `Hᴱ = Gᴱ` convention for Margules/van Laar/Scatchard) (4)
- [x] **Implement liquid volume models** (~1–2h) — Rackett (Spencer-Danner) + Thomson/COSTALD (18) in `engine/src/liquid_volume.rs`
- [x] **PyO3 bindings** — `liquid_molar_volume`, `activity_ln_gamma`, `activity_excess_{gibbs,enthalpy,entropy}`, and the `VolumeModel` enum (M5+ binding rule)
- [x] **Rust + Python tests** — closed-form checks vs Table 2.3, a numerical-derivative oracle for the analytical Wilson Hᴱ, Gibbs-Duhem consistency
- [x] **Create milestone notebook** (~2–3h) — `notebooks/03_activity_models.ipynb` per CLAUDE.md *Notebook Conventions*: Chapter II §2.2 snippets, gamma vs. composition plots, excess Gibbs energy, ≥2 user exercises
- [x] **Update the notebook catalogue** (~0.3h) — add to `deploy/NOTEBOOKS.md` + version bump

### Milestone 8.2 — Performance Foundation (complete)
*Executed by Claude Code using Claude Fable 5*

Tracks C + E of PERFORMANCE_PROPOSAL.md. Measure first, then the free wins; no
thermodynamic behavior change (gated by the existing test suite).

- [x] **Stand up criterion benches** — `engine/benches/engine_bench.rs`: α dispatch, Z-factor, pure ln φ, saturation, activity γ; informational CI bench job (`bench-rust`, non-blocking deltas). Baseline (Apple M-series, LTO release): α ≈ 3.1 ns, Z-factor ≈ 35 ns, pure ln φ ≈ 82 ns, Wilson γ (ternary) ≈ 29 ns
- [x] **Python boundary benchmark** — `scripts/bench_ffi_boundary.py`: FFI dispatch ≈ 7 ns overhead, scalar `z_factor` ≈ 130 ns, `ln_phi` ≈ 154 ns (the baseline the M10 batch API is measured against)
- [x] **Add `[profile.release]`** — `lto = "fat"`, `codegen-units = 1` (kept `panic = "unwind"` for PyO3); dropped the unused `ndarray` dep
- [x] **Allocation-free cubic/Z path** — `solve_real` → `([f64; 3], usize)` with a 3-element sorting network; `z_factor` root selection by direct min/max scan (no filter/collect/sort)
- [x] **`EosState` caching struct** — α, dα/dTr, A, B, U, W computed once per (T, P, comp), shared by Z/fugacity/departure; `WilsonCache` for the Λ matrix + virial B-matrix reuse
- [x] **Small-n stack allocation + Broyden fix** — `smallvec` composition arrays (n ≤ 8); Sherman–Morrison in-place inverse update (O(n²)/iter, no per-iteration `clone().lu()`)

### Milestone 8.3 — Mixing Rules + Multicomponent Fugacity + Derivative Core (complete)
*Executed by Claude Code using Claude Fable 5*

- [x] **Implement 8 a/b mixing rules** — Classical, IVDW, IIVDW, WS (21), HOV, HVS, MHV1, MHV2, plus the 3 Schmidt-Wenzel/Patel-Teja C-parameter rules (4); written once against the generalized (A, B, U, W) mixture core (26) in `engine/src/mixture.rs`
- [x] **Implement multicomponent fugacity** — one closed-form partial fugacity coefficient for every EOS/rule (9); 3-param EOS (4); Chao-Seader multicomp (4)
- [x] **Exact-derivative core (§L)** — analytic ∂ln φ̂ᵢ/∂nⱼ for cubic EOS + classical mixing; exotic rules (WS, MHV1/2) and 3-parameter EOS differentiated with `num-dual` dual numbers (27); FD kept as test oracle only. **Lands before any M9 flash code.**

### Milestone 8.4 — Mixture Energy Properties + Validation (complete)
*Executed by Claude Code using Claude Fable 5*

- [x] **Implement enthalpy/entropy** — `engine/src/energy.rs`: ideal Cp integration, EOS departure functions (9) with **analytic** `T·dA_mix/dT` for every rule (GE rules via `T·d(Gᴱ/RT)/dT = −Hᴱ/RT`), excess H/S, reference-state assembly
- [x] **Write mixture model tests** — Rust (`mixture.rs`, `energy.rs`) + Python (`test_m8_mixture.py`): golden values, textbook-PR oracle, VB6 C_cal / Pascal Chao-Seader constants, Gibbs-Duhem/Lewis-Randall/Euler invariants, analytic-vs-FD derivatives — validation tests pass

## Milestone 9: Flash & Regression

Algorithm suite modernized 2026-07-01 (Track A of PERFORMANCE_PROPOSAL.md);
every Newton loop consumes the M8.3 analytic/AD Jacobians (§L).

*Complete (shipped in v0.8.0) — every flash algorithm, its PyO3 bindings and
tests (`test_m9_flash.py`), the Chapter IV validation, and notebooks 04–09, by
Claude Code using Claude Fable 5.*

- [x] **Implement Wilson init + stability analysis** — Wilson K-value correlation (29); tangent-plane-distance stability test (7) supplying phase count + non-trivial K estimates (§I) — `flash/init.rs`, `flash/stability.rs`
- [x] **Implement isothermal flash** — GDEM-accelerated SS (25) (§J); Rachford-Rice via Halley inside the Leibovici–Neoschil window (23) with bisection safeguard (§F); φ-φ + γ-φ K-value dispatch in `flash/system.rs`. Newton-on-lnK polish is a follow-on refinement
- [x] **Implement bubble point (T and P)** — Wilson-seeded SS with multiplicative-P / bisection-T outer solve (§K); φ-φ and γ-φ — `flash/bubble.rs`, shared core `flash/incipient.rs`
- [x] **Implement dew point (T and P)** — same structure (§K) — `flash/dew.rs`
- [x] **Implement phase-envelope continuation** — Michelsen predictor-corrector (24), adaptive step through the critical point (§K) — `flash/envelope.rs`. Unified incipient-phase (n+2)-variable Newton with min-Gibbs root selection through the critical point; `trace_envelope_py` binding + tests
- [x] **Implement adiabatic flash** — warm-started nested T-loop (bisection outer, K-seeded inner flash) (§M) — `flash/adiabatic.rs`, `flash_isothermal_warm`
- [x] **Implement critical point** — Heidemann (16) with **dual-number** Helmholtz derivatives (§G); 2-D Newton on {λ_min, cubic form} — `flash/critical.rs`
- [x] **Implement kij regression** — Brent minimization replacing golden section (4) — `flash/kij_regression.rs` + `numerics::root_finding::brent_minimize`
- [x] **Implement Aij regression** — Levenberg-Marquardt for Margules/VanLaar/Wilson (4) — `flash/aij_regression.rs`
- [x] **Extend criterion benches** — RR + isothermal flash — `engine/benches/engine_bench.rs`
- [x] **Validate Chapter IV cases** — isothermal flash (Table 4.10) reproduced vs published x₁/y₁/β; kij fit validated on the sub-critical subset — `engine/tests/chapter_iv_validation.rs`. Full-dataset kij + the remaining table groups land with the phase envelope + notebooks
- [x] **Create milestone notebooks** — per CLAUDE.md *Notebook Conventions*, one per Chapter IV table group; each executes top-to-bottom, reproduces the referenced table(s), and has ≥2 exercises with solutions (deterministic build scripts `scripts/build_notebook_m9_*.py`):
  - `notebooks/04_bubble_dew_point.ipynb` — Table 4.6 (van Laar bubble P) + dew/bubble-T demos
  - `notebooks/05_flash_calculations.ipynb` — Table 4.10 (isothermal, exact) + Table 4.4 (adiabatic round-trip)
  - `notebooks/06_critical_points.ipynb` — Tables 4.1–4.2 (within the thesis band)
  - `notebooks/07_kij_regression.ipynb` — Tables 4.11–4.12 (sub-critical subset)
  - `notebooks/08_aij_regression.ipynb` — recovers the Table 4.5 van Laar parameters by LM
  - `notebooks/09_3d_phase_surfaces.ipynb` — 3-D showcase (phase-envelope dome + critical locus + P–x–y sail from pre-computed CSVs) + README hero image
- [x] **Update the notebook catalogue** — `deploy/NOTEBOOKS.md` rows present; rebuilt `notebooks/index.ipynb`

## Milestone 10: Python Bindings, Wrapper & Batch API

*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Create PyO3 bindings** (~4–6h) — M5–M9 free-function surface + the M10 persistent `System` `#[pyclass]` (`engine/src/py_system.rs`, registered in `py_bindings.rs`)
- [x] **Build Python `System` class** (~3–4h) — `python/src/vle/system.py`: `flash_pt`/`flash_ph`, `bubble_*`/`dew_*`, `critical_point`, `phase_envelope`, all backed by the persistent pyclass; name-based construction, friendly aliases, unit-aware inputs
- [x] **Build batch numpy API** (~5–7h) — rust-numpy array-in/array-out for every property + flash (zero-copy); `allow_threads` + rayon over state points; chunked warm-start chains (Track D)
- [x] **Rerun boundary benchmark** (~1–2h) — `scripts/bench_batch_api.py` vs the M8.2 baseline: z_factor 16×, flash_pt 10× (parallel) vs a scalar loop, 20% fewer iterations warm-started. *External comparison vs `thermo` / CoolProp deferred (libs not in env).*
- [x] **Create result dataclasses** (~1–2h) — `python/src/vle/results.py`: FlashResult, BubbleResult, DewResult, CriticalResult + BatchFlashResult / BatchSaturationResult
- [x] **Build component database** (~2–3h) — `scripts/build_components_json.py` → `vle/data/components.json` + `notebooks/data/components.json` (15 compounds); read via `vle.components`, shipped in the wheel
- [x] **Build plotting helpers** (~2–3h) — `python/src/vle/plots.py`: Pxy, Txy, phase-envelope diagrams (matplotlib, optional dep)
- [x] **Write Python test suite** (~2–3h) — `test_system.py`, `test_components.py`, `test_batch.py` (scalar↔batch parity), `test_validation.py` (Chapter IV Tables 4.1–4.2, 4.10, 4.11–4.12); 337 tests pass
- [x] **Write installation guide** (~1h) — README Quickstart (`vle.System` + batch tour) + `deploy/NOTEBOOKS.md` `vle`→`vle-thermo` fixes
- [x] **Create milestone notebook** (~2–3h) — `notebooks/01_introduction.ipynb` per CLAUDE.md *Notebook Conventions*; executes top-to-bottom, 2 exercises with solutions
- [x] **Update the notebook catalogue** (~0.3h) — `deploy/NOTEBOOKS.md` + rebuilt `notebooks/index.ipynb`

## Milestone 11: Chapter IV Walkthrough

*Executed by Claude Code using Claude Opus 4.8 (1M context)*

Notebooks 01–08 ship incrementally through Milestones 4–10. This milestone is the capstone: one new walkthrough notebook covering all Chapter IV results.

- [x] **Re-run all prior milestone notebooks** (~1–2h) — fresh kernel via nbclient; all 15 pass (notebook `09`'s relative-`data/` path only resolves from `notebooks/` CWD)
- [x] **Create `notebooks/10_chapter4_validation_walkthrough.ipynb`** (~4–6h) — per CLAUDE.md *Notebook Conventions*: narrated §4.1–§4.7 through `vle.System`, % error vs. published values, 2 exercises; executes top-to-bottom
- [x] **Update the notebook catalogue** (~0.3h) — `deploy/NOTEBOOKS.md` marked complete; rebuilt `notebooks/index.ipynb`

## Milestone 12: Downstream Derivative & Database Release (vle-thermo 0.9.x)

Closes the five upstream gaps identified by `stages-thermo` (planned first
downstream consumer). Full technical spec: [DERIVATIVE_RELEASE_PLAN.md](DERIVATIVE_RELEASE_PLAN.md).
Two releases: **v0.8.2** (12.1 fast-track) then **v0.9.0** (12.2–12.5).
Total ~25–38h. All sub-milestones **done** 2026-07-05→07-06 (Claude Opus 4.8):
M12.1 (v0.8.2 data), M12.2–12.5 (v0.9.0 — Rust DB, T/P derivatives, real Cp +
partial molar H + γ-φ enthalpy, notebook + benches). Both tags shipped
2026-07-06. **v0.9.1** (2026-07-06, Claude Fable 5) patched the Wong-Sandler
departure-enthalpy `db/dT` bug the M12.3 invariant surfaced (plan §7).

### Milestone 12.1 — Component DB Expansion + Cp Coefficients (→ v0.8.2, ~4–6h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Extend `scripts/build_components_json.py`** — add toluene, ethanol, acetone, chloroform, isobutane, isopentane, n-octane, n-nonane, n-decane (15 → 24 compounds); add `cp_coeffs` (dimensionless Cp°/R polynomial matching `energy::ideal_cp`), `cp_t_range`, `cp_source` for all 24; sourced via `thermo` 0.6.0 / `chemicals` 1.5.2 (POLING_POLY degree-4 fits), cross-checked vs Poling 5th ed. (30); both JSON copies regenerated (byte-identical)
- [x] **Thread `cp_coeffs` Python → engine** — extended the frozen `vle.components.Component` dataclass (+ `cp_t_range`, `cp_source`) and `_to_component`; `vle.System` now passes `cp_coeffs` into `_engine.System` (fixes silently-zero ideal Cp for DB-built systems)
- [x] **Extend the `vle-db` static seed** — same 9 compounds in `seed_chapter4.sql`; `vle-db validate chapter4` unaffected
- [x] **Tests** (`python/tests/test_components_cp.py`) — per-compound `psat(tb)` round-trip (1% new / 5% legacy) + pinned Cp°(298.15 K) literature values (±1%); benzene–toluene bubble-T smoke (355–370 K); nonzero-ideal-Cp regression on `enthalpy_entropy`; `test_db` count 15→24
- [x] **Docs + version** — parameter reference Cp°/R section, README/SETUP/package-README compound counts; bumped to **0.8.2** (shipped 2026-07-06)

### Milestone 12.2 — Rust-Side Component Database (~4–6h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Canonical engine copy** — `engine/data/components.json` via `include_str!`; `scripts/build_components_json.py` emits all three copies; pytest drift guard (`test_rust_db.py`, engine copy ≡ wheel copy, byte-identical)
- [x] **`engine/src/db.rs`** — new `component-db` cargo feature (optional `serde`/`serde_json`, default off, enabled by `python`); `component(name)` / `available()` over a `OnceLock<HashMap>`; name normalization mirrors `vle.components.get` (trim + lowercase); JSON → `Component` fills `cp_coeffs`, `psat_coeffs`, `sat_model=Antoine`, documents the polar/PRSV fields left at `Component::default()`
- [x] **PyO3 bindings + tests** — `db_component` (canonical-unit dict), `db_available` (M5+ rule) + wheel tests; Rust tests: case/whitespace/miss, all 24 parse, benzene+toluene spot-checks — 170 engine tests, 418 pytest pass

### Milestone 12.3 — T/P Derivatives of Fugacity & K-Values (~8–12h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **T/P-generic §L core** — `mixture_params<D>` / `ln_phi_all_generic<D>` / `three_param_uw<D>` / `pure_params<D>` + activity `ge_terms` (`ln_gamma_all_generic`, `wilson_lambda`) take `t: D, p: D`; new `eos::alpha_generic<D>` + `eos_dimensionless_generic<D>` propagate duals through α(Tr); scalar path proven unchanged (equivalence tests). *Breaking public-signature change → 0.9.0*
- [x] **`d_ln_phi_d_t` / `d_ln_phi_d_p`** — **dual-universal** (exact for every EOS×rule, ≈2× a scalar call); analytic 2-param fast-path deferred (DERIVATIVE_RELEASE_PLAN.md §7)
- [x] **`k_values_with_derivs`** — `KValueDerivs {k, d_ln_k_d_t, d_ln_k_d_p}`; φ-φ = L−V difference; γ-φ term-by-term (∂lnγ/∂T dual, `d_psat_dt`, φˢᵃᵗ(T) chain, Poynting T/P, −1/P); K field bit-identical to `k_values`; ideal-gas + cubic vapor
- [x] **Tests + bindings** — `_fd` oracles (fugacity + K, φ-φ and γ-φ); Gibbs–Helmholtz + volumetric invariants — **surfaced a pre-existing Wong-Sandler departure-enthalpy bug** (fixed post-12.5: `h_departure_rt_mix` dropped WS's T-dependent co-volume `db/dT` term — root cause + fix in DERIVATIVE_RELEASE_PLAN.md §7; `t_dln_a_dt_mix` itself was correct); PyO3 `System.d_ln_phi_d_t/_d_p/k_values_with_derivs` + wrappers + `test_m12_derivatives.py`

### Milestone 12.4 — Real Cp, Partial Molar Enthalpy & γ-φ Phase Enthalpy (~6–9h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **`partial_molar_enthalpy`** — `H̄ᵢ = h°ᵢ(T) − RT²·∂lnφ̂ᵢ/∂T` (pure identity over 12.3); Euler test `Σxᵢ·H̄ᵢ = H` (classical + MHV1)
- [x] **`phase_cp`** — Σxᵢ·Cp°ᵢ + Cp^R via `Dual2_64` through the T-generic core (`mixture::residual_cp`; num-dual 0.11 second-order verified working under rust 1.85 — no fallback needed); FD-of-analytic-H oracle + ideal-gas limit test
- [x] **γ-φ `phase_enthalpy_entropy` (SystemSpec-level)** — φ-φ / vapor delegate to `energy::`; γ-φ liquid = ideal − Clausius–Clapeyron condensation `ΔH_vap,ᵢ = RT²·(dPsatᵢ/dT)/Psatᵢ` (Ref (4), `TERMOIII.PAS:283/294`) + `excess_h_s`; hand-assembled methanol/water van Laar liquid-H test
- [x] **Bindings** — `System.phase_cp`, `System.partial_molar_enthalpy` + wrappers; **routed `System.enthalpy_entropy` through the new dispatch** (γ-φ systems now return a value instead of erroring — documented behavior change); `test_m12_energy.py`

### Milestone 12.5 — Notebook, Benches & v0.9.0 Release (~3–5h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Milestone notebook** — `notebooks/11_derivatives_and_database.ipynb` via `scripts/build_notebook_m12.py`, per CLAUDE.md *Notebook Conventions* (setup cell, Chapter II §2.3 context, DB tour, K(T) tangent plot, Cp vs T, Euler assertion, 2 exercises with collapsed solutions); executes top-to-bottom in a fresh kernel (21 cells)
- [x] **Benches** — `k_values` vs `k_values_with_derivs` (measured ~3.5×: computes k + dT + dP) and `phase_cp` in `engine/benches/engine_bench.rs`
- [x] **Doc sync + release** — full CLAUDE.md pre-push list; `python/README.md` / `engine/README.md` 0.9.0 API story; `deploy/NOTEBOOKS.md` row; bump to **0.9.0** (shipped 2026-07-06)

## Milestone 13: Steam Tables — `vle-steam` (IAPWS-IF97)

New dependency-free workspace crate implementing the IAPWS Industrial
Formulation 1997 (IF97) — "VLE for water only" — surfaced as `vle.steam`.
Full spec: [STEAM_TABLES_PLAN.md](STEAM_TABLES_PLAN.md). Ships as **v0.10.0**.
Total ~27–39h (13.1–13.6). Correctness ground truth = the R7-97(2012)
computer-program verification tables (asserted to 9 sig figs).

### Milestone 13.1 — Crate scaffold + region 4 + region detection (~3–5h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **`steam/` workspace member** — crate `vle-steam`, zero mandatory deps (`approx` dev-only), registered in root `Cargo.toml`
- [x] **Region 4 saturation line** — `psat(T)` / `tsat(P)` (both closed-form, Eqs. 30–31) + analytic `dPsat/dT`; verified vs Tables 35/36
- [x] **B23 boundary + region map** — `b23_p`/`b23_t` (Table 25); `region_of(T,P)` with saturation + out-of-range handling
- [x] **Error type + boundary helpers** — hand-rolled `SteamError` (Display/Error), kPa↔MPa conversion; 12 tests pass

### Milestone 13.2 — Regions 1 & 2 (Gibbs + properties) (~6–8h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Coefficient tables** — regions 1/2/3/5 series (`coefficients.rs`) transcribed from R7-97(2012) Tables 2/10/11/30/37/38
- [x] **Region 1** — Gibbs `γ(π,τ)` + all six derivatives → `Props` (v,u,h,s,cp,cv,w); verified vs Table 5
- [x] **Region 2** — ideal `γ°` + residual `γʳ` + derivatives; verified vs Table 15 (a subagent web cross-check caught a `10⁻¹⁶`-vs-`10⁻⁶` typo in the last residual coefficient, invisible in v/h but −45% in cp)
- [x] **Shared property maps** — `gibbs_props` / `helmholtz_props` with mass-basis unit bookkeeping (`1 kPa = 1 kJ/m³`)

### Milestone 13.3 — Region 3 (Helmholtz + ρ-iteration) + region 5 (~5–7h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Region 3** — Helmholtz `φ(δ,τ)` (40 terms) + density iteration (Brent, `solve.rs`) for `(T,P)` queries; verified vs Table 33 + density round-trips; near-critical saturated-density solver
- [x] **Region 5** — high-T Gibbs (ideal + residual); verified vs Table 42
- [x] **Continuity tests** — `v/h/s` agree across the 1/3, 2/3, 2/5 seams within 0.1%

### Milestone 13.4 — State API + backward equations + consistency (~5–7h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **State constructors** — `SteamState::tp/tx/px/ph/ps` + `sat_t/sat_p`; phase classification, two-phase quality logic; `latent_heat`; `.molar()` view
- [x] **Backward equations** — region-1 `T(p,h)` / `T(p,s)` (Tables 6/8, verified vs Tables 7/9) as fast seeds + Newton polish to exactness; region 2+ via bracketed forward solve
- [x] **Consistency tests** — `h=u+pv` (all regions), Clausius–Clapeyron `h_fg≈T·v_fg·dPsat/dT`, `ph/ps(tp)→T` round-trips, two-phase quality round-trip

### Milestone 13.5 — PyO3 bindings + `vle.steam` wrapper + batch numpy (~4–6h) — **Done**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Engine wiring** — `steam` feature (`dep:vle-steam`, `python` includes it, re-export `vle_thermo::steam`); `engine/src/py_steam.rs` (`SteamState`/`SatState` pyclasses + module fns + rayon/GIL-released batch kernels)
- [x] **Python wrapper** — `vle.steam.Water(...)` mode dispatch + `saturation`/`psat`/`tsat`/`latent_heat` + batch `properties`/`ph_flash`/`sat_table`; pint quantities + gauge pressure via the existing registry
- [x] **Tests** — `python/tests/test_steam.py` (18): table verification, unit/gauge handling, batch-vs-scalar, quality logic — full pytest 443 passed / 1 skipped; wheel doctests pass

### Milestone 13.6 — Notebook, README, docs & v0.10.0 release (~4–6h) — **Shipped (v0.10.0)**
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Milestone notebook** — `notebooks/12_steam_tables.ipynb` (build script `scripts/build_notebook_m13.py`): saturation-table page, T–s dome plot, isentropic turbine expansion (worked example), flash-steam recovery + reboiler-duty exercises with collapsed solutions; executes top-to-bottom (24 cells)
- [x] **`steam/README.md`** (crates.io page, compiling example) + criterion benches (`steam/benches/steam_bench.rs`)
- [x] **CLAUDE.md release-rule entry (#12) + architecture tree; full doc sync (README, package READMEs, NOTEBOOKS); version bumped to v0.10.0; `vle-steam` wired into `publish-crate.sh` + `release.yml`**
- [x] **Operator step (YubiKey-gated):** signed `v0.10.0` tag pushed + published (vle-units → vle-steam → vle-thermo); GitHub Release is Latest (2026-07-08)

---

## Milestone 14: NRTL Activity Model + Ammonia (vle-thermo 0.11.0)
*Phase 21 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

Upstream enabler for `stages-thermo` Milestone 2 (Ponchon–Savarit): a proper
heat-of-mixing model + ammonia for the ammonia–water enthalpy–composition method.
Design record: [NRTL_AMMONIA_PLAN.md](NRTL_AMMONIA_PLAN.md).

- [x] **NRTL model** — `ActivityModel::Nrtl` (project ID 37, first free above the
      legacy space). General multicomponent γ via the column-sum form, written once
      generic over the scalar type (f64 value path + dual T/composition path);
      analytic Hᴱ from one T-seeded `num-dual` evaluation. Tests: binary closed-form
      γ, ternary generic-vs-f64, Hᴱ-vs-central-difference oracle, discriminant.
- [x] **`alpha` non-randomness matrix (option B)** — symmetric N×N field threaded
      through `SystemSpec`, `GeSpec`, the `System` pyclass, and the activity / energy
      / mixture layers (correct for ternary → serves the later extractive systems)
- [x] **PyO3 bindings** — `alpha=` on the four `activity_*` free functions, the
      `System` constructor, and `fit_aij` (NRTL energies fitted, α fixed);
      `python/tests/test_m8_activity.py` NRTL cases
- [x] **Python wrapper** — `"nrtl"` alias + `alpha=` kwarg on `vle.System`
- [x] **Ammonia in the component DB** — added to `scripts/build_components_json.py`
      (`RAW_NEW` + Cp°/R quartic + two-point anchor); regenerated all three JSON
      copies; Rust `all_25_compounds_parse` + ammonia spot-check; Python DB tests
- [x] **Milestone notebook** — `notebooks/13_nrtl_ammonia.ipynb` (γ, exothermic
      Hᴱ, bubble-P–x curve at α = 0.2; α-sensitivity + Aij-regression exercises);
      executes top-to-bottom; catalogued in `deploy/NOTEBOOKS.md`
- [~] **NH₃–H₂O NRTL parameters** — qualitatively correct behavior demonstrated
      (α = 0.2 + illustrative energies: negative deviation, exothermic mixing,
      ammonia-rich vapor). Regression against a published bubble-P–x dataset is
      deferred (rigorous ammonia–water chart is `stages-thermo`'s job from
      reference data — matches the plan's "qualitative / few-%" accuracy bar)
- [x] **Operator step (YubiKey-gated):** version bumped → v0.11.0; signed
      `v0.11.0` tag pushed; `release.yml` publishes vle-thermo to crates.io + PyPI

---

## Milestone 15: iOS/macOS FFI — `vle-ffi` (Rust → Swift via UniFFI)
*Phase 22 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Fable 5*

Local-only Apple builds (no CI, no published/committed binaries — see the
hard constraint in [IOS_FFI_PLAN.md](IOS_FFI_PLAN.md); drafted as "M14",
renumbered on adoption). No release: nothing on crates.io/PyPI changed. No
milestone notebook (Swift isn't executable from Jupyter) — the learning doc
+ XCTest suite fill that role.

### 15.1 `ffi/` wrapper crate end-to-end (~4–6h) — **done**
- [x] `ffi/` (`vle-ffi`, `publish = false`, `staticlib`+`lib`) + workspace
      membership; engine dep with `component-db` + `steam`, **never** `python`
- [x] `ffi/uniffi-bindgen/` bin crate (`uniffi-bindgen-swift`, feature `cli`,
      version-locked via one workspace Cargo.lock) + `ffi/uniffi.toml`
      (`ffi_module_name = "VleFFI"` — must match modulemap + binaryTarget)
- [x] `scripts/build-ios.sh`: 3 targets → library-mode bindgen → plain
      (non-`framework`) `module.modulemap` → `xcodebuild -create-xcframework`
      → copy generated Swift → `swift test`; iOS 16 / macOS 13 pinned

### 15.2 Real API surface + tests (~6–10h) — **done**
- [x] `version()`; `db_available()` / `db_component()` → `ComponentData`
      (lossless mirror of `types::Component`, incl. `SatModel`)
- [x] Steam: `steam_tp/tx/px/ph/ps`, `steam_sat_t/sat_p`, `steam_latent_heat`
      → `SteamStateData` / `SatPropsData` (mass-basis kJ/kg units documented)
- [x] `VleSystem` UniFFI object: `new(components:…)` + `from_db(names:…)`;
      enums `CubicEosKind` (22), `ActivityModelKind` (6), `MixingRuleKind`
      (11), `VaporSpec`/`LiquidSpec` (associated values); `SystemOptions`
      (kij/aij/alpha/vl/delta/ge_model; empty = unused sentinel); methods
      `flash_tp`, `bubble_p/t`, `dew_p/t`, `k_values`
- [x] `VleFfiError` (`NotFound`/`InvalidInput`/`Flash`/`Steam`) with the same
      error-classification policy as the Python bindings
- [x] 15 Rust tests (`cargo test -p vle-ffi`) + 10 XCTests through the real
      boundary (IF97 Table 5 point, 1-atm boiling, Ch. IV flash config,
      bubble/dew bracketing, error mapping) — the FFI analog of the M5+
      PyO3-parity rule

### 15.3 Documentation (~3–5h) — **done**
- [x] `docs/en/ios/README.md` learning guide (C ABI, lift/lower, .a vs
      XCFramework anatomy, device-vs-sim arm64, modulemap gotcha, why
      binaries stay out of git, troubleshooting table)
- [x] README (Swift channel + project tree), `.gitignore`
      (`*.xcframework`, `*.generated.swift`, `.build/`), deploy/README note,
      ROADMAP/TODO/MODERNIZATION_PLAN sync as M15/Phase 22
- [ ] Future (separate repo): `vle-ios` SwiftUI Multiplatform app consuming
      `swift/VleThermo` as a local package

---

## Milestone 16: Android/Kotlin FFI — `vle-ffi` → Kotlin via UniFFI
*Phase 23 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Fable 5*

Local-only builds again (no CI, no committed binaries — see
[ANDROID_FFI_PLAN.md](ANDROID_FFI_PLAN.md), incl. the framework decision
log and the parked C#/.NET route). No release; no milestone notebook
(Kotlin isn't executable from Jupyter).

### 16.1 Kotlin pipeline on the existing wrapper (~2–3h) — **done**
- [x] `ffi/` `crate-type` += `"cdylib"` (JNA loads shared libs); no new
      FFI surface — M15's API is the API
- [x] `ffi/uniffi-bindgen/` second bin `uniffi-bindgen`
      (`uniffi::uniffi_bindgen_main()`, general CLI) + `ffi/uniffi.toml`
      `[bindings.kotlin]` (package `dev.migueljackson.vle.ffi`; no
      `android = true` so one binding serves Android + desktop JVM)
- [x] `scripts/build-android.sh`: cargo-ndk (arm64-v8a + x86_64, `ABIS=`
      override) → host lib → library-mode Kotlin bindgen → module drop-in
      → optional host-JVM tests; host-verified end-to-end on this Mac
      (cdylib builds, bindgen emits `vle_ffi.kt`)

### 16.2 Gradle module + smoke tests (~2–4h) — **done (code); Studio run pending**
- [x] `kotlin/` standalone Gradle build (no wrapper committed) +
      `kotlin/VleThermo/` `com.android.library` module: AGP 8.7 /
      Kotlin 2.1 / compileSdk 35 / minSdk 24; JNA 5.17 `@aar` +
      test-scope desktop jar; `jna.library.path` → `target/release/`
- [x] 5 committed smoke tests through the real JNA boundary
      (`VleThermoSmokeTest.kt`): version, water lookup (critical point in
      canonical units), IF97 Psat(373.15 K), Ch. IV heptane/butane RKS
      flash, `InvalidInput` error mapping
- [ ] First Android Studio verification on the dev machine (open
      `kotlin/`, run tests; then emulator via the app repo)

### 16.3 Documentation (~2–3h) — **done**
- [x] `ANDROID_FFI_PLAN.md` design record (decision log: Kotlin/Compose
      over MAUI/Avalonia; WSA dead; Compose Multiplatform = Windows path)
- [x] `docs/en/android/README.md` learning guide (pipeline diagram,
      prerequisites, consume-from-Android-Studio, Windows DLL leg,
      troubleshooting table)
- [x] `docs/en/dotnet/README.md` — C#/.NET route documented and parked:
      uniffi-bindgen-cs targets 0.31, workspace pins 0.32, no downgrade
      planned (dated 2026-07-12)
- [x] README (Kotlin channel + tree), CLAUDE.md (build chain),
      deploy/README (channel row), `.gitignore` (generated bindings,
      jniLibs, Gradle wrapper), ROADMAP/TODO/MODERNIZATION_PLAN sync as
      M16/Phase 23
- [ ] Future (separate repo): Compose Multiplatform app (Android +
      Windows desktop) consuming `kotlin/VleThermo` by path

---

## Milestone 17: Web/JavaScript FFI — `vle-wasm` → the browser via wasm-bindgen
*Phase 24 of MODERNIZATION_PLAN.md*
*Executed by Claude Code using Claude Fable 5*

Local-only builds again (no CI, no committed binaries, nothing on npm —
see [WEB_UI_PLAN.md](WEB_UI_PLAN.md), incl. the framework decision log,
the verified feasibility spike, and the single-threaded/rayon
decomposition). No release; no milestone notebook (JS isn't executable
from Jupyter).

### 17.1 The wasm wrapper crate (~2–3h) — **done**
- [x] `wasm/` (`vle-wasm`, `publish = false`, cdylib+rlib) — sibling of
      `ffi/` (UniFFI has no JS backend at our pin; wasm-bindgen is the
      standard); engine with `component-db` + `steam`, never `python`;
      workspace member
- [x] M15/M16's API in JS form: `version`, DB lookups + custom components
      (plain camelCase objects via serde-wasm-bindgen), steam tables,
      `VleSystem` (`flashTp`, `bubbleP/T`, `dewP/T`, `kValues`);
      compositions as `Float64Array`; model names as forgiving strings
      (`"RKS1972"`, `"van-laar"`) or tagged objects; errors thrown as JS
      `Error`s with the family's message prefixes; JsValue confined to
      thin shims over a host-testable `SystemCore`
- [x] `scripts/build-wasm.sh`: preflight → Node smoke tests →
      `wasm-pack build --target web --release` → `wasm/pkg/` (~360 KB
      wasm / ~150 KB gzipped)

### 17.2 Verification (~1h) — **done**
- [x] 19 host-side unit tests (`cargo test -p vle-wasm`): parsing,
      validation, Ch. IV flash vs direct engine call, bubble/dew
      consistency
- [x] 5 smoke tests through the real JS↔wasm boundary
      (`wasm-pack test --node wasm`): version, water lookup, IF97 1-atm
      boiling row, Ch. IV Table 4.10 flash (β in the thesis band), error
      mapping (`invalid input:` / `component not found` prefixes)
- [x] Package-level sanity: `wasm/pkg` imported in Node as a consumer
      would (init → flash → bubbleP → thrown-error check)

### 17.3 Documentation (~2h) — **done**
- [x] `WEB_UI_PLAN.md` design record adopted (decision log: React+wasm
      over Flutter/React Native; Kotlin/Compose kept as native escape
      hatch; shells = packaging, deferred to the app repo)
- [x] `docs/en/web/README.md` learning guide (wasm-bindgen theory, React
      quickstart with units, Web Worker pattern, plotly.js 3-D surfaces,
      Tauri/Electron/Capacitor/PWA notes, troubleshooting)
- [x] README (JS channel + tree + status), CLAUDE.md (build chain),
      deploy/README (channel row + WebAssembly section), `.gitignore`
      (`wasm/pkg/`, `node_modules/`), ROADMAP/TODO/MODERNIZATION_PLAN
      sync as M17/Phase 24
- [ ] Future (separate repo): React app — website + Tauri/Electron/
      Capacitor shells — consuming `wasm/pkg` by path

---

## Performance Track: External Audit Response
*No new MODERNIZATION_PLAN phase — refines Phase 15 (§F/§I/§J) and the mixture core in place*
*Executed by Claude Code using Claude Opus 5 (1M context)*

Plan of record: [OPTIMIZATION_PLAN_PART1.md](OPTIMIZATION_PLAN_PART1.md) (flash layer) and
[OPTIMIZATION_PLAN_PART2.md](OPTIMIZATION_PLAN_PART2.md) (mixture core). Provenance and
lessons: [OPTIMIZATION_AUDIT_HISTORY.md](OPTIMIZATION_AUDIT_HISTORY.md).

### Benchmark foundation (~3–4h)
- [x] `flash_multi` criterion group — inner RR solve, one K-value evaluation, the whole
      driver and the TPD stability test measured **separately** at n = 2, 4, 6, 8
- [x] `mixture` criterion group — `mixture_params` vs `z_mix` vs `ln_phi_mix`, plus the
      composition-Jacobian, activity and virial paths
- [x] Baseline captured; audit's headline confirmed — Rachford-Rice is 1–15 % of the flash,
      K-value thermodynamics ~70 %, and `mixture_params` is 40–57 % of `ln_phi_mix`

### Part 1 — flash layer (~7–10h)
- [x] §1/§2 — `FlashWorkspace` (private), `ln_phi_mix_into`, `ln_k_values_into`,
      `ln_poynting_factor`, `wilson_ln_k`; the flash iterates on `ln K` end-to-end
- [x] §3 — RR safeguards: boundary-only validation, absent/degenerate component filtering
      in the pole bracket, scale-aware Halley acceptance, scale-safe pole nudge,
      bracket-width stop criterion, Brent bracket-halving
- [x] §5 — GDEM trust region (μ < 0.95, gain ≤ 4, |ln K| ≤ 80, trial buffer, retrospective
      residual-decrease rollback)
- [x] §7/§8 — one shared mixture state for both cubic roots, `TrialWorkspace`, explicit
      `zᵢ ≤ 0` rejection
- [x] **Benchmarked and reverted**: precomputing `cᵢ`/`zᵢcᵢ` (+30…+200 %) and the
      f(0)/f(1) bracket probe (+25…+200 %). Reasoning preserved in-code at both sites
- [x] Rejected — §4 (SIMD/unrolling): wrong `n` regime for this engine
- [x] Corrected `MODERNIZATION_PLAN.md`'s false claim that the §J Newton finish shipped

### Part 2 — mixture core (~8–12h)
- [x] §1 — `mixture::TpCache` (public) + `flash::SystemTpCache` (crate-internal):
      pure-component EOS parameters, γ-φ `Psat`/`φˢᵃᵗ`/Poynting constants and the virial
      `Bᵢⱼ` built once per (T, P) instead of twice per outer iteration
- [x] §5 — `activity::ActivityTpCache` (flat Wilson Λ, NRTL τ/G/τG); NRTL drops from an
      O(n³)-`exp` per-component path to O(n²)
- [x] §9 — flat, single-pass virial `Bᵢⱼ`
- [x] §4 — √Aᵢ/√Bᵢ hoisted, built only for the rules that use them
- [x] **The finding the audit missed** — `quad_a` took its cross-parameter closure as
      `&dyn Fn`, i.e. n² *indirect* calls per mixture evaluation; monomorphizing the loop
      is what paid, not the square-root hoist
- [x] Rejected with reasons — §2 (`Component` SoA), §3 for `kij`, §8 (Broyden has **no
      production callers**), §11, §12 as written, §13's `unsafe`

### Verification (~1–2h)
- [x] 291 Rust tests (12 new across both parts) + 450 Python tests green;
      Chapter IV Tables 4.10 / 4.11–4.12 intact
- [x] `cargo fmt --check` clean, `cargo +1.97.0 clippy --workspace --all-targets` 0 warnings
- [x] Python wheel rebuilt (`maturin develop --release`); `flash_pt` mass balance 5.6e-17,
      `flash_pt_batch` 200 000 points at 0.29 µs/pt, NRTL γ-φ flash verified end to end

### Deferred (each its own change)
- [ ] Audit P2 §6 — const-generic dual for composition derivatives (Wong-Sandler is 5.1×
      the analytic path; largest remaining algorithmic win, highest accuracy risk)
- [ ] Audit P2 §7 (`MixtureScratch`) and §10 (Rayon SoA batch output — profile first)
- [ ] Audit P1 §6 — the §J Newton finish, as its own milestone with its own validation
- [ ] Audit P1 §7 — multi-seed stability (a correctness change, not a perf one)
- [ ] Audit P1 §9 — analytic envelope Jacobian (benchmark the envelope first)

---

## Summary

| Milestone | Est. Total | Status |
|-----------|-----------|--------|
| 0. Foundation | — | Done |
| 1. Documentation & Translation | ~20–28h | **Done** |
| 2. Dev Environment & Scaffolding | ~9–12h | **Done** |
| 3. Units Library | ~19–26h | **Done** |
| 4. Component Database | ~12–15h | **Done** |
| 5. CI/CD + Auto-Deploy | ~16–22h | **Done** |
| 6. Numerics | ~16–20h | **Done** |
| 7. Pure Component Models | ~28–37h | **Done** (7.1–7.4 shipped; v0.3.0–v0.6.0) |
| 8. Mixture Models + Performance Foundation | ~41–57h | **Done** (8.1 in v0.7.0; 8.2–8.4 in v0.8.0) |
| 9. Flash & Regression | ~44–62h | **Done** (all algorithms + bindings + tests + Ch. IV validation + notebooks 04–09; shipped in v0.8.0) |
| 10. Python Bindings, Wrapper & Batch API | ~25–36h | **Done** (System wrapper + batch API + component DB + plots + tests + intro notebook; external thermo/CoolProp bench deferred; shipped in v0.8.0) |
| 11. Chapter IV Walkthrough | ~5–8h | **Done** (walkthrough notebook 10 + all 15 notebooks re-verified + catalogue complete; shipped in v0.8.0) |
| 12. Downstream Derivative & Database Release | ~25–38h | **Done** — M12.1 (v0.8.2: 24-compound DB + Cp), M12.2 (Rust-side `component-db` DB), M12.3 (T/P derivatives of fugacity + K), M12.4 (real Cp + partial molar H + γ-φ enthalpy), M12.5 (notebook 11 + benches); shipped as **v0.9.0**, plus **v0.9.1** patch (WS departure-enthalpy fix) |
| 13. Steam Tables — `vle-steam` (IAPWS-IF97) | ~27–39h | **Shipped as v0.10.0** — 13.1–13.6 done (crate, all 5 regions + saturation + backward eqs, verified vs R7-97 tables; PyO3 `vle.steam` + batch numpy; notebook 12; README + benches). Signed tag pushed + published |
| 14. NRTL Activity Model + Ammonia | ~14–20h | **Shipped as v0.11.0** — NRTL model (general multicomponent, analytic Hᴱ via `num-dual`), `alpha` matrix threaded, PyO3 + Python wrapper, ammonia in the 25-compound DB, milestone notebook 13. Rigorous NH₃–H₂O param regression deferred (qualitative demo shipped) |
| 15. iOS/macOS FFI (`vle-ffi` via UniFFI) | ~13–21h | **Done (unreleased)** — `ffi/` wrapper crate + bindgen bin, `scripts/build-ios.sh` (3 Apple targets → XCFramework), `swift/VleThermo` package (10 XCTests green), learning doc `docs/en/ios/`. Local-build artifact only; app itself is a future separate repo |
| 16. Android/Kotlin FFI (`vle-ffi` via UniFFI) | ~6–10h | **Code complete** — `cdylib` + general bindgen bin + `[bindings.kotlin]`, `scripts/build-android.sh` (cargo-ndk ABIs + host lib → Kotlin bindgen), `kotlin/VleThermo` Gradle module (5 smoke tests), docs `docs/en/android/` + parked `docs/en/dotnet/`. First Android Studio run pending; app (Android + Compose-Desktop Windows) is a future separate repo |
| 17. Web/JavaScript FFI (`vle-wasm` via wasm-bindgen) | ~5–8h | **Done** — `wasm/` wrapper crate (plain-object records, Float64Array comps, JS-Error mapping), `scripts/build-wasm.sh` → `wasm/pkg` npm package (~150 KB gzipped), 19 host + 5 boundary tests green, guide `docs/en/web/`. React app + shells (Tauri/Electron/Capacitor) are a future separate repo |
| Performance Track: External Audit Response | ~18–26h | **Parts 1 & 2 done** — `flash_multi` + `mixture` bench groups and a measured baseline; Part 1 (flash layer): `FlashWorkspace`, `*_into` kernels, log-form K end-to-end, RR safeguards, trust-region GDEM, shared-mixture-state min-Gibbs. Part 2 (mixture core): `mixture::TpCache` + `flash::SystemTpCache`, `ActivityTpCache`, flat single-pass virial, `quad_a` devirtualized. Cumulative: flash **−24…−28 %**, stability **−44…−51 %**, NRTL γ **−68 %**, virial **−21 %**. Four audit recommendations benchmarked and **rejected as regressions or dead code**. Deferred: audit P2 §6/§7/§10 + P1 §J Newton finish, §7 multi-seed stability, §9 envelope Jacobian — see `OPTIMIZATION_PLAN_PART1.md` / `OPTIMIZATION_PLAN_PART2.md` |
| **Total** | **~345–488h** | |

Each active milestone's total now includes: milestone notebook (~2–4h) + notebook-catalogue update (~0.3h). Deploying to the hosted hub is a separate operator-side step in a private operator repository, not counted here.
