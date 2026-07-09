# Mixing Rules — A Student's Guide

*How a pure-component equation of state becomes a mixture model, why the choice
matters, and which rules `vle-thermo` implements.*

This guide is written for chemical engineering students working through this
repository. It assumes you have met cubic equations of state (Peng-Robinson,
Soave-Redlich-Kwong, …) for **pure** substances — if not, start with
[Chapter II of the research paper](research-paper/chapter-2-vle-theory.md),
which develops the theory this code implements.

---

## 1. The problem mixing rules solve

A cubic equation of state (CEOS) for a pure substance needs only two (sometimes
three) parameters:

- `a` — the **attractive** parameter, measuring how strongly molecules pull on
  each other. Computed from the critical temperature `T_c`, the critical
  pressure `P_c`, and (through the α-function) the acentric factor `ω`.
- `b` — the **co-volume**, the volume the molecules themselves occupy.
  Computed from `T_c` and `P_c`.

Those formulas are only valid for a *pure* substance. A mixture is not a pure
substance with average properties — it is many species interacting at the
molecular level. To apply a CEOS to a mixture, we must build **mixture
parameters** `a_mix` and `b_mix` out of the pure-component `aᵢ`, `bᵢ` and the
composition `x`. The recipes for doing that are called **mixing rules**.

> From [Chapter II §2.1](research-paper/chapter-2-vle-theory.md): *"in order to
> use a CEOS for the correlation and prediction of phases in multicomponent
> systems, a composition dependence of the species comprising the mixture must
> be introduced into these parameters. This dependence is achieved through
> mixing rules."*

The choice is not cosmetic. With the same EOS, the same components, and the
same conditions, two different mixing rules can predict bubble pressures that
differ by tens of percent — because the mixing rule is where the physics of
*unlike-pair interactions* enters the model.

## 2. Where mixing rules sit in a VLE calculation

Every flash, bubble-point, or dew-point calculation in this codebase follows
the same chain:

```
pure-component data (T_c, P_c, ω)
        │  α-function
        ▼
pure aᵢ(T), bᵢ
        │  ← MIXING RULE (this guide) + kij / activity model
        ▼
mixture a_mix(x, T), b_mix(x)
        │  cubic solver (Z-factor)
        ▼
fugacity coefficients φ̂ᵢ  →  K-values  →  flash / bubble / dew
```

Two details matter for understanding the code:

1. The engine works with the **dimensionless** parameters
   `A = a·P/(R·T)²` and `B = b·P/(R·T)`, so the mixing-rule code in
   [`engine/src/mixture.rs`](../../engine/src/mixture.rs) mixes `Aᵢ`, `Bᵢ`
   rather than `aᵢ`, `bᵢ`. The physics is identical.
2. The **partial fugacity coefficient** `ln φ̂ᵢ` — the quantity VLE actually
   needs — requires the *composition derivatives* of the mixture parameters
   (written `Āᵢ`, `B̄ᵢ` in the code). Every mixing rule therefore has to supply
   not just `a_mix` but also how `a_mix` changes when a mole of species *i* is
   added. This is why adding a new mixing rule is more work than one formula.

## 3. The classical (van der Waals) family

### 3.1 The one-fluid quadratic rule

The oldest and still most-used recipe is the **van der Waals one-fluid** rule
([Chapter II, Eqs. 2.6–2.8](research-paper/chapter-2-vle-theory.md)):

`a_mix = ΣᵢΣⱼ xᵢ xⱼ aᵢⱼ`,  with  `aᵢⱼ = √(aᵢ aⱼ) (1 − kᵢⱼ)`

`b_mix = Σᵢ xᵢ bᵢ`

Read it physically: the attraction of the mixture is a sum over all *pairs* of
molecules (hence quadratic in `x`), where an unlike pair `i–j` attracts with
the **geometric mean** of the pure-component attractions. The co-volume is a
simple mole-fraction average — molecules take up their own space regardless of
who their neighbors are.

### 3.2 The binary interaction parameter kᵢⱼ

The geometric-mean assumption is only an approximation, so each unlike pair
gets one small empirical correction, the **binary interaction parameter**
`kᵢⱼ` (symmetric, `kᵢⱼ = kⱼᵢ`, `kᵢᵢ = 0`):

- `kᵢⱼ = 0` — pairs of chemically similar molecules (e.g. two n-alkanes).
- `kᵢⱼ ≈ 0.05–0.15` — chemically *dissimilar* pairs (CO₂–hydrocarbon,
  H₂S–hydrocarbon, N₂–hydrocarbon).
