# VLE Thermodynamic Calculator Modernization Plan

## Context

Two original thermodynamic programs are being modernized into a unified library:

1. **VB6 program** (`legacy/vb6/`) -- A production-grade Vapor-Liquid Equilibrium calculator developed as part of the thesis: *"Desarrollo de un Programa Computacional para el Cálculo del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente Windows"* by Miguel Roberto Jackson Ugueto and Carlos Fernando Mendible Porras, Proyecto de Grado, Universidad Simón Bolívar, Sartenejas, April 1999. Advisors: Prof. Coray M. Colina and Prof. Jean-Marie Ledanois. It implements 19+ cubic equations of state, virial models, 5 activity coefficient models, 8 mixing rules, and flash calculation algorithms across ~450KB of source code.

2. **Pascal program** (`legacy/pascal/`) -- An earlier thermodynamic package (Caracas, 1989) by Francisco Avelino Da Silva and Luis Alberto Baez Linde (4), written for classic Macintosh in 6 Pascal units (TERMOI-TERMOVI, ~2,500 lines). It shares significant core functionality with the VB6 program but adds unique EOS families (Schmidt-Wenzel, Patel-Teja, Chao-Seader), an Antoine vapor pressure model, and Aij regression for activity model binary parameters.

A detailed comparison of both programs is in `PASCAL_VB6_COMPARISON.md`. The shared functionality is merged into single implementations; unique features from each program are added as separate functions in the common library.

The goal is to modernize both into a **fast Rust computation engine** with a **Python wrapper library** and **Jupyter notebooks** that reproduce the research paper's calculations interactively.

---

## Academic References

This project is based on the thesis by Jackson & Mendible (1999, Universidad Simón Bolívar) and the earlier work by Da Silva & Báez (1989). The references below are cited throughout the codebase and modernization plan, formatted in ACS (American Chemical Society) style.

**Attribution requirement: All Rust code derived from the Pascal codebase (`legacy/pascal/`) must include a comment citing Reference (4). Comment format: `// Ref (4): Da Silva & Báez (1989), legacy/pascal/TERMOxx.PAS`**

Similarly, when implementing an algorithm from a specific paper below, cite the reference number in a comment at the function or module level.

(1) Da Silva, F. A.; Báez, L.; Müller, E. A User Friendly Program for Vapor-Liquid Equilibrium. *Chem. Eng. Educ.* **1990**, *24*.

(2) Jackson, M.; Mendible, C.; Olivera Fuentes, C.; Ledanois, J. M.; Colina, C. M. USBQbics para Windows: Computer Calculation of Thermodynamic Properties of Pure Substances and Multicomponent Mixtures Using Cubic Equations of State. *Memorias de las X Jornadas Científico Técnicas de Ingeniería*; Universidad del Zulia, 1998.

(3) Sandler, S. I. *Chemical Engineering Thermodynamics*, 2nd ed.; Wiley: New York, 1989.

(4) Da Silva, F. A.; Báez, L. Desarrollo de un Paquete Computacional para la Predicción de Propiedades Termodinámicas y de Equilibrio de Fases. Thesis, Universidad Simón Bolívar, 1989.

(5) Abbott, M. M. Cubic Equations of State: An Interpretive Review. In *Equations of State in Engineering and Research*; Chao, K. C., Robinson, R. L., Eds.; ACS Symposium Series 182; American Chemical Society: Washington, D.C., 1979; pp 47–97.

(6) Fotouh, K.; Shukla, K. A Comparative Study of Numerical Methods for Calculating Phase Equilibria in Fluid Mixtures from an Equation of State. *Chem. Eng. Sci.* **1996**, *51* (15), 3763.

(7) Michelsen, M. L. The Isothermal Flash Problem. Part I. Stability. *Fluid Phase Equilib.* **1982**, *9*, 1.

(8) Eubank, P. T.; Elhassan, A. E.; Barrufet, M. A.; Whiting, W. B. Area Method for Prediction of Fluid Phase Equilibria. *Ind. Eng. Chem. Res.* **1992**, *31*, 942.

(9) Müller, E.; Olivera Fuentes, C.; Estévez, L. General Expressions for Multicomponent Fugacity Coefficients and Residual Properties from Cubic Equations of State. *Lat. Am. Appl. Res.* **1989**, *19* (2), 99.

(10) Stockfleth, R.; Dohrn, R. An Algorithm for Calculating Critical Points in Multicomponent Mixtures Which Can Easily Be Implemented in Existing Programs to Calculate Phase Equilibria. *Fluid Phase Equilib.* **1998**, *145*, 43.

(11) Null, H. R. *Phase Equilibrium in Process Design*; Wiley: New York, 1970.

(12) Poling, E. A.; Prausnitz, J. M. Thermodynamic Properties from a Cubic Equation of State: Avoiding Trivial Roots and Spurious Derivatives. *Ind. Eng. Chem. Process Des. Dev.* **1981**, *20*, 127.

(13) Gundersen, T. Numerical Aspects of the Implementation of Cubic Equations of State in Flash Calculation Routines. *Comput. Chem. Eng.* **1982**, *6* (3), 245.

(14) Asselineau, L.; Bogdanic, G.; Vidal, J. A Versatile Algorithm for Calculating Vapor-Liquid Equilibria. *Fluid Phase Equilib.* **1979**, *3*, 273.

