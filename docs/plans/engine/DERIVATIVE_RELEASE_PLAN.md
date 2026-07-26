# Downstream Derivative & Database Release Plan (Milestone 12 → vle-thermo 0.9.x)

*Planning document, 2026-07-05. Prepared by Claude Code using Claude Fable 5 for
execution by Claude Opus 4.8.*

**Status: SHIPPED as Milestone 12 / Phase 19 — all five upstream gaps closed.**
M12.1 released in **v0.8.2** (2026-07-05); M12.2–12.5 in **v0.9.0** (2026-07-06);
**v0.9.1** (2026-07-06) is the follow-up patch for the Wong-Sandler
departure-enthalpy `db/dT` bug that the M12.3 Gibbs–Helmholtz invariant test
surfaced (§7). This file remains the *design record*; the as-built state is
[ROADMAP.md](../../../ROADMAP.md) Milestone 12 and
[MODERNIZATION_PLAN.md](../MODERNIZATION_PLAN.md) Phase 19.

> **Correction (2026-07-26).** This header previously read *"This document is a
> plan only — nothing in it has been implemented yet."* True on 2026-07-05, stale
> from 2026-07-06 onward. Caught while building
> [the Plan & Audit History](../README.md); see CLAUDE.md *Completion Claims Must
> Be Verified Against the Code*.

**Origin**: `stages-thermo` — the staged-separation (distillation) library planned
as the first downstream consumer of `vle-thermo` (see its `PLAN.md` §7
"The thermo adapter and upstream vle-thermo gaps", currently at
`~/dev/stages-thermo/PLAN.md`). Its API-mapping audit against vle-thermo **0.8.1**
identified five upstream gaps. This plan closes all five, in this repo, under this
repo's rules. It is the work stages-thermo's roadmap calls
"M4 — Upstream: vle-thermo derivative release (0.9.x, in the vle repo)".

**Placement in the planning hierarchy**: this milestone is **Milestone 12** in
[ROADMAP.md](../../../ROADMAP.md) / [TODO.md](../../../TODO.md) and **Phase 19** in
[MODERNIZATION_PLAN.md](../MODERNIZATION_PLAN.md). Those three documents carry the
milestone/task/phase view; this document carries the full technical detail (the
same division of labor as [PERFORMANCE_PROPOSAL.md](PERFORMANCE_PROPOSAL.md) vs.
its Phase 11 / Tracks A–E entries).

---

## 1. The five gaps (verbatim from the downstream audit)

| # | Need (per column-stage evaluation) | vle-thermo 0.8.1 today | Resolution |
|---|---|---|---|
| G1 | Components for the classic distillation examples | toluene, ethanol, acetone, chloroform, i-C4, i-C5, n-C8+ **absent** (DB has 15 compounds) | **M12.1** — expand `components.json` to 24 compounds |
| G2 | Ideal-gas Cp data reachable from the DB | `Component.cp_coeffs` exists in Rust (`engine/src/types.rs:96`) but the JSON DB has **no Cp fields** and the Python `Component` dataclass (`python/src/vle/components.py:48`) can't carry them | **M12.1** — add `cp_coeffs` to the JSON schema + thread Python → engine |
| G3 | Rust-side component database | **Missing** — JSON DB is Python-side only; Rust consumers hand-build `Component` structs | **M12.2** — vendored JSON + loader behind a `component-db` feature |
| G4 | ∂lnφ̂ᵢ/∂T, ∂lnφ̂ᵢ/∂P, ∂K/∂T, ∂K/∂P | **Missing** — `d_ln_phi_d_n` (`engine/src/mixture.rs:806`) covers composition only; no T/P derivative anywhere in `engine/src/` | **M12.3** — analytic/dual `d_ln_phi_d_t`, `d_ln_phi_d_p`, `k_values_with_derivs` |
| G5 | Real-mixture Cp (∂H/∂T), partial molar enthalpy (∂H/∂nⱼ), packaged γ-φ phase enthalpy | **Missing** — only per-component `ideal_cp` (`engine/src/energy.rs:69`); `phase_enthalpy_entropy` (`energy.rs:487`) is the φ-φ (EOS-departure) path; the γ-φ liquid enthalpy exists only as unassembled building blocks (`ideal_enthalpy_mix` + `excess_h_s`) | **M12.4** — `phase_cp`, `partial_molar_enthalpy`, γ-φ `phase_enthalpy_entropy` |

Why these matter downstream (context for implementation decisions): a rigorous
column solver (Naphtali–Sandholm) does damped full Newton on the MESH equations
with an exact block-tridiagonal Jacobian. The Jacobian blocks need
∂lnK/∂T, ∂lnK/∂x|y (already public via `d_ln_phi_d_n`), ∂H/∂T, and ∂H/∂nⱼ per
stage. Making these **analytic/dual (never finite-difference)** is this repo's own
standing rule (CLAUDE.md *Algorithm Choices*, §L of MODERNIZATION_PLAN.md) — this
milestone extends the existing §L derivative architecture from composition to
temperature and pressure.

