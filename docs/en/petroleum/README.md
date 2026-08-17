# Petroleum Characterization and Refinery Thermodynamics

**A learning guide to Milestones 19 and 20 — `vle_thermo::petroleum` /
`vle.petroleum` (characterization) and `vle_thermo::refinery` / `vle.refinery`
(refinery thermodynamics)**

Written by [Claude](https://claude.ai/code) through Claude Code — the
characterization sections (§1–§10) by Anthropic's Claude Opus 5 (1M context),
the Milestone 20 material (§11, the §8 re-examination, references 42–45 in
§13, and this revision) by Claude Fable 5. **Miguel Jackson was the editor** — he set the scope, corrected
the engineering, and approved publication. **A disclosure, since it matters
here:** this document was written by the same agents that wrote the code it
describes, and it makes accuracy claims about that code. Do not take the
models' word for any of them. Every number in §7 is asserted by a named test
you can run — `cargo test -p vle-thermo petroleum::` (and
`cargo test -p vle-thermo refinery::` for §11) — and the ones that are
*unflattering* are in §8, asserted from both sides so that fixing them breaks
the test and forces this document to be corrected.

This document explains what petroleum characterization is, why a thermodynamics
engine needs it, and what every function in the module does — and then, in
§11, the refinery methods that Milestone 20 built on top of it. It is written
to be read start to finish by somebody who has never seen a crude assay, and to
be grepped later by somebody who just wants a signature.

Companion material:

- [`notebooks/15_petroleum_characterization.ipynb`](../../../notebooks/15_petroleum_characterization.ipynb) — the same ideas, executable
- [`notebooks/16_refinery_thermodynamics.ipynb`](../../../notebooks/16_refinery_thermodynamics.ipynb) — the Milestone 20 material, executable
- [`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](../../plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md) — the design record (§2, U1 + U2 for M19; U4 + U5 for M20)
- [`MODERNIZATION_PLAN.md`](../../plans/MODERNIZATION_PLAN.md) — Phases 26 and 27

---

## Why these modules get a document of their own

Most of this engine is a *library*: you call a function, you get a number. The
petroleum module (and the refinery layer built on it, §11) is different — it is
a **foundation for designing a piece of equipment**,
and the reason it is explained at this length is that the equipment in question
is the one the whole project is pointed at.

A distillation column is designed component by component, stage by stage
(§1 has the hundred-year history of *why* it is done that way). Crude oil has no
components. Everything in this document exists to bridge that gap, and getting
the bridge wrong does not produce an obviously wrong answer — it produces a
plausible column with the wrong product yields. That is the kind of error worth
a long document.

The column itself is not built here. It is built in the cousin repository,
**[`stages-thermo`](https://github.com/miguelju/stages-thermo)** — the
staged-separation library that carries the actual column-design algorithms and
calls this repo for all of its thermodynamics. Its flagship solver is
Naphtali–Sandholm, full Newton on the MESH equations; the Boston–Britt
inside-out algorithm is its next milestone, and is required precisely because
of the component counts this module produces. The two repositories are
deliberately split along that seam:
**`vle-thermo` answers "what does this mixture do at this T and P"; `stages-thermo`
answers "how many trays do I need".** Neither is much use without the other.

That makes this module a **prerequisite**, not a side quest. `stages-thermo`
cannot start a crude tower without a component list, and this is where the
component list comes from — which is also why the shared design record
([`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](../../plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md))
spans both repositories rather than living in one.

---

## Table of contents

