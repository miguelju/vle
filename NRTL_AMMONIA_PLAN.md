# Plan: Add NRTL activity model + ammonia to vle-thermo

**Status:** SHIPPED as Milestone 14 / v0.11.0 (2026-07-09) — §§1–7 are the executed design
record. **§8 (methanol–water van Laar vs NRTL comparison) is a PROPOSED follow-up, not yet
scheduled.** **Driver:** downstream — stages-thermo Milestone 2
(Ponchon–Savarit) needs a proper heat-of-mixing model and the ammonia component to teach
the ammonia–water enthalpy–composition method. This is the vle-side upstream milestone that
gates stages-thermo M2 (sibling to the derivative release tracked in `DERIVATIVE_RELEASE_PLAN.md`).
When executed, add ROADMAP.md / TODO.md / MODERNIZATION_PLAN.md entries + a model-attribution line
per this repo's milestone rules.

## Why NRTL (and not UNIQUAC / extended UNIQUAC / a Helmholtz EOS)

Design conclusion from the stages-thermo M2 planning conversation:

- vle-thermo today has Ideal / VanLaar / Wilson / Scatchard / Margules. Van Laar & Margules give
  only the crude legacy `Hᴱ = Gᴱ, Sᴱ = 0` (T-independent params); Wilson has a proper analytic
  T-derived `Hᴱ`. None model ammonia–water's large heat of mixing well.
- **NRTL is the right general investment.** It is the standard model for aqueous-associating and
  polar mixtures and lifts the *whole* aqueous-nonideal ladder stages-thermo will use — methanol /
  ethanol / 2-propanol / acetone–water and the later extractive/azeotropic ternaries — not just
  ammonia–water. On a single binary it has 3 adjustable knobs (τ₁₂, τ₂₁, α₁₂) vs plain UNIQUAC's 2,
  so it fits VLE **and** `Hᴱ` at least as well.
- **Plain UNIQUAC is strictly worse here:** no more accurate on this binary (its size-asymmetry edge
  is wasted on two small molecules) *and* it forces new per-component structural fields `r`, `q` onto
  every DB entry. NRTL reuses the existing `aij` matrix pattern.
- **Extended UNIQUAC / Helmholtz EOS are single-use luxuries.** The models that actually reproduce the
  textbook ammonia–water chart are *extended* UNIQUAC (Thomsen–Rasmussen: Debye–Hückel + speciation) or
  a Helmholtz-energy EOS (Tillner-Roth & Friend). Their distinguishing capability serves **nothing else**
  on the stages roadmap (every other planned system is neutral / non-electrolyte, and the cubic EOS already
  meets the ~1% cross-simulator target). So we add NRTL once for broad benefit, and stages-thermo will
  reproduce the ammonia–water *textbook chart* from reference data (Ibrahim–Klein / Tillner-Roth) rather
  than build single-use thermodynamics. Full lesson: stages-thermo `M2_PONCHON_SAVARIT_PLAN.md`.

## Repo conventions that apply (from this repo's CLAUDE.md)

- **PyO3 bindings + tests in the same commit series** — no pure-Rust-without-Python.
- **Analytic derivatives** (`∂lnγ/∂T`, `dGᴱ/dT`); numeric forms only as test oracles (`num-dual`).
- **Doc-sync before push:** ROADMAP.md, TODO.md, MODERNIZATION_PLAN.md, PERFORMANCE_PROPOSAL.md,
  DERIVATIVE_RELEASE_PLAN.md, and the three package READMEs (`engine/README.md`, `python/README.md`,
  `steam/README.md`). Package-page READMEs are immutable per published version.
- **Pre-push gates:** private-data grep + `cargo fmt --check` (`hooks/pre-push`, `core.hooksPath hooks`);
  never `--no-verify`. clippy is CI-only.
- **Python env:** `~/miniconda3/envs/vle/bin/{python,pytest,maturin,jupyter}`; `maturin develop` from `python/`.
- **Release:** bump **both** `Cargo.toml [workspace.package] version` and `python/pyproject.toml`, land on
  `main`, then Miguel signs a `vX.Y.Z` tag (YubiKey) → `release.yml` publishes PyPI (OIDC) + crates.io.

