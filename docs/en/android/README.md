# vle on Android & Windows Desktop — Kotlin via UniFFI

How the Rust engine becomes a Kotlin library (`kotlin/VleThermo`) that an
**Android app** (Jetpack Compose) and a **Windows desktop app** (Compose
Multiplatform) consume. The Kotlin sibling of [the iOS guide](../ios/README.md);
design record: [ANDROID_FFI_PLAN.md](../../plans/delivery/ANDROID_FFI_PLAN.md).

Everything is **built locally and never committed**: the repo ships source +
`scripts/build-android.sh`. No CI builds these artifacts, by design.

## How it works (the 30-second version)

```
ffi/ (vle-ffi, Rust)                     one wrapper crate, two consumers
  │
  ├─ staticlib (.a)  ──► Swift / Xcode          (M15, scripts/build-ios.sh)
  └─ cdylib (.so/.dylib/.dll) ──► Kotlin / JNA  (M16, scripts/build-android.sh)
                                    │
              ┌─────────────────────┴──────────────────────┐
              ▼                                            ▼
   Android app (Jetpack Compose)             Windows/macOS/Linux desktop app
   loads libvle_ffi.so from the APK          (Compose Multiplatform, JVM)
                                             loads vle_ffi.dll / .dylib / .so
```

- The `#[uniffi::export]` attributes in `ffi/src/` make the compiler embed a
  C ABI + interface metadata into the compiled library.
- `ffi/uniffi-bindgen` (binary `uniffi-bindgen`, uniffi's general CLI) reads
  that metadata out of the **compiled** library and generates the Kotlin
  wrapper — data classes, sealed classes, enums, and functions that `throw`
  `VleFfiException`. Bindings can never drift from the code.
- The generated Kotlin calls the C ABI through **JNA** (Java Native Access) —
  no hand-written JNI anywhere.
- One flavor runs everywhere JVM-shaped: the same generated file works on
  Android **and** on the desktop JVM (that's why `android = true` is *not*
  set in `ffi/uniffi.toml`).

## Prerequisites

Works on **macOS, Linux, or Windows**. This Mac and the Windows machine can
each do the full Android side; only the Windows `.exe` leg needs Windows.

1. **Rust** (the workspace toolchain you already have) + Android targets:

   ```sh
   rustup target add aarch64-linux-android x86_64-linux-android
   ```

2. **cargo-ndk** — teaches cargo to drive the NDK's cross-linkers:

   ```sh
   cargo install cargo-ndk
   ```

3. **Android Studio** with an SDK + **NDK** (SDK Manager → SDK Tools → NDK).
   Then export the NDK location if cargo-ndk doesn't find it on its own:

   ```sh
   export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/<version>"   # macOS
   ```

4. (Optional, for command-line tests) a Gradle on PATH, or generate a local
   wrapper once with `gradle wrapper` inside `kotlin/` — the wrapper is
   gitignored because it contains a `.jar` (no binaries in the repo).

## Build

```sh
scripts/build-android.sh                 # ABIs: arm64-v8a + x86_64
ABIS="arm64-v8a" scripts/build-android.sh --skip-tests   # quick device-only
```

The script is idempotent — rerun after any Rust change. It produces (all
gitignored):

| Artifact | Where | Used by |
|---|---|---|
| `libvle_ffi.so` per ABI | `kotlin/VleThermo/src/main/jniLibs/<abi>/` | packaged into the AAR/APK by AGP |
| host `libvle_ffi.dylib`/`.so` | `target/release/` | bindgen input, Gradle unit tests, Compose Desktop dev |
| `vle_ffi.kt` | `kotlin/VleThermo/src/main/kotlin/dev/migueljackson/vle/ffi/` | the entire Kotlin API |

## Consume from Android Studio

`kotlin/` is a standalone Gradle build — **File → Open → `vle/kotlin`** works
directly (Studio will offer its bundled Gradle; accept). For your app repo,
include the module by path in the app's `settings.gradle.kts`:

```kotlin
include(":vlethermo")
project(":vlethermo").projectDir = file("/Users/you/dev/vle/kotlin/VleThermo")
```

then in the app module's `build.gradle.kts`:

```kotlin
dependencies {
    implementation(project(":vlethermo"))
}
```

and call it:

```kotlin
import dev.migueljackson.vle.ffi.*

val sys = VleSystem.fromDb(
    names = listOf("n-heptane", "n-butane"),
    vapor = VaporSpec.Cubic(eos = CubicEosKind.RKS1972),
    liquid = LiquidSpec.Cubic(eos = CubicEosKind.RKS1972),
    mixingRule = MixingRuleKind.CLASSICAL,
    options = null,
)
val r = sys.flashTp(t = 300.0, p = 100.0, z = listOf(0.5, 0.5))   // K, kPa abs
```

Units are the **canonical engine units** everywhere (K, kPa absolute; steam
is mass-basis kJ/kg) — unit conversion belongs in the app layer, same rule
as Swift.

**Custom components:** the bundled 25-compound catalogue is read-only, but
`VleSystem(components = …)` accepts `ComponentData` records you construct
yourself (an "add component" form in the UI), and you can mix catalogue and
custom freely. Persisting the user's components (Room, DataStore, a JSON
file) is the app's job — the engine is stateless.

