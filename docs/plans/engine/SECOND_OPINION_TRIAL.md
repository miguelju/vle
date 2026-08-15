# Second-Opinion Trial — an external model on the composition-derivative path

*Trial run 2026-08-15. Harness and grading by Claude Code using Claude Opus 5
(1M context). The external answer is by OpenAI `gpt-5.6-sol` and is reproduced
verbatim, unedited, in
[`second-opinion/dualn-response-sol.md`](second-opinion/dualn-response-sol.md).*

**Nothing in this document has been merged.** The engine on `main` is
unchanged. This is a record of an experiment: a deferred optimization was
handed to a second model, and its answer was compiled, tested and benchmarked
rather than believed. What it got right and what it got wrong are both recorded
below, because both are the point.

---

## 1. Why run this at all

[`OPTIMIZATION_AUDIT_HISTORY.md`](OPTIMIZATION_AUDIT_HISTORY.md) records that an
external audit produced four textbook-correct recommendations that either made
this engine measurably slower or targeted code nothing calls. The lesson written
down at the time, and repeated in `CLAUDE.md`:

> An auditor restricted to *reading* files cannot tell what is hot, what is
> reachable, or what is worth doing. Give a reviewing model the ability to run
> the benchmarks and the test suite, or treat its output as hypotheses to
> measure — never as a plan to execute.

This trial is that lesson applied deliberately. The external model got no tools,
no repository access and no ability to run anything. It got a problem statement,
the real code, the real constraints and the real baseline numbers — and every
claim it made was then measured here.

## 2. Why this target

The chosen problem is **`d_ln_phi_d_n`** in
[`engine/src/mixture.rs`](../../../engine/src/mixture.rs) — the composition
Jacobian `∂ln φ̂ᵢ/∂nⱼ`.

Classical and IVDW mixing with a 2-parameter EOS take a hand-derived analytic
closed form. Everything else — Wong-Sandler, Huron-Vidal, MHV1, MHV2, and every
3-parameter EOS — falls back to dual numbers, **one full `Dual64` evaluation per
Jacobian column**. Each sweep re-runs the entire mixture evaluation to extract a
single column.

[`OPTIMIZATION_PLAN_PART2.md`](OPTIMIZATION_PLAN_PART2.md) §6 calls this "the
largest remaining algorithmic win" *and* "the highest accuracy risk", and
deferred it twice. That combination is what makes it the right trial subject:

| criterion | why this target qualifies |
|---|---|
| **Valuable** | The plan's own ranking puts it first among what remains |
| **Self-contained** | One function plus its generic callees; no repo-wide reachability judgement needed |
| **Deferred on risk, not on doubt** | The destination was never in question, only the route |
| **Gradeable** | Three independent oracles already exist (below) |

The last row is the decisive one. A recommendation that cannot be falsified is
not worth soliciting.

### The three oracles

1. **Correctness** — `dual_jacobian_matches_fd_for_exotic_rules` and
   `dual_jacobian_matches_fd_for_3param_eos` compare against a central-difference
   oracle at 1e-4; `classical_analytic_jacobian_matches_fd_and_is_symmetric`
   compares the analytic path at 1e-5.
2. **Invariant** — the Jacobian must be symmetric to ~1e-9 relative
   (a Gibbs-Duhem consequence), asserted in the same test.
3. **Speed** — criterion bench `mixture/d_ln_phi_d_n_wong_sandler_n4`.

## 3. The model

Selected after checking OpenAI's current model documentation rather than from
memory — the lineup had moved twice since the assistant's knowledge cutoff, and
the name that would have been guessed (`gpt-5.5`, or a `-codex` variant) was not
the right answer.

| | |
|---|---|
| Model | **`gpt-5.6-sol`** — the flagship tier of the GPT-5.6 generation |
| Why this one | GPT-5.6 replaced numbered variants with named capability tiers: **Sol** (flagship, positioned for "complex coding and long sessions"), **Terra** (~½ the price, ≈ GPT-5.5 performance), **Luna** (fastest/cheapest). Sol is the tier for hard problems |
| Not `-codex` | The newest codex-branded model available on this key is `gpt-5.3-codex` (2026-02-08) — two generations behind Sol |
| Reasoning effort | `xhigh` (the spectrum is `none / low / medium / high / xhigh / max`) |
| Context | 1.05 M tokens, 128 K max output |
| Price | $5 / M input, $30 / M output |

