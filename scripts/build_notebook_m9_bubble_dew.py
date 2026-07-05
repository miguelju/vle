#!/usr/bin/env python3
"""Generate ``notebooks/04_bubble_dew_point.ipynb`` — Milestone 9 (Ch. IV §4.3–4.5).

Bubble- and dew-point calculations via the γ-φ path: the van Laar bubble
pressures of research-paper Table 4.6 (methanol/water), plus dew-point and
bubble-temperature demonstrations. Structure follows CLAUDE.md *Notebook
Conventions*; executed top-to-bottom in a fresh kernel before saving.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "04_bubble_dew_point.ipynb"

SETUP_CELL = (
    "# Optional: pull the latest vle-thermo from PyPI.\n"
    "# Uncomment if you want the newest released version instead of\n"
    "# whatever is currently in your kernel.\n"
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
        "# Bubble & Dew Points — Milestone 9\n"
        "\n"
        "The **bubble point** is where a liquid first boils (an infinitesimal "
        "bubble of vapor appears); the **dew point** is where a vapor first "
        "condenses (an infinitesimal drop of liquid appears). Together they "
        "bound the two-phase region. This notebook reproduces the research "
        "paper's **Table 4.6** bubble pressures (methanol/water with the van "
        "Laar activity model, §4.3) and demonstrates the dew-point and "
        "bubble-temperature solvers (§4.4–4.5)."
    ))
    c.append(md(
        "## Setup (optional)\n\nThe cell below is **commented out by default**. "
        "Uncomment it to pull the latest `vle-thermo` from PyPI."
    ))
    c.append(code(SETUP_CELL))

    c.append(md(
        "## Context — the γ-φ saturation condition\n"
        "\n"
        "From [Chapter IV §4.3](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md), "
        "a non-ideal liquid is described by activity coefficients $\\gamma_i$ "
        "(the γ-φ approach). With an ideal vapor, the equilibrium ratio is "
        "the *modified Raoult's law* $K_i = \\gamma_i P^{\\mathrm{sat}}_i / P$, "
        "and the saturation conditions are\n"
        "\n"
        "$$ \\text{bubble:}\\ \\sum_i K_i x_i = 1, \\qquad "
        "\\text{dew:}\\ \\sum_i y_i / K_i = 1. $$\n"
        "\n"
        "The engine converges the incipient-phase composition by successive "
        "substitution and adjusts the free variable (T or P) until the "
        "saturation sum equals 1."
    ))
    c.append(md(
        "## What this milestone built\n"
        "\n"
        "- `vle._engine.bubble_pressure_py(...)` / `bubble_temperature_py(...)` "
        "→ `(P or T, incipient vapor y, K)`.\n"
        "- `vle._engine.dew_pressure_py(...)` / `dew_temperature_py(...)` → "
        "`(P or T, incipient liquid x, K)`.\n"
    ))

    c.append(md(
        "## Worked example — Table 4.6 (van Laar bubble pressure)\n"
        "\n"
        "A methanol(1)/water(2) mixture at **298 K**, described by the van Laar "
        "model with $\\Lambda_{12} = 0.5853$, $\\Lambda_{21} = 0.3458$ "
        "(Table 4.5, Orbey & Sandler), ideal vapor. We use reduced-Antoine "
        "saturation coefficients calibrated to the pure vapor pressures."
    ))
    c.append(code(
        "import vle._engine as e\n"
        "\n"
        "# methanol(1), water(2): (Tc [K], Pc [kPa], omega).\n"
        "tcs = [512.6, 647.1]\n"
        "pcs = [8097.0, 22064.0]\n"
        "om  = [0.564, 0.344]\n"
        "# Reduced-Antoine ln(P/Pc) = a1 - a2/(a3 + T), fit to the pure Psat's.\n"
        "psat = [[7.493, 3603.0, -34.29],   # methanol\n"
        "        [6.240, 3803.0, -46.00]]   # water\n"
        "# van Laar parameters (Table 4.5): aij[0][1]=Λ12, aij[1][0]=Λ21.\n"
        "aij = [[0.0, 0.5853], [0.3458, 0.0]]\n"
        "\n"
        "def bubble_P(x1):\n"
        "    return e.bubble_pressure_py(\n"
        "        tcs, pcs, om, [x1, 1 - x1], 298.0,\n"
        "        vapor_kind='ideal', liquid_kind='activity',\n"
        "        liquid_activity=e.ActivityModel.VanLaar, aij=aij,\n"
        "        psat_coeffs=psat, tol=1e-10)\n"
        "\n"
        "p, y, k = bubble_P(0.4943)\n"
        "print(f'x1 = 0.4943 ->  P = {p:.3f} kPa (thesis 10.976)   y1 = {y[0]:.4f} (thesis 0.8334)')"
    ))
    c.append(code(
        "# Reproduce the full Table 4.6 and compare.\n"
        "table = [\n"
        "    # (x1, y1_thesis, P_thesis)\n"
        "    (0.0873, 0.4416, 5.1998),\n"
        "    (0.1900, 0.6287, 7.0028),\n"
        "    (0.3417, 0.7538, 9.1151),\n"
        "    (0.4943, 0.8334, 10.9757),\n"
        "    (0.6919, 0.9090, 13.2939),\n"
        "    (0.8492, 0.9583, 15.1678),\n"
        "]\n"
        "print(f\"{'x1':>6} {'y1':>8} {'y1_ref':>8} {'P':>8} {'P_ref':>8} {'%P':>6}\")\n"
        "for x1, y1_ref, p_ref in table:\n"
        "    p, y, _ = bubble_P(x1)\n"
        "    ep = abs(p - p_ref) / p_ref * 100\n"
        "    print(f'{x1:6.4f} {y[0]:8.4f} {y1_ref:8.4f} {p:8.3f} {p_ref:8.4f} {ep:6.2f}')\n"
        "    # y is model-robust and must match tightly; P carries the small\n"
        "    # saturation-correlation difference the thesis itself flags (<~2%).\n"
        "    assert abs(y[0] - y1_ref) < 0.01, f'y1 off at x1={x1}'\n"
        "    assert ep < 2.5, f'P error {ep:.2f}% at x1={x1}'\n"
        "print('\\nTable 4.6 reproduced (y tight, P within the Psat-correlation band).')"
    ))
    c.append(md(
        "The vapor compositions match the thesis to better than 1%, and the "
        "pressures to ~1% — the residual pressure difference is exactly what "
        "§4.3 attributes to the saturation-pressure correlation and the "
        "calculation precision."
    ))

    c.append(md(
        "## Dew point and bubble temperature\n"
        "\n"
        "The same system exposes the inverse solves. A useful self-consistency "
        "check: the **bubble temperature** at the bubble *pressure* we just "
        "computed must return the original 298 K."
    ))
    c.append(code(
        "x1 = 0.4943\n"
        "p_bub, _, _ = bubble_P(x1)\n"
        "t_back, y_back, _ = e.bubble_temperature_py(\n"
        "    tcs, pcs, om, [x1, 1 - x1], p_bub,\n"
        "    vapor_kind='ideal', liquid_kind='activity',\n"
        "    liquid_activity=e.ActivityModel.VanLaar, aij=aij, psat_coeffs=psat, tol=1e-9)\n"
        "print(f'bubble T at P={p_bub:.3f} kPa -> {t_back:.3f} K (should be ~298)')\n"
        "assert abs(t_back - 298.0) < 0.5\n"
        "\n"
        "# Dew pressure at the same T for a vapor of composition y = [0.6, 0.4].\n"
        "p_dew, x_inc, _ = e.dew_pressure_py(\n"
        "    tcs, pcs, om, [0.6, 0.4], 298.0,\n"
        "    vapor_kind='ideal', liquid_kind='activity',\n"
        "    liquid_activity=e.ActivityModel.VanLaar, aij=aij, psat_coeffs=psat, tol=1e-10)\n"
        "print(f'dew P (y1=0.6) = {p_dew:.3f} kPa, incipient liquid x1 = {x_inc[0]:.4f}')"
    ))

    c.append(md(
        "## Exercise 1 — the P–x–y diagram\n"
        "\n"
        "Build the classic pressure–composition diagram for methanol/water at "
        "298 K: plot the bubble pressure vs the *liquid* composition $x_1$ and "
        "the same pressure vs the *vapor* composition $y_1$ on one figure. The "
        "region between the two curves is two-phase."
    ))
    c.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "%matplotlib inline\n"
        "# TODO: for x1 in np.linspace(0.02, 0.98, 25): compute (P, y1) via\n"
        "# bubble_P and plot P-vs-x1 and P-vs-y1 on the same axes.\n"
    ))
    c.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "xs = np.linspace(0.02, 0.98, 25)\n"
        "Ps, ys = [], []\n"
        "for x1 in xs:\n"
        "    p, y, _ = bubble_P(float(x1))\n"
        "    Ps.append(p); ys.append(y[0])\n"
        "plt.plot(xs, Ps, '-', label='bubble (liquid x1)')\n"
        "plt.plot(ys, Ps, '-', label='dew (vapor y1)')\n"
        "plt.xlabel('methanol mole fraction'); plt.ylabel('P (kPa)')\n"
        "plt.title('methanol/water P-x-y at 298 K, van Laar'); plt.legend(); plt.grid(True)\n"
        "plt.show()\n"
        "```\n"
        "</details>"
    ))

    c.append(md(
        "## Exercise 2 — does the mixture form an azeotrope?\n"
        "\n"
        "An azeotrope is where the bubble and dew curves touch ($x_1 = y_1$). "
        "Using the bubble solver, find whether methanol/water has one at 298 K "
        "by locating where $y_1 - x_1$ changes sign (or confirm it does not)."
    ))
    c.append(code(
        "# TODO: over x1 in np.linspace(0.02, 0.98, 49), compute y1 - x1 and\n"
        "# report whether it crosses zero (an azeotrope) or keeps one sign.\n"
    ))
    c.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "diffs = [bubble_P(float(x1))[1][0] - x1 for x1 in np.linspace(0.02, 0.98, 49)]\n"
        "sign_changes = sum(1 for a, b in zip(diffs, diffs[1:]) if a * b < 0)\n"
        "print('azeotrope present' if sign_changes else 'no azeotrope: y1 > x1 throughout')\n"
        "```\n"
        "Methanol is more volatile than water across the whole range here, so "
        "$y_1 > x_1$ always and there is no azeotrope with these van Laar "
        "parameters.\n"
        "</details>"
    ))

    c.append(md(
        "## References\n"
        "\n"
        "- Research paper [Chapter IV §4.3–4.5 — Bubble/Dew Points](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md) (Tables 4.5–4.9).\n"
        "- Activity models: [Chapter II §2.2](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md) and the [activity-models notebook](03_activity_models.ipynb).\n"
        "- (21) Orbey & Sandler — the van Laar parameters.\n"
        "- Algorithm details: [`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md) §K and `engine/src/flash/{bubble,dew}.rs`.\n"
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
