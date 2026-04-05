# VLE — Vapor-Liquid Equilibrium Calculator

A modern Rust + Python reimplementation of a multicomponent vapor-liquid equilibrium (VLE) thermodynamic calculator, built from two legacy academic codebases using AI-assisted development.

## About This Project

This project modernizes legacy thermodynamic software — originally written in VB6 (1999) and Pascal (1989) — into a fast **Rust computation engine** with **Python bindings** (via PyO3) and **Jupyter notebooks** for interactive exploration.

**This is an educational project** demonstrating how AI coding tools like [Claude Code](https://claude.ai/code) can be used to understand, analyze, and modernize legacy scientific code. The entire modernization process — from analyzing ~17,500 lines of VB6/Pascal, mapping algorithms to academic references, proposing performance improvements, and planning the new architecture — was conducted with Claude Code as a development partner.

### Original Research

This work is based on the thesis:

> **"Desarrollo de un Programa Computacional para el Cálculo del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente Windows"**
>
> *Miguel Roberto Jackson Ugueto* and *Carlos Fernando Mendible Porras*
>
> Proyecto de Grado, Universidad Simón Bolívar, Sartenejas, April 1999
>
> Advisors: Prof. Coray M. Colina and Prof. Jean-Marie Ledanois

The VB6 program in turn builds upon an earlier Pascal package:

> **(4)** Da Silva, F. A.; Báez, L. Desarrollo de un Paquete Computacional para la Predicción de Propiedades Termodinámicas y de Equilibrio de Fases. Thesis, Universidad Simón Bolívar, 1989.

The full research paper is available in both [English](docs/en/research-paper/README.md) and [Spanish](docs/es/research-paper/) in the `docs/` directory.

## Features

### Thermodynamic Models
- **22+ cubic equations of state**: Peng-Robinson, RKS, van der Waals, Schmidt-Wenzel, Patel-Teja, and 17 more variants
- **Chao-Seader** liquid fugacity correlation (with special H2/methane handling)
- **Second virial equation** (Pitzer/Tsonopoulos correlation)
- **5 activity coefficient models**: Wilson, van Laar, Margules, Scatchard-Hildebrand, Ideal
- **8 mixing rules**: Classical (IVDW, IIVDW), Wong-Sandler, Huron-Vidal, MHV1, MHV2, and more

### Calculations
- Bubble point (temperature and pressure)
- Dew point (temperature and pressure)
- Isothermal flash (Rachford-Rice)
- Adiabatic flash
- Mixture critical point (Heidemann-Khalil algorithm)
- Binary interaction parameter regression (kij)
- Activity model parameter regression (Aij) with analytical Jacobians
- Saturation pressure (Antoine, Riedel, Muller, RPM)
- Residual and excess thermodynamic properties (H, S, G)

### Algorithm Improvements Over Legacy Code
The modernization introduces several numerical improvements over the original VB6/Pascal implementations:

| Algorithm | Legacy | Modernized | Benefit |
|-----------|--------|-----------|---------|
| NR Jacobian | Full numerical (m+1 evals/step) | Broyden quasi-Newton (1 eval/step) | ~25x fewer evaluations |
| kij optimization | Golden section (linear) | Brent's method (superlinear) | ~2x fewer iterations |
| Root finding | Regula Falsi (can stall) | Illinois / Brent's | No stalling, superlinear |
| dα/dT derivatives | 5-point numerical stencil | Analytical for all 22+ EOS | Eliminates 4 evals/call |
| Excess enthalpy | Numerical dGE/dT | Analytical for all 5 models | No cancellation errors |
| Rachford-Rice | Newton-Raphson (quadratic) | Halley's method (cubic) | Faster convergence |
| Critical point | Numerical Helmholtz derivatives | Analytical (2-param EOS) | Dominant cost eliminated |

See [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md) for full details and justifications.

## Project Structure

```
vle/
├── engine/                  # Rust computation engine (PyO3 bindings)
├── python/                  # Python wrapper library
├── notebooks/               # Jupyter notebooks (reproduce thesis results)
├── docs/
│   ├── en/research-paper/   # English translation (navigatable)
│   └── es/research-paper/   # Spanish original
├── legacy/
│   ├── vb6/                 # Original VB6 source (reference)
│   └── pascal/              # Original Pascal source (reference) (4)
├── MODERNIZATION_PLAN.md    # Detailed implementation plan
├── PASCAL_VB6_COMPARISON.md # Legacy codebase comparison
└── CLAUDE.md                # Claude Code development guidance
```

## Getting Started

> **Status**: This project is in the planning/early development phase. The Rust engine and Python bindings are not yet implemented.

### Prerequisites (planned)
- Rust 1.75+ with cargo
- Python 3.10+
- maturin (for building PyO3 bindings)

### Build (planned)
```bash
cd engine && cargo build --release    # Build Rust engine
cd python && maturin develop          # Build Python bindings
```

## Documentation

- [Modernization Plan](MODERNIZATION_PLAN.md) — Full technical plan with academic references, algorithm mapping, and performance improvements
- [Pascal vs VB6 Comparison](PASCAL_VB6_COMPARISON.md) — Detailed comparison of the two legacy codebases
- [Research Paper (English)](docs/en/research-paper/README.md) — Navigatable English translation
- [Research Paper (Spanish)](docs/es/research-paper/) — Original Spanish text

## Academic References

The implementation cites 22 academic references (ACS format). Key ones include:

- **(4)** Da Silva, F. A.; Báez, L. Thesis, Universidad Simón Bolívar, 1989. — Pascal codebase origin
- **(5)** Abbott, M. M. In *Equations of State in Engineering and Research*; ACS, 1979. — General cubic EOS form
- **(9)** Müller, E.; Olivera Fuentes, C.; Estévez, L. *Lat. Am. Appl. Res.* **1989**, *19* (2), 99. — Multicomponent fugacity
- **(16)** Heidemann, R. A.; Khalil, A. M. *AIChE J.* **1980**, *26* (5), 769. — Critical point algorithm
- **(19)** Michelsen, M. L. *Fluid Phase Equilib.* **1982**, *9*, 21. — Rachford-Rice / phase split
- **(21)** Orbey, H.; Sandler, S. I. Cambridge University Press, 1998. — Advanced mixing rules

The full reference list and code mapping is in [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md#academic-references).

## Built with Claude Code

**This entire project was built with [Claude Code](https://claude.ai/code)**, Anthropic's CLI coding agent. Every aspect — from analyzing 20,000 lines of legacy code to writing this README — was done collaboratively between a human developer and an AI agent working directly in the terminal.

### What the agent did

In a single ~3-hour session, Claude Code:

1. **Read and understood ~20,000 lines of legacy code** across two languages (VB6 and Pascal), with Spanish variable names and no inline documentation
2. **Compared both codebases** function-by-function, identifying identical, overlapping, and unique algorithms
3. **Mapped every algorithm to its academic paper** — tracing 22 references from a Spanish research thesis to specific functions in the source code
4. **Designed the modernization architecture** — Rust + PyO3 + Python with enum-driven dispatch matching the legacy Select Case / case patterns
5. **Proposed 8 algorithm improvements** with detailed justifications (e.g., Broyden quasi-Newton replacing full numerical Jacobians for ~25x fewer evaluations)
6. **Restructured the repository**, created 13 interlinked English documentation files with navigatable cross-references, and formatted all citations to ACS scientific style
7. **Generated a complete project plan** with 70+ tasks across 9 milestones

### Time comparison

| | With Claude Code | Without AI (estimated) |
|---|---|---|
| **Foundation work done so far** | ~3 hours | ~15–22 working days (3–4.5 weeks) |
| **Full project estimate** | ~150–200 hours (~4–6 weeks) | ~6,000–12,000 hours (~3–6 person-years) |
| **Speedup factor** | **~40–60x** | baseline |

The biggest time savings come from code comprehension (the agent processes thousands of lines in seconds), cross-referencing (mapping algorithms across two codebases, a research paper, and 22 academic references simultaneously), and bulk transformations (reformatting citations, renaming paths, and creating interlinked documents across 20+ files).

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

The original research paper content and legacy source code are included for educational and reference purposes.

## Authors & Contributors

### Modernization
- **Miguel Roberto Jackson Ugueto** ([@miguelju](https://github.com/miguelju)) — Main developer. Co-author of the original VB6 thesis (1999), leading the Rust + Python modernization.
- **Carlos Fernando Mendible Porras** ([@cmendible](https://github.com/cmendible)) — Co-author of the original VB6 thesis (1999). Carlos was instrumental in the design and development of the original thermodynamic library.

### Original Pascal Program (1989)
- **Francisco Avelino Da Silva** — Co-author of the Pascal codebase (4)
- **Luis Alberto Báez Linde** — Co-author of the Pascal codebase (4)

### Thesis Advisors (1999)
- Prof. Coray M. Colina, Universidad Simón Bolívar
- Prof. Jean-Marie Ledanois, Universidad Simón Bolívar
