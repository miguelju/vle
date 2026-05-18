# vle-thermo

Vapor-liquid equilibrium (VLE) thermodynamic calculator with a Rust computation
engine and a Python interface.

[![PyPI](https://img.shields.io/pypi/v/vle-thermo.svg)](https://pypi.org/project/vle-thermo/)
[![Python](https://img.shields.io/pypi/pyversions/vle-thermo.svg)](https://pypi.org/project/vle-thermo/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/miguelju/vle/blob/main/LICENSE)

A modern Rust + Python port of two legacy thermodynamic codebases (VB6 ~15,000
lines + Pascal ~2,500 lines). The Rust core (`vle-thermo` crate) does the
computation; this Python package wraps it for interactive use in scripts and
Jupyter notebooks.

## Install

```sh
pip install vle-thermo
```

Optional extras:

```sh
pip install "vle-thermo[plot]"   # adds matplotlib
pip install "vle-thermo[db]"     # adds `thermo` for extended component-database seeding
pip install "vle-thermo[dev]"    # adds pytest, maturin
```

The distribution name on PyPI is **`vle-thermo`**, but the import name is
**`vle`** (following the common distribution-vs-import split, like
`Pillow` → `import PIL` or `python-dateutil` → `import dateutil`):

```python
import vle
```

## Status

`0.1.x` — the component database, CLI, and units layer work today. Numerical
kernels (flash algorithms, equation-of-state solvers, parameter regression) are
under active development. Treat `0.1.x` as pre-release; semver promises begin
at 1.0.

See the [roadmap](https://github.com/miguelju/vle/blob/main/ROADMAP.md) for
what's shipped vs. planned.

## What works today

```sh
# Initialize the bundled component database (SQLite) and seed with the 15
# compounds from Chapter IV of the source thesis.
vle-db init
vle-db seed --source chapter4

# Browse and inspect
vle-db list
vle-db show methane
vle-db validate chapter4
```

```python
from vle.db import list_components, get_component

for c in list_components():
    print(c.name, c.tc, c.pc, c.omega)

methane = get_component("methane")
```

Unit-aware input/output (gauge pressure, °C, °F, psi, barg, mmHg, …):

```python
from vle.units import ureg, Q_

T = Q_(25, "degC")                # 298.15 K internally
P = Q_(3.5, "bar").to("kPa")      # 350 kPa
```

## Features (full roadmap)

- **22+ cubic equations of state** — Peng-Robinson, RKS, van der Waals,
  Schmidt-Wenzel, Patel-Teja, and more
- **5 activity coefficient models** — Wilson, van Laar, Margules,
  Scatchard-Hildebrand, Ideal
- **11 mixing rules** — Classical (IVDW, IIVDW), Wong-Sandler, Huron-Vidal,
  MHV1/MHV2
- **6 saturation pressure correlations** — Antoine, Riedel, Müller, RPM
- **6 flash calculation types** — bubble/dew point (T/P), isothermal
  (Rachford-Rice), adiabatic
- **Parameter regression** — kij (binary interaction) and Aij (activity
  model)

## Use it in Jupyter

A curated set of notebooks reproducing Chapter IV of the source thesis ships
alongside the project at
<https://github.com/miguelju/vle/tree/main/notebooks>. To run them in your own
environment:

```sh
pip install "vle-thermo[plot]" jupyterlab
git clone https://github.com/miguelju/vle.git
cd vle/notebooks
jupyter lab
```

For a zero-setup option, a ready-to-run Docker image is published at
`ghcr.io/miguelju/vle-thermo`:

```sh
docker run --rm -p 8888:8888 ghcr.io/miguelju/vle-thermo:latest
```

## Origin

Based on the thesis *"Desarrollo de un Programa Computacional para el Cálculo
del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente
Windows"* (Jackson & Mendible, Universidad Simón Bolívar, 1999), with
additional models from Da Silva & Báez (1989). See the
[research paper](https://github.com/miguelju/vle/tree/main/docs/en/research-paper)
(English translation) for algorithms, parameters, and their academic references.

## License

MIT. See [LICENSE](https://github.com/miguelju/vle/blob/main/LICENSE).
