# vle in the Browser — JavaScript/TypeScript via WebAssembly

The same Rust engine that powers the Python package and the Swift/Kotlin
apps also compiles to **WebAssembly (wasm)** — a portable binary format
every modern browser, Node.js, and webview executes at near-native speed.
One local build produces `wasm/pkg/`, a ready-to-install npm package, and
the thermodynamics then runs **client-side**: a React website needs no
compute server, and the desktop/mobile shells (Tauri, Electron, Capacitor)
need no backend code at all.

This is the third sibling of the [iOS/macOS guide](../ios/README.md) (M15)
and the [Android/Kotlin guide](../android/README.md) (M16). Design record:
[WEB_UI_PLAN.md](../../../WEB_UI_PLAN.md).

## How it works (the 30-second version)

- The wrapper crate `wasm/` (`vle-wasm`) depends on the engine **without
  the `python` feature** and marks its API with `#[wasm_bindgen]`
  attributes — the wasm analog of `ffi/`'s UniFFI annotations.
- **wasm-bindgen** is the standard Rust↔JS binding generator: the compiler
  emits wasm exports plus metadata; the generator reads that metadata and
  writes the matching **JS glue and TypeScript declarations**.
- **wasm-pack** drives the whole pipeline (cargo build → bindgen →
  `wasm-opt` size pass) and lays the output out as an npm package:
  `wasm/pkg/{vle_wasm_bg.wasm, vle_wasm.js, vle_wasm.d.ts, package.json}`.
- At runtime your app calls `await init()` once (the browser compiles the
  wasm module), then every call is an ordinary typed function call.
  Records cross the boundary as **plain JS objects** (camelCase keys),
  compositions as **`Float64Array`s** (a plain `number[]` also works), and
  Rust `Err`s are **thrown as JS `Error`s**.

Everything is generated, nothing is committed or published — the repo
distributes the recipe, not the artifact, exactly like the Swift and
Kotlin channels.

## Prerequisites

- Rust toolchain (`rustup`), plus the wasm target:
  `rustup target add wasm32-unknown-unknown`
- **wasm-pack**: `cargo install wasm-pack` (or `brew install wasm-pack`)
- Node.js — for the boundary smoke tests and for any React tooling
  (the build itself works without it: `scripts/build-wasm.sh --skip-tests`)

## Build

```sh
git clone https://github.com/miguelju/vle.git && cd vle
scripts/build-wasm.sh        # → wasm/pkg/  (~360 KB wasm, ~150 KB gzipped)
```

The script runs the smoke tests in Node first, then builds the release
package. `--target web` emits a universal ES module that works both
through a bundler (Vite, webpack) and from a plain
`<script type="module">`.

## Consume from React (or any JS project)

```sh
npm install /path/to/vle/wasm/pkg     # local path — nothing on npm
```

Initialize once, then call the engine directly. **Units are canonical
engine units everywhere: temperature K, pressure kPa absolute, mole
fractions summing to 1** (steam quantities are mass-basis: kJ/kg, m³/kg).
Unit conversion belongs in your UI layer.

```js
import init, { version, VleSystem, steamSatP, dbAvailable } from "vle-wasm";

await init();                          // compile + instantiate the wasm once
console.log(version());                // "0.11.0"

// The Chapter IV Table 4.10 flash — n-heptane/n-butane, RKS, 300 K, 100 kPa.
const sys = VleSystem.fromDb(["n-heptane", "n-butane"], "RKS1972", "RKS1972");
const r = sys.flashTp(300.0, 100.0, [0.5, 0.5]);
console.log(r.beta, r.x, r.y, r.twoPhase);   // 0.1967, [0.612, 0.388], …, true

// γ-φ: van Laar liquid, explicit object form + activity parameters.
const mw = VleSystem.fromDb(
  ["methanol", "water"],
  "idealGas",
  { kind: "activity", model: "vanLaar" },
  "classical",
  { aij: [[0, 0.8041], [0.5619, 0]] },
);
const bub = mw.bubbleP(298.15, [0.4, 0.6]);  // kPa absolute in bub.value

// Steam tables (IAPWS-IF97) — the classic saturation-table row at 1 atm.
const row = steamSatP(101.325);
console.log(row.t, row.hFg);                 // 373.12 K, 2256.5 kJ/kg

console.log(dbAvailable());                  // the 25 bundled components
```

Model names are case-insensitive and separator-blind (`"van-laar"` ==
`"vanLaar"`); an EOS name as the vapor/liquid string means a cubic phase.
Custom components are plain object literals — only `name`, `tc` (K), `pc`
(kPa), and `omega` are required:

```js
const sys = new VleSystem(
  [{ name: "custom", tc: 500.0, pc: 3000.0, omega: 0.25,
     psatCoeffs: [4.1, 2500.0, -40.0] }],   // reduced-Antoine fit
  "idealGas", "ideal");
```

Errors throw JS `Error`s whose message prefix is the category — the same
split as the Swift/Kotlin/Python surfaces:

```js
try { sys.flashTp(300, 100, [1.0]); }        // wrong length for 2 components
catch (e) { e.message; }                     // "invalid input: z has 1 entries …"
// Other prefixes: "component not found …", "flash calculation failed: …",
// "steam tables error: …"
```

