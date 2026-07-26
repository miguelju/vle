# Web UI Plan — `vle-wasm` → the browser via wasm-bindgen (Milestone 17)

*Adopted and executed 2026-07-12. The third sibling of
[IOS_FFI_PLAN.md](IOS_FFI_PLAN.md) (M15) and
[ANDROID_FFI_PLAN.md](ANDROID_FFI_PLAN.md) (M16): same source-only
philosophy, third consumer language (TypeScript/JavaScript). Tracked as
Milestone 17 / Phase 24 (ROADMAP.md / TODO.md / MODERNIZATION_PLAN.md
synced in the milestone commit).*

## Goal

Make the vle engine callable from **JavaScript/TypeScript**, so one
WebAssembly build can power:

- a **pure React website** — the engine runs *client-side* in the
  visitor's browser; the site is static files with zero server compute,
- a **Windows desktop app** — the same React bundle wrapped in **Tauri 2**
  or **Electron**,
- an **Android app** — the same bundle wrapped in **Tauri 2** or
  **Capacitor** (Electron is desktop-only),
- interactive **3-D phase surfaces** (the README heroes, live) via
  plotly.js/three.js — WebGL is the strongest 3-D plotting stack on any
  platform in this repo.

The repo ships **source + one build script only** — exactly the M15/M16
rules: no CI involvement (`release.yml` untouched), no committed binaries,
**nothing published to npm**, and no app in this repo (the React app lives
in a future separate repo, like the SwiftUI and Compose apps).

## Why wasm + web-shells (decision log, 2026-07-12)

Evaluated for the "Windows + Android + Web UI" triple:

| Route | Verdict |
|---|---|
| **Flutter** | Rejected — needs a second Rust bindgen (`flutter_rust_bridge`) parallel to UniFFI, adds Dart, and its 3-D story (`flutter_scene`) is preview-grade on the master channel as of mid-2026. |
| **React Native** (+ react-native-windows) | Rejected — its Windows leg is the framework's least-used limb (Microsoft itself pushed WinUI at Build 2026), and with no DOM there is no plotly.js: the weakest 3-D story of every option. (`uniffi-bindgen-react-native` would reuse our UniFFI setup if this is ever revisited.) |
| **Kotlin / Compose Multiplatform** (M16) | Kept as the *native* path — stable on Android + desktop JVM, but web support is Beta/canvas-rendered, 3-D is niche (Korender/Materia), and it can't cover the website. Remains the escape hatch if a webview UI ever disappoints on Android. |
| **React + wasm, wrapped per platform** | **Chosen.** One TypeScript/React/plotly.js codebase covers website, Windows, and Android; the engine compiles to wasm **unchanged** (spike verified, below); React is the largest UI ecosystem in existence; and the shell choice (Tauri vs Electron vs Capacitor) becomes a low-stakes, late-binding decision — no app code moves between them. |

**Shell notes:** Tauri 2 (stable since 2024-10, mobile targets first-class)
is the leading candidate for both Windows and Android since one config
covers both; Electron is the mature desktop-only alternative (~25× larger
installer, but the biggest corpus); Capacitor is the well-trodden
web-to-Android wrapper. Because the engine runs *inside* the webview as
wasm, none of the shells needs any backend code — they are packaging, not
architecture. The decision is deferred to the app repo.

## Feasibility spike (2026-07-12) — verified

`cargo build --target wasm32-unknown-unknown -p vle-thermo --features
steam,component-db` compiled **cleanly with zero changes** — everything
wasm-hostile (pyo3, numpy, rayon) is already gated behind the `python`
feature; the default stack (nalgebra, num-dual, smallvec, uom, vle-units,
vle-steam, serde) is pure-Rust math. A throwaway `cdylib` calling the real
engine, executed in Node.js:

| Call | Result | Reference |
|---|---|---|
| Ch. IV Table 4.10 flash (n-heptane/n-butane, RKS, 300 K, 100 kPa), β | 0.19669 | thesis 0.19889 — within the 1–5 % band (bundled-DB Tc/Pc/ω vs the thesis's) |
| IF97 `psat(373.15 K)` | 101.418 kPa | exact |
| IF97 `psat(647.096 K)` | 22 064.000 kPa | exact (critical point) |

Release binary with the full flash suite + steam tables + embedded
25-compound DB: **225 KB (89 KB gzipped)** with `opt-level = "s"`, `lto`,
`codegen-units = 1` — smaller than the README's hero PNGs.

Single-threaded speed (Node.js, same binary): **5.7 µs per complete
Table 4.10 flash** — including the per-call DB lookup and spec rebuild, so
an upper bound. A 50×50 P–x–y surface (~2,500 flashes) is **~14 ms**,
under one 60 fps frame; a dense 200×200 P–v–T grid is ~230 ms.

## Hard constraints (inherited from M15/M16)

1. **Source-only repo.** The wasm-pack output (`wasm/pkg/`: `.wasm`,
   JS glue, `.d.ts`) and `node_modules/` are gitignored. Committed: the
   wrapper crate source, the build script, smoke tests, docs.
2. **No CI, no registries.** All builds are local
   (`scripts/build-wasm.sh`); `release.yml` untouched; nothing published
   to npm — the app repo consumes `wasm/pkg` by local path
   (`"vle-wasm": "file:../vle/wasm/pkg"`), mirroring how Xcode and Android
   Studio consume `swift/VleThermo` / `kotlin/VleThermo`.
3. **Engine without `python`.** The wrapper depends on `vle-thermo` with
   `component-db` + `steam`, never pyo3 — same as `vle-ffi`.
4. **No app here.** The React app (and its Tauri/Electron/Capacitor
   wrapping) is a future separate repo.
5. **Single-threaded wasm** — a deferral with a documented upgrade path,
   not a wall. The Python batch API's "multithread trick" (M10, Track D)
   is really two tricks, and they map to the browser separately:
   - *GIL release* (responsiveness — other threads keep running while
     Rust computes) maps to running the engine in a **plain Web Worker**
     off the main thread. Trivial, no special build, and it's the half
     users feel: this **is** the M17 pattern, documented in the guide.
   - *rayon* (throughput — N× cores on big sweeps) maps to wasm threads
     via `wasm-bindgen-rayon`. Deferred: it needs an atomics-enabled
     nightly `-Z build-std` toolchain, and SharedArrayBuffer only exists
     on **cross-origin-isolated** pages (COOP/COEP headers on every
     response — host config that must also be replicated in each shell's
     webview, and that breaks non-CORP third-party embeds).
   - The spike numbers say the interactive use case doesn't need rayon
     (14 ms per surface grid single-threaded); only 100k-point
     README-style batch sweeps would (~570 ms → ~70–100 ms on 8 cores).
     Revisit if an in-browser bulk workload (e.g. a live kij-regression
     playground) appears; in Tauri/Electron a native sidecar/napi module
     with real rayon is the alternative that skips wasm threads entirely.
   - The third Python trick — zero-copy array crossings (rust-numpy) —
     carries over for free as `Float64Array` views (Architecture, above).

## Architecture

A new thin wrapper crate — UniFFI has no JS backend in our pinned version,
and wasm-bindgen is the standard, so this is a **sibling** of `ffi/`, not
an extension of it:

- `wasm/` (`vle-wasm`, `publish = false`, `crate-type = ["cdylib"]`) —
  depends on `vle-thermo` (features `component-db`, `steam`) +
  `wasm-bindgen`, `serde`, `serde-wasm-bindgen`, `js-sys`.
  Release profile: `opt-level = "s"`, `lto = true`, `codegen-units = 1`.
- **Exported API mirrors `vle-ffi`'s surface** (same names where the
  languages allow, so the three guides read as one family):
  - `version()`;
  - component DB: `dbAvailable(): string[]`,
    `dbComponent(name): ComponentData | undefined`, plus custom
    components from literals;
  - steam: `waterTP(t, p)`, `waterTX`/`waterPX`/`waterPH`/`waterPS`,
    `psat`/`tsat`, saturation-table rows;
  - `VleSystem` (a `#[wasm_bindgen]` struct holding components + model
    choices): `flashTp`, `bubbleP/T`, `dewP/T`, `kValues` — 22 cubic EOS,
    6 activity models, 11 mixing rules, same enums.
  - Result structs cross the boundary via `serde-wasm-bindgen` (plain JS
    objects with TypeScript definitions); compositions and surface grids
    as `Float64Array` (one copy, no per-element chatter). Geometry for
    3-D plots (P–x–y sheets, envelope polylines, P–v–T grids) is computed
    **Rust-side** and returned as flat arrays — the React layer only
    renders.
  - Errors: `FlashError`/`SteamError` → thrown JS `Error`s with the Rust
    message (wasm-bindgen's `Result<T, JsError>`), the analog of M15/M16
    error mapping.
- Canonical units at this boundary, as everywhere: **K** and **kPa
  absolute**. Unit-string parsing stays a UI-layer concern (document
  the conversion snippet in the guide; don't port `pint`).
- `docs/en/web/README.md` — the learning guide, sibling of
  `docs/en/ios/README.md` and `docs/en/android/README.md`: what
  wasm/wasm-bindgen are, how a call crosses the JS↔wasm boundary, how
  this differs from UniFFI, Web Worker pattern, and the quickstarts for
  React web, Tauri, Electron, and Capacitor.

## Build pipeline (`scripts/build-wasm.sh`, idempotent)

1. `rustup target add wasm32-unknown-unknown` (no-op when present).
2. `wasm-pack build wasm/ --target web --release` →
   `wasm/pkg/` (`vle_wasm_bg.wasm`, ES-module JS glue, full TypeScript
   `.d.ts`, `package.json`). `--target web` produces a universal ES
   module that works in Vite/webpack bundlers *and* from a plain
   `<script type="module">` — one artifact for all four consumers.
3. Print next steps (the `file:` install line + guide link), mirroring
   the iOS/Android scripts.

Prerequisite: `wasm-pack` (`cargo install wasm-pack`), checked with a
friendly error like cargo-ndk in `build-android.sh`.

## Verification ladder (all local)

1. `cargo test -p vle-wasm` — host-side unit tests of the wrapper logic.
2. `wasm-pack test --node wasm/` — **5 smoke tests through the real wasm
   boundary** (`wasm-bindgen-test`), the analog of the M16 JNA smoke
   tests: version string, water DB lookup, IF97 1-atm boiling point,
   Ch. IV Table 4.10 heptane/butane flash, error mapping
   (`flashTp` with bad mole fractions throws).
3. Browser sanity: serve a scratch `index.html` importing `wasm/pkg`
   (not committed), confirm the same numbers in a real browser console.
4. The future app repo adds the plotly.js rendering layer on top.

## deploy/README.md update (required step)

Extend the distribution story with the new channel, keeping the "source +
recipe, not artifacts" framing:

1. **New row in the "How `vle-thermo` reaches users" table:**
   *JavaScript/TypeScript developers building web, desktop (Tauri/
   Electron), or mobile (Tauri/Capacitor) apps* — GitHub (source) + local
   build: clone, `scripts/build-wasm.sh`, then `npm install` the
   generated `wasm/pkg` by path — see docs/en/web/README.md.
2. **New section "WebAssembly — the browser and JS runtimes"** (sibling
   of the crates.io/PyPI sections) briefly explaining:
   - what the wasm build is (the same Rust source compiled to a portable
     binary that any modern browser/Node executes; ~90 KB gzipped);
   - **React web**: import the package, `await init()`, call the engine
     client-side — the site stays static files, no server;
   - **Tauri 2**: the same React bundle as a Windows *and* Android app;
     with the engine as wasm inside the webview, the Rust backend needs
     no commands — Tauri is packaging only;
   - **Electron**: same bundle as a desktop-only app (Chromium bundled;
     bigger but maximally battle-tested); optional `napi-rs` native
     module as a future full-native-speed upgrade path;
   - **Capacitor**: same bundle wrapped for Android (and iOS) when only
     mobile is needed;
   - one-line steer: shells are packaging decisions made in the app repo,
     not here.
3. Keep the **C#/.NET parked-route note** (added 2026-07-12, below the
   channels table): evaluated but not offered — bindgen version-blocked —
   with the pointer to [docs/en/dotnet/README.md](../../en/dotnet/README.md).
   The wasm section must not imply .NET is a supported consumer.
4. Update the `deploy/` layout tree at the bottom if it gains no new
   files (it shouldn't — the guide lives in `docs/en/web/`).

The update also touches the root `README.md` (a "JavaScript / Web" install
subsection next to Swift and Kotlin) — same commit.

## Milestone mapping

Tracked as **Milestone 17 / Phase 24** (ROADMAP.md, TODO.md,
MODERNIZATION_PLAN.md — including the Milestone 0 phase-count line). Like
M15/M16: no release (nothing on crates.io/PyPI changes), no milestone
notebook (JS isn't executable from Jupyter — the learning doc + smoke
tests fill that role). `.gitignore` gains `wasm/pkg/` and
`node_modules/`.

## Explicitly out of scope (future repos / milestones)

- The React app itself (components, plotly.js scenes, routing, styling).
- Shell selection + packaging (Tauri vs Electron vs Capacitor) and store
  distribution.
- npm publication (revisit only if a downstream consumer can't build
  locally — would need a `PUBLISHING.md` + release.yml change and its own
  decision log).
- wasm threads / SharedArrayBuffer parallelism — the throughput half of
  the Python batch trick (see Hard constraint 5 for the full decomposition
  and the `wasm-bindgen-rayon` + COOP/COEP upgrade path). The plain Web
  Worker pattern in the docs is the intended M17 answer.
- PWA packaging (free once the website exists; an app-repo concern).
