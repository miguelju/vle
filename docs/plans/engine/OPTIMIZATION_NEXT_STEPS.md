# Optimization Next Steps — sequencing the open items

*Drafted 2026-10-30 by Claude Code using Claude Opus 5.5, after v0.18.0.*

**Status: every step is done, plus one addition found afterwards (Part 2
§10, 2026-11-11); nothing is released.** Step 0 (Part 1 §12),
the correctness findings it raised (Part 1 §13), Step 1 (Part 2 §8), Step 2
(Part 1 §14) and Step 3 (Part 1 §15, Part 2 §9) are all unreleased. **No
release until every step is done** (owner's decision, 2026-10-31). That
condition is now met. The release itself is still the owner's call. Items
in the inventory that no step covers stay open (see *Still open*). *Unreleased
changes* at the end is the draft of the release notes. This is an ordering of the
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
| ~~Fixed-width dual (`DualSVec64<W>`) for WS/MHV `∂lnφ̂ᵢ/∂nⱼ`~~ | Part 2 §6, §8; `SECOND_OPINION_TRIAL.md` | **done** (Step 1): 2.5× (n = 4) → 4.2× (n = 50) |
| `MixtureScratch<'a, D>` instead of `SmallVec` for duals | Part 2 §7 | performance, same path as §6; no profile taken yet |
| ~~Simultaneous (n+1)-variable Newton for bubble/dew T~~ | Part 1 §8, §11, §14; ROADMAP M9.1 | **done** (Step 2): warm start −43 %, near-critical 2–5× |
| Newton on ln P for near-critical bubble/dew pressure | ROADMAP M9.1 | robustness/performance |
| Full Newton finish on ln K for the isothermal flash | Part 1 §5, §6 | performance, unmeasured need |
| ~~Multi-seed stability analysis~~ | Part 1 §7, §12, §13 | **done**: Wilson pair then near-pure seeds; 250 → 0 |
| ~~Analytic phase-envelope Jacobian~~ | Part 1 §9, §15 | **done** (Step 3): 1.8× (n = 2) → 3.7× (n = 10) |
| ~~Rayon batch writes straight into SoA output~~ | Part 2 §9, §10 | **done** (Step 3): parallel batch −24…−32 % |
| ~~One-pass stage thermodynamics; duplicated departure enthalpy~~ | Part 2 §10 (found 2026-11-11 by profiling the downstream consumer) | **done**, unreleased: `stage_thermo_into` 4.5–4.8× the five calls; `phase_enthalpy_entropy` −50 % |
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

**Done (unreleased, 2026-10-31)**: see Part 2 §8. Ported as
`mixture::d_ln_phi_d_n_dual::<W>` with W = 4 for n ≤ 4 and W = 8 above,
chosen by a width sweep. The patch's W = 16 / 32 buckets were 1.6× / 3.1×
slower at n = 20. Wong-Sandler `d_ln_phi_d_n` runs 2.5× faster at n = 4, 4.0×
at n = 8, 3.6× at n = 20 and 4.2× at n = 50, with MHV1 the same. It is still
O(n³). Costs: the engine's release compile is +33 %; wasm size is unchanged.
No in-tree flash calls `d_ln_phi_d_n` yet, so flash benches do not move.
Step 2 is the first in-tree consumer. The §7 `MixtureScratch` follow-on stays
open because no profile has been taken. Original plan text follows.

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

**Done (unreleased, 2026-11-01)**: see Part 1 §14. The full Newton step on
{ln K, ln T} uses the incipient phase's exact ∂ln φ̂/∂n (Step 1's dual where
the mixing rule needs it). It switches in near the root only where SS is
slow. Both rules share the v0.18.0 acceptance test, so warm ≡ cold still
holds and every answer is unchanged. The unused ∂/∂P derivative is no
longer computed, which turned out to be most of the cold-solve gain.

On the 10-component RKS mixture, a 0.5 K / 5 % guess costs 7.9 µs (bubble)
and 7.5 µs (dew), against 13.9 / 13.2 µs in v0.18.0: −43 %, where §11 had
measured ≈ 25–30 %. Cold solves are −28 % / −27 %. Near the PR
methane/ethane critical region solves are 2–5× faster: K evaluations
20–27 → 6–7 cold and 17–23 → 4 warm.

Rejected, measured: an unconditional full step made the cold 10-component
dew point 31 % slower, because SS there already contracts 30× per step.
Two other switch rules lost elsewhere (§14).

**Downstream consumer:** realistic warm starts now pay about 1.8×, so a
caller with its own Newton warm path should re-measure against
`*_temperature_from` before keeping it. The floor is now per-iteration cost
(3 K evaluations from a near guess), not iteration count.

Follow-ons still open:
- Newton on ln P for the pressure solves.
- The full step for γ-φ dew points (needs ∂ln γ/∂x).
- A Newton finish for the isothermal flash (only if measured).