- `kᵢⱼ` is **fitted to experimental VLE data**, not predicted. This repository
  regresses it in [`engine/src/flash/kij_regression.rs`](../../engine/src/flash/kij_regression.rs);
  the research paper's validation case fits CO₂/n-butane to `kᵢⱼ = 0.1357`
  ([Chapter IV, Tables 4.11–4.12](research-paper/chapter-4-validation.md)).

A useful mental model: `kᵢⱼ` is a *one-knob* correction. It can fix the
magnitude of unlike attraction, but it cannot reshape how the mixture's
non-ideality varies with composition. That limitation is what motivates
everything in §4.

### 3.3 Variants in this codebase

The legacy VB6 program (and therefore this engine) carries three classical
variants:

- **`Classical`** — the quadratic rule exactly as above.
- **`IVDW`** ("improved van der Waals one-fluid") — same `a_mix`; the legacy
  program writes `b_mix = ΣᵢΣⱼ xᵢxⱼ(bᵢ+bⱼ)/2`, which is algebraically identical
  to the linear `Σ xᵢbᵢ` (expand it and see!). In this engine `Classical` and
  `IVDW` share one code path.
- **`IIVDW`** ("two-fluid") — makes the interaction correction
  **composition-dependent and asymmetric**: `kₘ(i,j) = kᵢⱼ·xᵢ + kⱼᵢ·xⱼ` with
  `kᵢⱼ ≠ kⱼᵢ` allowed. The two extra degrees of freedom help for moderately
  polar/asymmetric binaries, at the price of a more complicated composition
  derivative (see the comment block in
  [`engine/src/mixture.rs`](../../engine/src/mixture.rs) for the derivation).

**When to use the classical family:** nonpolar and slightly polar systems —
natural gas, refinery hydrocarbons, light gases in oil — at any pressure. This
is the default (`mixing_rule="classical"`) and the right choice for most
hydrocarbon work. It **fails** for strongly non-ideal liquids (alcohols, water,
associating species), where no single `kᵢⱼ` can represent the liquid phase.

## 4. The Gᴱ (EOS/Gᴱ) family: putting an activity model inside the EOS

Strongly non-ideal liquid mixtures (methanol–water, acetone–chloroform, …)
are traditionally described by **activity-coefficient models** (van Laar,
Wilson, NRTL, …) — see [Chapter II §2.2](research-paper/chapter-2-vle-theory.md).
Those models are accurate for liquids at low pressure but say nothing about the
vapor, about density, or about high pressure. A CEOS handles all of those, but
with classical mixing it cannot represent the liquid non-ideality.

The **Gᴱ-based mixing rules** (also called *EOS/Gᴱ* rules) get both: they
choose `a_mix` so that the **excess Gibbs energy predicted by the EOS matches
the one predicted by an activity model** at some reference pressure. You keep
the EOS framework (works at any pressure, consistent densities and enthalpies,
supercritical components are no problem) and inherit the activity model's
skill with non-ideal liquids.

All of these rules therefore require you to supply *two* models: the EOS
**and** an activity model with its interaction parameters (`aij` matrix).

The members implemented here, in historical order:

- **Huron-Vidal, original (`HuronVidalOriginal`, 1979)** — matches the EOS Gᴱ
  to the activity model at **infinite pressure**:
  `a_mix/b_mix = Σ xᵢ·(aᵢ/bᵢ) + Gᴱ/c*`, where `c*` is a constant of the EOS
  family. Simple and robust; its known quirk is that activity-model parameter
  tables fitted at *low* pressure are not strictly valid at the
  infinite-pressure reference, so literature parameters carry over only
  approximately.
- **Huron-Vidal simplified (`HuronVidalSimplified`)** — same structure with an
  extra `Σ xᵢ·ln(b_mix/bᵢ)` term (a Flory-Huggins-like size correction).
- **MHV1 (`MHV1`, Michelsen 1990)** and **MHV2 (`MHV2`)** — move the matching
  point to **zero pressure**, which is where activity-model parameters are
  actually fitted, so published Wilson/van Laar/NRTL tables can be reused
  more faithfully. MHV1 solves a linear equation for `a_mix/b_mix`
  (`q₁ = −0.593`); MHV2 a quadratic one (`q₁ = −0.478`, `q₂ = −0.0047`),
  which improves accuracy for asymmetric systems.