## 2. Current-state audit (verified 2026-07-05 against the working tree at v0.8.1)

Facts the implementer can rely on (re-verify signatures before coding; line
numbers drift):

- **K-values**: `pub fn k_values(spec: &SystemSpec, t: f64, p: f64, x: &[f64], y: &[f64]) -> Result<Vec<f64>, FlashError>`
  at `engine/src/flash/system.rs:135`. Dispatches on `spec.liquid`
  (`LiquidModel::Cubic` φ-φ, `LiquidModel::Activity` γ-φ via `gamma_phi_k`,
  `LiquidModel::IdealSolution`, `LiquidModel::ChaoSeader`).
- **Generic mixture core**: `pub fn mixture_params<D: DualNum<f64> + Copy>(spec: &MixtureSpec, t: f64, p: f64, x: &[D]) -> Result<MixtureParams<D>, MixError>`
  at `engine/src/mixture.rs:319`. **T and P are plain `f64`** — the dual machinery
  currently differentiates composition only. Value path `ln_phi_all_generic<D>`
  (`mixture.rs:726`), scalar wrapper `ln_phi_mix` (`mixture.rs:786`).
- **Composition derivatives**: `d_ln_phi_d_n` (`mixture.rs:806`) — analytic branch
  for Classical/IVDW + 2-parameter EOS (`d_ln_phi_d_n_classical`, `mixture.rs:862`),
  dual-sweep branch (one `num_dual::Dual64` column per component,
  `mixture.rs:824-846`) for WS / HV / MHV1 / MHV2 / 3-parameter EOS.
- **Analytic T-derivative building blocks that already exist**:
  `d_alpha_d_tr(eos, tr, comp)` (`engine/src/eos.rs:633`, analytic for all shipped
  variants, also cached on `EosState`); `t_dln_a_dt_mix(spec, t, p, x)`
  (`engine/src/energy.rs:246`, analytic for every mixing rule — classical via
  per-component dαᵢ/dT, GE rules via `T·d(Gᴱ/RT)/dT = −Hᴱ/RT`).
- **Enthalpy layer** (`engine/src/energy.rs`): `ideal_cp` (`:69`),
  `ideal_enthalpy_integral` (`:82`), `ideal_enthalpy_mix` (`:128`),
  `h_departure_rt_mix` (`:182`), `s_departure_r_mix` (`:204`), `excess_h_s`
  (`:508`), `phase_enthalpy_entropy(spec, t, p, x, phase, t_ref, p_ref, h_ref, s_ref)`
  (`:487`). `num-dual` is **not** currently used in `energy.rs`.
- **Component struct**: `engine/src/types.rs:70-138` — all fields `pub`, including
  `cp_coeffs: [f64; 5]` (Cp°/R polynomial: `Cp°(T) = R·Σ aₖ·Tᵏ`, R = 8.31451
  kJ/(kmol·K)) and `psat_coeffs` + `sat_model`.
- **JSON DB**: `python/src/vle/data/components.json` (a synced copy lives at
  `notebooks/data/components.json`), generated by
  `scripts/build_components_json.py`. 15 compounds: methane, ethane, propane,
  n-butane, n-pentane, CO₂, H₂S, benzene, cyclohexane, methylcyclohexane,
  n-hexane, n-heptane, methanol, water, 2-propanol. Per-compound fields:
  `formula, cas, mw, tc, pc, omega, zc, vc, tb, psat_coeffs, psat_source` —
  **no Cp fields**. Loaded by `python/src/vle/components.py` (frozen dataclass
  `Component` at `:48`, loader at `:85`).
- **Bindings**: `engine/src/py_bindings.rs` (free `#[pyfunction]`s) +
  `engine/src/py_system.rs` (the `System` pyclass with `*_batch` numpy methods),
  both behind the `python` feature. The pyclass constructor **already accepts**
  `cp_coeffs: Vec<Vec<f64>>` (`py_system.rs:308`) — but the Python wrapper can
  never populate it from the bundled DB because the dataclass/JSON lack the field.
  `System.enthalpy_entropy` / `enthalpy_entropy_batch` exist (`py_system.rs:607`,
  `:875`) and call the φ-φ `phase_enthalpy_entropy`.
- **Test-oracle house pattern**: inline `#[cfg(test)]` central-difference helpers
  suffixed `_fd` / `_numerical` (`mixture.rs:1263 jac_fd`, `energy.rs:626
  t_dln_a_fd`, `eos.rs:1296 d_alpha_numerical`) validating each analytic/dual
  derivative. Extend this pattern; never ship FD in the production path.