## 1. NRTL model — design

- **Parameterization (mirror Wilson's energy convention).** Store the interaction energy `gᵢⱼ − gⱼⱼ`
  (kJ/kmol) in the existing `aij` off-diagonals; define `τᵢⱼ = aij[i][j] / (R·T)`,
  `Gᵢⱼ = exp(−αᵢⱼ · τᵢⱼ)`. Because `τ` carries T-dependence through `1/T`, the existing `num-dual`
  generic path yields exact `∂lnγ/∂T` and a nonzero analytic `Hᴱ` for free — exactly as Wilson does.
  **Implement the general multicomponent form** (option B below exists precisely for ternary+):
  `ln γᵢ = Σⱼτⱼᵢ Gⱼᵢ xⱼ / Σₖ Gₖᵢ xₖ + Σⱼ [xⱼGᵢⱼ / Σₖ Gₖⱼxₖ]·(τᵢⱼ − Σₘ xₘτₘⱼGₘⱼ / Σₖ Gₖⱼxₖ)`.
  The binary reduction `ln γ₁ = x₂²[ τ₂₁ (G₂₁/(x₁+x₂G₂₁))² + τ₁₂ G₁₂/(x₂+x₁G₁₂)² ]` (and symmetric)
  serves as a closed-form test oracle.
  If the NH₃–H₂O fit is inadequate, escalate to `τ = a + b/T` (a second matrix) — **not** in this pass.

- **Non-randomness α storage — option B (chosen).** NRTL needs `αᵢⱼ` (symmetric, a *pair* property).
  The stack currently threads a single `aij` matrix.
  - Option A (overload the `aij` diagonal for α) is lowest-touch but **binary-only** — 3 pair-α's can't
    map onto 3 component-diagonal slots, so it breaks for ternary and would need redo for stages-thermo M9.
  - **Option B (chosen):** add a parallel symmetric `alpha: Vec<Vec<f64>>` field to `SystemSpec`, the
    `System` pyclass, and `GeSpec`, threaded through `ln_gamma` / `_all` / `_generic` / `excess_*`.
    Larger diff, but correct for ternary → serves stages-thermo M9's extractive acetone–methanol–water.

## 2. NRTL — files to touch

- `engine/src/activity.rs`
  - Add `Nrtl = 37` to `ActivityModel`. **Not 26**: the discriminants live in the single legacy
    VB6 model-ID space (`CubicEos` 0–20, `TADiPGammaModel` activity 21–25, `TADiPMR` mixing rules
    26–33, project-assigned C-rules 34–36), and 26 is already `MixingRule::WongSandler`
    (`engine/src/mixing.rs:38`). 37 is the first free ID. Extend
    `discriminant_values_match_legacy` with `Nrtl as i32 == 37` plus a comment that NRTL has no
    legacy counterpart — the value is project-assigned, chosen to never collide with the legacy space.
  - NRTL branches in `ln_gamma` (~L102), `ln_gamma_all` (~L272), `ln_gamma_all_generic<D>` (~L335),
    `excess_enthalpy` (~L469).
  - `nrtl_tau` / `nrtl_g` helpers in both f64 and `_generic<D: DualNum<f64>>` forms.
- `engine/src/flash/system.rs` — add `alpha` to `SystemSpec` (~L28); pass into the `ln_gamma_all(...)`
  (~L163) and `dln_gamma_dt` (~L354, the dual-number ∂lnγ/∂T helper) calls.
- `engine/src/mixture.rs` — add `alpha` to `GeSpec` (~L114) so NRTL can also feed the GE mixing
  rules (WS/HOV/HVS/MHV1/MHV2). **Decided: thread it now**, not "if needed later" — the signature
  churn happens in this pass anyway, NRTL-inside-Wong-Sandler is a standard pairing, and skipping
  it would require an explicit `MixError::Unsupported` guard to avoid silently-wrong γ's; threading
  is the same effort without the dead end. `ge_terms` in `params_generic` forwards `alpha` like it
  forwards `vl`/`delta`.
- `engine/src/py_system.rs` — `System::new` signature (~L287) + `spec()` (~L145): add an `alpha=` kwarg.
- `engine/src/py_bindings.rs` — extend the four `activity_*` `#[pyfunction]`s (~L873) to accept α;
  optionally extend `fit_aij_py` (~L1802) to fit NRTL τ's with α fixed.
- `python/src/vle/system.py` — add `"nrtl": ActivityModel.Nrtl` to `_ACTIVITY_ALIASES` (~L75) + an
  `alpha=` kwarg on `System.__init__`.

## 3. Tests (same commit series)

- `engine/src/activity.rs` `#[cfg(test)]`:
  - copy `wilson_excess_enthalpy_matches_numerical_oracle` (~L668) for NRTL — analytic `Hᴱ` vs a
    central-difference `−T² · d(Gᴱ/T)/dT` oracle (the analytical-vs-oracle mandate);
  - add NRTL to `ln_gamma_all_matches_per_component_for_every_model`;
  - a closed-form binary-NRTL γ check at a known point.
- `python/tests/test_m8_activity.py` — NRTL analogues of `activity_ln_gamma` / `activity_excess_*`.
- `engine/src/flash/aij_regression.rs` — an NRTL round-trip on synthetic data (α fixed).

## 4. Ammonia in the component DB

- `scripts/build_components_json.py` is the **single generator** for all three JSON copies. Add ammonia
  to `RAW_NEW` — tuple `(formula, cas, mw, tc, pc, omega, zc, vc, tb, antoine, vliq)` — plus a `CP` entry
  (degree-4 `Cp°/R` quartic, **load-bearing for every enthalpy balance**) and a `TWO_POINT` anchor if there
  is no liquid-range Antoine. Approx values (verify against Poling/Prausnitz *Properties of Gases & Liquids*,
  DIPPR): mw ≈ 17.03, tc ≈ 405.4 K, pc ≈ 11333 kPa, omega ≈ 0.253, tb ≈ 239.8 K, vliq ≈ 24.7 cm³/mol.
  Then regenerate: `~/miniconda3/envs/vle/bin/python scripts/build_components_json.py`.
- `engine/src/db.rs` — bump the count test `all_24_compounds_parse` (~L191): rename to
  `all_25_compounds_parse`, assert 25, update the "M12.1 database holds 24 compounds" comment;
  add an ammonia spot-check mirroring `spot_check_new_compound_vs_json_literals`.
- Python DB tests: `python/tests/test_db.py`, `test_rust_db.py`, `test_components.py`, `test_components_cp.py`.

## 5. NH₃–H₂O NRTL parameters

- **Prefer published params** (Aspen/DECHEMA NRTL for ammonia–water, α ≈ 0.2–0.3) over a from-scratch
  regression — more reliable. Validate against one VLE dataset (bubble-P vs x at fixed T).
- **Fallback:** regress τ's via `fit_aij` (`engine/src/flash/aij_regression.rs`, LM on bubble-pressure
  residuals; α fixed). Note `fit_aij` is hardwired to a 2-param `(aij[0][1], aij[1][0])` fit — reuse as-is
  with α fixed, or generalize the residual builder to inject α.
