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

*References (23)–(29) were added 2026-07-01 with the performance/algorithm modernization plan ([PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md)); they support the modern flash algorithms (§F, §I–§M below) that supersede the legacy iteration schemes.*

(23) Leibovici, C. F.; Neoschil, J. A New Look at the Rachford-Rice Equation. *Fluid Phase Equilib.* **1992**, *74*, 303–308.

(24) Michelsen, M. L. Calculation of Phase Envelopes and Critical Points for Multicomponent Mixtures. *Fluid Phase Equilib.* **1980**, *4*, 1–10.

(25) Crowe, C. M.; Nishio, M. Convergence Promotion in the Simulation of Chemical Processes — the General Dominant Eigenvalue Method. *AIChE J.* **1975**, *21* (3), 528–533.

(26) Michelsen, M. L.; Mollerup, J. M. *Thermodynamic Models: Fundamentals and Computational Aspects*, 2nd ed.; Tie-Line Publications: Holte, 2007.

(27) Rehner, P.; Bauer, G. Application of Generalized (Hyper-) Dual Numbers in Equation of State Modeling. *Front. Chem. Eng.* **2021**, *3*, 758090.

(28) Michelsen, M. L. State Function Based Flash Specifications. *Fluid Phase Equilib.* **1999**, *158–160*, 617–626.

(29) Wilson, G. M. A Modified Redlich-Kwong Equation of State, Application to General Physical Data Calculations. Paper 15c, 65th National AIChE Meeting, Cleveland, OH, 1969.

*Reference (30) was added 2026-07-05 with the downstream derivative/database release plan ([DERIVATIVE_RELEASE_PLAN.md](DERIVATIVE_RELEASE_PLAN.md)); it is the provenance standard for the bundled component-property data (critical constants, ideal-gas Cp polynomials) added in Phase 19.*

(30) Poling, B. E.; Prausnitz, J. M.; O'Connell, J. P. *The Properties of Gases and Liquids*, 5th ed.; McGraw-Hill: New York, 2001.

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
| (7) Michelsen (1982) Part I | Tangent-plane-distance stability analysis — pre-test before every flash (§I) | `flash/stability.rs` |
| (8) Eubank et al. (1992) | Area method for phase equilibria | Future work (not implemented) |
| (9) Müller et al. (1989) | General multicomponent fugacity coefficients and residual properties from cubic EOS (Eqs 2.28–2.34 of thesis) | `eos/multicomp.rs` |
| (10) Stockfleth & Dohrn (1998) | Numerical compositional derivatives for fugacity — legacy approach; superseded by analytic derivatives + dual-number AD (§L), retained only as a test oracle | Test oracles only |
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
| (23) Leibovici & Neoschil (1992) | Rachford-Rice bracketing window — guaranteed convergence + negative flash (§F) | `flash/isothermal.rs` |
| (24) Michelsen (1980) | Phase-envelope continuation (predictor-corrector through the critical point) (§K) | `flash/envelope.rs` |
| (25) Crowe & Nishio (1975) | GDEM acceleration of successive substitution (§J) | `flash/isothermal.rs` |
| (26) Michelsen & Mollerup (2007) | Generalized EOS core architecture; analytic composition derivatives; flash methodology (§J, §K, §L) | `eos/multicomp.rs`, `flash/*` |
| (27) Rehner & Bauer (2021) | Dual-number automatic differentiation for exotic mixing-rule derivatives (§L) | `mixing/rules.rs` (generic scalar type) |
| (28) Michelsen (1999) | State-function-based flash specifications (PH flash) (§M) | `flash/adiabatic.rs` |
| (29) Wilson (1969) | Wilson K-value correlation for flash/bubble/dew initialization (§I) | `flash/init.rs` |
| (30) Poling, Prausnitz & O'Connell (2001) | Property-data provenance: critical constants cross-check + ideal-gas Cp°/R polynomials for the bundled component DB (Phase 19) | `engine/data/components.json`, `python/src/vle/data/components.json` |

---

## Algorithm Performance Improvements

The modernized Rust code improves on several legacy numerical methods. Each subsection describes the legacy approach, the proposed improvement, and the justification.

> **2026-07-01 update**: sections §I–§M were added (and §F upgraded) as part of the
> performance/algorithm modernization plan — see [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md)
> for the full rationale. They replace the thesis-era flash iteration schemes with the
> modern (largely Michelsen-derived) methodology: stability-tested, acceleration-boosted,
> Newton-finished, with exact derivatives everywhere. The companion engineering work
> (allocation-free hot paths, batch numpy API, benchmarks) lives in the *Performance
> Engineering* section below.

### A. Newton-Raphson Jacobian (`numerics/newton_raphson.rs`)

- **Legacy**: VB6 `clsLVE.cls` `NR_JacobianMatrix` computes the full Jacobian numerically via a 5-point stencil, requiring (m+1) function evaluations per Newton-Raphson iteration for the 2n+4 flash system (10).
- **Improvement**: Broyden's quasi-Newton rank-1 update after the first iteration (1 function evaluation per step). Full numerical Jacobian refresh every K=5 steps or when convergence stalls. This is a standard approach for large nonlinear systems where the Jacobian is expensive to compute.
- **Justification**: For a 10-component mixture (m=24), this reduces evaluations from ~25/step to ~1 for most steps. The 2n+4 system evaluation is the dominant cost in flash calculations.
- **Risk mitigation**: Broyden can diverge if the initial Jacobian is poor. The periodic refresh (every K steps) and automatic fallback to full Jacobian on stalled convergence mitigate this.
- **2026-07 note**: once §L (analytic composition derivatives) lands, full Newton with an exact Jacobian becomes the primary flash driver and Broyden is demoted to the fallback for residuals whose Jacobian is not cheaply available. Implementation fix regardless of role: stop cloning + re-factorizing J every iteration (Sherman–Morrison inverse update, O(n²)/iter — see Performance Engineering §C4 in PERFORMANCE_PROPOSAL.md).

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

### F. Halley's Method for Rachford-Rice, inside the Leibovici–Neoschil Window (`flash/isothermal.rs`)

- **Legacy**: Both VB6 and Pascal use Newton-Raphson on the scalar Rachford-Rice equation f(β) = Σ zi(Ki-1)/(1+β(Ki-1)) = 0 (quadratic convergence) (19).
- **Improvement**: Halley's method (cubic convergence) **solved inside the Leibovici–Neoschil window** β ∈ (1/(1−K_max), 1/(1−K_min)) (23), started at the window midpoint with a bisection safeguard. For the Rachford-Rice equation, f, f', and f'' are all computed from the same summation loop with trivial additional arithmetic per component:
  - f' = -Σ zi(Ki-1)²/(1+β(Ki-1))²
  - f'' = 2Σ zi(Ki-1)³/(1+β(Ki-1))³