- **Versions**: workspace `0.8.1` (root `Cargo.toml:31`), `python/pyproject.toml`
  `0.8.1`, `num-dual = { version = "0.11", default-features = false }` in
  `engine/Cargo.toml`.

## 3. Sub-milestones and release strategy

Two releases, because G1/G2 gate stages-thermo's very first milestone
(benzene–toluene McCabe–Thiele) while G4/G5 aren't needed until its rigorous
solver:

| Sub-milestone | Content | Ships in | Est. |
|---|---|---|---|
| **M12.1** | Component DB expansion + Cp coefficients (G1 + G2) | **v0.8.2** (fast-track, data + Python only) | 4–6 h |
| **M12.2** | Rust-side component database (G3) | v0.9.0 | 4–6 h |
| **M12.3** | T/P derivatives of fugacity + `k_values_with_derivs` (G4) | v0.9.0 | 8–12 h |
| **M12.4** | Real Cp, partial molar enthalpy, γ-φ phase enthalpy (G5) | v0.9.0 | 6–9 h |
| **M12.5** | Milestone notebook, benches, doc sync, v0.9.0 release | v0.9.0 | 3–5 h |

Total ≈ **25–38 h**. Execution order is 12.1 → 12.5 strictly; 12.4 depends on
12.3's T-generic core; 12.3 and 12.2 are independent of each other.

Version-bump rationale: v0.8.2 is additive data + Python threading (no Rust API
change). v0.9.0 carries the new Rust API surface **and** one deliberate breaking
change to public generic signatures (§6) — a 0.x minor bump is the semver-correct
vehicle for that, and matches the "target 0.9.x" contract stages-thermo pins
against.

---

## 4. Technical design

### M12.1 — Component DB expansion + Cp coefficients (→ v0.8.2)

**New compounds (9, taking the DB from 15 to 24):**

| Compound | Why (downstream validation case) |
|---|---|
| toluene | benzene–toluene — *the* McCabe–Thiele teaching binary (stages M1) |
| ethanol | ethanol–water azeotrope; azeotropic-distillation hard case |
| acetone | acetone–methanol–water extractive case; acetone–chloroform |
| chloroform | acetone–chloroform maximum-boiling azeotrope |
| isobutane (i-C4) | debutanizer / depropanizer literature columns (Seader & Henley Ch. 10) |
| isopentane (i-C5) | same |
| n-octane | wide-boiling absorber / naphtha cases |
| n-nonane | same |
| n-decane | same (absorber oil) |

**Schema addition (all 24 compounds, not just the new 9):** per-compound keys

- `cp_coeffs` — 5-entry list, the **dimensionless Cp°/R polynomial in T [K]**
  matching the engine convention `Cp°(T) = R·Σₖ aₖ·Tᵏ` exactly
  (`engine/src/energy.rs:69`, R = 8.31451 kJ/(kmol·K)). Sources publish Cp° in
  J/(mol·K) or cal-based polynomials — **conversion to Cp°/R is part of the work
  and must be verified numerically**, not eyeballed.
- `cp_t_range` — `[t_min, t_max]` in K, the fit's validity window.
- `cp_source` — provenance string (same spirit as the existing `psat_source`).

**Implementation steps:**

1. Extend `scripts/build_components_json.py` to emit the new compounds and the Cp
   fields (data via the `thermo` library / DIPPR, exactly as the original 15 were
   sourced; run with `~/miniconda3/envs/vle/bin/python`). Cross-check Tc/Pc/ω/Tb
   of every new compound against Poling, Prausnitz & O'Connell 5th ed. (new
   reference (30) in MODERNIZATION_PLAN.md).
2. Regenerate **both** JSON copies (`python/src/vle/data/components.json`,
   `notebooks/data/components.json`) from the script — never hand-edit the JSON.
3. Python threading: add `cp_coeffs: list[float]`, `cp_t_range`, `cp_source` to
   the frozen `Component` dataclass (`python/src/vle/components.py`), populate in
   `_to_component`, and pass `cp_coeffs` through `vle.System` construction into
   the engine pyclass (`system.py` → `_engine.System(cp_coeffs=...)` — the
   parameter already exists at `engine/src/py_system.rs:308`; today it is silently
   left empty, which makes every enthalpy that includes the ideal part wrong for
   DB-built systems. Fixing that silent zero is the point of this threading).
4. Extend the SQLite side for parity: `vle-db` static seed
   (`python/src/vle/db/seed.py`) grows the same 9 compounds so `vle-db show/list`
   and the JSON DB tell one story. (`vle-db validate chapter4` is untouched —
   the Chapter IV set is a subset of the original 15.)
