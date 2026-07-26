# Steam Tables Plan — `vle-steam` (IAPWS-IF97 water/steam properties)

**Status: SHIPPED as Milestone 13 / Phase 20 — released in v0.10.0
(2026-07-07).** This file remains the *design record* — the formulation
choice, the alternatives considered, and the rationale. The as-built state
lives in [ROADMAP.md](../../../ROADMAP.md) Milestone 13 and
[MODERNIZATION_PLAN.md](../MODERNIZATION_PLAN.md) Phase 20; the code is the
[`steam/`](../../../steam) crate (`vle-steam` on crates.io, `vle.steam` in the
wheel). Two later sub-milestones extended it beyond this plan: **13.7**
(transport properties) and **13.8** (the IF97 performance audit — see
[`steam_audit.md`](steam_audit.md)).

> **Correction (2026-07-26).** This header previously read *"PROPOSED — not yet
> scheduled … Nothing in this file has been implemented yet."* That was true when
> the plan was written on 2026-07-07, and was never updated when the work shipped
> the same week. It is the mirror image of the failure described in CLAUDE.md
> *Completion Claims Must Be Verified Against the Code* — a status line that
> stopped describing the code — caught while building
> [the Plan & Audit History](../README.md).

---

## 1. Why steam tables, and which formulation

Steam tables are the single most-used thermodynamic reference in chemical
engineering practice: sizing reboilers and condensers, steam-header balances,
flash-steam recovery, turbine/valve calculations, deaerators, evaporators.
Every printed steam table in a modern handbook is *computed from* one specific
open standard, so we implement that standard directly rather than interpolate
tabulated data.

### Chosen formulation: IAPWS-IF97 (primary)

**IAPWS-IF97** — *"Revised Release on the IAPWS Industrial Formulation 1997
for the Thermodynamic Properties of Water and Steam"*, IAPWS R7-97(2012),
freely available from <http://www.iapws.org/relguide/IF97-Rev.html>. Textbook
form: Wagner & Kretzschmar, *International Steam Tables*, 3rd ed., Springer
(2019).

Why IF97 and not something else:

| Candidate | Verdict |
|---|---|
| **IAPWS-IF97 (1997, rev. 2012)** | ✅ **Primary.** The *industrial* standard — what commercial steam tables, power-plant codes, and process simulators use. Explicit Gibbs equations g(p,T) in the main regions → **no iteration** for the everyday (T,P) query. Closed-form Psat(T) **and** Tsat(P). Official verification tables make testing exact. |
| IAPWS-95 (Wagner & Pruß 2002) | Scientific standard; higher accuracy but Helmholtz f(ρ,T) — every (T,P) query needs density iteration. Planned later as an optional **validation oracle**, not v1. |
| Antoine / correlations already in `engine/saturation.rs` | Far too coarse; no h/s/ρ surface; not a steam table. |
| Wrapping an existing crate (`seuif97`, `rusteam`) | Conflicts with the project's learning purpose and PyO3/units/notebook integration style. We implement from the IAPWS release documents; `seuif97` (thermalogic/RustSEUIF97) is used only as a **dev-dependency cross-check oracle** in tests. |

IF97 validity range: **273.15 K – 2273.15 K**, up to **100 MPa** (50 MPa above
1073.15 K) — covers every conceivable process-engineering use.

### IF97 structure (what actually gets implemented)

| Region | Physical domain | Equation form | Notes |
|---|---|---|---|
| 1 | Compressed/subcooled liquid, 273.15–623.15 K | Gibbs g(p,T), 34 terms | Explicit in (T,P) |
| 2 | Superheated vapor up to 1073.15 K, ≤100 MPa | Gibbs g(p,T) = ideal + residual | Explicit in (T,P) |
| 3 | Near-critical, 623.15–863.15 K inside the B23 line | Helmholtz f(ρ,T) | Needs ρ iteration at given (T,P) — Brent/Newton from `numerics/` patterns |
| 4 | **Saturation line**, 273.15–647.096 K | Quadratic implicit equation | **Closed-form both ways**: Psat(T) and Tsat(P). The heart of the two-phase API |
| 5 | High-T steam 1073.15–2273.15 K, ≤50 MPa | Gibbs g(p,T) | Small; include for completeness |
| — | Backward eqs. T(p,h), T(p,s) for regions 1–2 | Polynomial | Makes PH/PS flash essentially non-iterative |

All properties come from analytical derivatives of g or f (γ_π, γ_τ, γ_ππ,
γ_ττ, γ_πτ …) — fully consistent with the project rule "analytical
derivatives, never finite differences".

---

## 2. Proposed usage modes (the API, from a practitioner's viewpoint)

The three modes Miguel specified, plus the ones a chemical engineer reaches
for daily. Every mode returns a single `SteamState` (Rust) / `Water` state
object (Python) carrying: `T, P, region, phase, x (quality, if two-phase), ρ,
v, u, h, s, cp, cv, w (speed of sound)`.

### Core state constructors

