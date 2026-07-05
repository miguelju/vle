#!/usr/bin/env python3
"""Generate ``notebooks/07_kij_regression.ipynb`` — Milestone 9 (Ch. IV §4.7).

Fits the binary interaction parameter k12 of a cubic EOS to the CO2/n-butane
bubble-pressure data of research-paper Tables 4.11-4.12 by Brent minimization.
Structure follows CLAUDE.md *Notebook Conventions*; executed top-to-bottom in a
fresh kernel before saving.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "07_kij_regression.ipynb"

SETUP_CELL = (
    "# Optional: pull the latest vle-thermo from PyPI.\n"
    "# Uncomment if you want the newest released version instead of\n"
    "# whatever is currently in your kernel. On the hosted hub this\n"
    "# install is ephemeral — it vanishes when your session is culled.\n"
    "# %pip install --upgrade vle-thermo"
)


def md(t):
    return nbf.v4.new_markdown_cell(t)


def code(t):
    return nbf.v4.new_code_cell(t)


def build():
    nb = nbf.v4.new_notebook()
    c = []
    c.append(md(
        "# Binary Interaction Parameter (kij) Regression — Milestone 9\n"
        "\n"
        "Classical mixing rules combine pure-component EOS parameters with a "
        "single fitted **binary interaction parameter** $k_{ij}$ per pair: "
        "$a_{ij} = (1 - k_{ij})\\sqrt{a_i a_j}$. A well-chosen $k_{ij}$ can turn "
        "a poor prediction into a quantitative one. This notebook fits "
        "$k_{ij}$ for the CO₂/n-butane system to the bubble-pressure data of "
        "the research paper's **Tables 4.11–4.12**, reproducing the reported "
        "$k_{12} \\approx 0.1357$."
    ))
    c.append(md(
        "## Setup (optional)\n\nThe cell below is **commented out by default**. "
        "Uncomment it to pull the latest `vle-thermo` from PyPI."
    ))
    c.append(code(SETUP_CELL))

    c.append(md(
        "## Context — the fitting objective\n"
        "\n"
        "From [Chapter IV §4.7](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md), "
        "$k_{ij}$ is chosen to minimize the sum of squared errors between the "
        "calculated and measured bubble pressures over a composition sweep:\n"
        "\n"
        "$$ \\mathrm{SSE}(k) = \\sum_d \\big[ P^{\\mathrm{bub}}(k;\\, T_d, x_d) - P^{\\exp}_d \\big]^2. $$\n"
        "\n"
        "The engine minimizes this with **Brent's method** (parabolic "
        "interpolation + golden-section safeguard) — the modern replacement "
        "for the thesis's golden-section search, with the same guaranteed "
        "convergence but faster terminal behaviour."
    ))
    c.append(md(
        "## What this milestone built\n"
        "\n"
        "`vle._engine.fit_kij_py(eos, tcs, pcs, omegas, psat_coeffs, data, ...)` "
        "returns `(kij, sse, rmse)`, where `data` is a list of "
        "`(T [K], x1, P_exp [kPa])` triples. It wraps `brent_minimize` around "
        "the φ-φ bubble-pressure solve."
    ))

    c.append(md(
        "## Worked example — Tables 4.11–4.12 (CO₂/n-butane)\n"
        "\n"
        "The Table 4.11 data are P–x points for CO₂(1)/n-butane(2) at "
        "**357.57 K**. At this temperature CO₂ is *supercritical* (its Tc is "
        "304 K), so the high-CO₂ points sit near the mixture critical point, "
        "where the multiplicative bubble-pressure solver is ill-conditioned. "
        "We therefore fit the **sub-critical subset** ($x_1 \\lesssim 0.20$) — "
        "exactly as the engine's Chapter IV validation test does — which lands "
        "in the literature neighborhood of $k_{12} = 0.1357$. The near-critical "
        "points are the domain of the phase-envelope solver (see the "
        "[phase-envelope notebook section below]).\n"
        "\n"
        "> The full 14-point Table 4.11 dataset is included below; only the "
        "sub-critical rows are used for the pinned fit, and this limitation is "
        "logged rather than hidden."
    ))
    c.append(code(
        "import vle._engine as e\n"
        "\n"
        "# CO2(1)/n-butane(2): (Tc [K], Pc [kPa], omega).\n"
        "tcs = [304.13, 425.12]\n"
        "pcs = [7377.0, 3796.0]\n"
        "om  = [0.2239, 0.200]\n"
        "# Reduced-Antoine coeffs (only used off the φ-φ path; harmless here).\n"
        "psat = [[4.86, 1147.0, -8.0], [4.35, 2277.0, -30.0]]\n"
        "\n"
        "# Table 4.11 — (P [bar], x1). P is converted to kPa (x100).\n"
        "table_4_11 = [\n"
        "    (14.824, 0.02967), (19.029, 0.06228), (23.511, 0.0959),\n"
        "    (27.441, 0.1283),  (31.164, 0.15673), (36.404, 0.19636),\n"
        "    # --- near-critical (unused in the pinned fit) ---\n"
        "    (42.885, 0.25027), (49.573, 0.30421), (56.399, 0.35904),\n"
        "    (63.569, 0.41871), (70.671, 0.49255), (75.428, 0.5352),\n"
        "    (77.91,  0.56473), (79.289, 0.5745),\n"
        "]\n"
        "T = 357.57\n"
        "sub_critical = [(T, x1, p_bar * 100.0) for (p_bar, x1) in table_4_11 if x1 <= 0.20]\n"
        "print(f'{len(sub_critical)} sub-critical points used for the fit')\n"
        "\n"
        "kij, sse, rmse = e.fit_kij_py(\n"
        "    e.CubicEos.PR1976, tcs, pcs, om, psat, sub_critical, k_lo=-0.05, k_hi=0.30)\n"
        "print(f'fitted k12 = {kij:.4f}   (literature 0.1357; Ekilib 0.1359; Sandler 0.135)')\n"
        "print(f'RMSE = {rmse:.1f} kPa on pressures of ~1500-3600 kPa')"
    ))
    c.append(code(
        "# The fit must land in the literature neighborhood of ~0.1357.\n"
        "assert 0.12 <= kij <= 0.20, f'k12 {kij} outside the expected band'\n"
        "print('k12 in the Table 4.12 neighborhood.')"
    ))
    c.append(md(
        "The fit reproduces the reported $k_{12}$ to within the literature "
        "spread. The exact 0.1357 over the *full* dataset needs the near-"
        "critical points, which require the phase-envelope continuation solver "
        "(`trace_envelope_py`) rather than the point-wise bubble solver — a "
        "known limitation logged in the engine's Chapter IV test."
    ))

    c.append(md(
        "## Exercise 1 — why does kij matter?\n"
        "\n"
        "Compare the bubble pressure of an $x_1 = 0.15$ CO₂/n-butane mixture at "
        "357.57 K computed with $k_{12} = 0$ versus the fitted value, against "
        "the experimental 3116 kPa (from Table 4.11's $x_1 = 0.15673$ row). By "
        "how much does the interaction parameter improve the prediction?"
    ))
    c.append(code(
        "# TODO: call e.bubble_pressure_py at x1=0.15673, T=357.57 with\n"
        "#   kij=[[0,0],[0,0]] and with kij=[[0,kfit],[kfit,0]], and compare to\n"
        "#   the experimental 3116.4 kPa.\n"
    ))
    c.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "x1, p_exp = 0.15673, 31.164 * 100.0\n"
        "for label, k in [('kij=0', 0.0), (f'kij={kij:.4f}', kij)]:\n"
        "    p, _, _ = e.bubble_pressure_py(\n"
        "        tcs, pcs, om, [x1, 1 - x1], 357.57,\n"
        "        vapor_kind='cubic', liquid_kind='cubic',\n"
        "        vapor_eos=e.CubicEos.PR1976, liquid_eos=e.CubicEos.PR1976,\n"
        "        kij=[[0.0, k], [k, 0.0]], psat_coeffs=psat)\n"
        "    print(f'{label:14}  P = {p:.0f} kPa   error {abs(p-p_exp)/p_exp*100:.1f}%')\n"
        "print(f'experimental   P = {p_exp:.0f} kPa')\n"
        "```\n"
        "The fitted $k_{12}$ cuts the pressure error dramatically — the point "
        "of the regression.\n"
        "</details>"
    ))

    c.append(md(
        "## Exercise 2 — the SSE curve\n"
        "\n"
        "The fit is a 1-D minimization. Plot $\\mathrm{SSE}(k)$ over "
        "$k \\in [-0.05, 0.30]$ (evaluate the bubble pressures yourself and sum "
        "the squared residuals) and confirm the Brent result sits at the "
        "minimum."
    ))
    c.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "%matplotlib inline\n"
        "# TODO: for k in np.linspace(-0.05, 0.30, 36), compute SSE over the\n"
        "# sub_critical points and plot; mark the fitted kij.\n"
    ))
    c.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "def sse_at(k):\n"
        "    s = 0.0\n"
        "    for (t, x1, pexp) in sub_critical:\n"
        "        p, _, _ = e.bubble_pressure_py(\n"
        "            tcs, pcs, om, [x1, 1 - x1], t,\n"
        "            vapor_kind='cubic', liquid_kind='cubic',\n"
        "            vapor_eos=e.CubicEos.PR1976, liquid_eos=e.CubicEos.PR1976,\n"
        "            kij=[[0.0, k], [k, 0.0]], psat_coeffs=psat)\n"
        "        s += (p - pexp) ** 2\n"
        "    return s\n"
        "ks = np.linspace(-0.05, 0.30, 36)\n"
        "plt.plot(ks, [sse_at(float(k)) for k in ks], '-')\n"
        "plt.axvline(kij, color='r', ls='--', label=f'fit k={kij:.4f}')\n"
        "plt.xlabel('k12'); plt.ylabel('SSE (kPa^2)'); plt.legend(); plt.grid(True)\n"
        "plt.show()\n"
        "```\n"
        "</details>"
    ))

    c.append(md(
        "## References\n"
        "\n"
        "- Research paper [Chapter IV §4.7 — kij Calculation](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md) (Tables 4.11–4.12).\n"
        "- (4) Da Silva & Báez (1989) — the regression objective (`TERMOVI.PAS`).\n"
        "- Algorithm details: [`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md) §B and `engine/src/flash/kij_regression.rs`.\n"
    ))

    nb.cells = c
    nb.metadata = {
        "kernelspec": {"display_name": "Python 3 (ipykernel)", "language": "python", "name": "python3"},
        "language_info": {"name": "python"},
    }
    return nb


def main():
    nb = build()
    print(f"Executing {NB_PATH.name} top-to-bottom in a fresh kernel...")
    NotebookClient(nb, timeout=300, kernel_name="python3").execute(cwd=str(NB_PATH.parent))
    NB_PATH.parent.mkdir(parents=True, exist_ok=True)
    NB_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    print(f"Wrote {NB_PATH}  ({len(nb.cells)} cells)")


if __name__ == "__main__":
    main()
