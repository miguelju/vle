#!/usr/bin/env python3
"""Generate ``notebooks/03_activity_models.ipynb`` — the Milestone 8.1 notebook.

Covers the activity-coefficient layer of the γ-φ model: the five activity
models of research-paper Table 2.3 (Ideal, Margules, van Laar, Wilson,
Scatchard-Hildebrand), the excess properties Gᴱ / Hᴱ / Sᴱ (eqs 2.44–2.46), and
the Rackett / Thomson liquid molar volumes the Wilson and Scatchard models need.
Ref (4): Da Silva & Báez (1989); liquid volume: Hankinson & Thomson (18).

Structure follows CLAUDE.md *Notebook Conventions*. Generated deterministically
and executed top-to-bottom in a fresh kernel before saving.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "03_activity_models.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Activity Coefficient Models — Milestone 8.1\n"
        "\n"
        "Cubic equations of state describe the **vapor** phase well but predict "
        "strongly non-ideal **liquid** mixtures (water + alcohol, say) poorly. "
        "The γ-φ approach fixes this by modelling the liquid with an *activity "
        "coefficient* $\\gamma_i$ — a multiplier on Raoult's law that captures "
        "how much component $i$ 'dislikes' (γ > 1) or 'likes' (γ < 1) its "
        "neighbours. This milestone ports the five activity models of the "
        "research paper's Table 2.3, their excess properties $G^E, H^E, S^E$, "
        "and the saturated-liquid molar volumes that the Wilson and "
        "Scatchard-Hildebrand models depend on. Ref (4): Da Silva & Báez "
        "(1989); liquid volume: Hankinson & Thomson (18)."
    ))

    # ---- Optional upgrade cell (CLAUDE.md §2b) --------------------------
    cells.append(md(
        "## Setup (optional)\n"
        "\n"
        "The cell below is **commented out by default**. Uncomment it if "
        "you want to use the latest `vle-thermo` released on PyPI instead "
        "of whatever version is currently installed in your kernel."
    ))
    cells.append(code(
        "# Optional: pull the latest vle-thermo from PyPI.\n"
        "# Uncomment if you want the newest released version instead of\n"
        "# whatever is currently in your kernel.\n"
        "# %pip install --upgrade vle-thermo"
    ))

    # ---- Research-paper context -----------------------------------------
    cells.append(md(
        "## Context — the activity-coefficient method\n"
        "\n"
        "From [Chapter II §2.2](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md), "
        "the liquid-phase fugacity in the γ-φ model is\n"
        "\n"
        "$$\\hat f_i^L = x_i\\,\\gamma_i\\,f_i^0\\,F_i,$$\n"
        "\n"
        "where $F_i$ is the Poynting factor and $\\gamma_i$ comes from one of the "
        "models in **Table 2.3**:\n"
        "\n"
        "| Model | Parameters | $\\ln\\gamma_i$ |\n"
        "|---|---|---|\n"
        "| Wilson | $\\Lambda_{ij}=(V_j^L/V_i^L)\\,e^{-(\\lambda_{ij}-\\lambda_{ii})/RT}$ | $-\\ln(\\sum_j x_j\\Lambda_{ij})+1-\\sum_k x_k\\Lambda_{ki}/\\sum_j x_j\\Lambda_{kj}$ |\n"
        "| Scatchard-Hildebrand | $\\delta_i,\\;V_i^L$ | $V_i^L(\\delta_i-\\delta_{mix})^2/RT$, $\\delta_{mix}=\\sum_i\\Phi_i\\delta_i$ |\n"
        "| Margules (binary) | $A_{12},A_{21}$ | $x_2^2[A_{12}+2(A_{21}-A_{12})x_1]$ |\n"
        "| van Laar (binary) | $A_{12},A_{21}$ | $A_{12}/[1+(A_{12}x_1)/(A_{21}x_2)]^2$ |\n"
        "\n"
        "and the **excess properties** follow directly from the activity "
        "coefficients (eqs 2.44–2.46):\n"
        "\n"
        "$$G^E = RT\\sum_i x_i\\ln\\gamma_i,\\quad "
        "H^E = -T^2\\,\\partial(G^E/T)/\\partial T,\\quad "
        "S^E = (H^E-G^E)/T.$$\n"
        "\n"
        "Per the project's algorithm rules the temperature derivative in $H^E$ is "
        "**analytical**. The Wilson, Scatchard, and van Laar models need a "
        "liquid molar volume $V_i^L$, obtained from the Rackett or "
        "Thomson/COSTALD (18) correlation."
    ))

    # ---- What was built --------------------------------------------------
    cells.append(md(
        "## What was built in Milestone 8.1\n"
        "\n"
        "Through `vle._engine`:\n"
        "\n"
        "- `liquid_molar_volume(model, tc, pc, t, zra=…, vstar=…, omega_srk=…)` "
        "— Rackett (`VolumeModel.Rackett`) or Thomson/COSTALD "
        "(`VolumeModel.Thomson`), returning cm³/mol.\n"
        "- `activity_ln_gamma(model, i, x, aij, vl=…, delta=…, t=…)` — "
        "$\\ln\\gamma_i$ for any of the five `ActivityModel` variants.\n"
        "- `activity_excess_gibbs / _enthalpy / _entropy(model, x, aij, …)` — "
        "$G^E, H^E, S^E$ in kJ/kmol (and kJ/(kmol·K) for $S^E$).\n"
        "\n"
        "The `aij` matrix convention is per-model: Wilson uses interaction "
        "energies $\\lambda_{ij}-\\lambda_{ii}$ in kJ/kmol; van Laar/Margules "
        "use dimensionless $A_{ij}$ with zero diagonal; Scatchard/Ideal ignore "
        "it."
    ))

    # ---- Worked example: setup ------------------------------------------
    cells.append(md(
        "## Worked example — ethanol (1) / water (2)\n"
        "\n"
        "A textbook positively-deviating binary. We first get the liquid molar "
        "volumes from Rackett, then drive the activity models with them."
    ))
    cells.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "import vle._engine as e\n"
        "%matplotlib inline\n"
        "\n"
        "print('vle-thermo', e.version())\n"
        "\n"
        "T = 298.15  # K\n"
        "# Critical data + Rackett ZRA for ethanol and water.\n"
        "ETHANOL = dict(tc=513.9, pc=6148.0, zra=0.248)\n"
        "WATER   = dict(tc=647.3, pc=22120.0, zra=0.235)\n"
        "\n"
        "vl = [\n"
        "    e.liquid_molar_volume(e.VolumeModel.Rackett, ETHANOL['tc'], ETHANOL['pc'], T, zra=ETHANOL['zra']),\n"
        "    e.liquid_molar_volume(e.VolumeModel.Rackett, WATER['tc'],   WATER['pc'],   T, zra=WATER['zra']),\n"
        "]\n"
        "print(f'V_L  ethanol={vl[0]:.1f}  water={vl[1]:.1f}  cm3/mol')\n"
        "assert 50 < vl[0] < 65 and 15 < vl[1] < 20"
    ))

    # ---- gamma vs composition -------------------------------------------
    cells.append(md(
        "### Activity coefficients vs composition\n"
        "\n"
        "We use a van Laar fit ($A_{12}=1.6$, $A_{21}=0.9$) and a Wilson fit "
        "(energies in kJ/kmol). Both should give $\\gamma_i \\to 1$ as "
        "$x_i \\to 1$ (pure component) and the largest $\\gamma_i$ at infinite "
        "dilution."
    ))
    cells.append(code(
        "x1 = np.linspace(1e-4, 1 - 1e-4, 80)\n"
        "\n"
        "vanlaar_aij = [[0.0, 1.6], [0.9, 0.0]]          # dimensionless A12, A21\n"
        "wilson_aij  = [[0.0, 3300.0], [1800.0, 0.0]]    # (lambda_ij - lambda_ii) kJ/kmol\n"
        "\n"
        "def gammas(model, aij, **kw):\n"
        "    g1, g2 = [], []\n"
        "    for a in x1:\n"
        "        xs = [a, 1 - a]\n"
        "        g1.append(np.exp(e.activity_ln_gamma(model, 0, xs, aij, t=T, **kw)))\n"
        "        g2.append(np.exp(e.activity_ln_gamma(model, 1, xs, aij, t=T, **kw)))\n"
        "    return np.array(g1), np.array(g2)\n"
        "\n"
        "vl_g1, vl_g2 = gammas(e.ActivityModel.VanLaar, vanlaar_aij)\n"
        "wi_g1, wi_g2 = gammas(e.ActivityModel.Wilson, wilson_aij, vl=vl)\n"
        "\n"
        "fig, ax = plt.subplots(1, 2, figsize=(11, 4), sharey=True)\n"
        "ax[0].plot(x1, vl_g1, label=r'$\\gamma_1$ (ethanol)')\n"
        "ax[0].plot(x1, vl_g2, label=r'$\\gamma_2$ (water)')\n"
        "ax[0].set_title('van Laar'); ax[0].set_xlabel(r'$x_1$'); ax[0].set_ylabel(r'$\\gamma_i$'); ax[0].legend()\n"
        "ax[1].plot(x1, wi_g1, label=r'$\\gamma_1$ (ethanol)')\n"
        "ax[1].plot(x1, wi_g2, label=r'$\\gamma_2$ (water)')\n"
        "ax[1].set_title('Wilson'); ax[1].set_xlabel(r'$x_1$'); ax[1].legend()\n"
        "for a in ax: a.axhline(1.0, ls=':', c='grey')\n"
        "plt.tight_layout(); plt.show()\n"
        "\n"
        "# Pure-component limits: gamma -> 1.\n"
        "assert abs(vl_g1[-1] - 1.0) < 1e-2 and abs(vl_g2[0] - 1.0) < 1e-2\n"
        "# van Laar infinite-dilution limit: ln gamma_1 -> A12.\n"
        "assert abs(np.log(vl_g1[0]) - 1.6) < 1e-2"
    ))

    # ---- excess properties ----------------------------------------------
    cells.append(md(
        "### Excess Gibbs energy and enthalpy\n"
        "\n"
        "$G^E$ vanishes at both pure limits and peaks near equimolar. For "
        "Wilson the temperature dependence of $\\Lambda_{ij}$ makes $H^E \\ne "
        "G^E$ (a real heat of mixing); for van Laar the legacy convention gives "
        "$H^E = G^E$."
    ))
    cells.append(code(
        "ge_vl = [e.activity_excess_gibbs(e.ActivityModel.VanLaar, [a, 1 - a], vanlaar_aij, t=T) for a in x1]\n"
        "ge_wi = [e.activity_excess_gibbs(e.ActivityModel.Wilson, [a, 1 - a], wilson_aij, vl=vl, t=T) for a in x1]\n"
        "he_wi = [e.activity_excess_enthalpy(e.ActivityModel.Wilson, [a, 1 - a], wilson_aij, vl=vl, t=T) for a in x1]\n"
        "\n"
        "plt.figure(figsize=(7, 4))\n"
        "plt.plot(x1, ge_vl, label=r'$G^E$ van Laar')\n"
        "plt.plot(x1, ge_wi, label=r'$G^E$ Wilson')\n"
        "plt.plot(x1, he_wi, '--', label=r'$H^E$ Wilson')\n"
        "plt.axhline(0.0, ls=':', c='grey')\n"
        "plt.xlabel(r'$x_1$'); plt.ylabel('kJ/kmol')\n"
        "plt.title('Excess properties — ethanol/water'); plt.legend(); plt.tight_layout(); plt.show()\n"
        "\n"
        "# G^E vanishes at the pure limits.\n"
        "assert abs(ge_wi[0]) < 1.0 and abs(ge_wi[-1]) < 1.0"
    ))

    # ---- analytical vs numerical HE -------------------------------------
    cells.append(md(
        "### The analytical $H^E$ is exact\n"
        "\n"
        "The engine's analytical Wilson $H^E$ matches a finite-difference "
        "evaluation of $-T^2\\,d(G^E/T)/dT$ — the test-oracle pattern the "
        "project uses everywhere derivatives appear."
    ))
    cells.append(code(
        "xs = [0.4, 0.6]\n"
        "h = 1e-2\n"
        "g_over_t = lambda tt: e.activity_excess_gibbs(e.ActivityModel.Wilson, xs, wilson_aij, vl=vl, t=tt) / tt\n"
        "he_num = -T * T * (g_over_t(T + h) - g_over_t(T - h)) / (2 * h)\n"
        "he_ana = e.activity_excess_enthalpy(e.ActivityModel.Wilson, xs, wilson_aij, vl=vl, t=T)\n"
        "print(f'H^E analytical = {he_ana:8.2f} kJ/kmol   numerical = {he_num:8.2f} kJ/kmol')\n"
        "assert abs(he_ana - he_num) < 1e-2 * abs(he_num)"
    ))

    # ---- Exercise 1 ------------------------------------------------------
    cells.append(md(
        "## Exercise 1 — Margules vs van Laar\n"
        "\n"
        "Using the same $A_{12}=1.6$, $A_{21}=0.9$, plot $\\ln\\gamma_1$ vs "
        "$x_1$ for **both** the Margules and van Laar models on one axis. Where "
        "do they differ most? Fill in the `# TODO`."
    ))
    cells.append(code(
        "aij = [[0.0, 1.6], [0.9, 0.0]]\n"
        "# TODO: build lists `lg_marg` and `lg_vl` of ln(gamma_1) over x1 for the\n"
        "# Margules and van Laar models, then plot both against x1.\n"
    ))
    cells.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "lg_marg = [e.activity_ln_gamma(e.ActivityModel.Margules, 0, [a, 1 - a], aij) for a in x1]\n"
        "lg_vl   = [e.activity_ln_gamma(e.ActivityModel.VanLaar,  0, [a, 1 - a], aij) for a in x1]\n"
        "plt.figure(figsize=(7, 4))\n"
        "plt.plot(x1, lg_marg, label='Margules')\n"
        "plt.plot(x1, lg_vl, label='van Laar')\n"
        "plt.xlabel(r'$x_1$'); plt.ylabel(r'$\\ln\\gamma_1$')\n"
        "plt.title('Margules vs van Laar'); plt.legend(); plt.tight_layout(); plt.show()\n"
        "# They agree at the dilute and pure limits; the largest gap is mid-range.\n"
        "```\n"
        "</details>"
    ))

    # ---- Exercise 2 ------------------------------------------------------
    cells.append(md(
        "## Exercise 2 — Scatchard-Hildebrand needs no fitting\n"
        "\n"
        "The Scatchard-Hildebrand (regular-solution) model is *predictive*: it "
        "needs only pure-component solubility parameters $\\delta_i$ and liquid "
        "volumes — no binary data. For a non-polar pair "
        "($\\delta_1 = 7.4$, $\\delta_2 = 8.9\\,(\\mathrm{cal/cm^3})^{0.5}$, "
        "$V^L = [131, 89]\\,\\mathrm{cm^3/mol}$), compute and plot $\\gamma_1$ "
        "across composition, then confirm $\\gamma_1 \\to 1$ as $x_1 \\to 1$."
    ))
    cells.append(code(
        "delta = [7.4, 8.9]\n"
        "vl_sh = [131.0, 89.0]\n"
        "# TODO: g1 = exp(activity_ln_gamma(ScatchardHildebrand, 0, [a,1-a], [[0,0],[0,0]],\n"
        "#       vl=vl_sh, delta=delta, t=T)) for a in x1; plot; assert the pure limit.\n"
    ))
    cells.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "g1 = [np.exp(e.activity_ln_gamma(e.ActivityModel.ScatchardHildebrand, 0, [a, 1 - a],\n"
        "      [[0.0, 0.0], [0.0, 0.0]], vl=vl_sh, delta=delta, t=T)) for a in x1]\n"
        "plt.figure(figsize=(7, 4))\n"
        "plt.plot(x1, g1)\n"
        "plt.axhline(1.0, ls=':', c='grey')\n"
        "plt.xlabel(r'$x_1$'); plt.ylabel(r'$\\gamma_1$')\n"
        "plt.title('Scatchard-Hildebrand (predictive)'); plt.tight_layout(); plt.show()\n"
        "assert abs(g1[-1] - 1.0) < 1e-2\n"
        "```\n"
        "</details>"
    ))

    # ---- References ------------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- **(4)** Da Silva, F. A.; Báez, L. (1989) — the five activity models "
        "and excess properties (`legacy/pascal/TERMOIII.PAS`, "
        "`legacy/vb6/clsActivityMulticomp.cls`).\n"
        "- **(18)** Hankinson, R. W.; Thomson, G. H. — COSTALD liquid molar "
        "volume.\n"
        "- [Chapter II §2.2](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md) "
        "— activity-coefficient method and Table 2.3.\n"
        "- [`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md) "
        "— Phase 10 (activity models).\n"
        "- Previous notebook: [`02d_advanced_saturation.ipynb`](02d_advanced_saturation.ipynb)."
    ))

    nb.cells = cells
    nb.metadata = {
        "kernelspec": {
            "display_name": "Python 3 (ipykernel)",
            "language": "python",
            "name": "python3",
        },
        "language_info": {"name": "python"},
    }
    return nb


def main() -> None:
    nb = build()
    print(f"Executing {NB_PATH.name} top-to-bottom in a fresh kernel...")
    client = NotebookClient(nb, timeout=180, kernel_name="python3")
    client.execute(cwd=str(NB_PATH.parent))
    NB_PATH.parent.mkdir(parents=True, exist_ok=True)
    NB_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    print(f"Wrote {NB_PATH}  ({len(nb.cells)} cells)")


if __name__ == "__main__":
    main()