- **Wong-Sandler (`WongSandler`, 1992)** — the theoretically most careful
  member, and the one recommended by Orbey & Sandler
  [(21)](research-paper/references.md). Besides matching the activity model's
  *Helmholtz* energy at infinite pressure, it enforces the one exact boundary
  condition statistical mechanics gives us: the **second virial coefficient of
  a mixture must be quadratic in composition**. It mixes `(b − a/RT)`
  quadratically (with its own `kᵢⱼ`) and extracts `a_mix`, `b_mix` from that —
  note that here `b_mix` is *not* linear in `x`. This gives correct
  low-density behavior *and* activity-model liquid behavior.

**When to use the Gᴱ family:** polar / associating / hydrogen-bonding mixtures
— alcohol–water, acetone–water, and their ternaries — especially when you also
need elevated pressure, supercritical components, or EOS-consistent enthalpies
(so a single model covers both phases). At low pressure with no such needs, a
plain activity-coefficient approach (`liquid_model="activity"`, the γ-φ route)
is simpler and equally accurate.

## 5. C-parameter rules (three-parameter EOS only)

Two of the EOS inherited from the Pascal legacy code — **Schmidt-Wenzel** and
**Patel-Teja** — carry a *third* parameter `c`, which also has to be mixed.
These rules (Ref (4): Da Silva & Báez, 1989) answer only that narrow question
and are **not** alternatives to the rules above (`a` and `b` still use a
classical rule):

- **`PatelTejaC`** — `c_mix = Σ xᵢ·cᵢ` (mole-fraction average), for Patel-Teja.
- **`PatelTejaUSBC`** — `c_mix = Σ(xᵢ√Bᵢ·cᵢ) / Σ(xᵢ√Bᵢ)` (√B-weighted), for
  the Patel-Teja USB variant.
- **`SchmidtWenzelC`** — `c_mix = Σ(xᵢ√Aᵢ·ωᵢ) / Σ(xᵢ√Aᵢ)` (√A-weighted, using
  the acentric factor as the third parameter), for Schmidt-Wenzel.

In practice you never select these by hand: the engine wires the correct
C-rule to each three-parameter EOS automatically (see `MixtureSpec` in
[`engine/src/mixture.rs`](../../engine/src/mixture.rs)), matching the legacy
Pascal behavior. They are listed here so the `MixingRule` enum makes sense.

## 6. Choosing a rule — a practical guide

| Your system | Recommended rule | Why |
|---|---|---|
| Hydrocarbons, natural gas, air, light gases | `classical` (kᵢⱼ = 0 for similar pairs) | Geometric mean is nearly exact for nonpolar pairs |
| CO₂ / H₂S / N₂ with hydrocarbons | `classical` + fitted `kij` | One knob suffices; well-tabulated kᵢⱼ exist |
| Moderately polar, asymmetric binaries | `2vdw` (IIVDW) | Composition-dependent, asymmetric kᵢⱼ |
| Alcohol–water, ketone–water, associating mixtures at low P | γ-φ approach (`liquid_model="activity"`), no EOS mixing rule needed | Activity model alone is accurate at low P |
| Same chemistry, but high P / supercritical components / need EOS enthalpies | `wong-sandler` (first choice) or `mhv1`/`mhv2` + an activity model | EOS/Gᴱ: activity-model liquid + EOS pressure range |
| Reusing published low-pressure activity parameters inside an EOS | `mhv1` / `mhv2` | Zero-pressure reference matches how the tables were fitted |
| Schmidt-Wenzel or Patel-Teja EOS | (automatic C-rule) | Third parameter mixed per Ref (4) |

Rules of thumb worth internalizing:

- **Start simple.** `classical` with a sensible `kij` answers most questions.
  Reach for EOS/Gᴱ only when the liquid is genuinely non-ideal.
- **A mixing rule cannot rescue bad parameters.** Wong-Sandler with a poorly
  fitted activity model is worse than classical with a good `kij`.
- **Validate against data** — this repository's own test suite pins model
  results to the research paper's Chapter IV tables; do the same with at least
  one experimental point whenever you switch rules.

## 7. What `vle-thermo` implements

All eleven variants of the `MixingRule` enum
([`engine/src/mixing.rs`](../../engine/src/mixing.rs)), with their Python
aliases (`vle.System(mixing_rule=...)`):