5. **Tests** (`python/tests/`):
   - every compound: `psat(tb) ≈ 101.325 kPa` within 1% (existing round-trip
     pattern), `cp_coeffs` present, Cp°(298.15 K) within 1% of a pinned
     literature value per compound (pin the values in the test, cite the source
     in a comment);
   - `System(["benzene", "toluene"], ...)` smoke: bubble-T at 101.325 kPa for
     x = [0.5, 0.5] lands in the physically sensible 355–370 K band (guards the
     new Antoine data end-to-end);
   - `enthalpy_entropy` of a DB-built system now includes a nonzero ideal-Cp
     contribution (regression test for the silent-zero fix).
6. Docs: `docs/en/parameters/parameter_reference.md` (note Cp°/R polynomials now
   bundled), `python/README.md` compound count if stated, README feature list if
   it names the DB size.
7. Release: bump both versions to **0.8.2**, tag per PUBLISHING.md. This is the
   "tiny vle-thermo PR first" that stages-thermo M1 explicitly waits on.

### M12.2 — Rust-side component database (G3)

**Design:** vendored JSON + lazy parsed lookup, feature-gated so the core crate
keeps zero new mandatory dependencies.

- Canonical data file moves to **`engine/data/components.json`** (inside the
  crate directory so `cargo package` ships it; embedded via `include_str!`).
  `scripts/build_components_json.py` becomes the single generator for **all
  three** copies (engine, python wheel, notebooks). Add a test on the Python side
  asserting the engine and wheel copies are byte-identical (cheap drift guard;
  CI runs pytest already).
- New module `engine/src/db.rs` behind a new cargo feature **`component-db`**
  (pulls optional `serde`/`serde_json`; default **off** — downstream opts in;
  the `python` feature enables it so the bindings can expose lookups).
