# Petroleum Pseudocomponent & Crude-Column Plan

*Planning document, 2026-07-25. Prepared by Claude Code using Claude Opus 5
(1M context). **Status, 2026-08-16: §1.1 and §2's U1–U6 are all implemented
— the upstream half of this plan is complete and released (M18 in v0.14.0,
M19 + M20 in v0.15.0).***

> **Update, 2026-08-15 — blocker B1 is fixed.** §1.1's k_ij = 0 collapse, the
> sparse correction and the rank-1 derivative block shipped as **Milestone 18**
> and are measured linear to N = 300 (`ln_phi_mix` 30.7× faster, the
> composition Jacobian 50.4×). Numbers, the threshold a sweep corrected, and
> the one sub-item that is only partial live in
> [`OPTIMIZATION_PLAN_PART2.md`](OPTIMIZATION_PLAN_PART2.md) §7.
>
> **Update, 2026-08-16 — U1 and U2 shipped as Milestone 19.**
> `engine/src/petroleum/` (7 submodules, 121 unit tests) + `vle.petroleum`
> (37 wheel-level tests) + `notebooks/15_petroleum_characterization.ipynb`.
> Distillation-curve interconversion covers all four bases by both method
> families and matches Riazi (2005) Examples 3.2–3.5 and two API *Technical
> Data Book* worked examples to \< 0.15 °C; cutting supports equal-volume,
> equal-boiling-range and explicit product boundaries; the four
> critical-property families plus Lee–Kesler ω and four Zc correlations
> reproduce measured Tc/Pc/ω/M/Vc for ten pure hydrocarbons (worst case for
> the default family: Tc 1.3 %, Pc 5.1 %, M 6.0 %); Maxwell–Bonnell recovers
> normal boiling points from this crate's own Antoine equations to a 0.19 %
> mean (1.09 % worst). **Two gaps are recorded rather than glossed:** the Kesler–Lee `CF`
> naphthene correction is not implemented (ideal-gas Cp° up to 15.9 % low on
> ring compounds), and Edmister–Okamoto's 0–10 % EFV row genuinely crosses its
> neighbour, so D86 → EFV is rejected on feeds narrower than ~250 K.
>
> **Update, 2026-08-16 (later the same day) — U4 and U5 shipped as Milestone 20.**
> `engine/src/refinery/` (Lee–Kesler departure pure + mixture, Peneloux
> translation), `flash/free_water.rs` (the water-decant flash),
> `LiquidModel::{GraysonStreed, BraunK10}` with their (T, P)-constant parts
> cached, and a closed-form Maxwell–Bonnell inversion so BK10 costs an
> Antoine evaluation per component. 37 Rust + 13 Python tests, notebook 16.
> **Two scope decisions are recorded, not glossed:** the free-water model is
> the decant approximation (no three-liquid stability search — cannot find a
> second hydrocarbon liquid, neglects dissolved water); Braun's
> pressure-correction charts are not implemented (K10 scaled Raoult-style).
> Measured: 0.10 ms per Lee–Kesler mixture enthalpy at N = 300 from Python.
> A finding on the way: the legacy "Chao–Seader" table is Grayson–Streed's
> 1963 refit and never carried the regular-solution γ — kept as is, documented,
> the 1961 table added alongside.

**Target capability.** Simulate an **atmospheric crude distillation column**
with **hundreds of pseudocomponents** — the canonical refinery unit, and the
workload that most thoroughly stresses everything this project has built.

> **Status change, 2026-07-26.** This is no longer a speculative downstream ask:
> the crude distillation unit is now the **terminal goal of a downstream staged-separation (distillation) consumer**
> of this crate. Consequently **M18/M19/M20 are
> the gating path for a stated headline capability**, not optional-someday work,
> and inside-out has been promoted downstream from a stretch milestone to a
> required solver shipping *before* that consumer's v1.0. Its own milestone
> numbering was reworked at the same time — §3 below lists the deliverables.

**Placement.** Three new milestones in this repo — **18**, **19**, **20**
(Phases **25**, **26**, **27** in [MODERNIZATION_PLAN.md](../MODERNIZATION_PLAN.md)) —
plus the column-side work in the downstream consumer (the inside-out solver and
the crude tower itself), tracked in that project's own plans. This document is the shared
technical record for the **upstream** half, the same division of labor as
[DERIVATIVE_RELEASE_PLAN.md](DERIVATIVE_RELEASE_PLAN.md) vs. its Milestone 12
entries; the downstream half lives with the consumer.

---

## 1. The headline finding

