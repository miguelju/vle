# Optimization Plan — Part 1: Flash Calculation Deep-Dive

**Status: complete (2026-07-25).** Executed by Claude Code using Claude Opus 5 (1M context).

Plan of record for the engine's response to **Part 1** of
[`optimizations_audit.md`](optimizations_audit.md), the external performance audit
of the flash layer. Part 2 of that audit (the global package optimization) is
**not** covered here — see [`OPTIMIZATION_AUDIT_HISTORY.md`](OPTIMIZATION_AUDIT_HISTORY.md)
for where it sits and why it matters more than Part 1 does.

This document records three things the audit itself does not contain: a
**measured** baseline, a **second-auditor evaluation** of each recommendation,
and the **result** of executing the accepted ones. Two of the audit's concrete
Part 1 proposals turned out to be measurable regressions on this codebase; both
are documented below and in the code, because a rejected optimization with a
number attached is worth more to a reader than a silent omission.

---

## 1. The measured baseline

Criterion was already wired into this repo (Milestone 8.2 — `engine/Cargo.toml`
dev-dependency, `[[bench]] engine_bench`, `harness = false`). What was missing
was **multicomponent** flash coverage, so the audit's central claim could not be
tested against this engine. `engine/benches/engine_bench.rs` gained a
`flash_multi` group that measures the three layers **separately** at
n = 2, 4, 6, 8, over an n-alkane series (methane → n-octane) at 350 K / 2000 kPa
— a state that is genuinely two-phase at every n, so every size measures the
same kind of work rather than an early single-phase bail-out.

Baseline, Apple Silicon, `lto = "fat"` + `codegen-units = 1`:

| n | `isothermal_n{n}` | `k_values_n{n}` | `rachford_rice_n{n}` | `stability_n{n}` |
|---|---|---|---|---|
| 2 | 3.317 µs | 268.9 ns | 4.12 ns | 7.78 µs |
| 4 | 3.951 µs | 307.2 ns | 37.8 ns | 10.82 µs |
| 6 | 5.006 µs | 350.5 ns | 48.1 ns | 15.97 µs |
| 8 | 5.948 µs | 403.8 ns | 90.3 ns | 10.73 µs |

The flash converges in 7–10 outer iterations at every n, which attributes the
driver's time as:

| n | K-value thermodynamics | Rachford-Rice | orchestration overhead |
|---|---|---|---|
| 2 | **81 %** | 1 % | 18 % |
| 4 | **70 %** | 9 % | 21 % |
| 8 | **68 %** | 15 % | 17 % |

**The audit's headline claim is confirmed by measurement.** Rachford-Rice is
1–15 % of the flash; the surrounding thermodynamics is ~70 %.

It also bounds what Part 1 can deliver. Part 1's own recommendations target the
17–21 % orchestration slice plus whatever can be shaved off the edges of
`k_values`. The 70 % lives inside `mixture_params` → `ln_phi_mix`, which the
audit files under **Part 2** §1/§3/§4. A 2× flash speedup is a Part 2 outcome.

---

## 2. Second-auditor verdict on the nine Part 1 recommendations

| # | Audit recommendation | Verdict | Outcome |
|---|---|---|---|
| 1 | Reusable `FlashWorkspace`, `*_into` kernels | **Executed**, modified | shipped |
| 2 | Keep K in log form end-to-end | **Executed** | shipped |
| 3 | RR preprocessing + safeguards | **Split**: safeguards executed, preprocessing rejected | partly shipped |
| 4 | RR SIMD / 4-way unroll | **Rejected** | not shipped |
| 5 | GDEM trust region | **Executed**, modified | shipped |
| 6 | Full Newton finish | **Deferred** | not shipped |
| 7 | Stability rewrite | **Split**: buffers + numerics executed, multi-seed deferred | partly shipped |
| 8 | `min_gibbs_ln_phi` shares one mixture state | **Executed**, modified | shipped |
| 9 | Analytic envelope Jacobian | **Deferred** | not shipped |

### Where this plan departs from the audit, and why

**§1 — inline width 8, not 16.** The audit proposes
`SmallVec<[f64; 16]>`. The mixture core already standardizes on
`type Buf<D> = SmallVec<[D; 8]>`; introducing a second inline width for no
measured reason is exactly the inconsistency that rots a codebase. A
`FlashWorkspace` is built **once per flash and reused across every outer
iteration**, so a spill at n > 8 costs one allocation amortized over the whole
solve. Width 8 it is.

**§1 — "error strings allocate every iteration" is factually wrong.**
`FlashError::Thermo(e.to_string())` allocates only on the failure path.

