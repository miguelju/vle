# Building the vle engine for iOS & macOS (Rust → Swift via UniFFI)

This guide takes you from `git clone` to calling the thermodynamics engine
from Swift — and explains *why* each step exists, for a reader who knows a
little Rust or Swift but has never done cross-language FFI. Everything
builds **locally on a Mac**; no CI is involved by design (IOS_FFI_PLAN.md).

---

## 1. The one-paragraph overview

The engine (`engine/`, `steam/`) is pure Rust. Rust can't be called from
Swift directly, but both languages can speak **C**. So we compile a small
wrapper crate (`ffi/`) into a C-compatible **static library** for each
Apple target, let **UniFFI** generate the Swift code that talks to that C
layer, bundle the libraries into a **`VleFFI.xcframework`**, and wrap the
whole thing in a normal Swift package (`swift/VleThermo/`) that any Xcode
app imports like any other dependency.

```
 ffi/ (Rust wrapper)──cargo──▶ libvle_ffi.a  ×3 targets
        │                          │
        │ uniffi-bindgen-swift     │ xcodebuild -create-xcframework
        ▼                          ▼
 VleFFI.generated.swift  +  VleFFI.xcframework
        └──────────┬───────────────┘
             swift/VleThermo  (Swift package)
                   │  "Add Local Package…"
             your iOS / macOS app
```

## 2. Why the binaries are not in the repo

You will not find `VleFFI.xcframework` on GitHub — only the source that
produces it. Three reasons:

- **Size.** The XCFramework is ~60 MB (three fully-optimized copies of the
  engine). Git keeps every version of every file forever, so committing it
  would balloon the repository within a few releases.
- **Reviewability.** Nobody can code-review a binary. Anyone should be able
  to reproduce the artifact from the source they *can* review — the same
  philosophy as the rest of this repo's CI/CD ("the operator path always
  works").
- **You need the toolchain anyway.** The only people who consume this
  artifact are building an Apple app, which means they already have Xcode —
  and rebuilding is one command.

## 3. Prerequisites (one-time setup)

1. **Xcode** (from the App Store) plus command-line tools:

   ```sh
   xcode-select --install
   ```

