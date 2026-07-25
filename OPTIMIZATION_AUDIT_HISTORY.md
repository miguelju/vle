# How the Performance Audit Happened — Provenance and Lessons

A record of where [`optimizations_audit.md`](optimizations_audit.md) came from, what it
got right, what it got wrong, and what the whole exercise teaches about using
one AI to review another's plan.

This repo is a **learning repo**. The audit is a good artifact and a genuinely
useful one — but taking it at face value would have made this engine *slower* in
two places. That gap is the most instructive thing in this document, so it is
written up in full rather than quietly fixed.

---

## 1. The chain of custody

Four models, three roles, in order.

### Step 1 — Gemini Pro wrote the prompt

Miguel asked **Gemini Pro** to author a prompt that would get a rigorous Rust
performance audit out of the Codex CLI. Gemini produced this, verbatim:

```
codex exec -o optimizations_audit.md "Strictly limit your analysis to the following
Rust files: $(find . -name '*.rs' | tr '\n' ' '). Ignore all non-Rust files in this
workspace.

I am building a high-performance thermodynamic calculation library in Rust. A previous
AI generated a modernization plan titled 'numpy for thermo'. The accepted plan includes:
- Track A: Modern flash algorithms (Halley's method for Rachford-Rice inside the
  Leibovici-Neoschil window, GDEM-accelerated successive substitution into full Newton
  on ln(K), phase envelope continuation).
- Track B: Generalized mixture core with exact derivatives (analytic for classical,
  dual-number AD via 'num-dual' for exotic rules).
- Track C: Zero-allocation hot paths, caching via a per-point EosState struct, and
  SmallVec/stack arrays for component bounds.
- Track D: Rayon parallelism for batch processing and zero-copy FFI via rust-numpy
  with GIL release.

I need you to act as an elite Rust performance engineer and perform a ruthless
architectural audit to find what this plan missed. I want to squeeze the absolute
maximum execution speed out of this engine.

Please output a comprehensive Markdown document divided into two parts:

1. Flash Calculation Deep-Dive: Analyze the multicomponent flash calculation
   implementation (Rachford-Rice root finding, phase stability testing, Newton/GDEM
   iterations). Given the proposed modern algorithms, what are the hidden bottlenecks?
   Identify areas where we can improve CPU cache line utilization, apply SIMD
   (vectorization) to the K-value or fugacity arrays inside the tightest loops, or
   eliminate hidden allocations. Are there numerical edge cases where the Halley/Newton
   approach degrades that we haven't protected against?

2. Global Package Optimization: Analyze the remaining thermodynamic calculations, the
   EosState caching strategy, and the matrix core. Identify suboptimal memory layouts
   (e.g. Array of Structs vs Struct of Arrays slowing down Rayon threads), locking
   overhead, or excessive heap usage. If 'nalgebra' or 'num-dual' introduces overhead
   for small n-component mixtures, propose faster stack-based alternatives.

Provide concrete, highly optimized Rust code snippets for every proposed enhancement."
```

### Step 2 — Codex CLI (gpt-5.6-sol) wrote the audit

Run through `codex exec`, output to `optimizations_audit.md`. It was **fast** —
which is the thread this document pulls on.

### Step 3 — Claude Opus 5 benchmarked and second-audited it

Recorded in [`OPTIMIZATION_PLAN_PART1.md`](OPTIMIZATION_PLAN_PART1.md) and
[`OPTIMIZATION_PLAN_PART2.md`](OPTIMIZATION_PLAN_PART2.md): build a measured
baseline, evaluate each recommendation against *this* codebase, execute the ones
that survive, and quantify the result. Both parts produced at least one
recommendation that benchmarked worse than the code it replaced.

### Step 4 — Miguel arbitrated

The lead-architect role. For Part 1, approval was explicitly gated on the
execution plan before any file was modified, and two structural questions were
settled by the owner rather than the model: whether `FlashWorkspace` should be
public API (answer: no, private until the batch path needs it), and whether the
four deferrals stood (answer: yes). Part 2 was delegated with "implement what
makes sense" — which is a different and more demanding instruction, because it
moves the reject/defer judgment onto the model and makes the written record of
*why* each item was rejected the only audit trail.

