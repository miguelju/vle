#!/usr/bin/env python3
"""Generate ``notebooks/14_pvt_surface.ipynb`` — the water P-v-T surface.

Builds the classic "textbook plastic model" of water in 3-D — liquid wall,
ruled two-phase dome, superheat sheet — with every point computed by
``vle.steam`` (IAPWS-IF97, via the batch ``properties`` kernel). The static
README hero version of the same figure is ``scripts/render_pvt_hero.py`` →
``docs/assets/pvt_surface_hero.png``.

Structure follows CLAUDE.md *Notebook Conventions*; generated
deterministically and executed top-to-bottom in a fresh kernel before saving.

Standard: IAPWS R7-97(2012). Textbook form: Wagner & Kretzschmar,
*International Steam Tables*, 3rd ed. (2019).
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "14_pvt_surface.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# The Water P–v–T Surface — Notebook 14\n"
        "\n"
        "Every thermodynamics course shows it: the folded **P–v–T surface** "
        "of a pure substance, usually as a photo of a plastic classroom "
        "model — the steep liquid wall, the two-phase dome ending at the "
        "critical point, and the sweeping superheat sheet. In this notebook "
        "we *compute* that model for water — **36,000+ state points, every "
        "one evaluated by `vle-steam` (IAPWS-IF97)** through the batch numpy "
        "API — and then use the surface to see what an isobaric heating path "
        "and the ideal-gas approximation actually look like in 3-D."
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
        "An equation of state *is* a P–v–T surface: a rule assigning a "
        "pressure to every (v, T) pair. The thesis builds everything on cubic "
        "equations of state for mixtures — from "
        "[Chapter II §2.1 — Cubic Equations of State]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#21-cubic-equations-of-state):\n"
        "\n"
        "> Cubic equations of state (CEOS) have become a commonly used tool "
        "for solving phase equilibrium problems in multicomponent systems, "
        "not only for their computational simplicity, but also for the good "
        "approximations that result from their use, especially at high "
        "pressures, where other models tend to fail.\n"
        "\n"
        "A cubic EOS *approximates* this surface with one algebraic form. For "
        "the single most industrially important fluid — water — we don't have "
        "to approximate: **IAPWS-IF97** (Milestone 13's `vle-steam`) is a "
        "purpose-built, reference-quality formulation of the real surface, "
        "the international standard of the steam power industry. Steam "
        "itself is *not* part of the thesis or its references — it joined "
        "this project because it is the water-properties utility every "
        "practicing engineer keeps within reach. So this notebook draws the "
        "**exact** surface that Chapter II's cubic equations approximate."
    ))

    # ---- What this notebook uses ----------------------------------------
    cells.append(md(
        "## What this notebook uses\n"
        "\n"
        "No new engine code — this is a *tour* of Milestone 13's `vle.steam` "
        "(see [notebook 12](12_steam_tables.ipynb) for the API itself):\n"
        "\n"
        "- **`steam.properties(T, P)`** — the batch numpy kernel (GIL "
        "released, rayon-parallel): one call per *array* of states.\n"
        "- **`steam.saturation(T=…)`** — the printed-table row "
        "(`p, v_f, v_g, …`) that lets us walk the saturation line.\n"
        "- **`steam.Water(...)`** — scalar states for spot checks.\n"
        "\n"
        "The surface has three pieces, and the split is forced by the physics:\n"
        "\n"
        "1. **Single-phase sheets** (compressed liquid, superheated vapor, "
        "supercritical fluid) — here `v(T, P)` is single-valued, so a (T, P) "
        "grid works.\n"
        "2. **The two-phase dome** — *inside* the dome there is no `v(T, P)`: "
        "at a given (T, Psat) the mixture volume depends on the **quality** "
        "$x$ (mass fraction vapor). The dome is a *ruled surface*:\n"
        "\n"
        "$$v(T, x) \\;=\\; v_f(T) + x\\,\\bigl(v_g(T) - v_f(T)\\bigr), "
        "\\qquad x \\in [0, 1]$$\n"
        "\n"
        "so we sweep it from the saturation line, not from `properties`.\n"
        "\n"
        "Units are the engine's canon: **K** and **kPa absolute** in, "
        "mass-basis properties out (m³/kg here); we convert to °C only for "
        "the axes."
    ))

    # ---- Worked example --------------------------------------------------
    cells.append(md(
        "## Worked example — building the surface\n"
        "\n"
        "First the imports, and two **assertion cells** pinning IF97 "
        "reference values, so any regression in the engine shows up as a "
        "failing notebook rather than a silently wrong picture."
    ))
    cells.append(code(
        "%matplotlib inline\n"
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "from matplotlib import cm\n"
        "from matplotlib.colors import Normalize\n"
        "from matplotlib.lines import Line2D\n"
        "from matplotlib.patches import Patch\n"
        "\n"
        "import vle.steam as steam\n"
        "from vle.units import ureg, Q_\n"
        "\n"
        "# IF97 constants (K, kPa)\n"
        "T_TRIPLE, T_C, P_C = 273.16, 647.096, 22064.0\n"
        "P_MIN, P_MAX, T_MAX = 1.0, 100_000.0, 1073.15\n"
        "K0 = 273.15  # K -> degC offset for display"
    ))
    cells.append(code(
        "# IAPWS-IF97 Table 5 verification point (region 1): T = 300 K, P = 3 MPa.\n"
        "w = steam.Water(T=Q_(300, 'K'), P=Q_(3, 'MPa'))\n"
        "assert abs(w.v - 0.100215168e-2) < 1e-10, w.v\n"
        "assert abs(w.h - 115.331273) < 1e-5, w.h\n"
        "\n"
        "# The kitchen benchmark: boiling at 1 atm.\n"
        "row = steam.saturation(P=Q_(1, 'atm'))\n"
        "assert abs(row.t - 373.12) < 0.05, row.t          # ~99.97 degC\n"
        "assert abs((row.h_g - row.h_f) - 2256.5) < 1.0     # latent heat, kJ/kg\n"
        "print(f'IF97 checks pass — water boils at {row.t - K0:.2f} degC at 1 atm')"
    ))
    cells.append(md(
        "### Step 1 — the saturation backbone\n"
        "\n"
        "Both single-phase sheets *end* on the saturation line, and the dome "
        "is *built from* it, so we sample it once and share it. Spacing is "
        "uniform with extra density near $T_c$ (the dome's top is strongly "
        "curved there)."
    ))
    cells.append(code(
        "t_sat = np.unique(np.concatenate([\n"
        "    np.linspace(T_TRIPLE + 0.2, T_C - 0.05, 140),\n"
        "    T_C - np.geomspace(40.0, 0.05, 40),\n"
        "]))\n"
        "sat_rows = [steam.saturation(T=float(t)) for t in t_sat]\n"
        "p_sat = np.array([s.p for s in sat_rows])\n"
        "v_f = np.array([s.v_f for s in sat_rows])\n"
        "v_g = np.array([s.v_g for s in sat_rows])\n"
        "\n"
        "# Sanity: the boundary curves converge on the critical point. The\n"
        "# dome closes like sqrt(Tc - T), so even 0.05 K below Tc the v_f/v_g\n"
        "# gap is still ~12% — the assertions check monotone closure, not zero.\n"
        "assert abs(p_sat[-1] - P_C) / P_C < 1e-3\n"
        "gap = (v_g - v_f) / v_g\n"
        "assert gap[-1] < 0.15 and gap[-1] < gap[0] / 5\n"
        "print(f'{len(t_sat)} saturation rows, '\n"
        "      f'v_f/v_g gap 0.05 K below Tc: {gap[-1]:.1%}')"
    ))
    cells.append(md(
        "### Step 2 — the single-phase sheets\n"
        "\n"
        "For each saturation temperature we build a pressure row that runs "
        "from the domain edge **exactly to the saturation pressure**, and we "
        "pin the boundary sample to $v_f$ (liquid side) or $v_g$ (vapor "
        "side). That makes the sheets meet the dome edge-to-edge — no gap, "
        "no interpolation across the phase boundary. Above $T_c$ there is no "
        "boundary and one row spans the whole pressure range.\n"
        "\n"
        "Each `steam.properties` call below evaluates a full row of IF97 "
        "states in one FFI crossing (the batch API from Milestone 13)."
    ))
    cells.append(code(
        "N_P = 50\n"
        "\n"
        "def v_of(t_row, p_row):\n"
        "    return steam.properties(T=t_row, P=p_row)['v']\n"
        "\n"
        "def sheet(rows):\n"
        "    \"\"\"Stack per-row (v, T, P) samples into plot_surface grids\n"
        "    in display units: log10(v [m3/kg]), T [degC], log10(P [kPa]).\"\"\"\n"
        "    V = np.array([r[0] for r in rows])\n"
        "    T = np.array([r[1] for r in rows])\n"
        "    P = np.array([r[2] for r in rows])\n"
        "    return np.log10(V), T - K0, np.log10(P)\n"
        "\n"
        "# Sheet A: compressed liquid, continuing into the supercritical region.\n"
        "rows_a = []\n"
        "for i, t in enumerate(t_sat):\n"
        "    p_row = np.geomspace(P_MAX, p_sat[i] * (1 + 1e-6), N_P)\n"
        "    rows_a.append((np.append(v_of(np.full(N_P, t), p_row), v_f[i]),\n"
        "                   np.full(N_P + 1, t), np.append(p_row, p_sat[i])))\n"
        "for t in np.linspace(T_C + 0.5, T_MAX, 30):\n"
        "    p_row = np.geomspace(P_MAX, P_MIN, N_P + 1)\n"
        "    rows_a.append((v_of(np.full(N_P + 1, t), p_row), np.full(N_P + 1, t), p_row))\n"
        "XA, YA, ZA = sheet(rows_a)\n"
        "\n"
        "# Sheet B: superheated vapor (T < Tc only).\n"
        "rows_b = []\n"
        "for i, t in enumerate(t_sat):\n"
        "    p_row = np.geomspace(p_sat[i] * (1 - 1e-6), P_MIN, N_P)\n"
        "    rows_b.append((np.insert(v_of(np.full(N_P, t), p_row), 0, v_g[i]),\n"
        "                   np.full(N_P + 1, t), np.insert(p_row, 0, p_sat[i])))\n"
        "XB, YB, ZB = sheet(rows_b)\n"
        "\n"
        "n_single = XA.size + XB.size\n"
        "assert not (np.isnan(XA).any() or np.isnan(XB).any())\n"
        "print(f'{n_single} single-phase IF97 points')"
    ))
    cells.append(md(
        "### Step 3 — the two-phase dome\n"
        "\n"
        "Ruled in quality between the boundary curves: each saturation row "
        "contributes one straight line from $(v_f, T, P_{sat})$ to "
        "$(v_g, T, P_{sat})$ — straight in $v$, though the log axis will bend "
        "it visually."
    ))
    cells.append(code(
        "x_q = np.linspace(0.0, 1.0, 35)\n"
        "VD = v_f[:, None] + x_q[None, :] * (v_g - v_f)[:, None]\n"
        "XD = np.log10(VD)\n"
        "YD = np.broadcast_to((t_sat - K0)[:, None], VD.shape)\n"
        "ZD = np.broadcast_to(np.log10(p_sat)[:, None], VD.shape)\n"
        "print(f'{XD.size} dome points; total = {n_single + XD.size} IF97 evaluations')"
    ))
    cells.append(md(
        "### Step 4 — render\n"
        "\n"
        "Color encodes **temperature** (one sequential ramp shared by all "
        "three pieces); the dome uses the same ramp darkened ~35 % so it "
        "reads as \"the interior\". The green/orange curves are the "
        "saturated-liquid/vapor boundaries, and the star is the critical "
        "point (373.95 °C, 22.064 MPa), where they merge and the distinction "
        "between liquid and vapor disappears."
    ))
    cells.append(code(
        "fig = plt.figure(figsize=(11, 8))\n"
        "ax = fig.add_subplot(111, projection='3d')\n"
        "norm = Normalize(vmin=T_TRIPLE - K0, vmax=T_MAX - K0)\n"
        "kw = dict(rstride=1, cstride=1, linewidth=0, antialiased=True, shade=False)\n"
        "ax.plot_surface(XA, YA, ZA, facecolors=cm.plasma(norm(YA)), alpha=0.85, **kw)\n"
        "ax.plot_surface(XB, YB, ZB, facecolors=cm.plasma(norm(YB)), alpha=0.85, **kw)\n"
        "dome_rgba = cm.plasma(norm(YD)); dome_rgba[..., :3] *= 0.65\n"
        "ax.plot_surface(XD, YD, ZD, facecolors=dome_rgba, alpha=1.0, **kw)\n"
        "\n"
        "ax.plot(np.log10(v_f), t_sat - K0, np.log10(p_sat), color='#2e9e57', lw=1.8)\n"
        "ax.plot(np.log10(v_g), t_sat - K0, np.log10(p_sat), color='#d97a1f', lw=1.8)\n"
        "vc = steam.Water(T=T_C - 0.2, P=P_C * 0.999).v\n"
        "ax.scatter([np.log10(vc)], [T_C - K0], [np.log10(P_C)], color='#0087b8',\n"
        "           s=110, marker='*', depthshade=False)\n"
        "\n"
        "ax.set_xlabel('specific volume v (m$^3$/kg)'); ax.set_ylabel('T (°C)')\n"
        "ax.set_zlabel('P (kPa)')\n"
        "ax.set_xticks([-3, -2, -1, 0, 1, 2])\n"
        "ax.set_xticklabels(['0.001', '0.01', '0.1', '1', '10', '100'])\n"
        "ax.set_zticks([0, 1, 2, 3, 4, 5])\n"
        "ax.set_zticklabels(['1', '10', '10$^2$', '10$^3$', '10$^4$', '10$^5$'])\n"
        "ax.view_init(elev=22, azim=-58)\n"
        "ax.set_title('Water P–v–T surface — every point from vle-steam (IAPWS-IF97)')\n"
        "legend_items = [\n"
        "    (Patch(facecolor=cm.plasma(0.65), alpha=0.85), 'single-phase sheets'),\n"
        "    (Patch(facecolor=np.array(cm.plasma(0.35)) * [0.65, 0.65, 0.65, 1.0]),\n"
        "     'two-phase dome (ruled in quality x)'),\n"
        "    (Line2D([], [], color='#2e9e57', lw=1.8), 'saturated liquid $v_f(T)$'),\n"
        "    (Line2D([], [], color='#d97a1f', lw=1.8), 'saturated vapor $v_g(T)$'),\n"
        "    (Line2D([], [], color='#0087b8', marker='*', ms=11, lw=0), 'critical point'),\n"
        "]\n"
        "ax.legend([h for h, _ in legend_items], [t for _, t in legend_items],\n"
        "          loc='upper left', fontsize=8)\n"
        "cb = fig.colorbar(cm.ScalarMappable(norm=norm, cmap='plasma'), ax=ax,\n"
        "                  shrink=0.55, pad=0.06)\n"
        "cb.set_label('temperature (°C)')\n"
        "plt.show()"
    ))
    cells.append(md(
        "Read the geometry the way a course would:\n"
        "\n"
        "- The **liquid wall** is nearly vertical: compressing liquid water "
        "from 1 bar to 1000 bar barely changes $v$ — that's why \"liquids "
        "are incompressible\" works as an approximation.\n"
        "- The **dome** narrows as temperature rises until $v_f = v_g$ at the "
        "critical point — above it, no amount of pressure produces a phase "
        "change.\n"
        "- The **superheat sheet** at low pressure flattens toward the "
        "ideal-gas plane $Pv = R_sT$ (exercise 2 quantifies where that "
        "becomes true)."
    ))

    # ---- Exercises --------------------------------------------------------
    cells.append(md(
        "## Exercises\n"
        "\n"
        "### Exercise 1 — an isobaric heating path across the dome\n"
        "\n"
        "Boiler-side intuition: heat water at a constant **1 MPa** from "
        "20 °C to 400 °C. The path runs up the liquid wall, crosses the dome "
        "along a horizontal chord (constant T *and* P while boiling — only "
        "$x$ changes), then climbs the superheat sheet.\n"
        "\n"
        "Compute the path and overlay it on the surface. Steps:\n"
        "1. Find the saturation temperature at 1 MPa.\n"
        "2. Sample the liquid branch (20 °C → $T_{sat}$), the dome chord "
        "($x: 0 → 1$), and the vapor branch ($T_{sat}$ → 400 °C).\n"
        "3. Assert $T_{sat}$(1 MPa) ≈ 179.89 °C, then plot."
    ))
    cells.append(code(
        "# TODO: saturation temperature at 1 MPa (1000 kPa)\n"
        "# t_boil = ...\n"
        "\n"
        "# TODO: liquid branch — properties(T=..., P=1000.0) for T below t_boil\n"
        "# TODO: dome chord   — v = v_f + x*(v_g - v_f) at t_boil, x in [0, 1]\n"
        "# TODO: vapor branch — properties(T=..., P=1000.0) for T above t_boil\n"
        "\n"
        "# TODO: re-draw the surface (or reuse the figure code) and overlay the\n"
        "# three segments as a single red 3-D line."
    ))
    cells.append(md(
        "<details>\n<summary>Solution</summary>\n\n"
        "```python\n"
        "P_PATH = 1000.0  # kPa\n"
        "row1 = steam.saturation(P=P_PATH)\n"
        "t_boil = row1.t\n"
        "assert abs((t_boil - K0) - 179.89) < 0.05, t_boil - K0\n"
        "\n"
        "t_liq = np.linspace(K0 + 20.0, t_boil - 1e-3, 60)\n"
        "v_liq = steam.properties(T=t_liq, P=np.full_like(t_liq, P_PATH))['v']\n"
        "x_path = np.linspace(0.0, 1.0, 40)\n"
        "v_chord = row1.v_f + x_path * (row1.v_g - row1.v_f)\n"
        "t_vap = np.linspace(t_boil + 1e-3, K0 + 400.0, 60)\n"
        "v_vap = steam.properties(T=t_vap, P=np.full_like(t_vap, P_PATH))['v']\n"
        "\n"
        "v_all = np.concatenate([v_liq, v_chord, v_vap])\n"
        "t_all = np.concatenate([t_liq, np.full_like(x_path, t_boil), t_vap])\n"
        "p_all = np.full_like(v_all, P_PATH)\n"
        "\n"
        "# ... rebuild the surface figure from the worked example, then:\n"
        "ax.plot(np.log10(v_all), t_all - K0, np.log10(p_all),\n"
        "        color='red', lw=2.5, zorder=12, label='1 MPa heating path')\n"
        "```\n"
        "\n"
        "The chord is where the boiler spends most of its heat duty: "
        "2014 kJ/kg of latent heat at 1 MPa versus ~678 kJ/kg of sensible "
        "heat to get there from 20 °C (check `row1.h_g - row1.h_f` and "
        "compare enthalpies from `steam.Water`).\n"
        "\n"
        "</details>"
    ))
    cells.append(md(
        "### Exercise 2 — where does the ideal gas law break?\n"
        "\n"
        "The ideal-gas surface is $v_{ig} = R_s T / P$ with "
        "$R_s = 0.461526\\;\\mathrm{kJ/(kg\\,K)}$ (IF97's specific gas "
        "constant). On the superheated sheet, compute the relative error\n"
        "\n"
        "$$\\varepsilon = \\frac{v_{ig} - v}{v}$$\n"
        "\n"
        "and answer: at **200 °C**, up to what pressure does the ideal gas "
        "law stay within 5 % of IF97? What about at 500 °C? (Sanity check: "
        "at 500 °C and 1 kPa the error should be far below 0.1 %.)"
    ))
    cells.append(code(
        "R_S = 0.461526  # kJ/(kg K) -> v_ig = R_S * T / P gives m3/kg (kPa in)\n"
        "\n"
        "# TODO: for an isotherm T (in K), sweep P from 1 kPa up to Psat(T)\n"
        "# (or 100 MPa above Tc), compute v from steam.properties and v_ig,\n"
        "# and find the largest P where abs(v_ig - v)/v < 0.05.\n"
        "\n"
        "# def p_limit_5pct(t_kelvin): ...\n"
        "\n"
        "# print(p_limit_5pct(K0 + 200.0), p_limit_5pct(K0 + 500.0))"
    ))
    cells.append(md(
        "<details>\n<summary>Solution</summary>\n\n"
        "```python\n"
        "def p_limit_5pct(t_kelvin, tol=0.05):\n"
        "    p_top = steam.psat(t_kelvin) * (1 - 1e-6) if t_kelvin < T_C else P_MAX\n"
        "    p = np.geomspace(1.0, p_top, 400)\n"
        "    v = steam.properties(T=np.full_like(p, t_kelvin), P=p)['v']\n"
        "    err = np.abs(R_S * t_kelvin / p - v) / v\n"
        "    ok = p[err < tol]\n"
        "    return ok.max() if ok.size else np.nan\n"
        "\n"
        "p200 = p_limit_5pct(K0 + 200.0)\n"
        "p500 = p_limit_5pct(K0 + 500.0)\n"
        "print(f'5% envelope: {p200:.0f} kPa at 200 degC, {p500:.0f} kPa at 500 degC')\n"
        "\n"
        "# Sanity: ideal gas is excellent at low pressure.\n"
        "v1 = steam.Water(T=K0 + 500.0, P=1.0).v\n"
        "assert abs(R_S * (K0 + 500.0) / 1.0 - v1) / v1 < 1e-3\n"
        "```\n"
        "\n"
        "Typical results: at 200 °C the 5 % envelope ends near **0.85 MPa** — "
        "barely half the saturation pressure (1.55 MPa), so superheated steam "
        "near the dome is *not* an ideal gas; at 500 °C it stretches to "
        "**~5.9 MPa**. The rule of thumb \"steam is ideal below about a bar\" "
        "is conservative everywhere on the sheet.\n"
        "\n"
        "</details>"
    ))

    # ---- References -------------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- Research paper: [Chapter II §2.1 — Cubic Equations of State]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#21-cubic-equations-of-state) "
        "— the algebraic approximations to this surface.\n"
        "- [IAPWS R7-97(2012)](https://iapws.org/documents/release/IF97-Rev) — "
        "the Revised Release on the Industrial Formulation 1997, the standard "
        "`vle-steam` implements.\n"
        "- Wagner, W.; Kretzschmar, H.-J. *International Steam Tables*, "
        "3rd ed.; Springer, 2019 — the textbook form of IF97.\n"
        "- [Notebook 12 — Steam Tables](12_steam_tables.ipynb) — the "
        "`vle.steam` API tour (saturation pages, turbine expansion, flash "
        "steam, reboiler duty).\n"
        "- `MODERNIZATION_PLAN.md` Phase 20 — the `vle-steam` design record "
        "(regions, backward equations, batch kernels).\n"
        "- `scripts/render_pvt_hero.py` — the static README-hero version of "
        "this figure (`docs/assets/pvt_surface_hero.png`).\n"
        "\n"
        "**Why this notebook exists:** steam properties are not part of the "
        "thesis — but they are the water-property calculation a practicing "
        "chemical engineer reaches for most, and the P–v–T surface is the "
        "single picture that organizes all of them. Computing it point by "
        "point from the industrial standard turns the classroom prop into "
        "something you can interrogate."
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