2. **Rust** via [rustup](https://rustup.rs):

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **The three Apple compile targets.** Your default Rust toolchain only
   knows how to produce code for *this Mac*; these add the iOS instruction
   encodings and platform libraries:

   ```sh
   rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin
   ```

## 4. Quick start: clone → build → test

```sh
git clone https://github.com/miguelju/vle.git
cd vle
scripts/build-ios.sh
```

That single script (≈5 minutes the first time, mostly Rust release
compilation) ends with the package's XCTest suite running natively on your
Mac. If you see `DONE: …/VleFFI.xcframework`, everything works. Re-run it
after any Rust change — it is idempotent.

To use it from an app: open your app project in Xcode → **File → Add
Package Dependencies… → Add Local…** → select `vle/swift/VleThermo` → add
the `VleThermo` product to your app target. Then:

```swift
import VleThermo

// Steam tables (IAPWS-IF97). Canonical units: K, kPa absolute, kJ/kg.
let boiling = try steamSatP(p: 101.325)
print(boiling.t)        // 373.12 K
print(boiling.hFg)      // 2256.5 kJ/kg (latent heat)

// Mixture VLE straight from the bundled component database.
let sys = try VleSystem.fromDb(
    names: ["n-heptane", "n-butane"],
    vapor: .cubic(eos: .rks1972),
    liquid: .cubic(eos: .rks1972),
    mixingRule: .classical,
    options: nil
)
let flash = try sys.flashTp(t: 300.0, p: 100.0, z: [0.5, 0.5])
print(flash.beta)       // vapor fraction
```

One app project can target **both iOS and macOS**: create it as a
*Multiplatform* SwiftUI app in Xcode and the same package serves both —
the XCFramework carries a native slice for each. (An iOS-only app also
runs on Apple Silicon Macs in "Designed for iPhone" mode, but a real macOS
destination gets you native windows and menus for free.)

---

## 5. The theory: how Rust talks to Swift

### 5.1 The C ABI — the lingua franca

Rust and Swift each have their own memory layouts, name mangling, and
calling conventions, and neither understands the other's. What they *both*
understand is the **C ABI** (Application Binary Interface): the decades-old
contract for "here is how a function is named in the binary, how arguments
go in registers/stack, how results come back". Cross-language FFI between
almost any two compiled languages funnels through C for this reason.

So the recipe is always: Rust exposes `extern "C"` functions operating on
C-representable data (integers, floats, raw pointers and byte buffers) →
Swift imports the matching C header → each side converts between its rich
native types and those primitives. Those conversions are called **lowering**
(rich → C) and **lifting** (C → rich) in UniFFI vocabulary.

### 5.2 What UniFFI does for us

Writing that boundary by hand means hundreds of `unsafe` conversions —
tedious and easy to get fatally wrong (double-free, use-after-free, encoding
bugs). [UniFFI](https://mozilla.github.io/uniffi-rs/) (Mozilla; used in
production by Firefox and 1Password) generates both sides from annotations
in `ffi/src/`:

| You write (Rust)                          | Swift sees                              |
|-------------------------------------------|-----------------------------------------|
| `#[derive(uniffi::Record)] struct …`       | a value `struct` with the same fields   |
| `#[derive(uniffi::Enum)] enum …`           | an `enum` (associated values supported) |
| `#[derive(uniffi::Object)] struct …`       | a `class` backed by a Rust `Arc`        |
| `#[derive(uniffi::Error)] enum …`          | an `Error` enum you can `catch`         |
| `#[uniffi::export] fn … -> Result<T, E>`   | a `throws` function returning `T`       |

Two details worth internalizing:

- **Objects vs records.** A record is *copied* across the boundary every
  call (fine for results like `FlashSplit`). An object (`VleSystem`) stays
  in Rust memory; Swift holds a reference, and Rust's `Arc` reference count
  is decremented when the Swift object deinits — that's the memory-safety
  story, and why you never call `free()` from Swift.
- **Panics don't crash the app.** UniFFI's scaffolding catches a Rust panic
  at the boundary and converts it into a Swift error (this is why the
  workspace keeps Rust's default unwinding panic mode).

We use UniFFI's **proc-macro mode** (annotations directly on the Rust code)
rather than the older separate `.udl` interface file — one source of truth,
nothing to keep in sync. And bindings are generated in **library mode**:
`ffi/uniffi-bindgen` reads the interface metadata out of the *compiled*
`libvle_ffi.a`, so the generated Swift can never drift from the binary.
(A shortcut tool, [cargo-swift](https://github.com/antoniusnaumann/cargo-swift),
automates this whole pipeline; we keep the steps explicit so they stay
teachable.)

### 5.3 Why a separate `ffi/` crate

The engine's idiomatic Rust API (borrowed slices, generics, data-carrying
enums) is not FFI-shaped — UniFFI wants owned `Vec`s, flat records, simple
enums. `ffi/` defines those flat mirrors (`ComponentData`, `SteamStateData`,
`FlashSplit`, …) and converts. It also keeps `uniffi` out of the published
crates' dependency tree entirely, exactly like the `python` feature keeps
pyo3 optional — and critically, the iOS build compiles the engine **without
the `python` feature**, so pyo3/Python never enters the Apple graph.

### 5.4 Static libraries, and why three of them

A **static library** (`.a`) is an archive of compiled object code that gets
*copied into* the app binary at link time — the right choice on iOS, where
apps are sandboxed single bundles (dynamic linking to third-party dylibs is
heavily restricted). The trade-off is that a `.a` is compiled for exactly
one target, and Apple's targets are stricter than "one per CPU":

- `aarch64-apple-ios` — physical iPhones/iPads
- `aarch64-apple-ios-sim` — the iOS **simulator** on Apple Silicon
- `aarch64-apple-darwin` — native macOS

The first two run on the *same arm64 CPU* but are **different platforms**:
different availability rules, different platform libraries, a deliberate
marker in the binary so you can't ship simulator code to a device. That's
why "just one arm64 build" doesn't exist. (Historical footnote: when a
single *platform* needed multiple *CPU architectures* — e.g. Intel + ARM
simulators — you'd merge those with `lipo` into one fat library *before*
framework assembly. We skip Intel entirely; this is an Apple Silicon repo.)

### 5.5 The XCFramework

An **XCFramework** is Apple's container for "the same library, once per
platform". Look inside the generated one — it's just a directory:

```
VleFFI.xcframework/
├── Info.plist                     # index: which slice for which platform
├── ios-arm64/
│   ├── libvle_ffi.a
│   └── Headers/
│       ├── vle_ffiFFI.h           # the C header UniFFI generated
│       └── module.modulemap       # tells Swift "these headers = module VleFFI"
├── ios-arm64-simulator/…
└── macos-arm64/…
```

Xcode reads `Info.plist` and links the right slice for whatever you're
building — device, simulator, or Mac. No conditional build settings in your
app, ever.

The **`module.modulemap`** deserves its gotcha callout: Swift does not
import bare C headers; it imports *modules*, and a file named exactly
`module.modulemap` is how a headers directory declares one. Get the name
wrong and you'll stare at `no such module 'VleFFI'`. The module name must
agree in three places (our build script enforces this): `ffi_module_name`
in `ffi/uniffi.toml` (what the generated Swift `import`s), `--module-name`
passed to the bindgen (what the modulemap declares), and the
`binaryTarget(name:)` in `Package.swift`. A second subtlety we hit while
building this: the bindgen's `--xcframework` flag emits a
`framework module` declaration, which is only correct when slices are
`.framework` bundles — for bare static-library slices like ours the plain
`module` form is required.

### 5.6 Reading the generated code (recommended!)

After a build, skim
`swift/VleThermo/Sources/VleThermo/VleFFI.generated.swift` — it is verbose
but readable, and it demystifies everything above. Find `func steamSatP`
and follow it: arguments are **lowered** into a `RustBuffer` (a length +
pointer struct), the `extern "C"` scaffolding function is called, the
result buffer is **lifted** field-by-field back into the `SatPropsData`
struct, and a status word is checked to decide whether to `throw
VleFfiError`. Hand-writing that for every function is the job you just
delegated to UniFFI.

## 6. What `scripts/build-ios.sh` actually does

| Step | Command (abridged) | Why |
|---|---|---|
| 1 | `cargo build -p vle-ffi --release --target …` ×3 | one static lib per Apple platform variant |
| 2 | `cargo run -p vle-uniffi-bindgen -- …ios/release/libvle_ffi.a --swift-sources` | generate the Swift wrapper *from the compiled artifact* |
| 3 | same, `--headers --modulemap --module-name VleFFI --modulemap-filename module.modulemap` | the C header + module declaration each slice carries |
| 4 | `xcodebuild -create-xcframework -library … -headers … -output …` | bundle the three slices |
| 5 | `cp …/vle_ffi.swift swift/VleThermo/Sources/VleThermo/VleFFI.generated.swift` | the wrapper source joins the Swift package |
| 6 | `swift test` | XCTests cross the real FFI boundary on the macOS slice — no simulator needed |

Deployment targets are pinned in the script (`iOS 16.0`, `macOS 13.0`) and
mirrored in `Package.swift` — change them together.

All of steps 1–5's outputs are gitignored (`build/`, `*.xcframework`,
`*.generated.swift`). What *is* committed: `ffi/` (Rust source),
`ffi/uniffi.toml`, the script, `Package.swift`, `Extensions.swift`
(hand-written Swift sugar), and the tests.

