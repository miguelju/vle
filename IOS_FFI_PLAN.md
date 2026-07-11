# iOS FFI Plan — `vle-ffi` (Rust → Swift via UniFFI, local-only builds)

**Status: IMPLEMENTED (2026-07-11) as Milestone 15 / Phase 22** — renumbered
from the "M14" this plan assumed, per §8.5, because NRTL + ammonia landed as
M14 first. This file remains the *design record* (alternatives considered,
rationale); the as-built state lives in ROADMAP.md M15 /
MODERNIZATION_PLAN.md Phase 22, and the user-facing guide is
[`docs/en/ios/README.md`](docs/en/ios/README.md). Two as-built deviations
from §4: the bindgen's `--module-name` only names the *modulemap* (the
generated Swift's import is set via `ffi_module_name` in `ffi/uniffi.toml`),
and the `--xcframework` flag is not used (it emits a `framework module`
declaration — wrong for our bare static-library slices; plain `module` is
what Xcode needs there).

**Hard constraint (Miguel):** all compilation happens **on this Mac** — no
GitHub Actions, no CI involvement, no cross-repo deploy coupling. `release.yml`
is untouched; the XCFramework is a locally built artifact, never a published
release asset.

---

## 1. Architecture overview

```
┌────────────────────────────  this repo  ────────────────────────────┐
│  engine/ (vle-thermo)   steam/ (vle-steam, planned)   units/        │
│           ▲ rlib                  ▲ rlib                            │
│           └───────┬───────────────┘                                 │
│                ffi/ (vle-ffi, NEW — publish = false)                │
│   #[uniffi::export] wrapper API · crate-type = ["staticlib","lib"]  │
│                        │ cargo build --target …                     │
│      libvle_ffi.a (device) · libvle_ffi.a (simulator) · (macOS)     │
│                        │ uniffi-bindgen-swift                       │
│      VleFFI.swift + vle_ffiFFI.h + module.modulemap                 │
│                        │ xcodebuild -create-xcframework             │
│                 VleFFI.xcframework                                  │
│                        │                                            │
│   swift/VleThermo/  — local Swift Package (binaryTarget + wrapper)  │
└─────────────────────────────────────────────────────────────────────┘
                         │  "Add Local Package…" in Xcode
              iOS app project (separate repo, e.g. vle-ios)
```

Three deliverables in this repo:

1. **`ffi/` crate (`vle-ffi`)** — the UniFFI-annotated wrapper.
2. **`scripts/build-ios.sh`** — one command from clean checkout to
   `VleFFI.xcframework` + generated Swift sources.
3. **`swift/VleThermo/`** — a local Swift package (Package.swift +
   `binaryTarget` + the generated wrapper source + XCTests) so consuming from
   Xcode is drag-free and the bindings are testable *on this Mac* without
   opening a simulator.

The actual iOS **app** (Xcode project, UI) lives in a separate repo later —
this repo owns everything up to and including the Swift package boundary.

## 2. Why UniFFI, and which mode

**UniFFI** (Mozilla, <https://github.com/mozilla/uniffi-rs>) generates the
C-ABI scaffolding *and* idiomatic Swift wrappers — memory management (Rust
objects surface as Swift classes backed by `Arc`), error mapping (Rust
`Result`/`thiserror` → Swift `throws`), and type conversion — the parts that
are tedious and unsafe to hand-write with raw `extern "C"`. Used in
production by Firefox and 1Password. Alternatives rejected:

- *Hand-rolled `extern "C"` + cbindgen*: maximal learning of raw FFI but ~10×
  the unsafe surface; UniFFI's generated code is itself readable and the
  learning doc (§7) walks through it.
- *swift-bridge*: nice but a smaller ecosystem; UniFFI is the de-facto
  standard and multi-language (Kotlin later, if Android ever calls).
- *cargo-swift*: a convenient wrapper around this exact pipeline; not used so
  that the build steps stay explicit and teachable, but noted in docs as the
  "shortcut" alternative.

**Mode: proc-macros, not UDL.** Modern UniFFI (0.29+) lets us annotate Rust
directly — `#[derive(uniffi::Record)]`, `#[derive(uniffi::Enum)]`,
`#[uniffi::export]`, `uniffi::setup_scaffolding!()` — no separate `.udl`
interface file to keep in sync. Bindings are generated with the
**`uniffi-bindgen-swift`** binary in *library mode* (reads the compiled
`.a`/dylib, emits `.swift` + header + modulemap).

## 3. The `ffi/` wrapper crate

A separate crate rather than annotating `engine/` directly, because:

- `vle-thermo`'s idiomatic API (generics, `SmallVec`, data-carrying enums,
  borrowed slices) is not FFI-shaped; UniFFI wants owned `Vec`s, plain
  records, and fieldless-or-simple enums. The wrapper defines flat DTOs
  (`FlashResult`, `SteamStateDto`, …) and converts.
