# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Release & Push Rules

**Before every `git push` or major release**, you MUST review and update all documentation to reflect current state:

1. **README.md** — Features list, project structure tree, time estimates, status
2. **ROADMAP.md** — Check off completed milestones, update in-progress items
3. **TODO.md** — Check off completed tasks, update time estimates and summary table
4. **MODERNIZATION_PLAN.md** — Update if architecture or phases changed
5. **CLAUDE.md** — Update if new conventions, paths, or tools were introduced
6. **PASCAL_VB6_COMPARISON.md** — Update if new legacy code analysis was done
7. **docs/en/research-paper/** — Update if translations were completed or links changed
8. **deploy/README.md** and **deploy/NOTEBOOKS.md** — Update if any generic install step, env var, or prerequisite changed
9. **deploy/.env.example** — Update if any new environment variable was added (with a safe example value, never a real one)

Do NOT push until all documentation accurately reflects the current state of the code. When in doubt, read each file and verify.

**Pre-push private-data gate** (run from repo root before every push):

```sh
# Anything matching this pattern in a to-be-pushed file is a leak.
git diff --cached origin/main -- ':!deploy/local' ':!deploy/.env' \
  | grep -E '(migueljackson\.dev|163\.192\.214\.135|cloudflareaccess\.com|BEGIN (RSA |EC )?PRIVATE KEY)' \
  && { echo "ABORT: private infrastructure detail found in staged changes"; exit 1; } \
  || echo "clean: no private details in diff"
```

If the grep hits anything, stop and move the offending content under `deploy/local/` or replace it with an `${ENV_VAR}` / `example.com` placeholder before pushing.

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

## Key Technical Decisions

- **Rust enums + match** map directly to VB6's `Select Case` dispatch over 22+ EOS variants, 5 activity models, 8 mixing rules
- **nalgebra** replaces hand-rolled Gauss elimination; **ndarray** for array ops
- Implement analytical derivatives for ALL variants (not just the 5 Pascal models) for dα/dT and dGE/dT. Numerical versions retained only as test oracles. See "Algorithm Choices" below.
- All 5 activity models and 6 flash calculation types are identical in both programs — single implementation each
- Pascal's 3-parameter EOS (Schmidt-Wenzel, Patel-Teja) require special C-parameter mixing rules not needed by 2-parameter EOS

## Reference Citation Requirements

This project is based on academic research. Code derived from legacy sources must cite the originating reference:

- **Pascal-derived code** (`legacy/pascal/`): Must cite Reference (4) — Da Silva, F. A.; Báez, L. (1989). Use the comment format:
  `// Ref (4): Da Silva & Báez (1989), legacy/pascal/TERMOxx.PAS`
- **Algorithm references**: When implementing an algorithm from a specific paper, cite the reference number in ACS style in the module-level doc comment. The full reference list (ACS format) and mapping are in `MODERNIZATION_PLAN.md` under "Academic References" and "Reference-to-Code Mapping".
- Key references used in code: (5) Abbott — cubic EOS form, (9) Müller et al. — multicomponent fugacity, (10) Stockfleth & Dohrn — numerical Jacobian, (12) Poling & Prausnitz — root selection, (14) Asselineau et al. — high-pressure NR, (16) Heidemann & Khalil — critical point, (18) Hankinson & Thomson — liquid density, (19) Michelsen Part II — Rachford-Rice, (21) Orbey & Sandler — Wong-Sandler mixing rules.

## Algorithm Choices

The modernized code improves on several legacy numerical methods. When implementing these algorithms:

- **Root finding**: Use Brent's method (not Regula Falsi) as the default bracketed root finder in `numerics/root_finding.rs`. Illinois method available as lightweight alternative.
- **kij optimization**: Use Brent's method (not golden section) in `flash/kij_regression.rs`.
- **Newton-Raphson Jacobian**: Use Broyden quasi-Newton update after first iteration in `numerics/newton_raphson.rs`, with periodic full Jacobian refresh every K steps.
- **Rachford-Rice**: Use Halley's method (cubic convergence) in `flash/isothermal.rs`.
- **dα/dT and dGE/dT**: Always implement analytical derivatives. Numerical derivatives exist only as test oracles.
- **Helmholtz derivatives** (critical point): Analytical for 2-parameter cubic EOS; numerical fallback for exotic mixing rules.
- **Cubic solver**: Cardano's method (keep as-is). Add (12) Poling & Prausnitz robustness for near-degenerate cases.

See `MODERNIZATION_PLAN.md` "Algorithm Performance Improvements" for full justifications.

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

The project ships a public, generic deployment story in parallel with a private,
operator-specific one. Both evolve together, but **only the public track is
ever committed to GitHub**.

### Two-track layout

| Track | Location | Scope | In git? |
|-------|----------|-------|---------|
| Public — generic | `deploy/README.md`, `deploy/NOTEBOOKS.md`, `deploy/.env.example`, `deploy/docker/`, `deploy/compose/`, `deploy/scripts/` | Works on any ARM64 Linux host with Docker + a reverse proxy + a header-setting auth gateway | Yes |
| Private — operator-specific | `deploy/local/DEPLOYMENT.md`, `deploy/local/deploy-notes/milestone-*.md`, `deploy/.env` | Hostname, IPs, cert paths, admin emails, Cloudflare Access team, real secrets | **No** — gitignored |

The `.gitignore` already blocks `deploy/local/`, `deploy/.env`, `*.pem`, `*.key`.
Do not undo these entries. When adding new private files, put them under
`deploy/local/` so they are covered automatically.

### What must never appear in a committed file

- Real domain names — use `vle.example.com` or `${DOMAIN}` in examples.
- Real public IP addresses — use `203.0.113.10` (TEST-NET-3) or `${HOST_IP}`.
- Real email addresses — use `admin@example.com` or `${JUPYTERHUB_ADMIN_EMAIL}`.
- Real Cloudflare Access team names — use `example.cloudflareaccess.com`.
- Any value from `deploy/local/.env` or `deploy/local/DEPLOYMENT.md`.
- TLS certificate or key material, even if expired.
- VPS provider, region, instance size, or SSH/Tailscale routing details.

If you need a non-generic value in a public file, it is almost certainly wrong —
push it through `.env` / `.env.example` via a `${VAR}` substitution instead.

### Per-milestone deployment workflow

Every non-completed milestone that produces a user-facing artifact (notebook,
CLI, or library API) ends with **four parallel steps**, executed in this
order after validation tests pass:

1. **Create the milestone notebook** — see *Notebook Conventions* below.
2. **Update the public deployment docs** — `deploy/README.md`, `deploy/NOTEBOOKS.md`,
   and `deploy/.env.example` get any new generic install step, env var, or
   prerequisite that this milestone introduced.
3. **Update the private deployment notes** — append a new
   `deploy/local/deploy-notes/milestone-NN.md` with the operator-specific
   build/rebuild/restart commands for Miguel's host, referencing the values in
   `deploy/local/.env` and `deploy/local/DEPLOYMENT.md`.
4. **Deploy the notebook to JupyterHub** — rebuild the notebook image, restart
   the hub, and verify the notebook is reachable through the production URL.
   Record the outcome at the bottom of `deploy/local/deploy-notes/milestone-NN.md`.

Steps 2 and 3 happen **in the same commit series** so the two tracks never
diverge. After all milestones complete, Milestone 10 performs one final
redeployment that includes the Chapter IV walkthrough notebook.

## Notebook Conventions

Every milestone-level Jupyter notebook MUST follow a professional structure so
the collection works as a coherent learning path for users on the hub.

**Required sections (top to bottom):**

1. **Title + one-sentence motivation** (H1 + lead paragraph).
2. **Context from the research paper** — quote or paraphrase the relevant
   chapter/section/table from `docs/en/research-paper/`, with a relative link
   back to the source. Use blockquotes for direct quotes and cite the chapter:
   e.g. *"From [Chapter II §2.3](../docs/en/research-paper/chapter-2-vle-theory.md#23-cubic-equations-of-state)..."*.
3. **What was built in this milestone** — short prose pointing at the modules,
   structs, or CLI commands that the reader will call.
4. **Worked example** — one fully-executed example end-to-end, with markdown
   explaining each step, matching a result in the research paper where possible.
5. **User exercises — at least 2** — each with: a problem statement, a template
   code cell containing `# TODO:` markers, and a hidden/collapsed solution cell
   (use a `<details>` block in markdown, or a separate "Solutions" section at
   the bottom).
6. **References** — cross-links to the research paper, the parameter reference,
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