(15) Peng, D.; Robinson, D. B. A Rigorous Method for Predicting the Critical Properties of Multicomponent Systems from an Equation of State. *AIChE J.* **1977**, *23* (2), 137.

(16) Heidemann, R. A.; Khalil, A. M. The Calculation of Critical Points. *AIChE J.* **1980**, *26* (5), 769.

(17) Assael, M. J.; Trusler, J. P.; Tsolakis, T. F. *Thermophysical Properties of Fluids*; Imperial College Press, 1996.

(18) Hankinson, R. W.; Thomson, G. H. A New Correlation for Saturated Densities of Liquids and Their Mixtures. *AIChE J.* **1979**, *25* (4), 653.

(19) Michelsen, M. L. The Isothermal Flash Problem. Part II. Phase-Split Calculation. *Fluid Phase Equilib.* **1982**, *9*, 21.

(20) Anderson, T. F.; Prausnitz, J. M. Computational Methods for High-Pressure Phase Equilibria and Other Fluid-Phase Properties Using a Partition Function. 2. Mixtures. *Ind. Eng. Chem. Process Des. Dev.* **1980**, *19*, 9.

(21) Orbey, H.; Sandler, S. I. *Modeling Vapor-Liquid Equilibria. Cubic Equations of State and Their Mixing Rules*, 1st ed.; Cambridge University Press, 1998.

(22) Smith, J. M.; Van Ness, H. C.; Abbott, M. M. *Introduction to Chemical Engineering Thermodynamics*, 5th ed.; McGraw-Hill, 1996.

---

## Reference-to-Code Mapping

| Ref | Algorithm / Concept | Target Rust Module(s) |
|-----|--------------------|-----------------------|
| (1) Da Silva et al. (1990) | Published description of Ekilib program | Context only |
| (2) Jackson et al. (1998) | Published description of USBQbics program | Context only |
| (3) Sandler (1989) | General thermodynamic framework; validation data | Validation reference |
| **(4) Da Silva & Báez (1989)** | **Pascal codebase origin**: parabolic interpolation for bubble/dew, adiabatic flash, kij golden section, K-value estimates, 3-param EOS (Schmidt-Wenzel, Patel-Teja), Chao-Seader, Antoine, Aij regression with analytical Jacobian, analytical dα/dT, analytical excess enthalpy | `flash/*`, `eos/three_param.rs`, `eos/chao_seader.rs`, `eos/alpha.rs`, `saturation/pressure.rs`, `flash/aij_regression.rs`, `activity/models.rs` |
| (5) Abbott (1979) | General cubic EOS form: k1, k2, k3 parameterization (Table 2.1 of thesis) | `eos/cubic_params.rs` |
| (6) Fotouh & Shukla (1996) | Numerical problems near critical point | Context only (not implemented) |
| (7) Michelsen (1982) Part I | Gibbs energy minimization / stability analysis | Future work (not implemented) |
| (8) Eubank et al. (1992) | Area method for phase equilibria | Future work (not implemented) |
| (9) Müller et al. (1989) | General multicomponent fugacity coefficients and residual properties from cubic EOS (Eqs 2.28–2.34 of thesis) | `eos/multicomp.rs` |
| (10) Stockfleth & Dohrn (1998) | Numerical compositional derivatives for fugacity; numerical Jacobian for 2n+4 flash system | `numerics/newton_raphson.rs` |
| (11) Null (1970) | Fugacity equality as necessary but not sufficient condition for equilibrium | Design consideration |
| (12) Poling & Prausnitz (1981) | Avoiding trivial roots and spurious derivatives; root selection logic in cubic solver | `eos/cubic_solver.rs` |
| (13) Gurdensen (1982) | Numerical aspects of cubic EOS in flash; volume root selection | `eos/cubic_solver.rs` |
| (14) Asselineau et al. (1979) | Newton-Raphson multivariable VLE near critical; 2nd stage of high-pressure bubble/dew algorithm | `flash/bubble.rs`, `flash/dew.rs` |
| (15) Peng & Robinson (1977) | Critical point prediction for multicomponent systems | `flash/critical.rs` |
| (16) Heidemann & Khalil (1980) | Critical point calculation algorithm (Eqs 2.14–2.23 of thesis) | `flash/critical.rs` |
| (17) Assael et al. (1996) | General VLE algorithm framework (Figs 2.2, 2.4, 2.6 of thesis) | `flash/` module structure |
| (18) Hankinson & Thomson (1979) | Saturated liquid density correlation (Thomson/COSTALD model) | `activity/liquid_volume.rs` |
| (19) Michelsen (1982) Part II | Phase split / Rachford-Rice framework | `flash/isothermal.rs` |
| (20) Anderson & Prausnitz (1980) | High-pressure VLE; 2nd stage of bubble/dew algorithm (Fig 2.7 of thesis) | `flash/bubble.rs`, `flash/dew.rs` |
| (21) Orbey & Sandler (1998) | Wong-Sandler, Huron-Vidal, and advanced mixing rules; validation data for bubble point | `mixing/rules.rs` |
| (22) Smith, Van Ness & Abbott (1996) | Dew point validation data (Tables 4.7–4.8) | `python/tests/test_validation.py` |

---

## Algorithm Performance Improvements

The modernized Rust code improves on several legacy numerical methods. Each subsection describes the legacy approach, the proposed improvement, and the justification.

### A. Newton-Raphson Jacobian (`numerics/newton_raphson.rs`)