**§1 — `FlashWorkspace` stays private.** It is a performance detail, not
something a caller should have to name. `flash_isothermal` builds one per call:
one allocation per flash instead of ~50. When the Rayon batch path wants a
per-worker workspace (Part 2 §10) this grows a public
`flash_isothermal_with_workspace`; committing published-crate API surface ahead
of a real consumer is premature. **Approved by the repo owner before execution.**

**§3 — "reject a feed that is not normalized" rejected.** Rachford-Rice is
scale-invariant in `z`; rejecting unnormalized feeds is a gratuitous API break.
The checks that catch genuine bugs (non-finite/negative `zᵢ`, non-finite or
non-positive `Kᵢ`, empty mixture, non-positive tolerance) **are** implemented.

**§3 — the `RrTerm` precomputation was benchmarked and rejected.** See §3 below.

**§3 — the f(0)/f(1) physical-bracket probe was benchmarked and rejected.**
See §3 below.

**§4 — SIMD/unrolling rejected outright.** Every Chapter IV validation case is
≤ 4 components; the largest bench is 8. A 4-wide unroll leaves a remainder loop
doing most of the work at n = 2–6, and the audit itself concedes SIMD will not
help binary or ternary mixtures. It would buy a fraction of a layer that is
1–15 % of the flash, at the cost of making a deliberately readable numerical
kernel unreadable. Revisit only if a large-n or batch-dominated workload appears.

**§5 — the residual-decrease test is retrospective, not prospective.** The
audit says "evaluate the accelerated candidate and accept it only if the
residual norm decreases". Evaluating a candidate costs a full `k_values` —
about 70 % of an iteration. Instead the cheap guards apply for free, and the
decrease test runs on the *next* iteration against the saved plain-SS
alternative. Zero extra cost in the common case; a single wasted model
evaluation only when the acceleration was actively harmful.

**§6 — Newton finish deferred.** See §5 below; this one has a documentation
history worth reading.

**§8 — `MixturePoint<const N: usize>` rejected as a shape.** Right idea, wrong
type. A const-generic `[f64; N]` would infect the public API for no gain over
the existing `Buf`. The same 2× comes from an internal
"build `MixtureParams` once, evaluate both roots against it" split.

**§7 multi-seed and §9 envelope Jacobian deferred.** Both are large and neither
is a performance fix in the Part 1 sense — more stability seeds make stability
*slower* (and more correct), and the envelope rewrite is 40 lines of hand-indexed
block assembly consuming three separate derivative paths. Each deserves its own
change with its own tests. The envelope also has **no benchmark yet**, so
deferring it is also "measure before rewriting".

---

## 3. Two audit recommendations that benchmarked as regressions

Both live in Part 1 §3, both sound obviously right, and both made this engine
slower. They are documented in the code at the site where a future reader would
otherwise re-propose them.

### Precomputing `cᵢ = Kᵢ − 1` and `zᵢcᵢ` — a 30–200 % regression

The argument is sound: `c` and `zc` are invariant for a whole solve but were
rebuilt on every Halley evaluation. The arithmetic tells a different story.

Per component per evaluation, the original does one subtract, one FMA, one
multiply and **one division**. The precomputed version does one FMA, two
multiplies and **one division**. The hoist saves a subtract and a multiply —
cheap ALU work that already hides under the division's latency — while the
preparation pass costs *more* than that for a solve converging in a handful of
iterations. Worse, `[{c, zc}]` is an array-of-structs, so it strides the loads
that `z` and `k` deliver contiguously; the audit warns about exactly this
layout problem in Part 2 §3.

Measured (`rachford_rice_n{2,4,6,8}`): **+30 % to +200 %.** Reverted.

### Probing f(0) and f(1) to narrow the bracket to [0,1] — a 25–200 % regression

The audit suggests testing the physical bracket first so the hot path does not
carry the full negative-flash window. But for a real mixture the
Leibovici–Neoschil poles are set by the most and least volatile components, and
already sit *just* outside [0, 1] — for the benchmark binary, the window is
[−0.038, 1.012]. Narrowing it to [0, 1] saves essentially no iterations, while
the two probes cost two extra divisions per component up front.

At n = 2 the probe alone took the solve from 12.5 ns to 38.9 ns. Reverted.

### What *was* kept from §3

- Input validation (non-finite/negative `z`, non-positive/non-finite `K`, empty
  mixture, bad tolerance) — but **only at the API boundary**. The flash driver
  validates its feed once at entry and never re-checks the K-values it produced
  itself. That is the audit's own Part 2 §13 principle ("separate checked APIs
  from unchecked hot kernels") applied to the flash, and it is why the public
  `rachford_rice` bench shows +7…+85 % (it now validates) while the flash it
  serves got 12–15 % *faster*.
- Excluding `zᵢ = 0` and degenerate `|Kᵢ − 1| ≤ 1e-14` components **from the
  pole bracket** (they contribute ~0 to the sums anyway, so they stay in
  `rr_fdd`, but letting them pin a pole is what breaks things).
