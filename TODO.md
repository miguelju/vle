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
- [x] Create modernization plan with 17 implementation phases
- [x] Map algorithms to 22 academic references (ACS format)
- [x] Propose 8 algorithm performance improvements (A–H)
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
- [x] **`.github/workflows/_build.yml`** (~2h) — reusable cibuildwheel matrix: Linux x64 self-hosted ephemeral, Linux arm64 hosted (`ubuntu-24.04-arm`), macOS arm64 self-hosted, Windows hosted, CPython 3.10+ abi3
- [x] **`.github/workflows/ci.yml`** (~1h) — push/PR/dispatch: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, call `_build.yml` (artifact-only); fork-PR guard on every self-hosted job
- [x] **`[tool.cibuildwheel]` block in `python/pyproject.toml`** (~0.5h) — abi3, manylinux_2_28, `pytest {package}/tests`, skip pp + musllinux
- [x] **First `#[pymodule]` in engine/** (~3–4h) — add `pyo3` `abi3-py310` feature to `engine/Cargo.toml`; gate enums in `engine/src/types.rs` with `#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]`; create `engine/src/py_bindings.rs` with `#[pymodule] fn _engine(...)` exposing `version()` + the four enums; add `python/tests/test_engine.py`
- [x] **`docs/ci.md`** (~1h) — developer overview, ephemerality table, fork-PR guard, retry flow, badges
- [x] **`docs/runners/linux-setup.md`** (~1.5h) — Proxmox LXC + Docker + `myoung34/github-runner:latest` ephemeral container; PAT setup; verification; scaling
- [x] **`docs/runners/macos-setup.md`** (~1.5h) — Mac mini M1 launchd service; toolchain bootstrap (Xcode CLT, rustup, four Pythons via python.org or `uv`, maturin); periodic-cleanup checklist
- [x] **`.github/workflows/release.yml`** (~3h) — `v*` tag: call `_build.yml`, then `publish-pypi` (Trusted Publishing OIDC), `publish-crates` (1Password-loaded token; `vle-units` then `vle-thermo`), `gh-release` (wheels + sdist attached). *(M5 also shipped an auto-deploy job; it was later removed when the deployment moved to `homelab-iac` — see MODERNIZATION_PLAN.md.)*
- [x] **Drop `git pull` from `deploy/scripts/deploy.sh`** (~0.3h) — tag-checkout happens in the deploy wrapper; deploy.sh becomes pure docker build + up
- [x] **Private auto-deploy installer** (`deploy/local/auto-deploy/{vle-deploy, install-rocky.sh, install-oracle.sh, README.md}`) (~2h) — `/usr/local/bin/vle-deploy` wrapper with tag-regex validation; one-shot installers add the `command="..."` restriction to `~/.ssh/authorized_keys`; fail2ban on rocky
- [x] **PUBLISHING.md rewrite** (~0.5h) — drop GHCR section; add "Cutting a release" subsection with the tag-push flow
- [x] **`deploy/FAILOVER.md`** (~0.3h) — replace `deploy.sh` references that assumed git-pull
- [x] **End-to-end smoke test** (~1h → actual ~4h across v0.1.0–v0.1.2) — pushed `v0.1.0` 2026-05-22; packages landed on all three registries; sandbox redeploy hit three latent Dockerfile bugs (arm64 hardcode, missing workspace root, wheel-glob mismatch) that required v0.1.1 + v0.1.2 patch tags to fully resolve. v0.1.2 ran fully green end-to-end: all 5 build jobs, publish-pypi (env approval), publish-crates, gh-release, deploy-sandbox (rocky + Oracle containers restarted). `pip install vle-thermo==0.1.2 && vle._engine.version() == "0.1.2"` verified.
- [x] **Manual config outside the repo** (~2h, one-time) — 1Password vault + Service Account; PyPI Trusted Publisher; rocky + Oracle SSH installers; runners (`vle-runner-01` Linux LXC + `vle-mac-01` Mac mini) registered and online

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

## Milestone 8: Mixture Models

Like Milestone 7, this milestone is split into sub-milestones (8.1–8.3), each
independently shippable with its own tests, notebook contribution, and version
bump. The single planned notebook (`03_activity_models.ipynb`) ships with 8.1.

### Milestone 8.1 — Activity Models + Liquid Volume (complete)
*Executed by Claude Code using Claude Opus 4.8 (1M context)*

- [x] **Implement 5 activity models** (~4–6h) — Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand in `engine/src/activity.rs`, formulas from research-paper Table 2.3; each with analytical excess enthalpy (Wilson closed-form; the legacy `Hᴱ = Gᴱ` convention for Margules/van Laar/Scatchard) (4)
- [x] **Implement liquid volume models** (~1–2h) — Rackett (Spencer-Danner) + Thomson/COSTALD (18) in `engine/src/liquid_volume.rs`
- [x] **PyO3 bindings** — `liquid_molar_volume`, `activity_ln_gamma`, `activity_excess_{gibbs,enthalpy,entropy}`, and the `VolumeModel` enum (M5+ binding rule)
- [x] **Rust + Python tests** — closed-form checks vs Table 2.3, a numerical-derivative oracle for the analytical Wilson Hᴱ, Gibbs-Duhem consistency
- [x] **Create milestone notebook** (~2–3h) — `notebooks/03_activity_models.ipynb` per CLAUDE.md *Notebook Conventions*: Chapter II §2.2 snippets, gamma vs. composition plots, excess Gibbs energy, ≥2 user exercises
- [x] **Update the notebook catalogue** (~0.3h) — add to `deploy/NOTEBOOKS.md` + version bump

### Milestone 8.2 — Mixing Rules + Multicomponent Fugacity (not started)

- [ ] **Implement 8+ mixing rules** (~6–8h) — IVDW, IIVDW, WS (21), HOV, HVS, MHV1, MHV2, Clasica_I, plus Schmidt-Wenzel/Patel-Teja C-parameter mixing (4)
- [ ] **Implement multicomponent fugacity** (~4–6h) — partial fugacity coefficients for all mixing rules (9), 3-param EOS (4), Chao-Seader multicomp (4)

### Milestone 8.3 — Mixture Energy Properties + Validation (not started)

- [ ] **Implement enthalpy/entropy** (~3–4h) — ideal Cp integration, departure functions (9), condensation enthalpy (4), reference state handling
- [ ] **Write mixture model tests** (~3–4h) — compare against VB6/Pascal outputs — validation tests pass
- [ ] **Refresh the hosted hub** (~0.3h) — operator-side: run `deploy-vle` in `homelab-iac`, verify the new notebook

## Milestone 9: Flash & Regression

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
- [ ] **Update the notebook catalogue** (~0.3h) — add the new notebooks to `deploy/NOTEBOOKS.md`; touch `deploy/README.md` only if a distribution channel changed
- [ ] **Refresh the hosted hub** (~0.3h) — operator-side: run `deploy-vle` in `homelab-iac`, verify each new notebook

## Milestone 10: Python Bindings & Wrapper

- [ ] **Create PyO3 bindings** (~4–6h) — expose core types as `#[pyclass]`, calculation functions as `#[pyfunction]`, `VleEngine` class
- [ ] **Build Python `System` class** (~3–4h) — high-level API: `system.bubble_point_T()`, `system.flash_isothermal()`, etc.
- [ ] **Create result dataclasses** (~1–2h) — FlashResult, BubbleResult, DewResult with fields matching thesis notation
- [ ] **Build component database** (~2–3h) — `notebooks/data/components.json` with common substances (Tc, Pc, w, Cp coefficients, etc.)
- [ ] **Build plotting helpers** (~2–3h) — Pxy, Txy, phase envelope diagrams via matplotlib
- [ ] **Write Python test suite** (~2–3h) — `test_validation.py` reproducing all Chapter IV results — validation tests pass
- [ ] **Write installation guide** (~1h) — end-user: `pip install`, basic usage example
- [ ] **Create milestone notebook** (~2–3h) — `notebooks/01_introduction.ipynb` per CLAUDE.md *Notebook Conventions*: Chapter I + Appendix B snippets, `vle.System` API tour, first flash calculation end-to-end, ≥2 user exercises
- [ ] **Update the notebook catalogue** (~0.3h) — add to `deploy/NOTEBOOKS.md`; touch `deploy/README.md` only if a distribution channel changed
- [ ] **Refresh the hosted hub** (~0.3h) — operator-side: run `deploy-vle` in `homelab-iac`, verify the new notebook

## Milestone 11: Chapter IV Walkthrough & Final Deployment

Notebooks 01–08 ship incrementally through Milestones 4–10. This milestone is the capstone: one new walkthrough notebook covering all Chapter IV results, plus a final operator-side hub refresh of every notebook.

- [ ] **Re-run all prior milestone notebooks** (~1–2h) — fresh kernel, Run All, verify no cell errors — validation pass
- [ ] **Create `notebooks/09_chapter4_validation_walkthrough.ipynb`** (~4–6h) — per CLAUDE.md *Notebook Conventions*: narrated end-to-end walkthrough of [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md) §4.1–§4.7, running the library against every Table 4.1–4.12 and reporting % error vs. published values, ≥2 user exercises
- [ ] **Update the notebook catalogue** (~0.3h) — `deploy/NOTEBOOKS.md` catalogue marked complete
- [ ] **Final hub refresh** (~0.5h) — operator-side: run `deploy-vle` (mode=full) in `homelab-iac`, verify every notebook in the catalogue opens and Run-All succeeds on the hub

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
| 8. Mixture Models | ~25–35h | In progress (8.1 complete; 8.2–8.3 not started) |
| 9. Flash & Regression | ~38–52h | Not started |
| 10. Python Bindings & Wrapper | ~19–27h | Not started |
| 11. Ch. IV Walkthrough & Final Deploy | ~7–11h | Not started |
| **Total** | **~209–285h** | |

Each active milestone's total now includes: milestone notebook (~2–4h) + notebook-catalogue update (~0.3h). Deploying to the hosted hub is a separate operator-side step in the `homelab-iac` repo, not counted here.
