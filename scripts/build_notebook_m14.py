#!/usr/bin/env python3
"""Generate ``notebooks/13_nrtl_ammonia.ipynb`` — the Milestone 14 notebook
(NRTL activity model + ammonia, vle-thermo 0.11.0).

NRTL (Renon & Prausnitz, 1968) is the standard local-composition model for
aqueous-associating and polar mixtures. This notebook works the ammonia–water
binary: activity coefficients, the (large, negative) heat of mixing that only a
temperature-dependent model captures, and a bubble-pressure curve — the
liquid-phase piece a downstream staged-separation (distillation) consumer needs for the
ammonia–water Ponchon–Savarit (enthalpy–composition) method.

Also contains the NRTL_AMMONIA_PLAN.md §8 follow-up (added 2026-07-12): the
"same data, two models" van Laar-vs-NRTL comparison on the thesis's own
methanol–water Table 4.6 data, with an LM refit exercise on the α
non-randomness. The Table 4.6 literals are copied verbatim from
``build_notebook_m9_bubble_dew.py`` (notebook 04) — single source of truth.

Structure follows CLAUDE.md *Notebook Conventions*; generated deterministically
and executed top-to-bottom in a fresh kernel before saving.

Model: NRTL, Renon, H.; Prausnitz, J. M. *AIChE J.* 1968, 14, 135. The
non-randomness α = 0.2 is the value recommended for ammonia mixtures (Junqua
et al., "Reconsideration of the alpha coefficient in NRTL…", *Int. J. Refrig.*
2019, and Zhang et al., I&EC Res. 2017, 56, 12525). The binary interaction
energies here are **illustrative** (chosen to reproduce the qualitative negative
deviation / exothermic mixing of NH₃–H₂O); a production correlation regresses
them against experimental VLE data — see NRTL_AMMONIA_PLAN.md §5.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "13_nrtl_ammonia.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# NRTL Activity Model + Ammonia — Milestone 14\n"
        "\n"
        "The **NRTL** (Non-Random Two-Liquid; Renon & Prausnitz, 1968) model is "
        "the workhorse for the strongly non-ideal, hydrogen-bonding mixtures that "
        "cubic equations of state describe poorly — alcohol–water, acetone–water, "
        "and the **ammonia–water** pair at the heart of absorption refrigeration. "
        "This notebook adds NRTL and ammonia to `vle-thermo` and uses them to "
        "compute the activity coefficients, the **heat of mixing**, and a "
        "bubble-pressure curve of NH₃–H₂O — the liquid-phase model a downstream "
        "staged-separation (distillation) consumer needs for the ammonia–water "
        "enthalpy–composition (Ponchon–Savarit) method."
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
        "The γ-φ approach models the liquid phase with an **activity "
        "coefficient**. From "
        "[Chapter II §2.2 — Activity Coefficient Models]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#22-activity-coefficient-models):\n"
        "\n"
        "> simple CEOS are only applicable to mixtures of molecules that do not "
        "possess strong specific interactions, and they tend to fail in "
        "predicting liquid phase properties … better results are obtained when "
        "the fugacity of each component in the liquid phase is estimated using "
        "an activity coefficient model.\n"
        "\n"
        "The thesis's Table 2.3 lists **Wilson, Scatchard-Hildebrand, Margules, "
        "and van Laar**. NRTL is the modernization's addition to that set: like "
        "Wilson it is a *local-composition* model whose parameters are "
        "interaction **energies**, so — unlike the dimensionless Margules/van "
        "Laar — it carries a genuine temperature dependence and therefore a "
        "non-zero **excess enthalpy**. From "
        "[§2.2.1 — Energy Properties using Activity Coefficient Models]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#221-energy-properties-using-activity-coefficient-models), "
        "the excess properties are\n"
        "\n"
        "$$G^E = RT\\sum_i x_i \\ln\\gamma_i, \\qquad "
        "H^E = -T^2\\,\\frac{\\partial (G^E/T)}{\\partial T}, \\qquad "
        "S^E = \\frac{H^E - G^E}{T}. \\tag{2.44–2.46}$$\n"
        "\n"
        "For NRTL the derivative in (2.45) is taken **analytically** (via "
        "dual-number automatic differentiation), never by finite difference — "
        "the same rule the rest of the engine follows."
    ))

    # ---- What was built -------------------------------------------------
    cells.append(md(
        "## What Milestone 14 built\n"
        "\n"
        "- **`ActivityModel.Nrtl`** — the general multicomponent NRTL model. For "
        "a pair its activity coefficients are\n"
        "\n"
        "$$\\ln\\gamma_1 = x_2^2\\left[\\tau_{21}"
        "\\left(\\tfrac{G_{21}}{x_1+x_2 G_{21}}\\right)^2 + "
        "\\tau_{12}\\tfrac{G_{12}}{(x_2+x_1 G_{12})^2}\\right],\\quad "
        "\\tau_{ij} = \\frac{g_{ij}-g_{jj}}{RT},\\quad "
        "G_{ij} = e^{-\\alpha_{ij}\\tau_{ij}}$$\n"
        "\n"
        "  with the symmetric non-randomness $\\alpha_{ij}=\\alpha_{ji}$. The "
        "engine implements the full multicomponent form; the binary above is "
        "just its two-component reduction.\n"
        "- **The `alpha` matrix** — a new symmetric $N\\times N$ argument on "
        "`vle.System` (and the `activity_*` functions), alongside the `aij` "
        "interaction-energy matrix.\n"
        "- **Ammonia** in the bundled component database — `System([\"ammonia\", "
        "\"water\"])` now resolves both components.\n"
        "\n"
        "The interaction energies are stored in `aij` as $g_{ij}-g_{jj}$ in "
        "**kJ/kmol** (the same energy convention as Wilson)."
    ))

    # ---- Worked example: setup ------------------------------------------
    cells.append(md(
        "## Worked example — ammonia–water at 40 °C\n"
        "\n"
        "We parameterize NH₃(1)–H₂O(2) with $\\alpha = 0.2$ — the value "
        "recommended for ammonia mixtures, where the phase behavior is unusually "
        "sensitive to the non-randomness — and illustrative interaction energies "
        "chosen to reproduce the system's hallmark **negative deviation from "
        "Raoult's law** and **exothermic mixing**. (A production correlation "
        "regresses the energies against experimental VLE data; the shape here is "
        "physically correct, the magnitudes are illustrative.)"
    ))
    cells.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "%matplotlib inline\n"
        "\n"
        "import vle._engine as e\n"
        "from vle import System\n"
        "from vle.units import ureg, Q_\n"
        "\n"
        "# NH3(1)-H2O(2) NRTL parameters.\n"
        "#   aij[i][j] = g_ij - g_jj  [kJ/kmol]   (energy convention, like Wilson)\n"
        "#   alpha     = non-randomness (symmetric); 0.2 is the ammonia value.\n"
        "AIJ = [[0.0, -1800.0], [-1200.0, 0.0]]     # illustrative energies\n"
        "ALPHA = [[0.0, 0.2], [0.2, 0.0]]\n"
        "T = Q_(40, 'degC')                          # 313.15 K\n"
        "print('T =', T.to('K'))"
    ))

    # ---- Worked example: gamma + HE -------------------------------------
    cells.append(md(
        "### Activity coefficients and the heat of mixing\n"
        "\n"
        "We sweep composition and evaluate $\\gamma_i$ and the excess enthalpy "
        "$H^E$ directly through the engine's activity bindings. Two signatures of "
        "ammonia–water should appear: both $\\gamma_i < 1$ (negative deviation — "
        "the components *like* each other) and $H^E < 0$ (mixing releases heat)."
    ))
    cells.append(code(
        "Tk = T.to('K').magnitude\n"
        "x1 = np.linspace(1e-4, 1 - 1e-4, 41)\n"
        "\n"
        "def gammas_and_HE(x1v):\n"
        "    x = [x1v, 1 - x1v]\n"
        "    g1 = e.activity_ln_gamma(e.ActivityModel.Nrtl, 0, x, AIJ, alpha=ALPHA, t=Tk)\n"
        "    g2 = e.activity_ln_gamma(e.ActivityModel.Nrtl, 1, x, AIJ, alpha=ALPHA, t=Tk)\n"
        "    he = e.activity_excess_enthalpy(e.ActivityModel.Nrtl, x, AIJ, alpha=ALPHA, t=Tk)\n"
        "    return np.exp(g1), np.exp(g2), he\n"
        "\n"
        "g1, g2, he = np.vectorize(gammas_and_HE)(x1)\n"
        "\n"
        "fig, (axg, axh) = plt.subplots(1, 2, figsize=(11, 4))\n"
        "axg.plot(x1, g1, label=r'$\\gamma_{NH_3}$')\n"
        "axg.plot(x1, g2, label=r'$\\gamma_{H_2O}$')\n"
        "axg.axhline(1.0, color='gray', lw=0.8, ls='--')\n"
        "axg.set_xlabel(r'$x_{NH_3}$'); axg.set_ylabel(r'$\\gamma_i$')\n"
        "axg.set_title('Activity coefficients (NRTL, 40 °C)'); axg.legend()\n"
        "\n"
        "axh.plot(x1, he, color='C3')\n"
        "axh.axhline(0.0, color='gray', lw=0.8, ls='--')\n"
        "axh.set_xlabel(r'$x_{NH_3}$'); axh.set_ylabel(r'$H^E$  [kJ/kmol]')\n"
        "axh.set_title('Excess enthalpy — exothermic mixing')\n"
        "fig.tight_layout(); plt.show()"
    ))
    cells.append(md(
        "The activity coefficients dip well below 1 and both return to 1 at their "
        "pure-component limits; $H^E$ is negative across the whole range and peaks "
        "(in magnitude) near equimolar — exactly the exothermic behavior that "
        "makes ammonia–water useful as an absorption working pair, and that a "
        "temperature-independent model (Margules, van Laar) **cannot** reproduce "
        "($H^E \\equiv G^E$ there). Let's pin these facts."
    ))
    cells.append(code(
        "# Numeric checkpoints (a regression here means a real behavioral change).\n"
        "gm1, gm2, he_mid = gammas_and_HE(0.5)\n"
        "assert gm1 < 1.0 and gm2 < 1.0, 'NH3-H2O shows negative deviation (gamma<1)'\n"
        "assert he_mid < 0.0, 'mixing is exothermic (H^E < 0)'\n"
        "# As x_NH3 -> 0 the *solvent* (water) gamma -> 1 and H^E -> 0; the\n"
        "# ammonia gamma tends to its (finite, far-from-1) infinite-dilution value.\n"
        "g1_inf, g2_end, he_end = gammas_and_HE(1e-4)\n"
        "assert abs(g2_end - 1.0) < 1e-3 and abs(he_end) < 1.0\n"
        "# Thermodynamic consistency S^E = (H^E - G^E)/T through the wheel.\n"
        "xm = [0.5, 0.5]\n"
        "ge = e.activity_excess_gibbs(e.ActivityModel.Nrtl, xm, AIJ, alpha=ALPHA, t=Tk)\n"
        "se = e.activity_excess_entropy(e.ActivityModel.Nrtl, xm, AIJ, alpha=ALPHA, t=Tk)\n"
        "assert abs(se - (he_mid - ge) / Tk) < 1e-9\n"
        "print(f'gamma_NH3(0.5)={gm1:.3f}  gamma_H2O(0.5)={gm2:.3f}  H^E(0.5)={he_mid:.1f} kJ/kmol')"
    ))

    # ---- Worked example: bubble-P --------------------------------------
    cells.append(md(
        "### Bubble-pressure curve\n"
        "\n"
        "Now the same model inside a `System`: the γ-φ bubble pressure of the "
        "liquid at 40 °C as a function of ammonia loading. The vapor is nearly "
        "pure ammonia even at modest liquid fractions — the reason an "
        "ammonia–water absorber can strip ammonia into the vapor so effectively."
    ))
    cells.append(code(
        "sys = System(['ammonia', 'water'], vapor_model='ideal',\n"
        "             liquid_model='activity', activity='nrtl', aij=AIJ, alpha=ALPHA)\n"
        "\n"
        "xs = np.linspace(0.05, 0.85, 17)\n"
        "P = np.array([sys.bubble_pressure([xi, 1 - xi], T).value for xi in xs])\n"
        "yNH3 = np.array([sys.bubble_pressure([xi, 1 - xi], T).y[0] for xi in xs])\n"
        "\n"
        "fig, ax = plt.subplots(figsize=(6, 4))\n"
        "ax.plot(xs, P, 'o-', label='bubble P')\n"
        "ax.set_xlabel(r'$x_{NH_3}$ (liquid)'); ax.set_ylabel('P [kPa]')\n"
        "ax.set_title('NH$_3$–H$_2$O bubble pressure (NRTL, 40 °C)')\n"
        "ax2 = ax.twinx(); ax2.plot(xs, yNH3, 's--', color='C1', label=r'$y_{NH_3}$')\n"
        "ax2.set_ylabel(r'$y_{NH_3}$ (vapor)'); ax2.set_ylim(0.9, 1.005)\n"
        "fig.legend(loc='upper left', bbox_to_anchor=(0.13, 0.87))\n"
        "fig.tight_layout(); plt.show()\n"
        "\n"
        "assert np.all(np.diff(P) > 0), 'bubble P must rise with ammonia loading'\n"
        "assert np.all(yNH3 > xs), 'vapor is enriched in the volatile ammonia (y > x)'\n"
        "print(f'P range: {P.min():.0f} - {P.max():.0f} kPa')"
    ))
    cells.append(md(
        "> **Accuracy note.** These curves are *qualitatively* correct — the "
        "signs, shapes, and limits are right — but the magnitudes depend on the "
        "illustrative energies and on an ideal-gas vapor. Ammonia's Antoine "
        "correlation is extrapolated above its fitted range here, and the γ-φ "
        "path with an ideal vapor loses accuracy as pressure climbs into the MPa "
        "range. For quantitative work, regress the NRTL energies against "
        "experimental NH₃–H₂O VLE and use a cubic (or reference) vapor. The "
        "textbook ammonia–water enthalpy–composition chart is reproduced from "
        "reference data by a downstream staged-separation consumer, not from this fit."
    ))

    # ---- §8 follow-up: van Laar vs NRTL on methanol–water ---------------
    cells.append(md(
        "## Same data, two models: van Laar vs NRTL on methanol–water\n"
        "\n"
        "The thesis's validated methanol–water case ([Chapter IV Table 4.6]"
        "(../docs/en/research-paper/chapter-4-validation.md), reproduced in "
        "[notebook 04](04_bubble_dew_point.ipynb)) uses **van Laar** — and the "
        "README's P–x–y hero image deliberately keeps it, because provenance "
        "wins there. But now that NRTL is in the engine, a fair question is: "
        "*would NRTL describe that same mixture better?*\n"
        "\n"
        "Here we answer it the way a practitioner would with a DECHEMA table: "
        "**fit NRTL to the thesis's own six bubble-pressure points** "
        "(Levenberg–Marquardt via `fit_aij_py` — Milestone 9's regression "
        "driving Milestone 14's model — with $\\alpha_{12} = 0.30$ held fixed, "
        "the standard aqueous-alcohol value) and overlay both models on the "
        "data."
    ))
    cells.append(code(
        "from vle import components\n"
        "\n"
        "# Thesis Table 4.6: methanol(1)/water(2) bubble points at 298 K.\n"
        "# Literals copied from notebook 04 (single source of truth) —\n"
        "# see scripts/build_notebook_m9_bubble_dew.py.\n"
        "T_MW = 298.0                                   # K\n"
        "TABLE46 = [\n"
        "    # (x1, y1_thesis, P_thesis [kPa])\n"
        "    (0.0873, 0.4416, 5.1998),\n"
        "    (0.1900, 0.6287, 7.0028),\n"
        "    (0.3417, 0.7538, 9.1151),\n"
        "    (0.4943, 0.8334, 10.9757),\n"
        "    (0.6919, 0.9090, 13.2939),\n"
        "    (0.8492, 0.9583, 15.1678),\n"
        "]\n"
        "# van Laar parameters the thesis validated against (Table 4.5).\n"
        "AIJ_VL = [[0.0, 0.5853], [0.3458, 0.0]]\n"
        "# NRTL non-randomness, fixed during the fit.\n"
        "ALPHA_MW = [[0.0, 0.30], [0.30, 0.0]]\n"
        "\n"
        "meoh, h2o = components.get('methanol'), components.get('water')\n"
        "\n"
        "sys_vl = System(['methanol', 'water'], vapor_model='ideal',\n"
        "                liquid_model='activity', activity='van_laar', aij=AIJ_VL)\n"
        "\n"
        "def rmse(sys_):\n"
        "    r = [sys_.bubble_pressure([x1, 1 - x1], T_MW).value - p_ref\n"
        "         for x1, _, p_ref in TABLE46]\n"
        "    return float(np.sqrt(np.mean(np.square(r))))\n"
        "\n"
        "rmse_vl = rmse(sys_vl)\n"
        "print(f'van Laar (thesis Table 4.5 parameters): RMSE = {rmse_vl:.4f} kPa')"
    ))
    cells.append(code(
        "# Fit the two NRTL energies to the six thesis points (alpha fixed).\n"
        "# fit_aij_py takes the pure-component data explicitly — we feed it the\n"
        "# same bundled-DB methanol/water the System uses, so the fit and the\n"
        "# System evaluate identical saturation pressures.\n"
        "data = [(T_MW, x1, p_ref) for x1, _, p_ref in TABLE46]\n"
        "a12_mw, a21_mw, sse_mw, rmse_fit, iters_mw = e.fit_aij_py(\n"
        "    e.ActivityModel.Nrtl,\n"
        "    [meoh.tc, h2o.tc], [meoh.pc, h2o.pc], [meoh.omega, h2o.omega],\n"
        "    [list(meoh.psat_coeffs), list(h2o.psat_coeffs)],\n"
        "    data, 1000.0, 1000.0, alpha=ALPHA_MW,\n"
        ")\n"
        "print(f'NRTL fit: g12-g22 = {a12_mw:.1f}, g21-g11 = {a21_mw:.1f} kJ/kmol '\n"
        "      f'({iters_mw} LM iterations)')\n"
        "\n"
        "sys_nrtl = System(['methanol', 'water'], vapor_model='ideal',\n"
        "                  liquid_model='activity', activity='nrtl',\n"
        "                  aij=[[0.0, a12_mw], [a21_mw, 0.0]], alpha=ALPHA_MW)\n"
        "rmse_nrtl = rmse(sys_nrtl)\n"
        "print(f'NRTL (fitted to Table 4.6):             RMSE = {rmse_nrtl:.4f} kPa')"
    ))
    cells.append(code(
        "# Overlay: both model curves over the thesis points, plus the activity\n"
        "# coefficients each model implies.\n"
        "x_mw = np.linspace(0.03, 0.97, 33)\n"
        "P_vl   = [sys_vl.bubble_pressure([x1, 1 - x1], T_MW).value for x1 in x_mw]\n"
        "P_nrtl = [sys_nrtl.bubble_pressure([x1, 1 - x1], T_MW).value for x1 in x_mw]\n"
        "\n"
        "AIJ_NRTL_MW = [[0.0, a12_mw], [a21_mw, 0.0]]\n"
        "def gam(model, i, x1, **kw):\n"
        "    return np.exp(e.activity_ln_gamma(model, i, [x1, 1 - x1], **kw))\n"
        "g1_vl   = [gam(e.ActivityModel.VanLaar, 0, x1, aij=AIJ_VL) for x1 in x_mw]\n"
        "g2_vl   = [gam(e.ActivityModel.VanLaar, 1, x1, aij=AIJ_VL) for x1 in x_mw]\n"
        "g1_nrtl = [gam(e.ActivityModel.Nrtl, 0, x1, aij=AIJ_NRTL_MW, alpha=ALPHA_MW, t=T_MW) for x1 in x_mw]\n"
        "g2_nrtl = [gam(e.ActivityModel.Nrtl, 1, x1, aij=AIJ_NRTL_MW, alpha=ALPHA_MW, t=T_MW) for x1 in x_mw]\n"
        "\n"
        "fig, (axp, axg) = plt.subplots(1, 2, figsize=(11, 4))\n"
        "axp.plot(x_mw, P_vl, '-', label='van Laar (thesis params)')\n"
        "axp.plot(x_mw, P_nrtl, '--', label=r'NRTL (fitted, $\\alpha=0.30$)')\n"
        "axp.plot([r[0] for r in TABLE46], [r[2] for r in TABLE46], 'ko',\n"
        "         mfc='none', label='Table 4.6 data')\n"
        "axp.set_xlabel(r'$x_{CH_3OH}$'); axp.set_ylabel('bubble P [kPa]')\n"
        "axp.set_title('Methanol–water at 298 K'); axp.legend()\n"
        "\n"
        "axg.plot(x_mw, g1_vl, '-', color='C0', label=r'$\\gamma_1$ van Laar')\n"
        "axg.plot(x_mw, g2_vl, '-', color='C2', label=r'$\\gamma_2$ van Laar')\n"
        "axg.plot(x_mw, g1_nrtl, '--', color='C0', label=r'$\\gamma_1$ NRTL')\n"
        "axg.plot(x_mw, g2_nrtl, '--', color='C2', label=r'$\\gamma_2$ NRTL')\n"
        "axg.axhline(1.0, color='gray', lw=0.8, ls=':')\n"
        "axg.set_xlabel(r'$x_{CH_3OH}$'); axg.set_ylabel(r'$\\gamma_i$')\n"
        "axg.set_title('Activity coefficients'); axg.legend(fontsize=8)\n"
        "fig.tight_layout(); plt.show()"
    ))
    cells.append(md(
        "Both models sit on the data. The freshly fitted NRTL edges out van "
        "Laar's *literature* parameters on RMSE — unsurprisingly, since NRTL "
        "was fitted to these exact six points while the van Laar constants "
        "came from Table 4.5 (Orbey & Sandler) — but both errors are of order "
        "**1% of the pressure**, inside the saturation-correlation band the "
        "thesis itself flags. The honest conclusion: **for a fully miscible "
        "binary on a single isotherm, van Laar and NRTL fit comparably.** "
        "NRTL's real advantages are structural, not a better one-isotherm "
        "fit:\n"
        "\n"
        "- **temperature extrapolation** — $\\tau = \\Delta g/(RT)$ carries "
        "built-in T-dependence; van Laar's constants are frozen at 298 K;\n"
        "- a **non-zero analytic $H^E$** (the whole point of the NH₃–H₂O "
        "sections above; van Laar's crude legacy $H^E = G^E$ has the wrong "
        "shape *and* no T-dependence);\n"
        "- **multicomponent and LLE reach** — the general NRTL form extends "
        "to ternaries (extractive/azeotropic distillation) where van Laar "
        "does not.\n"
        "\n"
        "That is also why the README hero image *stays* van Laar: it "
        "showcases the thesis's validated configuration, and NRTL would not "
        "visibly improve the picture."
    ))
    cells.append(code(
        "# Pin the comparison so a regression shows up as a failing notebook.\n"
        "assert iters_mw < 100, 'NRTL LM fit failed to converge'\n"
        "assert rmse_nrtl <= 1.5 * rmse_vl, (\n"
        "    'NRTL fit should be comparable to van Laar on its own data')\n"
        "# Spot gammas: methanol-water shows mild POSITIVE deviation (contrast\n"
        "# with NH3-H2O's gamma < 1 above) and both models agree on that.\n"
        "g1v = gam(e.ActivityModel.VanLaar, 0, 0.5, aij=AIJ_VL)\n"
        "g1n = gam(e.ActivityModel.Nrtl, 0, 0.5, aij=AIJ_NRTL_MW, alpha=ALPHA_MW, t=T_MW)\n"
        "assert 1.0 < g1v < 1.3 and 1.0 < g1n < 1.3\n"
        "print(f'RMSE: van Laar {rmse_vl:.4f} kPa, NRTL {rmse_nrtl:.4f} kPa; '\n"
        "      f'gamma1(0.5): van Laar {g1v:.4f}, NRTL {g1n:.4f}')"
    ))

    # ---- Exercises ------------------------------------------------------
    cells.append(md(
        "## Exercises\n"
        "\n"
        "### Exercise 1 — how much does α matter?\n"
        "\n"
        "For ammonia mixtures the non-randomness α is unusually influential. "
        "Recompute $\\gamma_{NH_3}$ at $x_{NH_3}=0.3$ for $\\alpha \\in "
        "\\{0.2, 0.3, 0.47\\}$ (energies fixed) and report how much the activity "
        "coefficient moves."
    ))
    cells.append(code(
        "# TODO: for each a in (0.2, 0.3, 0.47), build ALPHA = [[0,a],[a,0]],\n"
        "# evaluate ln gamma of NH3 (index 0) at x=[0.3, 0.7], T=Tk, and print\n"
        "# gamma = exp(ln gamma).\n"
    ))
    cells.append(md(
        "<details><summary>Show solution</summary>\n"
        "\n"
        "```python\n"
        "for a in (0.2, 0.3, 0.47):\n"
        "    al = [[0.0, a], [a, 0.0]]\n"
        "    lng = e.activity_ln_gamma(e.ActivityModel.Nrtl, 0, [0.3, 0.7], AIJ, alpha=al, t=Tk)\n"
        "    print(f'alpha={a}:  gamma_NH3 = {np.exp(lng):.4f}')\n"
        "```\n"
        "\n"
        "The activity coefficient shifts by several percent across the α range — "
        "which is why fixing α = 0.3 out of habit is a poor default for ammonia, "
        "and α = 0.2 is recommended instead.\n"
        "</details>"
    ))
    cells.append(md(
        "### Exercise 2 — recover NRTL energies by regression\n"
        "\n"
        "The engine can fit the two interaction energies to bubble-pressure data "
        "(Levenberg–Marquardt, with α held fixed). Generate a synthetic "
        "\"experimental\" dataset from the parameters above, then fit fresh "
        "energies back and confirm the fit reproduces the pressures. This is the "
        "machinery a real NH₃–H₂O correlation uses — only with measured data."
    ))
    cells.append(code(
        "# TODO: build a list of (T[K], x1, P_exp[kPa]) tuples from sys.bubble_pressure\n"
        "# at x1 in [0.2, 0.35, 0.5, 0.65, 0.8], then call e.fit_aij_py(...) with\n"
        "# model=Nrtl, the ammonia/water tcs/pcs/omegas/psat_coeffs, alpha=ALPHA,\n"
        "# and initial guesses a12_0=a21_0=-500. Inspect the returned rmse.\n"
    ))
    cells.append(md(
        "<details><summary>Show solution</summary>\n"
        "\n"
        "```python\n"
        "from vle import components\n"
        "nh3, h2o = components.get('ammonia'), components.get('water')\n"
        "data = []\n"
        "for x1v in (0.2, 0.35, 0.5, 0.65, 0.8):\n"
        "    P = sys.bubble_pressure([x1v, 1 - x1v], T).value\n"
        "    data.append((Tk, x1v, P))\n"
        "\n"
        "a12, a21, sse, rmse, iters = e.fit_aij_py(\n"
        "    e.ActivityModel.Nrtl,\n"
        "    [nh3.tc, h2o.tc], [nh3.pc, h2o.pc], [nh3.omega, h2o.omega],\n"
        "    [list(nh3.psat_coeffs), list(h2o.psat_coeffs)],\n"
        "    data, -500.0, -500.0, alpha=ALPHA,\n"
        ")\n"
        "print(f'fitted g12-g22={a12:.1f}, g21-g11={a21:.1f} kJ/kmol; rmse={rmse:.3e} kPa')\n"
        "assert rmse < 1.0  # reproduces the synthetic pressures\n"
        "```\n"
        "\n"
        "The RMSE collapses to well under 1 kPa — the regressor recovers energies "
        "consistent with the data (they need not equal the originals: α-fixed "
        "P–x data does not identify the energies uniquely, but it pins the "
        "pressures the model predicts).\n"
        "</details>"
    ))
    cells.append(md(
        "### Exercise 3 — is α identifiable from one isotherm?\n"
        "\n"
        "Exercise 1 showed that at *fixed energies*, changing α moves "
        "$\\gamma_{NH_3}$ by several percent. Now flip the question for the "
        "methanol–water comparison: **refit** the NRTL energies to Table 4.6 "
        "with $\\alpha_{12} = 0.20$ and $0.47$ (instead of 0.30) and compare "
        "the RMSEs. What does that say about fitting α itself from a single "
        "isotherm?"
    ))
    cells.append(code(
        "# TODO: for a in (0.20, 0.47): build al = [[0, a], [a, 0]], call\n"
        "# e.fit_aij_py(...) exactly as in the comparison section (same data,\n"
        "# same initial guesses), and print the fitted energies + rmse.\n"
    ))
    cells.append(md(
        "<details><summary>Show solution</summary>\n"
        "\n"
        "```python\n"
        "for a in (0.20, 0.47):\n"
        "    al = [[0.0, a], [a, 0.0]]\n"
        "    f12, f21, _, r, it = e.fit_aij_py(\n"
        "        e.ActivityModel.Nrtl,\n"
        "        [meoh.tc, h2o.tc], [meoh.pc, h2o.pc], [meoh.omega, h2o.omega],\n"
        "        [list(meoh.psat_coeffs), list(h2o.psat_coeffs)],\n"
        "        data, 1000.0, 1000.0, alpha=al,\n"
        "    )\n"
        "    print(f'alpha={a}: g12-g22={f12:7.1f}  g21-g11={f21:7.1f} kJ/kmol  '\n"
        "          f'rmse={r:.4f} kPa  ({it} iters)')\n"
        "```\n"
        "\n"
        "All three α choices converge to essentially the **same RMSE** — the "
        "energies simply shift to compensate. On a single isotherm of P–x data "
        "α is **not identifiable**: fixing it to a literature value (0.30 for "
        "aqueous alcohols, 0.20 for ammonia) and fitting only the energies is "
        "the statistically honest procedure. Discriminating α requires data "
        "the parameter actually leverages — multiple temperatures, $H^E$ "
        "measurements, or LLE.\n"
        "</details>"
    ))

    # ---- References -----------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- Renon, H.; Prausnitz, J. M. Local Compositions in Thermodynamic "
        "Excess Functions for Liquid Mixtures. *AIChE J.* **1968**, *14*, 135. "
        "(The NRTL model.)\n"
        "- Research paper: "
        "[Chapter II §2.2 — Activity Coefficient Models]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#22-activity-coefficient-models) "
        "and [§2.2.1 — Energy Properties]"
        "(../docs/en/research-paper/chapter-2-vle-theory.md#221-energy-properties-using-activity-coefficient-models) "
        "(the excess-property equations 2.44–2.46).\n"
        "- On α = 0.2 for ammonia mixtures: Junqua, G. *et al.* Reconsideration "
        "of the α coefficient in the NRTL model … refrigerant mixtures. "
        "*Int. J. Refrig.* **2019**; Zhang, X. *et al.* I&EC Res. **2017**, "
        "*56*, 12525.\n"
        "- Design record: `NRTL_AMMONIA_PLAN.md` (§8 = the van Laar-vs-NRTL "
        "comparison); `MODERNIZATION_PLAN.md` Phase 21 (Milestone 14).\n"
        "- Table 4.6 data and the thesis's van Laar validation: "
        "[notebook 04](04_bubble_dew_point.ipynb) and "
        "[Chapter IV §4.3](../docs/en/research-paper/chapter-4-validation.md).\n"
        "\n"
        "**Why this milestone exists:** ammonia–water is the classic absorption "
        "working pair, and teaching its enthalpy–composition (Ponchon–Savarit) "
        "method downstream needs a liquid model with a real heat of mixing plus "
        "ammonia in the database. NRTL supplies both — and lifts the whole "
        "aqueous-nonideal ladder (alcohol/acetone–water and the later "
        "extractive ternaries), not just this one pair."
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