An audit of both repos against this target says the binding constraint is
**not the thermodynamics**. The EOS layer, the activity models, the flash
algorithms and the derivative core are all adequate. Two *structural* things
break at N ≈ 300, and both have clean fixes that are worth doing on their own
merits:

| | Blocker | Where | Consequence at N = 300 |
|---|---|---|---|
| **B1** | The classical mixing rule is unconditionally **O(N²)**, even when every k_ij is zero | `engine/src/mixture.rs`, `quad_a` | ~90 000 cross-terms per fugacity evaluation for a quantity computable in ~300 |
| **B2** | Naphtali–Sandholm's Jacobian is dense per stage | the downstream column solver | Blocks of (2N+1)² = 361 000 entries; the block-Thomas sweep is ~10¹⁰ flops **per Newton iteration** |

Everything else in this plan is new physics or new correlations — real work,
but ordinary work. B1 and B2 are the two places where the *shape* of the code
has to change.

### 1.1 B1 — the quadratic form collapses when k_ij = 0

`quad_a` (`engine/src/mixture.rs`) always runs the full double loop:

```rust
for i in 0..n {
    let mut row = D::from(0.0);
    for j in 0..n { row += x[j] * a_ij(i, j); }
    a += x[i] * row;
    a_bar[i] = row * 2.0;
}
```

`kij_at` already treats an empty matrix as all-zero, and **a crude assay is
exactly the case where the matrix is empty**: binary interaction parameters
between petroleum pseudocomponents are conventionally zero. With k_ij = 0 the
cross-parameter factorizes, `A_ij = √(A_i)·√(A_j)`, and the whole form
collapses to a single pass:

$$S=\sum_i x_i\sqrt{A_i},\qquad A = S^2,\qquad \bar A_i = 2\sqrt{A_i}\,S$$

**O(N) instead of O(N²) — a 300× reduction in the inner loop, for a bit-identical
result** (up to floating-point summation order). Real assays carry a handful of
non-zero pairs (N₂, CO₂, H₂S against the hydrocarbons), so the general form is
`O(N + nnz)` with a sparse correction list:

$$A = S^2 - \sum_{(i,j)\in\mathcal{K}} x_i x_j\,\sqrt{A_iA_j}\,k_{ij}\cdot 2$$

The same collapse applies to the analytic composition-derivative block
(`d_ln_phi_d_n_classical`): it becomes a **rank-1 update** rather than a dense
N×N matrix, so the Jacobian block can be *applied* without ever being *formed*.
That is what makes B2's fix tractable too.

**This is worth doing whether or not the crude column ever gets built.** It is
a pure speedup of an existing hot path, with no new physics, benefiting every
current user of classical mixing.

### 1.2 B2 — the solver choice, not the solver quality

The downstream consumer's flagship is Naphtali–Sandholm: full Newton on MESH with a
block-tridiagonal Jacobian, (2N+1) unknowns per stage. At N = 300, S = 50 that
is ~10¹⁰ flops per iteration and gigabytes of Jacobian — seconds per iteration
at best.

This is precisely why every commercial simulator solves crude towers with
**inside-out** (Boston–Britt): the rigorous thermodynamics is called only in a
sparse *outer* loop to refit simple local K- and H-models, and the *inner* loop
— which does the actual iterating — is O(S·N).

**So the pseudocomponent ask reorders the downstream roadmap: inside-out moves
from Milestone 11 (stretch) onto the critical path.** It is the enabling
algorithm for this workload, not a historical footnote. Naphtali–Sandholm
remains the right flagship for the 5–20-component columns it was scoped for.

*(2026-07-26 update: acted on. Downstream, **inside-out is now required and
executes before v1.0**, so the consumer's v1.0 ships both solvers. It also reached back into
that project's still-unwritten MESH infrastructure — the thermo-provider boundary becomes a trait, profiles
become structure-of-arrays, and the spec system gains a general
computed-stream-property variant — because those are free before three solvers
consume the old shapes and a rewrite afterwards. Note the non-gate: **M18 does
not gate the downstream inside-out solver.** Inside-out on a 5–20-component column is buildable and
validatable immediately; M18 gates the C ≈ 300 performance claim.)*

---

## 2. Upstream gaps (this repo)

Verified against the working tree at v0.12.0. Re-verify signatures before
coding; line numbers drift.

