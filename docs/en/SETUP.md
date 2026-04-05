# Developer Setup Guide

Setup instructions for building and developing the VLE thermodynamic calculator on macOS.

---

## Prerequisites

### Rust Toolchain

Install Rust via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:

```bash
rustc --version   # Should be 1.75+
cargo --version
```

### Python Environment (conda)

Requires Python 3.10+. We use conda for environment management:

```bash
# Install Miniconda if not already installed
# https://docs.conda.io/en/latest/miniconda.html

# Create the vle environment
conda create -n vle python=3.11 -y
conda activate vle
```

### maturin (Rust-Python Bridge)

Install maturin inside the conda environment:

```bash
conda activate vle
pip install maturin
```

---

## Project Structure

```
vle/
├── engine/          # Rust crate (core computation)
│   ├── Cargo.toml
│   └── src/
├── python/          # Python wrapper package
│   ├── pyproject.toml
│   └── src/vle/
├── notebooks/       # Jupyter notebooks
├── docs/            # Documentation
└── legacy/          # Original VB6 and Pascal source (reference only)
```

---

## Building

### Rust Engine

```bash
cd engine
cargo build --release
cargo test
```

### Python Package (via maturin)

From the `python/` directory (ensure conda env is active):

```bash
conda activate vle
cd python
maturin develop    # Build and install in conda env (debug mode)
maturin develop --release   # Optimized build
```

### Verify Installation

```bash
conda activate vle
python -c "import vle; print(vle.__version__)"
```

---

## Development Workflow

### Running Tests

```bash
conda activate vle

# Rust tests
cd engine && cargo test

# Python tests
cd python && python -m pytest tests/

# Single Rust test
cargo test test_cardano_cubic

# Single Python test
python -m pytest tests/test_pure_eos.py -k "test_pr_fugacity"
```

### Rebuilding After Changes

After modifying Rust code, rebuild the Python bindings:

```bash
conda activate vle
cd python && maturin develop
```

### Jupyter Notebooks

```bash
conda activate vle
conda install jupyter matplotlib numpy -y
jupyter notebook notebooks/
```

---

## Key Dependencies

### Rust (`engine/Cargo.toml`)

| Crate | Purpose |
|-------|---------|
| `nalgebra` | Linear algebra (replaces hand-rolled Gauss elimination) |
| `ndarray` | N-dimensional arrays |
| `pyo3` | Python bindings |
| `approx` | Approximate floating-point comparisons (tests) |

### Python (`python/pyproject.toml`)

| Package | Purpose |
|---------|---------|
| `maturin` | Build backend for PyO3 |
| `numpy` | Array operations |
| `matplotlib` | Plotting (Pxy, Txy diagrams) |
| `pytest` | Testing |

---

## IDE Setup

### VS Code (Recommended)

Extensions:
- `rust-analyzer` — Rust language support
- `Python` — Python language support
- `Even Better TOML` — Cargo.toml editing

### Environment Variables

No special environment variables required. The Rust engine is a self-contained crate.

---

## Troubleshooting

**`maturin develop` fails with "can't find Python"**:
Ensure conda env is active: `conda activate vle`

**`cargo build` fails with linker errors on macOS**:
Install Xcode command line tools: `xcode-select --install`

**Python can't import `vle`**:
Rebuild with `maturin develop` from the `python/` directory with `conda activate vle`.

---

[Back to README](../../README.md)