---

## 2. "Did Codex miss that Part 2 is where the improvement lives?"

**Miguel's read was right, but the failure is subtler than "it missed it."**

Codex did *not* miss it. Read the audit's own opening:

> *Rachford–Rice is not the bottleneck. Repeated state construction, redundant
> EOS evaluations, pointer-chasing matrices, transient vectors, scalar
> transcendental work, and finite-difference Jacobians dominate execution time.*
>
> *The highest-value change is to make every calculation operate through a
> caller-owned, reusable workspace and a prepared immutable model.*

And §14's payoff order leads with `PreparedSystem` / `TpCache` — Part 2 §1. It
closes with: *"The first six items should deliver substantially more speed than
substituting Halley for Newton in Rachford–Rice."*

So the conclusion is stated, twice, correctly. **What went wrong is the
structure, and it is a real failure with real consequences.**

### Failure 1 — it accepted the prompt's framing over its own finding

Gemini's prompt asked for "Part 1: Flash Deep-Dive" then "Part 2: Global
Optimization" — a *scope* split. Codex found that the value distribution runs
almost perpendicular to that split, said so in the summary, and then filled the
sections in exactly as asked anyway. The result reads as a priority ordering it
is not. An audit whose headline is "you are looking at the wrong layer" should
restructure around that finding, or at minimum put a "read §14 first" banner at
the top of Part 1.

The practical consequence: a reader executing top-to-bottom — which is the
natural way to consume a numbered plan — spends their first pass on the layer
holding 17–21 % of the time, and puts a `SmallVec` inline-width debate ahead of
the 70 %. That is precisely what almost happened here.

### Failure 2 — no measurements, therefore no calibration

This is the deeper problem, and it is what "it did it super fast" bought.

The audit contains **zero numbers**. Not one measured cost, not one profile, not
one estimate of what any change is worth. Every recommendation carries equal
rhetorical weight, so a 45 % win (`min_gibbs_ln_phi` evaluating the entire
mixture path twice) sits beside a 200 % *regression* (precomputing `cᵢ = Kᵢ − 1`)
in the same numbered list, in the same confident register.

The measured reality:

| Audit item | Audit's framing | Measured |
|---|---|---|
| P1 §8 min-Gibbs shares one mixture state | one bullet | **−36…−46 %** on stability |
| P2 §1 per-`(T,P)` cache | one of fourteen sections | **−15 %** on the flash, on top of Part 1 |
| P2 §5 activity matrix caching | one of fourteen sections | **−68 %** on NRTL γ |
| P1 §2 keep K in log form | one section | **−16 %** on the γ-φ K-value path |
| P2 §9 flatten the virial matrix | one of fourteen sections | **−21 %** |
| P1 §1 workspace / `*_into` | the lead recommendation | part of a **−12…−15 %** flash |
| P2 §4 precompute `sqrt_ai` | stated as obvious | **≈0 %** until an unrelated `&dyn Fn` was removed |
| P1 §3 precompute `cᵢ`, `zᵢcᵢ` | "Precompute `cᵢ`" — stated as obvious | **+30…+200 % regression** |
| P1 §3 probe f(0), f(1) first | recommended for the hot path | **+25…+200 % regression** |
| P1 §4 SIMD | a full section with code | rejected; wrong `n` regime entirely |
| P2 §8 stop inverting the Broyden Jacobian | a full section with code | rejected; **nothing calls Broyden** |

Two of eight concrete Part 1 proposals are net regressions on this engine.
Neither is *wrong in principle* — hoisting a loop invariant and narrowing a
bracket are both textbook-correct instincts. They fail here for reasons only a
benchmark surfaces:

- The `cᵢ` hoist saves a subtract and a multiply per component per evaluation —
  cheap ALU work that already hides under the division's latency the loop is
  actually limited by — while the preparation pass costs more than it saves for
  a solve converging in a handful of iterations. It also converts two
  contiguous arrays into an array-of-structs, striding the loads. The audit
  warns about that exact layout problem in Part 2 §3, three sections later.
