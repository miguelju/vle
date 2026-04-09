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

### Units of Measurement (Independent Add-On)
- **Dimensional analysis** via the 7 SI base dimensions (L, M, T, I, Θ, N, J)
- **Rust**: `uom` crate for compile-time dimension checking (zero runtime cost)
- **Python**: `pint` library for runtime unit conversion
- Supports temperature (K, °C, °F, °R), pressure (kPa, bar, atm, psi, mmHg, torr), energy (kJ/kmol, J/mol, cal/mol, BTU/lbmol), and more
- Works standalone — can be used in other projects

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
├── data/                    # Component property database
│   ├── schema.sql           # SQLite schema (version-controlled)
│   ├── seed_chapter4.sql    # Chapter IV validation data (version-controlled)
│   └── components.db        # SQLite database (generated, gitignored)
├── scripts/                 # Data extraction utilities (see scripts/README.md)
├── python/src/vle/          # Python package
│   ├── db/                  # Component database (connection, queries, models)
│   └── cli/                 # CLI tool (vle-db)
├── notebooks/               # Jupyter notebooks
│   └── 00_component_database.ipynb  # Interactive component browser
├── units/                   # Independent units crate (planned — dimensional analysis)
├── engine/                  # Rust computation engine (scaffolded — enums, structs, PyO3 bindings)
├── docs/
│   ├── en/research-paper/   # English translation (navigatable)
│   ├── en/units/            # Units add-on design document
│   └── es/research-paper/   # Spanish original
├── legacy/
│   ├── vb6/                 # Original VB6 source (~15,000 lines, reference)
│   └── pascal/              # Original Pascal source (~2,500 lines, reference) (4)
├── ROADMAP.md               # Milestones and progress tracking
├── TODO.md                  # Tasks with time estimates
├── MODERNIZATION_PLAN.md    # 16-phase implementation plan
├── PASCAL_VB6_COMPARISON.md # Legacy codebase comparison
└── CLAUDE.md                # Claude Code development guidance and conventions
```

## Development Workflow

This project is developed incrementally using [Claude Code](https://claude.ai/code) as an AI development partner. Each milestone follows a **plan-then-execute** cycle, tracked across three synchronized documents:

| Document | Purpose |
|----------|---------|
| [`ROADMAP.md`](ROADMAP.md) | Milestones — high-level goals and deliverables |
| [`TODO.md`](TODO.md) | Tasks — actionable items with time estimates per milestone |
| [`MODERNIZATION_PLAN.md`](MODERNIZATION_PLAN.md) | Phases — detailed technical implementation plan (16 phases) |

### Resuming work from a new machine

```bash
# 1. Clone the repository
git clone <repo-url> && cd vle

# 2. Review current progress
cat ROADMAP.md          # Which milestones are done?
cat TODO.md             # Which tasks remain?

# 3. Initialize the component database (generated, not in git)
pip install thermo                                          # optional, for extended seeding
PYTHONPATH=python/src python -m vle.cli.main init
PYTHONPATH=python/src python -m vle.cli.main seed --source chapter4
PYTHONPATH=python/src python -m vle.cli.main validate chapter4

# 4. Start a Claude Code session and continue the next milestone
claude
```

### How a milestone is executed

1. **Plan** — Claude Code reads the relevant legacy code, documentation, and academic references, then proposes an implementation plan.
2. **Review** — The developer reviews the plan, asks questions, and requests adjustments.
3. **Execute** — Claude Code implements the plan incrementally (code, tests, documentation).
4. **Validate** — Results are verified against Chapter IV test cases (8 validation systems from the thesis).
5. **Commit** — All documentation (`ROADMAP.md`, `TODO.md`, `MODERNIZATION_PLAN.md`) is updated to reflect the current state before pushing.

Each milestone records which AI model was used (e.g., `Claude Opus 4.6 (1M context)`) in the commit and documentation for reproducibility tracking.

### Project conventions

- All code, documentation, and project management follow the rules in [`CLAUDE.md`](CLAUDE.md).
- Every function that accepts or returns a physical quantity documents the units in its doc comment.
- All internal calculations use **absolute** pressure in **kPa** — never gauge pressure.
- Phase numbering in `MODERNIZATION_PLAN.md` always matches milestone execution order.

## Getting Started

> **Status**: Milestones 0–2 complete (documentation, translation, dev environment & scaffolding). Milestone 4 (component database) in progress. Milestones 3, 5–10 (units, numerics, models, flash, bindings, notebooks) not yet started.

### Prerequisites
- Python 3.10+ (for the component database and future Python wrapper)
- Rust 1.75+ with cargo (for the future engine build)
- maturin (for building PyO3 bindings)

### Component Database (available now)
```bash
PYTHONPATH=python/src python -m vle.cli.main init               # Create database
PYTHONPATH=python/src python -m vle.cli.main seed --source chapter4  # Seed 15 compounds
PYTHONPATH=python/src python -m vle.cli.main list               # Browse components
PYTHONPATH=python/src python -m vle.cli.main show methane       # View details
```

### Build
```bash
conda activate vle                    # Activate conda environment
cd engine && cargo build --release    # Build Rust engine
cd python && maturin develop          # Build Python bindings
python -c "import vle; print(vle.__version__)"  # Verify
```

## Documentation

- [Developer Setup Guide](docs/en/SETUP.md) — Prerequisites, build instructions, and development workflow (Rust, Python/conda, maturin)
- [Dimensional Analysis](docs/en/units/dimensional-analysis.md) — Units add-on design: SI dimensions, gauge pressure, extensible unit registry
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
