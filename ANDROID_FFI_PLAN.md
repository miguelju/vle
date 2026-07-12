# Android/Kotlin FFI Plan — `vle-ffi` → Kotlin via UniFFI (Milestone 16)

*Adopted 2026-07-12. The sibling of [IOS_FFI_PLAN.md](IOS_FFI_PLAN.md) (M15):
same wrapper crate, same source-only philosophy, second consumer language.*

## Goal

Make the vle engine callable from **Kotlin**, so one codebase can power:

- a native **Android app** (Jetpack Compose, built in Android Studio), and
- a **Windows desktop app** via **Compose Multiplatform** (the same Compose
  UI running on the desktop JVM — a real `.exe`, no emulator).

The repo ships **source + one build script only** — exactly the M15 rules:
no CI involvement (`release.yml` untouched), no committed or published
binaries, no Android app in this repo (the app lives in a future separate
repo, like the SwiftUI app).

## Why Kotlin + Compose (decision log, 2026-07-12)

Evaluated for the "Windows desktop + Android app" pair:

| Route | Verdict |
|---|---|
| **.NET MAUI** | Rejected — its strength is native mobile feel, not Windows desktop; 2026 stability record is poor (regression-heavy releases, team layoffs); and the C# bindgen is blocked anyway (below). |
| **Avalonia** | Strong on desktop, but its Android community is very small, and the same C# bindgen blocker applies. |
| **Windows Subsystem for Android** ("just run the APK on Windows") | Dead — Microsoft removed WSA from the Store on 2025-03-05. Emulators are a dev tool, not a distribution channel. |
| **Kotlin + Jetpack Compose / Compose Multiplatform** | **Chosen.** First-party UniFFI Kotlin bindgen (zero version-skew risk — it ships inside the `uniffi` crate this workspace already pins), the healthiest Android ecosystem, and Compose Multiplatform gives the Windows desktop app from the same UI code. Android Studio has first-class Claude Code support (JetBrains plugin) and Google's agent-friendly Android CLI. |

### The C#/.NET route: documented, deliberately parked

`uniffi-bindgen-cs` (NordSecurity) tops out at **uniffi 0.31** while this
workspace pins **`uniffi = "0.32"`**, and there are **no plans to downgrade**.
Status, options, and the full would-be route live in
[docs/en/dotnet/README.md](docs/en/dotnet/README.md) (dated 2026-07-12 —
re-check before acting on it).

## Hard constraints (inherited from M15)

1. **Source-only repo.** Generated Kotlin (`vle_ffi.kt`), cross-compiled
   `.so` trees (`jniLibs/`), Gradle caches, and the Gradle wrapper (it
   contains a `.jar`) are all gitignored. Committed: Gradle build files,
   the smoke test, the build script, docs.
2. **No CI.** All builds are local (`scripts/build-android.sh`);
   `release.yml` is untouched.
3. **Engine without `python`.** `vle-ffi` already depends on `vle-thermo`
   with `component-db` + `steam` and never pyo3 — unchanged.
4. **No app here.** The Compose app (Android + Windows desktop) is a future
   separate repo consuming `kotlin/VleThermo` by path.

## Architecture

Everything reuses the M15 wrapper — **zero new FFI surface**:

- `ffi/` (`vle-ffi`) gains `"cdylib"` in `crate-type`: Kotlin/JNA loads a
  *shared* library at runtime (`libvle_ffi.so` on Android/Linux,
  `libvle_ffi.dylib` on macOS, `vle_ffi.dll` on Windows), unlike Xcode,
  which links the `staticlib`.
- `ffi/uniffi-bindgen/` gains a second 3-line binary, `uniffi-bindgen`
  (uniffi's *general* CLI: `generate --language kotlin`), next to the
  Swift-specific `uniffi-bindgen-swift`. Same version-lockstep guarantee:
  one workspace `Cargo.lock` resolves generator and scaffolding to the
  same uniffi 0.32.
- `ffi/uniffi.toml` gains `[bindings.kotlin]`:
  `package_name = "dev.migueljackson.vle.ffi"`, `cdylib_name = "vle_ffi"`.
  `android = true` is deliberately **not** set — the default plain-JVM
  flavor runs on Android *and* on the desktop JVM (Compose for Desktop,
  Gradle unit tests), which is the whole point.
- `kotlin/VleThermo/` — Android library module (the analog of
  `swift/VleThermo`): hand-written Gradle files + 5 committed smoke tests;
  generated bindings and `jniLibs/` dropped in by the script. JNA
  (`net.java.dev.jna`) is its one runtime dependency.

The exported API is exactly M15's: `version()`, the component DB
(`dbAvailable`/`dbComponent` — read-only catalogue; **custom components are
fully supported** by constructing `ComponentData` from literals and passing
them to `VleSystem(components:)`), steam tables, and the `VleSystem` object
(22 cubic EOS, 6 activity models, 11 mixing rules, `flashTp`,
`bubbleP/T`, `dewP/T`, `kValues`).

## Build pipeline (`scripts/build-android.sh`, idempotent)

1. `cargo ndk -t arm64-v8a -t x86_64 -o kotlin/VleThermo/src/main/jniLibs
   build -p vle-ffi --release` — one `.so` per ABI, laid out the way AGP
   expects. arm64-v8a = every modern device + the Apple Silicon emulator;
   x86_64 = Intel/Windows emulators. `ABIS=…` env var overrides.
2. `cargo build -p vle-ffi --release` — the **host** shared library, used
   as bindgen input, by Gradle unit tests (`jna.library.path`), and by
   Compose for Desktop during development.
3. Library-mode Kotlin bindgen from the compiled host artifact →
   `kotlin/VleThermo/src/main/kotlin/dev/migueljackson/vle/ffi/vle_ffi.kt`.
4. Host-JVM smoke tests (`gradle :VleThermo:test`) if a Gradle is
   available — the analog of `swift test` on the macOS slice.

**Windows leg (for Compose Desktop packaging):** on the Windows machine,
`cargo build -p vle-ffi --release` produces `vle_ffi.dll`; the desktop app
bundles it (details in [docs/en/android/README.md](docs/en/android/README.md)).

## Verification ladder (all local)

1. `cargo test -p vle-ffi` — unchanged M15 Rust wrapper tests.
2. 5 host-JVM smoke tests through the real JNA boundary
   (`kotlin/VleThermo/src/test/…/VleThermoSmokeTest.kt`): version string,
   water lookup, IF97 1-atm boiling point, Ch. IV heptane/butane flash,
   error mapping.
3. Android Studio: open `kotlin/`, run the same tests, then an emulator
   smoke test from the future app repo.

## Milestone mapping

Tracked as **Milestone 16 / Phase 23** (ROADMAP.md, TODO.md,
MODERNIZATION_PLAN.md). Like M15: no release (nothing on crates.io/PyPI
changes), no milestone notebook (Kotlin isn't executable from Jupyter — the
learning doc + smoke tests fill that role).
