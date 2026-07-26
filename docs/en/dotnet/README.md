# vle from C# / .NET — status and route (via `uniffi-bindgen-cs`)

> **Status: parked — version-blocked as of 2026-07-12.** Everything in this
> document was verified on that date; re-check the
> [uniffi-bindgen-cs releases](https://github.com/NordSecurity/uniffi-bindgen-cs/releases)
> before acting on it.

This repo's FFI layer (`ffi/`, UniFFI) can in principle serve **C#/.NET**
the same way it serves Swift (M15) and Kotlin (M16): a generated wrapper
over the compiled `vle_ffi` library, consumable from any Windows UI stack
(WinUI 3, WPF, MAUI, Avalonia) in Visual Studio. This document records why
that path is **not currently buildable**, what would unblock it, and the
exact route for when it unblocks.

## Why it's blocked (as of 2026-07-12)

- C# is **not** one of uniffi-rs's first-party binding languages (those are
  Kotlin, Swift, Python, Ruby). C# support comes from a third-party
  generator: [`uniffi-bindgen-cs`](https://github.com/NordSecurity/uniffi-bindgen-cs)
  by NordSecurity.
- UniFFI requires the **generator and the scaffolding compiled into the
  library to target the same uniffi-rs version** — that's why this
  workspace's own bindgen binaries live in `ffi/uniffi-bindgen/`, resolved
  by the same `Cargo.lock`.
- As of 2026-07-12, the latest `uniffi-bindgen-cs` release is
  **`v0.11.0+v0.31.0`**, i.e. it targets **uniffi-rs 0.31**. This repo pins
  **`uniffi = "0.32"`** (`ffi/Cargo.toml`), and **there are no plans to
  downgrade** — the Swift and Kotlin paths are on 0.32 and downgrading a
  working two-platform setup to chase a third is the wrong trade.

## Options (in order of preference)

1. **Wait for `uniffi-bindgen-cs` to target 0.32.** NordSecurity's
   `vX.Y.Z+vA.B.C` tag convention tracks uniffi-rs releases deliberately;
   they have historically caught up within months. Watch the releases page.
2. **Build your own generator in Rust supporting the current uniffi.**
   `uniffi-bindgen-cs` is open source (its generator walks uniffi's
   component-interface model and emits C#) — porting it forward to 0.32, or
   maintaining a fork pinned to whatever this workspace uses, is a real but
   nontrivial Rust project. Only worth it if (1) stalls and the .NET
   consumer becomes important.
3. ~~Downgrade the workspace to uniffi 0.31~~ — rejected (see above).

Note the contrast that drove the Milestone 16 decision: the **Kotlin**
bindgen ships *inside* the `uniffi` crate itself, so it is always at
exactly the workspace's version by construction. The version-skew problem
this page documents structurally cannot happen on the Kotlin path — one of
the reasons the Windows desktop app goes through **Compose Multiplatform**
(see [ANDROID_FFI_PLAN.md](../../plans/delivery/ANDROID_FFI_PLAN.md)) instead of .NET.

## The route, for when it unblocks

The groundwork already exists — `ffi/` builds a `cdylib` (added for
Kotlin/M16), which is exactly what .NET P/Invoke loads:

1. **Build the native library** on Windows:
   `cargo build -p vle-ffi --release` → `target\release\vle_ffi.dll`
   (add `--target aarch64-pc-windows-msvc` for Windows-on-ARM).
2. **Install the matching generator** (tag must match the workspace's
   uniffi version — `+v0.32.x`, once it exists):

   ```sh
   cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v<X.Y.Z>+v0.32.<z>
   ```

3. **Generate the C# wrapper** (library mode, same as Swift/Kotlin):

   ```sh
   uniffi-bindgen-cs --library target/release/vle_ffi.dll --out-dir dotnet/generated
   ```

4. **Package as a NuGet** class library: the generated `vle_ffi.cs` plus
   the DLL under `runtimes/win-x64/native/` (and `win-arm64` if built).
   Any Visual Studio UI project — WinUI 3, WPF, MAUI, Avalonia — references
   it like a normal package; the FFI layer doesn't care which UI sits on top.
5. Add a `[bindings.csharp]`-appropriate config if needed (namespace etc.)
   per the uniffi-bindgen-cs docs of that release.

Same repo rules as Swift/Kotlin would apply: local builds only, generated
code and DLLs gitignored, no CI, no committed binaries.
