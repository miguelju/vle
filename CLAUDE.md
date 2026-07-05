# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Release & Push Rules

**Before every `git push` or major release**, you MUST review and update all documentation to reflect current state:

1. **README.md** — Features list, project structure tree, time estimates, status
2. **ROADMAP.md** — Check off completed milestones, update in-progress items
3. **TODO.md** — Check off completed tasks, update time estimates and summary table
4. **MODERNIZATION_PLAN.md** — Update if architecture or phases changed
4b. **PERFORMANCE_PROPOSAL.md** — Update if a performance track (A–E) decision changed
5. **CLAUDE.md** — Update if new conventions, paths, or tools were introduced
6. **PASCAL_VB6_COMPARISON.md** — Update if new legacy code analysis was done
7. **docs/en/research-paper/** — Update if translations were completed or links changed
8. **deploy/README.md** and **deploy/NOTEBOOKS.md** — Update if any generic install step, env var, or prerequisite changed
9. **deploy/.env.example** — Update if any new environment variable was added (with a safe example value, never a real one)
10. **`python/README.md`** (the **PyPI** long-description for [`vle-thermo`](https://pypi.org/project/vle-thermo/)) — Update whenever the change affects the Python-facing story: new/removed public API (`vle.System` methods, the `_batch` numpy API, CLI commands), status/version language, feature list, or install steps. Every code snippet must run verbatim against the current wheel — execute it (`~/miniconda3/envs/vle/bin/python`) before committing.
11. **`engine/README.md`** (the **crates.io** page for [`vle-thermo`](https://crates.io/crates/vle-thermo), via `readme = "README.md"` in `engine/Cargo.toml`) — Update whenever the change affects the Rust-crate story: public API/type names, EOS/model coverage, `WIP` markers, status/version language, or the `vle-thermo = "0.x"` install snippet. Any `rust` code block must compile — verify it (`cargo run --example <tmp>` or a doctest) before committing.

**Package-page docs are immutable per published version.** The PyPI and crates.io pages render the README **bundled with each release** — editing `python/README.md` / `engine/README.md` does **not** refresh the live page. The page only updates when a **new version is published** (bump `[workspace.package] version` in the root `Cargo.toml` + `version` in `python/pyproject.toml`, then tag `v<x.y.z>` → `release.yml`). So a doc-only fix to either README still requires a patch release to become visible; batch such fixes into the next release rather than tagging a release solely for a typo, unless the staleness is materially misleading (e.g. "under active development" for shipped features). The two package READMEs, the root `README.md`, `engine/Cargo.toml` `description`, and `python/pyproject.toml` `description` must tell a mutually consistent version/status story.

Do NOT push until all documentation accurately reflects the current state of the code. When in doubt, read each file and verify.

**Pre-push private-data gate** (run from repo root before every push):

```sh
# Anything matching this pattern in a to-be-pushed file is a leak.
git diff --cached origin/main -- ':!deploy/local' ':!deploy/.env' \
  | grep -E '(163\.192\.214\.135|cloudflareaccess\.com|BEGIN (RSA |EC )?PRIVATE KEY)' \
  && { echo "ABORT: private infrastructure detail found in staged changes"; exit 1; } \
  || echo "clean: no private details in diff"
```

If the grep hits anything, stop and move the offending content under `deploy/local/` or replace it with an `${ENV_VAR}` / `example.com` placeholder before pushing.

**Pre-push formatting gate (`hooks/pre-push`)**: the repo ships a versioned git
pre-push hook that runs `cargo fmt --check` and **blocks the push on any diff** —
mirroring the first step of the CI `lint (fmt + clippy)` job, where a fmt failure
also skips clippy. Activate it once per clone with `git config core.hooksPath hooks`
(it's a local setting, not committed). clippy is intentionally *not* in the hook:
the local rustc trips a pyo3 `E0133` across every `#[pyfunction]` (a toolchain/pyo3
mismatch, not our code), so clippy is enforced only in CI. **Always run
`cargo fmt --check` before pushing** even if the hook is active — never `--no-verify`.

Note: addresses on `migueljackson.dev` (e.g. `admin@migueljackson.dev`, `git@migueljackson.dev`) are Miguel's intentional public / professional identity and are safe to include in committed files — `Cargo.toml` / `pyproject.toml` `authors`, git commit author fields, READMEs, etc. They are **not** covered by this gate. The JupyterHub deployment now lives in a separate private operator repository, so no deployment hostnames, IPs, or Cloudflare Access team names should appear in this repo's committed files at all — use `example.com` / `${VAR}` placeholders (see the rule list below).

## Phase / Milestone Synchronization Rules

The project's work is described at three levels of detail that MUST stay in sync:

- **`ROADMAP.md`** — milestones (goals, high-level deliverables)
- **`TODO.md`** — tasks grouped by milestone (with time estimates)
- **`MODERNIZATION_PLAN.md`** — implementation phases (technical detail per phase)

**Invariants that MUST hold at all times:**

1. **Phase numbering in `MODERNIZATION_PLAN.md` follows milestone execution order in `ROADMAP.md` / `TODO.md`.** Phase 1 is the first-executed milestone's work; the last phase is the last-executed milestone's work. Do not number phases by what was drafted first — number them by execution order.
2. **Every milestone in `ROADMAP.md` maps to one or more phases in `MODERNIZATION_PLAN.md`, and vice versa.** No milestone without a phase, no phase without a milestone.
3. **Each phase in `MODERNIZATION_PLAN.md` carries a `*(Milestone N)*` tag** in its header.
4. **Each milestone in `ROADMAP.md` carries a `*Phase N of MODERNIZATION_PLAN.md*` (or `*Phases N–M of MODERNIZATION_PLAN.md*`) pointer** under its header.

**When you add, remove, rename, or reorder a milestone or a phase, you MUST in the same commit:**

- Renumber phases in `MODERNIZATION_PLAN.md` end-to-end so execution order is preserved.
- Update every `*Phase N of MODERNIZATION_PLAN.md*` pointer in `ROADMAP.md`.
- Update every `*(Milestone N)*` tag in `MODERNIZATION_PLAN.md`.
- Update the phase count mentioned in `ROADMAP.md`'s Milestone 0 ("Create modernization plan with N implementation phases").
- Grep for `Phase \d+` across all `*.md` files to catch stray references.

After any such change, re-read all three files and verify the invariants above before committing.

## Milestone Tracking Rules

**When completing a milestone**, you MUST record the LLM model used in:
1. **ROADMAP.md** — Add `*Executed by Claude Code using <model name and version>*` under the milestone header
2. **TODO.md** — Add the same line under the milestone section header
3. **Git commit message** — Include the `Co-Authored-By` trailer with the model name

The model name must be the exact model powering the session (e.g., `Claude Opus 4.6 (1M context)`, `Claude Sonnet 4.6`, etc.). This tracks which AI model was used for each phase of the project.

## Units Documentation Rules

**Every Rust function and Python wrapper function** that accepts or returns a physical quantity MUST state the units in its doc comment.

**Rust example:**
```rust
/// Calculate saturation pressure at the given temperature.
///
/// # Arguments
/// * `temperature` — Temperature in **K** (Kelvin)
///
/// # Returns
/// Saturation pressure in **kPa**
pub fn saturation_pressure(temperature: f64) -> f64 { ... }
```

**Python example:**
```python
def saturation_pressure(temperature: float) -> float:
    """Calculate saturation pressure at the given temperature.

    Args:
        temperature: Temperature in **K** (Kelvin)

    Returns:
        Saturation pressure in **kPa**
    """
```

**Canonical internal units** (used by the VLE engine, matching legacy VB6/Pascal code):
- Temperature: **K** (absolute)
- Pressure: **kPa** (absolute — **never** gauge)
- Energy (molar): **kJ/kmol**
- Entropy (molar): **kJ/(kmol·K)**
- Volume (molar): **cm³/mol**
- Amount: **kmol**
- Gas constant R: **8.31451 kJ/(kmol·K)**

**Absolute vs. Gauge Pressure**: All VLE calculations use **absolute pressure**. Gauge pressure (barg, psig, kPag) is converted to absolute at the API boundary via `P_abs = P_gauge × scale + P_atm`. Atmospheric pressure (P_atm) is a **runtime-configurable parameter** stored in the unit registry — it must **never** be hardcoded. Default: 101.325 kPa (1 standard atm). Users change it via `registry.set_atmospheric_pressure()` (Rust) or `set_atmospheric_pressure()` (Python). When writing functions that accept pressure, always document whether the parameter expects absolute or gauge units. Engine-internal functions always take absolute kPa. See [`docs/en/units/dimensional-analysis.md`](docs/en/units/dimensional-analysis.md) §3.4 for the full explanation.

User-facing APIs should accept unit strings (e.g., `"25 degC"`, `"1 atm"`, `"3.5 barg"`) via the `units` library (see `units/` crate and `python/src/vle/units.py`).

**User-extensible units**: The library ships with VLE defaults (including gauge pressure units: barg, psig, kPag) but must remain **extensible**. Users can add custom units (e.g., `mmH2O`, `atg`) via the runtime `UnitRegistry` in Rust or `ureg.define()` in Python, without modifying library source. When adding new code, do not hard-code the list of accepted units — always go through the registry. See [`docs/en/units/dimensional-analysis.md`](docs/en/units/dimensional-analysis.md) §7 for the extension API and rules.

## Project Overview

This is a **VLE (Vapor-Liquid Equilibrium) thermodynamic calculator** being modernized from two legacy codebases into a Rust + Python stack:

- **`legacy/vb6/`** — Production VB6 COM/DLL (~15,000 lines, 27 class modules + 2 BAS modules) from the thesis: *"Desarrollo de un Programa Computacional para el Cálculo del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente Windows"* (Jackson & Mendible, USB, 1999). The primary source for most EOS variants, mixing rules, virial equation, and flash algorithms.
- **`legacy/pascal/`** — Mac Pascal program from Reference (4): Da Silva & Báez (1989). 6 units (TERMOI–TERMOVI, ~2,500 lines). Contributes Schmidt-Wenzel, Patel-Teja, Chao-Seader EOS, Antoine vapor pressure, and Aij regression with analytical Jacobians. **All code derived from this source must cite (4).**
- **`docs/es/`** — Original research paper and program documentation in Spanish. **`docs/en/`** — English translations (navigatable, with cross-linked references).

The target architecture is documented in `MODERNIZATION_PLAN.md` and the merge strategy in `PASCAL_VB6_COMPARISON.md`. The navigatable English research paper is at `docs/en/research-paper/README.md`.

## Target Architecture

```
engine/     — Rust crate (core computation), PyO3 bindings via maturin
python/     — Python wrapper package (high-level API, plotting, component DB)
notebooks/  — Jupyter notebooks reproducing research paper results
docs/       — English translations and parameter reference
```

**Build chain:** Rust (engine/) -> PyO3/maturin -> Python native module -> Python wrapper (python/) -> Jupyter notebooks

## Python Environment (conda `vle` env — mandatory)

**All Python work in this repo goes through the dedicated `vle` conda environment. Never invoke a bare `python`/`python3`/`pip`/`pytest`, and never create a `.venv` in the repo.** Use the env's binaries directly by absolute path (more reliable from non-interactive shells than `conda activate`):

- `~/miniconda3/envs/vle/bin/python` — running any Python script or one-liner
- `~/miniconda3/envs/vle/bin/pytest` — running `python/tests/`
- `~/miniconda3/envs/vle/bin/maturin` — building/installing the PyO3 wheel (`maturin develop` from `python/`)
- `~/miniconda3/envs/vle/bin/jupyter` — executing notebooks (`nbconvert --execute`)

If the env is ever missing, recreate it with conda (`conda create -n vle python=3.12` + `pip install maturin pytest` + the deps in `python/pyproject.toml`) rather than falling back to the system Python. Note: `conda env list` from a non-interactive shell sometimes misses envs — check `ls ~/miniconda3/envs/` directly.

## Key Technical Decisions

- **Rust enums + match** map directly to VB6's `Select Case` dispatch over 22+ EOS variants, 5 activity models, 8 mixing rules
- **nalgebra** replaces hand-rolled Gauss elimination (**ndarray** is dropped until the batch API needs it)
- Implement analytical derivatives for ALL variants (not just the 5 Pascal models) for dα/dT and dGE/dT. Numerical versions retained only as test oracles. See "Algorithm Choices" below.
- All 5 activity models and 6 flash calculation types are identical in both programs — single implementation each
- Pascal's 3-parameter EOS (Schmidt-Wenzel, Patel-Teja) require special C-parameter mixing rules not needed by 2-parameter EOS
- **Performance plan (2026-07, PERFORMANCE_PROPOSAL.md)**: the engine stays in Rust (language question settled); mixture code is written once against the generalized (A, B, U, W) form; composition derivatives are exact (analytic for classical mixing, **`num-dual`** dual-number AD for Wong-Sandler/MHV — never finite differences except as test oracles); the Python surface gains a batch numpy API (**rust-numpy** + **rayon**, GIL released in batch kernels). Hot-path rules: no heap allocation inside iteration loops, T-dependent quantities computed once per state via an `EosState` cache, criterion benches guard regressions.

## Reference Citation Requirements

This project is based on academic research. Code derived from legacy sources must cite the originating reference:

- **Pascal-derived code** (`legacy/pascal/`): Must cite Reference (4) — Da Silva, F. A.; Báez, L. (1989). Use the comment format:
  `// Ref (4): Da Silva & Báez (1989), legacy/pascal/TERMOxx.PAS`
- **Algorithm references**: When implementing an algorithm from a specific paper, cite the reference number in ACS style in the module-level doc comment. The full reference list (ACS format) and mapping are in `MODERNIZATION_PLAN.md` under "Academic References" and "Reference-to-Code Mapping".
- Key references used in code: (5) Abbott — cubic EOS form, (9) Müller et al. — multicomponent fugacity, (10) Stockfleth & Dohrn — numerical Jacobian, (12) Poling & Prausnitz — root selection, (14) Asselineau et al. — high-pressure NR, (16) Heidemann & Khalil — critical point, (18) Hankinson & Thomson — liquid density, (19) Michelsen Part II — Rachford-Rice, (21) Orbey & Sandler — Wong-Sandler mixing rules.

## Algorithm Choices

The modernized code improves on several legacy numerical methods. When implementing these algorithms:

- **Root finding**: Use Brent's method (not Regula Falsi) as the default bracketed root finder in `numerics/root_finding.rs`. Illinois method available as lightweight alternative.
- **kij optimization**: Use Brent's method (not golden section) in `flash/kij_regression.rs`; warm-start each data point's objective from its neighbor.
- **Flash Jacobians**: Full Newton with the exact Jacobian from the derivative core (below) is the primary driver. Broyden quasi-Newton (`numerics/broyden.rs`) is the fallback for residuals without a cheap Jacobian.
- **Composition derivatives (∂ln φ̂ᵢ/∂nⱼ)**: Exact, never finite-difference. Analytic closed forms for cubic EOS + classical mixing; `num-dual` dual-number AD for Wong-Sandler/MHV1/MHV2 (mixing rules are written generic over the scalar type). FD survives only as a test oracle.
- **Rachford-Rice**: Halley's method (cubic convergence) **inside the Leibovici–Neoschil window** with a bisection safeguard in `flash/isothermal.rs` — guaranteed convergence, negative flash included.
- **Isothermal flash**: Wilson-correlation K init → TPD stability analysis (`flash/stability.rs`) → GDEM-accelerated successive substitution → Newton on ln K once the residual < ~1e-3. Never plain SS with trivial-solution guards.
- **Bubble/dew**: Log-variable Newton on {ln K, ln T or ln P}. Near-critical traversal via Michelsen phase-envelope continuation (`flash/envelope.rs`), not differential dP/dT stepping. The thesis two-stage scheme exists only as a test oracle.
- **Adiabatic (PH) flash**: Warm-started nested loop (inner flash seeded with previous T's K-values).
- **Aij regression**: Levenberg-Marquardt (not plain Newton) with the analytical Jacobian in `flash/aij_regression.rs`.
- **dα/dT and dGE/dT**: Always implement analytical derivatives. Numerical derivatives exist only as test oracles.
- **Helmholtz derivatives** (critical point): Analytical for 2-parameter cubic EOS; dual-number AD (not FD) for exotic mixing rules.
- **Cubic solver**: Cardano's method (keep as-is). Add (12) Poling & Prausnitz robustness for near-degenerate cases. Must not heap-allocate (`([f64; 3], usize)` return).

See `MODERNIZATION_PLAN.md` "Algorithm Performance Improvements" (§A–§M) + "Performance Engineering", and `PERFORMANCE_PROPOSAL.md` for full justifications.

## PyO3 Bindings Rule (M5+)

**From Milestone 5 forward, every milestone that adds Rust functionality must expose the new public functions or types as PyO3 bindings in the same commit series.** Pure-Rust-without-Python is not acceptable.

- New `#[pyfunction]`s go in `engine/src/py_bindings.rs` (or a co-located `#[pymodule]` block).
- New public types (structs / enums) get `#[cfg_attr(feature = "python", pyo3::pyclass(...))]` so they are exposed to Python alongside the Rust API.
- Add at least one test in `python/tests/test_engine.py` (or a new sibling file) per new binding, exercising it through the wheel.
- CI runs every test against the built wheel via cibuildwheel, so a missing binding is a hard failure — not a doc-day oversight.
- **Why this rule**: Python is a first-class consumer for this project. New Rust functionality that can't be called from Python is incomplete from the user's perspective. Wiring up bindings as the Rust code is written is dramatically cheaper than retrofitting them later.

The minimal scaffolding (`vle._engine` exposing `version()` plus the four enum types) shipped in M5 alongside the CI/CD setup, so the binding layer is load-bearing from the very first numerical algorithm in M6.

## Domain Context

- This is a thermodynamics/chemical engineering codebase. Variables like Tc, Pc, w (acentric factor), Zc, Ki, kij, Aij, alpha(Tr), Z-factor, phi (fugacity coefficient), gamma (activity coefficient) are standard notation.
- The original code and docs are in Spanish. Variable names in legacy code use Spanish (e.g., `TEbullicion` = boiling temperature, `PresionBR` = bubble/dew pressure, `EntalpiadeCondensacion` = condensation enthalpy).
- The research paper's Chapter IV validation cases (7 systems) are the primary correctness benchmark — results must match within 1–5%.

## Validation Cases (from Chapter IV)

1. Critical points — 4 mixtures with PR EOS (Tables 4.1–4.2)
2. Adiabatic flash — benzene/cyclohexane/methylcyclohexane/n-hexane (Tables 4.3–4.4)
3. Bubble point pressure — methanol/water with van Laar at 298K (Table 4.6)
4. Dew point temperature — 2-propanol/water with Wilson (Table 4.7)
5. Dew point pressure — 2-propanol/water with Wilson (Table 4.8)
6. Bubble point temperature — 4-component with Raoult's law (Table 4.9)
7. Isothermal flash — n-heptane/butane with RKS at 300K, 100kPa (Table 4.10)
8. kij regression — CO2/butane, kij=0.1357 (Tables 4.11–4.12)

## Deployment Rules

This repo distributes through **three channels only**: crates.io, PyPI, and the
**example notebooks** (run locally, or on any Jupyter). It no longer carries a
deployment — the multi-user Docker/JupyterHub stack that used to live under
`deploy/` moved to a **separate private operator repository** (an Ansible role
+ a gated deploy workflow). `deploy/` now holds only
`README.md` (distribution channels), `NOTEBOOKS.md` (host-agnostic notebook
guide), and `scripts/publish-{crate,pypi}.sh`. See `PUBLISHING.md` for the
release flow.

### After a release: refreshing the hub (operator-only, not in this repo)

`release.yml` publishes to crates.io + PyPI + GitHub Releases on a `v*` tag and
**deploys nowhere**. To update the hosted teaching hub, the operator runs the
gated deploy workflow in a separate private operator repository, choosing
`mode=notebooks` (content refresh) or `mode=full` (engine rebuild from source).
Nothing in this repo triggers that — keep it that way (no cross-repo deploy
coupling).

### Per-milestone artifact workflow

Every milestone that produces a user-facing artifact ends with:

1. **Create the milestone notebook** — see *Notebook Conventions* below.
2. **Update the notebook docs** — add the notebook to the `deploy/NOTEBOOKS.md`
   catalogue and note any new prerequisite; touch `deploy/README.md` only if a
   distribution channel changed.

Deploying that notebook to the hub is a **separate operator-side step** in a
private operator repository, not part of the milestone commit here.

### Keep private infrastructure out of committed files

The pre-push gate (see *Release & Push Rules*) blocks real domain names, public
IPs, Cloudflare Access team names, and TLS key material. In any committed
example use placeholders — `vle.example.com`, `203.0.113.10` (TEST-NET-3),
`admin@example.com`, `example.cloudflareaccess.com`, or `${VAR}`. All
operator-specific values (real hostnames, the tunnel token, cert paths) live
**only** in a separate private operator repository.

## Notebook Conventions

Every milestone-level Jupyter notebook MUST follow a professional structure so
the collection works as a coherent learning path for learners working through them.

**Required sections (top to bottom):**

1. **Title + one-sentence motivation** (H1 + lead paragraph).
2. **Optional setup cell (commented `%pip install --upgrade vle-thermo`)**
    — immediately after the title and before the
    research-paper context, every milestone notebook includes a short
    "Setup (optional)" markdown cell followed by a code cell containing
    exactly:

    ```python
    # Optional: pull the latest vle-thermo from PyPI.
    # Uncomment if you want the newest released version instead of
    # whatever is currently in your kernel.
    # %pip install --upgrade vle-thermo
    ```

    The cell stays **commented out by default** — the notebook must
    execute top-to-bottom without it being run. The point is to make the
    user's "I want to test against the latest published wheel" path
    obvious, not to actually do the install on the hub.
3. **Context from the research paper** — quote or paraphrase the relevant
   chapter/section/table from `docs/en/research-paper/`, with a relative link
   back to the source. Use blockquotes for direct quotes and cite the chapter:
   e.g. *"From [Chapter II §2.3](../docs/en/research-paper/chapter-2-vle-theory.md#23-cubic-equations-of-state)..."*.
4. **What was built in this milestone** — short prose pointing at the modules,
   structs, or CLI commands that the reader will call.
5. **Worked example** — one fully-executed example end-to-end, with markdown
   explaining each step, matching a result in the research paper where possible.
6. **User exercises — at least 2** — each with: a problem statement, a template
   code cell containing `# TODO:` markers, and a hidden/collapsed solution cell
   (use a `<details>` block in markdown, or a separate "Solutions" section at
   the bottom).
7. **References** — cross-links to the research paper, the parameter reference,
   and any MODERNIZATION_PLAN sections that describe the underlying algorithm.

**Other requirements:**

- All cells must execute top-to-bottom in a fresh kernel (no hidden state).
  Verify this with `jupyter nbconvert --to notebook --execute` before committing.
- Use `import matplotlib.pyplot as plt` and inline `%matplotlib inline`
  (JupyterLab renders it fine) for plots.
- Import units as `from vle.units import ureg, Q_` and express inputs with
  explicit units, e.g. `T = Q_(300, "K")`.
- Pin numeric expectations (research-paper table values) in assertion cells so
  regressions show up as a failing notebook, not a silent number drift.
- Snippets quoted from the research paper must preserve the original wording
  and cite the source. Never paraphrase equations — render them with LaTeX.

## Source Code Navigation

### VB6 (`legacy/vb6/`)
- `McommonFunctions.bas` — Shared numerics: cubic solver (line ~324), Gauss elimination (line ~24), EOS family constants (line ~273)
- `clsQbicsPure.cls` — Pure component EOS: all 19 alpha functions (line ~1719), Z-factor, fugacity, departure H/S
- `clsQbicsMulticomp.cls` — Mixture EOS: partial fugacity coefficients, mixing rules (line ~395)
- `clsActivityMulticomp.cls` — All 5 activity coefficient models
- `clsLVE.cls` — All flash calculations: bubble/dew T/P, isothermal/adiabatic flash, kij regression
- `clsSatPressureSolver.cls` — Saturation pressure correlations, Regula Falsi
- `clsVirial.cls` / `clsVirialMulticomp.cls` — Virial equation (pure + mixture)

### Pascal (`legacy/pascal/`)
- `TERMOI.PAS` — Constants, Antoine vapor pressure, cubic solver, saturation models
- `TERMOII.PAS` — All EOS (including Schmidt-Wenzel, Patel-Teja, Chao-Seader), mixing rules, fugacity
- `TERMOIII.PAS` — Activity models, excess properties, liquid volume, condensation enthalpy
- `TERMOIV.PAS` — Flash calculations, mixture critical point (ZCriticoMezcla)
- `TERMOV.PAS` — Aij regression with analytical Jacobian (line ~297)
- `TERMOVI.PAS` — kij golden section search, Gaussian elimination
