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

Do NOT push until all documentation accurately reflects the current state of the code. When in doubt, read each file and verify.

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
- Temperature: **K**
- Pressure: **kPa**
- Energy (molar): **kJ/kmol**
- Entropy (molar): **kJ/(kmol·K)**
- Volume (molar): **cm³/mol**
- Amount: **kmol**
- Gas constant R: **8.31451 kJ/(kmol·K)**

User-facing APIs should accept unit strings (e.g., `"25 degC"`, `"1 atm"`) via the `units` library (see `units/` crate and `python/src/vle/units.py`).

**User-extensible units**: The library ships with VLE defaults but must remain **extensible**. Users can add custom units (e.g., `mmH2O`, `barg`) via the runtime `UnitRegistry` in Rust or `ureg.define()` in Python, without modifying library source. When adding new code, do not hard-code the list of accepted units — always go through the registry. See [`docs/en/units/dimensional-analysis.md`](docs/en/units/dimensional-analysis.md) §7 for the extension API and rules.

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
