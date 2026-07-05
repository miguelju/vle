#!/usr/bin/env python3
"""Generate ``notebooks/08_aij_regression.ipynb`` — Milestone 9 (Pascal-origin Aij fit).

Fits activity-model binary parameters (A12, A21) to bubble-pressure data by
Levenberg-Marquardt, recovering the van Laar parameters of research-paper
Table 4.5 from the Table 4.6 methanol/water data. Structure follows CLAUDE.md
*Notebook Conventions*; executed top-to-bottom in a fresh kernel before saving.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "08_aij_regression.ipynb"

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
        "# Activity-Model Parameter (Aij) Regression — Milestone 9\n"
        "\n"
        "Activity models (van Laar, Wilson, Margules) carry two binary "
        "parameters $(A_{12}, A_{21})$ that are fitted to experimental data. "
        "This notebook fits them by **Levenberg–Marquardt** — the modern "
        "replacement for the thesis's plain Newton–Raphson (Ref (4), Pascal "
        "`TERMOV.PAS`) — and shows a satisfying closure: starting from the "
        "methanol/water bubble-pressure data of research-paper **Table 4.6**, "
        "it *recovers the van Laar parameters of Table 4.5* that generated it."
    ))
    c.append(md(
        "## Setup (optional)\n\nThe cell below is **commented out by default**. "
        "Uncomment it to pull the latest `vle-thermo` from PyPI."
    ))
    c.append(code(SETUP_CELL))

    c.append(md(
        "## Context — why Levenberg–Marquardt\n"
        "\n"
        "The fit minimizes the bubble-pressure residuals "
        "$r_d = P^{\\mathrm{bub}}(A_{12}, A_{21};\\, T_d, x_d) - P^{\\exp}_d$ "
        "over the two parameters. Levenberg–Marquardt interpolates between "
        "**Gauss–Newton** (fast near the optimum) and **gradient descent** "
        "(robust far from it) through a damping parameter $\\lambda$, so it "
        "converges gracefully from a poor initial guess where the thesis's "
        "plain Newton could diverge. See "
        "[Chapter IV](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md) "
        "and the activity models in "
        "[Chapter II §2.2](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md)."
    ))
    c.append(md(
        "## What this milestone built\n"
        "\n"
        "`vle._engine.fit_aij_py(model, tcs, pcs, omegas, psat_coeffs, data, "
        "a12_0, a21_0, vl=[], ...)` returns "
        "`(a12, a21, sse, rmse, iterations)`, where `data` is a list of "
        "`(T [K], x1, P_exp [kPa])` triples. It wraps a Levenberg–Marquardt "
        "loop around the γ-φ bubble-pressure solve."
    ))

    c.append(md(
        "## Worked example — recover the Table 4.5 van Laar parameters\n"
        "\n"
        "The Table 4.6 methanol(1)/water(2) bubble pressures at 298 K were "
        "computed with the van Laar parameters $\\Lambda_{12} = 0.5853$, "
        "$\\Lambda_{21} = 0.3458$ (Table 4.5). If we hand those pressures to "
        "the regression as if they were experimental data, it should recover "
        "the parameters — a clean end-to-end check of the whole fitting "
        "pipeline."
    ))
    c.append(code(
        "import vle._engine as e\n"
        "\n"
        "# methanol(1), water(2).\n"
        "tcs = [512.6, 647.1]\n"
        "pcs = [8097.0, 22064.0]\n"
        "om  = [0.564, 0.344]\n"
        "psat = [[7.493, 3603.0, -34.29], [6.240, 3803.0, -46.0]]\n"
        "\n"
        "# Table 4.6 (x1, P [kPa]) at 298 K — the 'experimental' data.\n"
        "data = [\n"
        "    (298.0, 0.0873, 5.1998), (298.0, 0.1900, 7.0028),\n"
        "    (298.0, 0.3417, 9.1151), (298.0, 0.4943, 10.9757),\n"
        "    (298.0, 0.6919, 13.2939), (298.0, 0.8492, 15.1678),\n"
        "]\n"
        "\n"
        "a12, a21, sse, rmse, iters = e.fit_aij_py(\n"
        "    e.ActivityModel.VanLaar, tcs, pcs, om, psat, data,\n"
        "    a12_0=0.4, a21_0=0.4)   # deliberately-off initial guess\n"
        "print(f'fitted:  A12 = {a12:.4f} (Table 4.5: 0.5853)   A21 = {a21:.4f} (0.3458)')\n"
        "print(f'RMSE = {rmse:.3f} kPa   converged in {iters} LM iterations')"
    ))
    c.append(code(
        "# The recovered parameters must be close to the Table 4.5 values, and\n"
        "# the fit must reproduce the pressures to well under 1%.\n"
        "assert abs(a12 - 0.5853) < 0.03, f'A12 {a12} off'\n"
        "assert abs(a21 - 0.3458) < 0.03, f'A21 {a21} off'\n"
        "assert rmse < 0.1, f'rmse {rmse} kPa too large'\n"
        "print('van Laar parameters recovered from the P-x data.')"
    ))
    c.append(md(
        "Starting from a deliberately wrong guess $(0.4, 0.4)$, LM converges to "
        "within a few percent of the true van Laar parameters — the small "
        "residual reflects the difference between the saturation-pressure "
        "correlation used here and the one behind the tabulated pressures."
    ))

    c.append(md(
        "## Exercise 1 — robustness to the initial guess\n"
        "\n"
        "Re-run the fit from several very different starting points "
        "(e.g. $(0.1, 0.1)$, $(1.0, 0.05)$, $(0.05, 1.0)$) and confirm LM "
        "reaches essentially the same optimum each time. This is the property "
        "that plain Newton lacks."
    ))
    c.append(code(
        "# TODO: loop over a few (a12_0, a21_0) starts, call fit_aij_py, and\n"
        "# print the converged (a12, a21) for each.\n"
    ))
    c.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "for a0, b0 in [(0.1, 0.1), (1.0, 0.05), (0.05, 1.0), (0.6, 0.35)]:\n"
        "    a, b, _, r, it = e.fit_aij_py(\n"
        "        e.ActivityModel.VanLaar, tcs, pcs, om, psat, data, a0, b0)\n"
        "    print(f'start ({a0:.2f},{b0:.2f}) -> A12={a:.4f} A21={b:.4f} rmse={r:.3f} ({it} it)')\n"
        "```\n"
        "All starts converge to the same neighborhood — LM's damping keeps it "
        "in the basin of attraction.\n"
        "</details>"
    ))

    c.append(md(
        "## Exercise 2 — fit a Wilson model instead\n"
        "\n"
        "Fit the **Wilson** model (`e.ActivityModel.Wilson`) to the same data. "
        "Wilson needs the liquid molar volumes, passed via `vl=[…]` in cm³/mol "
        "(methanol ≈ 40.7, water ≈ 18.07), and its parameters are energies in "
        "kJ/kmol (start near a few hundred). Does Wilson fit the P–x data as "
        "well as van Laar?"
    ))
    c.append(code(
        "# TODO: call e.fit_aij_py with e.ActivityModel.Wilson, vl=[40.7, 18.07],\n"
        "# and starting guesses around a few hundred kJ/kmol; report the RMSE.\n"
    ))
    c.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "a, b, _, r, it = e.fit_aij_py(\n"
        "    e.ActivityModel.Wilson, tcs, pcs, om, psat, data,\n"
        "    a12_0=500.0, a21_0=500.0, vl=[40.7, 18.07])\n"
        "print(f'Wilson: A12={a:.1f} A21={b:.1f} kJ/kmol  rmse={r:.3f} kPa ({it} it)')\n"
        "```\n"
        "Wilson reproduces the pressures comparably well; its parameters are "
        "less tightly identified from bubble-pressure data alone, so the exact "
        "values depend more on the starting point than van Laar's do.\n"
        "</details>"
    ))

    c.append(md(
        "## References\n"
        "\n"
        "- Research paper [Chapter IV](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md) (Tables 4.5–4.6) and [Chapter II §2.2](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md) (activity models).\n"
        "- (4) Da Silva & Báez (1989) — the Aij regression (`TERMOV.PAS`).\n"
        "- (21) Orbey & Sandler — the van Laar parameters (Table 4.5).\n"
        "- Algorithm details: `engine/src/flash/aij_regression.rs`; see also the [activity-models notebook](03_activity_models.ipynb).\n"
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
