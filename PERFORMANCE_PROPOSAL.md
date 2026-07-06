# Performance & Algorithm Proposal — "numpy for thermo"

*Adopted 2026-07-01. Status: accepted; folded into [MODERNIZATION_PLAN.md](MODERNIZATION_PLAN.md)
(Algorithm Performance Improvements §A–§M + the Performance Engineering section),
[ROADMAP.md](ROADMAP.md), and [TODO.md](TODO.md). This document is the rationale record;
the planning docs are the executable plan.*

*Implementation status (2026-07-03): **Track C** (engine mechanics) and **Track E**
(measure-first: criterion benches + FFI boundary benchmark + informational CI bench
job) shipped with Milestone 8.2. **Track B** (generalized (A, B, U, W) mixture core
with exact analytic/`num-dual` composition derivatives) shipped with Milestone 8.3
(`engine/src/mixture.rs`). **Track A** algorithm choices land with the Milestone 9
flash work; **Track D** (batch numpy API) with Milestone 10.*

## Goal

Make `vle-thermo` a **very fast, always-convergent** thermodynamics library driven from
Python the way numpy is: array-in / array-out, with the heavy work in native code.
Calculation speed and guaranteed convergence are the top-level goals; there are no
external API-stability constraints on the Rust side.

## TL;DR

**Stay in Rust.** An audit of the engine (2026-07-01, v0.7.0) found nothing a language
change would fix — the current costs are architectural (allocations in hot paths,
redundant recomputation, a scalar-only Python boundary, default build flags), and the
biggest algorithmic wins are in code that doesn't exist yet (M8 mixing rules, M9 flash).
The plan has five tracks:

| Track | Content | Lands in |
|-------|---------|----------|
| **A** | Modern flash algorithms (stability test, GDEM→Newton, windowed Halley RR, envelope continuation) | Milestone 9 (Phase 15) |
| **B** | Generalized mixture core with exact derivatives (analytic + dual-number AD) | Milestone 8.3 (Phases 12–13) |
| **C** | Engine mechanics: zero-allocation hot path, `EosState` caching, build profile | Milestone 8.2 (Phase 11) |
| **D** | Batch numpy API, GIL release, rayon parallelism, `System` handle | Milestone 10 (Phase 17) |
| **E** | criterion + Python-boundary benchmarks (before everything else) | Milestone 8.2 (Phase 11) |

Sequencing rule: **E and C first** (measure, then free wins), **B before any flash code**
(the derivative architecture must exist before Newton loops consume it), **A on top of B**,
**D last** (the ergonomics layer over a now-fast core).

---

## 1. The language question

Rust compiles through LLVM to the same machine code as C++ or Fortran for scalar f64
math. The engine already uses static dispatch everywhere (enum + `match`, monomorphized
closures; zero `dyn` in inner loops), so there is no interpreter, vtable, or GC overhead
to escape by switching languages.

- **C++/Fortran**: identical codegen, worse tooling, no safety net, and ~6,000 lines of
  already-validated code would need re-porting. Net negative.
- **Julia**: excellent for research, but shipping it inside a Python wheel means bundling
  a JIT runtime with multi-second warmup — the opposite of a snappy numpy-like import.
- **GPU**: cubic-EOS flash is branchy, small-n (2–10 components), variable-iteration
  work. GPUs only pay off at millions of *independent* state points; a rayon-parallel
  CPU batch API covers realistic workloads. The batch API design keeps the door open.

Where the speed actually lives, in descending order of impact:
**algorithm choice** (10–100× on convergence-limited problems) → **API shape**
(amortizing Python overhead, 100–1000× for grid workloads) → **memory behavior**
(allocations, caching, ~2–5×) → **build flags** (~10–30%) → language (~0×).

## 2. Audit findings (what costs time today, v0.7.0)

- `cubic::solve_real` heap-allocates a `Vec` per call and `z_factor` allocates a second
  one to filter/sort roots (`engine/src/numerics/cubic.rs:107`, `engine/src/eos.rs:824`).
  This will be the innermost call of every flash iteration.
- α(T) is computed up to **three times per property call**: `ln_phi_pure` calls
  `ab_dimensionless` (computes α) *and* `z_factor` (computes it again);
  `h_departure_rt` a third time. No caching of any T-dependent quantity.