- API (all returning canonical-unit `Component`s, docstrings state units per the
  CLAUDE.md rule):
  ```rust
  /// Look up a bundled component by name (case-insensitive; common aliases).
  pub fn component(name: &str) -> Option<Component>;
  /// Names of all bundled components, sorted.
  pub fn available() -> Vec<String>;
  ```
  Backed by a `std::sync::OnceLock<HashMap<String, Component>>` parsed on first
  use. Name normalization mirrors `python/src/vle/components.py` (lowercase,
  and the same alias set the Python `get` accepts — read that module and match
  its behavior, don't invent a second convention).
- JSON → `Component` mapping fills `cp_coeffs` (from M12.1's new field),
  `psat_coeffs` + `sat_model = Antoine`, and leaves fields the JSON doesn't carry
  at their `Component::default()` values — document which fields those are in the
  module doc comment.
- **PyO3 (same-commit rule)**: `#[pyfunction] db_component(name) -> PyResult<...>`
  and `db_available()` in `py_bindings.rs` (guarded on the feature), plus a test
  in `python/tests/` asserting `db_component("toluene").tc` matches the JSON.
  The existing pure-Python loader stays the wheel's primary path (no churn);
  the binding exists so the Rust DB is testable through the wheel.
- Rust tests: lookup hit/miss/alias/case, all 24 parse, spot-check tc/pc/ω of
  one legacy and one new compound against the JSON literals.

### M12.3 — T/P derivatives of fugacity and K-values (G4)

This is the heart of the release. Strategy mirrors §L exactly: **analytic where
closed forms are cheap (classical rules + 2-parameter EOS), dual-number AD
everywhere else, FD only as test oracle.**

**Step 1 — make the generic core generic in T and P.** Change

```rust
pub fn mixture_params<D: DualNum<f64> + Copy>(spec, t: f64, p: f64, x: &[D])   // today
pub fn mixture_params<D: DualNum<f64> + Copy>(spec, t: D,  p: D,  x: &[D])     // after
```

and the same for `ln_phi_all_generic<D>` (`mixture.rs:726`) and every helper on
the path (`three_param_uw<D>` at `mixture.rs:590`, the GE `ge_terms` path in
`engine/src/activity.rs` — Wilson's Λᵢⱼ(T), MHV/HV/WS assembly, and
`chao_seader_ln_phi` if feasible; if Chao-Seader resists genericization, give it
hand-analytic dlnφ/dT, dlnφ/dP — it is a closed-form polynomial in (Tr, Pr), so
both routes are mechanical). α functions gain a generic evaluation path so duals
propagate through α(Tr): either `alpha_generic<D>(eos, tr: D, comp)` beside the
existing `alpha`/`d_alpha_d_tr`, or (equivalent and cheaper) assemble the dual's
derivative slot directly from the existing analytic `d_alpha_d_tr` — implementer's
choice, but **do not** finite-difference α. Existing scalar callers pass `t`/`p`
unchanged (`f64: DualNum<f64>`), so call-site churn is type-inference-only; this
is nonetheless a **public-signature break** — see §6.

**Step 2 — new public derivative functions** in `mixture.rs`, mirroring
`d_ln_phi_d_n`'s two-branch dispatch:

```rust
/// ∂ln φ̂ᵢ/∂T at constant P, composition. T in **K**; result in **1/K**.
pub fn d_ln_phi_d_t(spec: &MixtureSpec, t: f64, p: f64, x: &[f64], phase: PhaseId)
    -> Result<Vec<f64>, MixError>;
/// ∂ln φ̂ᵢ/∂P at constant T, composition. P in **kPa**; result in **1/kPa**.
pub fn d_ln_phi_d_p(spec: &MixtureSpec, t: f64, p: f64, x: &[f64], phase: PhaseId)
    -> Result<Vec<f64>, MixError>;
```

- **Analytic branch** (Classical/IVDW + 2-parameter EOS, same gate as
  `d_ln_phi_d_n`'s at `mixture.rs:813`): differentiate the closed-form lnφ̂ᵢ
  through (A, B, āᵢ, b̄ᵢ, Z). dA/dT comes from the existing `d_alpha_d_tr` /
  `t_dln_a_dt_mix` machinery; dZ/dT and dZ/dP by implicit differentiation of the
  cubic, `dZ = −(∂f/∂T|Z)/(∂f/∂Z)` (guard the near-critical `∂f/∂Z → 0` case the
  same way the §H cubic robustness work does — fall back to the dual branch if
  the pivot is tiny, and say so in a comment).
- **Dual branch** (everything else): one extra evaluation of
  `ln_phi_all_generic::<Dual64>` with the dual seed on T (or P) and real
  composition. Cost ≈ 2× a scalar lnφ call — negligible next to a flash.
- **Pressure identity as cross-check**: ∂lnφ̂ᵢ/∂P = V̄ᵢ/(RT) − 1/P. Whether the
  production `d_ln_phi_d_p` is the identity or the dual/analytic route is the
  implementer's call; the *other* route becomes a unit test.

**Step 3 — packaged K-value derivatives** in `flash/system.rs`:

```rust
pub struct KValueDerivs {
    pub k: Vec<f64>,            // Kᵢ = yᵢ/xᵢ at (T, P, x, y)
    pub d_ln_k_d_t: Vec<f64>,   // ∂ln Kᵢ/∂T  [1/K]
    pub d_ln_k_d_p: Vec<f64>,   // ∂ln Kᵢ/∂P  [1/kPa]
}
pub fn k_values_with_derivs(spec: &SystemSpec, t: f64, p: f64, x: &[f64], y: &[f64])
    -> Result<KValueDerivs, FlashError>;
```

- φ-φ: `∂lnKᵢ/∂T = ∂lnφ̂ᵢᴸ/∂T − ∂lnφ̂ᵢⱽ/∂T` (same for P).
- γ-φ: differentiate **term-by-term exactly what `gamma_phi_k` assembles** (read
  that function; keep the two in lockstep — a shared helper is better than a
  parallel reimplementation). The pieces: ∂lnγᵢ/∂T = −H̄ᵢᴱ/(RT²) (analytic via the
  §E machinery where closed forms exist — Wilson — dual over the T-generic
  activity path otherwise); dlnPsatᵢ/dT = (dPsat/dT)/Psat (exists:
  `saturation::d_psat_dt`, analytic for Antoine); the Poynting factor's T and P
  derivatives (closed form, `exp[V_L(P−Psat)/RT]`); vapor-side ∂lnφ̂ᵢⱽ from step 2;
  the explicit −ln P term for ∂/∂P.
- Composition derivatives of K are deliberately **not** in `KValueDerivs` — they
  already exist as `d_ln_phi_d_n` per phase and cost O(n) dual sweeps; callers
  who need the full Jacobian block assemble it themselves. Document that in the
  doc comment with the assembly formula.
- Optional-but-cheap: a `k_values_with_derivs_batch` numpy binding following the
  `py_system.rs:642` pattern (rayon + `allow_threads`). Include if time permits;
  the scalar binding is the hard requirement.

**Step 4 — tests** (the invariants are the real spec; all use pinned mixtures
from the existing test set — methane/ethane PR, methanol/water van Laar,
CO₂/n-butane WS, a Patel-Teja ternary):

1. FD oracles per house pattern: `d_ln_phi_d_t` / `d_ln_phi_d_p` vs central
   differences across the EOS × rule matrix (analytic branch AND dual branch),
   rel. tol ~1e-6.
2. **Gibbs–Helmholtz consistency**: `Σᵢ xᵢ·∂lnφ̂ᵢ/∂T = −H^R/(RT²)` against the
   independent analytic `h_departure_rt_mix` — this catches sign and assembly
   errors FD oracles can miss.
3. **Volumetric consistency**: `Σᵢ xᵢ·∂lnφ̂ᵢ/∂P = (Z−1)/P` against `z_factor`.
4. `k_values_with_derivs.k` ≡ `k_values` bit-for-bit on the same inputs; the
   derivative fields vs FD of `k_values` in T and P, φ-φ **and** γ-φ paths.
5. Python-side: bindings exercised through the wheel
   (`python/tests/test_m12_derivatives.py`), including one γ-φ case.

### M12.4 — Real Cp, partial molar enthalpy, packaged γ-φ enthalpy (G5)

Everything here rides on M12.3's T-generic core plus two thermodynamic
identities — implement via the identities, not new differentiation machinery:

- **Partial molar residual enthalpy**: `H̄ᵢ^R = −RT²·∂lnφ̂ᵢ/∂T`. So:
  ```rust
  /// Partial molar enthalpy H̄ᵢ of one phase, **kJ/kmol**.
  pub fn partial_molar_enthalpy(spec: &MixtureSpec, t: f64, p: f64, x: &[f64],
      phase: PhaseId, t_ref: f64, h_ref: &[f64]) -> Result<Vec<f64>, MixError>;
  ```
  = per-component ideal `h°ᵢ(T)` (existing `ideal_enthalpy_integral` + `h_ref`)
  − RT²·`d_ln_phi_d_t`. One call, exact, no new derivative code.
- **Real-mixture isobaric heat capacity**:
  ```rust
  /// Isobaric heat capacity Cp of one phase, **kJ/(kmol·K)**.
  pub fn phase_cp(spec: &MixtureSpec, t: f64, p: f64, x: &[f64], phase: PhaseId)
      -> Result<f64, MixError>;
  ```
  = Σxᵢ·`ideal_cp` + Cp^R, where Cp^R = ∂H^R/∂T needs a **second** T-derivative
  of G^R/RT. Route: second-order duals (`num_dual::Dual2_64`) through the
  T-generic `ln_phi_all_generic` — one evaluation yields H^R and Cp^R together.
  **Verify first** that num-dual 0.11 with `default-features = false` exposes a
  second-order scalar under rust-version 1.85; if not, the fallback is analytic
  d²α/dTr² for the 2-parameter classical branch (mechanical: the α forms are
  elementary) + central-FD *of the analytic first derivative* as oracle-only,
  with the dual path revisited when the dependency allows. Do not ship FD.
- **Packaged γ-φ phase enthalpy/entropy** — the assembly stages-thermo would
  otherwise do in its adapter, done once here instead. New function in
  `flash/system.rs` (it needs `SystemSpec` to know the liquid model, mirroring
  `k_values`' dispatch):
  ```rust
  /// H and S of one phase under the system's model pair (φ-φ or γ-φ).
  /// Returns (**kJ/kmol**, **kJ/(kmol·K)**).
  pub fn phase_enthalpy_entropy(spec: &SystemSpec, t: f64, p: f64, comp: &[f64],
      phase: PhaseId, t_ref: f64, p_ref: f64, h_ref: &[f64], s_ref: &[f64])
      -> Result<(f64, f64), FlashError>;
  ```
  - Vapor / φ-φ liquid: delegate to the existing `energy::phase_enthalpy_entropy`.
  - γ-φ liquid: `H_L = Σxᵢ[h°ᵢ(T) − ΔH_vap,ᵢ(T)] + Hᴱ(x, T)` with the
    condensation enthalpy via Clausius–Clapeyron from the saturation layer's
    `d_psat_dt` (this is the Pascal `TERMOIII.PAS:283`/`294` path that Phase 14
    explicitly deferred — cite `// Ref (4): Da Silva & Báez (1989),
    legacy/pascal/TERMOIII.PAS` and read the legacy source before implementing;
    the entropy analog assembles the same way). Hᴱ/Sᴱ from the existing
    `excess_h_s`. Reference-state discipline: same `t_ref/p_ref/h_ref/s_ref`
    convention as the φ-φ path — one convention across both branches, asserted
    by a test that the two paths agree for an ideal system where they must.
- **PyO3 (same-commit rule)**: `System.phase_cp`, `System.partial_molar_enthalpy`,
  and route the existing `System.enthalpy_entropy` through the new
  SystemSpec-level function so γ-φ systems stop silently getting the φ-φ liquid
  path (check what it does today for `LiquidModel::Activity` and preserve/fix
  deliberately — this is a behavior change to document in the release notes).
  Batch variants optional, scalar mandatory. Tests through the wheel.
- **Tests**:
  1. Euler consistency: `Σxᵢ·H̄ᵢ = H` vs `phase_enthalpy_entropy` (machine-ish
     tolerance — both sides are analytic).
  2. `phase_cp` vs central-FD of the analytic H(T) (oracle), plus the pure-
     component ideal-gas limit (`phase_cp → Σxᵢ·ideal_cp` as P → 0).
  3. γ-φ liquid enthalpy: methanol/water van Laar against a hand-assembled
     `ideal + condensation + excess` value in the test (pin the number), and a
     γ-φ vs φ-φ sanity delta on a mildly nonideal system.

### M12.5 — Notebook, benches, docs, v0.9.0 release

- **Milestone notebook** `notebooks/11_derivatives_and_database.ipynb`
  (deterministic build script `scripts/build_notebook_m12.py`, per house
  pattern), following CLAUDE.md *Notebook Conventions* (setup cell, research-
  paper context — Chapter II §2.3's K-value/enthalpy framework is the anchor —
  worked example, ≥2 exercises with collapsed solutions, pinned assertions):
  - tour of the 24-compound DB (new compounds, Cp data);
  - K(T) sensitivity: plot lnKᵢ(T) for a binary with the tangent line from
    `d_ln_k_d_t` drawn at a point — the "derivatives are exact" money shot;
  - mixture Cp vs T (ideal vs real, pressure family);
  - partial-molar-enthalpy bar chart + the Euler-sum assertion cell;
  - a "why this milestone exists" markdown cell pointing at downstream
    staged-separation work.
- **Benches**: extend `engine/benches/engine_bench.rs` with
  `k_values_with_derivs` vs `k_values` (target: analytic branch ≤1.5×, dual
  branch ≤2.5× a plain `k_values` call) and `phase_cp`.
- **Doc sync (the CLAUDE.md pre-push list, in full)**: README (features, DB
  size, doc map), ROADMAP/TODO check-offs + model-attribution lines,
  MODERNIZATION_PLAN Phase 19 status, `python/README.md` + `engine/README.md`
  (new API story; remember both are immutable-per-published-version — they ship
  with the 0.9.0 release, so their snippets must run against the 0.9.0 wheel /
  compile against the 0.9.0 crate **before** tagging), `deploy/NOTEBOOKS.md`
  catalogue row for notebook 11, parameter reference (Cp section).
- Bump workspace + pyproject to **0.9.0**, tag `v0.9.0` per PUBLISHING.md
  (idempotent pipeline; YubiKey-signed tag by Miguel).

---

## 5. What is deliberately NOT in this milestone

- **γ-φ / non-cubic PH (adiabatic) flash** — today `flash_adiabatic` is
  cubic-only; M12.4's γ-φ enthalpy makes extending it *possible*, but downstream
  doesn't need it for v1 and it drags in flash-loop work. Follow-on candidate.
- **∂K/∂x packaging** — already available via `d_ln_phi_d_n` per phase; no new
  wrapper.
- **Cp coefficients for the optional ~70K-compound `thermo` seeding path** — the
  bundled 24 get curated Cp data; the bulk path stays as-is.
- **DB expansion beyond the 24** — nitrogen/oxygen/etc. can ride any later
  release; keep this one scoped to what downstream validation cases name.

## 6. Breaking-change register (for the 0.9.0 release notes)

1. `mixture_params<D>` / `ln_phi_all_generic<D>` (and generic helpers on that
   path) change `t: f64, p: f64` → `t: D, p: D`. Scalar callers are source-
   compatible in practice (`f64` satisfies the bound; inference handles it), but
   it is a public-signature change. Enumerate actual breakage while implementing
   (the compiler is the auditor: internal call sites live in `mixture.rs`,
   `energy.rs`, `flash/critical.rs`, `activity.rs`).
2. `System.enthalpy_entropy` behavior for γ-φ systems changes if it currently
   applies the φ-φ liquid path (verify, then document).
3. Everything else is additive.

## 7. Risks & open questions

1. **num-dual 0.11 second-order duals** under `default-features = false` +
   rust-version 1.85 — verify before building M12.4's Cp on it (fallback in
   §M12.4). Low risk, checked in an hour.
2. **Cp°/R unit conversion errors** are the classic silent-data bug (J/mol/K vs
   cal vs Cp/R; T vs T/1000 polynomial variables across sources). Mitigation:
   the pinned Cp°(298.15 K) literature assertions per compound — treat a failing
   pin as data, not tolerance, until proven otherwise.
3. **Antoine quality for the heavy tail** (n-C9, n-C10) — reduced-Antoine fits
   degrade far from Tb; the `psat(tb) ≈ 1 atm` gate catches gross errors, but
   record each fit's T-window in `psat_source` like the existing entries do.
4. **Near-critical dZ/dT in the analytic branch** (∂f/∂Z → 0) — guarded fallback
   to the dual branch, per M12.3 step 2.
5. **γ-φ K-derivative / `gamma_phi_k` drift** — the derivative assembly must
   mirror the value assembly term-for-term; prefer refactoring `gamma_phi_k` so
   both share one term list over writing a parallel copy.
6. **Downstream co-development loop**: stages-thermo builds against the
   *published* crate (its CI rule). If mid-milestone testing needs unpublished
   API, downstream uses a local `[patch.crates-io]` — nothing in this repo
   changes for that.

### Findings surfaced during M12.3

- **Wong-Sandler departure-enthalpy discrepancy (pre-existing — FIXED
  post-M12.5, 2026-07-06).** M12.3's Gibbs–Helmholtz invariant
  (`Σxᵢ ∂lnφ̂ᵢ/∂T = −H^R/(RT²)`) held to machine precision for the classical
  cubic and the Huron-Vidal / MHV1 / MHV2 GE rules, but Wong-Sandler's
  `h_departure_rt_mix` was ~1% inconsistent with the exact `ln φ̂ᵢ(T)` (≈1e-4
  vapor, ≈1.7e-3 liquid at the tested state). **Root cause** (not the one
  originally suspected): `t_dln_a_dt_mix`'s WS branch was *correct* — its
  `T·d(ln A_mix)/dT` matches both the FD oracle and a dual-AD sweep of
  `mixture_params` to machine precision. The bug was in the **departure-
  enthalpy formula itself**, which assumed `T·d(ln B_mix)/dT = −1` (i.e. a
  T-independent dimensional co-volume). That holds for every linear-b rule,
  but WS's `b_mix = Q̃(T)/(1−D̃(T))` drifts with temperature, so the formula
  silently dropped the `db/dT` term. **Fix:** `t_dln_ab_dt_mix` now also
  returns `T·d(ln B_mix)/dT` (analytic; `−1` for all rules except WS), and
  `h_departure_rt_mix` applies the correction
  `−δ·[(Z−1) + A·Ĩ]` with `δ = T·d(ln b_mix)/dT`, using the EOS-root identity
  `B/(Z−B) − A·B·∂Ĩ/∂B = (Z−1) + A·Ĩ` to avoid per-(U,W)-branch `∂Ĩ/∂B`
  algebra. The Gibbs–Helmholtz identity now holds to ~1e-15 for **all** rules
  including WS; the tracking test flipped to the regression test
  `mixture::tests::wong_sandler_departure_enthalpy_matches_gibbs_helmholtz`,
  and `gibbs_helmholtz_identity_vs_departure_enthalpy` covers WS too.
  `s_departure_r_mix` (H − G) inherits the fix; Chapter IV validation
  (classical rules, δ = 0 exactly) is untouched. M12.4's
  `partial_molar_enthalpy` was never affected (built on `d_ln_phi_d_t`).

### Deferred within M12.3 (implementer's-choice latitude in §4)

- **Analytic fast-path for `d_ln_phi_d_t` / `d_ln_phi_d_p`.** The plan permits
  "dual-number AD everywhere else"; both derivatives ship as the **dual branch
  universally** (exact for every rule/EOS, ≈2× a scalar `ln φ̂` call). The
  hand-analytic closed form for classical + 2-parameter EOS (differentiating
  through the implicit `dZ/dT` with the near-critical `∂f/∂Z → 0` pivot guard)
  is a pure performance optimization and is deferred — the invariant tests pin
  correctness independently of the route, and the dual cost is negligible next
  to a flash. Revisit if a bench shows the T/P-derivative path is hot.

## 8. Execution rules that bind this milestone (read before starting)

All standing CLAUDE.md rules apply; the ones most easily violated here:

- **Reiterate the milestone** (goal + task list) before implementing, on
  "execute Milestone 12.x".
- **PyO3 bindings in the same commit series** as each new public Rust function,
  with at least one wheel-level test each.
- **Units in every docstring** (Rust + Python) — K, kPa abs, kJ/kmol,
  kJ/(kmol·K), 1/K, 1/kPa as appropriate.
- **Educational comments** (beginner-friendly Rust idiom notes) + **citation
  comments**: (26) Michelsen & Mollerup for the derivative identities, (27)
  Rehner & Bauer for dual-number AD, (4) Da Silva & Báez for the condensation-
  enthalpy path, (30) Poling/Prausnitz/O'Connell for property data.
- **Analytic/dual only** — FD survives exclusively as `_fd` test oracles.
- Conda env: `~/miniconda3/envs/vle/bin/{python,maturin,pytest,jupyter}` only.
- `cargo fmt --check` before every push; YubiKey signing flow (Miguel commits
  and tags; Claude prepares messages in `/tmp/*.txt` and pushes).
- Model-attribution lines in ROADMAP/TODO on completion of each sub-milestone.
