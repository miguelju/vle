# Optimization Plan — Part 2: Global Package Optimization

**Status: the high-value subset is complete (2026-07-25).** Executed by Claude Code using Claude Opus 5 (1M context).

Plan of record for the engine's response to **Part 2** of
[`optimizations_audit.md`](optimizations_audit.md). Part 1 (the flash layer) is
[`OPTIMIZATION_PLAN_PART1.md`](OPTIMIZATION_PLAN_PART1.md); the provenance of the
audit itself is [`OPTIMIZATION_AUDIT_HISTORY.md`](OPTIMIZATION_AUDIT_HISTORY.md).

Part 1 established by measurement that ~70 % of an isothermal flash is the
K-value evaluation. Part 2 is where that 70 % lives. As in Part 1, every
recommendation was benchmarked rather than assumed — and again, the single most
valuable finding came from a measurement that contradicted the audit.

---

## 1. The measured baseline

`engine/benches/engine_bench.rs` gained a `mixture` group that opens up
`ln_phi_mix` — the function `k_values` calls twice — into its three layers, so
the split between composition-*dependent* and composition-*independent* work is
visible:

| n | `mixture_params` | `z_mix` (+ cubic solve) | `ln_phi_mix` (+ fugacity) | `mixture_params` share |
|---|---|---|---|---|
| 2 | 49.4 ns | 84.8 ns | 125.1 ns | **40 %** |
| 4 | 63.2 ns | 93.4 ns | 140.6 ns | **45 %** |
| 8 | 105.5 ns | 141.0 ns | 184.2 ns | **57 %** |

Plus the surrounding model paths:

| Bench | Baseline | Note |
|---|---|---|
| `ln_gamma_all_nrtl_n4` | 330.3 ns | **4.7× Wilson** — the O(n³) per-component path |
| `ln_gamma_all_wilson_n4` | 69.6 ns | rebuilds *and heap-allocates* Λ every call |
| `virial_ln_phi_mix_n4` | 225.6 ns | **more than a cubic `ln_phi_mix`** (140.6 ns) |
| `d_ln_phi_d_n_wong_sandler_n4` | 1.285 µs | **5.1×** the analytic classical path (252 ns) |

`mixture_params` at 40–57 % of `ln_phi_mix` — which is ~70 % of the flash — makes
it ~30–40 % of the whole solve, and almost all of it is composition-independent.
That is audit Part 2 §1, and it is where the work went.

---

## 2. Second-auditor verdict on the fourteen Part 2 recommendations

| # | Audit recommendation | Verdict |
|---|---|---|
| 1 | `PreparedModel` / `TpCache` — cache at the right granularity | **Executed**, restructured |
| 2 | `Component` → struct-of-arrays | **Rejected** |
| 3 | Remove `Vec<Vec<f64>>` from hot matrices | **Partly executed** (virial, activity); rejected for `kij` |
| 4 | Precompute `sqrt_ai` in classical mixing | **Executed** — but see §3, it was not the real cost |
| 5 | Cache Wilson Λ / NRTL τ, G at fixed T | **Executed** |
| 6 | Const-generic `DualN` for composition derivatives | **Deferred** |
| 7 | `MixtureScratch<'a, D>` instead of `SmallVec` for duals | **Deferred** |
| 8 | Stack LU; stop inverting the Broyden Jacobian | **Rejected** |
| 9 | Flatten + single-pass virial `Bᵢⱼ` | **Executed** |
| 10 | Rayon batch writes straight into SoA output | **Deferred** |
| 11 | Cache-line alignment / false sharing | **Rejected** (no measured contention) |
| 12 | Function-pointer kernel dispatch outside inner loops | **Rejected as written**; the real problem was found and fixed |
| 13 | Separate checked APIs from `unsafe` unchecked kernels | **Principle adopted, `unsafe` rejected** |
| 14 | Suggested implementation order | Followed for §1–§5, reordered by measurement |

### Where this plan departs from the audit, and why

**§1 — restructured to avoid duplicating the mixing rules.** The audit proposes
a new `PreparedModel` + `TpCache` pair holding flattened copies of everything.
Built literally, that means a second implementation of every mixing rule: one
against `MixtureSpec`, one against `PreparedModel`. Instead, `pure_params`
became an *input*:

```rust
pub fn mixture_params<D>(spec, t, p, x)          // builds `pure`, delegates
fn  mixture_params_with<D>(spec, t, x, &PureParams<D>)   // the mixing algebra
```

The composition-dependent algebra stays written **once**, generic over the
scalar type, serving both the `f64` value path and the dual-number derivative
paths. `TpCache` simply owns a `PureParams<f64>` and a `(eos, T, P, n)` key.
Same hoist, no fork.

**§2 — `Component` SoA rejected.** `Component` is *the* public type of this
crate: it appears in `SystemSpec`, `MixtureSpec`, the component database, the
PyO3 bindings, the UniFFI Swift/Kotlin surface and the wasm bindings. A parallel
SoA representation is either a breaking change across all of them or a second
representation to keep in sync forever. And §1 removes most of the motivation:
components are now iterated **once per (T, P)** instead of ~20 times per solve,
so the cold-metadata-through-cache-lines problem is amortized to near nothing.
Revisit only if a profile after §1 still shows component iteration hot.

**§3 — `kij` flattening rejected, for now.** Flattening the *public*
`&[Vec<f64>]` would break `SystemSpec`, `MixtureSpec`, both PyO3 modules and the
FFI crates. The internal alternative — copying into a flat buffer per call —
adds work on the uncached path. Meanwhile LLVM already hoists the row pointer
`kij[i]` out of the inner `j` loop, so the pointer chase is per-row, not
per-element. The measured cost did not justify the blast radius. The audit's
flat-matrix argument *was* applied where it costs nothing: the virial `Bᵢⱼ`
(§9) and the activity matrices (§5), both of which are internal.

