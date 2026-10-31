# Optimization Next Steps — sequencing the open items

*Drafted 2026-10-30 by Claude Code using Claude Opus 5.5, after v0.18.0.*

**Status: in progress.** Step 0 is done (Part 1 §12) and so are the
correctness findings it raised (Part 1 §13), both unreleased. Steps 1–3 are
not started. **No release until every step is done** (owner's decision,
2026-10-31): see *Unreleased changes* at the end, which is the draft of the
release notes. This is an ordering of the
optimization items that
[`OPTIMIZATION_PLAN_PART1.md`](OPTIMIZATION_PLAN_PART1.md) (flash layer),
[`OPTIMIZATION_PLAN_PART2.md`](OPTIMIZATION_PLAN_PART2.md) (mixture core) and
`ROADMAP.md` Milestone 9.1 left open. It adds no new recommendations. Every
item points back to the section that records the evidence for it. When a step
ships, record the measurement in that Part 1 / Part 2 section, not here. Update
this file's status lines so it never claims more than the code does.

---

## Inventory: what is open

| Item | Source | Kind |
|---|---|---|
| ~~Stability check on flash non-convergence~~ | Part 1 §10, §12; ROADMAP M9.1 | **done** (Step 0) |
| ~~Liquid–liquid (water/HC) stability sweep~~ | Part 1 §10 caveat, §12 | **done** (Step 0); γ-φ cannot be swept (stability is cubic-only) |
| ~~Stability on the single-phase exit for LL-prone systems~~ | Part 1 §12, §13 | **done**: pure-component screen on every cubic single-phase exit; 6 109 → 31 wrong answers |
| ~~Public `stability_analysis` reports an unconverged trial as `Stable`~~ | Part 1 §12, §13 | **done**: now `NoConvergence` (behaviour change, no signature change) |
| Fixed-width dual (`DualSVec64<W>`) for WS/MHV `∂lnφ̂ᵢ/∂nⱼ` | Part 2 §6; `SECOND_OPINION_TRIAL.md` | performance, patch exists |
| `MixtureScratch<'a, D>` instead of `SmallVec` for duals | Part 2 §7 | performance, same path as §6 |
| Simultaneous (n+1)-variable Newton for bubble/dew T | Part 1 §8, §11; ROADMAP M9.1 | performance + warm-start payoff |
| Newton on ln P for near-critical bubble/dew pressure | ROADMAP M9.1 | robustness/performance |
| Full Newton finish on ln K for the isothermal flash | Part 1 §5, §6 | performance, unmeasured need |
| ~~Multi-seed stability analysis~~ | Part 1 §7, §12, §13 | **done**: Wilson pair then near-pure seeds; 250 → 0 |
| Analytic phase-envelope Jacobian | Part 1 §9 | performance |
| Rayon batch writes straight into SoA output | Part 2 §10 | performance, needs a profile |
| Flat `kij` storage | Part 2 §3 | parked: needs a 1.0 API break |
| `Component` struct-of-arrays | Part 2 §2 | parked: no profile shows it |

---

## Step 0: stability fallback on flash failure (small, correctness)

**Done (unreleased, 2026-10-31)** — see Part 1 §12 for the numbers. In short:
all 89 hydrocarbon and 6 water/hydrocarbon failures on the rebuilt sweep
resolve. 2 of them are genuine near-critical splits: §10 had called them
stable because their 200-iteration trials had not converged. The fallback
therefore acts only on a *converged* stable verdict, and retries from the
trial K when the verdict is unstable. Normal-path benches are within noise.
The LL sweep found the bigger problem: wrong single-phase answers at 42 % of
a PR water/hydrocarbon grid. That changes what comes next; see *After Step
0* below. Original plan text follows.

- When `flash_isothermal` returns non-convergence, run `stability_analysis`.
  If the feed is stable, return a single-phase result instead of the error.
- **Evidence:** Part 1 §10. On a 60 × 60 T–P grid of six PR hydrocarbon
  mixtures, 55 of ~21 600 points (0.25 %) fail near the critical locus, and
  stability says *stable* at every one. The normal path pays nothing.
- **Same change:** run the liquid–liquid sweep (water/hydrocarbon, γ-φ) that
  §10 says was never done. Its result decides whether multi-seed stability
  (Part 1 §7) is needed at all.
- **Why first:** about a day of work with no speed risk. Step 2 needs the same
  "Newton failed, what now" fallback, and this sets the pattern.
- **Done when:** the 55 failure points return single-phase answers in a test,
  the normal-path flash benchmarks are unchanged, and the LL sweep result is
  recorded in Part 1 §10.

## After Step 0: the liquid–liquid finding — **done** (Part 1 §13)

The owner sequenced this before Step 1 (2026-10-31). Shipped: multi-seed
stability, a pure-component TPD screen on every cubic single-phase exit, and
an honest verdict for unconverged trials. Water/hydrocarbon wrong answers
fell from 6 109 to 31, and hydrocarbon answers are identical. The cost is
recorded as a cost: vapor single-phase exits +97…+148 %, stable-feed
`stability_analysis` 2–4.5×. Original text follows.