- A scale-aware Halley acceptance test, replacing `denom.abs() > 0.0`.
- A scale-safe pole nudge (`max(1e-10·span, 16·ε·|v|)`) replacing the fixed
  `1e-10·span`, which was simultaneously too coarse for a narrow window and too
  fine to move a large-magnitude endpoint to a different representable number.
- A bracket-width stop criterion, so a β pinned to machine precision returns
  instead of burning the whole iteration budget.
- A **correct** Brent bracket-halving safeguard: force a bisection whenever the
  bracket has failed to halve over two steps. (The first implementation of this
  never updated its reference width and was silently inert; fixed.)

---

## 4. What shipped

**`ln_phi_mix_into` / `ln_phi_mix_min_gibbs_into`** (`engine/src/mixture.rs`) —
`ln_phi_all_generic` split into `mixture_params` + `ln_phi_from_params_generic`,
so both cubic roots can be evaluated against **one** shared mixture state.
`ln_phi_mix` is reimplemented on top of `ln_phi_mix_into`: one implementation of
the thermodynamics, not two.

**`ln_k_values_into`** (`engine/src/flash/system.rs`) — returns **ln K**
directly. `k_values` is now a thin `exp` over it. The γ-φ path assembles
`ln Kᵢ = ln γᵢ + ln Psatᵢ + ln φᵢˢᵃᵗ + ln POYᵢ − ln φ̂ᵢⱽ − ln P` additively
instead of exponentiating four terms and dividing the products. Supporting
log-form primitives: `saturation::ln_poynting_factor`, `flash::init::wilson_ln_k`
— each of which the corresponding value function is now defined in terms of, so
they cannot drift.

**`FlashWorkspace`** (`engine/src/flash/isothermal.rs`) — nine reused
per-component buffers. The driver iterates on `ln K` throughout; per iteration
this removes `n` `exp` (inside `k_values`), `2n` `ln` (the residual and an
`ln_k` rebuild that fired every iteration but was consumed only every fifth),
and 4–5 heap allocations.

**Guarded GDEM** — `μ < 0.95`, gain capped at `1/(1−μ) ≤ 4`, candidate `ln K`
bounded to ±80, written to a trial buffer so a rejected proposal cannot corrupt
the iterate, plus the retrospective residual-decrease rollback described above.

**Stability** (`engine/src/flash/stability.rs`) — `TrialWorkspace` reuses the
three per-iteration buffers, `min_gibbs_ln_phi_into` replaces the owned return,
and a feed with `zᵢ ≤ 0` is now rejected explicitly instead of producing a
NaN-derived verdict via `ln 0`.

**`FlashError::InvalidInput`** — new variant, mapped to Python `ValueError`
(not `RuntimeError`) in **both** `py_bindings.rs` and `py_system.rs`.

---

## 5. The Newton finish (§6) — and a documentation correction

The audit is right that `flash_isothermal` has no Newton polish. Checking what
this repo claimed turned up an **internal inconsistency**:

- `ROADMAP.md`'s Milestone 9 checkbox has always said *"(analytic-Jacobian
  Newton polish is a follow-on refinement)"* — accurate.
- `MODERNIZATION_PLAN.md`'s Milestone 9 progress note said *"Milestone 9 is
  complete: every Phase 15 algorithm is implemented"*, and its shipped-items
  list read *"GDEM-accelerated successive substitution → Newton on ln Kᵢ with
  analytic Jacobian (§J)"* — **not** accurate.

Both `MODERNIZATION_PLAN.md` lines are corrected in the same change as this
document, and §J now carries an explicit status paragraph.

The deferral stands on its merits. At the 7–14 outer iterations the Chapter IV
and benchmark systems actually take, a terminal Newton saves a handful of
iterations. Against that: near criticality `ln K → 0` and the naive {ln K, β}
system becomes rank-deficient, so it needs `n−1` independent composition
variables or β eliminated through Rachford-Rice with an explicitly projected
Jacobian, plus a trust region or line search and a conditioning test before
switching over. That is a milestone with its own validation, not a line item in
a performance pass — and folding it in here would have made every number in §6
below impossible to attribute.

---

## 6. Results

Measured back-to-back on one machine (`git stash` the source changes, keep the
bench file, `--save-baseline`, restore, `--baseline`), so machine drift is not
in the numbers. The untouched control benches — `alpha`, `z_factor`,
`saturation` — move by ±1–3 %, which is the noise floor.

