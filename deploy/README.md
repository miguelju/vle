# Registry Distribution — PyPI & crates.io

This folder covers the two channels where `vle-thermo` is **published as a
package**: PyPI for Python users, crates.io for Rust users. Both are fed by
the same source tree and the same `v*` tag, and both are automated by
[`release.yml`](../.github/workflows/release.yml).

> **Looking for the other channels?** Swift (iOS/macOS), Kotlin (Android and
> Compose Desktop), JavaScript/WebAssembly, the bundled notebooks, and the
> parked C#/.NET route are **not** published to a registry — the repo ships
> the build recipe instead of the artifact. They all live in
> **[`../distribution/README.md`](../distribution/README.md)**.

This repository is open-source educational software, and **how the code
reaches its users** is itself part of what it teaches. The two registry
channels exist for two different kinds of user, and the shape of each
package follows from what that user already has installed.

## The two registry channels

| Audience | Channel | Install |
|---|---|---|
| Python users (most readers) | **PyPI** | `pip install vle-thermo` |
| Rust developers embedding the engine | **crates.io** | `cargo add vle-thermo` &nbsp; `cargo add vle-units` &nbsp; `cargo add vle-steam` |

Both share **one source of truth**: the same Rust source is published to
crates.io and compiled into the PyPI wheels. There are no parallel
implementations to keep in sync.

### crates.io — Rust source distribution

[crates.io](https://crates.io) is the official public registry for Rust
libraries (analogous to PyPI for Python or npm for JavaScript). Three crates
from this project live there:

- **[`vle-thermo`](https://crates.io/crates/vle-thermo)** — the numerical
  engine (cubic EOS solvers, activity models, flash algorithms).
- **[`vle-units`](https://crates.io/crates/vle-units)** — the unit registry
  and dimensional-analysis layer; usable standalone in any Rust project
  that needs thermodynamic-flavored units (see [`units/README.md`](../units/README.md)).
- **[`vle-steam`](https://crates.io/crates/vle-steam)** — IAPWS-IF97 steam
  tables, dependency-free and usable on its own ("VLE for water only";
  see [`steam/README.md`](../steam/README.md)).

Publishing is one command per crate: `cargo publish`. crates.io stores
*source code*; when a downstream user runs `cargo add vle-thermo` and then
`cargo build`, their own machine compiles the source into a native library.
That works because every Rust developer has a Rust toolchain on hand — for
Rust users, "source distribution + local compile" is the natural shape.

The release pipeline runs this for you: see [`scripts/publish-crate.sh`](scripts/publish-crate.sh)
for the manual/operator path and [`../PUBLISHING.md`](../PUBLISHING.md) for the
full release flow.

### PyPI — pre-compiled binary wheels

[PyPI](https://pypi.org) (the Python Package Index) is the standard
registry `pip` reads from. The Python package
**[`vle-thermo`](https://pypi.org/project/vle-thermo/)** lives there.

Python users don't typically have a Rust toolchain, so PyPI wouldn't accept
the source-distribution model. Instead, this project ships **pre-compiled
binary wheels** — one per (operating system, CPU architecture) combination:
Linux x86_64, Linux aarch64, macOS arm64, Windows x86_64. Each wheel
embeds a pre-built native extension module, so `pip install vle-thermo`
lands ready-to-import code on the user's machine without any compiler
involvement.

The wheels are produced by **[`maturin`](https://maturin.rs)**, the
build tool that packages PyO3-using Rust crates into Python wheels. CI
runs [`cibuildwheel`](https://cibuildwheel.pypa.io) once per
(OS, arch) combination, calling maturin inside each container, and uploads
the resulting matrix of wheels to PyPI. The full mechanics — what PyO3
generates, what maturin does step-by-step, how a function call traverses
the FFI boundary — live in [`python/README.md`](../python/README.md), in
the section "How the Python package wraps Rust". The manual/operator path is
[`scripts/publish-pypi.sh`](scripts/publish-pypi.sh).

## The publish scripts are an operator escape hatch

Neither script is called by CI. `release.yml` publishes with
`pypa/gh-action-pypi-publish` (Trusted Publishing / OIDC) and
`cargo publish -p <crate>` directly. The scripts under `scripts/` exist so
that **the operator path always works without Actions** — a release can be
cut by hand from any machine with the credentials, exactly as CI would do
it. See [`../PUBLISHING.md`](../PUBLISHING.md).

## Layout

```
deploy/
├── README.md       this file — the two registry channels
└── scripts/
    ├── publish-crate.sh   cargo publish (vle-units, vle-steam, vle-thermo)
    └── publish-pypi.sh    maturin build + publish to PyPI
```

Everything that is **not** a registry publish — notebooks, Swift, Kotlin,
WebAssembly, C#/.NET — is in [`../distribution/`](../distribution/README.md).