- **Legacy**: VB6 `clsLVE.cls` `NR_JacobianMatrix` computes the full Jacobian numerically via a 5-point stencil, requiring (m+1) function evaluations per Newton-Raphson iteration for the 2n+4 flash system (10).
- **Improvement**: Broyden's quasi-Newton rank-1 update after the first iteration (1 function evaluation per step). Full numerical Jacobian refresh every K=5 steps or when convergence stalls. This is a standard approach for large nonlinear systems where the Jacobian is expensive to compute.
- **Justification**: For a 10-component mixture (m=24), this reduces evaluations from ~25/step to ~1 for most steps. The 2n+4 system evaluation is the dominant cost in flash calculations.
- **Risk mitigation**: Broyden can diverge if the initial Jacobian is poor. The periodic refresh (every K steps) and automatic fallback to full Jacobian on stalled convergence mitigate this.

### B. Golden Section to Brent's Method (`flash/kij_regression.rs`)

- **Legacy**: Pascal `TERMOVI.PAS` and VB6 `clsLVE.cls` use golden section search for kij optimization (linear convergence, reduction ratio 0.618034 per step) (4).
- **Improvement**: Brent's method combining golden section with inverse quadratic interpolation (superlinear convergence while maintaining the bracketing guarantee). The VB6 codebase already uses Brent's method for adiabatic flash temperature search (`clsLVE.cls`, citing Numerical Recipes) -- the same logic is reused.
- **Justification**: Achieves superlinear convergence while maintaining bracketing safety. Typically converges in roughly half the iterations of pure golden section.

### C. Regula Falsi to Illinois/Brent's (`numerics/root_finding.rs`)

- **Legacy**: VB6 `clsSatPressureSolver.cls` and `clsQbicsPure.cls` use Regula Falsi for saturation temperature and pressure root finding (can stall indefinitely on one endpoint when the function is convex or concave).
- **Improvement**: Illinois algorithm (modified Regula Falsi: halve the function value on the stalled endpoint) as the lightweight option, or Brent's method as the robust default. Both maintain the bracketing guarantee while achieving superlinear convergence.
- **Justification**: Standard Regula Falsi is known to stall on monotone convex/concave functions. The Illinois modification fixes this at zero additional cost per iteration. Brent's method adds inverse quadratic interpolation for even faster convergence.

### D. Analytical dα/dT for All EOS Variants (`eos/alpha.rs`)

- **Legacy**: VB6 `clsQbicsPure.cls` uses a 5-point numerical stencil (4 function evaluations) for dA/dT across all 19 EOS variants. Pascal `TERMOII.PAS:411` (4) has analytical `Aa_T` for 5 EOS variants (VdW-Adachi, RKS, RKS-Polar, PR, Patel-Teja).
- **Improvement**: Implement analytical dα/dTr for ALL 22+ EOS variants. Every alpha function is an explicit closed-form expression of Tr and the acentric factor, so its derivative is straightforward to derive symbolically.
- **Justification**: This derivative is called inside every enthalpy/entropy departure calculation, which is called inside every flash iteration. Eliminating 4 function evaluations per call compounds across the entire solver. The Pascal code (4) already proves feasibility for 5 models.
- **Implementation**: Each variant in the `Alpha` enum gets a `d_alpha_d_tr(&self, tr: f64) -> f64` method alongside the existing `alpha(&self, tr: f64) -> f64`. Numerical derivatives are retained only as test oracles for verifying the analytical implementations.

### E. Analytical dGE/dT for Excess Enthalpy (`activity/models.rs`)

- **Legacy**: VB6 `clsActivityMulticomp.cls` uses finite difference (2 function evaluations, h=0.001) for excess enthalpy via numerical dGE/dT. Pascal `TERMOIII.PAS` (4) has analytical expressions for Wilson.
- **Improvement**: Analytical dGE/dT for all 5 activity models. Known closed forms: Ideal (zero), Margules (HE = GE since GE/T cancels), van Laar (HE = GE since parameters scale as 1/T), Wilson (temperature-dependent Lambda yields explicit dGE/dT), Scatchard-Hildebrand (depends on dVl/dT).
- **Justification**: Eliminates numerical cancellation errors that plague finite differences when GE varies slowly with T (common at moderate pressures). Also saves 2 evaluations per HE calculation.

### F. Halley's Method for Rachford-Rice (`flash/isothermal.rs`)

- **Legacy**: Both VB6 and Pascal use Newton-Raphson on the scalar Rachford-Rice equation f(β) = Σ zi(Ki-1)/(1+β(Ki-1)) = 0 (quadratic convergence) (19).
- **Improvement**: Halley's method (cubic convergence). For the Rachford-Rice equation, f, f', and f'' are all computed from the same summation loop with trivial additional arithmetic per component:
  - f' = -Σ zi(Ki-1)²/(1+β(Ki-1))²
  - f'' = 2Σ zi(Ki-1)³/(1+β(Ki-1))³
- **Justification**: Cubic convergence at negligible extra cost (one additional multiply-accumulate per component in the sum). Also consider the Leibovici & Neoschil negative flash formulation for robustness when β is near 0 or 1.

### G. Analytical Helmholtz Derivatives for Critical Point (`flash/critical.rs`)

