# Plan & Audit History

Every planning and audit document this project has produced, what it decided,
and whether the code ever caught up with it.

This folder exists because of a rule in [`CLAUDE.md`](../../CLAUDE.md):
**a plan document is a claim; the code is the fact.** Plans here are *not*
deleted once executed. A shipped plan becomes a **design record** — the
alternatives that were considered, the reason one was chosen, and (just as
importantly) the optimizations that were tried and **rejected because they
measured slower**. Those rejection entries are load-bearing: they are what
stops the next reader, human or model, from re-proposing a change that was
already tried and found worse.

The split is by subject:

- **[`engine/`](engine/)** — the calculations. Thermodynamics, numerics,
  performance.
- **[`delivery/`](delivery/)** — how the engine reaches other languages and
  platforms.

One document here is **live**, not a record: `MODERNIZATION_PLAN.md` is the
27-phase implementation plan, still the technical half of the
ROADMAP → TODO → PLAN trio described in `CLAUDE.md`'s synchronization rules.

---

## Engine & calculations

| Document | Adopted | Kind | Status | Milestone |
|---|---|---|---|---|
| [`MODERNIZATION_PLAN.md`](MODERNIZATION_PLAN.md) | 2026-04-04 | **live plan** | Active — 27 phases; started at 17, grown 10 times since | all |
| [`PERFORMANCE_PROPOSAL.md`](engine/PERFORMANCE_PROPOSAL.md) | 2026-07-01 | plan | Accepted, folded into the trio. Tracks C+E → M8.2, B → M8.3, A → M9, D → M10 | 8.2 – 10 |
| [`DERIVATIVE_RELEASE_PLAN.md`](engine/DERIVATIVE_RELEASE_PLAN.md) | 2026-07-05 | plan | **Shipped** — all five downstream gaps closed (v0.8.2, v0.9.0, + patch v0.9.1) | 12 |
| [`STEAM_TABLES_PLAN.md`](engine/STEAM_TABLES_PLAN.md) | 2026-07-07 | plan | **Shipped** in v0.10.0; extended by M13.7 – 13.8 in **v0.13.0** (transport properties + the IF97 performance pass). The 13.7 milestone notebook remains open | 13 |
| [`NRTL_AMMONIA_PLAN.md`](engine/NRTL_AMMONIA_PLAN.md) | 2026-07-07 | plan | **Shipped** in v0.11.0. §8 (van Laar vs NRTL comparison) is a *proposed* follow-up, not scheduled | 14 |
| [`optimizations_audit.md`](engine/optimizations_audit.md) | 2026-07-24 | **audit** (external) | Delivered and answered in full; kept verbatim, never edited | — |
| [`OPTIMIZATION_PLAN_PART1.md`](engine/OPTIMIZATION_PLAN_PART1.md) | 2026-07-25 | plan | **Complete** — the flash layer's measured response to Part 1 | → v0.12.0 |
| [`OPTIMIZATION_PLAN_PART2.md`](engine/OPTIMIZATION_PLAN_PART2.md) | 2026-07-25 | plan | **High-value subset complete** — the mixture core's response to Part 2 | → v0.12.0 |
| [`OPTIMIZATION_AUDIT_HISTORY.md`](engine/OPTIMIZATION_AUDIT_HISTORY.md) | 2026-07-25 | provenance | Living record of how the audit was produced and reviewed | — |
| [`steam_audit.md`](engine/steam_audit.md) | 2026-07-25 | **audit** (external) | Delivered; two items adopted directly, the rest superseded by measuring the layer underneath | 13.8 |
| [`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md) | 2026-07-25 | plan | **Proposed — not started.** Nothing in it is implemented | 18 – 20 |
| [`SECOND_OPINION_TRIAL.md`](engine/SECOND_OPINION_TRIAL.md) | 2026-08-15 | **experiment** (external model) | **Run and graded.** Nothing merged — the engine is unchanged. Revises the *reason* `OPTIMIZATION_PLAN_PART2.md` §6 is deferred | — |

## Delivery & platforms

| Document | Adopted | Kind | Status | Milestone |
|---|---|---|---|---|
| [`IOS_FFI_PLAN.md`](delivery/IOS_FFI_PLAN.md) | 2026-07-07 | plan | **Implemented** 2026-07-11. Local-build artifact — never published, never in CI | 15 |
| [`ANDROID_FFI_PLAN.md`](delivery/ANDROID_FFI_PLAN.md) | 2026-07-12 | plan | **Code complete**; first Android Studio run still pending | 16 |
| [`WEB_UI_PLAN.md`](delivery/WEB_UI_PLAN.md) | 2026-07-12 | plan | **Complete** | 17 |

## Still in the repository root

| Document | Why it isn't here |
|---|---|
| [`ROADMAP.md`](../../ROADMAP.md) | Live milestone tracker. Read this first to learn what is done. |
| [`TODO.md`](../../TODO.md) | Live task list with time estimates, one section per milestone. |
| [`PASCAL_VB6_COMPARISON.md`](../../PASCAL_VB6_COMPARISON.md) | Legacy-code *analysis*, not a plan or an audit — the study of the two original programs that everything else was planned from. |
| [`PUBLISHING.md`](../../PUBLISHING.md) | Operating procedure for cutting a release, not a plan. |

Distribution is documented separately: [`deploy/README.md`](../../deploy/README.md)
for the two registries (PyPI, crates.io), and
[`distribution/README.md`](../../distribution/README.md) for every other channel.

---

## How the plans arrived — five eras

### 1. Origins — April 2026

Two legacy programs, no plan. The first work was reading:
~15,000 lines of VB6 from the 1999 thesis and ~2,500 lines of Mac Pascal from
1989. That study became [`PASCAL_VB6_COMPARISON.md`](../../PASCAL_VB6_COMPARISON.md),
and the merge strategy it produced became `MODERNIZATION_PLAN.md` — originally
**17 phases**, now 27.

The same analysis produced the algorithm-improvement table still shown in the
root [`README.md`](../../README.md): Broyden instead of a full numerical
Jacobian, Brent instead of golden section, Halley instead of Newton for
Rachford–Rice. It was a good table. It was also written before a single
benchmark existed, and several of its rows have since been superseded — which
is the whole reason this history document exists.

### 2. The performance turn — 1–5 July 2026

[`PERFORMANCE_PROPOSAL.md`](engine/PERFORMANCE_PROPOSAL.md) reframed the project
around one goal: **"numpy for thermo"** — arrays of states, not one state at a
time. It settled the language question (the engine stays in Rust), and made one
decision that reaches into every module written since: **composition derivatives
are exact — analytic where a closed form exists, `num-dual` dual-number AD where
it doesn't — and finite differences survive only as test oracles.** It also added
academic references (23)–(29) and grew the improvement plan from §A–§H to §A–§M.

Four days later, [`DERIVATIVE_RELEASE_PLAN.md`](engine/DERIVATIVE_RELEASE_PLAN.md)
arrived from an unusual direction: **a downstream consumer**. `stages-thermo`, a
staged-separation library planned on top of this one, audited its own API needs
against `vle-thermo` 0.8.1 and found five gaps. This is the first plan in the
repo written to close gaps somebody else identified — and all five closed.

### 3. Capability plans — 7–8 July 2026

Two plans in one day, both shipping within a week.
[`STEAM_TABLES_PLAN.md`](engine/STEAM_TABLES_PLAN.md) added IAPWS-IF97 water
properties as a **dependency-free sibling crate** rather than bending the mixture
engine around a single substance — "VLE for water only", usable standalone.
[`NRTL_AMMONIA_PLAN.md`](engine/NRTL_AMMONIA_PLAN.md) was again downstream-driven:
`stages-thermo`'s Ponchon–Savarit work needed a proper heat-of-mixing model and
the ammonia component, so NRTL was chosen over UNIQUAC and a Helmholtz EOS, with
the reasoning recorded rather than assumed.

### 4. Reaching other languages — 7–12 July 2026

Three plans, one philosophy. [`IOS_FFI_PLAN.md`](delivery/IOS_FFI_PLAN.md) built
the `ffi/` UniFFI wrapper crate and a Swift package;
[`ANDROID_FFI_PLAN.md`](delivery/ANDROID_FFI_PLAN.md) reused the *same* wrapper
for Kotlin, covering Android and — via Compose Multiplatform — Windows desktop;
[`WEB_UI_PLAN.md`](delivery/WEB_UI_PLAN.md) went to WebAssembly through
wasm-bindgen instead, because UniFFI has no JavaScript backend at the pinned
version.

The shared rule across all three: **the repo distributes the recipe, not the
artifact.** Nothing is published, nothing binary is committed, none of it ever
runs in CI. C#/.NET was evaluated and deliberately parked — the community
bindings generator lags the pinned UniFFI version.

The iOS plan also carries a small lesson in its own header: it was drafted
assuming it would be "Milestone 14", and had to be renumbered to 15 when NRTL
landed first. Plans predict; the roadmap records.

### 5. The audit era — 24–25 July 2026

The most instructive week in the project, and the reason two of this repo's
standing rules exist.

An **external performance audit** was commissioned on the flash layer. Its chain
of custody is worth stating precisely, because it is the subject of
[`OPTIMIZATION_AUDIT_HISTORY.md`](engine/OPTIMIZATION_AUDIT_HISTORY.md):

> **Gemini Pro** wrote the prompt → **Codex CLI (gpt-5.6-sol)** wrote
> [`optimizations_audit.md`](engine/optimizations_audit.md) → **Claude Opus 5**
> benchmarked every recommendation and second-audited it → **Miguel** arbitrated
> the disagreements.

The audit was genuinely good. It also would have made this engine **slower** in
two places: two textbook-correct recommendations were implemented, measured,
found to regress, and reverted — with the regression recorded at the call site so
nobody re-proposes them. Only `cargo bench` could tell which recommendations were
which. That is the origin of the rule that **a performance claim needs a
measurement, not an argument.**

The audit also surfaced something no benchmark could: `MODERNIZATION_PLAN.md` had
asserted for two milestones that Milestone 9 shipped a Newton finish on ln K with
an analytic Jacobian. It never did. The module's own docs said so, and the
`ROADMAP.md` checkbox said so — but the plan's *summary* claimed otherwise, and
that is the line a reader trusts. It took an outside reviewer with no stake in
the plan to notice. That is the origin of **"code is the fact"**.

The responses split by layer:
[`OPTIMIZATION_PLAN_PART1.md`](engine/OPTIMIZATION_PLAN_PART1.md) (flash) and
[`OPTIMIZATION_PLAN_PART2.md`](engine/OPTIMIZATION_PLAN_PART2.md) (mixture core),
the latter covering the ~70 % of a flash that Part 1 proved was K-value
evaluation — plus one finding the audit missed entirely.

The same week, a second external audit,
[`steam_audit.md`](engine/steam_audit.md), hit the IF97 crate. Two of its items
were adopted directly; measuring the layer *underneath* them then found far more,
including a **latent correctness bug** (a region-3 density bound of 1000 kg/m³
where the IF97 fit is not valid). The headline: the `inverse/ph_vapor` path went
from 20.038 µs to 1.9679 µs — **10.18× faster**.

### The frontier — where the plans point now

[`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md)
(2026-07-25) is the only *unstarted* plan in this folder. It targets an
**atmospheric crude distillation column with hundreds of pseudocomponents** —
the workload that stresses everything built so far — as Milestones 18–20. Its
checkboxes in `ROADMAP.md` and `TODO.md` are all unchecked, and per this repo's
rules they stay that way until the code exists and a test asserts its behaviour.