1. [The problem](#1-the-problem)
2. [The domain, from first principles](#2-the-domain-from-first-principles)
3. [The pipeline](#3-the-pipeline)
4. [Function reference](#4-function-reference)
5. [The correlations, written out](#5-the-correlations-written-out)
6. [The Python surface](#6-the-python-surface)
7. [Validation — what was checked against what](#7-validation--what-was-checked-against-what)
8. [Known gaps, and why they are gaps](#8-known-gaps-and-why-they-are-gaps)
9. [Design decisions worth knowing](#9-design-decisions-worth-knowing)
10. [Recipes](#10-recipes)
11. [Refinery thermodynamics — Milestone 20](#11-refinery-thermodynamics--milestone-20)
12. [Where to go next](#12-where-to-go-next)
13. [Academic references](#13-academic-references)

Appendix: [A note on the illustration](#a-note-on-the-illustration).

---

## 1. The problem

Every other module in this engine starts the same way. You name a compound —
`"benzene"` — and a database hands back its critical temperature, critical
pressure and acentric factor. From those three numbers a cubic equation of state
produces the `a` and `b` parameters, and everything else follows.

**Crude oil breaks that assumption completely.** A barrel of crude is a mixture
of many thousands of distinct molecules. Nobody has separated them; nobody has
measured their critical properties; nobody ever will. There is no component list
to look up.

And yet crude columns get designed anyway — and have been, for roughly a
century. Not by refineries, which operate the plants; by process licensors and
engineering contractors, whose design methods are **component-based from top to
bottom**. Every one of them, from a 1925 graphical construction to a 2026
process simulator, wants the same thing: a list of components, an equilibrium
stage, and a mass and energy balance around it. The method assumes a component
list. Crude oil does not have one.

The trick the industry settled on to reconcile those two facts is worth stating
plainly, because the whole module is an implementation of it:

> Do not ask *what* is in the barrel. Ask **how much of it boils by what
> temperature**, and **how dense it is**. Slice that boiling curve into narrow
> ranges, pretend each slice is a single compound, and estimate that
> compound's properties from just two numbers: the slice's mid-boiling point
> and its density.

Each slice is a **pseudocomponent**. It is a fiction — no molecule in the barrel
has exactly those properties — but it is a *useful* fiction, because the
thermodynamics of a mixture depends mostly on the distribution of volatilities,
and a few hundred narrow slices reproduce that distribution well.

Two numbers in, six out, a few hundred times over. That is this module.

### A method older than the computers that run it

The genuinely striking thing about all of this — and the reason it is worth a
page in a learning repository rather than a footnote — is how little the
*model* has changed. What changed was the arithmetic.

| year | contribution | reference |
|---|---|---|
| **1893** | **Sorel**, working on alcohol, gives the first theoretical analysis of continuous distillation: an equilibrium stage, with mass and energy balances written around it. Accurate, and laborious — every stage needed trial and error. | Sorel, *La Rectification de l'Alcool*, Gauthier-Villars, Paris, **1893** |
| **1922** | **Lewis** notices that vapour and liquid flows are often nearly constant. Assume they are, and Sorel's trial-and-error disappears. **Ponchon** and **Savarit**, independently and in the same year, solve it graphically on an enthalpy–composition diagram instead. | Lewis, *Ind. Eng. Chem.* **1922**, *14*, 492<br>Ponchon, *Tech. Moderne* **1922**, *13*, 20<br>Savarit, *Arts et Métiers* **1922**, *65*, 142 |
| **1925** | **McCabe & Thiele** turn Lewis's simplification into the staircase diagram every chemical engineer still learns. It spread fast because it made *visible* why a column works. | McCabe & Thiele, *Ind. Eng. Chem.* **1925**, *17*, 960 |
| **1932–33** | Multicomponent, and pointedly petroleum: **Lewis & Matheson**, *"Design of Rectifying Columns for Natural and Refinery Gasoline"*, and **Thiele & Geddes**, *"Computation of Distillation Apparatus for Hydrocarbon Mixtures"* — stage-by-stage methods, by hand. In the same two years, **Watson & Nelson** at UOP publish the characterization factor. | Lewis & Matheson, *Ind. Eng. Chem.* **1932**, *24*, 494<br>Thiele & Geddes, *Ind. Eng. Chem.* **1933**, *25*, 289<br>Watson & Nelson, *Ind. Eng. Chem.* **1933**, *25*, 880 |
| **1932–48** | **Fenske**, **Gilliland** and **Underwood** give the shortcut trio — minimum stages, minimum reflux, and the correlation between them — that let an engineer size a column in an afternoon with a slide rule. | Fenske, *Ind. Eng. Chem.* **1932**, *24*, 482<br>Gilliland, *Ind. Eng. Chem.* **1940**, *32*, 1220<br>Underwood, *Chem. Eng. Prog.* **1948**, *44*, 603 |
| **1958** | **Amundson & Pontinen**, *"Multicomponent Distillation Calculations on a Large Digital Computer"*: the equations, they realised, are convenient to solve once written in **matrix form**. This is the hinge. | Amundson & Pontinen, *Ind. Eng. Chem.* **1958**, *50*, 730 |
| **1966–71** | **Wang & Henke**'s tridiagonal sweep; then **Naphtali & Sandholm**'s simultaneous Newton linearization — still in commercial simulators today. | Wang & Henke, *Hydrocarbon Process.* **1966**, *45* (8), 155<br>Naphtali & Sandholm, *AIChE J.* **1971**, *17* (1), 148 |
| **1974–78** | **Boston & Sullivan**, then **Boston & Britt**: the **inside-out** algorithm, which fits cheap local models to expensive rigorous thermodynamics precisely so a several-hundred-component column stays tractable. | Boston & Sullivan, *Can. J. Chem. Eng.* **1974**, *52* (1)<br>Boston & Britt, *Comput. Chem. Eng.* **1978**, *2*, 109 |

*Full citations, including the secondary source used to check this chronology, are in [§13](#historical-sources-context-only--none-of-these-is-implemented-here).*

Read down that table and notice what is *not* in it. Nobody replaced the
equilibrium stage. Nobody replaced the component list. And **nobody designs a
column by simulating molecules** — molecular simulation is a real and active
field, but it lives upstream, in predicting the properties that feed a
correlation, never in the column calculation itself. A modern process simulator
solving a crude tower is doing Sorel's 1893 calculation — the same balances around the same idealised stage — at a
speed and scale he could not have imagined, on a component list that is largely
invented. The 1958 paper is titled *"…on a Large Digital Computer"* precisely
because the computer was the news; the model was already thirty years old and
did not change.

Two of those entries are load-bearing in this repository right now. **Watson &
Nelson's 1933 factor is [`watson_k`](#41-gravity)** in `petroleum/gravity.rs`, a
93-year-old correlation on the hot path of every characterization. And
**Boston & Britt's inside-out** is the solver the downstream `stages-thermo`
project needs in order to run a column at the component counts this module
produces — which is why Milestone 18 spent its effort making the mixture core
scale to N = 300 rather than making it cleverer.

The correlations in §5 are of the same vintage and the same character:
empirical, fitted to bulk measurements, published in °R and psia, and still in
service because nothing better-founded has turned out to be more useful. That
is not a criticism. It is the reason this module exists at all.

### Why it matters here

`PETROLEUM_PSEUDOCOMPONENT_PLAN.md` identifies the atmospheric crude
distillation column as the workload that stresses everything this project has
built: hundreds of components, strongly non-ideal, with a real energy balance.
Characterization is the gate. Nothing downstream can start without a component
list, and this module is where the component list comes from.

---

## 2. The domain, from first principles

### 2.1 Specific gravity and API gravity

**Specific gravity (SG)** in petroleum always means the **60/60 °F** value: the
density of the liquid at 60 °F divided by the density of water at 60 °F. It is
dimensionless. It is *not* the same as the density at 20 °C that a chemistry
handbook prints, and mixing the two is a real source of error.

**API gravity** is a rescaling invented so that the numbers get *bigger* as the
oil gets *lighter*:

$$^\circ\mathrm{API} = \frac{141.5}{SG} - 131.5$$

The scale is anchored on water: SG = 1.000 gives exactly 10 °API. Above 10 the
oil floats; below 10 it sinks. Typical values:

| material | °API | SG |
|---|---|---|
| light sweet crude | 35–45 | 0.80–0.85 |
| medium crude | 25–35 | 0.85–0.90 |
| heavy crude | 15–25 | 0.90–0.97 |
| bitumen | < 10 | > 1.00 |

Refineries quote API gravity because a higher number means a more valuable
barrel, and the scale makes that read the right way round.

### 2.2 The Watson characterization factor

If you can know only one thing about a petroleum fraction beyond its boiling
point, know this one. **Watson K** (also called the UOP characterization factor)
compares a fraction's boiling point against its density:

$$K_W = \frac{\left(T_b \text{ in } ^\circ\mathrm{R}\right)^{1/3}}{SG}$$

Why does that combination mean anything? Because boiling point measures roughly
how big the molecules are, and density measures roughly how tightly they pack —
which for hydrocarbons is a question of **how much hydrogen** they carry. A
straight-chain paraffin is hydrogen-rich and floats; an aromatic ring of the same
boiling point is hydrogen-poor and sinks. So the ratio sorts hydrocarbons by
chemical family:

| Watson K | family | example |
|---|---|---|
| 12.5 – 13.0 | **paraffinic** — straight-chain alkanes | n-heptane, 12.71 |
| 11 – 12 | **naphthenic** — saturated rings | methylcyclohexane, 11.31 |
| ~10 | **aromatic** — benzene rings | benzene, 9.74 |

Those three numbers are computed by this module's own tests, which is a small
but real demonstration that the °R conversion inside `watson_k` is correct: run
the same formula in kelvin and n-heptane comes out at 10.4, misclassified as an
aromatic.

`K_W` is an input to the heat-capacity correlation and to the Maxwell–Bonnell
vapor-pressure correction, and it is the number an engineer quotes when asked
"what kind of crude is it?"

### 2.3 Average boiling points — there are five

A pure compound has *a* boiling point. A mixture does not — it has a
distribution — and different physical properties want different averages of that
distribution. Correlations are explicit about which one they want, and using the
wrong one is a quiet, plausible-looking error.

Given per-cut boiling points $T_{b,i}$ with volume fractions $v_i$, weight
fractions $w_i$ and mole fractions $x_i$:

| average | definition | typically used for |
|---|---|---|
| **VABP** volume | $\sum v_i T_{b,i}$ | the raw mean; input to the other four |
| **WABP** weight | $\sum w_i T_{b,i}$ | liquid density, viscosity |
| **MABP** molar | $\sum x_i T_{b,i}$ | ideal-gas properties, molecular weight |
| **CABP** cubic | $\left[\sum v_i T_{b,i}^{1/3}\right]^3$ | pseudocritical temperature |
| **MeABP** mean | $\tfrac{1}{2}(\mathrm{MABP} + \mathrm{CABP})$ | Watson K, critical properties, enthalpy — **the usual default** |

They are always ordered

$$\mathrm{WABP} \ge \mathrm{VABP} \ge \mathrm{CABP} \ge \mathrm{MeABP} \ge \mathrm{MABP}$$

and all five collapse onto the same number for a narrow cut. Two of those
inequalities have clean reasons worth internalising:

- **WABP ≥ VABP** because weighting by mass favours the heavy, high-boiling tail.
- **CABP ≤ VABP** by *Jensen's inequality*: $x^{1/3}$ is concave, so the cube of
  the mean cube-root is at most the mean. This is not an empirical observation —
  it is a theorem, and `cubic_average_never_exceeds_volume_average` asserts it
  across three different spreads.

### 2.4 Four kinds of distillation curve

"How much of this oil boils below 200 °C" sounds like one question. It is four,
because the answer depends on the apparatus you measure it with.

| basis | what it is | separation |
|---|---|---|
| **ASTM D86** | the cheap, fast, universal lab test — a single-stage flask at atmospheric pressure | ~1 theoretical plate — poor |
| **TBP** (ASTM D2892) | a real 15-plate column at 5:1 reflux; the closest thing to "the actual boiling points of the molecules" | ~15 plates — good |
| **D2887 / SimDist** | gas chromatography, elution time calibrated against n-alkanes. Reports **weight** percent | effectively perfect |
| **EFV** | equilibrium flash vaporization — one equilibrium stage, no fractionation at all | 0 plates |

The physical content is simple: **more theoretical plates, wider apparent boiling
range.** A good column pulls the light ends out early and holds the heavy ends
back, so it reports a wider spread for identical material. A single flash
separates almost nothing and reports the flattest curve of the four.

![Four distillation methods, ordered by fractionation power. Left to right: an
EFV flash drum with no internals; an ASTM D86 round-bottomed flask with a
side-arm and thermometer; a TBP column with internal trays, an overhead
condenser and a reboiler; and a D2887 benchtop gas chromatograph with a coiled
capillary column. Beneath each is its distillation curve — the same gentle
S-shape in all four, but climbing through a progressively greater temperature
range from left to right.](../../assets/distillation-bases.png)

*The picture is the argument: the apparatus on top determines the curve
underneath. Same oil in every panel — only the separation changes. (Schematic,
not to scale; the tray count in the third panel is illustrative.
[How this image was made](#a-note-on-the-illustration).)*

**Every property correlation downstream is written against TBP**, because TBP is
the one that approximates a list of molecular boiling points. Almost every assay
you will actually be handed is D86 (for light products) or D2887 (for anything
modern). So conversion is the first thing that happens, always.

### 2.5 Pseudocomponents

Slicing the TBP curve gives a list of narrow boiling ranges. Each becomes a
`Component` carrying:

- a **normal boiling point** — the volume-average of the curve across the slice;
- a **specific gravity** — from a measured gravity curve if you have one, or
  from an assumption of chemical uniformity if you do not (§9.2);
- **Tc, Pc, ω, M, Vc, Zc** — correlated from those two numbers;
- an **ideal-gas Cp° polynomial** and a **vapor-pressure correlation**, so it
  can carry an energy balance and a K-value;
- since Milestone 20, the three numbers the refinery methods of §11 read — a
  **Watson K**, a Rackett **Z_RA**, and a regular-solution **solubility
  parameter**.

The design constraint that shaped the whole module: a pseudocomponent must be an
**ordinary `Component`**, so that nothing in `flash`, `mixture` or `energy` needs
a special case for correlated properties. It is, and
`pseudocomponents_drive_a_real_flash` proves it by running a 12-component
Peng-Robinson flash straight off an assay.

---

## 3. The pipeline

```
   Assay { curve, gravity }
        │
        ├─ convert_curve      ──▶  TBP basis                   distillation.rs
        ├─ cut_curve          ──▶  N slices: Tb + share        cuts.rs
        ├─ cut_gravities      ──▶  SG per slice                assay.rs (§9.2)
        ├─ estimate           ──▶  M, Tc, Pc, ω, Vc, Zc        properties.rs
        ├─ ideal_gas_cp_coeffs──▶  Cp°(T) polynomial           cp.rs
        ├─ pseudo-Antoine     ──▶  Psat(T)                     assay.rs (§9.3)
        └─ δ, K_W, Z_RA       ──▶  what the M20 methods read   assay.rs (§11)
                                    │
                                    ▼
                    Vec<Component> + mole fractions
                                    │
                                    ▼
                    flash / mixture / energy — unchanged
                    (+ refinery / free_water, §11)
```

Seven submodules in `petroleum/`, each one layer (the refinery layer that
consumes their output is §11):

| module | responsibility |
|---|---|
| [`gravity`](#41-gravity) | API ↔ SG, Watson K, the five average boiling points |
| [`distillation`](#42-distillation) | D86 ↔ TBP ↔ D2887 ↔ EFV interconversion |
| [`cuts`](#43-cuts) | a TBP curve into N slices |
| [`properties`](#44-properties) | Tb + SG → M, Tc, Pc, ω, Vc, Zc |
| [`cp`](#45-cp) | ideal-gas Cp° for a fraction |
| [`vapor_pressure`](#46-vapor_pressure) | Maxwell–Bonnell |
| [`assay`](#47-assay) | the whole pipeline |

---

## 4. Function reference

Units throughout are the crate's canonical ones — **temperature in K, pressure in
kPa absolute, molecular weight in g/mol, volume in cm³/mol** — regardless of what
units the underlying correlation was published in. See §9.1 for why the imperial
conversions live *inside* each function rather than at the caller.

Every fallible function returns `Result<_, PetroleumError>`.

### 4.1 `gravity`

Small, shared quantities that the rest of the module depends on.

| function | signature | returns |
|---|---|---|
| `api_from_sg` | `(sg: f64) -> Result<f64>` | API gravity, °API |
| `sg_from_api` | `(api: f64) -> Result<f64>` | specific gravity, dimensionless |
| `watson_k` | `(tb: f64, sg: f64) -> Result<f64>` | Watson K, dimensionless |
| `average_boiling_points` | `(d86_10, d86_30, d86_50, d86_70, d86_90: f64) -> Result<AverageBoilingPoint>` | all five averages, K |
| `weighted_boiling_point` | `(tb: &[f64], fractions: &[f64]) -> Result<f64>` | weighted mean, K |
| `cubic_boiling_point` | `(tb: &[f64], volume_fractions: &[f64]) -> Result<f64>` | cubic average, K |
| `blend_watson_k` | `(kw: &[f64], weight_fractions: &[f64]) -> Result<f64>` | blended Watson K |

**`AverageBoilingPoint`** — `{ vabp, wabp, mabp, cabp, meabp }`, all f64 in K.

Notes worth having:

- `api_from_sg` and `sg_from_api` are exact inverses, and both anchor water at
  10 °API exactly.
- `watson_k` converts to °R internally. That conversion is load-bearing, not
  cosmetic — see §2.2.
- `average_boiling_points` takes an **ASTM D86** curve (API Procedure 2B1.1) and
  works in °F internally. It clamps `(VABP − 32)` at zero so an LPG-range cut
  whose volume average is below the freezing point of water returns finite
  numbers instead of `NaN`.
- `weighted_boiling_point` and `cubic_boiling_point` normalize their weights, so
  callers never have to pre-normalize.

### 4.2 `distillation`

**`DistillationBasis`** — `D86 | Tbp | D2887 | Efv`, with

- `.name() -> &'static str` — human-readable, used in error messages;
- `.is_weight_basis() -> bool` — true only for `D2887`, which is a chromatogram.

**`DistillationCurve`** — `{ basis, fractions: Vec<f64>, temperatures: Vec<f64> }`.
Fractions are strictly increasing in `[0, 1]`; temperatures are non-decreasing
and in K. The constructor enforces both.

| method | signature | notes |
|---|---|---|
| `new` | `(basis, fractions, temperatures) -> Result<Self>` | rejects mismatched lengths, < 2 points, out-of-range or non-increasing fractions, non-positive or *decreasing* temperatures |
| `len` | `(&self) -> usize` | |
| `is_empty` | `(&self) -> bool` | always `false`; present for clippy |
| `temperature_at` | `(&self, fraction: f64) -> f64` | linear interpolation; **extrapolates** past both ends along the end segments |
| `fraction_at` | `(&self, temperature: f64) -> f64` | the inverse; result is deliberately **not** clamped to `[0, 1]` |
| `resample` | `(&self, fractions: &[f64]) -> Result<Self>` | re-express on a different grid |

`STANDARD_GRID: [f64; 7] = [0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95]` — the grid
the API interconversion procedures are defined on. `curve.resample(&STANDARD_GRID)`
is the one-line fix when a difference procedure complains about a missing 50 %
point.

The conversions come in two families.

**Point-wise power laws** — each point converts independently, so they need no
particular grid:

| function | direction | formula |
|---|---|---|
| `d86_to_tbp_riazi` | D86 → TBP | $T^* = a\,T^b$, T in **K** |
| `tbp_to_d86_riazi` | TBP → D86 | $T = (T^*/a)^{1/b}$ |
| `d86_to_efv` | D86 → EFV | $T^* = a\,T^b SG^c$, T in **K** |
| `efv_to_d86` | EFV → D86 | the exact inverse |

The two EFV functions take an extra `sg: f64` — an EFV curve is a real
phase-equilibrium calculation in disguise, and how much a cut flashes depends on
how dense it is.

**Difference (delta) methods** — the API procedures. They convert the 50 % point
with one power law, convert each *temperature difference* between adjacent grid
points with its own, and rebuild the curve by accumulating outward from 50 %:

| function | direction | procedure |
|---|---|---|
| `d86_to_tbp_daubert` | D86 → TBP | API 3A1.1 |
| `tbp_to_d86_daubert` | TBP → D86 | 3A1.1 inverted |
| `d2887_to_tbp` | SimDist → TBP | API 3A3.1 |
| `tbp_to_d2887` | TBP → SimDist | 3A3.1 inverted |
| `d2887_to_d86` | SimDist → D86 | API 3A3.2 |

All of these work in **°F** internally (§9.1) and require a 50 % point on the
curve. Accumulating outward from 50 % rather than from the initial point is what
keeps an error in one difference from propagating across the whole curve.

**The router:**

```rust
pub fn convert_curve(
    curve: &DistillationCurve,
    target: DistillationBasis,
    sg: Option<f64>,
) -> Result<DistillationCurve, PetroleumError>
```

Routes any basis to any other through TBP as the hub, preferring the API
difference procedures where they exist. D86 ↔ EFV and D2887 → D86 short-circuit
to their direct procedures rather than taking a lossy two-hop route. `sg` is
required only when the route touches EFV, and the error says so if you forget it.

```
         D2887                        EFV
           │ API 3A3.1                 │ Edmister-Okamoto
           ▼                           ▼
         ═══════════ TBP ══════════ D86
                         API 3A1.1
```

### 4.3 `cuts`

**`CutSpec`** — how to slice:

| variant | slices are equal in | use it when |
|---|---|---|
| `EqualVolume { n }` | volume fraction | you want N pseudocomponents and do not care where they land — the default for feeding a column model |
| `EqualTemperature { n }` | boiling range | you want even resolution in temperature, so the flat middle of the curve does not absorb all the detail |
| `Boundaries { boundaries: Vec<f64> }` | nothing — you say where | you are modelling **real products**: naphtha, kerosene, diesel, AGO. The boundaries are the tower's own draw specifications |

`Boundaries` takes the **internal** boundaries only; the curve's own initial and
final boiling points close the ends, so *k* boundaries give *k* + 1 cuts.
Boundaries outside the curve's span are an error, not a silent clamp.

**`Cut`** — `{ index, fraction, x_lower, x_upper, t_lower, t_upper, tb }`, with
`.width() -> f64` giving `t_upper − t_lower`. `fraction` is normalized so the
cuts sum to exactly 1 even when the curve does not run from 0 to 1 — assays
routinely start at 5 % and stop at 95 %.

```rust
pub fn cut_curve(
    curve: &DistillationCurve,
    spec: &CutSpec,
) -> Result<Vec<Cut>, PetroleumError>
```

The curve **must** be on `Tbp`; cutting a D86 curve directly would quietly bias
every cut, so this is an error naming the fix.

One detail that earns its complexity: each cut's `tb` is the **volume-average**
of the curve across the slice, obtained by *integrating* the piecewise-linear
interpolant with panel edges at every interior knot — not by sampling the
midpoint. For a narrow cut the two agree to machine precision (the mean of a
linear function *is* its midpoint value). For a slice that straddles a knee in
the curve they differ by more than 0.5 K, and the average is the defensible one.
Both halves of that claim are asserted separately in the tests.

### 4.4 `properties`

**`PropertyMethod`** — which critical-property family:

| variant | year | gives | shape |
|---|---|---|---|
| `RiaziDaubert1980` | 1980 | M, Tc, Pc, Vc | $\theta = a\,T_b^b SG^c$ — three constants, nothing else |
| `ApiRiaziDaubert1987` | 1987 | M, Tc, Pc, Vc | the same plus an $\exp(dT_b + eSG + fT_bSG)$ factor. **The default** — the API's own recommendation |
| `KeslerLee` | 1976 | M, Tc, Pc | polynomials in Tb and SG; the refinery-standard pairing with Lee–Kesler enthalpies |
| `Twu` | 1984 | M, Tc, Pc, Vc | a **perturbation** about the n-alkane of the same boiling point |

**`ZcMethod`** — `LeeKesler | Reid | Salerno | Nath`, with `.zc(omega) -> f64`.
All four are one-liners in the acentric factor and agree to within about 0.005
over the range a petroleum cut can have; the choice rarely matters.

**`PseudoProperties`** — `{ tb, sg, watson_k, mw, tc, pc, vc, zc, omega }`, in K,
kPa, g/mol and cm³/mol.

The individual correlations are public so they can be used (and taught)
separately:

| family | functions |
|---|---|
| Riazi–Daubert 1980 | `mw_riazi_daubert_1980`, `tc_riazi_daubert_1980`, `pc_riazi_daubert_1980`, `vc_riazi_daubert_1980(tb, sg, mw)` |
| API 1987 | `mw_api_1987`, `tc_api_1987`, `pc_api_1987`, `vc_api_1987(tb, sg, mw)` |
| Kesler–Lee | `tc_kesler_lee`, `pc_kesler_lee`, `mw_kesler_lee` |
| Twu | `properties_twu(tb, sg) -> Result<(mw, tc, pc, vc)>` |

The two `vc_*` functions take `mw` because both are published on a **mass** basis
(ft³/lb and cm³/g respectively) and need the molecular weight to reach cm³/mol.
Twu's `vc` is already molar.

```rust
pub fn acentric_lee_kesler(tb: f64, tc: f64, pc: f64, sg: f64) -> Result<f64, PetroleumError>
```

Two branches on the reduced boiling point $T_{br} = T_b/T_c$:

- **$T_{br} < 0.8$** — the Lee–Kesler vapor-pressure correlation evaluated at the
  normal boiling point, where $P/P_c = 1\,\mathrm{atm}/P_c$ by definition. This
  is a *derivation*, not a fit: it is what ω means.
- **$T_{br} \ge 0.8$** — a direct Kesler–Lee fit in $K_W$ and $T_{br}$. The first
  branch degenerates as $T_{br} \to 1$ (its denominator passes through zero), and
  heavy cuts do reach 0.8, so this branch exists to cover them.

```rust
pub fn estimate(
    method: PropertyMethod,
    tb: f64,
    sg: f64,
    zc_method: ZcMethod,
) -> Result<PseudoProperties, PetroleumError>
```

The dispatcher. ω always comes from `acentric_lee_kesler` evaluated on the Tc and
Pc the chosen family produced — mixing a Kesler–Lee ω with Twu criticals would be
inconsistent. Zc and Vc are kept mutually consistent through
$Z_c = P_c V_c / (R T_c)$: families that publish a critical volume use it and
derive Zc from it; `KeslerLee`, which publishes none, takes Zc from `zc_method`
and derives Vc.

### 4.5 `cp`

| function | signature | returns |
|---|---|---|
| `ideal_gas_cp_mass` | `(watson_k: f64, t: f64) -> Result<f64>` | Cp°, **kJ/(kg·K)** |
| `ideal_gas_cp_molar` | `(watson_k: f64, mw: f64, t: f64) -> Result<f64>` | Cp°, **kJ/(kmol·K)** |
| `ideal_gas_cp_coeffs` | `(watson_k: f64, mw: f64) -> Result<[f64; 5]>` | the `Cp°/R = Σ aₖTᵏ` polynomial, T in **K** |

`ideal_gas_cp_coeffs` is the one that matters structurally: it emits exactly the
five-element array `Component::cp_coeffs` already holds, so a pseudocomponent
drops into the same `crate::energy` machinery every named compound uses, with no
special case anywhere. Kesler–Lee is quadratic in temperature, so `a₃` and `a₄`
come back zero.

All three **refuse to extrapolate** outside the fitted Watson-K window of
9.5–13.5, returning an error rather than a number nobody should trust. Silently
extrapolating a heat-capacity correlation is how a column energy balance ends up
quietly wrong.

### 4.6 `vapor_pressure`

Heavy petroleum **cracks before it boils**. A vacuum gas oil that would boil
around 500 °C at atmospheric pressure decomposes somewhere above 350 °C, so its
distillation has to be run under vacuum — and then the measured temperatures are
not boiling points at all, they are boiling points *at 10 mmHg*. Every
correlation in `properties` wants the **normal** boiling point. Maxwell–Bonnell
bridges the two, and it is what makes ASTM D1160 and D2892 vacuum data usable.

| function | signature | returns |
|---|---|---|
| `normal_boiling_point` | `(t: f64, p: f64, watson_k: Option<f64>) -> Result<f64>` | the atmospheric equivalent temperature (AET), **K** |
| `boiling_point_at_pressure` | `(tb: f64, p: f64, watson_k: Option<f64>) -> Result<f64>` | boiling temperature at `p`, **K** — the exact inverse |
| `vapor_pressure` | `(t: f64, tb: f64, watson_k: Option<f64>) -> Result<f64>` | vapor pressure, **kPa** |
| `ln_vapor_pressure` | `(t: f64, tb: f64, watson_k: Option<f64>) -> Result<f64>` | `ln` of the vapor pressure in kPa, extrapolating the outer branches — the K-value form (M20) |

`watson_k` is `Option` because the correction vanishes identically at
$K_W = 12$ — the correlation's n-hexane reference — so `None` is the honest
choice when the fraction is n-hexane-like or simply unknown.

`vapor_pressure` *is* inverted in closed form (since Milestone 20): on a fixed
`Q` branch the relation is a quadratic in $\log_{10} P$, so each of the three
branches is solved algebraically and the root inside its own branch is kept —
see §8.3 for the details and the step case. The original Brent solve survives
as fallback and as the oracle the closed form is tested against.

Two quirks of the published fit are documented and asserted rather than smoothed
over; see §8.2 and §8.3.

### 4.7 `assay`

**`GravitySpec`**:

- `ConstantWatsonK { bulk_sg }` — one gravity for the barrel; per-cut gravities
  follow from assuming chemical uniformity (§9.2).
- `Curve { fractions, sg }` — a measured gravity curve. **Prefer this whenever
  you have it**; it is strictly better information.

**`Assay`** — `{ curve, gravity, property_method, zc_method, name_prefix }`.

| method | signature | notes |
|---|---|---|
| `new` | `(curve, gravity) -> Result<Self>` | defaults to `ApiRiaziDaubert1987`, `LeeKesler`, prefix `"PC"` |
| `with_property_method` | `(self, method) -> Self` | builder |
| `with_zc_method` | `(self, method) -> Self` | builder |
| `with_name_prefix` | `(self, prefix) -> Self` | builder |
| `characterize` | `(&self, spec: &CutSpec) -> Result<Vec<Pseudocomponent>>` | the full record per cut |
| `mixture` | `(&self, spec: &CutSpec) -> Result<(Vec<Component>, Vec<f64>)>` | just what a flash needs |
| `conventional_watson_k` | `(&self, spec: &CutSpec) -> Result<f64>` | the **textbook** Watson K of the barrel (§9.2) |

**`Pseudocomponent`** — `{ cut: Cut, properties: PseudoProperties, mole_fraction: f64, component: Component }`.
The `Cut` says where in the barrel it came from, `PseudoProperties` what it is,
and `component` is the engine-ready object. Since Milestone 20 the `Component`
also carries `watson_k`, a Rackett `zra`, and a regular-solution
`solubility_param` (δ = √((ΔHᵥₐₚ − RT)/Vᴸ) at 25 °C from the cut's own Antoine
fit) — the inputs Braun K10, Peneloux and Grayson–Streed read (§11).

**Volume, weight and mole.** Distillation curves are volumetric — except ASTM
D2887, which is a chromatogram and therefore by weight. Getting that wrong
silently misweights the whole barrel, so `Assay` tracks the basis from the source
curve:

$$n_i \propto v_i \cdot SG_i / M_i \quad\text{(volume basis)}
\qquad n_i \propto w_i / M_i \quad\text{(weight basis)}$$

### 4.8 Errors

**`PetroleumError`** — four variants, all describing *input* problems except the
last. The correlations themselves are closed algebraic forms and cannot fail once
their inputs are sane.

| variant | means |
|---|---|
| `Curve(String)` | a distillation curve had fewer than two points, or mismatched axes |
| `CutPoints(String)` | fractions or cut boundaries out of range, unsorted, or duplicated |
| `InvalidInput(String)` | a gravity, temperature or pressure outside the physically meaningful range — or outside a correlation's fitted window |
| `NoConvergence(String)` | an iterative solve (Twu's molecular-weight inversion; the Maxwell–Bonnell inversion's Brent fallback) did not converge, or a vapor pressure falls outside the correlation's window |

`petroleum_error_is_input(&e) -> bool` classifies them for the binding layer,
which maps the first three to Python `ValueError` and the last to `RuntimeError`.
It lives next to the variants it classifies so the two cannot drift apart.

---

## 5. The correlations, written out

Every equation below is transcribed in the units it was **published** in. The
implementation converts at its boundary (§9.1), so these can be checked against
the source paper line by line.

### 5.1 Distillation interconversion

**Riazi–Daubert point-wise**, ref (34). $T^* = a\,T^b$, **T in K**:

| vol % interval | 0–10 | 10–30 | 30–50 | 50–70 | 70–90 | 90–95 | 95–100 |
|---|---|---|---|---|---|---|---|
| $a$ | 0.9177 | 0.5564 | 0.76517 | 0.9013 | 0.8821 | 0.9552 | 0.8177 |
| $b$ | 1.0019 | 1.09 | 1.0425 | 1.0176 | 1.0226 | 1.011 | 1.0355 |

**API Procedure 3A1.1 (D86 ↔ TBP)**, ref (35). **All in °F**:

$$T_{\mathrm{TBP},50\%} = 0.87180 \left(T_{\mathrm{D86},50\%}\right)^{1.0258}
\qquad \Delta T_{\mathrm{TBP}} = A \left(\Delta T_{\mathrm{D86}}\right)^{B}$$

| vol % interval | 0–10 | 10–30 | 30–50 | 50–70 | 70–90 | 90–100 |
|---|---|---|---|---|---|---|
| $A$ | 7.4012 | 4.9004 | 3.0305 | 2.5282 | 3.0419 | 0.11798 |
| $B$ | 0.60244 | 0.71644 | 0.80076 | 0.82002 | 0.75497 | 1.6606 |

The °F convention is not incidental. The 50 % relation's fixed point is 204 °F,
a real mid-distillate temperature; run the same coefficients in K or °R and every
result is silently biased. `riazi_example_3_3_d86_to_tbp_api_difference_method`
is the test that pins it.

**API Procedure 3A3.1 (SimDist → TBP)**, ref (35). **°F**, and the two 50 %
points are taken to be **equal** — gas chromatography separates so well that its
median already *is* the true boiling median:

$$T_{\mathrm{TBP},50\%} = T_{\mathrm{SD},50\%}
\qquad \Delta T_{\mathrm{TBP}} = C \left(\Delta T_{\mathrm{SD}}\right)^{D}$$

| vol % interval | 5–10 | 10–30 | 30–50 | 50–70 | 70–90 | 90–95 | 95–100 |
|---|---|---|---|---|---|---|---|
| $C$ | 0.15779 | 0.011903 | 0.05342 | 0.19861 | 0.31531 | 0.97476 | 0.02172 |
| $D$ | 1.4296 | 2.0253 | 1.6988 | 1.3975 | 1.2938 | 0.8723 | 1.9733 |

Note the grid starts at **5 %**, not 0 % — a chromatogram has no meaningful
initial point.

**API Procedure 3A3.2 (SimDist → D86)**, ref (35). **°F**:

$$T_{\mathrm{D86},50\%} = 0.77601 \left(T_{\mathrm{SD},50\%}\right)^{1.0395}
\qquad \Delta T_{\mathrm{D86}} = E \left(\Delta T_{\mathrm{SD}}\right)^{F}$$

| vol % interval | 0–10 | 10–30 | 30–50 | 50–70 | 70–90 | 90–100 |
|---|---|---|---|---|---|---|
| $E$ | 0.3047 | 0.06069 | 0.07978 | 0.14862 | 0.30785 | 2.6029 |
| $F$ | 1.1259 | 1.5176 | 1.5386 | 1.4287 | 1.2341 | 0.65962 |

**Edmister–Okamoto (D86 ↔ EFV)**, ref (39). $T_{\mathrm{EFV}} = a\,T_{\mathrm{D86}}^{b}\,SG^{c}$, **T in K**:

| vol % interval | 0–10 | 10–30 | 30–50 | 50–70 | 70–90 | 90–100 | 100+ |
|---|---|---|---|---|---|---|---|
| $a$ | 2.9747 | 1.4459 | 0.8506 | 3.268 | 8.2873 | 10.6266 | 7.9952 |
| $b$ | 0.8466 | 0.9511 | 1.0315 | 0.8274 | 0.6874 | 0.6529 | 0.6949 |
| $c$ | 0.4209 | 0.1287 | 0.0817 | 0.6214 | 0.934 | 1.1025 | 1.0737 |

See §8.1 for a real limitation of this table.

### 5.2 Average boiling points

API Procedure 2B1.1, from an ASTM D86 curve, **all in °F**:

$$\mathrm{VABP} = \tfrac{1}{5}(T_{10} + T_{30} + T_{50} + T_{70} + T_{90})
\qquad \mathrm{SL} = \frac{T_{90} - T_{10}}{80}$$

$$\begin{aligned}
\ln \Delta_{\mathrm{WABP}}  &= -3.062123 - 0.01829\,(\mathrm{VABP}-32)^{0.6667} + 4.45818\,\mathrm{SL}^{0.25} \\
\ln \Delta_{\mathrm{MABP}}  &= -0.563793 - 0.007981\,(\mathrm{VABP}-32)^{0.6667} + 3.04729\,\mathrm{SL}^{0.333} \\
\ln \Delta_{\mathrm{CABP}}  &= -0.23589 - 0.06906\,(\mathrm{VABP}-32)^{0.45} + 1.8858\,\mathrm{SL}^{0.45} \\
\ln \Delta_{\mathrm{MeABP}} &= -0.94402 - 0.00865\,(\mathrm{VABP}-32)^{0.6667} + 2.99791\,\mathrm{SL}^{0.333}
\end{aligned}$$

$$\mathrm{WABP} = \mathrm{VABP} + \Delta_{\mathrm{WABP}}, \qquad
\mathrm{MABP},\ \mathrm{CABP},\ \mathrm{MeABP} = \mathrm{VABP} - \Delta$$

SL is the 10–90 % slope in °F per volume percent; 80 is the span in percentage
points between the two, not a fitted constant.

### 5.3 Critical properties

**Riazi–Daubert (1980)**, ref (32), **Tb in °R, Pc in psia, Vc in ft³/lb**:

$$\begin{aligned}
M   &= 4.5673\times10^{-5}\;T_b^{2.1962}\;SG^{-1.0164} \\
T_c &= 24.2787\;T_b^{0.58848}\;SG^{0.3596} \\
P_c &= 3.12281\times10^{9}\;T_b^{-2.3125}\;SG^{2.3201} \\
V_c &= 7.5214\times10^{-3}\;T_b^{0.2896}\;SG^{-0.7666}
\end{aligned}$$

**Riazi–Daubert (1987) / API**, ref (33), **Tb in K, Pc in bar, Vc in cm³/g**:

$$\begin{aligned}
M   &= 42.965\;e^{\,2.097\times10^{-4}T_b - 7.78712\,SG + 2.08476\times10^{-3}T_b SG}\;T_b^{1.26007}\,SG^{4.98308} \\
T_c &= 9.5233\;e^{-9.314\times10^{-4}T_b - 0.544442\,SG + 6.4791\times10^{-4}T_b SG}\;T_b^{0.81067}\,SG^{0.53691} \\
P_c &= 3.1958\times10^{5}\;e^{-8.505\times10^{-3}T_b - 4.8014\,SG + 5.749\times10^{-3}T_b SG}\;T_b^{-0.4844}\,SG^{4.0846} \\
V_c &= 6.049\times10^{-2}\;e^{-2.6422\times10^{-3}T_b - 0.26404\,SG + 1.971\times10^{-3}T_b SG}\;T_b^{0.7506}\,SG^{-1.2028}
\end{aligned}$$

> **A worked consistency check, and why it is worth doing.** These two sets are
> the *same* correlations in different units, and converting one into the other
> is a cheap, powerful test of a transcription. Take the exponent on $T_b$ inside
> the exponential for $M$: $2.097\times10^{-4}$ per K. Divide by 1.8 to get per
> °R and you should land on the published °R-form value of
> $1.165\times10^{-4}$ — and you do. Then the prefactor: the °R form is 20.486,
> and $20.486 \times 1.8^{1.26007} = 42.96$, the K-form prefactor. All four
> coefficients check out this way. Doing that arithmetic *before* writing any
> code is what confirmed the K-form constants used here were remembered
> correctly rather than approximately.

**Kesler–Lee (1976)**, ref (36), **Tb in °R, Pc in psia**:

$$T_c = 341.7 + 811.1\,SG + (0.4244 + 0.1174\,SG)\,T_b + \frac{(0.4669 - 3.26238\,SG)\times10^{5}}{T_b}$$

$$\begin{aligned}
\ln P_c = \;& 8.3634 - \frac{0.0566}{SG}
 - \left(0.24244 + \frac{2.2898}{SG} + \frac{0.11857}{SG^2}\right)\!\times\!10^{-3}\,T_b \\
 &+ \left(1.4685 + \frac{3.648}{SG} + \frac{0.47227}{SG^2}\right)\!\times\!10^{-7}\,T_b^2
 - \left(0.42019 + \frac{1.6977}{SG^2}\right)\!\times\!10^{-10}\,T_b^3
\end{aligned}$$

$$\begin{aligned}
M = \;& -12272.6 + 9486.4\,SG + (4.6523 - 3.3287\,SG)\,T_b \\
 &+ (1 - 0.77084\,SG - 0.02058\,SG^2)\left(1.3437 - \frac{720.79}{T_b}\right)\frac{10^{7}}{T_b} \\
 &+ (1 - 0.80882\,SG + 0.02226\,SG^2)\left(1.8828 - \frac{181.98}{T_b}\right)\frac{10^{12}}{T_b^3}
\end{aligned}$$

**Twu (1984)**, ref (38), **Tb in °R, Pc in psia, Vc in ft³/lbmol**.

Twu is structurally different from the other three, and the difference is worth
understanding. It first asks *"what n-alkane boils at this temperature?"*,
computes that alkane's properties from a high-accuracy fit, and then **corrects**
for how far the real cut's density sits from the alkane's.

The n-alkane reference, with $\alpha = 1 - T_b/T_c^\circ$:

$$T_c^\circ = T_b\left(0.533272 + 0.191017\times10^{-3}T_b + 0.779681\times10^{-7}T_b^2 - 0.284376\times10^{-10}T_b^3 + \frac{0.959468\times10^{28}}{T_b^{13}}\right)^{-1}$$

$$\begin{aligned}
V_c^\circ &= \left[1 - \left(0.419869 - 0.505839\alpha - 1.56436\alpha^3 - 9481.70\alpha^{14}\right)\right]^{-8} \\
SG^\circ  &= 0.843593 - 0.128624\alpha - 3.36159\alpha^3 - 13749.5\alpha^{12} \\
P_c^\circ &= \left(3.83354 + 1.19629\alpha^{1/2} + 34.8888\alpha + 36.1952\alpha^2 + 104.193\alpha^4\right)^2
\end{aligned}$$

Each property then gets a density deviation $\Delta SG$ and a correction factor
$\left[(1+2f)/(1-2f)\right]^2$. When the cut *is* an n-alkane, $\Delta SG = 0$,
$f = 0$, and the correction is exactly 1 — which is why Twu's paraffin errors are
so small, and `twu_reference_recovers_n_alkane_properties_almost_exactly`
asserts precisely that.

The reference **molecular weight** is implicit: Twu publishes only
$T_b(\theta)$ with $\theta = \ln M^\circ$, so `twu_reference_mw` inverts it by
bisection over a 16–5000 g/mol bracket, and reports a `NoConvergence` error
rather than extrapolating off the bracket edge.

### 5.4 Acentric factor

**Lee–Kesler (1975)**, ref (37), for $T_{br} < 0.8$, evaluated at
$P_{br} = 1\,\mathrm{atm}/P_c$:

$$\omega = \frac{\ln P_{br} - 5.92714 + 6.09648/T_{br} + 1.28862\ln T_{br} - 0.169347\,T_{br}^6}
{15.2518 - 15.6875/T_{br} - 13.4721\ln T_{br} + 0.43577\,T_{br}^6}$$

**Kesler–Lee (1976)**, ref (36), for $T_{br} \ge 0.8$:

$$\omega = -7.904 + 0.1352\,K_W - 0.007465\,K_W^2 + 8.359\,T_{br} + \frac{1.408 - 0.01063\,K_W}{T_{br}}$$

### 5.5 Critical compressibility

| method | correlation | source |
|---|---|---|
| `LeeKesler` | $Z_c = 0.2905 - 0.085\,\omega$ | Lee & Kesler (1975), Eq. 21 |
| `Reid` | $Z_c = 0.2918 - 0.0928\,\omega$ | Reid, Prausnitz & Sherwood (1977) |
| `Salerno` | $Z_c = 0.291 - 0.08\,\omega - 0.016\,\omega^2$ | Salerno et al. (1985) |
| `Nath` | $Z_c = 0.2908 - 0.0825\,\omega$ | Nath (1985) |

### 5.6 Ideal-gas heat capacity

**Kesler–Lee (1976) / API Procedure 7D3.6**, refs (36) and (41).
**Cp° in Btu/(lb·°F), T in °R**:

$$C_p^\circ = A_0 + A_1 T + A_2 T^2$$

$$\begin{aligned}
A_0 &= -0.33886 + 0.02827\,K_W \\
A_1 &= \left(-0.9291 + 1.1543\,K_W - 0.0368\,K_W^2\right)\times 10^{-4} \\
A_2 &= -1.6658\times10^{-7}
\end{aligned}$$

That $K_W$ is the *entire* composition dependence is the striking part: one
number distinguishing a paraffin from an aromatic predicts a heat capacity to a
few percent. It works because ideal-gas Cp is essentially a count of vibrational
modes per unit mass, and hydrogen content — which is what $K_W$ measures — sets
that count. See §8.4 for the term that is missing.

### 5.7 Maxwell–Bonnell

Refs (40) and (41), API Procedure 5A1.19. A pressure-dependent group $Q$ —
three branches, because the fit changes at 2 mmHg and at 1 atm, **P in mmHg**:

$$Q = \begin{cases}
\dfrac{6.76156 - 0.987672\log_{10}P}{3000.538 - 43\log_{10}P} & P < 2 \\[2ex]
\dfrac{5.994296 - 0.972546\log_{10}P}{2663.129 - 95.76\log_{10}P} & 2 \le P < 760 \\[2ex]
\dfrac{6.412631 - 0.989679\log_{10}P}{2770.085 - 36\log_{10}P} & P \ge 760
\end{cases}$$

Then, with **T in K**:

$$T_b' = \frac{748.1\,Q\,T}{1 + T\left(0.3861\,Q - 0.00051606\right)}$$

and a Watson-K correction accounting for the fact that the whole thing was fit
on n-hexane:

$$T_b = T_b' + 1.3889\,f\,(K_W - 12)\log_{10}\!\frac{P}{760},
\qquad f = \mathrm{clamp}\!\left(\frac{1.8\,T_b - 659.67}{200},\,0,\,1\right)$$

The $(K_W - 12)$ factor is the physically meaningful part: at $K_W = 12$ the
fraction *is* the reference and the correction vanishes identically. The ramp $f$
runs from 0 at 659.67 °R (366.5 K) to 1 at 859.67 °R (477.6 K).

Because the ramp is evaluated at the *answer*, the forward relation is implicit
and is solved by fixed-point iteration — which is what makes
`boiling_point_at_pressure` an exact inverse rather than an approximate one.

> **A discrepancy resolved by measurement.** At least one open-source
> implementation drops the $(K_W - 12)$ factor on the upper $f = 1$ branch, which
> makes the correction fail to vanish at the reference. The form used here keeps
> it — and not merely on the authority of the published form. Tested against this
> crate's own Antoine equations for benzene and toluene at pressures more than
> half a decade away from atmospheric (the region where the two forms differ
> most), keeping the factor gives a mean error of **0.26 %** against **0.30 %**,
> and a worst case of **0.74 %** against **0.97 %**. Theory and measurement
> agree, so the factor stays.

### 5.8 Assay-level relations

**Rackett compressibility** (Yamada & Gunn, 1973), used to give a
pseudocomponent a liquid molar volume:

$$Z_{RA} = 0.29056 - 0.08775\,\omega$$

**Pseudo-Antoine fit** (§9.3), in the crate's reduced-Antoine form
$\ln(P^{sat}/P_c) = a_1 - a_2/(a_3 + T)$:

$$a_2 = \frac{\ln\left(P_c / 1\,\mathrm{atm}\right)}{1/T_b - 1/T_c},
\qquad a_1 = \frac{a_2}{T_c}, \qquad a_3 = 0$$

**Constant-Watson-K gravity closure** (§9.2):

$$K_W = \frac{\sum_i v_i \left(1.8\,T_{b,i}\right)^{1/3}}{SG_{\mathrm{bulk}}}
\quad\text{(volume basis)}, \qquad
K_W = \frac{1}{SG_{\mathrm{bulk}} \sum_i w_i \left(1.8\,T_{b,i}\right)^{-1/3}}
\quad\text{(weight basis)}$$

---

## 6. The Python surface

`vle.petroleum` wraps the Rust module. Model selection is by **string** rather
than enum class — `"api"`, `"kesler-lee"`, `"twu"`, `"d86"`, `"tbp"` — so a
Jupyter session does not need an import per model, and every parser lists the
valid spellings when it rejects one.

```python
from vle.petroleum import Assay

assay = Assay(
    fractions=[0.0, 0.1, 0.3, 0.5, 0.7, 0.9, 0.95],
    temperatures=[310.0, 380.0, 460.0, 540.0, 620.0, 730.0, 790.0],  # K, TBP
    basis="tbp",
    api_gravity=35.0,
)

cuts = assay.cuts(n=5)                       # list of dicts — pandas-ready
components, z = assay.components(n=30)       # list[vle.Component] + mole fractions
system, z = assay.to_system(n=30, eos="PR")  # ready to flash
```

| `Assay` method | returns |
|---|---|
| `tbp_curve()` | `(fractions, temperatures_in_K)` after conversion to TBP |
| `watson_k(n=30, ...)` | the barrel's textbook Watson K |
| `cuts(n=None, boundaries=None, equal_temperature=False)` | one dict per pseudocomponent |
| `components(...)` | `(list[Component], list[float])` |
| `to_system(..., **system_kwargs)` | `(vle.System, list[float])` |

Every cut dict carries: `index`, `name`, `fraction`, `mole_fraction`, `x_lower`,
`x_upper`, `t_lower`, `t_upper`, `tb`, `sg`, `api_gravity`, `watson_k`, `mw`,
`tc`, `pc`, `vc`, `zc`, `omega`, `cp_coeffs`, `psat_coeffs`, `zra`,
`liquid_volume`, `solubility_param` (the last added in M20). A dict rather than
a class because this is data to tabulate —
`pandas.DataFrame(cuts)` gives you the cut summary a refinery engineer expects.

Module-level functions mirror the Rust ones: `watson_k`, `api_from_sg`,
`sg_from_api`, `average_boiling_points`, `convert_curve`, `cut_curve`,
`estimate`, `acentric_factor`, `ideal_gas_cp`, `ideal_gas_cp_coeffs`,
`normal_boiling_point`, `boiling_point_at_pressure`, `vapor_pressure`. Plus the
constants `PROPERTY_METHODS` and `DISTILLATION_BASES`. The Milestone 20 surface
— `vle.refinery` and the `System` methods `flash_free_water`,
`lee_kesler_pseudocritical`, `lee_kesler_departure`,
`enthalpy_entropy_lee_kesler`, `peneloux_shifts`, `translated_molar_volume`,
`translated_density`, plus `liquid_model="grayson_streed"` / `"bk10"` — is
described in §11.

**Unit strings work everywhere a scalar temperature or pressure is accepted**,
resolved through the same `pint` registry as the rest of the package:

```python
petroleum.normal_boiling_point("365 degF", "10 mmHg", 12.5)   # -> 602.89 K
assay.cuts(boundaries=["175 degC", "340 degC"])                # product cuts
```

Errors map the way the rest of the bindings do: bad input becomes `ValueError`,
a failed solve becomes `RuntimeError`.

---

## 7. Validation — what was checked against what

A correlation module is only as trustworthy as its oracles, so it is worth being
explicit about which oracle covers which claim.

### 7.1 Distillation interconversion — published worked examples

| test | source | result |
|---|---|---|
| `riazi_example_3_3_d86_to_tbp_power_law` | Riazi (2005) Ex. 3.3 | matches to < 0.15 °C |
| `riazi_example_3_3_d86_to_tbp_api_difference_method` | Riazi (2005) Ex. 3.3 | matches to < 0.15 °C |
| `riazi_example_3_4_simdist_to_tbp` | Riazi (2005) Ex. 3.4 | matches to < 0.15 °C |
| `riazi_example_3_5_simdist_to_d86` | Riazi (2005) Ex. 3.5 | matches to < 0.15 °C |
| `riazi_example_3_2_tbp_to_d86_and_on_to_efv` | Riazi (2005) Ex. 3.2 | matches to < 0.5 °C |
| `api_data_book_example_d86_to_tbp_in_fahrenheit` | API TDB | matches to < 0.15 °F |
| `api_data_book_example_simdist_to_tbp_in_fahrenheit` | API TDB | matches to < 0.15 °F |

Published examples only exercise the **forward** direction, so every inverse is
additionally covered by a round-trip test to 1e-8.

### 7.2 Property correlations — measured pure-component data

The plan called for per-cut API Technical Data Book worked examples. None was
obtainable, so the correlations are validated instead against **measured**
Tc/Pc/ω/M/Vc for ten pure hydrocarbons from this repo's bundled component
database: n-C5 … n-C10, benzene, toluene, cyclohexane, methylcyclohexane — six
paraffins, two aromatics, two naphthenes.

That substitution is arguably the *stronger* test. These correlations are
**fitted** to pure-hydrocarbon data, so reproducing measured values is exactly
what they claim to do — and a mistyped coefficient cannot match across ten
compounds spanning three chemical families by accident. It is what caught two
transcription errors during implementation: a sign flip in the Lee–Kesler ω
numerator, and a psia-vs-bar mix-up in Twu's critical pressure that showed up as
a 1300 % error.

Worst absolute deviation over that set:

| family | Tc | Pc | M | Vc |
|---|---|---|---|---|
| Riazi–Daubert 1980 | 2.5 % | 8.2 % | 8.3 % | 4.6 % |
| **API / R–D 1987** (default) | **1.3 %** | **5.1 %** | **6.0 %** | **4.0 %** |
| Kesler–Lee | 1.9 % | 5.5 % | 8.1 % | — |
| Twu | 1.8 % | 15.3 % | 7.4 % | 6.0 % |
| acentric factor ω | < 1.6 % on all ten | | | |

Read those as **lower bounds** on the error you will see in practice: these are
pure compounds in the middle of the fitted range. A real vacuum-residue cut is
far outside it.

The clearest pattern is that **Twu's Pc is excellent on paraffins (< 1 %) and
poor on aromatics (11–15 %)** — exactly what its n-alkane-perturbation structure
predicts. `twu_is_the_best_method_on_paraffins_and_the_worst_on_aromatics`
asserts both halves, so the module's advice ("pick it for a paraffinic crude")
cannot go stale.

### 7.3 Maxwell–Bonnell — an independent oracle

Validated against this crate's own **Antoine equations**: feed the correlation a
hydrocarbon's real vapor pressure at some temperature and ask for its normal
boiling point. Over seven hydrocarbons and 320–520 K (77 usable points) the mean
error is **0.19 %** and the worst **1.09 %**. Nothing in the module was fitted to
those Antoine coefficients, which is what makes this a real test rather than a
tautology.

### 7.4 Ideal-gas Cp — measured heat-capacity polynomials

Checked against the measured `Cp°/R` polynomials in the bundled database over
**300–1000 K**:

| family | worst deviation |
|---|---|
| n-paraffins | **2.8 %** |
| aromatics | **3.1 %** |
| naphthenes | **15.9 %** — see §8.4 |

### 7.5 End to end

`pseudocomponents_drive_a_real_flash` characterizes an assay, hands the
components straight to `flash_isothermal` with no special casing, and asserts a
two-phase split with a component mass balance closed to 1e-12 and K-values
ordered light-to-heavy. That is the test that says the whole design constraint
held.

**Totals:** 125 Rust unit tests in `petroleum/` (121 from M19 plus the four closed-form-inversion tests M20 added to `vapor_pressure.rs`), 37 wheel-level Python tests; the refinery layer of §11 adds its own 33 Rust and 13 Python tests, counted there.

---

## 8. Known gaps, and why they are gaps

This repo's `CLAUDE.md` requires that partial completion be stated rather than
rounded up. These four are the module's honest edges. Each is documented at its
call site *and* asserted in a test, so none can be mistaken later for a
regression — or forgotten.

> **Re-examined for Milestone 20 (2026-08-16).** Every gap was revisited with
> the explicit goal of closing it. Verdicts: **8.1** and **8.2** are properties
> of the published correlations and stay as they are; **8.3** is narrowed — the
> inversion is now closed-form and deterministic at the step, though the step
> itself is still in the fit; **8.4** stays open, but the reason has changed
> from "no source found" to "two secondary sources found, and the evidence
> says not to ship it" — the evidence is recorded below so nobody repeats
> the search.

### 8.1 Edmister–Okamoto's EFV initial point

**What:** D86 → EFV fails on any curve that includes a 0 % point unless the feed
is very wide-boiling (roughly a 250 K span or more).

**Why:** the rows of a point-wise coefficient table are fitted independently, and
nothing forces the converted curve to stay monotone. Edmister–Okamoto's 0–10 %
row genuinely crosses its 10–30 % neighbour, so the converted EFV initial point
lands *above* the 10 % point. `DistillationCurve::new` rejects the decreasing
result rather than returning nonsense.

**What to do:** convert a **10–90 % curve**, dropping the initial point. A D86
initial boiling point is the least reproducible number on the whole report
anyway. Pinned by `efv_initial_point_row_crosses_its_neighbour_on_narrow_feeds`,
which also asserts that the workaround works and that a wide feed converts fine.

This one was found by the milestone notebook failing to execute — not by reading
the coefficients.

### 8.2 Maxwell–Bonnell is not an identity at 1 atm

Asking for the boiling point of a fraction at exactly 760 mmHg does not return
its normal boiling point exactly; it overshoots by **+0.22 K at Tb = 350 K rising
to +0.37 K at Tb = 750 K**. The Watson correction vanishes there
($\log_{10}(P/760) = 0$), so what is left is the raw $Q$ relation — and being an
empirical fit rather than an identity, it carries a residual. Well inside the
correlation's own stated accuracy. Asserted from both sides by
`is_nearly_but_not_exactly_an_identity_at_one_atmosphere`.

### 8.3 Maxwell–Bonnell steps at its branch boundary

The sub- and super-atmospheric $Q$ fits do not meet exactly, putting a
**0.35–0.55 K step** into any boiling-point curve that crosses 1 atm. It is
small, but it is a true discontinuity, and it is a property of the published
correlation, not of this implementation.
`the_branch_boundaries_step_by_the_documented_amount` pins the magnitude from
both sides.

**Narrowed in Milestone 20.** `vapor_pressure` used to invert the relation
numerically (Brent on $\log_{10} P$), and near the step it could converge on
either side of it depending on the bracket. It is now inverted **in closed
form**: for a fixed $Q$ branch the boiling-point relation is a quadratic in
$L = \log_{10} P$ (linear without the Watson correction), so each branch is
solved algebraically and the root inside its own branch is kept — verified
against the forward relation, agreeing with the old Brent oracle to $< 10^{-7}$
over 500+ points (`closed_form_inversion_matches_the_brent_oracle_everywhere`).
A temperature that falls *inside* the step has no root on any branch; that
case is detected and returns exactly the boundary pressure, deterministically
(`a_temperature_inside_the_branch_step_returns_the_boundary_pressure`). The
step is still there — the answer is still only correct to within its width —
but the behaviour at it is now defined rather than bracket-dependent, and the
routine costs a few dozen flops instead of a root solve, which is what let
Braun K10 use it per component per stage. The K-value form,
`ln_vapor_pressure`, additionally extends the outer branches past the
$10^{-4}$–$5\times10^{4}$ mmHg window so a light end far above its critical
point yields "$K \gg 1$" instead of an error.

### 8.4 The Kesler–Lee `CF` correction is not implemented

**This is the module's one genuinely missing feature.** Kesler & Lee's paper
carries a correction factor `CF`, applied when $10 < K_W < 12.8$, specifically to
fix ring compounds. Naphthenes are the only one of the three hydrocarbon families
that sits inside that window — which is exactly why they are the outlier in the
accuracy table.

**The measured cost: ideal-gas Cp° is up to 15.9 % low on naphthenes.** For a
paraffinic or aromatic assay the correlation is as good as its published
accuracy.

`naphthenes_are_the_documented_weak_spot` asserts the deviation is *both* under
16 % and over 5 %. The second half is deliberate: if somebody implements `CF`,
that test fails, and the right response is to tighten the bound and rewrite the
accuracy table — not to delete the test.

**What the Milestone 20 search found (2026-08-16), so it is not repeated.**
The primary source (Kesler & Lee, *Hydrocarbon Process.* 1976) was not
obtainable. Two *secondary* transcriptions of the ω-based `CF` term were:
DWSIM's `FluidProperties.vb` (`Cpig_lk`) and a 2025 review in *J. Chem. Petrol.
Eng.* 59(2), 405–426, which reproduces it as

$$C_p^{ig} = A_0 + A_1 T + A_2 T^2 - CF\,(B_0 + B_1 T + B_2 T^2), \qquad
CF = \left[\frac{(12.8 - K_W)(10 - K_W)}{10\,\omega}\right]^2$$

with $B_0 = 1.09223 - 2.48245\,\omega$, $B_2 = -(7.2661 - 9.2561\,\omega)\times10^{-7}$
and — here they disagree — $B_1 = -(3.434 - 7.14\,\omega)\times10^{-3}$ (the
review) versus $\times10^{-4}$ (DWSIM). The $A$ coefficients in both match this
module's implemented base exactly once converted to SI, which confirms the base
and makes the disagreement real rather than a units artefact. Both variants
were then evaluated against the same measured Cp° polynomials the accuracy
table uses:

| variant | cyclohexane 500 K | cyclohexane 1000 K | methylcyclohexane 1000 K |
|---|---|---|---|
| implemented (no `CF`) | −8.0 % | −15.9 % | −11.9 % |
| `CF`, $B_1 \times 10^{-3}$ | **+8.6 %** | **+20.1 %** | **+21.0 %** |
| `CF`, $B_1 \times 10^{-4}$ | **−18.7 %** | −12.9 % | −8.6 % |

Neither improves the naphthenes; one makes them worse everywhere. The likely
reason is that $CF \propto 1/\omega^2$ was fitted to *fractions* (ω ≳ 0.3), and
pure ring compounds at ω ≈ 0.21 sit where the term blows up. The same review
also gives the SG-based API form ($CF_0 = [100(12.8/K_W - 1)(10/K_W - 1)]^2$)
whose *base* is a different fit; that base does better on naphthenes at high T
(−11 % vs −16 %) but worse on paraffins at 1000 K (+4.4 % vs −1 %), and its
$CF_0$ term moves the naphthenes by only 1–3 points. So the verdict stands, for
a better reason than before: shipping either `CF` would trade a documented gap
on one family for an undocumented regression on another, on the strength of a
coefficient two sources cannot agree on. Closing this needs the primary paper.

---

## 9. Design decisions worth knowing

### 9.1 Units convert at the boundary, never in the coefficients

Nearly every correlation here was published in °R and psia, and several in °F.
The crate's canonical units are K and kPa. **Every public function takes and
returns canonical units; the imperial conversion happens inside, next to the
correlation, with the published form written out in the units it was published
in.**

The temptation is to pre-convert the coefficients once and store the
"simplified" K-form. Do not: that is exactly how a transcription error becomes
invisible. Keeping the published form intact means any constant can be checked
against the paper by eye — and, as §5.3 shows, cross-converting between two
published unit systems is itself a strong test.

### 9.2 Constant Watson K is chosen to close the gravity exactly

When an assay reports only a bulk gravity, per-cut gravities have to come from
somewhere. The standard assumption is chemical uniformity — one Watson factor for
the whole barrel — which gives $SG_i = (1.8\,T_{b,i})^{1/3}/K_W$ and leaves $K_W$
as a free parameter.

This module pins it by **requiring that the cuts blend back to the bulk gravity
you supplied**. Because $SG_i \propto 1/K_W$, the blending rule is linear in
$1/K_W$ and inverts in closed form — no iteration (§5.8). The characterized assay
therefore conserves volume and mass *exactly* rather than approximately, which
`constant_watson_k_cuts_blend_back_to_the_bulk_gravity_exactly` asserts to 1e-12.

**The trade-off, stated:** that $K_W$ is anchored on the **cubic** average boiling
point, whereas the textbook definition of Watson K uses the **mean** average. The
two differ by a few hundredths on a realistic crude. Exact gravity closure was
judged worth more than agreeing with the convention in the third decimal, and
`Assay::conventional_watson_k` reports the textbook value for anyone who needs
it. A test asserts the two differ but by less than 0.2, so the claim in both
directions stays honest.

### 9.3 The vapor-pressure fit is anchored on Tb and Tc

A pseudocomponent has no measured vapor-pressure data at all. But it has two
exact points on its own curve: $P^{sat}(T_b) = 1$ atm — the *definition* of the
normal boiling point, and the number the whole characterization was built around
— and $P^{sat}(T_c) = P_c$.

Two conditions determine two coefficients, so $a_3$ is set to zero and the fit
becomes the two-point Clausius–Clapeyron line in $\ln P$ against $1/T$ (§5.8).

Anchoring there rather than fitting a corresponding-states correlation matters:
it *guarantees* the pseudocomponent boils at the temperature the assay says it
boils at. A Riedel or Lee–Kesler `Psat` would be smoother across the middle but
would miss the boiling point by a few kelvin — and the boiling point is the
measurement.

### 9.4 Zc and Vc are never allowed to disagree

$Z_c = P_c V_c/(R T_c)$ is an identity, not a correlation. Families that publish
a critical volume use it and *derive* Zc; `KeslerLee`, which publishes none,
takes Zc from a corresponding-states correlation and *derives* Vc. Either way the
identity holds to machine precision, asserted for all four families.

### 9.5 Out of range is an error, not a number

`ideal_gas_cp_*` refuses Watson K outside 9.5–13.5. `properties_twu` refuses a
boiling point outside its n-alkane bracket. `vapor_pressure` refuses a pressure
outside 10⁻⁴–5×10⁴ mmHg (its K-value sibling `ln_vapor_pressure` is the one
deliberate exception — §8.3 says why). `cut_curve` refuses boundaries outside
the curve.

Silently extrapolating a correlation is how a column energy balance ends up
quietly wrong. Every one of these errors names what went wrong and, where there
is one, the fix.

### 9.6 Cut boiling points are integrated, not sampled

Each cut's `tb` is the exact volume-average of the interpolant across the slice,
with panel edges at every interior knot — not the midpoint value. For narrow cuts
the two are identical (the mean of a linear function is its midpoint value, to
machine precision); for a wide cut straddling a knee they differ by more than
0.5 K. Both halves are asserted, so the extra machinery has to keep earning its
place.

---

## 10. Recipes

**Characterize a D86 assay into 50 pseudocomponents:**

```python
from vle.petroleum import Assay
assay = Assay(fractions=[...], temperatures=[...], basis="d86", api_gravity=32.0)
system, z = assay.to_system(n=50, eos="PR")
```

**Reproduce a refinery's product yields:**

```python
cuts = assay.cuts(boundaries=["175 degC", "235 degC", "340 degC", "370 degC"])
for name, c in zip(["naphtha", "kero", "diesel", "AGO", "residue"], cuts):
    print(name, round(100 * c["fraction"], 1), "vol %", round(c["api_gravity"], 1), "degAPI")
```

**Convert a vacuum-distillation temperature to an atmospheric equivalent:**

```python
from vle.petroleum import normal_boiling_point
tb = normal_boiling_point("365 degF", "10 mmHg", 12.5)   # -> 602.89 K
```

**Compare correlation families on one cut:**

```python
from vle import petroleum
for m in petroleum.PROPERTY_METHODS:
    p = petroleum.estimate(500.0, 0.82, method=m)
    print(f"{m:22s} Tc={p['tc']:7.1f} K  Pc={p['pc']:7.1f} kPa  M={p['mw']:6.1f}")
```

**Use a measured gravity curve instead of an assumption:**

```python
assay = Assay(
    fractions=[...], temperatures=[...], basis="tbp",
    sg_fractions=[0.0, 0.5, 1.0], sg_values=[0.70, 0.85, 0.98],
)
```

---

## 11. Refinery thermodynamics — Milestone 20

*Module: `vle_thermo::refinery` + `flash::free_water` + two `LiquidModel`s;
Python: `vle.refinery` and seven `System` methods; notebook
[`16_refinery_thermodynamics.ipynb`](../../../notebooks/16_refinery_thermodynamics.ipynb).*

Everything above turns an assay into `Component`s and hands them to a cubic
EOS. That is a legitimate crude-tower model, but it is not what a crude tower
is *validated against*: refinery practice standardised — decades before cubic
EOS were trusted for liquids — on a specific set of older methods, and plant
data and textbook design cases are stated in them. Milestone 20 adds them, and
adds the free-water handling that stripping steam makes unavoidable. All of it
is built for the *outer loop* of an inside-out column solver: O(N) per stage
for the K-value methods, one O(N²) mixing pass for Lee–Kesler, and nothing
allocated inside an iteration.

### 11.1 What shipped, and the two scope decisions

| need | method | where | stated limit |
|---|---|---|---|
| a second liquid from stripping steam | **free-water (decant) flash** | `flash::free_water::flash_free_water`, `System.flash_free_water` | the industry's *approximation*, not a three-liquid stability search: cannot find a second hydrocarbon liquid; neglects water dissolved in the oil (~10⁻⁴) |
| K-values in hydrogen-rich / heavy service | **Grayson–Streed** $K_i = \nu_i\gamma_i/\hat\phi_i^V$ | `LiquidModel::GraysonStreed`; $\nu$ in `eos::regular_solution_ln_nu` | γ ≡ 1 when a component has no solubility parameter |
| K-values for heavy fractions at low pressure | **Braun K10** $K_i = P^{MB}_i(T; T_{b,i}, K_{W,i})/(\hat\phi_i^V P)$ | `LiquidModel::BraunK10` | Braun's pressure-correction charts are not implemented — the K10 value scaled Raoult-style |
| enthalpy / entropy | **Lee–Kesler** three-parameter corresponding states, pure and mixture | `refinery::lee_kesler`, `System.enthalpy_entropy_lee_kesler` | — |
| heavy-liquid density | **Peneloux** volume translation, SRK and PR | `refinery::volume_translation`, `System.translated_density` | the PR constants are the standard adaptation of Peneloux's SRK fit, not Peneloux's own |

### 11.2 The free-water model

Water and hydrocarbons are immiscible to a few parts in $10^4$, so a
two-phase (V–L) flash of a steam-containing feed is the wrong problem: it
converges — Rachford–Rice always has a root — to a single "liquid" that is a
few percent water in naphtha, which does not exist. The decant model states
the immiscibility as an assumption:

1. the hydrocarbon liquid contains no water;
2. a free-water phase, when present, is pure water, and the vapor is
   saturated with it — $y_w P = P^{sat}_w(T)$;
3. the hydrocarbons flash **as if the water were not there**, at their partial
   pressure $P_{hc} = P - y_w P$, with whatever models the `SystemSpec`
   carries (cubic, γ-φ, Grayson–Streed, BK10).

Case A assumes free water: $y_w = P^{sat}_w/P$, one dry flash at $P - P^{sat}_w$,
water in the vapor $= V_{hc}\,y_w/(1-y_w)$, free water by difference. If that
goes negative — or if $P^{sat}_w \ge P$ — case B: all water is vapor, and
$y_w = z_w/(V_{hc} + z_w)$ couples to the dry flash through $P_{hc}$; a short
fixed point closes it. The dry flashes are warm-started and reuse the spec
unchanged (the engine's flash tolerates a zero-feed component), so the whole
thing costs one or a few ordinary flashes. Water's $P^{sat}$ comes from the
water component's own saturation model unless an IF97 value is passed in.

### 11.3 Grayson–Streed, and a correction to the legacy record

The framework is $K_i = \nu_i \gamma_i / \hat\phi_i^V$: $\nu_i$ the pure-liquid
fugacity coefficient from the correlation
$\log_{10}\nu = \nu^0(T_r, P_r) + \omega\,\nu^1(T_r, P_r)$, $\gamma_i$ from
Scatchard–Hildebrand regular-solution theory ($\ln\gamma_i =
V_i(\delta_i - \bar\delta)^2/RT$), $\hat\phi_i^V$ from an EOS (Redlich–Kwong
classically). Two facts about the legacy Pascal code surfaced while adding it,
both now stated in `eos.rs`: the coefficient table it carries under the name
"Chao–Seader" is **Grayson & Streed's 1963 refit** (verified against two
independent sources — Ref (4) `TERMOII.PAS` and COMSOL's Table 2-2), and its
K-value never applied $\gamma_i$. `LiquidModel::ChaoSeader` is kept exactly as
it was and documented for what it is; `LiquidModel::GraysonStreed` is the
complete method; the 1961 Chao–Seader table is available as
`RegularSolutionSet::ChaoSeader1961` (verified against the FOSSEE OpenModelica
report and DWSIM). Hydrogen and methane get their special rows by name.
$\nu_i$ and the $\gamma$ constants are composition-independent, so the flash
hoists them into `SystemTpCache` and an iteration pays only the vapor
$\hat\phi$ and an O(N) $\gamma$. Pseudocomponents from §4.7 now carry a
solubility parameter — $\delta = \sqrt{(\Delta H_{vap} - RT)/V^L}$ at 25 °C from
the cut's own Antoine fit — so an assay's Grayson–Streed system has a real $\gamma$.

### 11.4 Braun K10 and the closed-form Maxwell–Bonnell

BK10 is a chart method: the K-value at 10 psia as a function of temperature and
normal boiling point, from Maxwell–Bonnell vapor pressures. Here it is
$K_i = P^{MB}_i(T; T_{b,i}, K_{W,i}) / (\hat\phi_i^V P)$ — the K10 value scaled
Raoult-style, without Braun's pressure-correction charts (the method's
low-pressure validity range assumes as much). What made it affordable is §8.3's
closed-form inversion: the $\ln P^{sat}$ per component is a few dozen flops and
cached per $(T, P)$, so BK10 is the cheapest K-value method in the engine per
stage. Pseudocomponents carry `Component::watson_k` for the correction; a
named compound leaves it at zero, which reads as "no correction".

### 11.5 Lee–Kesler

The 1975 three-parameter corresponding-states correlation: two reduced
Benedict–Webb–Rubin fluids (simple, $\omega = 0$; reference n-octane,
$\omega_r = 0.3978$), each solved for $V_r$ at $(T_r, P_r)$, and every property
interpolated linearly in $\omega$. Departure functions $(H - H^\circ)/RT_c$,
$(S - S^\circ)/R$, $\ln(f/P)$ and $Z$ follow analytically; mixtures through the
Lee–Kesler pseudo-critical rules ($\eta = 1$) or Plöcker's ($\eta = 0.25$).
It was validated **by identity rather than by transcribed table values**:
$(H - H^\circ)/RT_c = -T_r^2\,\partial\ln(f/P)/\partial T_r|_{P_r}$ numerically to
$2\times10^{-5}$ and $(S - S^\circ)/R = (H - H^\circ)/RT - \ln(f/P)$ to $10^{-10}$
at six states on both fluids — every $b, c, d, \beta, \gamma$ enters all three
formulas, so a transcription error cannot pass. $B(T_r)$ is checked against
Pitzer's $B^0/B^1$, and methane vapor against PR to 10–20 % (their known
disagreement). Root selection is a log-grid sign scan, Brent, and a Newton
polish; smallest bracket = liquid, largest = vapor, one bracket = both. Cost,
measured from Python at N = 300: **0.10 ms per mixture enthalpy** — the first
draft cost 0.31 ms because of a `powf` in the O(N²) loop, replaced by two
square roots for $\eta = 0.25$.

### 11.6 Peneloux

A constant per-component volume shift $c_i$ subtracted from the EOS volume,
$c_i = 0.40768\,(RT_c/P_c)(0.29441 - Z_{RA})$ for SRK and
$0.50033\,(RT_c/P_c)(0.25969 - Z_{RA})$ for PR, with $Z_{RA}$ from
`Component::zra` or $0.29056 - 0.08775\,\omega$. It changes every fugacity by the
same factor in both phases, so **no K-value moves** — the shift is not in the
fugacity path at all — while the liquid density is fixed where a bare cubic is
10–15 % light: n-heptane 596 → 691 kg/m³ (measured 684), n-decane 598 → 743
(727), SRK at 25 °C.

## 12. Where to go next

**Milestone 20 — Refinery Thermodynamics — has shipped** (§11 above), so this
module's sequel exists: the methods a refinery column is validated against,
built on the pseudocomponents made here. Its design record is
[`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](../../plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md) §2
(U4, U5); notebook 16 walks each method on the Milestone 19 assay and rebuilds
a Grayson–Streed K-value by hand; the Rust module docs
(`engine/src/refinery/mod.rs`, `lee_kesler.rs`, `volume_translation.rs`,
`engine/src/flash/free_water.rs`) carry the equations in full.

Beyond that, the downstream `stages-thermo` project consumes both modules for
its crude-tower work — see
[`PETROLEUM_PSEUDOCOMPONENT_PLAN.md`](../../plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md) §3.
What that project still needs from *this* repo is now short: the column side
of free water (D6 there), and, someday, the Kesler–Lee `CF` correction (§8.4)
with a primary source behind it.

---

## 13. Academic references

Numbering continues the crate-wide reference list in
[`MODERNIZATION_PLAN.md`](../../plans/MODERNIZATION_PLAN.md).

- **(31)** Riazi, M. R. *Characterization and Properties of Petroleum Fractions*;
  ASTM Manual Series MNL50: West Conshohocken, PA, **2005**.
  *The standard text. Examples 3.2–3.5 are the test oracles used here.*
- **(32)** Riazi, M. R.; Daubert, T. E. Simplify Property Predictions.
  *Hydrocarbon Processing* **1980**, *59* (3), 115–116.
  *The two-parameter Tb + SG correlations.*
- **(33)** Riazi, M. R.; Daubert, T. E. Characterization Parameters for Petroleum
  Fractions. *Ind. Eng. Chem. Res.* **1987**, *26* (4), 755–759.
  *The extended exponential form adopted by the API. This module's default.*
- **(34)** Riazi, M. R.; Daubert, T. E. Analytical Correlations Interconvert
  Distillation Curve Types. *Oil & Gas Journal* **1986**, *84*, 50–57.
  *The point-wise D86 ↔ TBP power laws.*
- **(35)** Daubert, T. E. Petroleum Fraction Distillation Interconversion.
  *Hydrocarbon Processing* **1994**, *73* (9), 75–78.
  *API Procedures 3A1.1, 3A3.1 and 3A3.2 — the difference methods.*
- **(36)** Kesler, M. G.; Lee, B. I. Improve Prediction of Enthalpy of Fractions.
  *Hydrocarbon Processing* **1976**, *55* (3), 153–158.
  *Critical properties, molecular weight, the heavy-branch acentric factor, and
  the ideal-gas Cp° correlation (with the `CF` term this module does not
  implement — §8.4).*
- **(37)** Lee, B. I.; Kesler, M. G. A Generalized Thermodynamic Correlation Based
  on Three-Parameter Corresponding States. *AIChE Journal* **1975**, *21* (3),
  510–527.
  *The acentric-factor definition used for Tbr < 0.8, and the Zc correlation;
  since M20 also the departure functions and mixing rules of §11.5.*
- **(38)** Twu, C. H. An Internally Consistent Correlation for Predicting the
  Critical Properties and Molecular Weights of Petroleum and Coal-Tar Liquids.
  *Fluid Phase Equilibria* **1984**, *16*, 137–150.
  *The n-alkane perturbation method.*
- **(39)** Edmister, W. C.; Okamoto, K. K. Applied Hydrocarbon Thermodynamics,
  Part 13: Equilibrium Flash Vaporization Correlations for Petroleum Fractions.
  *Petroleum Refiner* **1959**, *38* (9), 271–288.
  *The D86 ↔ EFV conversions.*
- **(40)** Maxwell, J. B.; Bonnell, L. S. *Vapor Pressure Charts for Petroleum
  Engineers*; Esso Research and Engineering: Linden, NJ, **1955**; also
  *Ind. Eng. Chem.* **1957**, *49*, 1187.
  *The vacuum ↔ atmospheric boiling-point correlation.*
- **(41)** American Petroleum Institute. *Technical Data Book — Petroleum
  Refining*, 6th ed.
  *Procedures 2B1.1 (average boiling points), 3A1.1 / 3A3.1 / 3A3.2
  (interconversion), 5A1.19 (Maxwell–Bonnell), 7D3.6 (ideal-gas Cp°).*

**Milestone 20 (§11)** — numbering continues in `engine/src/refinery/mod.rs`:

- **(42)** Grayson, H. G.; Streed, C. W. Vapor-Liquid Equilibria for High
  Temperature, High Pressure Hydrogen-Hydrocarbon Systems. *6th World Petroleum
  Congress*, Frankfurt, **1963**, Sect. VII, Paper 20, 233–245.
  *The ν⁰ refit every refinery package ships — and, it turned out, the table
  the legacy Pascal carried under the Chao–Seader name (§11.3).*
- **(43)** Chao, K. C.; Seader, J. D. A General Correlation of Vapor-Liquid
  Equilibria in Hydrocarbon Mixtures. *AIChE Journal* **1961**, *7* (4), 598–605.
  *The original framework `Kᵢ = νᵢγᵢ/φ̂ᵢⱽ` and the 1961 table.*
- **(44)** Péneloux, A.; Rauzy, E.; Fréze, R. A Consistent Correction for
  Redlich-Kwong-Soave Volumes. *Fluid Phase Equilibria* **1982**, *8*, 7–23.
  *Volume translation (§11.6).*
- **(45)** Plöcker, U.; Knapp, H.; Prausnitz, J. M. Calculation of High-Pressure
  Vapor-Liquid Equilibria from a Corresponding-States Correlation with Emphasis
  on Asymmetric Mixtures. *Ind. Eng. Chem. Process Des. Dev.* **1978**, *17*
  (3), 324–332.
  *The `η = 0.25` pseudo-critical exponent (§11.5).*

Supporting correlations cited inline: Yamada & Gunn (1973) for the Rackett
compressibility; Reid, Prausnitz & Sherwood (1977), Salerno et al. (1985) and
Nath (1985) for the alternative Zc correlations; Pedersen & Christensen,
*Phase Behavior of Petroleum Reservoir Fluids* (2007) for the PR adaptation of
the Peneloux shift; the Braun K10 charts are Maxwell–Bonnell (40) restated at
10 psia.

### Historical sources (context only — none of these is implemented here)

These are the citations behind the chronology in §1. They are deliberately
**not** given crate reference numbers: the numbered list is reserved for papers
whose algorithms this codebase implements, and none of these is one. Sorel
through Underwood is the pre-computer design canon; Amundson & Pontinen onward
is what changed when the arithmetic moved onto a machine.

- Sorel, E. *La Rectification de l'Alcool*; Gauthier-Villars: Paris, **1893**.
- Lewis, W. K. The Efficiency and Design of Rectifying Columns for Binary
  Mixtures. *Ind. Eng. Chem.* **1922**, *14*, 492.
- Ponchon, M. Étude graphique de la distillation fractionnée.
  *Tech. Moderne* **1922**, *13*, 20 and 53.
- Savarit, R. *Arts et Métiers* **1922**, *65*, 142, 178, 241 and 307.
- McCabe, W. L.; Thiele, E. W. Graphical Design of Fractionating Columns.
  *Ind. Eng. Chem.* **1925**, *17*, 960.
- Fenske, M. R. Fractionation of Straight-Run Pennsylvania Gasoline.
  *Ind. Eng. Chem.* **1932**, *24*, 482.
- Lewis, W. K.; Matheson, G. L. Studies in Distillation. Design of Rectifying
  Columns for Natural and Refinery Gasoline. *Ind. Eng. Chem.* **1932**,
  *24*, 494.
- Thiele, E. W.; Geddes, R. L. Computation of Distillation Apparatus for
  Hydrocarbon Mixtures. *Ind. Eng. Chem.* **1933**, *25*, 289.
- Watson, K. M.; Nelson, E. F. Improved Methods for Approximating Critical and
  Thermal Properties of Petroleum Fractions. *Ind. Eng. Chem.* **1933**,
  *25*, 880. — *the characterization factor implemented as* `watson_k`.
- Gilliland, E. R. Multicomponent Rectification. Estimation of the Number of
  Theoretical Plates as a Function of the Reflux Ratio. *Ind. Eng. Chem.*
  **1940**, *32*, 1220.
- Underwood, A. J. V. Fractional Distillation of Multicomponent Mixtures.
  *Chem. Eng. Prog.* **1948**, *44*, 603.
- Amundson, N. R.; Pontinen, A. J. Multicomponent Distillation Calculations on a
  Large Digital Computer. *Ind. Eng. Chem.* **1958**, *50*, 730.
- Wang, J. C.; Henke, G. E. Tridiagonal Matrix for Distillation.
  *Hydrocarbon Process.* **1966**, *45* (8), 155.
- Naphtali, L. M.; Sandholm, D. P. Multicomponent Separation Calculations by
  Linearization. *AIChE J.* **1971**, *17* (1), 148.
- Boston, J. F.; Sullivan, S. L. A New Class of Solution Methods for
  Multicomponent, Multistage Separation Processes. *Can. J. Chem. Eng.*
  **1974**, *52* (1).
- Boston, J. F.; Britt, H. I. A Radically Different Formulation and Solution of
  the Single-Stage Flash Problem. *Comput. Chem. Eng.* **1978**, *2*, 109–122.
  — *the inside-out algorithm, and still the workhorse in commercial process
  simulators.*
- Holland, C. D. History of the Development of Distillation Computer Models.
  *AIChE Symp. Ser.* **1983**, *79* (235), 15.
- Wankat, P. C. Separations: A Short History and a Cloudy Crystal Ball.
  *AIChE Annual Meeting* **2008**. — *the secondary source used to check the
  chronology above.*

---

## A note on the illustration

The figure in §2.4 was **generated by an image model**, not drawn by hand and
not plotted from data. Since this is a learning repository, here is exactly how,
and where the line is.

| | |
|---|---|
| model | `gpt-image-2` (OpenAI) |
| script | [`scripts/generate_image.py`](../../../scripts/generate_image.py) |
| prompt | [`scripts/prompts/distillation-bases.md`](../../../scripts/prompts/distillation-bases.md) |
| provenance | [`docs/assets/distillation-bases.png.json`](../../assets/distillation-bases.png.json) — model, size, quality, full prompt, token usage, cost |

**Why a generated image was the right tool here, and where it would be the wrong
one.** The thing §2.4 needs to convey is not a measurement. It is a *relationship*
— better apparatus, wider apparent boiling range — and the four curves in it are
schematic, carrying no data. That is precisely the case where an illustration
beats a plot: it can show the **apparatus** alongside the curve, which is the
causal half of the story a plot cannot draw.

Every figure in this repo that carries *numbers* — the P–v–T surface, the
notebook plots, the benchmark charts — is generated by matplotlib or plotly from
the engine's own output, and must stay that way. An image model does not compute;
it draws something that looks like a computation. Those are different things, and
the boundary between them is the one rule worth taking from this note.

**What it took, honestly.** Two attempts, about \$0.29. The first render drew all
four curves as accelerating exponentials and distinguished them by *slope* — a
plausible-looking picture of the wrong physics, since a real distillation curve
is a gentle S and what actually differs between the four methods is the total
vertical **span**. The prompt had said "steeper", and the model did exactly what
it was told; the error was the prompt's. Rewriting that one paragraph to describe
the curve shape explicitly, and to say that only the span varies, fixed it. The
first attempt's failure is recorded in the provenance JSON's `note` field rather
than deleted, for the same reason the rejected optimizations in
[`OPTIMIZATION_PLAN_PART1.md`](../../plans/engine/OPTIMIZATION_PLAN_PART1.md) are
kept: the next person to write a prompt like this should be able to see the
mistake without repeating it.

**What is still imperfect.** The third panel's column is drawn with roughly a
dozen trays rather than the fifteen its caption names — image models cannot
count, and no amount of prompting reliably fixes that. The vertical spans are
also exaggerated for legibility: the real EFV-to-D2887 span ratio on this
repo's own test curve is about 1:2, not the 1:5 the drawing suggests. Both are
acceptable in a schematic and both are stated here rather than hoped over.

**To regenerate or revise it:**

```sh
~/miniconda3/envs/vle/bin/python scripts/generate_image.py \
    scripts/prompts/distillation-bases.md \
    --out docs/assets/distillation-bases.png \
    --size 2048x1024 --quality high
```

The API key comes from 1Password at call time via
[`scripts/_openai_key.py`](../../../scripts/_openai_key.py), shared with
[`scripts/second_opinion.py`](../../../scripts/second_opinion.py). Nothing is
installed into the `vle` environment — both scripts are stdlib-only.
