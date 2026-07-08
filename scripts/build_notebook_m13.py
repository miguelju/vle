#!/usr/bin/env python3
"""Generate ``notebooks/12_steam_tables.ipynb`` — the Milestone 13 notebook
(industrial steam tables, IAPWS-IF97, vle-thermo 0.10.0).

"VLE for water only": the ``vle.steam`` surface over the dependency-free
``vle-steam`` crate implementing the IAPWS Industrial Formulation 1997. Works
the everyday practitioner scenarios — a saturation-table page, an isentropic
steam-turbine expansion, flash-steam recovery, and reboiler duty — with the
unit-aware ``Water(...)`` constructor (pint quantities + gauge pressure).

Structure follows CLAUDE.md *Notebook Conventions*; generated deterministically
and executed top-to-bottom in a fresh kernel before saving.

Standard: IAPWS R7-97(2012). Textbook form: Wagner & Kretzschmar,
*International Steam Tables*, 3rd ed. (2019).
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "12_steam_tables.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Steam Tables — Milestone 13\n"
        "\n"
        "Steam tables are the single most-used thermodynamic reference in "
        "chemical-engineering practice: sizing reboilers and condensers, "
        "balancing steam headers, recovering flash steam, and rating turbines "
        "and valves. Every printed steam table in a modern handbook is "
        "*computed from* one open standard — the **IAPWS Industrial Formulation "
        "1997 (IF97)** — so `vle-steam` implements that standard directly rather "
        "than interpolating tabulated numbers.\n"
        "\n"
        "This is, in effect, **VLE for water only**: a companion to the "
        "multicomponent `vle-thermo` engine, surfaced in Python as "
        "`vle.steam`. In this notebook we reproduce a printed saturation-table "
        "page, then work four scenarios an engineer meets weekly."
    ))

    # ---- Setup (optional) -----------------------------------------------
    cells.append(md(
        "## Setup (optional)\n"
        "\n"
        "The cell below is **commented out** — the notebook runs top-to-bottom "
        "against whatever `vle-thermo` is already in your kernel. Uncomment it "
        "to pull the newest released wheel from PyPI."
    ))
    cells.append(code(
        "# Optional: pull the latest vle-thermo from PyPI.\n"
        "# Uncomment if you want the newest released version instead of\n"
        "# whatever is currently in your kernel.\n"
        "# %pip install --upgrade vle-thermo"
    ))

    # ---- Context from the research paper --------------------------------
    cells.append(md(
        "## Context from the research paper\n"
        "\n"
        "The thesis this project modernizes is about the vapor–liquid "
        "equilibrium of *mixtures* via cubic equations of state; steam tables "
        "are the **pure-water reference standard** for exactly the energy "
        "properties it derives. From "
        "[Chapter II §2.1.3 — Energy Properties based on CEOS]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#213-energy-properties-based-on-ceos):\n"
        "\n"
        "> The calculation of energy properties (enthalpy and entropy) is "
        "essential for the energy balances of separation processes; these are "
        "obtained from the equation of state through the departure functions "
        "referenced to an ideal-gas state.\n"
        "\n"
        "For water specifically, IF97 replaces a cubic EOS with a purpose-built, "
        "reference-quality formulation. The same *(P, h)* and *(P, s)* flashes "
        "that Chapter II §2.3.2 describes for mixtures appear here as the "
        "throttling-valve and turbine calculations below — only now the numbers "
        "match a handbook to nine significant figures."
    ))

    # ---- What was built -------------------------------------------------
    cells.append(md(
        "## What Milestone 13 built\n"
        "\n"
        "A new dependency-free workspace crate **`vle-steam`** implementing all "
        "five IF97 regions plus the saturation line and backward equations, "
        "surfaced as **`vle.steam`**:\n"
        "\n"
        "| Region | Domain | Equation |\n"
        "|---|---|---|\n"
        "| 1 | compressed liquid, 273–623 K | Gibbs $g(p,T)$ |\n"
        "| 2 | superheated vapor, ≤ 1073 K | Gibbs $g(p,T)$ |\n"
        "| 3 | near-critical | Helmholtz $f(\\rho,T)$ (density iteration) |\n"
        "| 4 | **saturation line**, 273–647 K | closed-form both ways |\n"
        "| 5 | high-T steam, ≤ 2273 K | Gibbs $g(p,T)$ |\n"
        "\n"
        "The Python surface:\n"
        "\n"
        "- **`steam.Water(...)`** — a state from any of the pairs `(T,P)`, "
        "`(T,x)`, `(P,x)`, `(P,h)`, `(P,s)`; returns `T, P, region, phase, x, "
        "ρ, v, u, h, s, cp, cv, w`.\n"
        "- **`steam.saturation(T=…)` / `(P=…)`** — the printed saturation-table "
        "row (`v_f, v_g, h_f, h_g, h_fg, s_f, s_g, s_fg, u_f, u_g`).\n"
        "- **`steam.psat` / `tsat` / `latent_heat` / `psat_derivative`** — "
        "scalar helpers.\n"
        "- **`steam.properties` / `ph_flash` / `sat_table`** — batch numpy "
        "kernels (the GIL-released, rayon-parallel \"numpy for thermo\" path).\n"
        "\n"
        "All properties are **mass-basis** (kJ/kg, kJ/(kg·K), m³/kg). Inputs "
        "accept plain floats (`T` in K, `P` in kPa absolute), `pint` "
        "quantities, or unit strings like `\"180 degC\"` and `\"10 barg\"` — "
        "gauge pressure is resolved through the same registry the engine uses."
    ))

    cells.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "%matplotlib inline\n"
        "\n"
        "from vle import steam\n"
        "\n"
        "# A single (T, P) state — compressed liquid water at 300 K, 3 MPa.\n"
        "# This is R7-97(2012) Table 5 verification point 1.\n"
        "st = steam.Water(T=300.0, P=3000.0)  # K, kPa absolute\n"
        "print(st)\n"
        "print(f\"region {st.region}  phase {st.phase}\")\n"
        "print(f\"h = {st.h:.6f} kJ/kg   (IAPWS: 115.331273)\")\n"
        "assert abs(st.h - 115.331273) < 1e-5"
    ))

    # ---- Saturation-table page ------------------------------------------
    cells.append(md(
        "## A printed saturation-table page\n"
        "\n"
        "The classic steam-table page: pick a column of temperatures and read "
        "off saturation pressure, the saturated-liquid and -vapor volumes, "
        "enthalpies, and entropies. `steam.sat_table` returns these as numpy "
        "columns in one GIL-released call."
    ))
    cells.append(code(
        "T_C = np.array([50, 100, 150, 200, 250, 300, 350], dtype=float)  # °C\n"
        "tab = steam.sat_table(T_C + 273.15)\n"
        "\n"
        "print(f\"{'T[°C]':>6} {'Psat[kPa]':>11} {'v_f[m³/kg]':>12} \"\n"
        "      f\"{'v_g[m³/kg]':>12} {'h_f[kJ/kg]':>11} {'h_g[kJ/kg]':>11} \"\n"
        "      f\"{'h_fg[kJ/kg]':>12}\")\n"
        "for i, t in enumerate(T_C):\n"
        "    print(f\"{t:6.0f} {tab['p'][i]:11.3f} {tab['v_f'][i]:12.6f} \"\n"
        "          f\"{tab['v_g'][i]:12.5f} {tab['h_f'][i]:11.2f} \"\n"
        "          f\"{tab['h_g'][i]:11.2f} {tab['h_fg'][i]:12.2f}\")\n"
        "\n"
        "# Spot-check the 100 °C row against the handbook (1 atm boiling).\n"
        "assert abs(tab['p'][1] - 101.42) < 0.1        # Psat(100°C) ≈ 101.4 kPa\n"
        "assert abs(tab['h_fg'][1] - 2256.5) < 1.0     # latent heat ≈ 2257 kJ/kg"
    ))

    cells.append(md(
        "### The saturation dome\n"
        "\n"
        "Plotting the saturated-liquid and -vapor entropies against temperature "
        "traces the two-phase **dome** on a *T–s* diagram — the map every "
        "turbine and refrigeration cycle is drawn on. The apex is the critical "
        "point (647.096 K, 22.064 MPa)."
    ))
    cells.append(code(
        "Tdome = np.linspace(274.0, 646.0, 280)\n"
        "dome = steam.sat_table(Tdome)\n"
        "\n"
        "# Critical point: entropy → the common value of s_f, s_g as T → Tc.\n"
        "s_near_crit = steam.saturation(T=646.5)\n"
        "s_crit = 0.5 * (s_near_crit.s_f + s_near_crit.s_g)\n"
        "\n"
        "fig, ax = plt.subplots(figsize=(7, 5))\n"
        "ax.plot(dome['s_f'], Tdome, color='#1f77b4', label='saturated liquid')\n"
        "ax.plot(dome['s_g'], Tdome, color='#d62728', label='saturated vapor')\n"
        "ax.fill_betweenx(Tdome, dome['s_f'], dome['s_g'],\n"
        "                 color='#1f77b4', alpha=0.08)\n"
        "ax.scatter([s_crit], [647.096],\n"
        "           color='k', zorder=5, label='critical point')\n"
        "ax.set_xlabel('specific entropy  s  [kJ/(kg·K)]')\n"
        "ax.set_ylabel('temperature  T  [K]')\n"
        "ax.set_title('Water saturation dome (T–s), IAPWS-IF97')\n"
        "ax.legend()\n"
        "ax.grid(alpha=0.3)\n"
        "plt.show()"
    ))

    # ---- Worked example: turbine ----------------------------------------
    cells.append(md(
        "## Worked example — an isentropic steam turbine\n"
        "\n"
        "Steam enters a turbine at **40 bar, 400 °C** and expands to the "
        "condenser at **0.1 bar**. The turbine is 80 % isentropically "
        "efficient. Find the **outlet quality** and the **specific work**.\n"
        "\n"
        "The recipe is the textbook one, and it uses three `Water` modes:\n"
        "\n"
        "1. **Inlet state** from `(T, P)` → gives $h_1$ and $s_1$.\n"
        "2. **Ideal (isentropic) outlet** from `(P_\\text{out}, s_1)` → the "
        "reversible enthalpy $h_{2s}$.\n"
        "3. **Actual outlet** from the efficiency: "
        "$h_2 = h_1 - \\eta\\,(h_1 - h_{2s})$, then `(P_\\text{out}, h_2)` for "
        "the real quality.\n"
        "\n"
        "> Note the gauge-vs-absolute care: `\"40 bar\"` here is absolute. Use "
        "`\"40 barg\"` for gauge — the registry adds the atmospheric offset "
        "automatically."
    ))
    cells.append(code(
        "eta = 0.80\n"
        "inlet = steam.Water(T='400 degC', P='40 bar')      # absolute\n"
        "print('inlet :', inlet)\n"
        "\n"
        "# Ideal expansion is isentropic: same entropy, condenser pressure.\n"
        "ideal = steam.Water(P='0.1 bar', s=inlet.s)\n"
        "h2s = ideal.h\n"
        "\n"
        "# Real machine: 80% of the ideal enthalpy drop is realized.\n"
        "h2 = inlet.h - eta * (inlet.h - h2s)\n"
        "outlet = steam.Water(P='0.1 bar', h=h2)\n"
        "\n"
        "work = inlet.h - outlet.h            # kJ/kg\n"
        "print('ideal outlet :', ideal, f'  x_ideal={ideal.x:.4f}')\n"
        "print('real  outlet :', outlet, f'  x_real ={outlet.x:.4f}')\n"
        "print(f'\\nspecific work  w = {work:.1f} kJ/kg')\n"
        "print(f'exhaust is {outlet.phase} at x = {outlet.x:.3f} '\n"
        "      f'(T = {outlet.t-273.15:.2f} °C)')\n"
        "\n"
        "# Sanity: real exhaust is wetter than ideal, both inside the dome.\n"
        "assert 0.0 < ideal.x < outlet.x < 1.0\n"
        "assert work > 0"
    ))

    cells.append(md(
        "The 80 %-efficient machine leaves the steam **wetter** (lower quality) "
        "than the reversible one — irreversibility shows up as extra entropy, "
        "pushing the exhaust point rightward under the dome. Excessive wetness "
        "erodes turbine blades, which is why the exhaust quality is a design "
        "constraint. Let's draw both expansion paths on the *T–s* dome."
    ))
    cells.append(code(
        "fig, ax = plt.subplots(figsize=(7, 5))\n"
        "ax.plot(dome['s_f'], Tdome, color='#1f77b4')\n"
        "ax.plot(dome['s_g'], Tdome, color='#d62728')\n"
        "ax.fill_betweenx(Tdome, dome['s_f'], dome['s_g'], color='#1f77b4', alpha=0.08)\n"
        "\n"
        "# Expansion paths (vertical = isentropic ideal; slanted = real).\n"
        "ax.plot([inlet.s, ideal.s], [inlet.t, ideal.t], 'o--', color='#2ca02c',\n"
        "        label='ideal (isentropic)')\n"
        "ax.plot([inlet.s, outlet.s], [inlet.t, outlet.t], 'o-', color='#ff7f0e',\n"
        "        label=f'real (η={eta:.0%})')\n"
        "ax.annotate('inlet\\n40 bar, 400°C', (inlet.s, inlet.t),\n"
        "            textcoords='offset points', xytext=(8, -4))\n"
        "ax.set_xlabel('specific entropy  s  [kJ/(kg·K)]')\n"
        "ax.set_ylabel('temperature  T  [K]')\n"
        "ax.set_title('Steam-turbine expansion on the T–s dome')\n"
        "ax.legend(loc='lower left')\n"
        "ax.grid(alpha=0.3)\n"
        "plt.show()"
    ))

    # ---- Exercises ------------------------------------------------------
    cells.append(md(
        "## Exercises\n"
        "\n"
        "Two scenarios from daily practice. Try each in the template cell, then "
        "expand the solution to check."
    ))

    # Exercise 1: flash-steam recovery
    cells.append(md(
        "### Exercise 1 — Flash-steam recovery\n"
        "\n"
        "Saturated condensate leaves a steam trap at **10 barg** and is "
        "throttled to a flash vessel at **1 barg**. Throttling is isenthalpic "
        "(a valve does no work and — ideally — loses no heat). **What mass "
        "fraction of the condensate flashes to steam?**\n"
        "\n"
        "Hint: the condensate starts as saturated liquid ($x=0$) at 10 barg; "
        "its enthalpy is conserved through the valve; the downstream state is a "
        "`(P, h)` flash at 1 barg."
    ))
    cells.append(code(
        "# TODO:\n"
        "# 1. inlet = saturated liquid at 10 barg   -> steam.Water(P='10 barg', x=0.0)\n"
        "# 2. flash to 1 barg at constant enthalpy  -> steam.Water(P='1 barg', h=inlet.h)\n"
        "# 3. the flashed fraction is the downstream quality x\n"
        "flashed_fraction = ...\n"
        "# print(f'{flashed_fraction:.1%} of the condensate flashes to steam')"
    ))
    cells.append(md(
        "<details>\n<summary>Solution</summary>\n\n"
        "```python\n"
        "inlet = steam.Water(P='10 barg', x=0.0)      # saturated condensate\n"
        "flash = steam.Water(P='1 barg', h=inlet.h)   # isenthalpic throttle\n"
        "flashed_fraction = flash.x\n"
        "print(f'{flashed_fraction:.1%} of the condensate flashes to steam')\n"
        "print(f'downstream: {flash.phase} at {flash.t-273.15:.1f} °C')\n"
        "```\n\n"
        "About **13–14 %** of the condensate flashes — that flash steam is "
        "worth recovering to a low-pressure header instead of venting it. The "
        "driving force is the drop in saturation temperature: the liquid "
        "arrives hotter than its new boiling point, and the excess sensible "
        "heat boils part of it off.\n"
        "</details>"
    ))
    cells.append(code(
        "# Reference solution (executed so the notebook self-checks).\n"
        "inlet = steam.Water(P='10 barg', x=0.0)\n"
        "flash = steam.Water(P='1 barg', h=inlet.h)\n"
        "print(f'flashed fraction = {flash.x:.1%}')\n"
        "assert 0.10 < flash.x < 0.16\n"
        "assert flash.phase == 'two-phase'"
    ))

    # Exercise 2: reboiler duty
    cells.append(md(
        "### Exercise 2 — Reboiler steam demand\n"
        "\n"
        "A distillation reboiler needs **Q = 500 kW** of heat, supplied by "
        "condensing **4 barg** saturated steam (it enters as saturated vapor "
        "and leaves as saturated liquid, giving up its latent heat). **What "
        "steam flow rate, in kg/h, does the reboiler consume?**\n"
        "\n"
        "Hint: each kg of condensing steam releases $h_{fg}$ at 4 barg; the "
        "mass flow is $\\dot m = Q / h_{fg}$."
    ))
    cells.append(code(
        "Q_kW = 500.0\n"
        "# TODO:\n"
        "# 1. latent heat at 4 barg (kJ/kg) -> steam.latent_heat(...) or saturation(...).h_fg\n"
        "# 2. mass flow (kg/s) = Q / h_fg,  then × 3600 for kg/h\n"
        "m_dot_kg_per_h = ...\n"
        "# print(f'{m_dot_kg_per_h:.0f} kg/h of 4 barg steam')"
    ))
    cells.append(md(
        "<details>\n<summary>Solution</summary>\n\n"
        "```python\n"
        "sat = steam.saturation(P='4 barg')     # 4 barg ≈ 506 kPa absolute\n"
        "h_fg = sat.h_fg                        # kJ/kg\n"
        "m_dot = Q_kW / h_fg                    # kJ/s ÷ kJ/kg = kg/s\n"
        "m_dot_kg_per_h = m_dot * 3600.0\n"
        "print(f'Tsat = {sat.t-273.15:.1f} °C, h_fg = {h_fg:.0f} kJ/kg')\n"
        "print(f'steam demand = {m_dot_kg_per_h:.0f} kg/h')\n"
        "```\n\n"
        "At 4 barg the latent heat is ≈ 2100 kJ/kg, so 500 kW needs roughly "
        "**850 kg/h** of steam. Higher-pressure steam carries *less* latent heat "
        "per kg (the dome narrows), so a hotter reboiler that runs on high-"
        "pressure steam actually needs a slightly larger mass flow for the same "
        "duty — a small but real design consideration.\n"
        "</details>"
    ))
    cells.append(code(
        "# Reference solution (executed so the notebook self-checks).\n"
        "sat = steam.saturation(P='4 barg')\n"
        "m_dot_kg_per_h = Q_kW / sat.h_fg * 3600.0\n"
        "print(f'steam demand = {m_dot_kg_per_h:.0f} kg/h  (h_fg={sat.h_fg:.0f} kJ/kg)')\n"
        "assert 800 < m_dot_kg_per_h < 900"
    ))

    # ---- References -----------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- IAPWS. *Revised Release on the IAPWS Industrial Formulation 1997 for "
        "the Thermodynamic Properties of Water and Steam*; IAPWS R7-97(2012). "
        "The equations and the verification tables `vle-steam` is tested "
        "against.\n"
        "- Wagner, W.; Kretzschmar, H.-J. *International Steam Tables*, 3rd ed.; "
        "Springer, 2019. The textbook form of IF97.\n"
        "- Research paper: "
        "[Chapter II §2.1.3 — Energy Properties]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#213-energy-properties-based-on-ceos) "
        "and [§2.3.2 — Flash Calculations]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#232-flash-calculations) "
        "(the mixture analog of the `(P,h)`/`(P,s)` flashes used here).\n"
        "- Design record: `STEAM_TABLES_PLAN.md` (Milestone 13); "
        "`MODERNIZATION_PLAN.md` Phase 20.\n"
        "\n"
        "**Why this milestone exists:** water is the working fluid of most of "
        "the process industry, and its properties are a solved problem with an "
        "open reference standard. Shipping IF97 directly — dependency-free and "
        "tested to nine significant figures — gives every downstream energy "
        "balance a trustworthy water model, and gives the planned iOS build its "
        "first natural app: a steam-table calculator in your pocket."
    ))

    nb.cells = cells
    nb.metadata = {
        "kernelspec": {"display_name": "Python 3", "language": "python", "name": "python3"},
        "language_info": {"name": "python"},
    }
    return nb


def main() -> None:
    nb = build()
    client = NotebookClient(nb, timeout=600, kernel_name="python3")
    client.execute()
    NB_PATH.parent.mkdir(parents=True, exist_ok=True)
    with NB_PATH.open("w", encoding="utf-8") as fh:
        nbf.write(nb, fh)
    print(f"wrote {NB_PATH} ({len(nb.cells)} cells)")


if __name__ == "__main__":
    main()
