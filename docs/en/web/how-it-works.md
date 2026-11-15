# How the WebAssembly Build Works

The [web guide](README.md) shows how to build `wasm/pkg/` and call it. This
page explains what is inside that package and what happens on each call:
how Rust becomes a `.wasm` file, how a JavaScript value reaches a Rust
function and comes back, what it costs, and what the format cannot do. The
worked example is *Critical*, a game being built for
[migueljackson.dev](https://migueljackson.dev) that runs this engine in the
visitor's browser.

All numbers on this page were measured on vle 0.20.1 (wasm-pack 0.15.1,
rustc 1.99.1, Node 26 on an Apple-silicon laptop). They are one machine's
numbers; treat them as orders of magnitude.

## 1. What WebAssembly is

WebAssembly (wasm) is a binary instruction format for a small virtual
machine. Every current browser, Node.js and every webview ships that
machine and compiles the instructions to native code when the module loads.
Four properties matter for a numerical engine:

- **Four value types.** A wasm function takes and returns only `i32`,
  `i64`, `f32` and `f64` (plus opaque references to host objects). There
  are no strings, arrays or structs at the boundary.
- **One linear memory.** The module owns a single resizable byte array.
  Rust's heap, stack and static data all live in it. JavaScript can read
  and write that array; the module cannot see anything outside it.
- **Imports and exports only.** A module can call nothing but the
  functions the host passes in at instantiation. It has no file system, no
  network, no clock and no DOM unless JavaScript hands one over.
- **IEEE 754 arithmetic.** `f64` add, multiply, divide and square root are
  specified bit for bit, so the engine's floating-point core behaves the
  same in every browser.

The engine is pure computation over `f64`, so it fits this model without
change. The same `engine/` and `steam/` source that builds the Python wheel
builds the wasm module; only the wrapper crate differs.

## 2. From Rust source to `wasm/pkg/`

`scripts/build-wasm.sh` runs `wasm-pack build wasm --target web --release`.
That is three steps:

1. **`cargo build --target wasm32-unknown-unknown --release`.** The crate
   `wasm/` (`vle-wasm`) is a `cdylib` that depends on `vle-thermo` with the
   `component-db` and `steam` features and **without** `python`, so pyo3,
   numpy and rayon never enter the graph. The workspace release profile
   (fat LTO, one codegen unit) applies. `unknown-unknown` means no
   operating system and no C library: the output is a bare module.
2. **wasm-bindgen.** Each `#[wasm_bindgen]` item makes the compiler emit a
   plain wasm export plus a description of its real signature in a custom
   section. The wasm-bindgen CLI reads those descriptions, removes them,
   and writes the JavaScript that converts arguments and results
   (`vle_wasm.js`) and the TypeScript declarations (`vle_wasm.d.ts`).
3. **`wasm-opt`.** A size and speed pass over the finished binary.

The result:

| File | Size | Contents |
|---|---|---|
| `vle_wasm_bg.wasm` | 762 KB (325 KB gzipped) | the engine |
| `vle_wasm.js` | 37 KB (9 KB gzipped) | loader and argument conversion |
| `vle_wasm.d.ts` | 13 KB | types for editors and `tsc` |

Inside the `.wasm`: 540 functions in 677 KB of code, 79 KB of static data
(the 25-compound database and the IF97 coefficient tables), 39 exports and
47 imports. The imports are all small JavaScript helpers that wasm-bindgen
generates — create an object or array, set a property, iterate an input
array, build an `Error`, and print a panic with `console.error`. None of
them touches the network or the DOM.

`--target web` produces an ES module that loads the `.wasm` by URL. It
works through a bundler and from a plain `<script type="module">`.

## 3. Loading the module

```js
import init, { VleSystem } from "vle-wasm";
await init();
```

`init()` fetches `vle_wasm_bg.wasm` relative to the JavaScript file and
calls `WebAssembly.instantiateStreaming`, which compiles while the bytes
download. That needs the server to send `Content-Type: application/wasm`;
otherwise the glue falls back to a slower non-streaming path and logs a
warning. Node has no fetch-by-relative-URL for this, so a script passes the
bytes itself:

```js
import { readFileSync } from "node:fs";
import { initSync, VleSystem } from "./pkg/vle_wasm.js";
initSync({ module: readFileSync("./pkg/vle_wasm_bg.wasm") });
```

Instantiation allocates the linear memory (18 pages of 64 KB, 1.2 MB),
copies the static data into it and runs the crate's
`#[wasm_bindgen(start)]` function, which installs a panic hook so a Rust
panic prints its message to the console. In Node, compiling the module took
about 1 ms and instantiating it 0.2 ms; a browser adds the download.

The module is instantiated once per JavaScript realm. A Web Worker is a
separate realm and needs its own `init()`.

## 4. What a call does

Take `sys.flashTp(230, 3000, [0.5, 0.5])`. The wasm export has this
signature:

```
vlesystem_flashTp(i32 self, f64 t, f64 p, i32 z_ptr, i32 z_len)
    -> (i32 result, i32 error, i32 is_error)
```

The generated JavaScript and the Rust shim do the rest:

1. **Arguments in.** `t` and `p` are `f64` and pass directly. For `z`, the
   glue calls the module's exported allocator for `2 × 8` bytes, then
   copies the numbers into linear memory through a `Float64Array` view. A
   plain `number[]` and a `Float64Array` cost the same (5.2 µs per call
   either way). Rust receives `&[f64]` and frees the buffer when the call
   returns.
2. **The solve.** `SystemCore::flash_tp` calls the same
   `flash::flash_isothermal` the Python and Swift bindings call. Nothing
   crosses the boundary while it runs.
3. **Result out.** The Rust result is a struct (`beta`, `x`, `y`, `k`,
   `iterations`, `twoPhase`). `serde-wasm-bindgen` turns it into a plain
   JavaScript object by calling the imported helpers — one call per field
   and per array element. The object lives on the JavaScript heap; wasm
   holds only an index into a table of references, which it returns.
4. **Errors.** A Rust `Err` becomes a JavaScript `Error` built the same
   way, and the glue throws it when `is_error` is set. The message prefix
   (`invalid input:`, `flash calculation failed:`, …) is the category.

Two return paths exist, and they cost differently. A function that returns
`Vec<f64>` directly, such as `kValues`, hands back a pointer and a length;
the glue copies the bytes out as one `Float64Array` in a single step. A
function that returns a record pays for one host call per field. That is
the right trade for an API meant to be read (`r.beta`, `r.x`), and it is
why a bulk export, if one is ever added, should return flat typed arrays.

`VleSystem` itself is a class whose instances hold one number: the address
of the Rust object in linear memory. Components, model selections and the
kij matrix are converted once, in the constructor, and stay on the Rust
side. Building a system from the database took 1.6 ms; every later call
reuses it.

## 5. Memory

Linear memory grows on demand and never shrinks. The engine's solvers keep
their work arrays on the stack or in reused buffers, so a session of flash
calls does not grow it: memory was 1 245 184 bytes after instantiation and
the same after about 600 000 flash and K-value calls.

A `VleSystem` is Rust-owned memory that the JavaScript garbage collector
cannot see into. Call `sys.free()` when a system is no longer needed. The
glue also registers each instance with a `FinalizationRegistry`, which
frees it some time after the wrapper object is collected, but that timing
is not guaranteed. Calling a method after `free()` throws
`null pointer passed to rust`.

Result objects (`{ beta, x, … }`) are ordinary JavaScript objects and are
garbage-collected normally.

## 6. What it costs

Per call, in Node, on the 50/50 methane/ethane Peng–Robinson system:

| Call | Time |
|---|---|
| `nComponents()` (boundary crossing only) | under 0.01 µs |
| `version()` (returns a string) | 0.08 µs |
| `kValues` (two arrays in, one typed array out) | 0.5 µs |
| `flashTp`, single phase | 1.4 µs |
| `flashTp`, two phases, 11 iterations | 5.2 µs |
| `criticalPoint` | 66 µs |
| `traceEnvelope`, 400-point cap | 0.48 ms |

Crossing the boundary is nearly free; converting a result record is what
shows up on the cheapest calls. Against the native build, a whole 96×64
flash map took 55 ms in wasm and 22 ms natively for methane/ethane, and
168 ms against 99 ms for a five-component mixture. The two harnesses are
not identical (the wasm one includes the record conversion and the game's
own per-cell code), so read that as "roughly two times slower than
native", not as a precise ratio.

The gap has not been profiled. The usual contributors are that `exp` and
`ln` are Rust's portable implementations compiled into the module instead
of the platform's tuned library, that wasm has a smaller instruction set
than the native target, and that the runtime must keep every memory access
inside the linear memory.

## 7. What it cannot do

- **No threads.** The module is single-threaded. The parallel batch API
  the Python package has is not in the wasm build. Threads in wasm need
  `SharedArrayBuffer`, which needs cross-origin isolation headers on every
  response and a specially rebuilt standard library; `WEB_UI_PLAN.md` hard
  constraint 5 records why that is not built. The alternative is a Web
  Worker, which moves the work off the main thread without making it
  faster.
- **No unit strings.** The boundary takes kelvin and kilopascals absolute.
  The wrapper exports nothing from the `units` crate; convert in the UI.
- **A subset of the engine.** Exported: isothermal flash, the four bubble
  and dew points, K-values, the critical point, the phase envelope, the
  component database and the steam tables. Not exported: the adiabatic
  flash, stability analysis as a separate call, regression, energy
  properties and the petroleum and refinery modules.
- **Last-digit agreement with native is not promised.** Arithmetic is
  bit-identical across browsers, but `exp`, `ln` and `pow` come from a
  different library than a native build uses, so a result can differ from
  the Python wheel's in the last digits. The boundary tests compare
  against table values with a tolerance, not against native output bit
  for bit.
- **A panic is fatal to the call.** Expected failures are `Err`s and
  arrive as thrown `Error`s. A panic is a bug: it prints to the console
  and traps, and the module should be considered unusable afterwards.

## 8. Worked example: *Critical*

*Critical* is a guessing game on a pressure–temperature chart. The player
is given a mixture and a blank chart, spends seven probes to reveal patches
of the phase map, and plants a flag where they think the critical point is.
The game then reveals the phase envelope and the true critical point and
scores the miss. Its five levels are Peng–Robinson mixtures from the
bundled database, from 50/50 methane/ethane to a five-component natural gas
with 75 % methane.

As of November 2026 the game is unreleased and its source lives in the
site's repository, so the details below can change. The site is static
files. It carries a copy of `wasm/pkg/` built from a
recorded commit of this repository, and every thermodynamic number the
player sees is computed in their browser by three exports:

| Game event | Engine call | Work |
|---|---|---|
| Level starts | `VleSystem.fromDb(names, "PR1976", "PR1976")` | once per mixture |
| Level starts | `traceEnvelope(z, 100, 400)` | the hidden dome, 0.6–1.7 ms by level |
| Level starts | `criticalPoint(z, seed)` | the answer, 0.07–0.2 ms per attempt |
| Probe | `flashTp(T, P, z)` per cell within 6.5 cells | up to about 130 flashes, a few ms at most |
| Reveal | `flashTp` for every remaining cell of the 96×64 map | 6 144 flashes in total, 48–168 ms by level |

Each `flashTp` result is reduced in JavaScript to one number per cell — the
vapor fraction for a two-phase cell, or a flag for one phase or a failed
solve — and stored in a `Float32Array` that the canvas draws from. The
envelope's `t` and `p` arrays are drawn as the dome's outline.

Three things in the game's design follow directly from how the wasm build
behaves:

- **No server, no precomputed tables.** A probe is about a hundred flashes
  and takes less than a frame, so the map is computed where the player
  clicks. The reveal orders the remaining cells nearest the flag first;
  the whole map is at most 168 ms of work.
- **One `VleSystem` per mixture.** Construction is the only
  millisecond-scale setup cost, and the object is Rust-owned memory, so
  the game's interface to the engine is a system built once per mixture
  plus `free()`.
- **The game logic has no DOM in it.** The module that decides which
  engine calls to make takes the system as an argument. The site's test
  script loads the same `.wasm` in Node with `initSync` and runs every
  level's map, envelope and critical point against it. That script is
  how the 0.20.0 phase-map slowdown was found and how the
  0.20.1 fix was checked
  ([OPTIMIZATION_PLAN_PART1.md](../../plans/engine/OPTIMIZATION_PLAN_PART1.md)
  §22).

The game also shows where the engine's answers need a caller's judgment. A
critical-point solve is a Newton iteration that can fail for some
compositions, so the game checks each candidate against the envelope —
traced by a different algorithm — and retries from other starting
temperatures. A flash that does not converge is drawn as its own cell
state instead of being hidden.

## 9. Where to go next

- Build, install and API examples: [README.md](README.md)
- Design decisions and constraints:
  [WEB_UI_PLAN.md](../../plans/delivery/WEB_UI_PLAN.md)
- The wrapper source: `wasm/src/` — `system.rs` holds the `VleSystem`
  shim and the JavaScript-free `SystemCore` it wraps
- The other local-build channels:
  [distribution/README.md](../../../distribution/README.md)