- **Accuracy bar:** qualitative / few-% at moderate pressure; document the elevated-P boundary. Enough for
  stages-thermo to demonstrate CMO error honestly; **not** expected to match the Bošnjaković chart (that is
  route (b) in stages-thermo M2 — reference data).

## 6. Milestone notebook (repo convention — required)

A milestone that ships a user-facing model must end with a milestone notebook (see CLAUDE.md
*Per-milestone artifact workflow* + *Notebook Conventions*): NRTL γ's + `Hᴱ` for NH₃–H₂O,
the bubble-P-vs-x validation plot against the literature dataset from §5, the required
structure (setup cell, research-paper context, worked example, ≥2 exercises, references),
executed top-to-bottom via `nbconvert --execute`, and catalogued in `deploy/NOTEBOOKS.md`.

## 7. Release

Doc-sync the files above, model-attribution line, `cargo fmt --check`, bump both version fields
(→ **0.11.0**), land on `main`; Miguel signs the `v0.11.0` tag; `release.yml` publishes to crates.io + PyPI.
**stages-thermo M2 then bumps `vle-thermo = "0.11"`.**

## Verification (end-to-end)

- `cargo test -p vle-thermo` — NRTL analytic-`Hᴱ`-vs-oracle, discriminant, γ round-trip, ammonia DB spot-check.
- `maturin develop` (from `python/`) then
  `~/miniconda3/envs/vle/bin/pytest python/tests/test_m8_activity.py python/tests/test_db.py`.