- **Justification**: Cubic convergence at negligible extra cost (one additional multiply-accumulate per component in the sum). Inside the L-N window the RR function is monotonic and pole-free, so the safeguarded iteration **cannot diverge** — typically 2–4 iterations to machine precision. Solving over the full window (not just β ∈ [0, 1]) gives *negative flash* for free, which the stability layer (§I) and near-boundary flashes rely on.

### G. Analytical Helmholtz Derivatives for Critical Point (`flash/critical.rs`)

- **Legacy**: VB6 `clsLVE.cls` Heidemann algorithm (16) uses numerical 2nd and 3rd derivatives of Helmholtz free energy via finite differences (many function evaluations per iteration).
- **Improvement**: Analytical 2nd and 3rd derivatives of Helmholtz free energy for standard 2-parameter cubic EOS with classical mixing rules. These are well-known expressions that depend on a, b, and their compositional derivatives. Reserve numerical derivatives only for exotic mixing rules (Wong-Sandler, MHV1, MHV2) where analytical forms are prohibitively complex.
- **Justification**: The Heidemann inner loop repeatedly evaluates these derivatives. Analytical forms eliminate the dominant cost for the common case (classical mixing with PR or RKS). The thesis itself flags the numerical-derivative version as "rather slow" (Ch. IV §4.1).
- **2026-07 note**: these derivatives fall out of the §L derivative architecture (analytic for classical mixing, dual-number AD for exotic rules) — §G no longer needs a bespoke numerical fallback.

### H. Cardano Cubic Solver Robustness (`eos/cubic_solver.rs`)