- The f(0)/f(1) probe assumes the Leibovici–Neoschil window is meaningfully
  wider than [0, 1]. For a real mixture the poles are set by the most and least
  volatile components and land *just* outside — [−0.038, 1.012] for the
  benchmark binary. The narrowing saves no iterations; the probes cost two extra
  divisions per component.

The honest summary: **the audit is a strong hypothesis generator and a poor
prioritizer.** It reads the code well and knows the literature. It cannot tell
you what anything costs, and it did not say so.

### Failure 3 — it was not asked for evidence, and did not volunteer it

Worth pinning on the prompt, not the model. Gemini's prompt says "ruthless
architectural audit", "squeeze the absolute maximum execution speed", and
"provide concrete, highly optimized Rust code snippets for every proposed
enhancement". Every one of those pushes toward *volume of confident
recommendations*. None asks "what does this cost today", "what is your
confidence", or "which of these would you not do".

**The prompt got the audit it asked for.** A better one would have said: *before
recommending anything, tell me what a benchmark would need to show for each
recommendation to be worth it, and rank by expected payoff with your confidence
in each.*

### Part 2 repeated the pattern — including one miss that mattered

Part 2 confirmed both halves of the diagnosis. The audit's §1 (cache at the
right granularity), §5 (activity matrices) and §9 (virial) were all real and all
paid — §5 alone is −68 % on NRTL. But §4, "classical mixing unnecessarily
recomputes square roots", was again *true and not the bottleneck*: the hoist it
recommends changed almost nothing, because the same `n²` loop invoked its
cross-parameter closure through a **`&dyn Fn` trait object**. Every one of those
`n²` calls was an indirect call LLVM could not inline through, and it dominated
the arithmetic the audit wanted to remove. Monomorphizing the loop is what paid.

That miss is the sharper version of Failure 2. The audit *has* a section on
exactly this idea — §12, "fast-path dispatch must happen outside inner loops" —
but aimed it at introducing **new** function-pointer indirection into the
fugacity kernels (which would have made things slower), while a genuine vtable
call sat `n²`-deep in the mixing rule it had reviewed three sections earlier.
Reading for patterns finds the pattern; only measurement finds *where* it bites.

Part 2 also produced the audit's one clearly **factual** error: §8 devotes a
section and a code sketch to fixing `numerics/broyden.rs`, which has **no
production callers** — `grep` finds only a PyO3 utility binding. No flash,
bubble/dew, envelope or critical path uses it. Reviewing a file is not the same
as knowing whether anything runs it.

### Where Codex was clearly right

Credit where due — the audit's core diagnosis survived measurement intact:

- **RR is not the bottleneck.** Measured at 1–15 % of the flash. Correct, and it
  is the single most important thing in the document.
- **`min_gibbs_ln_phi` doubling work.** Called out precisely; fixing it was the
  largest single win in Part 1 (−36…−46 % on the stability path).
- **GDEM had no trust region.** Real, and `μ → 1` genuinely diverges.
- **The Newton finish is not implemented.** Correct — and it caught a
  documentation inconsistency this repo did not know it had (below).
- **`k_values` exponentiating terms only to divide them** was real, and fixing it
  gave −16 % on the γ-φ path.
- **The `SmallVec<[D; 8]>`-for-duals critique** (Part 2 §7) is sharp and, as far
  as anything measured so far shows, correct.
- **`EosState` caches the wrong granularity** (Part 2 §1) — the single most
  valuable structural observation in the document, and the reason the flash is
  now ~28 % faster than where it started.
- **NRTL and Wilson rebuilding invariant matrices** (Part 2 §5) and **the virial
  rebuilding a fragmented `Vec<Vec<f64>>`** (§9) — both precise, both paid.

---

## 3. The Newton-finish documentation bug the audit surfaced

Part 1 §6 says the module docs admit the Newton polish is future work. Checking
what this repo *claimed* found an internal contradiction:

- `ROADMAP.md` Milestone 9 checkbox: *"(analytic-Jacobian Newton polish is a
  follow-on refinement)"* — accurate, and always has been.
- `MODERNIZATION_PLAN.md` Milestone 9 progress note: *"Milestone 9 is complete:
  every Phase 15 algorithm is implemented"*, with the shipped list reading
  *"GDEM-accelerated successive substitution → Newton on ln Kᵢ with analytic
  Jacobian (§J)"* — **not** accurate.