- NH₃–H₂O bubble-P-vs-x vs one literature dataset (few-% target at moderate P).

## 8. Follow-up (PROPOSED 2026-07-11, Miguel): van Laar vs NRTL on methanol–water in notebook 13

**Status: proposed — not yet scheduled.** Origin: reviewing the README P–x–y "sail"
(methanol/water, van Laar), Miguel asked whether NRTL is the better model for that
mixture now that M14 shipped it. Decision: the **hero image stays van Laar** — it
showcases the thesis's validated configuration (Chapter IV Table 4.6 is methanol/water
*with van Laar* at 298.15 K), so provenance wins there. The comparison belongs in
**notebook 13** (the NRTL notebook), as a new worked section + exercise.

### Design

New section in `notebooks/13_nrtl_ammonia.ipynb` after the NH₃–H₂O material:
**"Same data, two models: van Laar vs NRTL on methanol–water."**

1. **Data:** the Table 4.6 bubble-pressure points (T = 298.15 K, x₁ sweep) — already
   transcribed in `notebooks/04_bubble_dew_point.ipynb`; reuse those literals so the two
   notebooks pin identical numbers.
2. **Fit NRTL to the thesis's own data** — no external parameter sourcing needed:
   `vle._engine.fit_aij_py(model=NRTL, …, data=[(T, x1, P_exp)…], alpha=[[0,0.3],[0.3,0]])`
   fits the two energies by Levenberg–Marquardt with α₁₂ = 0.30 held fixed (the standard
   aqueous-alcohol value). This showcases M9's regression + M14's model in one cell —
   the same workflow a practitioner uses with DECHEMA data.
3. **Compare:** overlay bubble-P–x₁ curves (van Laar with the thesis's A₁₂/A₂₁ vs the
   freshly fitted NRTL) over the Table 4.6 points; report RMSE of both fits; plot
   γ₁, γ₂ vs x₁ for both models. Expected outcome (state it honestly in prose): for this
   fully miscible binary at one temperature the two models fit comparably — NRTL's real
   advantages are T-extrapolation (τ = Δg/RT), a nonzero analytic Hᴱ, and multicomponent/
   LLE reach, not a better single-isotherm fit.
4. **Assertion cells:** NRTL fit converges; NRTL RMSE ≤ ~1.5× van Laar RMSE (guards
   against a silently broken fit without over-pinning); one spot γ value each model.
5. **Exercise (with hidden solution):** refit at α₁₂ = 0.2 and 0.47 and discuss
   sensitivity — connects back to the NH₃–H₂O α-sensitivity exercise already in the
   notebook. Optionally a second exercise: predict the 60 °C bubble curve with both
   models' 25 °C parameters and discuss which extrapolates more credibly (NRTL's
   T-dependence vs van Laar's constant A's) — needs a literature 60 °C dataset to
   grade against, else keep qualitative.

### Tasks

- [ ] Extend `scripts/build_notebook_m14.py` (it generates notebook 13) with the new
      section + exercise cells; regenerate; verify fresh-kernel execution
      (`jupyter execute`).
- [ ] Cross-check the Table 4.6 literals against notebook 04 (single source of truth:
      copy the same list, comment pointing at 04).
- [ ] Doc sync per CLAUDE.md: `deploy/NOTEBOOKS.md` row for notebook 13 gains
      "van Laar-vs-NRTL methanol–water comparison"; no ROADMAP/TODO/plan renumbering
      (this is a notebook update inside shipped M14 scope, not a new milestone).
- [ ] No release: notebooks distribute via GitHub; no crate/wheel change.

**Estimate:** 2–4 h. **Verification:** notebook executes top-to-bottom; assertions
green; both RMSE values printed; visual check of the overlay plot.
