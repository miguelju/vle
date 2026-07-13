# Distribution

This repository is open-source educational software, and **how the code
reaches its users** is itself part of what it teaches. `vle-thermo` travels
into other people's environments through a small handful of channels — and
each one exists for a different kind of user.

## How `vle-thermo` reaches users

| Audience | Channel | Install / launch |
|---|---|---|
| Python users (most readers) | **PyPI** | `pip install vle-thermo` |
| Learners working through the bundled notebooks | **GitHub** ([`notebooks/`](https://github.com/miguelju/vle/tree/main/notebooks)) | clone the repo, `pip install "vle-thermo[plot]" jupyterlab`, then open in your preferred notebook viewer — no special kernel needed |
| Rust developers embedding the engine | **crates.io** | `cargo add vle-thermo` &nbsp; `cargo add vle-units` |
| Swift developers building iOS/macOS apps | **GitHub (source) + local build** — nothing published, by design | clone the repo, `scripts/build-ios.sh`, then add `swift/VleThermo` as a local package in Xcode — see [docs/en/ios/README.md](../docs/en/ios/README.md) |
| Kotlin developers building Android / Compose Desktop apps | **GitHub (source) + local build** — nothing published, by design | clone the repo, `scripts/build-android.sh`, then include `kotlin/VleThermo` from Android Studio — see [docs/en/android/README.md](../docs/en/android/README.md) |
| JavaScript/TypeScript developers building web, desktop (Tauri/Electron), or mobile (Tauri/Capacitor) apps | **GitHub (source) + local build** — nothing published, by design | clone the repo, `scripts/build-wasm.sh`, then `npm install <path-to-vle>/wasm/pkg` — see [docs/en/web/README.md](../docs/en/web/README.md) |

The channels share **one source of truth**: the same Rust source is
published to crates.io, compiled into the PyPI wheels, and compiled locally
into the Swift package's XCFramework, the Kotlin module's `.so`s, and the
wasm package's `.wasm` module; the notebooks exercise that published
library. There are no parallel implementations to keep in sync. (The
Swift, Kotlin, and JavaScript rows are deliberately *not* registries: the
artifacts are machine-built binaries, so the repo distributes the recipe,
not the artifact.)

> **What about C#/.NET?** That route was evaluated and is deliberately
> **not** offered: the community C# bindings generator lags the UniFFI
> version this workspace pins, so the toolchain doesn't line up (as of
> 2026-07-12). The full analysis — and the complete would-be recipe, should
> the versions ever converge — lives in
> [docs/en/dotnet/README.md](../docs/en/dotnet/README.md).

### crates.io — Rust source distribution

[crates.io](https://crates.io) is the official public registry for Rust
libraries (analogous to PyPI for Python or npm for JavaScript). Two crates
from this project live there:

- **[`vle-thermo`](https://crates.io/crates/vle-thermo)** — the numerical
  engine (cubic EOS solvers, activity models, flash algorithms).
- **[`vle-units`](https://crates.io/crates/vle-units)** — the unit registry
  and dimensional-analysis layer; usable standalone in any Rust project
  that needs thermodynamic-flavored units (see [`units/README.md`](../units/README.md)).

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

### Notebooks from the repo (work through them locally)

If you'd rather learn from the bundled notebooks on your own machine —
in JupyterLab, classic Jupyter Notebook, VS Code's Jupyter extension, or
any other client — pull them straight from the
[`notebooks/`](https://github.com/miguelju/vle/tree/main/notebooks)
directory:

```sh
git clone https://github.com/miguelju/vle.git
cd vle
pip install "vle-thermo[plot]" jupyterlab    # or `jupyter` / VS Code Jupyter
jupyter lab notebooks/                        # opens the notebook folder
```

**Kernel:** nothing custom is required. The notebooks declare their
kernel as `python3` (the standard `ipykernel` name), so any Jupyter
install with a Python 3.10+ environment that has `vle-thermo` installed
will run them. There's nothing `vle`-specific about the kernel itself.

For the full host-agnostic prerequisites, install options, the notebook
catalogue, and the `components.db` setup, see [`NOTEBOOKS.md`](NOTEBOOKS.md).

> The `vle-thermo[plot]` extra pulls in `matplotlib`, which the
> notebooks use for P-x-y plots. Skip the `[plot]` if you only care
> about the numerical cells.

### WebAssembly — the browser and JS runtimes

The engine also compiles to **WebAssembly**: the same Rust source, built
locally into a small npm package (`scripts/build-wasm.sh` →
`wasm/pkg/`, ~150 KB gzipped including the flash suite, steam tables, and
the bundled component database). The thermodynamics then runs
**client-side in the visitor's browser** — and because every desktop and
mobile webview is also a browser, one React (or plain JS) codebase covers
four delivery shapes:

- **React web** — import the package, `await init()`, call the engine
  directly. The site stays static files: no compute server, no API, no
  hosting cost beyond the files themselves.
- **Tauri 2** — the same bundle as a **Windows** (or macOS/Linux) *and*
  **Android/iOS** app; ~10 MB installers on the system webview. With the
  engine as wasm inside the frontend, the shell carries no backend code.
- **Electron** — the same bundle as a desktop-only app; bundles Chromium
  (bigger, maximally battle-tested). A `napi-rs` native module is the
  documented upgrade path if a desktop workload ever needs full native
  speed with rayon threads.
- **Capacitor** — the same bundle wrapped for Android/iOS when only
  mobile is needed.

Which shell to use is a packaging decision made in the app repo, not
here. Build, API examples (units: K, kPa absolute), the Web Worker
pattern, and shell notes: [docs/en/web/README.md](../docs/en/web/README.md).

## Layout

```
deploy/
├── README.md       this file — the distribution channels
├── NOTEBOOKS.md    host-agnostic guide to running the bundled notebooks
└── scripts/
    ├── publish-crate.sh   cargo publish (vle-units then vle-thermo)
    └── publish-pypi.sh    maturin build + publish to PyPI
```