With Vite the package works out of the box; `vite-plugin-wasm` is only
needed for bundler-target builds, not this `--target web` module.

## Keep the UI responsive: the Web Worker pattern

Single calls are microseconds and surface grids are milliseconds (a 50×50
P–x–y sheet is ~14 ms), so most UIs can call the engine directly in an
event handler. For *bulk* work — dense P–v–T grids, long parameter sweeps
— move the engine into a **Web Worker** so the main thread never blocks.
This is the browser analog of the Python batch API releasing the GIL:

```js
// worker.js — the engine lives here, off the main thread.
import init, { VleSystem } from "vle-wasm";
const ready = init();
let sys;
onmessage = async ({ data }) => {
  await ready;
  if (data.kind === "system") {
    sys = VleSystem.fromDb(data.names, data.vapor, data.liquid);
  } else {
    // One message per sweep (not per point): ts is a Float64Array, and the
    // reply transfers its buffers instead of copying them.
    const betas = new Float64Array(data.ts.length);
    data.ts.forEach((t, i) => { betas[i] = sys.flashTp(t, data.p, data.z).beta; });
    postMessage({ betas }, [betas.buffer]);
  }
};
```

Parallel throughput (the rayon half of the Python trick) is deliberately
not built: wasm threads need a nightly-rebuilt std and cross-origin
isolation headers on every response. WEB_UI_PLAN.md hard constraint 5
records the full decomposition and the `wasm-bindgen-rayon` upgrade path
if an in-browser bulk workload ever appears.

## 3-D phase surfaces (plotly.js)

The browser is the best 3-D plotting platform this project targets:
[plotly.js surface plots](https://plotly.com/javascript/3d-surface-plots/)
are WebGL-backed and interactive by default. Compute the grid with the
engine, hand plotly the arrays:

```js
import Plot from "react-plotly.js";

// P–x–y "sail": bubble/dew pressure vs composition at a sweep of T.
const x = [...Array(41)].map((_, i) => i / 40);
const surface = temps.map(t =>
  x.map(x1 => sys.bubbleP(t, [x1, 1 - x1]).value));
<Plot data={[{ type: "surface", x, y: temps, z: surface }]} />
```

(The README hero images are exactly this kind of grid, computed by the
same engine — see `notebooks/09_3d_phase_surfaces.ipynb` for the physics.)

## Desktop and mobile shells

Because the engine runs *inside* the webview as wasm, every shell below
wraps the **same React bundle** with no backend code — the shell choice is
packaging, not architecture, and it lives in the app repo, not here:

- **Tauri 2** — Windows/macOS/Linux **and** Android/iOS from one config;
  ~10 MB installers (system webview). The Rust backend needs no commands —
  the wasm module already ships in the frontend assets.
- **Electron** — desktop-only, bundles Chromium (~100 MB) but is the most
  battle-tested shell. Optional upgrade path: a `napi-rs` native module
  linking the engine crate directly gives full native speed + rayon
  threads if a desktop workload ever outgrows wasm.
- **Capacitor** — the long-standing web→Android/iOS wrapper, if only
  mobile is needed.
- **PWA** — no wrapper at all: the website is installable from the
  browser once it ships a manifest + service worker.

## Testing

Three rungs, all local (`scripts/build-wasm.sh` runs the first two):

```sh
cargo test -p vle-wasm        # host-side: logic, parsing, validation (19 tests)
wasm-pack test --node wasm    # the real JS↔wasm boundary in Node (5 smoke tests)
```

The smoke tests are the wasm analog of the Kotlin JNA tests: version
string, DB lookup, IF97 boiling point, the Chapter IV Table 4.10 flash,
and error mapping. For a browser sanity check, serve a scratch page that
imports `wasm/pkg` and compare the same numbers in the console.

## Troubleshooting

- **`unknown cubic EOS "…"` / `unknown activity model "…"`** — model-name
  typo; the error lists examples. Names are case- and separator-blind.
- **`TypeError: Failed to fetch` on `init()`** — the `.wasm` file isn't
  being served (404) or the server sends the wrong MIME type;
  `application/wasm` enables streaming compilation. Any static host and
  the Vite dev server do this correctly out of the box.
- **Numbers look wrong by a constant factor** — check units: the boundary
  is K and kPa **absolute** (not °C, not bar, not gauge). Convert in the
  UI layer.
- **A call throws with `"flash calculation failed"`** — same numerical
  meaning as in Python/Swift/Kotlin: the state may be single-phase (check
  `twoPhase` on a flash that *did* return), near-critical, or the model
  selection unsupported for that path.
- **Rebuilt the Rust but the app still runs old code** — rerun
  `scripts/build-wasm.sh`, then restart the dev server so the bundler
  picks up the new `wasm/pkg` (with `npm install <path>` npm links the
  directory, so a rebuild is usually enough).

## Why is none of this in CI?

Same policy as the Swift and Kotlin channels (see
[deploy/README.md](../../../deploy/README.md)): the wasm package is a
machine-built binary, so the repo ships source + a build script, publishes
nothing to npm, and keeps `release.yml` untouched. `cargo test -p
vle-wasm` runs in the ordinary workspace test suite; the boundary tests
need Node and stay local, one `scripts/build-wasm.sh` away.
