# Plan: γ-φ Heat Capacity + Dual-Generic Saturation Derivatives (Milestone 12.6 → v0.16.0)

*Adopted 2026-08-18 (Claude Fable 5), at Miguel's request, as the last
downstream-gap item of the derivative release
([`DERIVATIVE_RELEASE_PLAN.md`](DERIVATIVE_RELEASE_PLAN.md) G5): the packaged
`phase_cp` shipped in M12.4 covers cubic (φ-φ) phases only, and stages-thermo
M5 (v0.4.0, 2026-08-18) had to fill the γ-φ liquid Cp with a central finite
difference in its adapter — the exact interim the derivative release exists to
retire. Small, self-contained, and independently valuable to vle-thermo's own
users; scheduled before stages-thermo M6.*

## 1. The gap, precisely

`ThermoProvider::dh_dt` (stages-thermo, `engine/src/thermo.rs`) is the phase
heat capacity a column solver needs for the temperature column of its enthalpy
balances (Naphtali–Sandholm Jacobian, inside-out's enthalpy-surrogate fit).
Today:

| Route | `∂ln K/∂T` | `∂H/∂T = Cp` |
|---|---|---|
| φ-φ (cubic both phases) | analytic — `k_values_with_derivs` (M12.3) | analytic — `energy::phase_cp` (M12.4, `Dual2` through `ln_phi_all_generic`) |
| γ-φ (activity liquid, ideal-gas vapor) | analytic — term-by-term (M12.3) | **missing upstream** → stages-thermo central FD |

Two smaller upstream facts feed the same fix:

- `saturation::d_psat_dt` is **analytic for Antoine only**; for Riedel /
  Müller / RPM / Polynomial it is a central difference (kept from the legacy
  `DPrVapor_DT`, TERMOI.PAS:236). It feeds `k_values_with_derivs`' γ-φ branch
  and the γ-φ enthalpy's condensation term, so those are "analytic" only for
  Antoine components (which is what the built-in DB ships — but pseudocomponents
  and users' own components may not be).
- `activity::excess_enthalpy` is a **per-model convention**, not one formula:
  Scatchard–Hildebrand Hᴱ = Gᴱ (T-independent), Margules / van Laar Hᴱ = Gᴱ
  (the legacy convention, Sᴱ = 0), Wilson analytic with Λ(T), NRTL by
  Gibbs–Helmholtz through a dual. A γ-φ Cp that is *consistent* with the
  shipped H (so that `Cp = dH/dT` to round-off, which is what any FD-oracle
  test asserts and what a Newton solver assumes) must differentiate **each
  model's own Hᴱ function**, not a fresh Gibbs–Helmholtz on all of them.

## 2. Design

### 2.1 Dual-generic saturation pressure (`saturation.rs`)

`psat_generic<D: DualNum<f64> + Copy>(model, comp, t: D) -> Result<D, SatError>`
for Antoine, Riedel, Müller, RPM and Polynomial — every one is elementary in
T (`ln`, `exp`, `powi`, `powf`, division), the same M12.3 pattern as
`ln_gamma_all_generic`. The `f64` entry points (`psat`, `psat_antoine`, …)
become thin wrappers with `D = f64` (`f64: DualNum<f64>`), so no numeric
change for callers. Then:

- `d_psat_dt` → one `Dual64` evaluation, **analytic for every model** (the
  central difference goes; Antoine's hand-written derivative stays as the
  test oracle for the dual path, and the old FD as the oracle for the others).
- **new** `d2_psat_dt2(model, comp, t)` → one `Dual2_64` evaluation
  returning `(Psat, dPsat/dT, d²Psat/dT²)`; and the derived
  `condensation_cp(model, comp, t)`:

  ```text
  ΔH_vap(T) = R T² p'/p                       (Clausius–Clapeyron, as shipped)
  d ΔH_vap / dT = R [ 2T p'/p + T² (p'' p − p'²)/p² ]
  ```

  `Maxwell` stays unsupported (it needs an EOS root loop, not a formula).

### 2.2 Excess heat capacity (`activity.rs`)

`excess_cp(model, x, aij, alpha, vl, delta, t) -> f64` = `d(excess_enthalpy)/dT`
**per model, of the shipped Hᴱ**:

| Model | Hᴱ as shipped | Cpᴱ |
|---|---|---|
| IdealSolution | 0 | 0 |
| ScatchardHildebrand | Σ xⱼVⱼ(δⱼ−δ)² (no T) | 0 |
| Margules, VanLaar | Gᴱ(T) = RT·g(T), g = Σxᵢ ln γᵢ | `Dual64` on g: R (g + T g′) — for T-independent ln γ this is Gᴱ/T |
| Wilson | analytic Σⱼ xⱼ (Σₖ xₖ aⱼₖ Λⱼₖ)/(Σₖ xₖ Λⱼₖ) | `Dual64` through a `D`-generic copy of that expression (`wilson_lambda_generic` exists) |
| NRTL | −RT² g′(T) | `Dual2_64` on g: −R (2T g′ + T² g″) — the `residual_cp` identity |

### 2.3 Packaged γ-φ Cp (`flash/system.rs`)

`pub fn phase_cp(spec: &SystemSpec, t, p, comp, phase) -> Result<f64, FlashError>`
next to `phase_enthalpy_entropy`, same dispatch:

- cubic phase → `energy::phase_cp(mixture_spec)` (unchanged);
- ideal-gas vapor → Σ yᵢ Cp°ᵢ(T);
- γ-φ liquid → `Σxᵢ Cp°ᵢ − Σxᵢ (dΔH_vap,ᵢ/dT) + Cpᴱ` — the term-by-term
  derivative of the shipped `H_L = H_ig − ΣxᵢΔH_vap,ᵢ + Hᴱ`;
- Virial vapor / Chao–Seader-family liquid → `Unsupported` (as for H).

### 2.4 Python (same-commit rule)

`System.phase_cp` is **re-routed** through the SystemSpec-level function, so a
γ-φ system returns a number instead of "needs a cubic model on that phase" —
a documented behaviour change (release notes), same shape as M12.4's
re-routing of `enthalpy_entropy`. New `System.d_psat_dt(i, t)` /
`d2_psat_dt2(i, t)` are **not** added (no downstream consumer); the
saturation derivatives stay Rust-level. Batch variant of `phase_cp` optional.

### 2.5 Downstream (stages-thermo, after the release)

Bump the pin to 0.16, replace the `ModelKind::GammaPhi` arm of
`ThermoSystem::dh_dt` with `vle_thermo::flash::phase_cp` (~10 lines), keep
`dh_dt_fd` and its test as the permanent oracle, and delete the "FD-interim"
notes in `thermo.rs`, `CLAUDE.md` and `docs/theory/mesh.md`.

## 3. Tests

1. `psat_generic` with `D = f64` reproduces every existing `psat_*` value
   bit-for-bit (the wrappers *are* the generic function).
2. `d_psat_dt` (dual) vs the Antoine closed form to 1e-12 relative; vs the
   old central difference for Riedel / Müller / RPM / Polynomial to 1e-6.
3. `d2_psat_dt2` vs central FD *of the analytic first derivative*, per model.
4. `excess_cp` vs central FD of `excess_enthalpy`, **per activity model**
   (van Laar, Margules, Wilson, NRTL, Scatchard–Hildebrand, ideal), so the
   per-model convention is pinned by a test and cannot drift.
5. `flash::phase_cp` γ-φ liquid vs central FD of `phase_enthalpy_entropy`'s H
   (methanol/water van Laar; ethanol/water NRTL) to 1e-6 relative; ideal-gas
   vapor equals Σ y Cp°; φ-φ path unchanged (existing tests).
6. Wheel: `System.phase_cp` on a γ-φ system is finite, positive, and equals
   the FD of `System.enthalpy_entropy` — and still equals the old value on a
   φ-φ system.

## 4. Deliverables and docs

- Code: §2.1–2.3, PyO3 §2.4, tests §3.
- Notebook: a **γ-φ Cp section appended to `11_derivatives_and_database.ipynb`**
  (not a new notebook — Cp belongs to the derivative story; collection count
  unchanged), executed in a fresh kernel.
- ROADMAP/TODO: **Milestone 12.6** sub-section under Milestone 12 (the
  13.7/13.8 precedent); `docs/plans/README.md` row; `DERIVATIVE_RELEASE_PLAN.md`
  §5/§7 gain a pointer to this plan; `python/README.md` version history +
  `engine/README.md` snippet `"0.16"`; root README latest-release line.
- Release **v0.16.0** (new public API → minor).

## 5. What is deliberately not here

- No γ-φ **partial molar** enthalpy or Cp (no consumer).
- No Maxwell-construction derivatives.
- No change to the per-model Hᴱ conventions themselves (that would be a
  thermodynamic-behaviour change to H, out of scope; if it is ever revisited,
  Cpᴱ follows automatically because it differentiates the shipped Hᴱ).
