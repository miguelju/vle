# Running the VLE Notebooks

This document describes the **generic** requirements for running the VLE
Jupyter notebooks. It is intentionally host-agnostic: the same instructions
apply whether you run them on your laptop, a CI runner, a managed notebook
service (Binder, Colab, SageMaker, etc.), or the multi-user JupyterHub stack
documented in [`README.md`](README.md).

> Looking to run the multi-user, auth-gated deployment instead?
> See [`deploy/README.md`](README.md) for the JupyterHub + Docker stack.

## Prerequisites

| Requirement                | Version        | Notes                                                |
|----------------------------|----------------|------------------------------------------------------|
| Python                     | ≥ 3.10         | The `vle` package targets 3.10+                      |
| Rust toolchain (build-time)| stable         | Only needed if installing from source via `maturin`  |
| JupyterLab or notebook     | ≥ 4 / ≥ 7      | Either works                                         |
| NumPy                      | ≥ 1.24         | Transitive dep of `vle`                              |
| SciPy                      | ≥ 1.11         | Used by some notebooks for comparison plots          |
| matplotlib                 | ≥ 3.7          | All notebooks that plot                              |

A 64-bit architecture (x86_64 or arm64) is required; 32-bit builds are not
supported.

## Install

### Option A — from a pre-built wheel (recommended)

```sh
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install vle jupyterlab numpy scipy matplotlib
```

### Option B — from source (for development)

```sh
git clone https://github.com/YOUR/vle.git
cd vle
python -m venv .venv && source .venv/bin/activate
pip install maturin jupyterlab numpy scipy matplotlib
cd python
maturin develop --release
```

`maturin develop` builds the Rust engine (`engine/`) as a native extension and
installs it into the active environment as `vle._engine`.

## Run

```sh
jupyter lab notebooks/
```

Open any `.ipynb` file. The notebooks are designed to execute top-to-bottom in
a fresh kernel — if a cell fails, restart the kernel and try again before
filing an issue.

## Notebook catalogue

The notebooks grow with the project. Each milestone adds one notebook
covering the feature it built; Milestone 10 finishes with a full walkthrough
of the research paper's Chapter IV validation cases.

| File                                                | Milestone | Topic                                           |
|-----------------------------------------------------|-----------|-------------------------------------------------|
| `00_component_database.ipynb`                       | 4         | Browse & query the SQLite component DB          |
| `01_introduction.ipynb`                             | 9         | Installation walk-through, basic API            |
| `02_pure_component.ipynb`                           | 6         | PVT behaviour, compare cubic EOS variants       |
| `03_activity_models.ipynb`                          | 7         | Activity coefficient models, excess Gibbs plots |
| `04_bubble_dew_point.ipynb`                         | 8         | Reproduce Tables 4.6–4.9                        |
| `05_flash_calculations.ipynb`                       | 8         | Reproduce Tables 4.3–4.4, 4.10                  |
| `06_critical_points.ipynb`                          | 8         | Reproduce Tables 4.1–4.2                        |
| `07_kij_regression.ipynb`                           | 8         | Reproduce Tables 4.11–4.12                      |
| `08_aij_regression.ipynb`                           | 8         | Activity model Aij fitting (Pascal-origin)      |
| `09_chapter4_validation_walkthrough.ipynb`          | 10        | End-to-end walkthrough of Chapter IV            |
| `m05_numerics.ipynb`                                | 5         | Demo of Brent / Illinois / Broyden / Halley     |

Notebooks appear in the table as they are authored; a missing file means the
corresponding milestone has not shipped its notebook yet.

## Data

Several notebooks (starting with `00_component_database.ipynb`) need
`data/components.db`, a SQLite file of critical properties, Antoine
coefficients, binary interaction parameters, and experimental VLE points. The
database is *not* checked into the repository — create it on first run:

```sh
vle-db init                    # create schema
vle-db seed --source chapter4  # seed 15 Chapter IV compounds + kij + van Laar + VLE
vle-db validate chapter4       # sanity-check that everything landed
```

`vle-db` is installed as a console script by `pip install vle`; as a fallback
you can always run `python -m vle.cli.main ...` instead. The default database
location is `data/components.db` relative to the repo root. Override with the
`VLE_DB_PATH` environment variable if you need it elsewhere.

For a larger compound pool (~70 k DIPPR/ChemSep compounds) install the
optional `thermo` dependency and seed from it:

```sh
pip install 'vle[db]'          # installs the thermo library
vle-db seed --source thermo    # seeds ~40 common industrial compounds by default
```

See [`docs/en/parameters/parameter_reference.md`](../docs/en/parameters/parameter_reference.md)
for the parameter inventory and [`data/schema.sql`](../data/schema.sql) for
the schema.

> When running through the JupyterHub Docker stack, you do not need to run
> `vle-db init` yourself — `Dockerfile.notebook` bakes a pre-seeded
> `components.db` into the image and a startup hook copies it into each user's
> `~/work/data/` on first login. See [`README.md`](README.md).

## Troubleshooting

**`ModuleNotFoundError: No module named 'vle'`** — the wheel (Option A) failed
to install, or `maturin develop` (Option B) was run outside the active venv.
Confirm with `python -c "import vle; print(vle.__file__)"`.

**`vle._engine not found`** — the Rust extension module wasn't built. Run
`maturin develop --release` from `python/` with your venv active.

**Plots don't render in JupyterLab** — make sure you're on JupyterLab ≥ 4.
Classic notebook 6.x is end-of-life.

**Kernel dies mid-calculation** — check memory; the critical-point and
adiabatic-flash notebooks can spike to a few hundred MB on large systems.
Set `CONTAINER_MEM_LIMIT` higher in your environment or run locally with more
headroom.