- **Legacy**: Already optimal (Cardano's analytical closed form). Both programs implement the same discriminant-based approach with trigonometric solution for three real roots.
- **Improvement**: Keep Cardano's method. Add robust handling for near-degenerate cases (discriminant ≈ 0) using the approach from (12) (Poling & Prausnitz) to avoid trivial roots and spurious derivatives. Apply (13) (Gurdensen) for volume root selection in flash contexts (choose correct phase root based on Gibbs energy comparison).
- **Justification**: Near the critical point, the cubic discriminant approaches zero and standard implementations suffer floating-point cancellation. This is a robustness improvement, not a speed improvement.

### I. Stability Analysis + Wilson Initialization (`flash/stability.rs`, `flash/init.rs`)

- **Legacy**: Neither program tests phase stability. Successive substitution runs with trivial-solution *guards* (VB6 `Compara`, `FranctionsEquals`) that detect convergence to identical compositions after the fact, then fall back to a second-stage Newton-Raphson. The thesis flags near-critical trivial solutions as its main robustness weakness (Ch. 2 §2.1.1, §2.3.3) and *describes* Michelsen's stability test without implementing it (Ch. 2 §2.3.2).
- **Improvement**: (1) initialize K-values with the Wilson correlation (29) — needs only Tc, Pc, ω, all in `Component`; (2) run a tangent-plane-distance (TPD) stability analysis (7) before every flash: it answers "is there actually a second phase?" and supplies non-trivial K estimates when there is.
- **Justification**: This is the *structural* fix for the trivial-solution problem — the guards become unnecessary rather than better. Stability analysis is the entry point of every modern flash implementation (26). Cost is a handful of cheap SS iterations on the tangent-plane function; benefit is eliminating an entire class of convergence failures plus wasted flash attempts on single-phase feeds.

### J. GDEM-Accelerated Successive Substitution → Newton Switch (`flash/isothermal.rs`)

- **Legacy**: Plain successive substitution on K-values to composition tolerance (linear convergence; the rate → 1 near the critical point, where SS stalls for hundreds of iterations).
- **Improvement**: Michelsen's composite scheme (19),(26): SS with General Dominant Eigenvalue Method extrapolation (25) every ~5 iterations, then switch to full Newton on ln Kᵢ (analytic Jacobian from §L) once the residual drops below ~10⁻³.
- **Justification**: GDEM turns 200 stalled SS iterations into ~20 essentially for free (it reuses stored residuals). The Newton finish gives quadratic terminal convergence, and switching only when close means Newton always starts inside its basin — each stage covers the other's weakness. This is the canonical structure of every serious modern flash code.

### K. Log-Variable Newton for Bubble/Dew + Phase-Envelope Continuation (`flash/bubble.rs`, `flash/dew.rs`, `flash/envelope.rs`)

- **Legacy**: SS inner loop + parabolic extrapolation on the outer T/P variable (4), with a second-stage NR fallback using numerical Jacobians (14); envelope traversal by differential dP/dT stepping (thesis eq. 2.51–2.52), which cannot pass the critical point.
- **Improvement**: Wilson-initialized SS for 3–5 iterations, then full Newton on the (n+1)-variable system {ln K₁..ln Kₙ, ln T or ln P} with the §L analytic Jacobian. For traversal: Michelsen's phase-envelope continuation (24) — parameterize by the most sensitive variable, predictor-corrector with adaptive step control.
- **Justification**: Log variables keep iterates positive by construction and make the Jacobian well-scaled, removing a class of line-search failures. Envelope continuation walks *through* the critical point smoothly — directly fixing the thesis's flagged near-critical weakness. The thesis two-stage scheme is retained as a test oracle only.

### L. Exact Composition Derivatives: Analytic + Dual-Number AD (`eos/multicomp.rs`, `mixing/rules.rs`)

- **Legacy**: All compositional fugacity derivatives (flash Jacobians, Heidemann–Khalil) computed by finite differences (10), chosen for generality across arbitrary EOS × mixing-rule combinations. The thesis names this as its main speed regret (Ch. IV §4.1).
- **Improvement**: Two-tier exact derivatives. (1) **Analytic ∂ln φ̂ᵢ/∂nⱼ** for cubic EOS + classical/vdW mixing — standard closed forms (26), written once against the generalized (A, B, U, W) mixture core. (2) For exotic mixing rules (Wong-Sandler, MHV1/2): **forward-mode automatic differentiation with dual numbers** (`num-dual` crate (27)) — the rule is written once as a function generic over the scalar type; dual evaluation yields derivatives exact to machine precision at ~2× the cost of one function evaluation.
- **Justification**: Newton flash/bubble/dew/critical-point iterations cost **one** residual evaluation instead of n+1 (finite differences), with no FD truncation noise degrading convergence near the solution — for a 10-component system that is ~10× less work per iteration *and* fewer iterations. AD keeps the legacy's "any model combination" generality without its cost, and eliminates FD step-size tuning entirely. **Sequencing rule: this architecture must exist before any flash code is written** (it is the foundation §I–§K and §G consume) — hence it lands with the mixing rules in Milestone 8.3, not with flash in Milestone 9.

### M. Warm-Started / State-Function PH Flash (`flash/adiabatic.rs`)

- **Legacy**: Nested loops — outer iteration on T (Brent), full isothermal flash from cold at every trial T.
- **Improvement**: Keep the nested structure but warm-start every inner flash with the previous T's converged K-values (1–3 inner iterations instead of a full solve). If benchmarks justify it, upgrade to Michelsen's state-function-based simultaneous Newton on (T, ln K) (28), sharing the §K Jacobian machinery.
- **Justification**: Warm starting is nearly free and cuts the dominant cost (repeated inner flashes) by ~5×. The simultaneous formulation converges quadratically but adds complexity — measure first.

---

## Performance Engineering

Companion engineering tracks to §A–§M, adopted 2026-07-01 — full rationale and audit
evidence in [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md). The language question was
re-examined and settled: **the engine stays in Rust** (identical LLVM codegen to C/C++/
Fortran for this workload; every measured cost is architectural, not linguistic).

- **Benchmarks first (Track E, Milestone 8.2)**: criterion micro/meso benches (α dispatch,
  Z-factor, ln φ, mixture ln φ̂, Rachford-Rice, full flash as they land) + a Python-side
  boundary benchmark. Informational CI job reports deltas. Later: external comparison vs
  `thermo` / CoolProp.
- **Zero-allocation hot path (Track C, Milestone 8.2)**: `solve_real` returns
  `([f64; 3], usize)` (no `Vec` per Z evaluation); a per-(T, P, composition) `EosState`
  struct caches αᵢ, dαᵢ/dT, aᵢ, bᵢ, a_mix, b_mix, A, B, U, W (+ the Wilson Λ and virial
  B matrices) so nothing is computed twice; stack-allocated composition arrays for n ≤ 8;
  Broyden updates its factorization in place.
- **Build profile (Track C, Milestone 8.2)**: `[profile.release]` with `lto = "fat"`,
  `codegen-units = 1` (not `panic = "abort"` — PyO3 needs unwinding); drop the unused
  `ndarray` dependency; wheels stay baseline-portable.
- **Batch numpy API (Track D, Milestone 10)**: array-in/array-out bindings via
  **rust-numpy** (one FFI crossing per array, zero-copy); `Python::allow_threads` +
  **rayon** parallelism over state points; a persistent `System` `#[pyclass]` handle
  (no per-call `Component` reconstruction); warm-start plumbing across batch points.
  This is the layer that makes the library behave like "numpy for thermo".

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

**2026-07-01 re-evaluation (language question re-opened and settled)**: with M0–M8.1 shipped, the choice was re-examined against C++/Fortran, Julia, and GPU offload as part of [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md). Conclusion: **stay in Rust** — every measured cost is architectural (allocations, redundant recomputation, scalar-only FFI, default build flags), not linguistic; Rust's LLVM codegen is equivalent to C++/Fortran for this workload, and a rewrite would discard ~6,000 lines of validated code. Three dependencies join the stack as the plan executes: **`num-dual`** (dual-number AD for exotic mixing-rule derivatives, §L), **rust-numpy** (the `numpy` crate — zero-copy numpy array bindings for the batch API), and **`rayon`** (data-parallel batch kernels). `ndarray` is dropped until the batch API actually needs it.

---

## Units of Measurement Add-On

An **independent `units/` Rust crate** (sibling to `engine/`) plus a Python companion (`python/src/vle/units.py`) provides dimensional-analysis-based unit conversion. The units library works standalone and wraps VLE API boundaries so users can pass inputs in any compatible unit (°C, °F, bar, atm, psi, etc.) while the engine operates on canonical metric units internally.

**Legacy units (confirmed in both VB6 and Pascal codebases)**: T in K, P in kPa/bar, molar energy in kJ/kmol, molar entropy in kJ/(kmol·K), R = 8.31451 kJ/(kmol·K) (VB6) / 8.3144 (Pascal). See `legacy/vb6/McommonFunctions.bas:3` and `legacy/pascal/TERMOI.PAS:13`, with Pascal's explicit units comment at `TERMOII.PAS:62-63`.

**Implementation**:
- Rust: `uom` crate (compile-time dimensional analysis via phantom types, zero runtime cost, 7 SI base dimensions)
- Python: `pint` library (runtime dimensional analysis, NumPy integration)
- Canonical internal units match legacy code exactly so validation data is reusable without conversion

**References**: Bridgman, P.W. *Dimensional Analysis*, Yale University Press, 1922; BIPM, *The International System of Units (SI)*, 9th ed., 2019.

**Detailed design document**: [`docs/en/units/dimensional-analysis.md`](docs/en/units/dimensional-analysis.md) explains the 7 SI base dimensions, dimensional homogeneity principle, conversion strategy, and the Rust phantom-type / Python runtime implementation approach. See also `ROADMAP.md` Milestone 3 for task breakdown.

---

## Deployment Strategy

This repo distributes through **crates.io + PyPI + the example notebooks** only. The multi-user JupyterHub + Docker stack that used to live under `deploy/` has been **moved to a separate private operator repository** (an Ansible role + a gated deploy workflow that deploys to both hub hosts as a hot standby). See `CLAUDE.md` → *Deployment Rules* and [`deploy/README.md`](../deploy/README.md) for the distribution channels.

`deploy/` now contains only:
  - `deploy/README.md` — the distribution channels (crates.io, PyPI, notebooks)
  - `deploy/NOTEBOOKS.md` — host-agnostic guide to running any VLE notebook
  - `deploy/scripts/publish-crate.sh`, `publish-pypi.sh` — manual publish paths

Each non-completed milestone (8–11) that ships a user-facing artifact ends with two steps after validation tests pass: (1) create a milestone notebook following CLAUDE.md *Notebook Conventions*, and (2) update the `deploy/NOTEBOOKS.md` catalogue. Tagging a release auto-publishes to PyPI + crates.io; refreshing the hosted teaching hub is a separate operator step in a private operator repository, not part of the release. Milestone 11 adds a Chapter IV walkthrough notebook.

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
│   ├── benches/                     # criterion benchmarks (Track E): alpha, Z, ln phi, RR, flash
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
│       │   ├── init.rs              # Wilson K-value initialization (29)
│       │   ├── stability.rs         # Tangent-plane-distance stability analysis (7) — §I
│       │   ├── bubble.rs            # Bubble point (T and P) — log-variable Newton (§K)
│       │   ├── dew.rs               # Dew point (T and P) — log-variable Newton (§K)
│       │   ├── envelope.rs          # Phase-envelope continuation (24) — §K
│       │   ├── isothermal.rs        # Isothermal flash — GDEM-SS → Newton (§J), L-N windowed Halley RR (§F)
│       │   ├── adiabatic.rs         # Adiabatic (PH) flash — warm-started nested / state-function (§M)
│       │   ├── critical.rs          # Mixture critical point (Heidemann + Pascal ZCriticoMezcla)
│       │   ├── kij_regression.rs    # Binary interaction parameter fitting (Brent, §B)
│       │   └── aij_regression.rs    # Activity model Aij regression (Levenberg-Marquardt + analytical Jacobian)
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
│   │   ├── db/                      # Component property database (SQLite)
│   │   │   ├── connection.py        # SQLite connection factory, schema init
│   │   │   ├── queries.py           # Lookup, insert, search functions
│   │   │   ├── models.py            # Dataclasses: ComponentRecord, KijRecord, etc.
│   │   │   └── seed.py              # Seeding logic (static SQL + optional thermo)
│   │   ├── cli/
│   │   │   └── main.py              # CLI: vle-db init, seed, validate, show, list, export
│   │   ├── components.py            # Built-in component database
│   │   ├── plotting.py              # Pxy, Txy, phase envelope plots
│   │   └── results.py              # FlashResult, BubbleResult dataclasses
│   └── tests/
│       ├── test_pure_eos.py
│       ├── test_activity.py
│       ├── test_bubble_dew.py
│       ├── test_flash.py
│       └── test_validation.py       # Chapter IV test cases
├── data/                            # Component property database (generated, not in git)
│   └── components.db                # SQLite database (gitignored, built by vle-db init+seed)
│                                      # Schema + seed SQL ships inside the wheel at
│                                      # python/src/vle/db/sql/
├── scripts/                         # Data extraction and validation utilities
│   ├── README.md                    # How to use these scripts
│   ├── extract_component_data.py    # Pull properties from thermo/DIPPR library
│   └── cross_validate_coolprop.py   # Cross-validate thermo vs CoolProp
├── notebooks/
│   ├── 00_component_database.ipynb  # Interactive component DB browser/editor
│   ├── 01_introduction.ipynb
│   ├── 02_pure_component.ipynb
│   ├── 03_activity_models.ipynb
│   ├── 04_bubble_dew_point.ipynb
│   ├── 05_flash_calculations.ipynb
│   ├── 06_critical_points.ipynb
│   ├── 07_kij_regression.ipynb
│   ├── 08_aij_regression.ipynb       # Activity model Aij fitting (from Pascal)
│   └── data/
│       └── experimental/            # Additional experimental data for validation
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

### Phase 2: Project Scaffolding *(Milestone 2)*
- Initialize Rust crate (`engine/`) with Cargo.toml, nalgebra + PyO3 dependencies
- Initialize Python package (`python/`) with pyproject.toml and maturin config
- Define all Rust enums merging both programs: `CubicEos` (22+ variants: 19 VB6 + Schmidt-Wenzel, Patel-Teja, Chao-Seader from Pascal), `ActivityModel` (5, identical in both), `MixingRule` (8 from VB6 + Patel-Teja/Schmidt-Wenzel C-parameter mixing from Pascal), `SatPressureModel` (6: Antoine from Pascal + 5 from VB6)
- Define core structs: `Component` (union of VB6 and Pascal fields, including Pascal's `momentoDip`, `delta`, `vl`), `Mixture`, `Flow`, `Tolerances`, `ReferenceState`
- Verify maturin builds and Python can import the empty module

### Phase 3: Units of Measurement Library *(Milestone 3)*
- Scaffold independent `units/` Rust crate with `uom` dependency — works without the VLE engine
- Built-in gauge pressure support (barg, psig, kPag → absolute kPa). Atmospheric pressure (P_atm) is a **runtime-configurable parameter** stored in the registry — never hardcoded. Default: 101.325 kPa. Configurable via `registry.set_atmospheric_pressure()` (Rust) / `set_atmospheric_pressure()` (Python). See `docs/en/units/dimensional-analysis.md` §3.4.
- Define VLE-specific quantity types (Temperature, Pressure, MolarEnergy, MolarEntropy, MolarVolume, Amount) as aliases for `uom`'s SI types
- Implement extensible runtime `UnitRegistry` supporting user-defined units alongside the compile-time typed API
- Unit string parser and canonical conversion (canonical: K, kPa, kJ/kmol, kJ/(kmol·K), cm³/mol, kmol)
- TOML unit file loader shared between Rust and Python
- Python wrapper `python/src/vle/units.py` around `pint`, exposing `ureg` for user extensions
- **Important**: All VLE calculations use absolute pressure internally. Gauge pressure (barg, psig, kPag) is converted to absolute kPa at the API boundary. See `docs/en/units/dimensional-analysis.md` §3.4 for the full explanation.
- See `docs/en/units/dimensional-analysis.md` for the full design and extension rules

### Phase 4: Component Property Database *(Milestone 4)*
- SQLite database (`data/components.db`) for component properties, binary interaction parameters (kij), activity model parameters (Aij), and experimental VLE data
- Schema: 4 tables — `components` (per-compound properties in canonical units), `kij_params` (model-dependent, symmetric), `activity_params` (asymmetric), `experimental_vle` (P-x-y data)
- Python `vle.db` package for database access (connection, queries, models)
- CLI tool (`python -m vle.cli.main`) for init, seed, validate, export
- Jupyter notebook (`notebooks/00_component_database.ipynb`) for interactive browsing/editing
- Static seed data from DIPPR via `thermo` library for Chapter IV validation compounds (15) and common industrial compounds (~50)
- Optional `thermo` dependency for on-demand seeding of ~70,000 compounds
- All values in canonical units: K, kPa (absolute), cm3/mol, kJ/(kmol·K), g/mol
- See `python/src/vle/db/sql/schema.sql` (bundled with the wheel) for full schema and `docs/en/parameters/parameter_reference.md` for parameter catalog

### Phase 5: CI/CD + Auto-Deploy *(Milestone 5)*
- **Three-workflow GitHub Actions architecture**:
  - `.github/workflows/_build.yml` (reusable) — cibuildwheel matrix over Linux x64 (self-hosted ephemeral), Linux arm64 (`ubuntu-24.04-arm` hosted), macOS arm64 (self-hosted Mac mini M1), Windows (hosted); CPython 3.10+ abi3 wheels; manylinux_2_28 baseline; `pytest {project}/tests` against each built wheel
  - `.github/workflows/ci.yml` — push/PR/dispatch: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, plus the wheel matrix as artifact-only; fork-PR guard (`if: github.event_name != 'pull_request' || …head.repo.full_name == github.repository`) on every self-hosted job
  - `.github/workflows/release.yml` — `v*` tag: PyPI Trusted Publishing (OIDC, no token), crates.io publish (`vle-units` then `vle-thermo`, token loaded from 1Password), GitHub Release with wheels + sdist. (It publishes only — the teaching-hub deploy moved to a separate private operator repository; see *Deployment Strategy*.)
- **First PyO3 module**: `engine/src/py_bindings.rs` exposes `version()` and the four enum types (`CubicEos`, `ActivityModel`, `MixingRule`, `SatPressureModel`) as `#[pyclass(eq, eq_int)]`. From this phase forward the **PyO3 Bindings Rule** (CLAUDE.md) is in force: every later milestone that adds Rust functionality also exposes it via PyO3 in the same commit series.
- **abi3 wheels**: PyO3 `abi3-py310` feature in `engine/Cargo.toml`; one wheel per (OS, arch) covers CPython 3.10+ including unreleased versions. Boundary-crossing overhead is negligible for VLE's workload (Python calls the engine once per `flash(...)`; the heavy work stays in Rust).
- **Secrets via 1Password Service Accounts**: a single GitHub secret (`OP_SERVICE_ACCOUNT_TOKEN`) loads the crates.io token at workflow runtime via `1password/load-secrets-action@v2`. Workflow files commit `op://vault/item/field` paths (locators, not values).
- **Auto-deploy (historical)**: M5 originally shipped a `deploy-sandbox` job that SSHed to two sandbox hosts via a `/usr/local/bin/vle-deploy` force-command wrapper. This was **removed** when the JupyterHub deployment moved to a separate private operator repository; the release pipeline now publishes only, and the hub is deployed from there (see *Deployment Strategy*).
- **Public docs**: `docs/ci.md` (developer overview, ephemerality table, fork-PR guard), `docs/runners/linux-setup.md` (Proxmox LXC + Docker + `myoung34/github-runner` ephemeral), `docs/runners/macos-setup.md` (Mac mini M1 launchd service + toolchain bootstrap).

### Phase 6: Numerics *(Milestone 6)*
- Cardano cubic solver (from `McommonFunctions.bas:324`) (12),(13) — see also §H for robustness improvements
- Gaussian elimination with partial pivoting (from `McommonFunctions.bas:24`) — replaced by nalgebra LU
- Brent's method and Illinois algorithm root finders — replace legacy Regula Falsi (from `clsSatPressureSolver.cls`) — see §C
- Utility functions: `SumFrac`, `Norm`, convergence checks

### Phase 7: Pure Component EOS *(Milestone 7 — split: 7.1 / 7.2 / 7.3 done; OL-family α → 7.4)*

**M7.1 — shipped in v0.3.0** (Claude Opus 4.7, 1M context):
- `family_constants(eos)` for all 22 variants (from `McommonFunctions.bas:273`) (5)
- α(Tr) + **analytical** dα/dTr for **PR1976, RKS1972, RK1949, VdW1870** — the four variants Chapter IV uses
- Z-factor via the existing Cardano cubic solver (2-parameter EOS only)
- Pure-component ln(φ) + dimensionless H^R/RT and S^R/R via the general Abbott integral
- Cleanly errors (not panics) on the 3-parameter EOS at the API surface — α functions for deferred variants panic with an `M7.x deferred` marker pointing at the legacy line.

**M7.2 — shipped in v0.4.0** (Claude Opus 4.8, 1M context — the α-function zoo):
- Ported the 12 self-contained 2-parameter α(Tr) functions from `clsQbicsPure.cls:1719`
  (Berthelot, VdWAda1984, RKSGD1978, RKSL1997, RP1978, PRL1997, VdWVald1989, RKSmn1980,
   RKSATmn1995, PRATmng1997, PRMmn1989, PRSV1986), each with an analytical dα/dTr verified
  against a central-difference oracle.
- New `eos_alpha_ex` / `eos_d_alpha_d_tr_ex` PyO3 bindings thread the per-component
  `Zc` / `m` / `n` / `g` / `K₁` parameters across the FFI (the ω-only `eos_alpha` is a
  strict subset).
- The three OL-family variants (VdWOL1998, RKOL1998, PROL1998) were **re-scoped to M7.4**:
  their α is `Tr·(1 + Σ hₖ·…)` where `SumHk` (`clsQbicsPure.cls:268`) reads the reduced
  saturation pressure, making α a function of the saturation correlation rather than of
  `(Tr, ω)` alone. They land with the M7.4 saturation layer.

**M7.3 — shipped in v0.5.0** (Claude Opus 4.8, 1M context — 3-parameter EOS + Chao-Seader, **Pascal-origin** Ref (4) from `TERMOII.PAS`):
- Schmidt-Wenzel: β(ω) third parameter, piecewise m(Tr) with a **guarded** Tr=1 derivative kink (one-sided analytical dα/dTr → finite entropy, vs the legacy NaN)
- Patel-Teja + Patel-Teja USB: fitted ξc(ω), Ωa/Ωb, dimensionless C=(1−3ξc)Pr/Tr; the USB variant shares the pure-component α (differs only in the M8 mixture C-rule)
- Chao-Seader liquid fugacity (`chao_seader_ln_phi` + `ChaoSeaderSpecies`) with the H₂ / methane special coefficient sets
- Z-factor + fugacity + departure for all three via a **unified general (U, W) cubic form** (U=uP/RT, W=w'(P/RT)²) that reuses the two-parameter algebra; verified to reproduce the legacy Patel-Teja and Schmidt-Wenzel cubics coefficient-for-coefficient
- C-parameter mixing rules (`mixing::c_mix`: mole-fraction, √B-weighted, √A-weighted) ready for M8
- `02c_three_param_eos.ipynb` live (`scripts/build_notebook_m73.py`)

**Key source files:** `legacy/vb6/clsQbicsPure.cls`, `legacy/pascal/TERMOII.PAS`

### Phase 8: Saturation Pressure *(Milestone 7 — Antoine in M7.1, advanced models in M7.4; done)*

**M7.1 — shipped in v0.3.0**:
- **Antoine** correlation (4): `ln(P/Pc) = a1 − a2/(a3+T)` from `legacy/pascal/TERMOI.PAS`
- Analytical `dPsat/dT = Psat · a2 / (a3+T)²`

**M7.4 — shipped in v0.6.0** (Claude Opus 4.8, 1M context — the rest of the saturation layer):
- OL-family α (VdWOL1998, RKOL1998, PROL1998) — `Tr·(1 + SumHk)` with the per-family
  h-tables (`clsQbicsPure.cls:268`); reads the reduced saturation pressure via the new
  `Component.sat_model` field, with an **analytical** dα/dTr (chain rule through dPsat/dT)
- Riedel, Müller, RPM correlations — unit-normalized to kPa (the legacy `ln(Pc/1.0135 bar)`
  reference becomes `ln(Pc/101.325 kPa)`); each reproduces ~1 atm at Tb
- Polynomial (DIPPR-101-style) evaluation `ln P = c0 + c1/T + c2·ln(T) + c3·T^c4`
- `pseudo_antoine` helper + generic `d_psat_dt` (analytical Antoine, central-difference else)
- Boiling-point calculation (closed-form Antoine, Brent for the others)
- Poynting correction `exp[V_L·(P − Psat)/(R·T)]` (canonical kPa units)
- Maxwell equal-area construction — successive substitution on equal fugacity over the cubic isotherm
- 8 new PyO3 bindings + `02d_advanced_saturation.ipynb` live (`scripts/build_notebook_m74.py`)

**Key source files:** `legacy/vb6/clsSatPressureSolver.cls`, `legacy/pascal/TERMOI.PAS`

### Phase 9: Virial Equation *(Milestone 7 — fully shipped in M7.1)*
- Pitzer correlation: `B⁰ = 0.083 − 0.422/T_r^1.6`, `B¹ = 0.139 − 0.172/T_r^4.2`
- Pure-component Z, ln(φ), H^R/RT, S^R/R with analytical `dB/dT`
- Multicomponent: quadratic Lewis-Randall mixing for `B_mix(T, x)`; partial fugacity coefficients `ln(φ̂_i)`
- **Key source files:** `legacy/vb6/clsVirial.cls`, `legacy/vb6/clsVirialMulticomp.cls`

### Phase 10: Activity Coefficient Models *(Milestone 8.1 — shipped in v0.7.0)*
- Ideal, Margules, van Laar, Wilson, Scatchard-Hildebrand (identical in both programs)
- Excess Gibbs energy, excess enthalpy/entropy — implement analytical dGE/dT for ALL models (Pascal (4) has analytical for Wilson; extend to all) — see §E
- Rackett and Thomson (18) liquid molar volume (VB6)
- Temperature-dependent Aij scaling: `VariarLosAij` (4) (from Pascal `TERMOIII.PAS:384`, handles Wilson/VanLaar/Margules differently)
- Wilson binary parameter calculation from infinite-dilution activity coefficients: `CalcParBinWilson` (4) (from Pascal `TERMOIII.PAS:342`, Newton-Raphson)
- **Key source files:** `legacy/vb6/clsActivityMulticomp.cls`, `legacy/pascal/TERMOIII.PAS`

### Phase 11: Performance Foundation *(Milestone 8.2)* — **done**

*Added 2026-07-01 — Tracks C + E of [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md). Pure speed and measurement work; no thermodynamic behavior change, gated by the existing test suite. Shipped: benches in `engine/benches/engine_bench.rs`, FFI benchmark in `scripts/bench_ffi_boundary.py`, `bench-rust` CI job, allocation-free cubic (`([f64;3],usize)`), `EosState`/`WilsonCache` caches, `smallvec` composition buffers, Sherman–Morrison Broyden.*

- criterion benchmark suite (`engine/benches/`): α dispatch, Z-factor, pure ln φ, saturation, activity γ; extended as mixture fugacity / RR / flash land in later phases
- Python-side boundary benchmark (scalar-loop vs future batch) to quantify FFI overhead
- Informational CI bench job (report deltas, non-blocking)
- `[profile.release]`: `lto = "fat"`, `codegen-units = 1` (keep `panic = "unwind"` — PyO3 converts panics to Python exceptions)
- Allocation-free cubic/Z path: `solve_real` → `([f64; 3], usize)`, root selection without filter/collect/sort
- `EosState` cache struct — α, dα/dTr, a, b, A, B, U, W computed once per (T, P, composition) and shared across Z/fugacity/departure calls; Wilson Λ and virial B matrices cached alongside
- Stack-allocated composition arrays for n ≤ 8 components
- Broyden: in-place factorization update (Sherman–Morrison), no per-iteration `clone().lu()`
- Drop the unused `ndarray` dependency

### Phase 12: Mixing Rules + Derivative Core *(Milestone 8.3)* — **done** (`engine/src/mixture.rs`)
- Classical (IVDW, IIVDW) with kij (IVDW identical in both programs)
- Wong-Sandler, Huron-Vidal (original + simplified), MHV1, MHV2 (21) (VB6)
- Schmidt-Wenzel C-parameter mixing (4): C = F/E weighted average using acentric factors (from Pascal `TERMOII.PAS:234`)
- Patel-Teja C-parameter mixing (4): two variants -- linear (PatelT) and square-root-weighted (PatelTUSB) (from Pascal `TERMOII.PAS:243`)
- **Derivative architecture (§L)**: mixing rules written once against the generalized (A, B, U, W) mixture core; classical rules carry hand-derived analytic composition derivatives; exotic rules (WS, MHV1/2) are generic over the scalar type and differentiated with `num-dual` (27) — finite differences retained only as test oracles
- **Key source files:** `legacy/vb6/clsQbicsMulticomp.cls:395`, `legacy/pascal/TERMOII.PAS:211`

### Phase 13: Multicomponent EOS *(Milestone 8.3)* — **done** (`engine/src/mixture.rs`)
- Partial fugacity coefficients in solution for all mixing rules (9) (Müller et al. general expressions, Eqs 2.28–2.34), written once against the generalized (A, B, U, W) mixture form (26)
- Analytic ∂ln φ̂ᵢ/∂nⱼ for cubic EOS + classical mixing (§L) — the Jacobian building block for Phases 15's Newton loops and §G's critical point
- Mixture Z-factor calculation
- 3-parameter EOS fugacity coefficients (4): Schmidt-Wenzel and Patel-Teja partial fugacity with u,w,delta,g auxiliary variables (from Pascal `TERMOII.PAS:317`, significantly more complex than 2-parameter EOS)
- Chao-Seader liquid fugacity for multicomponent mixtures (4) (from Pascal `TERMOII.PAS:386`)
- **Key source files:** `legacy/vb6/clsQbicsMulticomp.cls`, `legacy/pascal/TERMOII.PAS`

### Phase 14: Enthalpy & Entropy *(Milestone 8.4)* — **done** (`engine/src/energy.rs`)
- Ideal gas Cp integration (polynomial, identical in both programs) — `ideal_cp` + enthalpy/entropy integrals from the `Cpᵢ°/R` coefficients
- Departure functions for cubic EOS (9) and virial — generalized-cubic `H^R/(RT)`, `S^R/R` via Lewis-Randall over the mixture `Σ xᵢ ln φ̂ᵢ`
- Departure functions for 3-parameter EOS (4): Patel-Teja finite; Schmidt-Wenzel finite here via the engine's guarded analytic dα/dTr (the legacy Pascal returned NaN — a documented improvement)
- **Analytical** `T·dA_mix/dT` for **every** mixing rule (no 5-point stencil): classical/3-param via per-component `dαᵢ/dT`, GE rules via the exact `T·d(Gᴱ/RT)/dT = −Hᴱ/(RT)` identity. VB6's numerical stencil retained only as the test oracle — see §D
- Reference state handling (ideal-gas reference; LiqSat/VapSat anchoring via the excess-property liquid path lands with the M9 γ-φ flash)
- **Key source files:** `legacy/pascal/TERMOII.PAS`, `legacy/pascal/TERMOIII.PAS`
- *Deferred:* condensation enthalpy via Clausius-Clapeyron (Pascal `TERMOIII.PAS:283`) and the full liquid residual H/S condensation+excess path (`TERMOIII.PAS:294`) — originally slated for M9's γ-φ flash but never packaged (M9's adiabatic flash shipped φ-φ/cubic-only enthalpy). **Now scheduled as the packaged γ-φ `phase_enthalpy_entropy` in Phase 19 (Milestone 12.4)** — see [DERIVATIVE_RELEASE_PLAN.md](DERIVATIVE_RELEASE_PLAN.md)

### Phase 15: Flash Calculations *(Milestone 9)* — **done** (`engine/src/flash/`; notebooks 04–08 shipped)

*Rewritten 2026-07-01 — Track A of [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md). The thesis-era iteration schemes are replaced by the modern (Michelsen-derived) methodology; the legacy two-stage bubble/dew scheme is retained only as a test oracle. All Newton loops consume the Phase 12–13 analytic/AD Jacobians (§L).*

*Progress (Claude Fable 5): almost complete. Shipped — Wilson K-init (`init.rs`), Rachford-Rice via Halley in the Leibovici–Neoschil window (`isothermal.rs`, §F), TPD stability (`stability.rs`, §I), GDEM-accelerated SS isothermal flash (`isothermal.rs`, §J), the φ-φ / γ-φ K-value dispatch (`system.rs`), bubble/dew (T and P) (`bubble.rs`, `dew.rs`, shared `incipient.rs`, §K), adiabatic flash (`adiabatic.rs`, §M, warm-started), the mixture critical point (`critical.rs`, §G — Heidemann via dual-number Helmholtz derivatives + a 2-D Newton), and kij/Aij regression (`kij_regression.rs` with `brent_minimize`, `aij_regression.rs` with Levenberg-Marquardt). All exposed through PyO3 with Rust + Python tests; the criterion suite gained RR + flash benches; Chapter IV §4.6 (isothermal flash) is reproduced against the published table and §4.7 (kij) is validated on the sub-critical subset (`engine/tests/chapter_iv_validation.rs`). Phase-envelope continuation (24, §K) shipped too (`envelope.rs` — a unified incipient-phase (n+2)-variable Newton with min-Gibbs root selection that walks through the critical point). **Milestone 9 is complete: every Phase 15 algorithm is implemented, bound, and tested, the Chapter IV cases are validated against the published tables, and notebooks 04–08 (built by `scripts/build_notebook_m9_*.py`, executing top-to-bottom) reproduce the thesis results.***

- Wilson K-value correlation initial estimates (29) (§I) — supersedes Raoult-only initialization; Raoult retained for the γ-φ path
- Tangent-plane-distance stability analysis (7) (§I) before every flash — supplies "is there a second phase?" + non-trivial K estimates; structurally eliminates the thesis's trivial-solution failure mode
- Isothermal flash (19),(26): GDEM-accelerated successive substitution (25) → Newton on ln Kᵢ with analytic Jacobian (§J); Rachford-Rice inner solve via Halley inside the Leibovici–Neoschil window (23) with bisection safeguard (§F) — guaranteed convergence + negative flash
- Bubble point (T and P), Dew point (T and P) (§K): Wilson-initialized SS warm-up, then full Newton on {ln K, ln T or ln P} with analytic Jacobian; legacy parabolic/Asselineau two-stage scheme (4),(14),(17),(20) kept as a validation oracle
- Phase-envelope continuation (24) (§K): predictor-corrector traversal through the critical point — replaces the thesis's differential dP/dT stepping
- Adiabatic (PH) flash (§M): warm-started nested T-loop (Brent outer, K-seeded inner flash); optional upgrade to simultaneous state-function Newton (28) if benches justify
- Critical point calculation (15),(16): Heidemann algorithm with analytical Helmholtz derivatives from the §L core (see §G) + `ZCriticoMezcla` quick estimate (4) (Pascal `TERMOIV.PAS:136`, iterates on Ac/Bc matching)
- kij regression via Brent's method (4) — replaces golden section (L=0.618034) — see §B; warm-start each data point's bubble-P solve from its neighbor
- **Aij regression** (4) for activity model binary parameters (Pascal-only, `TERMOV.PAS:297`):
  - Levenberg-Marquardt with the analytical Jacobian for Margules, Van Laar, Wilson (supersedes plain Newton-Raphson — same per-iteration cost, graceful far-from-optimum behavior)
  - Calculates experimental gamma from VLE data (accounting for vapor non-ideality, Poynting, Chao-Seader options)
  - Second derivatives (DGamiDA12, DGamiDA21) computed analytically per model
  - Correlation factor analysis for quality of initial estimates
- Extend the criterion bench suite (Phase 11) with RR, isothermal flash, bubble/dew, and envelope benches
- **Key source files:** `legacy/vb6/clsLVE.cls`, `legacy/pascal/TERMOIV.PAS`, `legacy/pascal/TERMOV.PAS`, `legacy/pascal/TERMOVI.PAS`

### Phase 16: PyO3 Bindings *(Milestone 10)*
- Expose core types as `#[pyclass]`
- Expose calculation functions as `#[pyfunction]`
- Main `VleEngine` Python class with methods for each calculation type

### Phase 17: Python Wrapper Library + Batch API *(Milestone 10)*
- High-level `System` class for configuring VLE problems — backed by a persistent `#[pyclass]` handle holding components, model selections, and cached T-independent data (no per-call `Component` reconstruction)
- **Batch numpy API (Track D)**: array-in/array-out entry points for every property and flash via rust-numpy (zero-copy, one FFI crossing per array); `Python::allow_threads` + rayon parallelism over state points; warm-start plumbing across batch points (each flash seeded from its neighbor's converged K); scalar convenience methods become batch-of-one
- Result dataclasses (`FlashResult`, `BubbleResult`, `DewResult`) + batch result arrays
- Component database (JSON) with built-in common substances
- Plotting helpers (Pxy, Txy diagrams via matplotlib) — driven by the batch API
- Convenience API: `system.bubble_point_T()`, `system.flash_isothermal()`, etc.
- Boundary benchmark rerun (Phase 11 baseline vs batch API) + external comparison benches vs `thermo` / CoolProp

### Phase 18: Chapter IV Walkthrough *(Milestone 11)*

Notebooks 01–08 are produced by the milestone that builds the underlying feature (see table below), each following CLAUDE.md *Notebook Conventions*. Phase 18 is the capstone that adds the Chapter IV walkthrough notebook:

- **10_chapter4_validation_walkthrough**: Single end-to-end notebook that narrates every section of [`chapter-4-validation.md`](docs/en/research-paper/chapter-4-validation.md). For each of §4.1–§4.7 it quotes the research-paper text, runs the `vle` library against the referenced table (4.1–4.12), reports absolute and percent error against published values, and presents ≥2 user exercises (e.g. "repeat the kij regression for a different binary pair").

Refreshing the hosted teaching hub after a release remains a separate, optional operator step in a private operator repository, not part of this milestone — see `CLAUDE.md` → *Deployment Rules*.

**Notebook-to-milestone map (produced incrementally through Milestones 4–9):**

| Notebook                                    | Authored in  | Covers                                       |
|---------------------------------------------|--------------|----------------------------------------------|
| `00_component_database.ipynb`               | Milestone 4  | SQLite DB browsing, Chapter IV compounds     |
| `m06_numerics.ipynb`                        | Milestone 6  | Brent / Illinois / Broyden / Halley demos    |
| `01_introduction.ipynb`                     | Milestone 10 | Install + `vle.System` API tour              |
| `02_pure_component.ipynb`                   | Milestone 7  | PVT, EOS variant comparison, saturation      |
| `03_activity_models.ipynb`                  | Milestone 8  | Gamma plots, excess Gibbs, mixing rules      |
| `04_bubble_dew_point.ipynb`                 | Milestone 9  | Tables 4.6–4.9                               |
| `05_flash_calculations.ipynb`               | Milestone 9  | Tables 4.3–4.4, 4.10                         |
| `06_critical_points.ipynb`                  | Milestone 9  | Tables 4.1–4.2                               |
| `07_kij_regression.ipynb`                   | Milestone 9  | Tables 4.11–4.12                             |
| `08_aij_regression.ipynb`                   | Milestone 9  | Aij fitting (Pascal-origin)                  |
| `10_chapter4_validation_walkthrough.ipynb`  | Milestone 11 | End-to-end Chapter IV walkthrough            |

### Phase 19: Downstream Derivative & Database Release *(Milestone 12)* — **in progress** (M12.1 done)

*Added 2026-07-05. Full technical spec, current-state audit (with `path:line`
anchors), design decisions, breaking-change register, and risks:
[DERIVATIVE_RELEASE_PLAN.md](DERIVATIVE_RELEASE_PLAN.md). Prepared by Claude
Code using Claude Fable 5 for execution by Claude Opus 4.8.*

**Status (2026-07-05):** Milestone 12.1 (component DB → 24 compounds + bundled
ideal-gas Cp°/R coefficients, threaded Python → engine) is **complete** and
bumped to **v0.8.2**. Milestones 12.2–12.5 (Rust DB, T/P derivatives, real Cp /
partial molar enthalpy / γ-φ enthalpy, notebook + v0.9.0) remain planned.

The first downstream consumer of the published `vle-thermo` crate/wheel — the
planned `stages-thermo` staged-separation (distillation) library — audited the
0.8.1 public API against what a rigorous column solver (MESH / Naphtali–Sandholm
Newton with an exact block-tridiagonal Jacobian) needs per stage evaluation, and
found five gaps. This phase closes them by **extending the §L exact-derivative
architecture from composition to temperature and pressure**, and by making the
bundled property data reachable from Rust:

- **Component DB expansion (Milestone 12.1 → v0.8.2, fast-tracked)**: 15 → 24
  bundled compounds (toluene, ethanol, acetone, chloroform, i-C4, i-C5, n-C8,
  n-C9, n-C10 — the classic distillation/absorber validation set); every
  compound gains `cp_coeffs` (dimensionless Cp°/R polynomial matching
  `energy::ideal_cp`), `cp_t_range`, `cp_source` (30); the Python
  `vle.components.Component` dataclass and `vle.System` thread `cp_coeffs`
  through to the engine pyclass (fixing the silent zero-ideal-Cp defect for
  DB-built systems); `vle-db` static seed grows to the same 24.
- **Rust-side component database (Milestone 12.2)**: `engine/src/db.rs` behind a
  new `component-db` cargo feature — vendored `engine/data/components.json`
  (`include_str!`, one generator script for all copies, drift-guarded),
  `component(name)` / `available()` with Python-parity name normalization.
- **T/P derivatives of fugacity and K-values (Milestone 12.3)**: the generic
  mixture core (`mixture_params<D>`, `ln_phi_all_generic<D>`, activity
  `ge_terms`) becomes generic in `t: D, p: D` (breaking signature change →
  0.9.0); new `d_ln_phi_d_t` / `d_ln_phi_d_p` with the §L two-branch dispatch
  (analytic for classical + 2-parameter EOS via the existing `d_alpha_d_tr` /
  `t_dln_a_dt_mix` machinery and implicit dZ; single-seed dual-number AD (27)
  for WS/HV/MHV/3-parameter); `k_values_with_derivs` packaging
  {K, ∂lnK/∂T, ∂lnK/∂P} for both the φ-φ and γ-φ paths. Invariant tests:
  Gibbs–Helmholtz `Σxᵢ·∂lnφ̂ᵢ/∂T = −H^R/(RT²)`, volumetric
  `Σxᵢ·∂lnφ̂ᵢ/∂P = (Z−1)/P`, plus `_fd` oracles (FD never ships in production
  paths).
- **Enthalpy derivatives + packaged γ-φ enthalpy (Milestone 12.4)**:
  `partial_molar_enthalpy` via the identity `H̄ᵢ = h°ᵢ(T) − RT²·∂lnφ̂ᵢ/∂T`;
  `phase_cp` (ideal + Cp^R via second-order duals through the T-generic core);
  a `SystemSpec`-level `phase_enthalpy_entropy` whose γ-φ liquid branch finally
  packages the Phase 14 deferred Clausius-Clapeyron condensation + excess path
  (4) (`TERMOIII.PAS:283/294`). Euler-sum, FD-oracle, and ideal-gas-limit tests.
- **Notebook + release (Milestone 12.5)**: milestone notebook
  `11_derivatives_and_database.ipynb`, criterion benches for the new surface,
  full doc sync, version **0.9.0**.

Every new public function ships PyO3 bindings + wheel tests in the same commit
series (M5+ rule), with units in every docstring. Deliberately out of scope:
γ-φ adiabatic flash (enabled by 12.4, not needed downstream yet), ∂K/∂x
packaging (already available via `d_ln_phi_d_n`), DB growth beyond the 24.

**Key source files:** `engine/src/mixture.rs`, `engine/src/energy.rs`,
`engine/src/flash/system.rs`, `engine/src/types.rs`,
`scripts/build_components_json.py`, `python/src/vle/components.py`,
`legacy/pascal/TERMOIII.PAS` (4)

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
