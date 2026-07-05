# VLE — Vapor-Liquid Equilibrium Calculator

A modern Rust + Python reimplementation of a multicomponent vapor-liquid equilibrium (VLE) thermodynamic calculator, built from two legacy academic codebases using AI-assisted development.

![3-D phase surfaces computed by the vle-thermo engine: the methane/ethane phase-envelope dome with its critical locus, and the methanol/water P–x–y sail](docs/assets/phase_surfaces_hero.png)

*Every point above was computed by this library's Rust engine — the **phase-envelope dome** (left) traced through each mixture's critical point by Michelsen continuation, with the critical locus from the Heidemann–Khalil solver riding the ridge, and the **P–x–y "sail"** (right) from the γ-φ bubble/dew solvers. Explore and regenerate them in [`notebooks/09_3d_phase_surfaces.ipynb`](notebooks/09_3d_phase_surfaces.ipynb).*

## About This Project

This project modernizes legacy thermodynamic software — originally written in VB6 (1999) and Pascal (1989) — into a fast **Rust computation engine** with **Python bindings** (via PyO3) and **Jupyter notebooks** for interactive exploration.

**This is an educational project** demonstrating how AI coding tools like [Claude Code](https://claude.ai/code) can be used to understand, analyze, and modernize legacy scientific code. The entire modernization process — from analyzing ~17,500 lines of VB6/Pascal, mapping algorithms to academic references, proposing performance improvements, and planning the new architecture, through implementing and validating the full Rust engine and its Python bindings — was conducted with Claude Code as a development partner.

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

The full research paper is available in both [English](docs/en/research-paper/README.md) and [Spanish](docs/es/research-paper/README.md) in the `docs/` directory.

## Features

### Thermodynamic Models
- **22+ cubic equations of state**: Peng-Robinson, RKS, van der Waals, Schmidt-Wenzel, Patel-Teja, and 17 more variants
- **Chao-Seader** liquid fugacity correlation (with special H2/methane handling)
- **Second virial equation** (Pitzer/Tsonopoulos correlation)
- **5 activity coefficient models**: Wilson, van Laar, Margules, Scatchard-Hildebrand, Ideal
- **11 mixing rules**: Classical (IVDW, IIVDW), Wong-Sandler, Huron-Vidal (original + simplified), MHV1, MHV2, plus 3 C-parameter rules for the three-parameter EOS — all with **exact composition derivatives** (analytic or dual-number AD, never finite differences)

### Calculations
- Bubble point (temperature and pressure)
- Dew point (temperature and pressure)
- Isothermal flash (Rachford-Rice via Halley in the Leibovici–Neoschil window; tangent-plane stability analysis)
- Adiabatic flash
- Phase-envelope continuation through the critical point (Michelsen)
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
The modernization introduces several numerical improvements over the original VB6/Pascal implementations. This table is the **original improvement plan proposed by Claude Opus 4.6** during the initial legacy-code analysis (Milestone 0):

| Algorithm | Legacy | Modernized | Benefit |
|-----------|--------|-----------|---------|
| NR Jacobian | Full numerical (m+1 evals/step) | Broyden quasi-Newton (1 eval/step) | ~25x fewer evaluations |
| kij optimization | Golden section (linear) | Brent's method (superlinear) | ~2x fewer iterations |
| Root finding | Regula Falsi (can stall) | Illinois / Brent's | No stalling, superlinear |
| dα/dT derivatives | 5-point numerical stencil | Analytical for all 22+ EOS | Eliminates 4 evals/call |
| Excess enthalpy | Numerical dGE/dT | Analytical for all 5 models | No cancellation errors |
| Rachford-Rice | Newton-Raphson (quadratic) | Halley's method (cubic) | Faster convergence |
| Critical point | Numerical Helmholtz derivatives | Analytical (2-param EOS) | Dominant cost eliminated |

The plan was later expanded into [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md) — the "numpy for thermo" strategy adding the **exact-derivative mixture core** (analytic + `num-dual` dual-number AD, replacing every finite difference), the modern Michelsen flash suite (stability analysis, windowed Halley Rachford-Rice, phase-envelope continuation), a measured performance foundation, and the upcoming batch numpy API — and implemented, largely by Claude Fable 5, in Milestones 8.2–9. In the final architecture the exact analytic/AD Jacobians go further than the Broyden row above: full Newton uses them directly, with Broyden demoted to a fallback. See [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md) for full details and justifications.

## Install

The project is distributed on three registries — pick whichever matches how you
want to use it. All three track the same version and are built from the same
source tree.

### Python (PyPI)

```sh
pip install vle-thermo
```

Distribution name is `vle-thermo`, import name is `vle` (like Pillow → PIL):

```python
import vle
```

Optional extras: `pip install "vle-thermo[plot]"` (matplotlib),
`"vle-thermo[db]"` (extended component-database seeding via `thermo`).
See [python/README.md](python/README.md).

#### Quickstart

Build a system from component names (critical constants come from the bundled
database), then flash it. Temperatures are **K** and pressures **kPa absolute**
by default, but any input accepts a unit-aware value:

```python
from vle import System
from vle.units import Q_

# n-heptane / n-butane with the RKS equation of state (Chapter IV, Table 4.10).
sys = System(["n-heptane", "n-butane"], eos="RKS")

res = sys.flash_pt(300.0, 100.0, z=[0.5, 0.5])     # or flash_pt(Q_(26.85, "degC"), "1 bar", ...)
print(res.beta, res.x, res.y)                        # vapor fraction, liquid & vapor comps

# The batch API is the "numpy for thermo" path: one FFI crossing per array,
# GIL released, parallel across cores, warm-started along the sweep.
import numpy as np
ts = np.linspace(290.0, 340.0, 100_000)
out = sys.flash_pt_batch(ts, ps=np.array([100.0]), z=[0.5, 0.5])   # ~10× a Python loop
print(out.beta.shape, out.converged.sum())
```

A full guided tour — installation, the `System` API, unit handling, the batch
API, and plotting — is in [`notebooks/01_introduction.ipynb`](notebooks/01_introduction.ipynb).

### Rust (crates.io)

```sh
cargo add vle-thermo
# Optional: the companion units crate for gauge pressure / °C / psi parsing
cargo add vle-units
```

API docs: <https://docs.rs/vle-thermo>. See [engine/README.md](engine/README.md).

---

### Work through the notebooks

Install the library, grab the notebooks, and open them in your own Jupyter —
**no need to clone the whole repo**:

```sh
pip install "vle-thermo[plot]" jupyterlab

# Option 1 — just the notebooks/ folder (sparse, blobless git checkout):
git clone --depth 1 --filter=blob:none --sparse https://github.com/miguelju/vle.git
cd vle && git sparse-checkout set notebooks

# Option 2 — a single notebook, straight from GitHub raw:
curl -O https://raw.githubusercontent.com/miguelju/vle/main/notebooks/02_pure_component.ipynb

# Option 3 — the folder with no git history (needs Node):
npx degit miguelju/vle/notebooks vle-notebooks

jupyter lab notebooks/        # or the file/folder you fetched
```

The notebooks use the standard `python3` kernel, so any Jupyter on a Python
3.10+ environment with `vle-thermo` installed runs them. Full prerequisites and
the notebook catalogue are in [deploy/NOTEBOOKS.md](deploy/NOTEBOOKS.md); all
distribution channels (PyPI, crates.io, notebooks) are summarized in
[deploy/README.md](deploy/README.md).

Release process and registry maintenance: [PUBLISHING.md](PUBLISHING.md).

## Project Structure

```
vle/
├── data/                    # Component property database
│   └── components.db        # SQLite database (generated, gitignored)
├── scripts/                 # Data extraction utilities (see scripts/README.md)
├── python/src/vle/          # Python package
│   ├── system.py            # High-level vle.System API (persistent handle, unit-aware)
│   ├── components.py        # Bundled JSON component DB loader (name-based lookup)
│   ├── results.py           # Result dataclasses (Flash/Bubble/Dew/Critical + batch)
│   ├── plots.py             # Pxy / Txy / phase-envelope helpers (matplotlib)
│   ├── data/               # Bundled components.json (ships in wheel)
│   ├── db/                  # Component database (connection, queries, models)
│   │   └── sql/             # Bundled schema.sql + seed_chapter4.sql (ship in wheel)
│   └── cli/                 # CLI tool (vle-db)
├── notebooks/               # 14 Jupyter notebooks (00–09 + 01_introduction + index; see deploy/NOTEBOOKS.md)
│   └── data/                # Pre-computed 3-D surface datasets (CSV, committed)
├── units/                   # Independent units crate (dimensional analysis, gauge pressure, custom units)
├── engine/                  # Rust computation engine (complete: 22+ EOS, mixture core, flash suite, PyO3 bindings)
├── docs/
│   ├── en/research-paper/   # English translation (navigatable)
│   ├── en/units/            # Units add-on design document
│   └── es/research-paper/   # Spanish original (pdf/ + markdown/)
├── legacy/
│   ├── vb6/                 # Original VB6 source (~15,000 lines, reference)
│   └── pascal/              # Original Pascal source (~2,500 lines, reference) (4)
├── ROADMAP.md               # Milestones and progress tracking
├── TODO.md                  # Tasks with time estimates
├── MODERNIZATION_PLAN.md    # 18-phase implementation plan
├── PASCAL_VB6_COMPARISON.md # Legacy codebase comparison
└── CLAUDE.md                # Claude Code development guidance and conventions
```

## Development Workflow

This project is developed incrementally using [Claude Code](https://claude.ai/code) as an AI development partner. Each milestone follows a **plan-then-execute** cycle, tracked across three synchronized documents:

| Document | Purpose |
|----------|---------|
| [`ROADMAP.md`](ROADMAP.md) | Milestones — high-level goals and deliverables |
| [`TODO.md`](TODO.md) | Tasks — actionable items with time estimates per milestone |
| [`MODERNIZATION_PLAN.md`](MODERNIZATION_PLAN.md) | Phases — detailed technical implementation plan (18 phases) |

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

Each milestone records which AI model was used (e.g., `Claude Opus 4.6 (1M context)`, `Claude Fable 5`) in the commit and documentation for reproducibility tracking — see the model history in [Built with Claude Code](#built-with-claude-code) below.

### Project conventions

- All code, documentation, and project management follow the rules in [`CLAUDE.md`](CLAUDE.md).
- Every function that accepts or returns a physical quantity documents the units in its doc comment.
- All internal calculations use **absolute** pressure in **kPa** — never gauge pressure.
- Phase numbering in `MODERNIZATION_PLAN.md` always matches milestone execution order.

## Getting Started

**Status**: Milestones 0–9 are complete — the full engine (22+ EOS, mixing rules with exact derivatives, energy properties, and the modern flash/regression suite), Python bindings, and 13 notebooks, all validated against the thesis's Chapter IV tables. Milestones 10–11 (batch numpy API, final walkthrough) remain. Note the latest **published** release is v0.7.0 — the mixture core and flash suite are on `main` awaiting the next release, so build from source for those until then. Per-milestone detail: [ROADMAP.md](ROADMAP.md).

### Prerequisites
- Python 3.10+
- Rust 1.85+ with cargo (only if building the engine from source)
- maturin (for building PyO3 bindings from source)

### Component Database
```bash
PYTHONPATH=python/src python -m vle.cli.main init               # Create database
PYTHONPATH=python/src python -m vle.cli.main seed --source chapter4  # Seed 15 compounds
PYTHONPATH=python/src python -m vle.cli.main list               # Browse components
PYTHONPATH=python/src python -m vle.cli.main show methane       # View details
```

### Build (from source)
```bash
conda activate vle                    # Or your own Python 3.10+ environment
cargo build --release                 # Build the Rust workspace (engine + units)
cargo test --workspace                # Run the Rust test suite
(cd python && maturin develop --release)  # Build + install the Python bindings
python -c "import vle; print(vle._engine.version())"  # Verify
pytest python/tests/                  # Run the Python test suite
```

## Documentation

- [Developer Setup Guide](docs/en/SETUP.md) — Prerequisites, build instructions, and development workflow (Rust, Python/conda, maturin)
- [Dimensional Analysis](docs/en/units/dimensional-analysis.md) — Units add-on design: SI dimensions, gauge pressure, extensible unit registry
- [Modernization Plan](MODERNIZATION_PLAN.md) — Full technical plan with academic references, algorithm mapping, and performance improvements
- [Performance Proposal](PERFORMANCE_PROPOSAL.md) — The speed/convergence plan (2026-07): modern flash algorithms, exact-derivative core, batch numpy API
- [Pascal vs VB6 Comparison](PASCAL_VB6_COMPARISON.md) — Detailed comparison of the two legacy codebases
- [Research Paper (English)](docs/en/research-paper/README.md) — Navigatable English translation
- [Research Paper (Spanish)](docs/es/research-paper/README.md) — Original Spanish text (PDFs)

## Academic References

The implementation cites 29 academic references (ACS format). Key ones include:

- **(4)** Da Silva, F. A.; Báez, L. Thesis, Universidad Simón Bolívar, 1989. — Pascal codebase origin
- **(5)** Abbott, M. M. In *Equations of State in Engineering and Research*; ACS, 1979. — General cubic EOS form
- **(9)** Müller, E.; Olivera Fuentes, C.; Estévez, L. *Lat. Am. Appl. Res.* **1989**, *19* (2), 99. — Multicomponent fugacity
- **(16)** Heidemann, R. A.; Khalil, A. M. *AIChE J.* **1980**, *26* (5), 769. — Critical point algorithm
- **(19)** Michelsen, M. L. *Fluid Phase Equilib.* **1982**, *9*, 21. — Rachford-Rice / phase split
- **(21)** Orbey, H.; Sandler, S. I. Cambridge University Press, 1998. — Advanced mixing rules

The full reference list and code mapping is in [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md#academic-references).

## Built with Claude Code

**This entire project was built with [Claude Code](https://claude.ai/code)**, Anthropic's CLI coding agent. Every aspect — from analyzing 20,000 lines of legacy code to writing this README — was done collaboratively between a human developer and an AI agent working directly in the terminal.

### Model history

The project spans several generations of Claude models, and each milestone records which one executed it (in `ROADMAP.md`, `TODO.md`, and the commit trailers):

| Model | Milestones | Contribution |
|---|---|---|
| **Claude Opus 4.6** (1M context) | 0–2 | Foundation: legacy-code analysis, reference mapping, English translation, repo structure, the modernization plan |
| **Claude Opus 4.7** (1M context) | 3–6 | Units add-on, component database, CI/CD + publishing pipeline, numerics layer |
| **Claude Opus 4.8** (1M context) | 7–8.1 | The full pure-component EOS zoo (22+ α variants, 3-parameter EOS, saturation models, virial) and the activity-coefficient models |
| **Claude Fable 5** | 8.2–9 | **The major performance & algorithm modernization**: the measured performance foundation (benches, allocation-free hot paths, caches), the generalized (A, B, U, W) mixture core with **exact analytic/dual-number derivatives** replacing every finite difference, mixture energy properties, and the complete modern flash suite — stability analysis, guaranteed-convergence Rachford-Rice, bubble/dew, adiabatic flash, Heidemann–Khalil critical points, phase-envelope continuation through the critical point, and kij/Aij regression — validated against the thesis's Chapter IV tables |

In short: the project **started with Opus 4.6** doing the archaeology and planning, and **Fable 5 delivered the deep optimization work** — the exact-derivative architecture and modern algorithm suite that the original thesis identified as its own main weaknesses (Ch. IV §4.1).

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