---

## How to read a status in this folder

| Wording | What it means |
|---|---|
| **Shipped / Implemented / Complete** | The code exists, a test asserts it, and a release or milestone entry names it. Verifiable from `ROADMAP.md`. |
| **Code complete** | Written and tested, but one real-world validation step is still outstanding (M16's first Android Studio run). |
| **Accepted / Adopted** | The decision is binding on later work, but the document is rationale — the executable plan lives in the ROADMAP/TODO/PLAN trio. |
| **Proposed / not started** | Nothing is implemented. Treat every claim as intent. |
| **Audit** | Written by an *external* model, kept verbatim. Its recommendations are hypotheses to be measured, never a plan to execute. |

That last row is the hard-won one. On this project, four of one audit's
recommendations were textbook-correct and either regressed the engine or targeted
code nothing calls. An auditor that can only *read* files cannot know what is
hot, what is reachable, or what is worth doing. Give a reviewing model the
benchmarks and the test suite, or treat its output as hypotheses.

## Corrections made while writing this document

Building these tables meant checking every status line against `ROADMAP.md` and
the code. Two were wrong — both **under**-claiming, both left standing after the
work shipped:

- **`STEAM_TABLES_PLAN.md`** read *"PROPOSED — not yet scheduled … Nothing in
  this file has been implemented yet."* Milestone 13 shipped as **v0.10.0** on
  2026-07-07, and the `steam/` crate implements regions 1–5 with backward
  equations.
- **`DERIVATIVE_RELEASE_PLAN.md`** read *"This document is a plan only — nothing
  in it has been implemented yet."* All five gaps closed in **v0.8.2** and
  **v0.9.0**, with a **v0.9.1** patch.

Both headers now state what the code actually does, and both carry a dated
correction note. Recorded here rather than fixed quietly, because a stale status
line is the exact failure mode this folder is organized to prevent — it just
happens to be the harmless direction of it.