Miguel caught the same thing independently ("it is my understanding that
MODERNIZATION_PLAN.md is done, can you please check"). Both
`MODERNIZATION_PLAN.md` lines are now corrected and §J carries an explicit
status paragraph.

The lesson generalizes past this repo: **a milestone marked complete in a plan
document is a claim, not a fact.** The code is the fact. An outside auditor with
no stake in the plan found the gap in one pass.

---

## 4. What this says about AI-reviewing-AI

Five things worth carrying forward.

**1. A second auditor is worth it — and must be able to run the code.** The
value here did not come from Claude disagreeing with Codex on style. It came
from `cargo bench`. Two recommendations were reverted *only* because a number
contradicted them. A reviewer restricted to reading, as Codex was by its own
prompt ("strictly limit your analysis to the following Rust files"), cannot
produce that.

**2. Speed of generation is not free.** "It did it super fast" and "it missed
that Part 2 matters more" are the same observation. Structuring a document
around its own conclusion, and calibrating confidence per item, are the parts
that take time. They are also the parts a reader most depends on.

**3. Prompt framing propagates all the way through.** Gemini's Part 1 / Part 2
split was a reasonable way to scope an audit. It became the document's implied
priority order, which became the execution order, which nearly put the 70 % last.
The scoping decision made before any analysis existed shaped the outcome more
than the analysis did.

**4. Every rejected optimization needs its reasoning preserved *in the code*.**
The reverted items are documented at the call site — `rr_fdd`, `rr_solve` and
`quad_a` each carry a comment naming the audit section, the reasoning, and the
measured number. Without that, the next reader (human or model) re-proposes
them, because they still look obviously right. A rejected optimization with a
number attached is more valuable than a silent omission.

**6. "Is this code even reachable?" is a question a reading-only auditor cannot
answer.** Part 2 §8 spends a section and a code sketch on `numerics/broyden.rs`.
Nothing in the engine calls it. One `grep` for callers would have redirected
that effort — but the prompt scoped the auditor to *analyzing files*, and a file
looks equally important whether it runs a million times a second or never.

**5. Keep the human as arbiter on structural calls.** Two decisions were
genuinely the owner's: whether `FlashWorkspace` enters the published API of a
crate on crates.io, and whether four substantial deferrals stood. Neither is a
technical question with a right answer — both are judgments about scope and
API commitment. The pause-for-approval gate is what made them Miguel's.

---

## 5. Timeline

| When | What |
|---|---|
| 2026-07 | `PERFORMANCE_PROPOSAL.md` Tracks A–E accepted (the plan under audit) |
| 2026-07-25 | Gemini Pro authors the audit prompt |
| 2026-07-25 | Codex CLI (gpt-5.6-sol) produces `optimizations_audit.md` |
| 2026-07-25 | Claude Opus 5 extends the criterion suite with `flash_multi`; baseline captured |
| 2026-07-25 | Second-auditor evaluation; execution plan presented; Miguel approves |
| 2026-07-25 | Part 1 executed; two audit recommendations benchmarked and reverted |
| 2026-07-25 | `MODERNIZATION_PLAN.md` Newton-finish claim corrected |
| 2026-07-25 | Part 2 baseline: `mixture_params` measured at 40–57 % of `ln_phi_mix` |
| 2026-07-25 | Part 2 high-value subset executed; `&dyn Fn`-in-the-n²-loop found and fixed; §8 found to target dead code |

---

## References

- [`optimizations_audit.md`](optimizations_audit.md) — the audit itself, unmodified
- [`OPTIMIZATION_PLAN_PART1.md`](OPTIMIZATION_PLAN_PART1.md) — the flash layer: verdicts and numbers
- [`OPTIMIZATION_PLAN_PART2.md`](OPTIMIZATION_PLAN_PART2.md) — the mixture core: verdicts and numbers
- [`PERFORMANCE_PROPOSAL.md`](PERFORMANCE_PROPOSAL.md) — Tracks A–E, the plan the audit critiques
- [`MODERNIZATION_PLAN.md`](MODERNIZATION_PLAN.md) — §F/§I/§J, and the corrected Milestone 9 status