| Rust variant | Python alias | Kind | Needs |
|---|---|---|---|
| `Classical` | `"classical"` (default) | classical, quadratic one-fluid | optional `kij` |
| `IVDW` | `"vdw"` / `"1vdw"` | classical (same math as `Classical`) | optional `kij` |
| `IIVDW` | `"2vdw"` | classical, composition-dependent asymmetric kᵢⱼ | asymmetric `kij` |
| `WongSandler` | `"wong-sandler"` / `"wong_sandler"` | EOS/Gᴱ, 2nd-virial-consistent | `activity` + `aij` (+ optional `kij`) |
| `HuronVidalOriginal` | `"huron-vidal"` | EOS/Gᴱ, infinite-P reference | `activity` + `aij` |
| `HuronVidalSimplified` | — (pass `MixingRule.HuronVidalSimplified`) | EOS/Gᴱ, infinite-P + size term | `activity` + `aij` |
| `MHV1` | `"mhv1"` | EOS/Gᴱ, zero-P reference, linear | `activity` + `aij` |
| `MHV2` | `"mhv2"` | EOS/Gᴱ, zero-P reference, quadratic | `activity` + `aij` |
| `PatelTejaC` | (automatic with Patel-Teja EOS) | C-parameter | — |
| `PatelTejaUSBC` | (automatic with Patel-Teja USB EOS) | C-parameter | — |
| `SchmidtWenzelC` | (automatic with Schmidt-Wenzel EOS) | C-parameter | — |

For a variant without a string alias, pass the enum directly:
`from vle import MixingRule` then
`System(..., mixing_rule=MixingRule.HuronVidalSimplified)`.

The Gᴱ rules accept any of the engine's activity models (van Laar, Wilson,
Margules, Scatchard-Hildebrand, ideal solution) as the Gᴱ supplier.

**Units** (see [CLAUDE.md](../../CLAUDE.md) and the
[dimensional-analysis guide](units/dimensional-analysis.md)): temperatures in
K, pressures in kPa absolute; `kij` is dimensionless; the activity `aij`
matrix uses each model's own convention — **dimensionless** constants for van
Laar / Margules, **kJ/kmol** interaction energies for Wilson.

## 8. Worked example

Both snippets run against the published `vle-thermo` wheel.

**Classical rule + fitted kᵢⱼ** — the research paper's CO₂/n-butane system
(`kᵢⱼ = 0.1357` from [Tables 4.11–4.12](research-paper/chapter-4-validation.md)):

```python
from vle import System

s = System(["carbon dioxide", "n-butane"], eos="PR",
           mixing_rule="classical", kij=0.1357)
res = s.bubble_pressure(x=[0.2, 0.8], t=310.0)   # T in K
print(res.value)   # bubble pressure in kPa absolute (~2085 kPa)
print(res.y)       # equilibrium vapor composition
```

**Wong-Sandler + van Laar** — a polar system (methanol/water) where the
classical rule would struggle; the activity model supplies the liquid-phase
non-ideality *inside* the Peng-Robinson EOS:

```python
from vle import System

# Van Laar constants A12, A21 for methanol(1)/water(2) — dimensionless
aij = [[0.0, 0.8041],
       [0.5619, 0.0]]

s = System(["methanol", "water"], eos="PR",
           mixing_rule="wong-sandler", activity="van_laar", aij=aij)
res = s.bubble_pressure(x=[0.4, 0.6], t=337.0)
print(res.value)   # bubble pressure in kPa absolute (~60 kPa)
```

Try swapping `mixing_rule="wong-sandler"` for `"mhv1"`, `"mhv2"`, or
`"classical"` (drop `activity`/`aij`) and compare the predicted bubble
pressures — the spread you see *is* the mixing-rule sensitivity discussed in
§1.

## 9. Further reading

- [Research paper, Chapter II](research-paper/chapter-2-vle-theory.md) — the
  full VLE theory: CEOS, mixing rules (§2.1), activity models (§2.2), fugacity.
- [Research paper, Chapter IV](research-paper/chapter-4-validation.md) — the
  validation cases these implementations are pinned to.
- [`engine/src/mixing.rs`](../../engine/src/mixing.rs) — the `MixingRule` enum
  and C-parameter mixing, with per-variant doc comments.
- [`engine/src/mixture.rs`](../../engine/src/mixture.rs) — the full
  implementation, including every rule's composition derivatives (`Āᵢ`, `B̄ᵢ`)
  and the legacy VB6 line references.
- References [(21) Orbey & Sandler](research-paper/references.md) —
  Wong-Sandler mixing rules; [(4) Da Silva & Báez (1989)](research-paper/references.md)
  — C-parameter rules; Michelsen (1990) — MHV1/MHV2.
- [Parameter reference](parameters/parameter_reference.md) — where `T_c`,
  `P_c`, `ω`, and the interaction matrices come from.