**§8 — Broyden rejected on a factual basis.** The audit criticizes
`numerics/broyden.rs` for forming `J⁻¹` with `try_inverse`. But **Broyden has no
production callers in this engine** — `grep` finds only a PyO3 binding exposing
it as a standalone numerics utility. No flash, bubble/dew, envelope or critical
path uses it. Beyond that, the criticism misreads the method: Broyden's whole
point is the Sherman–Morrison *inverse* update, which is O(n²) per iteration
precisely because it never re-factorizes; the code already documents that the
O(n³) inversion happens only at refresh points. "Never form J⁻¹" is right for
methods that factor-and-solve each step, not for a quasi-Newton inverse update.
The audit itself hedges ("or use an inverse update only when profiling proves
beneficial") — profiling proves nothing here, because nothing calls it.

**§11 — false sharing rejected.** The audit already concedes it found no lock
contention and offers alignment speculatively. The batch path measures
0.29 µs/point across 200 000 points with linear scaling; there is no contention
signal to chase. Adding `#[repr(align(64))]` on a guess is cargo-cult tuning.

**§12 — function-pointer dispatch rejected as written; the real problem was
different.** Replacing an enum `match` with stored `fn` pointers converts a
call LLVM can inline into an indirect call it cannot. That is usually a
pessimization, and the audit offers no measurement. **But the underlying
instinct — "indirection inside the n² loop is costing you" — turned out to be
exactly right, in a place the audit did not look.** See §3 below.

**§13 — principle adopted, `unsafe` rejected.** Part 1 already applied
checked-boundary / unchecked-hot-path (validate the feed once, never per
iteration) with **zero** `unsafe`. Adding `get_unchecked` to a thermodynamics
library to remove bounds checks that branch-predict perfectly is a poor trade of
memory safety for noise.

**§6, §7, §10 deferred.** Each is real and each is its own project. §6 (a
hand-written const-generic dual to replace `num-dual`'s per-column sweeps) is
the largest remaining algorithmic win — Wong-Sandler composition derivatives are
5.1× the analytic path — but it means reimplementing dual arithmetic with
accuracy risk on the exact path M12.3's Gibbs–Helmholtz invariants guard. §10
needs a batch profile first: the batch already runs at 0.29 µs/point with the
GIL released, so the AoS→SoA transpose is not obviously dominant.

> **Correction, 2026-08-15 — §6's stated deferral reason is wrong.** The
> sentence above says §6 "means reimplementing dual arithmetic". It does not.
> `num-dual` 0.11.2 — already in the dependency tree — supplies
> `DualSVec64<W>`, a statically-sized multidirectional dual that is `Copy` for
> const `W` and therefore satisfies the existing `D: DualNum<f64> + Copy` bound
> **with no new arithmetic and no trait-bound edit**. A patch built on it
> compiled and passed all 194 tests, including the FD oracle and the symmetry
> invariant, and measured **566 ns vs the 1288 ns baseline at n = 4 (2.28×)**.
>
> Two things keep §6 deferred anyway, and neither is the accuracy risk quoted
> above. First, the win **does not scale**: at n = 20 the same patch measured
> 61.0 µs against a 61.6 µs baseline — 1.0× — because the block width is the
> real tunable, and re-measuring at `W = 8` gave **17.1 µs (3.6×)**. Second,
> the n = 4 classical path regressed +4.4 % (242.5 → 253.2 ns) for reasons
> nobody has explained. So §6 now needs *width tuning plus a benchmark set
> covering n > 8*, which is ordinary work — not the milestone-sized accuracy
> risk recorded above.
>
> Provenance and full numbers: [`SECOND_OPINION_TRIAL.md`](SECOND_OPINION_TRIAL.md).
> Nothing is merged; `d_ln_phi_d_n` on `main` is unchanged.

---

## 3. The finding the audit missed: `&dyn Fn` in the n² loop

Audit §4 says classical mixing wastes `n²` square roots evaluating
`(ai[i]*ai[j]).sqrt()`, and it is right — only `n` are needed. That hoist was
implemented. **It barely moved the benchmark.**

The reason is one line the audit never quotes. `mixture_params` computed the
quadratic mixing sum through a helper whose signature was:

```rust
let quad_a = |x: &[D], a_ij: &dyn Fn(usize, usize) -> D| -> (D, Buf<D>) { … };
```

`a_ij` is a **trait object**, invoked once per `(i, j)` pair — `n²` indirect
calls per mixture evaluation, through a vtable LLVM can neither inline nor
vectorize through. With that indirection in place, removing `n²` square roots
from inside the closure changed almost nothing: the call overhead, not the
`sqrt`, was the limit.

Converting `quad_a` from a closure taking `&dyn Fn` into a generic function
taking `F: Fn(usize, usize) -> D` monomorphizes the whole loop:

```rust
fn quad_a<D: DualNum<f64> + Copy, F: Fn(usize, usize) -> D>(
    n: usize, x: &[D], a_ij: F,
) -> (D, Buf<D>)
```

This is audit §12's instinct — "fast-path dispatch must happen outside inner
loops" — landing in the place that actually had dynamic dispatch in an inner
loop. The audit proposed introducing function pointers into the fugacity kernels
(which would have made things worse) while a genuine vtable call sat `n²`-deep
in the mixing rule it was reviewing three sections earlier.

**Lesson, same as Part 1's:** the audit generates good hypotheses and cannot
rank them. "Precompute the square roots" and "the cross-parameter closure is a
trait object" look equally plausible on paper. Only one of them was the
bottleneck, and only a benchmark could say which.

---

## 4. What shipped

**`mixture::TpCache` (public) + `flash::SystemTpCache` (crate-internal)** —
audit §1. `TpCache` holds one `PureParams<f64>` keyed on `(eos, T, P, n)`;
`SystemTpCache` composes up to two of them (the phases may use different EOS)
plus the γ-φ and virial constants below. `flash_isothermal` and
`stability_analysis` build one per solve. `TpCache::matches` makes a stale cache
an error, not a silently wrong number.

**γ-φ constants hoisted** — `Psatᵢ(T)`, `φᵢˢᵃᵗ(T, Psatᵢ)` and the Poynting
factor are all `(T, P)`-only, but were recomputed per component per iteration.
They now collapse into one precomputed vector, so the γ-φ K-value assembly at
iteration time is literally `ln Kᵢ = ln γᵢ(x) + constᵢ − ln φ̂ᵢⱽ(y)`.

**`activity::ActivityTpCache`** — audit §5. Flat row-major Wilson `Λᵢⱼ` and NRTL
`τᵢⱼ`, `Gᵢⱼ`, `τᵢⱼGᵢⱼ`. NRTL previously took the per-component fallback path,
recomputing the O(n²) column sums for **every** `i` — O(n³) `exp` calls per
`ln_gamma_all`. The free `ln_gamma_all` now routes through the cache too, so
even uncached callers get the O(n²) form.

**Flat, single-pass virial** — audit §9. `b_mix_matrix_flat` (one allocation,
contiguous rows) and `ln_phi_mix_virial_flat_into`, which computes the row dot
products once and reuses them for `B_mix` instead of traversing the matrix
twice. Cached per `(T)` in `SystemTpCache` for a virial vapor.

**`quad_a` devirtualized** and **√Aᵢ / √Bᵢ hoisted** — audit §4 plus §3 above.
The roots are built only for the rules that form `√(AᵢAⱼ)` cross terms; the
GE-based rules never touch them and no longer pay.

---

## 5. Results

Measured back-to-back on one machine (stash the source, `--save-baseline`,
restore, `--baseline`), so machine drift is excluded. The untouched control
benches (`alpha`, `z_factor`, `saturation`) moved **+3…+5 %** in this run, which
means every improvement below is *understated* by roughly that much.

> **Measurement note.** Neither Part 1 nor Part 2 is committed, so
> `git stash push -- engine/src` reverts **both**. The directly measured
> comparison below is therefore *original engine → Part 1 + Part 2*. Part 2's
> marginal column is derived from that and Part 1's independently measured
> delta; it is arithmetic on two drift-controlled ratios, not a third
> measurement.

| Benchmark | Original | After Part 1 | After Part 2 | **Cumulative** | Part 2 marginal |
|---|---|---|---|---|---|
| `isothermal_n2` | 3.47 µs | 2.98 µs | 2.64 µs | **−23.8 %** | −11.4 % |
| `isothermal_n4` | 4.12 µs | 3.62 µs | 3.08 µs | **−25.1 %** | −14.8 % |
| `isothermal_n6` | 5.24 µs | 4.44 µs | 3.79 µs | **−27.6 %** | −14.5 % |
| `isothermal_n8` | 6.10 µs | 5.20 µs | 4.39 µs | **−28.0 %** | −15.6 % |
| `isothermal_flash_rks_binary` | 2.89 µs | — | 2.35 µs | **−18.8 %** | — |
| `stability_n2` | 8.10 µs | 4.38 µs | 3.95 µs | **−51.2 %** | −9.9 % |
| `stability_n4` | 11.19 µs | 6.40 µs | 5.80 µs | **−48.2 %** | −9.5 % |
| `stability_n6` | 16.49 µs | 10.39 µs | 9.18 µs | **−44.3 %** | −11.6 % |
| `stability_n8` | 10.93 µs | 6.95 µs | 6.13 µs | **−43.9 %** | −11.7 % |

Component-level, measured directly:

| Benchmark | Before | After | Change |
|---|---|---|---|
| `mixture/ln_gamma_all_nrtl_n4` | 330.3 ns | 105.7 ns | **−67.7 %** |
| `mixture/virial_ln_phi_mix_n4` | 225.6 ns | 181.2 ns | **−20.8 %** |
| `mixture/ln_phi_mix_n8` | 202.1 ns | 180.7 ns | **−10.6 %** |
| `mixture/ln_phi_mix_n2` / `n4` | — | — | −5.8 % / −5.5 % |
| `flash_multi/k_values_n8` | 411.3 ns | 362.2 ns | −11.9 % |
| `derivatives/k_values_binary` | 338.6 ns | 322.0 ns | −4.9 % |

### Two honest negatives

- **`mixture/z_mix_n4` +17.5 %** (and `mixture_params_n2/n4` +4…+6 %). These are
  inlining/code-layout artifacts: `z_mix` is a thin public wrapper that no
  driver calls, and the regression reproduces across runs while every path the
  drivers *do* take (`ln_phi_mix`, `k_values`, the flash, stability) improved.
  `z_mix_n8` is −2.3 %, so it is not even monotonic in `n` — the signature of
  layout noise rather than added work.
- **`k_values_gamma_phi_n4` marginal +7.5 %** on the **uncached** public path.
  `ln_gamma_all` now builds an `ActivityTpCache` enum where it used to build a
  `WilsonCache` directly, which costs a branch. The flash never takes this path
  — it uses the cached form — and NRTL, the model that actually needed help, is
  −67.7 % on the same uncached path.

Note also that `k_values_n{2,4,6}` shows a ~0 % Part 2 marginal. That is
expected and correct: the public `k_values` bench exercises the **uncached**
route, so it sees only the sqrt hoist and the devirtualization. The `TpCache`
benefit is visible only where a caller sweeps composition at fixed `(T, P)` —
i.e. inside the flash, which is −25 %.

### Verification

- `cargo fmt --all --check` clean; `cargo +1.97.0 clippy --workspace --all-targets` **0 warnings**.
- `cargo test --workspace` — **291** tests (up from 287; 4 new: `TpCache`
  equivalence against the uncached path across three compositions and both
  roots, `TpCache` mismatch rejection, `ActivityTpCache` agreement with direct
  per-component evaluation for Wilson/NRTL/Van Laar/ideal, and flat-vs-nested
  virial agreement). Chapter IV Tables 4.10 and 4.11/4.12 intact.
- Python: `maturin develop --release`, **450 passed, 1 skipped**. `flash_pt`
  mass balance 5.6e-17; `flash_pt_batch` 200 000 points in 59 ms (0.29 µs/pt);
  NRTL γ-φ flash through `vle.System` converges with mass balance 1.1e-16.

---

## 6. What remains

In the order the evidence now supports:

1. **§6 — composition-derivative strategy.** Wong-Sandler `∂lnφ̂ᵢ/∂nⱼ` is 1.285 µs
   vs 243 ns analytic. One dual evaluation per Jacobian column is O(n³); a
   fixed-width multi-derivative dual would make it one pass. Largest remaining
   algorithmic win, and the highest accuracy risk — its own milestone.
   **Revised 2026-08-15:** the route is now known —
   `num_dual::DualSVec64<W>` in `W`-wide column blocks, measured at 2.28× for
   n = 4 and 3.6× for n = 20 *at the right `W`*, with no new dual arithmetic
   and no bound changes. The open work is choosing `W` per bucket **by
   measurement** and adding benchmarks above n = 8. See the correction note in
   §2 and [`SECOND_OPINION_TRIAL.md`](SECOND_OPINION_TRIAL.md). Still unmerged.
2. **§10 — batch SoA output.** Profile the batch path first.
3. **§3 — flat `kij`.** Only worth it bundled with a deliberate 1.0 API break.
4. **§2 — `Component` SoA.** Only if a post-§1 profile still shows it.
5. **§7 — `MixtureScratch`.** Same trigger as §6; they are the same code path.

---

## 7. Milestone 18 — the N-scalable mixture core

*Executed 2026-08-15 by Claude Code using Claude Opus 5 (1M context). Plan of
record: [`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](PETROLEUM_PSEUDOCOMPONENT_PLAN.md) §1.1 (blocker B1).*

Not an audit response — this one came from the petroleum track — but it lands
in the same file because it is the same code, and because it settles a
question Part 2 left open: **how does the mixture core behave as N grows?**
Every measurement in this plan above is at n ≤ 8.

### 7.1 What changed

`quad_a` ran its full double loop unconditionally. With every `kᵢⱼ = 0` the
cross-parameter factorizes and the form collapses to a single pass:

```text
S = Σⱼ xⱼ√Aⱼ        A = S²        Āᵢ = 2√Aᵢ·S
```

That is the crude-assay case exactly — interaction parameters between
petroleum pseudocomponents are conventionally zero, so `kij` is left empty and
the collapse is free to detect. Three forms now exist, selected by `kij_form`:

| form | when | cost |
|---|---|---|
| `quad_a_factorized` | `kij` empty, or an index proving all-zero | **O(N)** |
| `quad_a_sparse` | index with density < 0.15 | **O(N + nnz)** |
| `quad_a` | otherwise | O(N²) |

`KijIndex` scans the matrix once and lives in `TpCache`, because scanning an
N×N matrix per call is itself O(N²) and would defeat the purpose. The uncached
entry point keeps the free half (empty ⇒ factorized) and otherwise runs the
general loop, whose complexity a per-call scan could not have improved on.

The same collapse turns the analytic composition Jacobian into a **sum of
rank-1 outer products** — the only place `Aᵢⱼ` enters `d_ln_phi_d_n_classical`
is a single `−2Ĩ√Aᵢ√Aⱼ` term, and every other `(i, j)` term was already an
i-vector times a j-vector. So `d_ln_phi_d_n_apply` computes `J·v` in O(N)
without ever forming the block.

### 7.2 Measured — `mixture_scaling` criterion group

Whole-fugacity evaluation, PR1976, classical mixing:

| N | dense O(N²) | factorized O(N) | speedup | + `TpCache` |
|---:|---:|---:|---|---:|
| 10 | 278 ns | 225 ns | 1.24× | 141 ns |
| 50 | 1.845 µs | 502 ns | 3.68× | 240 ns |
| 100 | 6.623 µs | 833 ns | **7.95×** | 367 ns |
| 300 | 60.74 µs | 1.978 µs | **30.7×** | **825 ns** |

Composition Jacobian, formed vs applied:

| N | formed O(N²) | applied O(N) | speedup |
|---:|---:|---:|---|
| 10 | 652 ns | 320 ns | 2.0× |
| 50 | 7.304 µs | 939 ns | 7.8× |
| 100 | 26.08 µs | 1.672 µs | 15.6× |
| 300 | 216.7 µs | 4.297 µs | **50.4×** |

**The scaling is the point, not the ratio.** Dense grows 8.3× from N = 100 to
N = 300 (≈ 3² = 9, quadratic); factorized grows 2.37× and applied 2.57×
(≈ 3, linear). The milestone's own acceptance criterion was "linear growth or
it did not land" — it is linear.

The sparse path on a realistic assay pattern (three light gases against every
hydrocarbon, ~2 % fill) reads 5.25 µs at N = 300 against the dense 60.7 µs —
11.6×, also linear.

### 7.3 The threshold was wrong, and the sweep is what caught it

`SPARSE_KIJ_MAX_DENSITY` was first written as `0.25` with a comment claiming it
came from measurement. It had not been measured. A density sweep at N = 100
showed the dense path costs a flat ~5.91 µs regardless of fill, while sparse
grows linearly and crosses it at **d ≈ 0.19**:

| density | 0.010 | 0.050 | 0.099 | 0.198 |
|---|---:|---:|---:|---:|
| sparse | 476 ns | 1.609 µs | 3.241 µs | 6.135 µs |

At the originally chosen 0.25 the engine would have taken the sparse path in a
band where it is measurably *slower*. The constant is now **0.15**, below the
crossover, because the constant factor drifts with N and being on the wrong
side costs more when the matrix is large.

This is the *Related rule* from `CLAUDE.md` catching its author: a performance
claim written into a code comment without a number behind it was wrong within
the hour. The sweep bench was temporary and is not committed; its numbers live
here instead.

### 7.4 Wong-Sandler collapses too — for a different reason

`quad_a` is not the only quadratic in the core. **Wong-Sandler has its own**
(`bij_ws`), and Part 2 §1 measured its composition derivative at 5.1× the
analytic path. Its cross term is a **sum**, not a product:

```text
bijᵂ = ½(cᵢ + cⱼ)(1 − kᵢⱼ),   cᵢ = Bᵢ − Aᵢ
```

so with every `kᵢⱼ = 0` the double sum separates a different way — by
distributing rather than factoring:

```text
Qᵂ = ½ΣᵢΣⱼ xᵢxⱼ(cᵢ + cⱼ) = C·X       Σⱼ xⱼbijᵂ = ½(cᵢ·X + C)
```

with `C = Σxᵢcᵢ`, `X = Σxᵢ`. Written with `X` rather than assuming `Σx = 1`,
because the dual paths normalize *in dual arithmetic* and the identity has to
hold there too. `wong_sandler_collapse_matches_general_path` pins it at
N = 2/3/12/45 against a dense all-zeros matrix.

At n = 4 this is worth ~2 % — the cost there is the Wilson activity model and
the four dual sweeps, not the 16-term quadratic. It is an O(N²) → O(N) shape
change, so like the classical collapse its value is at scale.

### 7.5 A benchmark that cannot resolve what was asked of it

`d_ln_phi_d_n_classical_n4` was read three times across builds whose changes
touched **different mixing rules**:

| build | time |
|---|---:|
| before Milestone 18 | 242.0–243.0 ns |
| after the classical collapse | 230.3–231.6 ns |
| after the Wong-Sandler collapse | 241.8–244.1 ns |

Two consecutive runs of the *same* build agree to 0.5 % (239.9, 241.2 ns). So
the bench carries ~**±5 % build-to-build variance from code layout** against
~1 % run-to-run variance, and the middle row is not a Milestone 18 win — it is
the same artefact that produced the "unexplained" 4.4 % regression recorded in
[`SECOND_OPINION_TRIAL.md`](SECOND_OPINION_TRIAL.md) §6, now resolved there.

**Neither delta was real.** This is worth stating plainly because both were
briefly written down as findings: a single-digit-percent change on this
benchmark is evidence of nothing unless the build is otherwise identical. The
Milestone 18 claims that *do* hold are the ones with an order of magnitude
behind them — 30.7× and 50.4× at N = 300, where the asymptotics dwarf layout.

### 7.6 Allocation-free evaluation (U6) — done, with one cost recorded

`mixture_params_with` now **fills a caller-provided `MixtureParams`** instead
of returning one. Every mixing branch writes into `out.a_bar` / `out.b_bar`,
`three_param_uw` writes `out.u_bar` / `out.w_bar`, and the `quad_a` family
writes into caller slices (including the sparse path's row-correction
scratch). The public `MixtureWorkspace` owns those buffers, and
`ln_phi_mix_cached_ws_into` is the entry point that reuses them. Buffers only
grow, so a solve settles after its first evaluation and then allocates
nothing.

**The algebra is still written once** — this is a change of *where the
buffers live*, not a second implementation. That was the binding constraint
(§2, `PreparedModel` rejected), and it held.

Measured, same build (the only comparison layout noise cannot distort):

| N | allocating (`ln_phi_mix_cached_into`) | workspace (`..._ws_into`) | |
|---:|---:|---:|---|
| 10 | 165.1 ns | **83.9 ns** | 2.0× |
| 50 | 285.6 ns | **171.6 ns** | 1.7× |
| 100 | 447.7 ns | **302.0 ns** | 1.5× |
| 300 | 1081.3 ns | **876.0 ns** | **1.23×** |

The N = 300 figure matches the standalone probe that sized this work before it
started — four `SmallVec<[f64; 8]>` buffers at N = 300 cost **145.7 ns**
against an 825 ns call, ≈18 %. The measurement predicted the outcome.

**The cost, recorded rather than buried.** A *fresh* workspace is slightly
slower than the old code, because `resize` zero-fills four buffers the mixing
branch then overwrites, where the old path built three of them with `collect`
in a single pass. At N = 300 that is ~7 KB of redundant zeroing. It is paid
**once** by a reused workspace — and on every call by the compatibility
wrapper `ln_phi_mix_cached_into`, which constructs a fresh one.

That wrapper is on the flash's hot path, so the question that mattered was
whether the flash regressed. It did not — measured against the v0.12.0
figures recorded in [`OPTIMIZATION_PLAN_PART1.md`](OPTIMIZATION_PLAN_PART1.md) §6 on this machine:

| bench | v0.12.0 | now | |
|---|---:|---:|---|
| `flash_multi/isothermal_n4` | 3.58 µs | **2.93 µs** | −18 % |
| `flash_multi/isothermal_n8` | 5.21 µs | **3.94 µs** | −24 % |
| `flash_multi/stability_n4` | 6.40 µs | **5.33 µs** | −17 % |

The collapses gain the flash more than the extra fill costs it. **The
remaining work, if the fill ever matters, is threading a `MixtureWorkspace`
through `flash/system.rs`'s five `&SystemTpCache` entry points** — not
attempted here because the end-to-end measurement says there is nothing to
recover yet. Do not do it on the strength of the microbench alone; measure the
flash first.

---

## References

- [`optimizations_audit.md`](optimizations_audit.md) — the external audit
- [`OPTIMIZATION_PLAN_PART1.md`](OPTIMIZATION_PLAN_PART1.md) — the flash layer
- [`OPTIMIZATION_AUDIT_HISTORY.md`](OPTIMIZATION_AUDIT_HISTORY.md) — provenance and lessons
- [`PERFORMANCE_PROPOSAL.md`](PERFORMANCE_PROPOSAL.md) — Tracks A–E, the plan under audit