- Wilson recomputes the Λ matrix (with its `exp` calls) on every `ln_gamma` call —
  O(n³) for the full γ vector; the virial code rebuilds the B matrix three times per
  mixture fugacity call.
- The Python boundary is **scalar-only**: every call crosses FFI with primitive floats,
  rebuilds a `Component` (allocating a `String` + `Vec`) per call, and never releases
  the GIL. No numpy integration exists.
- Broyden clones and re-factorizes the full Jacobian (O(n³)) every iteration
  (`engine/src/numerics/broyden.rs:161`).
- **No `[profile.release]` anywhere**: wheels ship with `lto = false`,
  `codegen-units = 16`. `ndarray` is a declared-but-unused dependency.
- **No benchmarks exist** — no criterion, no timing tests, no baseline.

## 3. Track A — Modern flash algorithms (Milestone 9)

The thesis's scheme (successive substitution with trivial-solution guards, two-stage
Newton-Raphson fallback with *numerical* Jacobians) predates a generation of flash
methodology, mostly Michelsen's. Since M9 isn't built yet, adopting it costs nothing.
Details and citations: MODERNIZATION_PLAN.md §F (updated), §I, §J, §K, §M.

- **A1 — Rachford-Rice**: Halley's method *inside the Leibovici–Neoschil window*
  β ∈ (1/(1−K_max), 1/(1−K_min)) with a bisection safeguard. The RR function is
  monotonic and smooth inside the window, so convergence is **guaranteed** (typically
  2–4 iterations), and negative flash falls out for free. (§F)
- **A2 — Isothermal flash**: Wilson-correlation K init → tangent-plane-distance
  **stability analysis** (the structural fix for the thesis's flagged trivial-solution
  problem — guards become unnecessary rather than better) → successive substitution
  with **GDEM acceleration** → switch to full **Newton on ln Kᵢ** with the Track-B
  analytic Jacobian when the residual drops below ~10⁻³. (§I, §J)
- **A3 — Bubble/dew**: full Newton on {ln K₁..ln Kₙ, ln T or ln P} — log variables keep
  iterates positive and the Jacobian well-scaled. Near-critical traversal via
  **Michelsen's phase-envelope continuation** (predictor-corrector, adaptive step,
  walks *through* the critical point) instead of the thesis's dP/dT differential
  stepping. The thesis two-stage scheme is kept behind a test as an oracle. (§K)