| # | Capability | State today | Why the crude tower needs it |
|---|---|---|---|
| **U1** | **Petroleum characterization** — D86 ↔ TBP ↔ D2887 ↔ EFV interconversion; cutting a TBP curve into N pseudocomponents; T_b + SG → MW, T_c, P_c, ω, Z_c, V_c (Lee–Kesler, Twu, Riazi–Daubert, Kesler–Lee); Watson K | **shipped, M19** — `petroleum/{distillation,cuts,properties,gravity}.rs` | This *is* where "hundreds of pseudocomponents" come from. Nothing downstream can start without it |
| **U2** | **Fraction property correlations** — ideal-gas C_p° for petroleum fractions (API 7D3.6 from Watson K + gravity); Maxwell–Bonnell vapor pressure | **shipped, M19** — `petroleum/{cp,vapor_pressure}.rs`; `cp` emits the `Component.cp_coeffs` polynomial directly. **Partial:** the Kesler–Lee `CF` ring correction is absent (naphthene C_p° up to 15.9 % low) | Enthalpy balances and K-values per pseudocomponent |
| **U3** | **N-scalable mixture core** — the k_ij = 0 fast path and sparse-k_ij correction of §1.1, the matching rank-1 derivative block, plus an N-sweep benchmark (N = 10/50/100/300) guarding the scaling | **shipped, M18 (v0.14.0)** — linear to N = 300 | Without it every outer-loop thermo call is quadratic in assay size |
| **U4** | **Free-water / three-phase** — VLLE stability + flash, or at minimum a water-decant model | **shipped, M20 — the water-decant model** (`flash/free_water.rs`). Not a three-liquid stability search: cannot find a second hydrocarbon liquid, neglects dissolved water; the column side (D6) is downstream | Atmospheric towers run **stripping steam**. Water forms a second liquid phase in the overhead drum and every side stripper. Unavoidable, not an edge case |
| **U5** | **Refinery methods** — Grayson–Streed (a Chao–Seader extension), BK10, Lee–Kesler enthalpy departure, Peneloux volume translation | **shipped, M20** — `LiquidModel::{GraysonStreed, BraunK10}` (`flash/system.rs`, cached per (T, P)), `refinery/{lee_kesler,volume_translation}.rs`. **Partial:** BK10 without Braun's pressure-correction charts (stated); the legacy `ChaoSeader` found to carry the GS 1963 table without γ — kept, documented | These are what refinery cases are *validated against*; a bare cubic EOS gives heavy-cut densities and enthalpies too far off |
| **U6** | **Allocation-free N-component evaluation** — SoA buffers, arena reuse across stages, `TpCache` hoisted across a whole column solve | **shipped, M18** — `MixtureWorkspace` + `TpCache`; the M20 methods hoist their (T, P) constants into `SystemTpCache` too | At 50 stages × 300 components × Newton iterations, per-call `Vec`s dominate |

### Milestone mapping

- **Milestone 18 — N-Scalable Mixture Core** (U3 + U6). *Independent of
  everything else and independently valuable; do it first so the core is
  proven to scale before anything is built on top.*
- **Milestone 19 — Petroleum Characterization** (U1 + U2). The largest new
  module; gated by nothing. **Shipped 2026-08-16** as `engine/src/petroleum/`
  + `vle.petroleum` + notebook 15.
- **Milestone 20 — Refinery Thermodynamics** (U4 + U5). Depends on 19 for the
  fractions it applies to. **Shipped 2026-08-16** as `engine/src/refinery/` +
  `flash/free_water.rs` + the two new liquid models + `vle.refinery` + notebook 16.

---

## 3. Downstream work (the staged-separation consumer)

*The column-side deliverables, as agreed 2026-07-26; the detailed plan and its
milestone mapping live with the downstream consumer.*

| | Deliverable | Sequencing |
|---|---|---|
| **D1** | **Inside-out (Boston–Britt), now required.** Outer loop: rigorous `vle-thermo` K and H at the current profiles → fit the local models. Inner loop: solve MESH against the *simple* models. This is what makes N = 300 feasible | *before the consumer's v1.0* |
| **D2** | **Crude-tower topology.** Pumparounds (heat removal), side strippers (their own mini-columns with steam), a flash zone, overflash. Multi-feed and side draws already arrive with its MESH infrastructure; these are new structure | the crude-tower milestone |
| **D3** | **Product-quality specs.** D86 95 % points and gaps/overlaps, not mole-fraction purity. Needs a spec type that runs a distillation-curve calculation on a product stream — which loops back to U1 | last |
| **D4** | **Lumping / de-lumping.** The pragmatic escape hatch: solve ~20 lumped pseudocomponents, de-lump the converged profiles. Worth having even once inside-out lands, because it is what makes design *sweeps* interactive | first crude-tower step |
| **D5** | **Steam from `vle-steam`.** Stripping steam is best evaluated from IF97 rather than a cubic EOS, so `vle-steam` becomes a dependency of the crude-tower path. (Its transport properties, Milestone 13.7, then feed tray hydraulics and heat-transfer sizing later) | the crude-tower milestone |
| **D6** | **Three-phase / decanting *column* stages** *(added 2026-07-26)*. **U4 is not the whole story:** this plan listed free water as an upstream need only, but the *column* side is downstream work — a condenser that decants a water leg, side strippers where free water exists, a two-liquid stage in the MESH residual set, a decant draw in the material balance. It appeared in no checklist on either side before now. U4 supplies the thermodynamics; the stage model is the consumer's | the crude-tower milestone |