1. **(T, P) → state** — `Water::tp(T, P)`
   Region detection first. Single-phase → full property set. If the point
   lies on the saturation line within tolerance, `phase` reports
   `Saturated` and the state exposes `h_f, h_g, h_fg` (latent heat), but
   quality is *undefined* by (T,P) alone — documented explicitly (classic
   student trap: T and P are not independent inside the dome).
2. **(T, x) → state** — `Water::tx(T, x)`, `0 ≤ x ≤ 1`
   Computes **Psat(T)** (closed form, region 4), then mixes saturated
   liquid/vapor: `h = h_f + x·h_fg`, same for s, v, u. Latent heat always
   reported.
3. **(P, x) → state** — `Water::px(P, x)`
   Computes **Tsat(P)** (closed form), then as above.

### Flash constructors (the practitioner workhorses)

4. **(P, h) → state** — `Water::ph(P, h)`
   *The* mode for throttling valves (isenthalpic), flash drums, and
   condensate flash: if `h_f < h < h_g` at Tsat(P) → two-phase with
   `x = (h − h_f)/h_fg`; otherwise backward equation T(p,h) → single phase.
5. **(P, s) → state** — `Water::ps(P, s)`
   Isentropic turbine/pump calculations and efficiency corrections:
   ideal outlet state at `(P_out, s_in)`, then
   `h_out = h_in − η·(h_in − h_ideal)` → feed back through mode 4.

### Saturation-row queries (the printed-table experience)

6. **`Water::sat_t(T)` / `Water::sat_p(P)`** — one call returns the classic
   steam-table row: `Psat` (or `Tsat`), `v_f, v_g, h_f, h_g, h_fg, s_f, s_g,
   s_fg, u_f, u_g`. This is what you'd print to reproduce a handbook page.

### Convenience helpers

7. `latent_heat(T)` / `latent_heat_at_p(P)` — h_fg directly.
8. `quality_from_h(P, h)` / `quality_from_s(P, s)` — x alone.
9. Consistency helper `psat_derivative(T)` (analytic dPsat/dT from region 4)
   — used internally for the Clausius–Clapeyron test and useful for users
   doing their own derivative work.

### Worked practitioner scenarios (→ milestone notebook)

These become the notebook's worked example + exercises:

- **Flash-steam recovery**: condensate at 10 barg throttled to 1 barg — what
  fraction flashes to steam? (mode 4, gauge-pressure units via the existing
  registry)
- **Reboiler duty**: kg/h of 4 barg saturated steam to deliver Q kW (mode 6:
  h_fg at P).
- **Desuperheating**: water injection rate to bring superheated steam to
  saturation (modes 1 + 6, mass/energy balance).
- **Turbine**: isentropic expansion 40 bar/400 °C → 0.1 bar, η = 0.80 —
  outlet quality and power (modes 1, 5, 4).
- **Deaerator / steam-header balance**: mixing streams by enthalpy.

### Python & batch surface

- `vle.steam.Water(T=..., P=...)` / `(T=..., x=...)` / `(P=..., h=...)` … —
  keyword pairs select the mode; accepts pint quantities and unit strings
  (`"180 degC"`, `"10 barg"` — gauge handled by the existing UnitRegistry,
  never hardcoded atmosphere).
- `vle.steam.sat_table(T_array)` etc. — **batch numpy API** mirroring the
  existing `_batch` design (rust-numpy + rayon, GIL released): arrays of
  (T,P) or (P,h) in → arrays of h, s, ρ, x out. Steam property evaluation is
  exactly the "numpy for thermo" use case.

### Units decision (recommendation)

Steam tables are universally **mass-basis** (kJ/kg, m³/kg, kg/m³) while the
engine's canonical units are molar (kJ/kmol, cm³/mol). Recommendation:

- `vle-steam` public API: **T [K], P [kPa absolute]** (repo canon), properties
  **mass-basis** (kJ/kg, kJ/(kg·K), m³/kg) — what every practitioner expects.
- A `.molar()` view on the state converts via M_water = 18.015268 kg/kmol
  (IAPWS value) for consistency with the engine when mixing worlds.
- Internally the IF97 equations run in their native units (MPa, kJ/kg);
  conversion happens once at the API boundary. Every function documents its
  units per the CLAUDE.md units rule.

---

## 3. Crate & module design

New workspace member: **`steam/`** → crate **`vle-steam`** (published to
crates.io alongside `vle-thermo` and `vle-units`).

Why a separate crate rather than a module inside `engine/`:

- IF97 is a self-contained formulation with zero coupling to the mixture EOS
  machinery; a separate crate keeps `vle-thermo`'s story ("multicomponent
  VLE") clean and gives steam its own crates.io page/README.
- Dependency-free (pure `f64` math — not even nalgebra), which keeps it
  trivially portable to the iOS static-library build (see IOS_FFI_PLAN.md —
  a steam-tables iPhone app is the natural first FFI consumer).
- Mirrors the existing precedent: `engine` already depends on sibling
  `vle-units`.

Wiring:

```
steam/                     # new workspace member
  Cargo.toml               # vle-steam, zero mandatory deps
  README.md                # crates.io page (own release-rule entry in CLAUDE.md)
  src/
    lib.rs                 # public API: Water/SteamState, constructors, errors
    regions.rs             # region detection + B23 boundary
    region1.rs .. region5.rs
    region4.rs             # saturation line (Psat, Tsat, dPsat/dT)
    backward.rs            # T(p,h), T(p,s) for regions 1–2
    coefficients.rs        # the IF97 n_i/I_i/J_i tables, transcribed from R7-97(2012)
```

- `engine/Cargo.toml` gains `vle-steam = { path = "../steam", version, optional = true }`
  and features `steam = ["dep:vle-steam"]`, with `python` extended to include
  `steam` — so the wheel always ships it, while `cargo add vle-thermo` stays
  lean unless the feature is requested. (Alternative considered: expose only
  via the standalone crate. Rejected — the M5+ PyO3 rule requires Python
  parity, and bindings live in `engine/src/py_bindings.rs`.)
- PyO3 bindings: new `engine/src/py_steam.rs` (`SteamState` pyclass + module
  functions + batch kernels), surfaced as `vle.steam` in the wrapper.

## 4. Testing & validation strategy

1. **IAPWS verification tables as exact unit tests** — R7-97(2012) ships
   computer-program verification values for every region (e.g. Table 5 for
   region 1: at T=300 K, p=3 MPa → v=1.00215168×10⁻³ m³/kg,
   h=115.331273 kJ/kg, s=0.392294792 kJ/(kg·K); Table 35: psat(300 K)=
   3.53658941 kPa; Table 36: Tsat(0.1 MPa)=372.755919 K). Transcribe **all**
   table rows into tests; assert to the full published precision (9
   significant figures).
2. **Cross-check oracle**: `seuif97` as a *dev-dependency only*, sweeping a
   (T,P) grid across all regions and comparing h, s, v to tight tolerance.
3. **Thermodynamic-consistency tests** (no external data needed):
   `h = u + p·v` identity; Clausius–Clapeyron `h_fg ≈ T·v_fg·dPsat/dT` with
   the analytic region-4 derivative; round-trips `ph(P, h(tp(T,P))) → T`;
   region-boundary continuity (properties agree across the 1/3, 2/3, 2/5
   seams to IF97's stated tolerances).
4. **Python tests** through the wheel (`python/tests/test_steam.py`), incl.
   pint unit-string inputs, gauge pressure, and batch-vs-scalar agreement.
5. **Criterion bench** — a (T,P) point in each region + one PH flash, guarding
   hot-path regressions like the existing engine benches.

## 5. Phase breakdown & estimates (becomes Milestone 13 on adoption)

| Phase | Content | Est. |
|---|---|---|
| 13.1 | `steam/` crate scaffold; region-4 saturation line (Psat, Tsat, dPsat/dT); region detection + B23 boundary; verification-table tests | 3–5 h |
| 13.2 | Regions 1 & 2 (Gibbs + all property derivatives incl. cp, cv, w); full Table 5/15 tests | 6–8 h |
| 13.3 | Region 3 (Helmholtz, ρ-iteration via Brent per repo algorithm rules) + region 5; boundary-continuity tests | 5–7 h |
| 13.4 | State API (`tp/tx/px/ph/ps/sat_t/sat_p`), backward T(p,h)/T(p,s), quality logic, seuif97 oracle sweep, consistency tests | 5–7 h |
| 13.5 | PyO3 bindings + `vle.steam` wrapper + batch numpy kernels + pint units + gauge support | 4–6 h |
| 13.6 | Milestone notebook (practitioner scenarios above, per Notebook Conventions), `steam/README.md`, docs, benches; release **v0.10.0** | 4–6 h |
| 13.7 *(optional, later)* | Transport properties (viscosity IAPWS R12-08, thermal conductivity R15-11, surface tension R1-76) + IAPWS-95 as high-accuracy oracle | 8–12 h |

**Total (13.1–13.6): ~27–39 h.**

## 6. References to add on adoption (ACS style, MODERNIZATION_PLAN list)

- IAPWS. *Revised Release on the IAPWS Industrial Formulation 1997 for the
  Thermodynamic Properties of Water and Steam*; IAPWS R7-97(2012); IAPWS, 2012.
- Wagner, W.; Kretzschmar, H.-J. *International Steam Tables*, 3rd ed.;
  Springer: Berlin, 2019.
- Wagner, W.; Pruß, A. The IAPWS Formulation 1995 for the Thermodynamic
  Properties of Ordinary Water Substance for General and Scientific Use.
  *J. Phys. Chem. Ref. Data* **2002**, *31*, 387–535. *(only if/when 13.7 lands)*

## 7. Open decisions (recommendations made, Miguel to confirm)

1. **Separate `vle-steam` crate** vs. engine module → recommended: separate
   crate (reasons in §3).
2. **Mass-basis primary units** with `.molar()` view → recommended (§2).
3. **Version**: ship as **v0.10.0** (new public API surface = minor bump).
4. **Sequencing**: implement steam (M13) *before* the iOS FFI (M14) so the
   first iOS deliverable can be a steam-table app — see IOS_FFI_PLAN.md.