Original plan text follows.

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

**Done (unreleased, 2026-11-01).** Both measurements justified the change.

- **Envelope Jacobian** (Part 1 §15). It is now exact: the incipient phase's
  ∂ln φ̂/∂n, plus one two-lane T/P dual pass per phase, on the root
  min-Gibbs chose. It was a forward difference costing 2(n+2) min-Gibbs
  evaluations per Newton iteration. Tracing is 1.8× faster at n = 2 and
  3.7× at n = 10 (criterion −48.8 %). Every traced point matches `main` to
  ≤ 7e-9.
- **Batch SoA output** (Part 2 §9). The profile contradicted the deferral
  reason. The single-threaded transpose after the parallel region was
  19–23 % of a parallel 200 000-point call (3–4 % serial). Workers now write
  into flat column buffers that become the numpy arrays without another
  copy. Parallel batches are −32 % (n = 2) and −24 % (n = 10), serial −4 %,
  and the output is bit-identical.
- **Multi-seed stability** was done earlier (Part 1 §13).

### Still open (no step covers them)

The Step 2 follow-ons (Newton on ln P, the γ-φ dew full step, an
isothermal-flash Newton finish if measured), Part 2 §7 `MixtureScratch` (no
profile yet), and the *Parked* items below.

Original plan text follows.

- **Part 2 §10, batch SoA output:** profile the batch path first. It already
  runs at 0.29 µs/point with the GIL released.
- **Part 1 §9, analytic envelope Jacobian:** now measurable. The
  `envelope_pr_methane_ethane_40pts` benchmark exists (see the correction
  note in Part 1 §2).
- **Part 1 §7, multi-seed stability:** only if Step 0's liquid–liquid sweep
  finds missed phase splits.

## After Step 3: found by profiling the downstream consumer — **done** (Part 2 §10)

Not in the original inventory. A profile of the consumer's column solve on
this branch (2026-11-11) put 56 % of it in five overlapping calls per stage.
Shipped, unreleased: the departure enthalpy is computed once
(`phase_enthalpy_entropy` −50 %, adiabatic flash −19 %),
`k_values_with_t_derivs` is public (−32 % against the full call), and
`stage_thermo_into` returns K, ∂ln K/∂T, H and Cp of both phases from one
second-order dual pass per cubic phase (4.5–4.8× the five calls). The
consumer's 40 × 10 SRK solve goes 4.62 → 2.52 ms once it switches, which
needs a release of this crate first. Rejected, measured: widening the inline
buffers from 8 to 16.

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
- **Faster composition Jacobian for GE mixing rules** (Part 2 §8).
  `d_ln_phi_d_n` / `mixture_d_ln_phi_d_n` for Wong-Sandler, Huron-Vidal, MHV
  and 3-parameter EOS now carry 4–8 derivative directions per pass. Measured
  on Wong-Sandler and MHV1: 2.5×
  faster at 4 components, 4.2× at 50. Results agree with the previous path to
  1e-12.
- **Faster saturation temperatures** (Part 1 §14). `bubble_temperature`,
  `dew_temperature` and their `*_from` warm starts take a simultaneous
  (n+1)-variable Newton step near the root, with the same answers to solver
  tolerance. On a
  10-component mixture a realistic warm start is 43 % cheaper (13.9 →
  7.9 µs) and a cold solve 27–28 %; near a critical point solves are 2–5×
  faster. γ-φ dew points keep the previous update rule.
- **Faster phase envelopes** (Part 1 §15). `trace_envelope` /
  `System.phase_envelope` use an exact Jacobian in place of finite
  differences. They are 1.8× faster for a binary and 3.7× for 10
  components, and trace the same points.
- **Faster parallel batches** (Part 2 §9). `flash_pt_batch` and the four
  bubble/dew `_batch` methods build their numpy output without a
  single-threaded copy after the parallel region. Parallel calls are
  24–32 % faster, and the results are bit-identical.
- **One call for a column stage** (Part 2 §10). New `stage_thermo_into` /
  `System.stage_thermo` returns K-values, ∂ln K/∂T and the enthalpy and heat
  capacity of both phases. For a cubic liquid and vapor it is 4.5–4.8×
  faster than the five separate calls and returns the same numbers; other
  model pairs are assembled from those calls. New `k_values_with_t_derivs` /
  `System.k_values_with_t_derivs` skips the pressure derivative (−32 %).
- **Faster enthalpies** (Part 2 §10). `phase_enthalpy_entropy` /
  `System.enthalpy_entropy` on a cubic phase no longer computes the
  departure enthalpy twice: −50 %. The adiabatic (PH) flash is 19 % faster
  on the benchmarked binary. Same answers.
- **Costs, measured:** vapor single-phase flash exits ≈ 2–2.5× (+0.22–0.45 µs
  at n = 2–8), liquid exits +9–10 %, stable-feed stability 2–4.5×; two-phase
  flashes unchanged.