---

## 4. Sequencing

| Phase | Repo | Content | Gated by |
|---|---|---|---|
| ~~**A**~~ | vle | ~~**M18** — N-scalable core + N-sweep benches~~ **done, v0.14.0** | — |
| ~~**B**~~ | vle | ~~**M19** — characterization + fraction correlations~~ **done 2026-08-16** | — (parallel with A) |
| ~~**C**~~ | vle | ~~**M20** — free water + refinery methods~~ **done 2026-08-16** | B |
| **D** | downstream | inside-out solver (D1) | its own solver-robustness work — **not A** |
| **E** | downstream | petroleum feed path (D4) | B, D |
| **F** | downstream | topology + steam + three-phase (D2, D5, D6) | C, E |
| **G** | downstream | quality specs, C ≈ 300, validation (D3) | A, F |

A and B are independent and may interleave. **A is worth starting immediately
and on its own merits**, independent of whether the crude column is ever built.

*Corrected 2026-07-26:* phase D was previously shown as gated by A. It is not —
inside-out on a 5–20-component column can be built and validated against
Naphtali–Sandholm with no upstream work at all. **A gates the C ≈ 300 performance
claim (phase G), not the solver's existence**, which is what allows D1 to land
before the consumer's v1.0 without waiting on this repo.

## 5. Validation targets

A capability this large is only real if it reproduces a published case:

1. **Characterization** — reproduce a published assay's cut properties (API
   Technical Data Book worked examples) per pseudocomponent.
   *Met, with one substitution worth recording (2026-08-16).* The
   **interconversions** are validated exactly as planned, against Riazi (2005)
   Examples 3.2, 3.3, 3.4 and 3.5 and two API *Technical Data Book* worked
   examples, all to \< 0.15 °C. For the **property correlations** no per-cut
   API worked example was obtainable, so they are validated instead against
   *measured* Tc/Pc/ω/M/Vc for ten pure hydrocarbons from this repo's own
   component database (n-C5…n-C10, benzene, toluene, cyclohexane,
   methylcyclohexane). That substitution is arguably the stronger test: these
   correlations are *fitted* to pure-hydrocarbon data, so a mistyped
   coefficient cannot match across ten compounds spanning three hydrocarbon
   families by accident — which is exactly how two transcription errors (an ω
   sign and a Twu pressure unit) were caught during implementation.
2. **N-scaling** — the criterion N-sweep must show the mixing-rule cost growing
   **linearly**, not quadratically, from N = 10 to N = 300 with empty k_ij.
   This is the measurement that decides whether §1.1 actually worked; an
   argument is not admissible (CLAUDE.md, *performance claims need a
   measurement*).
3. **Crude tower** — Watkins' *Petroleum Refinery Distillation* design case is
   the canonical textbook target; ChemSep and DWSIM ship comparable examples
   for cross-checking product rates, draw temperatures and pumparound duties.

## 6. Alternatives considered

- **Keep Naphtali–Sandholm and exploit sparsity instead of adopting
  inside-out.** With k_ij = 0 the ∂lnK/∂x block is rank-structured, so a
  Sherman–Morrison update could replace the dense per-stage LU. Attractive
  because it reuses the flagship solver — but it optimizes the *inner* loop of
  a method whose cost is dominated by calling rigorous thermodynamics on every
  stage of every iteration, which is the thing inside-out exists to avoid.
  Worth revisiting as a *second* solver, not the first.
- **Lump aggressively and skip the N-scaling work** (D4 alone). Cheapest path
  to a converged column, and genuinely how a lot of engineering gets done —
  but it caps the product-quality resolution the whole exercise is for, and
  leaves B1 in place for every other user.
- **Do the region-3-style backward-equation treatment for characterization**
  (pre-tabulate cut properties). Premature: characterization is called once per
  assay, not per iteration. Not a hot path.