- **Legacy**: VB6 `clsLVE.cls` Heidemann algorithm (16) uses numerical 2nd and 3rd derivatives of Helmholtz free energy via finite differences (many function evaluations per iteration).
- **Improvement**: Analytical 2nd and 3rd derivatives of Helmholtz free energy for standard 2-parameter cubic EOS with classical mixing rules. These are well-known expressions that depend on a, b, and their compositional derivatives. Reserve numerical derivatives only for exotic mixing rules (Wong-Sandler, MHV1, MHV2) where analytical forms are prohibitively complex.
- **Justification**: The Heidemann inner loop repeatedly evaluates these derivatives. Analytical forms eliminate the dominant cost for the common case (classical mixing with PR or RKS).

### H. Cardano Cubic Solver Robustness (`eos/cubic_solver.rs`)

- **Legacy**: Already optimal (Cardano's analytical closed form). Both programs implement the same discriminant-based approach with trigonometric solution for three real roots.
- **Improvement**: Keep Cardano's method. Add robust handling for near-degenerate cases (discriminant ≈ 0) using the approach from (12) (Poling & Prausnitz) to avoid trivial roots and spurious derivatives. Apply (13) (Gurdensen) for volume root selection in flash contexts (choose correct phase root based on Gibbs energy comparison).
- **Justification**: Near the critical point, the cubic discriminant approaches zero and standard implementations suffer floating-point cancellation. This is a robustness improvement, not a speed improvement.

---

## Architecture Decision: Rust + PyO3

**Recommendation: Rust** for the computation engine, with PyO3/maturin for Python bindings.

**Why Rust over C/C++:**
- The VB6 code is heavily enum-driven (19 EOS variants, 5 activity models, 8 mixing rules dispatched via `Select Case`). Rust's `enum` + `match` with exhaustive checking maps perfectly and prevents silent fallthrough bugs.
- Memory safety eliminates index-out-of-bounds bugs common in array-heavy numerical code (the VB6 code passes arrays by reference everywhere with manual indexing).
- PyO3 generates native Python modules directly from Rust structs/functions -- no separate binding layer needed.
- maturin handles wheel building/distribution trivially.
- `nalgebra` crate replaces hand-rolled Gauss elimination; `ndarray` available for array ops.
- Compiles to native code with LLVM optimizations -- critical for the Newton-Raphson inner loop which evaluates the full system (2n+4) times per iteration.

---

## Units of Measurement Add-On

An **independent `units/` Rust crate** (sibling to `engine/`) plus a Python companion (`python/src/vle/units.py`) provides dimensional-analysis-based unit conversion. The units library works standalone and wraps VLE API boundaries so users can pass inputs in any compatible unit (°C, °F, bar, atm, psi, etc.) while the engine operates on canonical metric units internally.

**Legacy units (confirmed in both VB6 and Pascal codebases)**: T in K, P in kPa/bar, molar energy in kJ/kmol, molar entropy in kJ/(kmol·K), R = 8.31451 kJ/(kmol·K) (VB6) / 8.3144 (Pascal). See `legacy/vb6/McommonFunctions.bas:3` and `legacy/pascal/TERMOI.PAS:13`, with Pascal's explicit units comment at `TERMOII.PAS:62-63`.

**Implementation**:
- Rust: `uom` crate (compile-time dimensional analysis via phantom types, zero runtime cost, 7 SI base dimensions)
- Python: `pint` library (runtime dimensional analysis, NumPy integration)
- Canonical internal units match legacy code exactly so validation data is reusable without conversion

**References**: Bridgman, P.W. *Dimensional Analysis*, Yale University Press, 1922; BIPM, *The International System of Units (SI)*, 9th ed., 2019.

**Detailed design document**: [`docs/en/units/dimensional-analysis.md`](docs/en/units/dimensional-analysis.md) explains the 7 SI base dimensions, dimensional homogeneity principle, conversion strategy, and the Rust phantom-type / Python runtime implementation approach. See also `ROADMAP.md` Milestone 1.5 for task breakdown.

---

## Project Structure

```
vle/
├── units/                           # Independent units crate (dimensional analysis via uom)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── vle_units.rs             # Temperature, Pressure, MolarEnergy, etc.
│   │   ├── parsing.rs               # Parse "kPa", "degC", etc. to typed quantities
│   │   └── canonical.rs             # to_canonical() / from_canonical()
│   └── tests/
├── engine/                          # Rust crate (core computation)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                   # Crate root, re-exports
│       ├── constants.rs             # R=8.31451, Pi, universal constants
│       ├── types/
│       │   ├── mod.rs
│       │   ├── component.rs         # Component properties (Tc, Pc, w, Zc, etc.)
│       │   ├── mixture.rs           # Mixture definition + model selections
│       │   ├── flow.rs              # Stream: T, P, V, H, S, molar fractions
│       │   ├── tolerances.rs        # Convergence tolerances
│       │   └── reference_state.rs   # LiqSat/VapSat/IdealGas reference
│       ├── eos/
│       │   ├── mod.rs
│       │   ├── cubic_params.rs      # EOS family constants (K1,K2,K3,OmA,OmB,OmC,h[5])
│       │   ├── alpha.rs             # 22+ alpha(Tr) functions (19 VB6 + Schmidt-Wenzel, Patel-Teja, Chao-Seader from Pascal)
│       │   ├── cubic_solver.rs      # Cardano's method
│       │   ├── pure.rs              # Pure component: Z, fugacity, HR/SR, Maxwell
│       │   ├── three_param.rs       # Schmidt-Wenzel and Patel-Teja 3-parameter EOS (from Pascal)
│       │   ├── chao_seader.rs       # Chao-Seader liquid fugacity correlation (from Pascal, incl. H2/methane)
│       │   ├── multicomp.rs         # Mixture: partial fugacity coefficients
│       │   └── virial.rs            # Tsonopoulos virial (pure + multicomp)
│       ├── activity/
│       │   ├── mod.rs
│       │   ├── models.rs            # Wilson, van Laar, Margules, Scatchard-Hildebrand, Ideal
│       │   └── liquid_volume.rs     # Rackett / Thomson molar volume
│       ├── mixing/
│       │   ├── mod.rs
│       │   └── rules.rs             # Classical, WS, HOV, HVS, MHV1, MHV2, IVDW, IIVDW
│       ├── saturation/
│       │   ├── mod.rs
│       │   └── pressure.rs          # Antoine (from Pascal), Riedel, Muller, RPM, polynomial, Maxwell
│       ├── flash/
│       │   ├── mod.rs
│       │   ├── bubble.rs            # Bubble point (T and P)
│       │   ├── dew.rs               # Dew point (T and P)
│       │   ├── isothermal.rs        # Isothermal flash
│       │   ├── adiabatic.rs         # Adiabatic flash
│       │   ├── critical.rs          # Mixture critical point (Heidemann + Pascal ZCriticoMezcla)
│       │   ├── kij_regression.rs    # Binary interaction parameter fitting (golden section from both)
│       │   └── aij_regression.rs    # Activity model Aij regression (from Pascal, NR + analytical Jacobian)
│       ├── thermo/
│       │   ├── mod.rs
│       │   ├── enthalpy.rs          # Cp integration, departure functions
│       │   └── entropy.rs           # Departure functions
│       ├── numerics/
│       │   ├── mod.rs
│       │   ├── newton_raphson.rs    # NR with numerical Jacobian
│       │   ├── linear_algebra.rs    # LU decomposition (via nalgebra)
│       │   └── root_finding.rs      # Regula Falsi, parabolic interpolation
│       └── bindings.rs              # PyO3 module definition
├── python/                          # Python wrapper package
│   ├── pyproject.toml               # maturin build backend
│   ├── src/vle/
│   │   ├── __init__.py              # Public API
│   │   ├── _engine.pyi             # Type stubs for Rust bindings
│   │   ├── units.py                 # Pint-based unit conversion wrapper
│   │   ├── system.py                # High-level System class
│   │   ├── models.py                # Enums: EOS, ActivityModel, MixingRule
│   │   ├── components.py            # Built-in component database
│   │   ├── plotting.py              # Pxy, Txy, phase envelope plots
│   │   └── results.py              # FlashResult, BubbleResult dataclasses
│   └── tests/
│       ├── test_pure_eos.py
│       ├── test_activity.py
│       ├── test_bubble_dew.py
│       ├── test_flash.py
│       └── test_validation.py       # Chapter IV test cases
├── notebooks/
│   ├── 01_introduction.ipynb
│   ├── 02_pure_component.ipynb
│   ├── 03_activity_models.ipynb
│   ├── 04_bubble_dew_point.ipynb
│   ├── 05_flash_calculations.ipynb
│   ├── 06_critical_points.ipynb
│   ├── 07_kij_regression.ipynb
│   ├── 08_aij_regression.ipynb       # Activity model Aij fitting (from Pascal)
│   └── data/
│       ├── components.json          # Component property database
│       └── experimental/            # Experimental data for validation
├── docs/
│   ├── en/
│   │   ├── research-paper/          # English translation (navigatable)
│   │   │   ├── README.md            # Table of contents with links
│   │   │   ├── abstract.md
│   │   │   ├── chapter-1-introduction.md
│   │   │   ├── chapter-2-vle-theory.md
│   │   │   ├── chapter-3-architecture.md
│   │   │   ├── chapter-4-validation.md
│   │   │   ├── chapter-5-conclusions.md
│   │   │   ├── references.md
│   │   │   ├── list-of-figures.md
│   │   │   ├── list-of-tables.md
│   │   │   ├── list-of-symbols.md
│   │   │   └── appendices/
│   │   │       ├── appendix-a-analyst-manual.md
│   │   │       └── appendix-b-user-manual.md
│   │   └── parameters/
│   │       └── parameter_reference.md
│   └── es/
│       └── research-paper/          # Spanish originals (preserved)
│           ├── Portada.md
│           ├── Resumen.md
│           ├── Indice general.md
│           ├── Capitulo I.md ... Capitulo V.md
│           ├── REFERENCIAS.md
│           ├── Lista de simbolos.md
│           ├── Lista de Figuras.md
│           ├── Lista de Tablas.md
│           └── programdocs/
│               ├── Analista.md
│               └── dllManual.md
├── legacy/
│   ├── vb6/                         # Original VB6 source (reference)
│   │   └── *.cls, *.bas
│   └── pascal/                      # Original Pascal source (reference) (4)
│       └── TERMOI.PAS ... TERMOVI.PAS
```

---

## Implementation Phases

> **Phase numbering matches milestone order in [ROADMAP.md](ROADMAP.md) and [TODO.md](TODO.md).**
> Each milestone maps to one or more phases in this section. When adding, removing, or reordering phases, update ROADMAP.md's `*Phase N of MODERNIZATION_PLAN.md*` pointers in lockstep.

### Phase 1: Documentation & Translation *(Milestone 1)*
- Translate all 5 research paper chapters from Spanish to English
- Translate program documentation (Analista.md, dllManual.md) to English
- Create comprehensive parameter reference document
- Create architecture decision records

### Phase 2: Units of Measurement Library *(Milestone 1.5)*
- Scaffold independent `units/` Rust crate with `uom` dependency — works without the VLE engine
- Define VLE-specific quantity types (Temperature, Pressure, MolarEnergy, MolarEntropy, MolarVolume, Amount) as aliases for `uom`'s SI types
- Implement extensible runtime `UnitRegistry` supporting user-defined units alongside the compile-time typed API
- Unit string parser and canonical conversion (canonical: K, kPa, kJ/kmol, kJ/(kmol·K), cm³/mol, kmol)
- TOML unit file loader shared between Rust and Python
- Python wrapper `python/src/vle/units.py` around `pint`, exposing `ureg` for user extensions
- See `docs/en/units/dimensional-analysis.md` for the full design and extension rules

### Phase 3: Project Scaffolding *(Milestone 2)*
- Initialize Rust crate (`engine/`) with Cargo.toml, nalgebra + PyO3 dependencies
- Initialize Python package (`python/`) with pyproject.toml and maturin config
- Define all Rust enums merging both programs: `CubicEos` (22+ variants: 19 VB6 + Schmidt-Wenzel, Patel-Teja, Chao-Seader from Pascal), `ActivityModel` (5, identical in both), `MixingRule` (8 from VB6 + Patel-Teja/Schmidt-Wenzel C-parameter mixing from Pascal), `SatPressureModel` (6: Antoine from Pascal + 5 from VB6)
- Define core structs: `Component` (union of VB6 and Pascal fields, including Pascal's `momentoDip`, `delta`, `vl`), `Mixture`, `Flow`, `Tolerances`, `ReferenceState`
- Verify maturin builds and Python can import the empty module

### Phase 4: Numerics *(Milestone 3)*
- Cardano cubic solver (from `McommonFunctions.bas:324`) (12),(13) — see also §H for robustness improvements
- Gaussian elimination with partial pivoting (from `McommonFunctions.bas:24`) — replaced by nalgebra LU
- Brent's method and Illinois algorithm root finders — replace legacy Regula Falsi (from `clsSatPressureSolver.cls`) — see §C
- Utility functions: `SumFrac`, `Norm`, convergence checks

### Phase 5: Pure Component EOS *(Milestone 4)*
- `GeneralConstantsEOS` parameter lookup for 3 EOS families (from `McommonFunctions.bas:273`) (5)
- All 19 VB6 `alpha(Tr)` functions (from `clsQbicsPure.cls:1719`)
- **Pascal-origin EOS** (4) (from `legacy/pascal/TERMOII.PAS`):
  - Schmidt-Wenzel: 3-parameter cubic with acentric-factor-dependent covolume (Beta parameter), special C-parameter mixing
  - Patel-Teja: 3-parameter cubic with component-specific Zc, two mixing variants (PatelT, PatelTUSB)
  - Chao-Seader: liquid fugacity correlation with 10+ parameter fit (separate coefficient sets for normal compounds, H2, and methane)
- Z-factor calculation using cubic solver (handles 2-parameter and 3-parameter cubics)
- Fugacity coefficient, departure H/S
- Maxwell equal-area test for saturation
- **Key source files:** `legacy/vb6/clsQbicsPure.cls`, `legacy/pascal/TERMOII.PAS`

### Phase 6: Saturation Pressure *(Milestone 4)*
- **Antoine** correlation (4): ln(P/Pc) = a1 - a2/(a3+T) (from `legacy/pascal/TERMOI.PAS`)
  - Includes `PseudoAntoine` procedure for converting other models to local Antoine equivalents
- Riedel, Muller, RPM correlations (shared by both programs; VB6: `clsSatPressureSolver.cls:146`, Pascal: `TERMOI.PAS:154`)
- Database polynomial evaluation (VB6-only)
- Temperature derivatives (analytical for Antoine, numerical for others)
- Boiling point calculation (`TEbullicion` from Pascal / `SatTemperature` from VB6)
- Poynting correction factor (identical in both: `exp((P-Psat)*Vl/R/T/10)`)
- **Key source files:** `legacy/vb6/clsSatPressureSolver.cls`, `legacy/pascal/TERMOI.PAS`

### Phase 7: Virial Equation *(Milestone 4)*
- Pitzer correlation: B0 = 0.083 - 0.422/Tr^1.6, B1 = 0.139 - 0.172/Tr^4.2
- Pure and multicomponent Z, fugacity, HR/SR
- **Key source files:** `legacy/vb6/clsVirial.cls`, `legacy/vb6/clsVirialMulticomp.cls`

### Phase 8: Activity Coefficient Models *(Milestone 5)*
- Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand (identical in both programs)
- Excess Gibbs energy, excess enthalpy/entropy — implement analytical dGE/dT for ALL models (Pascal (4) has analytical for Wilson; extend to all) — see §E
- Rackett and Thomson (18) liquid molar volume (VB6)
- Temperature-dependent Aij scaling: `VariarLosAij` (4) (from Pascal `TERMOIII.PAS:384`, handles Wilson/VanLaar/Margules differently)
- Wilson binary parameter calculation from infinite-dilution activity coefficients: `CalcParBinWilson` (4) (from Pascal `TERMOIII.PAS:342`, Newton-Raphson)
- **Key source files:** `legacy/vb6/clsActivityMulticomp.cls`, `legacy/pascal/TERMOIII.PAS`

### Phase 9: Mixing Rules *(Milestone 5)*
- Classical (IVDW, IIVDW) with kij (IVDW identical in both programs)
- Wong-Sandler, Huron-Vidal (original + simplified), MHV1, MHV2 (21) (VB6)
- Schmidt-Wenzel C-parameter mixing (4): C = F/E weighted average using acentric factors (from Pascal `TERMOII.PAS:234`)
- Patel-Teja C-parameter mixing (4): two variants -- linear (PatelT) and square-root-weighted (PatelTUSB) (from Pascal `TERMOII.PAS:243`)
- **Key source files:** `legacy/vb6/clsQbicsMulticomp.cls:395`, `legacy/pascal/TERMOII.PAS:211`

### Phase 10: Multicomponent EOS *(Milestone 5)*
- Partial fugacity coefficients in solution for all mixing rules (9) (Müller et al. general expressions, Eqs 2.28–2.34)
- Mixture Z-factor calculation
- 3-parameter EOS fugacity coefficients (4): Schmidt-Wenzel and Patel-Teja partial fugacity with u,w,delta,g auxiliary variables (from Pascal `TERMOII.PAS:317`, significantly more complex than 2-parameter EOS)
- Chao-Seader liquid fugacity for multicomponent mixtures (4) (from Pascal `TERMOII.PAS:386`)
- **Key source files:** `legacy/vb6/clsQbicsMulticomp.cls`, `legacy/pascal/TERMOII.PAS`

### Phase 11: Enthalpy & Entropy *(Milestone 5)*
- Ideal gas Cp integration (polynomial, identical in both programs)
- Departure functions for cubic EOS (9) and virial
- Departure functions for 3-parameter EOS (4): Schmidt-Wenzel (note: marked as discontinuous in Pascal, returns NaN) and Patel-Teja (from Pascal `TERMOII.PAS:471`)
- Analytical dα/dT for ALL 22+ EOS variants — extend Pascal's (4) `Aa_T` (from `TERMOII.PAS:411`, analytical for VdW-Adachi, RKS, RKS-Polar, PR, Patel-Teja) to cover all VB6 models. VB6's numerical 5-point stencil retained only as test oracle — see §D
- Condensation enthalpy via Clausius-Clapeyron (4) (from Pascal `TERMOIII.PAS:283`)
- Liquid residual enthalpy/entropy via condensation + excess properties path (4) (from Pascal `TERMOIII.PAS:294`)
- Reference state handling (LiqSat, VapSat, IdealGas)
- **Key source files:** `legacy/pascal/TERMOII.PAS`, `legacy/pascal/TERMOIII.PAS`

### Phase 12: Flash Calculations *(Milestone 6)*
- Raoult's law initial estimates (4) (identical in both: Ki = Psat_i/P)
- Newton-Raphson solver with Broyden quasi-Newton Jacobian (10) (2n+4 equation system) — replaces VB6's full numerical Jacobian — see §A
- Bubble point (T and P), Dew point (T and P) (4),(14),(17),(20) — same algorithm in both programs (parabolic interpolation on ln(sum), low-pressure and high-pressure paths via Asselineau/Anderson-Prausnitz 2nd stage)
- Isothermal flash with Halley's method for Rachford-Rice (19) — replaces NR on scalar equation — see §F
- Adiabatic flash (nested T-loop with enthalpy balance, identical in both) (4),(17)
- Critical point calculation (15),(16): Heidemann algorithm (VB6) with analytical Helmholtz derivatives for 2-param EOS (see §G) + `ZCriticoMezcla` quick estimate (4) (Pascal `TERMOIV.PAS:136`, iterates on Ac/Bc matching)
- kij regression via Brent's method (4) — replaces golden section (L=0.618034) — see §B
- **Aij regression** (4) for activity model binary parameters (Pascal-only, `TERMOV.PAS:297`):
  - Newton-Raphson with analytical Jacobian for Margules, Van Laar, Wilson
  - Calculates experimental gamma from VLE data (accounting for vapor non-ideality, Poynting, Chao-Seader options)
  - Second derivatives (DGamiDA12, DGamiDA21) computed analytically per model
  - Correlation factor analysis for quality of initial estimates
- **Key source files:** `legacy/vb6/clsLVE.cls`, `legacy/pascal/TERMOIV.PAS`, `legacy/pascal/TERMOV.PAS`, `legacy/pascal/TERMOVI.PAS`

### Phase 13: PyO3 Bindings *(Milestone 7)*
- Expose core types as `#[pyclass]`
- Expose calculation functions as `#[pyfunction]`
- Main `VleEngine` Python class with methods for each calculation type

### Phase 14: Python Wrapper Library *(Milestone 7)*
- High-level `System` class for configuring VLE problems
- Result dataclasses (`FlashResult`, `BubbleResult`, `DewResult`)
- Component database (JSON) with built-in common substances
- Plotting helpers (Pxy, Txy diagrams via matplotlib)
- Convenience API: `system.bubble_point_T()`, `system.flash_isothermal()`, etc.

### Phase 15: Jupyter Notebooks *(Milestone 8)*
Each notebook reproduces specific results from the research paper:
- **01_introduction**: Overview, installation, basic API usage
- **02_pure_component**: PVT behavior, compare EOS variants (including 3-parameter Schmidt-Wenzel and Patel-Teja from Pascal), saturation curves
- **03_activity_models**: Gamma vs composition, excess Gibbs energy plots
- **04_bubble_dew_point**: Reproduce Tables 4.6-4.9 (methanol/water, 2-propanol/water)
- **05_flash_calculations**: Reproduce Tables 4.3-4.4, 4.10 (adiabatic, isothermal)
- **06_critical_points**: Reproduce Tables 4.1-4.2 (4 mixture critical points)
- **07_kij_regression**: Reproduce Tables 4.11-4.12 (CO2/butane)
- **08_aij_regression**: Demonstrate activity model Aij fitting (from Pascal program) -- regress Margules, Van Laar, Wilson parameters from experimental VLE data, compare analytical vs numerical Jacobian convergence

---

## Parameter Reference Document (to be created at `docs/parameters/parameter_reference.md`)

Will document all required parameters organized by category:

| Category | Parameters | Sources |
|----------|-----------|---------|
| **Critical Properties** | Tc, Pc, w (acentric factor), Zc, Vc | Reid, Prausnitz & Poling (4th ed); DIPPR 801 database; NIST WebBook |
| **EOS Family Constants** | K1, K2, K3, OmA, OmB, OmC, h[0..4]; Schmidt-Wenzel Beta; Patel-Teja OmB, Zc | Original papers (Peng-Robinson 1976, Soave 1972, van der Waals 1870, Schmidt & Wenzel 1980, Patel & Teja 1982, etc.) |
| **Chao-Seader Params** | ANor[0..9], Aele[10..14], AH2[0..9], AMe[0..9] | Chao & Seader (1961); hardcoded in Pascal `TERMOII.PAS:646` |
| **Alpha Function Params** | m, n, g coefficients per EOS variant | Stryjek & Vera (1986) for PRSV K1; Mathias & Copeman for MC params |
| **Binary Interaction (kij)** | kij matrix for each component pair | Experimental regression; Knapp et al. (1982); Coutinho et al. |
| **Activity Model Params** | Wilson: Lambda_ij; van Laar: A12,A21; Margules: A12,A21 | DECHEMA VLE Data Collection; Gmehling & Onken |
| **Cp Polynomials** | A, B, C, D coefficients | Reid, Prausnitz & Poling; DIPPR; Yaws |
| **Liquid Volume** | ZRA (Rackett), Thomson params | Spencer & Danner (1972); Thomson et al. |
| **Saturation Pressure** | Tb, Antoine (a1,a2,a3), Riedel/RPM/Muller correlation-specific coefficients | Antoine params: NIST; Riedel params: Reid et al. |
| **Other** | Dipole moment, liquid molar volume at Tb | McClellan (1963); CRC Handbook |

---

## Validation Strategy

Test against all 7 cases from Chapter IV of the research paper:

1. **Critical points** (4 mixtures with PR EOS) - Tables 4.1-4.2
2. **Adiabatic flash** (benzene/cyclohexane/methylcyclohexane/n-hexane) - Tables 4.3-4.4
3. **Bubble point pressure** (methanol/water with van Laar at 298K) - Table 4.6
4. **Dew point temperature** (2-propanol/water with Wilson) - Table 4.7
5. **Dew point pressure** (2-propanol/water with Wilson) - Table 4.8
6. **Bubble point temperature** (4-component with Raoult's law) - Table 4.9
7. **Isothermal flash** (n-heptane/butane with RKS at 300K, 100kPa) - Table 4.10
8. **kij regression** (CO2/butane, kij=0.1357) - Tables 4.11-4.12

All numerical results must match within stated tolerances (<1-5% depending on case).

---

## Notes

- **Documentation translation**: The existing docs are in Spanish. They will be translated **to English** and placed in `docs/en/`. The Spanish originals are in `docs/es/research-paper/`.
- **Independence**: The Rust engine is a standalone crate that can be used without Python. The Python package depends on the engine but adds high-level API, plotting, and component database. The notebooks depend on the Python package.
- **Incremental approach**: Each phase produces testable output. We can validate correctness at each stage before building on it.
- **Merge strategy (Pascal + VB6)**: See `PASCAL_VB6_COMPARISON.md` for the full analysis. Core algorithms (cubic solver, flash calculations, activity models, K-value iteration, Rachford-Rice) are identical in both programs and produce a single implementation. Unique features from each program are added as separate functions/modules in the common library. The Pascal program contributes: Schmidt-Wenzel EOS, Patel-Teja EOS (2 variants), Chao-Seader correlation, Antoine vapor pressure, Aij regression with analytical Jacobians, and analytical dA/dT expressions. The VB6 program contributes the remaining 11 EOS variants, 7 advanced mixing rules, virial equation, and the Heidemann critical point algorithm.
- **Pascal attribution (Ref (4))**: All Rust code derived from `legacy/pascal/` must include a source-level comment citing Reference (4): Da Silva & Báez (1989). This includes: 3-parameter EOS (Schmidt-Wenzel, Patel-Teja), Chao-Seader correlation, Antoine vapor pressure, analytical dα/dT expressions, parabolic interpolation for bubble/dew, Aij regression with analytical Jacobians, and all algorithms originating from TERMOI–TERMOVI. Comment format: `// Ref (4): Da Silva & Báez (1989), origpasprogram/TERMOxx.PAS`
- **Algorithm reference citations**: When implementing an algorithm from a specific paper listed in the Academic References section, cite the reference number in a Rust doc comment at the function or module level. See the Reference-to-Code Mapping table for the full mapping.
