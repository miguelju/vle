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
2. **§10 — batch SoA output.** Profile the batch path first.
3. **§3 — flat `kij`.** Only worth it bundled with a deliberate 1.0 API break.
4. **§2 — `Component` SoA.** Only if a post-§1 profile still shows it.
5. **§7 — `MixtureScratch`.** Same trigger as §6; they are the same code path.

---

## References

- [`optimizations_audit.md`](optimizations_audit.md) — the external audit
- [`OPTIMIZATION_PLAN_PART1.md`](OPTIMIZATION_PLAN_PART1.md) — the flash layer
- [`OPTIMIZATION_AUDIT_HISTORY.md`](OPTIMIZATION_AUDIT_HISTORY.md) — provenance and lessons
- [`PERFORMANCE_PROPOSAL.md`](PERFORMANCE_PROPOSAL.md) — Tracks A–E, the plan under audit
