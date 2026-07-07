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

`0.9.1` — pre-1.0, but the numerical core is live. Flash algorithms
(isothermal, bubble/dew *T* and *P*, adiabatic *PH*), 22+ cubic
equations of state, the five activity models, the mixture critical point,
and kij/Aij parameter regression all run through the Rust engine, exposed
through a unit-aware Python facade (`vle.System`) and a vectorized numpy
**batch API**. The bundled component database (24 compounds, with
ideal-gas Cp°/R coefficients), CLI, and units layer round it out.
`0.9.1` fixes a ~1% Wong-Sandler departure-enthalpy inconsistency
(residual enthalpy/entropy with the `"wong-sandler"` mixing rule now
satisfies the Gibbs–Helmholtz identity to machine precision).
`0.9.0` added **exact temperature/pressure derivatives** of fugacity and
K-values (`System.d_ln_phi_d_t`, `k_values_with_derivs`, dual-number AD),
**real-mixture heat capacity** (`System.phase_cp`) and **partial molar
enthalpy** (`System.partial_molar_enthalpy`) — the properties a downstream
staged-separation library needs. Treat `0.x` as pre-release; semver promises
begin at 1.0.

See the [roadmap](https://github.com/miguelju/vle/blob/main/ROADMAP.md) for
what's shipped vs. planned.

## Quick start

Build a system from component names and a model choice, then run flash and
saturation calculations. Inputs and outputs are unit-aware (`vle.units`):

```python
import vle
from vle.units import Q_

# n-heptane / n-butane with Redlich-Kwong-Soave (Chapter IV, Table 4.10)
sys = vle.System(["n-heptane", "n-butane"], eos="rks")

# Isothermal (PT) flash at 300 K, 100 kPa, equimolar feed
res = sys.flash_pt(Q_(300, "K"), Q_(100, "kPa"), z=[0.5, 0.5])
print(res.beta)   # vapor fraction ≈ 0.197
print(res.x)      # liquid composition (numpy array)
print(res.y)      # vapor composition

# Bubble-point pressure of an equimolar liquid at 300 K
bub = sys.bubble_pressure([0.5, 0.5], Q_(300, "K"))
print(bub.value)  # ≈ 127.8 kPa
```

`System` also exposes `dew_pressure`, `bubble_temperature`, `dew_temperature`,
`flash_ph` (adiabatic), `critical_point`, `phase_envelope`, `stability`,
`k_values`, `ln_phi`, `z_factor`, and `enthalpy_entropy`.

### Vectorized numpy batch API

Every calculation has a `_batch` sibling that takes numpy arrays and releases
the GIL, running the sweep in parallel across cores:

```python
import numpy as np
import vle
from vle.units import Q_

sys = vle.System(["n-heptane", "n-butane"], eos="rks")

ts = Q_(np.linspace(280.0, 320.0, 5), "K")
ps = Q_(np.full(5, 100.0), "kPa")

batch = sys.flash_pt_batch(ts, ps, z=[0.5, 0.5])
print(batch.beta)        # vapor fraction at each (T, P)
print(batch.converged)   # per-point convergence flags
```

## Component database & units

```sh
# Initialize the bundled component database (SQLite) and seed with the 24
# bundled compounds (15 Chapter IV + 9 distillation/absorber additions).
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
    print(c.name, c.tc, c.pc, c.w)  # w = acentric factor

methane = get_component("methane")
```

Unit-aware input/output (gauge pressure, °C, °F, psi, barg, mmHg, …):

```python
from vle.units import ureg, Q_

T = Q_(25, "degC")                # 298.15 K internally
P = Q_(3.5, "bar").to("kPa")      # 350 kPa
```

## Features

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

See [`deploy/NOTEBOOKS.md`](https://github.com/miguelju/vle/blob/main/deploy/NOTEBOOKS.md)
for the full host-agnostic guide and
[`deploy/README.md`](https://github.com/miguelju/vle/blob/main/deploy/README.md)
for the distribution story.

## How the Python package wraps Rust — `maturin` + `PyO3`

This project is partly educational, so it's worth explaining the build glue
in detail. Two tools split the work:

- **[PyO3](https://pyo3.rs)** is a Rust crate that handles the FFI bridge **at
  runtime**. You annotate Rust functions with `#[pyfunction]` and Rust types
  with `#[pyclass]`, and PyO3's procedural macros generate all the CPython C-API
  calls — argument unpacking, type conversion (Python `dict` ↔ Rust `HashMap`,
  Python `list` ↔ Rust `Vec`, etc.), GIL acquisition, and turning a
  `Result::Err(...)` into a Python `raise`.

- **[maturin](https://www.maturin.rs)** is a build tool that packages a
  PyO3-using Rust crate into a Python wheel **at build time**. PyO3 produces a
  Rust crate that *can be* a Python extension; maturin does the work of
  actually shipping it as one.

**What "FFI" means in "FFI bridge".** *FFI* stands for **Foreign Function
Interface** — the conventions and machinery that let code in one language
call functions written in another. Each language runtime has its own ideas
about how arguments are passed, how strings are laid out in memory, how
errors propagate, and how memory ownership works; you can't just call a
Rust function from Python directly any more than you can plug a US power
cord into a UK outlet. The universal adapter in practice is the **C ABI**
(Application Binary Interface): C compilers all agree on how arguments are
placed in registers and on the stack, how function names appear in the
symbol table, and how stack frames are laid out. Any language that can
produce C-compatible function signatures (Rust, Go, Zig, Swift) can be
called by any language that can call C (Python, Ruby, Lua, JavaScript via
N-API). PyO3 sits exactly on top of that contract: its macros generate
C-ABI functions from your Rust code, give them CPython-shaped signatures
(taking `PyObject*` arguments, returning `PyObject*`), and export the
`PyInit_<modulename>` symbol CPython's import loader looks for. That's the
"runtime" part of "runtime FFI bridge" — code that runs on **every call**
into the extension module to convert Python values into Rust values, drive
the Rust implementation, and convert the result back. (The build-time
counterpart — turning the cdylib into an importable `.so` and a
`pip`-installable wheel — is the part maturin handles.)

### Why this stack (vs. the alternatives)

The numerical kernel needs to be fast — Python alone isn't — but Python is
where the user lives (Jupyter, scripts, the data-science ecosystem). So we
needed something that:

1. Ships as a normal `pip install` (no separate Rust toolchain for the end user).
2. Works on every OS Python supports (Linux x86_64/aarch64, macOS arm64,
   Windows).
3. Marshals types automatically across the boundary.
4. Bridges error handling (Rust `Result` → Python exception).

The realistic alternatives:

- **`setuptools-rust`** — works, but predates `pyproject.toml` and requires a
  `setup.py` shim. More moving parts.
- **A hand-rolled `setup.py` + cargo invocation** — possible, fragile,
  reinvents wheel-packaging logic.
- **`cffi`** — only handles C-style FFI, not the higher-level PyO3 ergonomics
  (typed Python classes, automatic GIL handling, exception bridging).

`maturin` is the build tool the PyO3 maintainers built and recommend — it's
specifically aware of PyO3's abi3 mode, the wheel ABI tags, and the
cross-compilation gotchas. The full build configuration is one TOML block.

### What maturin actually does

A Python "native extension module" is a shared library — `.so` on Linux,
`.dylib` on macOS, `.pyd` on Windows — that CPython's `import` machinery can
`dlopen` and find a `PyInit_<modulename>` symbol in. To produce one from a
PyO3 Rust crate, maturin runs the following pipeline:

1. **Compile** the Rust crate as a `cdylib` (C-compatible dynamic library)
   with PyO3's `#[pymodule]` entry-point function compiled in.
2. **Link** against the right Python ABI. PyO3's `abi3-py310` feature builds
   one wheel that loads on CPython 3.10, 3.11, 3.12, 3.13, and every future
   3.x — instead of one wheel per Python version, you ship one per (OS, arch).
3. **Rename** the resulting `.so` to Python's import convention
   (`_engine.abi3.so` for abi3, `_engine.cpython-310-darwin.so` otherwise).
4. **Pack** that file plus the pure-Python sources into a standards-compliant
   `.whl` (the binary distribution format `pip` understands), with the right
   ABI/platform tags in the filename so `pip` picks the matching wheel for
   the user's machine.
5. **Repeat** (1)–(4) for every (OS, arch) combination in CI — `cibuildwheel`
   calls maturin once per platform, producing the matrix of pre-built wheels
   you see on PyPI.

The end user types `pip install vle-thermo`, pip selects the wheel matching
their platform, and the Rust code lands on their machine **already compiled**.
No Rust toolchain required.

### What that looks like in this repo

```
engine/                       Rust crate
├── Cargo.toml                ├── crate-type = ["cdylib", "rlib"]
│                             └── pyo3 dep, gated behind the "python" feature
└── src/py_bindings.rs        the #[pyfunction] + #[pymodule] glue lives here

python/                       Python package
├── pyproject.toml            [tool.maturin] points at ../engine/Cargo.toml
└── src/vle/                  pure-Python code (vle.db, vle.units, vle.cli, …)
    ├── _engine.abi3.so       ← dropped here by maturin at install time
    └── __init__.py           re-exports from vle._engine + Python helpers
```

The entire build configuration is the `[tool.maturin]` block in
`python/pyproject.toml`:

```toml
[tool.maturin]
manifest-path = "../engine/Cargo.toml"   # which Rust crate to build
features      = ["python"]                # enables PyO3 in engine/Cargo.toml
python-source = "src"                     # vle.py files live in src/vle/
module-name   = "vle._engine"             # the cdylib becomes this module
```

Two commands matter day-to-day:

- **`maturin develop`** — for local development. Builds the Rust crate, drops
  the resulting `.so` into `python/src/vle/`, and installs the Python package
  into the active virtualenv in **editable mode**. Pure-Python edits show up
  immediately; Rust edits need a re-run.
- **`maturin build`** — for distribution. Produces a `.whl` you can
  `pip install` or upload to PyPI.

### Tracing a call across the boundary

To see all of this concretely, follow the `version()` call:

1. **Rust side** — [`engine/src/py_bindings.rs`](https://github.com/miguelju/vle/blob/main/engine/src/py_bindings.rs)
   declares `#[pyfunction] fn version() -> &'static str` and registers it
   inside `#[pymodule] fn _engine(...)`. PyO3's macros expand these into
   CPython-callable C functions plus the `PyInit__engine` symbol the OS
   loader needs.
2. **Build** — `maturin develop` compiles `engine/` into
   `python/src/vle/_engine.abi3.so` with that `PyInit__engine` symbol.
3. **Python side** — `python/src/vle/__init__.py` does `from vle._engine
   import version`. The first time Python imports `vle._engine`, CPython
   `dlopen`s the `.so`, calls `PyInit__engine`, and gets a module object
   with `version` already bound.
4. **Runtime** — `vle.version()` is now a plain Python function call.
   PyO3's generated wrapper acquires the GIL, calls into the Rust
   implementation, converts the returned `&'static str` to a Python `str`,
   and hands it back to the interpreter.

The takeaway: maturin is what makes step 2 a single command. PyO3 is what
makes steps 1, 3, and 4 a handful of attributes instead of hundreds of
lines of hand-written C glue. Together they collapse "ship Rust to Python
users" into a normal Python development workflow.

## Origin

Based on the thesis *"Desarrollo de un Programa Computacional para el Cálculo
del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente
Windows"* (Jackson & Mendible, Universidad Simón Bolívar, 1999), with
additional models from Da Silva & Báez (1989). See the
[research paper](https://github.com/miguelju/vle/tree/main/docs/en/research-paper)
(English translation) for algorithms, parameters, and their academic references.

## License

MIT. See [LICENSE](https://github.com/miguelju/vle/blob/main/LICENSE).