- Keeps `uniffi` out of the published crates' dependency closure entirely
  (same isolation philosophy as the `python` feature).
- `publish = false` — this crate never goes to crates.io.

```toml
# ffi/Cargo.toml (sketch)
[package]
name = "vle-ffi"
publish = false
version.workspace = true
# …

[lib]
crate-type = ["staticlib", "lib"]   # staticlib → .a for Apple; lib → unit tests

[dependencies]
vle-thermo = { path = "../engine", features = ["component-db", "steam"] }
uniffi = "0.29"

[dev-dependencies]
uniffi = { version = "0.29", features = ["bindgen-tests"] }
```

**Critical feature rule:** the iOS build must compile `vle-thermo` **without
the `python` feature** — pyo3 must never enter the Apple dependency graph.
(Default features are already empty, so this holds as long as `ffi/` asks
only for `component-db` + `steam`.)

**Panics/unwinding:** the workspace deliberately doesn't set
`panic = "abort"` (PyO3 needs unwinding). That is also correct for UniFFI —
its scaffolding catches Rust panics at the FFI boundary and converts them to
Swift errors instead of tearing down the app. No profile changes needed.

### v1 exported API surface (pragmatic subset, grows later)

- `version() -> String` — smoke test.
- **Component DB**: `db_available() -> Vec<String>`,
  `db_component(name) -> ComponentDto` (Tc, Pc, ω, M, Cp coefficients…).
- **Steam tables** (once M13 lands): the full `Water` constructor set
  (`tp/tx/px/ph/ps/sat_t/sat_p`) returning a `SteamState` record — this alone
  makes a complete, genuinely useful iPhone steam-table app, which is why
  M13 (steam) is sequenced **before** M14.
- **Mixture VLE**: a `System`-like object (UniFFI *interface* → Swift class):
  set components + EOS/activity/mixing selections (as UniFFI enums mirroring
  the existing Rust enums), then `bubble_t/bubble_p/dew_t/dew_p/flash_tp`
  returning result records.
- Errors: one `VleFfiError` enum (`#[derive(uniffi::Error)]`) wrapping the
  engine's `thiserror` types → Swift `throws`.

Per the units rule, every exported function documents units; the FFI layer
speaks **repo-canonical units only** (K, kPa absolute, kJ/kg for steam,
kJ/kmol for mixtures). Unit-string parsing stays on the Swift side (or a
later `vle-units` export) — keep v1 numeric and explicit.

## 4. Build pipeline (`scripts/build-ios.sh`)

One idempotent script, runnable from a clean checkout, mirroring principle
"the operator path always works without Actions" — here there *is* no Actions
path at all, by design.

```bash
# 0. One-time prerequisites (script checks, doesn't auto-install):
rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin
#    Xcode 26 + command-line tools already present on this Mac.

# 1. Build static libs (release, per target)
cargo build -p vle-ffi --release --target aarch64-apple-ios        # device
cargo build -p vle-ffi --release --target aarch64-apple-ios-sim    # simulator (Apple Silicon)
cargo build -p vle-ffi --release --target aarch64-apple-darwin     # macOS slice → enables `swift test` locally
# IPHONEOS_DEPLOYMENT_TARGET pinned in the script (propose iOS 16.0).

# 2. Generate Swift bindings (library mode, from the built artifact)
cargo run -p uniffi-bindgen-swift -- \
  target/aarch64-apple-ios/release/libvle_ffi.a build/bindings \
  --swift-sources --headers --modulemap --module-name VleFFI
#    (a 3-line bin crate `ffi/uniffi-bindgen/` — the standard UniFFI pattern)

# 3. GOTCHA: the modulemap MUST be named exactly `module.modulemap` inside
#    each Headers dir, or Xcode won't find the module. The script renames it.

# 4. Assemble the XCFramework (one -library/-headers pair per slice)
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libvle_ffi.a      -headers build/headers \
  -library target/aarch64-apple-ios-sim/release/libvle_ffi.a  -headers build/headers \
  -library target/aarch64-apple-darwin/release/libvle_ffi.a   -headers build/headers \
  -output swift/VleThermo/VleFFI.xcframework

# 5. Copy generated .swift wrapper into swift/VleThermo/Sources/VleThermo/
# 6. (cd swift/VleThermo && swift test)   # XCTest against the macOS slice
```

Notes:

- **Intel simulator (`x86_64-apple-ios`) deliberately omitted** — this is an
  Apple Silicon Mac and the app targets modern devices. If ever needed:
  build the extra target and `lipo -create` it with the arm64-sim `.a`
  *before* `-create-xcframework` (an XCFramework holds one library per
  platform+variant; multi-arch within a variant is lipo's job). Documented,
  not built.
- **macOS slice is included** purely so `swift test` runs natively on this
  Mac — the fastest verification loop, no simulator boot.
- The workspace `[profile.release]` (fat LTO, 1 CGU) already gives the
  smallest/fastest static lib; add `strip = "debuginfo"` consideration in
  14.1 if `.a` size is noticeable.
- Toolchain note: local rustc is 1.91.x; MSRV 1.85 unaffected. The pyo3
  `E0133` clippy quirk is irrelevant here (no `python` feature in the graph).

### Git hygiene

All build products are **generated, never committed**: `.gitignore` gains
`build/`, `*.xcframework`, and `swift/VleThermo/Sources/VleThermo/*.generated.swift`
(regenerate via the script). The Swift package's *hand-written* files
(Package.swift, tests, any ergonomic Swift sugar over the generated API)
**are** committed. Pre-push private-data gate applies unchanged — nothing in
this pipeline touches infrastructure secrets.

## 5. Swift package + verification

`swift/VleThermo/Package.swift` (sketch):

```swift
// binaryTarget wraps the XCFramework; the "VleThermo" target adds the
// generated Swift wrapper + any hand-written ergonomic layer on top.
targets: [
  .binaryTarget(name: "VleFFI", path: "VleFFI.xcframework"),
  .target(name: "VleThermo", dependencies: ["VleFFI"]),
  .testTarget(name: "VleThermoTests", dependencies: ["VleThermo"]),
]
```

Verification ladder (all local):

1. `cargo test -p vle-ffi` — wrapper-level Rust tests.
2. `swift test` — XCTests through the *actual FFI boundary* on the macOS
   slice: `version()`, a component-DB lookup, a steam point asserted against
   an IAPWS verification value, one bubble-point vs. a Chapter IV number.
3. Xcode: new throwaway iOS app → *Add Local Package…* →
   `swift/VleThermo` → call `Water.px(p: 101.325, x: 1.0)` in a SwiftUI view
   → run in simulator. (Manual, documented step — the app itself is out of
   scope for this repo.)

## 6. Phase breakdown & estimates (becomes Milestone 14 on adoption)

| Phase | Content | Est. |
|---|---|---|
| 14.1 | `ffi/` crate + `uniffi-bindgen-swift` bin + `version()` end-to-end: script builds all 3 slices → XCFramework → Swift package → `swift test` green | 4–6 h |
| 14.2 | Real API surface: component DB, steam-table constructors, mixture `System` with enums/records/errors; Rust + Swift tests per binding (FFI analog of the M5+ PyO3 rule) | 6–10 h |
| 14.3 | Learning documentation `docs/en/ios/README.md` (§7) + README/deploy-doc touches + `.gitignore` + demo snippet for the future app repo | 3–5 h |

**Total: ~13–21 h.** Prereq: none strictly, but **M13 (steam) first** makes
14.2 far more compelling.

## 7. Learning documentation (this repo teaches, so the plan does too)

`docs/en/ios/README.md` will explain, for a reader who knows Rust or Swift
but not FFI:

- What a C ABI is and why Rust↔Swift interop funnels through it.
- Static vs. dynamic linking on iOS; what a **`.a`** is; why Apple requires
  per-target builds and what an **XCFramework** actually contains (walk the
  built directory tree: Info.plist, per-platform slices, Headers).
- What UniFFI generates and how to read it: the `#[uniffi::export]`
  scaffolding, the generated header, the Swift wrapper's lift/lower calls,
  how `Arc<T>` becomes a Swift class, how `Result<_, E>` becomes `throws`.
- Why simulator-arm64 and device-arm64 are *different targets* despite the
  same CPU (different platform ABI/availability), and where lipo fits.
- The `module.modulemap` naming gotcha and how Swift finds C headers.
- How to re-run everything: `scripts/build-ios.sh` is the single entry point.

## 8. Open decisions (recommendations made, Miguel to confirm)

1. **Sequencing: M13 (steam) → M14 (iOS)** — recommended; the steam-table
   app is the ideal first FFI consumer.
2. **iOS deployment target**: propose **iOS 16.0** (adjust freely).
3. **Scope of v1 FFI surface** (§3): DB + steam + basic mixture VLE;
   kij regression / envelopes / batch APIs deferred.
4. **App repo**: separate `vle-ios` repo later, consuming
   `swift/VleThermo` as a local package (or via a git submodule/path — decide
   when the app starts).
5. Milestone numbering (13/14) assumed; renumber on adoption if anything
   else lands first, per the Phase/Milestone sync rules.