## 4. Method

Everything is reproducible from the repo. The API key is read from 1Password at
call time and never touches disk.

```sh
# 1. Build the prompt from verbatim source line ranges (never hand-copied).
~/miniconda3/envs/vle/bin/python scripts/build_second_opinion_prompt.py

# 2. Ask. Stdlib urllib only — installs nothing into the `vle` env.
~/miniconda3/envs/vle/bin/python scripts/second_opinion.py \
    docs/plans/engine/second-opinion/dualn-prompt.md \
    --model gpt-5.6-sol --effort xhigh \
    --out docs/plans/engine/second-opinion/dualn-response-sol.md

# 3. Grade in an isolated worktree so `main` is never touched.
git worktree add /tmp/wt-dualn HEAD --detach
cd /tmp/wt-dualn
cargo bench --bench engine_bench -- d_ln_phi_d_n     # baseline first
#   ... apply the patch ...
cargo test  --release -p vle-thermo                   # all three oracles
cargo bench --bench engine_bench -- d_ln_phi_d_n     # then the verdict
```

[`scripts/build_second_opinion_prompt.py`](../../../scripts/build_second_opinion_prompt.py)
assembles the prompt from **line ranges of the real sources**, so the prompt can
never drift from the code it asks about. The generated prompt is committed at
[`second-opinion/dualn-prompt.md`](second-opinion/dualn-prompt.md).