Step 0's sweep turned up a correctness gap bigger than the one it closed. The
cold flash returns single phase at iteration 0 whenever Wilson K gives
Rachford–Rice no root. On PR water/hydrocarbon that answer is wrong at 6 109
of 14 400 grid points. The fix is the §10 cost question with a measured case
on each side: stability on the single-phase exit costs about one extra flash
and changes nothing on hydrocarbons. One candidate is a per-system trigger
(water present, or any kij ≥ 0.2). It needs measuring. Multi-seed stability
(Part 1 §7) belongs with it: two-seed stability misses 250 splits that the
flash finds on water/propane/n-heptane. This is a correctness item, so it
arguably goes before Step 1's performance work. **That ordering is the
owner's call and is not decided here.**

## Step 1: Part 2 §6, fixed-width dual for WS/MHV composition derivatives

- **Evidence:** Wong-Sandler `d_ln_phi_d_n` is ≈ 5.1× the analytic path
  (Part 2 §2). The trial patch built on `num_dual::DualSVec64<W>` needs no new
  dual arithmetic and no trait-bound change. It passed all 194 tests and
  measured **2.28× at n = 4** and **3.6× at n = 20 with `W = 8`** (1.0× at the
  wrong width).
- **One blocker is resolved.** The 4.4 % "regression" on the classical path
  was build-to-build code-layout variance of about ±5 %, not the patch
  (`SECOND_OPINION_TRIAL.md`, resolution note of 2026-08-15).
- **The patch no longer applies.** `git apply --check` on
  [`second-opinion/dualn-patch-as-tested.diff`](second-opinion/dualn-patch-as-tested.diff)
  fails at `engine/src/mixture.rs:1169` (checked 2026-10-30), because
  Milestone 18 rewrote that area. It needs porting, not applying.
- **Work:**
  1. Add WS (and MHV) `d_ln_phi_d_n` benchmarks above n = 8. Today WS is only
     benchmarked at n = 4 (`engine/benches/engine_bench.rs`,
     `d_ln_phi_d_n_wong_sandler_n4`). Use n = 4 / 8 / 20 / 50.
  2. Port the patch onto `main` on a branch.
  3. Sweep `W` per size bucket and choose by measurement.
  4. Keep the FD-oracle and symmetry tests green, and the M12.3
     Gibbs–Helmholtz invariants.
- **Follow-on (Part 2 §7):** if a profile after §6 still shows `SmallVec`
  cost on the dual path, do `MixtureScratch` next. It is the same code.
- **Done when:** merged, with before/after criterion numbers at every n
  recorded in Part 2 §6.

## Step 2: simultaneous (n+1)-variable Newton for bubble/dew temperature

- **Evidence:** Part 1 §11. The current 1/T Newton moves composition by one
  successive-substitution step per iteration, so composition converges
  linearly (≈ 8 iterations at N = 10). That is why a realistic warm start
  (v0.18.0 `*_from`) saves only ≈ 25–30 %, while an exact one saves 6.7×.
- **Why after Step 1:** the full Jacobian needs `∂lnφ̂ᵢ/∂nⱼ` every iteration.
  For WS/MHV systems Step 1 makes each iteration cheaper.
- **Contract to keep:** the same equation and tolerance as the cold solve,
  warm ≡ cold, and any failure falls back to the existing solver
  (`flash/incipient.rs`). Never accept `K ≡ 1` as convergence.
- **Follow-ons using the same machinery:**
  - Newton on ln P for near-critical bubble/dew pressure.
  - Newton finish on ln K for the isothermal flash (Part 1 §6), **only if**
    iteration counts show SS/GDEM is the bottleneck. Measure first.
- **Done when:** realistic-guess warm starts are re-measured against the
  Part 1 §11 table, and the downstream consumer is told whether to switch
  from its own Newton path.

## Step 3: only when a profile justifies it

- **Part 2 §10, batch SoA output:** profile the batch path first. It already
  runs at 0.29 µs/point with the GIL released.
- **Part 1 §9, analytic envelope Jacobian:** now measurable. The
  `envelope_pr_methane_ethane_40pts` benchmark exists (see the correction
  note in Part 1 §2).
- **Part 1 §7, multi-seed stability:** only if Step 0's liquid–liquid sweep
  finds missed phase splits.

## Parked

- **Part 2 §3, flat `kij`:** only bundled with a deliberate 1.0 API break.
- **Part 2 §2, `Component` SoA:** only if a post-§1 profile shows it.

---

## Unreleased changes (draft release notes)

Nothing below is published yet. Carry it into `README.md`, `python/README.md`
and `engine/README.md` when the optimization work is released.

- **Stability fallback on flash non-convergence** (Part 1 §12). A flash that
  runs out of iterations now asks the stability test before raising. It
  returns a single phase only on a converged stable verdict, and otherwise
  retries from the trial phase.
- **Liquid–liquid splits are no longer missed** (Part 1 §13). On PR
  water/hydrocarbon the cold flash used to return a wrong single-phase answer
  at 42 % of a T–P grid. Every cubic single-phase answer is now checked by a
  pure-component tangent-plane screen, and the wrong answers drop to 0.2 %.
  Hydrocarbon answers are unchanged.
- **Multi-seed stability**: `stability_analysis` / `System.stability` add
  near-pure trial phases when the Wilson pair finds nothing.
- **Behaviour change:** `stability_analysis` raises / returns
  `NoConvergence` when its trials are cut off before a verdict, instead of
  answering `Stable`. The Python `max_iter` default is now 1 000 (was 100).
- **Costs, measured:** vapor single-phase flash exits ≈ 2–2.5× (+0.22–0.45 µs
  at n = 2–8), liquid exits +9–10 %, stable-feed stability 2–4.5×; two-phase
  flashes unchanged.