- **A4 — Adiabatic (PH) flash**: warm-started nested loop first (inner flash seeded
  with the previous T's K-values → 1–3 inner iterations); simultaneous Newton on
  (T, ln K) à la Michelsen's state-function-based flash if benchmarks justify it. (§M)
- **A5 — Critical point**: Heidemann–Khalil with analytic Helmholtz derivatives —
  already planned (§G); Track B makes it nearly free.
- **A6 — Regression**: Brent for scalar kij (§B, unchanged); **Levenberg–Marquardt**
  instead of plain Newton for multi-parameter Aij regression (same per-iteration cost,
  graceful far-from-optimum behavior). The dominant cost is the per-data-point bubble-P
  objective — warm-starting each point from its neighbor matters more than the outer
  optimizer.

## 4. Track B — Generalized mixture core with exact derivatives (Milestone 8.3)

The single most important decision in this proposal: **make analytic/AD derivatives the
foundation of the mixture layer before writing any flash code.**

- **B1 — One (A, B, U, W) mixture code path.** The engine already unified 2-param and
  3-param EOS through the generalized cubic form (`eos.rs:955`). Extend that to
  mixtures: mixture Z, ln φ̂ᵢ, and departures are written **once**; each of the 22 EOS ×
  11 mixing rules only supplies (A, B, U, W) and their derivatives. This is the
  Michelsen–Mollerup architecture.
- **B2 — Analytic ∂ln φ̂ᵢ/∂nⱼ for cubic EOS + classical mixing.** Closed forms are
  standard. Newton flash, bubble/dew, and Heidemann–Khalil then cost **one** residual
  evaluation per iteration instead of n+1 (finite differences), with no FD noise
  degrading convergence — this fixes the exact weakness the thesis called out about
  itself (Ch. IV §4.1).
- **B3 — Dual-number AD as the generic fallback, not finite differences.** For exotic
  mixing rules (Wong-Sandler, MHV1/2), write the rule once as a function generic over
  the scalar type and evaluate with dual numbers (`num-dual` crate, built by the FeOS
  authors for exactly this). Derivatives **exact to machine precision** at ~2× one
  function evaluation — cheaper than FD's n+1 evaluations and immune to step-size
  tuning. Prior art worth reading during design: the FeOS crate (hyper-dual numbers
  over a reduced-Helmholtz core).

## 5. Track C — Engine mechanics (Milestone 8.2, pure wins, no behavior change)

- **C1** — Allocation-free cubic/Z path: `solve_real` returns `([f64; 3], usize)`;
  root selection by direct comparison, no filter/collect/sort.
- **C2** — A per-(T, P, composition) **`EosState`** struct caching αᵢ, dαᵢ/dT, aᵢ, bᵢ,
  a_mix, b_mix, A, B, U, W and derivatives — computed once, consumed by
  Z/fugacity/enthalpy/entropy. Natural carrier for the Wilson Λ and virial B matrices.
- **C3** — Stack-allocated composition arrays for n ≤ 8 (SmallVec/const generics).
- **C4** — Broyden: stop cloning + re-factorizing the Jacobian every iteration
  (Sherman–Morrison inverse update, O(n²)/iter). Note Broyden gets demoted to
  fallback-for-exotic-rules once Track B lands.
- **C5** — `[profile.release]`: `lto = "fat"`, `codegen-units = 1`. **Not**
  `panic = "abort"` (PyO3 needs unwinding to convert panics into Python exceptions).
  Drop the unused `ndarray` dependency. Keep wheels portable (no `target-cpu=native`
  in published artifacts; optional `x86-64-v3` builds later).

## 6. Track D — Batch numpy API (Milestone 10): what makes it "numpy for thermo"

- **D1** — Batch-first bindings via **rust-numpy** (the `numpy` crate): every property
  and flash accepts numpy arrays of state points and returns arrays — one FFI crossing,
  zero-copy views, results written into pre-allocated output arrays.
- **D2** — **Release the GIL** (`Python::allow_threads`) and parallelize batch kernels
  with **rayon**. State points are embarrassingly parallel. (What the GIL is, why a
  *numerical* library is the ideal case for releasing it, and which routines gain the
  most: see the **sidebar** below.)
- **D3** — A persistent **`System` `#[pyclass]` handle** holding components, model
  selections, and cached T-independent data — kills per-call `Component`
  reconstruction; scalar convenience methods become batch-of-one.
- **D4** — **Warm-start plumbing**: batch flash seeds each point from its neighbor's
  converged K — the natural access pattern of envelope tracing and regression, often
  3–5× fewer iterations across a grid.

### Sidebar (D2): "release the GIL", and why a numerical library is the ideal case

**What the GIL is.** CPython carries a **Global Interpreter Lock** — one process-wide
mutex that lets **only a single thread execute Python bytecode at a time**. It exists to
keep CPython's reference-counting memory management thread-safe. The practical cost: you
can start eight threads on eight cores and, for CPU-bound *Python* code, they still run
one at a time, taking turns holding the lock. Threads buy you concurrency for I/O waits,
not parallel arithmetic — which is why CPU-heavy Python usually reaches for
`multiprocessing` (separate processes, separate GILs) instead.

**What "releasing" it means.** The GIL only protects **Python objects and bytecode**.
Code running in a native extension that touches *no* Python objects — a Rust routine
crunching `f64`s in and out of numpy's raw buffers — does not need the lock. So the
extension can hand the GIL back for the duration of that native work and reacquire it
before returning. While it is released, other Python threads run, and — the part that
matters here — the native side is free to spin up **real OS threads across every core in
true parallel**, because none of that work goes through the interpreter. In PyO3 that is
`py.allow_threads(|| { … })`; the whole rayon-parallel kernel lives inside the closure,
and the lock is automatically retaken on the way out.

**Why the *numerical* nature of this library makes it the ideal case.** Releasing the
GIL only pays off to the extent that real time is spent in native code that ignores
Python. `vle-thermo` is almost nothing *but* that. A single flash point is Cardano cubic
root-solving, Wilson-correlation K initialization, TPD stability analysis, GDEM-
accelerated successive substitution, then Newton on lnK with an **exact** analytic /
`num-dual` Jacobian and an nalgebra linear solve (Tracks A–B) — hundreds of pure
floating-point EOS evaluations, zero Python objects in the loop. That is exactly the
work `allow_threads` is meant to cover: the arithmetic is the payload, not a thin wrapper
around it. A library that spent most of its time manipulating Python lists/dicts would
see almost no benefit; a solver whose runtime *is* the f64 iteration sees near-linear
core scaling. The batch layer is structured to preserve this — inputs are read as
zero-copy numpy slices, every point's result is collected into plain Rust structs, and
all Python-API calls (building the output arrays) happen **outside** the released region,
which is both a PyO3 requirement and the reason the parallel section stays pure math
(`engine/src/py_system.rs`, module doc + `PointOut`).

**Which routines gain the most** (all in the persistent `System` handle, `python/src/vle/system.py`
→ `engine/src/py_system.rs`; each has a `parallel=` switch, default on):

- **Tier 1 — iterative flash & saturation solvers (largest win).** `flash_pt_batch`
  (isothermal PT flash) and the four saturation batches — `bubble_pressure_batch`,
  `bubble_temperature_batch`, `dew_pressure_batch`, `dew_temperature_batch` (all through
  the shared `sat_batch`/`chunked_run` rayon kernel). Each point is a full iterative
  solve — tens to hundreds of EOS evaluations, Newton steps, and matrix factorizations —
  and points are independent, so the compute-per-point is high and the per-array FFI
  overhead is negligible. This is where GIL-release + rayon converts a Python `for`-loop
  into near-linear multi-core throughput. **kij/Aij regression** (`flash/kij_regression.rs`,
  `flash/aij_regression.rs`) sits here too: its dominant cost is a per-data-point bubble-P
  objective, an embarrassingly parallel fan-out (and it compounds with the D4 warm-start,
  cutting the iteration count on top of the parallelism). Grid workloads —
  **phase-envelope tracing** (`flash/envelope.rs`) and T–P property grids — are the
  canonical consumers.
- **Tier 2 — batch property evaluations (still worthwhile).** `z_factor_batch`,
  `ln_phi_batch`, `enthalpy_entropy_batch`. These are cheap per point (one `EosState`
  build + a departure evaluation, no iteration), so for these the **D1** win — one FFI
  crossing per array instead of per point, plus killing per-call `Component` rebuilds via
  the `System` handle (**D3**) — usually dominates the parallelism win. They still scale
  across cores on large arrays; the lock simply isn't the bottleneck the way it is for the
  iterative solvers.

**Nuance.** Python 3.13+ has an experimental **free-threaded ("no-GIL")** build, but the
standard CPython wheels `vle-thermo` ships still carry the GIL, so explicitly releasing it
around the native batch kernels remains the correct and necessary technique.

## 7. Track E — Measure first (Milestone 8.2)

criterion benches (α dispatch, Z-factor, pure ln φ, mixture ln φ̂, RR solve, full flash
as they land) + a Python-side boundary benchmark (scalar loop vs batch). CI job reports
deltas (informational). Later: comparison benchmarks against `thermo` (Bell) and
CoolProp — the external headline this library wants.

## 8. What we explicitly will NOT do

- **Rewrite in another language** — §1; every identified bottleneck is fixable in place.
- **GPU offload** — wrong workload shape at this scale; batch API keeps the option open.
- **Hand-written SIMD** — SoA batch kernels + LTO capture the auto-vectorization win;
  revisit only if benches show a specific kernel bound on it.
- **Trait-object model plugins** — enum + match dispatch is already the fast option.

## 9. Validation invariant

The Chapter IV cases (Tables 4.1–4.12) remain the correctness gate throughout: the
*algorithms* change, the thermodynamic answers must not (match within 1–5%). The thesis
two-stage bubble/dew scheme is retained as a test oracle for the modern path.