## The Windows desktop app (Compose Multiplatform)

The same generated Kotlin + JNA loads `vle_ffi.dll` on the desktop JVM:

1. On the Windows machine: clone the repo and run
   `cargo build -p vle-ffi --release` → `target\release\vle_ffi.dll`.
   (Only MSVC build tools + Rust needed — no NDK for this leg.)
2. In the Compose Desktop app, either put the DLL's directory on
   `jna.library.path`, or (for distribution) bundle it as a resource and
   extract-then-load at startup. During development the simplest is:

   ```kotlin
   // main.kt, before first FFI call
   System.setProperty("jna.library.path", "C:/Users/you/dev/vle/target/release")
   ```

3. macOS/Linux desktops work identically with the host library the build
   script already produced — useful for developing the desktop app on this
   Mac and only packaging on Windows.

## Testing

- **Rust side:** `cargo test -p vle-ffi` (unchanged from M15).
- **Kotlin side (host JVM, no emulator):** `cd kotlin && gradle :VleThermo:test`
  — 5 smoke tests through the real JNA boundary (version, water lookup,
  IF97 1-atm boiling, Ch. IV heptane/butane flash, error mapping). The
  script runs these automatically when a Gradle is available; Android
  Studio runs them from the gutter.
- **On-device:** from the app repo, on an emulator (arm64-v8a image on
  Apple Silicon, x86_64 on Intel/Windows) or a physical device.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `UnsatisfiedLinkError: unable to load library 'vle_ffi'` in unit tests | Host library missing — run `scripts/build-android.sh` (or `cargo build -p vle-ffi --release`); `jna.library.path` points at `target/release/` |
| Same error on a device/emulator | Missing ABI in `jniLibs/` — rebuild with the right `ABIS=…` (Apple Silicon emulator = arm64-v8a, Windows/Intel emulator = x86_64) |
| Generated imports are red in the IDE | Bindings not generated yet — run `scripts/build-android.sh` first |
| `uniffi-bindgen` errors about metadata/version | Scaffolding vs generator drift — should be impossible here (one workspace `Cargo.lock`); run `cargo clean -p vle-ffi` and rebuild |
| cargo-ndk can't find the NDK | Set `ANDROID_NDK_HOME` (see Prerequisites) |
| Studio asks for a Gradle wrapper | Expected — the wrapper is gitignored (contains a jar). Let Studio use its bundled Gradle or run `gradle wrapper` once |

## Why is none of this in CI?

Same reasoning as M15: the artifacts are large, rebuildable from source in
one command, and only meaningful on a developer's machine with an app to
consume them. `release.yml` (crates.io + PyPI) is deliberately untouched.