The prompt stated six hard constraints (mixing algebra written once; no
`unsafe`; public signature frozen because three binding surfaces depend on it;
the two accuracy tolerances; no heap allocation in inner loops; and a required
statement about behaviour as n grows toward the petroleum track's n ≈ 300). It
deliberately did **not** name any candidate type — finding one was the test.

## 5. What the model proposed

In one sentence: **use `num_dual::DualSVec64<W>`, a statically-sized
multidirectional dual the already-present `num-dual` 0.11.2 dependency provides,
and evaluate the Jacobian in `W`-wide column blocks instead of one column at a
time.**

Two things are worth flagging before the numbers.

**It found the crate-provided type.** `OPTIMIZATION_PLAN_PART2.md` §6 framed the
work as "a hand-written const-generic dual to replace `num-dual`'s per-column
sweeps", and deferred it partly on the accuracy risk of *reimplementing dual
arithmetic*. `DualSVec64<W>` is `Copy` for const `W`, so it satisfies the
existing `D: DualNum<f64> + Copy` bound with **no trait-bound edit and no new
arithmetic to get wrong**. Most of the risk the plan deferred on was avoidable.

**It rejected the premise it was handed.** The prompt asked for "one pass". The
model argued that single-pass does not mean an *n*× win, because propagating an
n-wide tangent through O(n²) mixing algebra is still O(n³):

> ```text
> current:  n(P + E)
> vector:   P + nE
> ```
> It is not `n` unless tangent propagation is free.

It then predicted ~520 ns at n = 4 against a 1285 ns baseline — a 2.5× win, not
the 5× the framing invited — and said so before any measurement existed.

## 6. Results — measured, not reported

All numbers from `cargo bench` on this machine (Apple Silicon, `--release`), in
one session. The baseline was re-measured first and reproduced the figure
recorded in `OPTIMIZATION_PLAN_PART2.md` (1.285 µs) to within 0.3 %.

### Did it compile?

**Not as written — two mechanical errors, both in `num-dual`'s wrapper API.**

| error | fix |
|---|---|
| `Grad::<W>::new(re, eps)` passed a bare `SVector` where a `Derivative` is required | wrap in `num_dual::Derivative::some(eps)` |
| `lnphi[i].eps[lane]` indexed a `Derivative`, which has no `Index` impl | `.eps.unwrap_generic(Const::<W>, U1)[lane]` |

Three edited lines (including one import). The *design* compiled; the model
misremembered the shape of two constructor/accessor calls in a mid-popularity
crate. The as-tested patch, including these fixes, is at
[`second-opinion/dualn-patch-as-tested.diff`](second-opinion/dualn-patch-as-tested.diff).

### Did it stay correct?

**Yes — 194/194 tests pass**, including all three oracles:

```
mixture::tests::classical_analytic_jacobian_matches_fd_and_is_symmetric ... ok
mixture::tests::dual_jacobian_matches_fd_for_exotic_rules ... ok
mixture::tests::dual_jacobian_matches_fd_for_3param_eos ... ok
test result: ok. 192 passed; 0 failed          (unit)
test result: ok. 2 passed; 0 failed            (Chapter IV validation)
```

The model had claimed the propagated derivatives are "mathematically identical",
differing only by "a few ulps from expression ordering", because it seeds the
normalized composition analytically (`dxₖ/dnⱼ = (δₖⱼS − nₖ)/S²`) instead of
letting dual division do it. The FD and symmetry tolerances held. It also
recommended *against* post-hoc symmetrization on the grounds that it would mask
real defects — the correct call.

### Was it faster?

| benchmark | baseline | patched | change |
|---|---:|---:|---|
| `d_ln_phi_d_n_wong_sandler_n4` | 1288.3 ns | **566.3 ns** | **−56.3 % (2.28×)** |
| `d_ln_phi_d_n_classical_n4` | 242.5 ns | 253.2 ns | +4.4 % ⚠ |

The Wong-Sandler win is real and reproducible. **Predicted 520 ns, measured
566 ns — an 8 % error on a number it had no way to test.**

The classical regression is small but reproducible *within a build* (two
independent runs at ~253 ns, `p = 0.85` between them). That path's code is
untouched — it early-returns before the new dispatch — so the suspicion at the
time was code-layout or i-cache fallout from four new monomorphizations.

> **Resolved 2026-08-15, during Milestone 18 — it was layout, and it is not
> specific to this patch.** `d_ln_phi_d_n_classical_n4` measured
> **242 ns → 231 ns → 243 ns** across three consecutive builds whose changes
> touched *other* mixing rules entirely, while two consecutive runs of the
> **same** build agreed to 0.5 % (239.9 ns, 241.2 ns). This benchmark carries
> roughly **±5 % build-to-build variance from code layout** against ~1 %
> run-to-run variance. The 4.4 % "regression" above sits inside that band and
> is not a cost of the patch — and neither was the 4.8 % "improvement"
> Milestone 18 briefly appeared to hand the same benchmark. Both were the same
> artefact. A single-digit-percent delta here is evidence of nothing unless the
> build is otherwise identical.

### The prediction that mattered most

The model was also asked what n = 20 would read. No such benchmark existed, so
one was written for this trial (20 pseudo-alkanes, Wilson/Wong-Sandler). It is
**not committed** — it lives only in the trial worktree.

| | |
|---|---:|
| Model's prediction for n = 20 | **60 000 ns** |
| Measured, patched | **60 955 ns** |
| Measured, baseline | 61 615 ns |

The prediction was accurate to **1.6 %** — and it is a prediction of *almost no
improvement*. The model had explained why in advance: at n = 20 its bucket table
selects `DualSVec64<32>`, so 12 of 32 lanes are wasted, and "this is still
effectively O(n³)".

**So the 2.28× win at n = 4 does not scale.** Anyone who had adopted this patch
on the strength of the headline number, without measuring n = 20, would have
concluded the petroleum-track problem was addressed. It is not.

### Where the model was wrong — and it is the tunable that matters

Its bucket table is the one part that did not survive contact with the
benchmark. Holding the design fixed and varying only the block width `W` at
n = 20:

| W | n = 20 time | vs baseline |
|---:|---:|---|
| 1 *(baseline, per-column)* | 61.6 µs | — |
| 4 | 23.0 µs | 2.7× |
| 8 | **17.1 µs** | **3.6×** |
| 16 | 26.6 µs | 2.3× |
| 32 *(the model's choice)* | 61.0 µs | 1.0× |

`W = 8` is **3.6× faster than the width the model picked** for this bucket — and
the model itself already used `W = 8` for its n > 32 fallback, with the right
reasoning attached ("maps well to AVX2", bounded working set). It simply did not
apply that reasoning below its own threshold. Wide lanes are actively harmful
well before n = 32.

This is the audit lesson reproduced exactly. The *design* was sound and could be
reasoned out from source. The *constant* could not, and it is worth 3.6×.

## 7. Verdict

| question | answer |
|---|---|
| Was the second opinion worth soliciting? | **Yes.** It found a crate-provided type that removes most of the risk §6 was deferred on, and it corrected the framing of the expected win |
| Was it usable as written? | **No.** Two compile errors, and a bucket table that costs 3.6× at n = 20 |
| Does it change `OPTIMIZATION_PLAN_PART2.md` §6? | **Yes** — see below |
| Is the petroleum-track blocker solved? | **No**, and the model said so itself. Complexity stays O(n³) |
| Cost | **$0.95** (8 257 input, 30 331 output tokens, of which 27 452 reasoning) |

### What this changes in the plan of record

§6 of [`OPTIMIZATION_PLAN_PART2.md`](OPTIMIZATION_PLAN_PART2.md) is deferred on
the premise that it "means reimplementing dual arithmetic with accuracy risk on
the exact path M12.3's Gibbs–Helmholtz invariants guard." **That premise is now
measured to be wrong**: `DualSVec64<W>` is supplied by a dependency already in
the tree, needs no new arithmetic, needs no trait-bound change, and passed all
194 tests unmodified on the first correct compile. The remaining work is
bucket-width tuning and a benchmark set that covers n > 8 — ordinary work, not a
milestone-sized accuracy risk.

That is a change to a *reason for deferral*, not a completion claim. **The
optimization is still not shipped.** `d_ln_phi_d_n` on `main` is byte-for-byte
what it was.

### Before any of this could be adopted

1. Choose block widths **by measurement at each bucket**, not by reasoning.
   n = 4 and n = 20 are measured here; 8, 16, 32 and ≥ 100 are not.
2. Commit the n = 20 benchmark (and probably n = 8 and n = 50) so the width
   table has a permanent guard.
3. ~~Explain or accept the +4.4 % classical regression~~ — **done, 2026-08-15**:
   it is code-layout variance, not a cost of the patch. See the resolution note
   in §6 above. The bench needs an identical build to be read at this
   resolution.
4. Verify through the PyO3 surface (`mixture_d_ln_phi_d_n`) and the M12.3
   Gibbs–Helmholtz invariants, neither of which this trial exercised.
5. Check compile time and wasm binary size against the added monomorphizations.

## 8. Would this be worth repeating?

On this evidence, yes, under three conditions:

- **The target is deferred on risk, not on unknown value.** A model that cannot
  profile cannot tell you what to work on. It can tell you how to do a thing you
  have already decided is worth doing.
- **Every oracle exists before the question is asked.** The grading here took
  longer than the asking, and that ratio is the healthy one.
- **The answer is measured, never merged on read-through.** Two of this
  answer's six deliverables were wrong in ways that read perfectly on the page:
  a constructor signature, and a constant worth 3.6×.

The cost of the opinion was $0.95. The cost of believing it without the
benchmark would have been a patch that looks like a 2.28× win and delivers 1.0×
on the workload the roadmap actually cares about.

---

## References

- [`second-opinion/dualn-prompt.md`](second-opinion/dualn-prompt.md) — the exact prompt sent
- [`second-opinion/dualn-response-sol.md`](second-opinion/dualn-response-sol.md) — the verbatim answer
- [`second-opinion/dualn-patch-as-tested.diff`](second-opinion/dualn-patch-as-tested.diff) — the patch as compiled and benchmarked
- [`OPTIMIZATION_PLAN_PART2.md`](OPTIMIZATION_PLAN_PART2.md) §6 — the deferral this tests
- [`OPTIMIZATION_AUDIT_HISTORY.md`](OPTIMIZATION_AUDIT_HISTORY.md) — the earlier external audit and its lessons
- [`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](PETROLEUM_PSEUDOCOMPONENT_PLAN.md) — the n ≈ 300 track this does *not* unblock
