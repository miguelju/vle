#!/usr/bin/env python3
"""Generate ``notebooks/02d_advanced_saturation.ipynb`` — the Milestone 7.4 notebook.

Covers the advanced saturation-pressure layer: the Riedel / Müller / RPM
corresponding-states correlations, the Maxwell equal-area construction, boiling
temperature and the Poynting correction, and the **OL-family α** (whose α is
coupled to the reduced saturation pressure). Ref (4): Da Silva & Báez (1989);
OL family: Olivera et al. (1998).

Structure follows CLAUDE.md *Notebook Conventions*. Generated deterministically
and executed top-to-bottom in a fresh kernel before saving.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "02d_advanced_saturation.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Advanced Saturation Models — Milestone 7.4 (v0.6.0)\n"
        "\n"
        "Antoine is accurate over a narrow band; extrapolating it to the "
        "critical point or to low reduced temperatures goes wrong. This "
        "milestone adds the **corresponding-states** saturation correlations "
        "(Riedel, Müller, RPM) that use $T_c$, $P_c$, $T_b$, and ω; the "
        "**Maxwell equal-area** construction (the thermodynamically exact "
        "saturation pressure of a cubic EOS); **boiling-temperature** "
        "inversion and the **Poynting** correction; and the **OL-family α** "
        "(VdWOL / RKOL / PROL), whose α reads the reduced saturation pressure "
        "— which is why it had to wait for this layer. Ref (4): Da Silva & "
        "Báez (1989); OL: Olivera et al. (1998)."
    ))

    # ---- Notebook sandbox notice (CLAUDE.md "Notebook Conventions" §2) -------
    cells.append(md(
        "> 💾 **Notebook sandbox notice — only applies if you're running this "
        "notebook on a shared JupyterLab someone set up for you.** If you were given "
        "a URL to a shared JupyterLab environment, treat it as an "
        "*educational sandbox*: edits you make to this notebook won't survive "
        "a container restart, the bundled `vle-thermo` version may lag PyPI, "
        "and any `pip install` you run inside this container is ephemeral "
        "(it vanishes when your session is culled). For real work, install "
        "`vle-thermo` in your own Jupyter environment with "
        "`pip install vle-thermo` and run the notebook there — see the "
        "[project README](https://github.com/miguelju/vle/blob/main/README.md). "
        "**If you opened this notebook in your own Jupyter, you can ignore "
        "this notice.**"
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
        "# whatever is currently in your kernel. On the hosted hub this\n"
        "# install is ephemeral — it vanishes when your session is culled.\n"
        "# %pip install --upgrade vle-thermo"
    ))

    # ---- Research-paper context -----------------------------------------
    cells.append(md(
        "## Context — saturation pressure beyond Antoine\n"
        "\n"
        "From [Chapter II](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md), "
        "the saturation (vapor) pressure $P^{sat}(T)$ is the backbone of "
        "low-pressure VLE. This milestone ports several models, all returning "
        "$P^{sat}$ in **kPa**:\n"
        "\n"
        "| Model | Inputs | Note |\n"
        "|---|---|---|\n"
        "| Antoine | `[a1,a2,a3]` | $\\ln(P/P_c)=a_1-a_2/(a_3+T)$; analytical $dP/dT$ |\n"
        "| Riedel | $T_c,P_c,T_b$ | Riedel-criterion corresponding states |\n"
        "| Müller | $T_c,P_c,T_b,\\omega$ | reduced-property correlation |\n"
        "| RPM | $T_c,P_c,T_b$ | Riedel-Plank-Miller |\n"
        "| Maxwell | a cubic EOS | equal-fugacity on the EOS isotherm (exact) |\n"
        "\n"
        "The **OL-family α** is special: "
        "$\\alpha(T_r)=T_r\\,(1+\\mathrm{SumHk})$, where SumHk is a polynomial "
        "in $-\\ln(P^{sat}_r/T_r)$ with family-specific coefficients "
        "(`clsQbicsPure.cls:268`). Because it reads the **reduced saturation "
        "pressure**, the OL α only makes sense once a saturation model exists "
        "— hence it lands here. In the engine the model is chosen per component "
        "via `Component.sat_model` (default Antoine); the binding "
        "`eos_alpha_ol` takes it explicitly."
    ))

    # ---- What was built --------------------------------------------------
    cells.append(md(
        "## What was built in Milestone 7.4\n"
        "\n"
        "Through `vle._engine`:\n"
        "\n"
        "- `sat_psat(model, t, tc, pc, omega, tb, coeffs)` and "
        "`sat_reduced_psat(...)` — Riedel / Müller / RPM / Polynomial; "
        "`sat_d_psat_dt(...)` (analytical for Antoine, numerical otherwise).\n"
        "- `sat_maxwell(eos, t, tc, pc, omega, coeffs)` — Maxwell equal-area "
        "saturation pressure for a cubic EOS.\n"
        "- `boiling_temperature(model, p, …)` — invert $P^{sat}(T)=P$ "
        "(closed form for Antoine, Brent otherwise).\n"
        "- `poynting_factor(p, psat, t, liquid_volume)`.\n"
        "- `eos_alpha_ol` / `eos_d_alpha_d_tr_ol` — the OL-family α with an "
        "**analytical** dα/dTr (via the chain rule through $dP^{sat}/dT$)."
    ))

    # ---- Worked example: setup ------------------------------------------
    cells.append(md("## Worked example\n\nImports and n-pentane saturation data."))
    cells.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "import vle._engine as e\n"
        "%matplotlib inline\n"
        "\n"
        "print('vle-thermo', e.version())\n"
        "\n"
        "P = dict(tc=469.7, pc=3370.0, omega=0.252, tb=309.2)   # n-pentane\n"
        "ANTOINE = [6.738, 3165.0, 0.0]                          # reduced Antoine fit\n"
        "\n"
        "def psat(model, t, coeffs=None):\n"
        "    return e.sat_psat(model, t, P['tc'], P['pc'], P['omega'], P['tb'], coeffs or [])"
    ))

    # ---- Saturation comparison plot -------------------------------------
    cells.append(md(
        "### Saturation curves: Antoine vs corresponding-states models\n"
        "\n"
        "All correlations are tuned to pass through 1 atm at the normal "
        "boiling point $T_b$; they diverge as you move away from it."
    ))
    cells.append(code(
        "Ts = np.linspace(290.0, 460.0, 60)\n"
        "models = [\n"
        "    ('Antoine', e.SatPressureModel.Antoine, ANTOINE),\n"
        "    ('Riedel',  e.SatPressureModel.Riedel,  None),\n"
        "    ('Muller',  e.SatPressureModel.Muller,  None),\n"
        "    ('RPM',     e.SatPressureModel.RPM,     None),\n"
        "]\n"
        "plt.figure(figsize=(7, 4))\n"
        "for name, model, coeffs in models:\n"
        "    plt.plot(Ts, [psat(model, t, coeffs) for t in Ts], label=name)\n"
        "plt.axhline(101.325, ls=':', c='grey', label='1 atm')\n"
        "plt.axvline(P['tb'], ls=':', c='grey')\n"
        "plt.xlabel('Temperature (K)'); plt.ylabel('P_sat (kPa)')\n"
        "plt.title('Saturation pressure — n-pentane'); plt.legend(); plt.tight_layout(); plt.show()\n"
        "\n"
        "# Every corresponding-states model gives ~1 atm at Tb:\n"
        "for _, model, _ in models[1:]:\n"
        "    assert abs(psat(model, P['tb']) - 101.325) / 101.325 < 0.05"
    ))

    # ---- Maxwell + boiling + poynting -----------------------------------
    cells.append(md(
        "### Maxwell construction, boiling point, and Poynting\n"
        "\n"
        "Maxwell finds the pressure where the cubic EOS's liquid and vapor "
        "roots have equal fugacity — the thermodynamically exact $P^{sat}$."
    ))
    cells.append(code(
        "T = 350.0\n"
        "pm = e.sat_maxwell(e.CubicEos.PR1976, T, P['tc'], P['pc'], P['omega'], ANTOINE)\n"
        "pa = psat(e.SatPressureModel.Antoine, T, ANTOINE)\n"
        "print(f'Maxwell (PR) P_sat = {pm:8.2f} kPa   Antoine P_sat = {pa:8.2f} kPa')\n"
        "\n"
        "# Boiling temperature: invert P_sat(T) = P, then round-trip.\n"
        "p_target = 200.0\n"
        "tb = e.boiling_temperature(e.SatPressureModel.Antoine, p_target,\n"
        "                           P['tc'], P['pc'], 0.0, 0.0, ANTOINE)\n"
        "print(f'T_boil at {p_target} kPa = {tb:.2f} K  ->  P_sat(T_boil) = '\n"
        "      f'{e.antoine_psat(tb, P[\"pc\"], ANTOINE):.2f} kPa')\n"
        "assert abs(e.antoine_psat(tb, P['pc'], ANTOINE) - p_target) / p_target < 1e-6\n"
        "\n"
        "# Poynting factor for the compressed liquid (V_L = 116 cm3/mol).\n"
        "for p in (pa, 1000.0, 5000.0):\n"
        "    print(f'Poynting at P={p:7.1f} kPa (P_sat={pa:.1f}): {e.poynting_factor(p, pa, T, 116.0):.4f}')"
    ))

    # ---- OL alpha --------------------------------------------------------
    cells.append(md(
        "### The OL-family α (saturation-coupled)\n"
        "\n"
        "`eos_alpha_ol` evaluates $\\alpha=T_r(1+\\mathrm{SumHk})$ using the "
        "chosen saturation model. Here we use the Antoine model (so dα/dTr is "
        "fully analytical) and compare the OL α to plain Peng-Robinson."
    ))
    cells.append(code(
        "trs = np.linspace(0.55, 0.98, 50)\n"
        "ol = {'VdW-OL': e.CubicEos.VdWOL1998, 'RK-OL': e.CubicEos.RKOL1998, 'PR-OL': e.CubicEos.PROL1998}\n"
        "plt.figure(figsize=(7, 4))\n"
        "for name, eos in ol.items():\n"
        "    a = [e.eos_alpha_ol(eos, tr, P['tc'], P['pc'], P['omega'],\n"
        "                        e.SatPressureModel.Antoine, 0.0, ANTOINE) for tr in trs]\n"
        "    plt.plot(trs, a, label=name)\n"
        "plt.plot(trs, [e.eos_alpha(e.CubicEos.PR1976, tr, P['omega']) for tr in trs],\n"
        "         '--', c='k', label='PR (reference)')\n"
        "plt.xlabel(r'$T_r$'); plt.ylabel(r'$\\alpha(T_r)$')\n"
        "plt.title('OL-family α vs Peng-Robinson (n-pentane)'); plt.legend(); plt.tight_layout(); plt.show()\n"
        "\n"
        "# Analytical OL dα/dTr matches a central-difference oracle.\n"
        "h, tr = 1e-6, 0.8\n"
        "args = (P['tc'], P['pc'], P['omega'], e.SatPressureModel.Antoine, 0.0, ANTOINE)\n"
        "ana = e.eos_d_alpha_d_tr_ol(e.CubicEos.PROL1998, tr, *args)\n"
        "num = (e.eos_alpha_ol(e.CubicEos.PROL1998, tr + h, *args)\n"
        "       - e.eos_alpha_ol(e.CubicEos.PROL1998, tr - h, *args)) / (2 * h)\n"
        "assert abs((ana - num) / ana) < 1e-4"
    ))

    # ---- Exercise 1 ------------------------------------------------------
    cells.append(md(
        "## Exercise 1 — boiling point vs pressure\n"
        "\n"
        "Compute the Antoine boiling temperature of n-pentane at "
        "$P = 50, 100, 200, 500$ kPa and confirm each round-trips "
        "($P^{sat}(T_{boil}) = P$). Fill in the `# TODO`."
    ))
    cells.append(code(
        "pressures = [50.0, 100.0, 200.0, 500.0]\n"
        "# TODO: for each p, compute tb = e.boiling_temperature(e.SatPressureModel.Antoine, p,\n"
        "#       P['tc'], P['pc'], 0.0, 0.0, ANTOINE); print tb; assert the round-trip.\n"
    ))
    cells.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "for p in pressures:\n"
        "    tb = e.boiling_temperature(e.SatPressureModel.Antoine, p, P['tc'], P['pc'], 0.0, 0.0, ANTOINE)\n"
        "    back = e.antoine_psat(tb, P['pc'], ANTOINE)\n"
        "    print(f'P={p:6.1f} kPa -> T_boil={tb:6.2f} K -> P_sat={back:6.1f} kPa')\n"
        "    assert abs(back - p) / p < 1e-6\n"
        "```\n"
        "</details>"
    ))

    # ---- Exercise 2 ------------------------------------------------------
    cells.append(md(
        "## Exercise 2 — Maxwell vs a correlation\n"
        "\n"
        "Compare the **Maxwell** (PR EOS) saturation pressure to the "
        "**Riedel** correlation across $T = 320 \\dots 440$ K, and plot the "
        "ratio $P^{Maxwell}/P^{Riedel}$. Where do they agree best?"
    ))
    cells.append(code(
        "Ts2 = np.linspace(320.0, 440.0, 25)\n"
        "# TODO: build `ratio = [sat_maxwell(...) / psat(Riedel, t) for t in Ts2]` and plot it.\n"
    ))
    cells.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "ratio = [e.sat_maxwell(e.CubicEos.PR1976, t, P['tc'], P['pc'], P['omega'], ANTOINE)\n"
        "         / psat(e.SatPressureModel.Riedel, t) for t in Ts2]\n"
        "plt.figure(figsize=(7, 4))\n"
        "plt.plot(Ts2, ratio)\n"
        "plt.axhline(1.0, ls=':', c='grey')\n"
        "plt.xlabel('Temperature (K)'); plt.ylabel('P_Maxwell / P_Riedel')\n"
        "plt.title('Maxwell (PR) vs Riedel — n-pentane'); plt.tight_layout(); plt.show()\n"
        "```\n"
        "</details>"
    ))

    # ---- References ------------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- **(4)** Da Silva, F. A.; Báez, L. — Riedel / Müller / RPM / Antoine "
        "correlations and the Maxwell construction (`legacy/pascal/TERMOI.PAS`, "
        "`legacy/vb6/clsSatPressureSolver.cls`, `clsQbicsPure.cls`).\n"
        "- Olivera et al. (1998) — the OL-family α (`clsQbicsPure.cls:268`).\n"
        "- [Chapter II](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md) — VLE theory.\n"
        "- [`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md) — Phase 8 (saturation) notes.\n"
        "- Three-parameter EOS + Chao-Seader: [`02c_three_param_eos.ipynb`](02c_three_param_eos.ipynb)."
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