| Benchmark | Before | After | Change |
|---|---|---|---|
| `flash_multi/isothermal_n2` | 3.49 µs | 3.00 µs | **−14.1 %** |
| `flash_multi/isothermal_n4` | 4.07 µs | 3.58 µs | **−12.1 %** |
| `flash_multi/isothermal_n6` | 5.17 µs | 4.38 µs | **−15.3 %** |
| `flash_multi/isothermal_n8` | 6.11 µs | 5.21 µs | **−14.7 %** |
| `flash/isothermal_flash_rks_binary` | 2.95 µs | 2.57 µs | **−12.9 %** |
| `flash_multi/stability_n2` | 8.18 µs | 4.43 µs | **−45.9 %** |
| `flash_multi/stability_n4` | 11.19 µs | 6.40 µs | **−42.8 %** |
| `flash_multi/stability_n6` | 16.34 µs | 10.29 µs | **−37.0 %** |
| `flash_multi/stability_n8` | 10.98 µs | 6.98 µs | **−36.4 %** |
| `flash_multi/k_values_n2` | 284.3 ns | 265.8 ns | −6.5 % |
| `flash_multi/k_values_n4` | 324.7 ns | 303.5 ns | −6.5 % |
| `flash_multi/k_values_n6` | 363.8 ns | 337.3 ns | −7.3 % |
| `flash_multi/k_values_n8` | 414.4 ns | 376.6 ns | −9.1 % |
| `flash_multi/k_values_gamma_phi_n4` | 160.7 ns | 135.3 ns | **−15.8 %** |
| `derivatives/k_values_binary` | 340.2 ns | 329.7 ns | −3.1 % |
| `flash_multi/rachford_rice_n2` | 4.30 ns | 7.94 ns | +84.5 % ¹ |
| `flash_multi/rachford_rice_n4` | 40.3 ns | 49.1 ns | +22.0 % ¹ |
| `flash_multi/rachford_rice_n6` | 50.7 ns | 66.8 ns | +31.7 % ¹ |
| `flash_multi/rachford_rice_n8` | 94.3 ns | 100.8 ns | +6.9 % ¹ |

¹ The public `rachford_rice` entry point now validates its inputs, which the
old one did not. That cost is **not** on the flash's hot path — the driver
validates its feed once at entry and calls the unchecked bracket + solve
thereafter, which is why the flash is 12–15 % faster in the same run. This is a
deliberate trade: a Python caller passing a bad array gets a `ValueError`
instead of a mysterious non-convergence, for a few nanoseconds on a call whose
FFI crossing already costs microseconds.

### Verification

- `cargo fmt --all --check` — clean.
- `cargo +1.97.0 clippy -p vle-thermo --all-targets` — zero warnings.
- `cargo test --workspace` — **196** engine tests (up from 180; 8 new covering
  the input validation, the absent-component filter, the degenerate-K bracket,
  the GDEM gain cap and bounds rejection, `ln_k_values_into` ↔ `k_values`
  agreement, output-length checking, and min-Gibbs equivalence against
  independent two-root evaluation) plus the 2 Chapter IV validation cases.
- Python: `maturin develop --release` in the `vle` conda env, **450 passed,
  1 skipped**. `FlashError::InvalidInput` verified to surface as `ValueError`.
  `System.flash_pt` mass balance closes to 1.1e-16; `flash_pt_batch` runs
  200 000 points in 59 ms (0.30 µs/point), 200 000/200 000 converged, and agrees
  with the scalar path to 1.6e-10.

---

## 7. What Part 1 did not touch

Carried forward, in the order the evidence supports:

1. **Part 2 §1 — the per-`(T,P)` cache.** The flash recomputes
   composition-*independent* pure-component parameters (α and the transcendental
   `Aᵢ`/`Bᵢ` work) twice per `k_values` call × ~10 iterations, when they change
   only with `(T, P)` — i.e. once per flash. This is the single largest
   remaining lever and the reason `k_values` still holds ~70 % of the flash.
2. Part 2 §3/§4 — flatten `Vec<Vec<f64>>` matrices, hoist `sqrt(Aᵢ)`.
3. Part 2 §5 — cache Wilson Λᵢⱼ and NRTL τᵢⱼ/Gᵢⱼ at fixed T.
4. Part 1 §9 — analytic envelope Jacobian (benchmark it first).
5. Part 1 §6 — the Newton finish, as its own milestone.
6. Part 1 §7 — multi-seed stability, as a correctness change.

---

## References

- [`optimizations_audit.md`](optimizations_audit.md) — the external audit this responds to
- [`OPTIMIZATION_AUDIT_HISTORY.md`](OPTIMIZATION_AUDIT_HISTORY.md) — provenance, and what the audit got right and wrong
- [`PERFORMANCE_PROPOSAL.md`](PERFORMANCE_PROPOSAL.md) — the accepted Track A–E plan the audit was asked to critique
- [`MODERNIZATION_PLAN.md`](MODERNIZATION_PLAN.md) §F, §I, §J — Rachford-Rice, stability, and the SS→Newton scheme