## 7. The exported API, in one screen

Everything speaks **canonical engine units**: temperature **K**, pressure
**kPa absolute**, compositions as mole fractions. Unit conversion is a UI
concern — do it in Swift (see `Extensions.swift` for °C/bar conveniences).

- `version() -> String` — smoke test.
- **Component database**: `dbAvailable()`, `dbComponent(name:)` →
  `ComponentData` (Tc, Pc, ω, M, Antoine coefficients, …).
- **Steam tables** (IAPWS-IF97): `steamTp`, `steamTx`, `steamPx`,
  `steamPh`, `steamPs` → `SteamStateData`; `steamSatT`, `steamSatP` →
  `SatPropsData`; `steamLatentHeat`.
- **Mixture VLE**: `VleSystem` (init with explicit `ComponentData` or
  `.fromDb(names:…)`; 22 cubic EOS, 6 activity models, 11 mixing rules as
  Swift enums) with `flashTp`, `bubbleP`, `bubbleT`, `dewP`, `dewT`,
  `kValues`.
- **Errors**: every failure is a typed `VleFfiError` case — `NotFound`,
  `InvalidInput`, `Flash`, `Steam` — with a message.

Deliberately *not* exported in v1: kij regression, phase envelopes, batch
APIs, unit-string parsing (see IOS_FFI_PLAN.md §3).

## 8. Extending the API

The FFI analog of the repo's PyO3 rule applies: **new engine functionality
that an app should reach gets its FFI export in the same commit series.**

1. Add the function/record to the right module in `ffi/src/` with
   `#[uniffi::export]` (units in the doc comment — house rule).
2. `cargo test -p vle-ffi` — Rust-side test first.
3. `scripts/build-ios.sh` — regenerates everything; the new symbol simply
   appears in Swift.
4. Add an XCTest in `swift/VleThermo/Tests/` exercising it through the
   boundary.

## 9. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `does not contain expected binary artifact 'VleFFI'` | Fresh clone, no build yet → `scripts/build-ios.sh` |
| `no such module 'VleFFI'` | modulemap missing/misnamed — rebuild; see §5.5 |
| `cannot find 'uniffi_vle_ffi_fn_…' in scope` | generated Swift and library out of sync (or module-name mismatch) — rerun the script so both regenerate |
| `error: cannot find target 'aarch64-apple-ios'` | `rustup target add …` (§3) |
| Swift tests pass but the iOS build fails in Xcode | you edited Rust and rebuilt only the macOS slice by hand — the script always builds all three |
| Simulator app crashes at launch with a link error | app accidentally linked the device slice — clean build folder; Xcode picks slices via the XCFramework automatically |

## 10. References

- [UniFFI user guide](https://mozilla.github.io/uniffi-rs/) — especially
  "Lifting and lowering" and the Swift chapter.
- [`IOS_FFI_PLAN.md`](../../plans/delivery/IOS_FFI_PLAN.md) — the design record for
  this pipeline (alternatives considered, open decisions).
- [Apple: distributing binary frameworks as Swift packages](https://developer.apple.com/documentation/xcode/distributing-binary-frameworks-as-swift-packages)
- The research paper behind the engine:
  [`docs/en/research-paper/`](../research-paper/README.md).
