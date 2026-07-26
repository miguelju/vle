# Distribution Beyond the Registries

Everything about how `vle-thermo` reaches users **other than** a package
registry: the bundled notebooks, and the three consumer languages the Rust
engine compiles into — Swift, Kotlin, and JavaScript/WebAssembly.

> **Looking for `pip install` or `cargo add`?** The two registry channels —
> PyPI and crates.io — are in **[`../deploy/README.md`](../deploy/README.md)**.

## The recipe, not the artifact

Every channel on this page is delivered as **source plus a build script**,
never as a published binary. That is a deliberate design decision, not a gap:

- The artifacts are machine-built binaries (an XCFramework, a set of
  per-ABI `.so` files, a `.wasm` module). Publishing them would mean
  committing binaries to git and maintaining a release matrix for three more
  ecosystems.
- Every developer targeting these platforms already has the toolchain
  (Xcode, Android Studio + the NDK, Node). "Clone and run one script" is a
  smaller ask than it looks.
- One source of truth. The same Rust source that is published to crates.io
  and compiled into the PyPI wheels is compiled here — there are no parallel
  implementations to keep in sync.

None of these builds ever run in CI, by design (see `CLAUDE.md`, *Target
Architecture*): no committed binaries, and the engine is always built
**without** the `python` feature.

## The channels

| Audience | How it is delivered | Get it |
|---|---|---|
| Learners working through the bundled notebooks | **GitHub** ([`notebooks/`](https://github.com/miguelju/vle/tree/main/notebooks)) | clone the repo, `pip install "vle-thermo[plot]" jupyterlab`, open in any notebook viewer — no special kernel needed |
| Swift developers building iOS/macOS apps | **source + local build** | `scripts/build-ios.sh` → add `swift/VleThermo` as a local package in Xcode — see [docs/en/ios/README.md](../docs/en/ios/README.md) |
| Kotlin developers building Android / Compose Desktop apps | **source + local build** | `scripts/build-android.sh` → include `kotlin/VleThermo` from Android Studio — see [docs/en/android/README.md](../docs/en/android/README.md) |
| JavaScript/TypeScript developers building web, desktop (Tauri/Electron), or mobile (Tauri/Capacitor) apps | **source + local build** | `scripts/build-wasm.sh` → `npm install <path-to-vle>/wasm/pkg` — see [docs/en/web/README.md](../docs/en/web/README.md) |

## Notebooks from the repo (work through them locally)

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

You don't have to clone the whole repository, either — three lighter ways to
get just the notebooks:

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

**Kernel:** nothing custom is required. The notebooks declare their
kernel as `python3` (the standard `ipykernel` name), so any Jupyter
install with a Python 3.10+ environment that has `vle-thermo` installed
will run them. There's nothing `vle`-specific about the kernel itself.

For the full host-agnostic prerequisites, install options, the notebook
catalogue, and the `components.db` setup, see [`NOTEBOOKS.md`](NOTEBOOKS.md).

> The `vle-thermo[plot]` extra pulls in `matplotlib`, which the
> notebooks use for P-x-y plots. Skip the `[plot]` if you only care
> about the numerical cells.

## Swift — iOS and macOS apps

The engine compiles into a Swift package for native Apple apps (steam
tables, the component database, and the mixture flash suite) through
**UniFFI**, which generates the Swift bindings from the `ffi/` wrapper
crate. One script builds all three Apple slices, runs the UniFFI bindgen,
and assembles `VleFFI.xcframework` — all of it gitignored:

```sh
git clone https://github.com/miguelju/vle.git && cd vle
scripts/build-ios.sh            # → swift/VleThermo
```

Then add `swift/VleThermo` as a local package in Xcode. Theory and the
step-by-step guide: [docs/en/ios/README.md](../docs/en/ios/README.md).
Design record: [`docs/plans/delivery/IOS_FFI_PLAN.md`](../docs/plans/delivery/IOS_FFI_PLAN.md).

## Kotlin — Android and Windows desktop apps

The same `ffi/` wrapper crate also generates a **Kotlin** library, which
covers two targets from one codebase: a native **Android** app (Jetpack
Compose) and a **Windows desktop** app (Compose Multiplatform on the
desktop JVM — a real `.exe`, no emulator). The script cross-compiles the
per-ABI `.so` files with `cargo-ndk`, builds a host library for the desktop
JVM, and runs the first-party UniFFI Kotlin bindgen:

```sh
git clone https://github.com/miguelju/vle.git && cd vle
scripts/build-android.sh        # → kotlin/VleThermo
```

Then open `kotlin/` in Android Studio. Guide:
[docs/en/android/README.md](../docs/en/android/README.md). Design record:
[`docs/plans/delivery/ANDROID_FFI_PLAN.md`](../docs/plans/delivery/ANDROID_FFI_PLAN.md).
Status: code complete, with the first Android Studio run still pending
(see `ROADMAP.md` Milestone 16).

## WebAssembly — the browser and JS runtimes

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
Design record: [`docs/plans/delivery/WEB_UI_PLAN.md`](../docs/plans/delivery/WEB_UI_PLAN.md).

> **What about C#/.NET?** That route was evaluated and is deliberately
> **not** offered: the community C# bindings generator lags the UniFFI
> version this workspace pins, so the toolchain doesn't line up (as of
> 2026-07-12). The full analysis — and the complete would-be recipe, should
> the versions ever converge — lives in
> [docs/en/dotnet/README.md](../docs/en/dotnet/README.md).

## Layout

```
distribution/
├── README.md       this file — every non-registry channel
└── NOTEBOOKS.md    host-agnostic guide to running the bundled notebooks
```

The build scripts themselves stay with the rest of the repo's tooling in
`scripts/` — `build-ios.sh`, `build-android.sh` and `build-wasm.sh` are named
by path from `Package.swift`, `build.gradle.kts`, the `ffi/` and `wasm/`
manifests, and `.gitignore`, so they belong next to their siblings there.
Each one is documented in its own per-platform guide above, and carries a
usage header at the top of the file. (`scripts/README.md` covers only the two
data-extraction scripts.)
