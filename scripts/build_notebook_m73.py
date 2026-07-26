#!/usr/bin/env python3
"""Generate ``notebooks/02c_three_param_eos.ipynb`` — the Milestone 7.3 notebook.

Covers the three-parameter cubic EOS (Schmidt-Wenzel, Patel-Teja, Patel-Teja
USB) and the Chao-Seader pure-liquid fugacity correlation, all ported from the
Pascal legacy (Ref (4): Da Silva & Báez, 1989, legacy/pascal/TERMOII.PAS).

Structure follows CLAUDE.md *Notebook Conventions*: title + motivation →
notebook sandbox notice → optional pip install → research-paper context →
what was built → worked example → ≥2 user exercises → references.

The notebook is generated deterministically and executed top-to-bottom in a
fresh kernel before saving, so an engine regression surfaces here rather than
on the hub.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "02c_three_param_eos.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Three-Parameter EOS & Chao-Seader — Milestone 7.3 (v0.5.0)\n"
        "\n"
        "The two-parameter cubic EOS (PR, RKS, …) fix the critical "
        "compressibility $Z_c$ at a single family value (0.307 for PR, "
        "0.333 for RKS), which is wrong for almost every real fluid. "
        "**Three-parameter EOS add a third constant $c$** so each component "
        "gets its own $Z_c$, improving liquid densities and polar-fluid "
        "behavior. This milestone ports the three Pascal three-parameter "
        "models — **Schmidt-Wenzel, Patel-Teja, and Patel-Teja USB** — plus "
        "the **Chao-Seader** semi-empirical liquid-fugacity correlation "
        "(with its special hydrogen and methane coefficient sets). "
        "Ref (4): Da Silva & Báez (1989)."
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
        "## Context — the third parameter\n"
        "\n"
        "From [Chapter II §2.3](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md), "
        "a cubic EOS writes pressure as a repulsive plus an attractive term. "
        "A **two-parameter** form fixes the attractive denominator with the "
        "family constants $(k_1, k_2)$:\n"
        "\n"
        "$$P = \\frac{R T}{V - b} - \\frac{a\\,\\alpha(T_r)}{V^2 + k_1 b V + k_2 b^2}.$$\n"
        "\n"
        "A **three-parameter** form generalizes the denominator to "
        "$V^2 + u V + w'$, where $u$ and $w'$ encode a component-specific "
        "third constant $c$ (equivalently, a fitted critical compressibility "
        "$\\xi_c$). In the engine this is handled uniformly: the same cubic "
        "and fugacity algebra is reused with the dimensionless groups "
        "$U = uP/RT$ and $W = w'(P/RT)^2$ (the two-parameter case is just "
        "$U = k_1 B,\\; W = k_2 B^2$).\n"
        "\n"
        "| EOS | third parameter | reduces $Z_c$ error via |\n"
        "|---|---|---|\n"
        "| Schmidt-Wenzel | $\\beta(\\omega)$ + acentric $c=\\omega$ | $\\omega$-dependent denominator |\n"
        "| Patel-Teja | fitted $\\xi_c(\\omega)$ | $c = (1-3\\xi_c)\\,RT_c/P_c$ |\n"
        "| Patel-Teja USB | same as PT (differs only in **mixture** $c$ rule) | $\\sqrt{B}$-weighted mixing (M8) |\n"
        "\n"
        "All three are ported from `legacy/pascal/TERMOII.PAS` (Ref (4)). "
        "The Schmidt-Wenzel $m(T_r)$ is **piecewise** — its slope is "
        "discontinuous at $T_r=1$; the engine reproduces the legacy values "
        "but uses a guarded one-sided derivative so the entropy stays finite "
        "there (the original returned NaN)."
    ))

    # ---- What was built --------------------------------------------------
    cells.append(md(
        "## What was built in Milestone 7.3\n"
        "\n"
        "Through `vle._engine`:\n"
        "\n"
        "- The three-parameter variants `CubicEos.SchmidtWenzel`, "
        "`.PatelTeja`, `.PatelTejaUSB` now flow through the **same** public "
        "functions as the two-parameter zoo: `eos_alpha`, `eos_d_alpha_d_tr`, "
        "`eos_z_factor`, `eos_ln_phi_pure`, `eos_h_departure_rt`, "
        "`eos_s_departure_r`.\n"
        "- `chao_seader_ln_phi(t, p, tc, pc, omega, species)` — the "
        "Chao-Seader liquid fugacity coefficient as $\\ln\\nu$, with "
        "`ChaoSeaderSpecies.{Normal, Hydrogen, Methane}`.\n"
        "- (Rust) `mixing::c_mix` — the $c$-parameter mixture rules, ready "
        "for the mixture work in M8.\n"
        "\n"
        "Every new α has an **analytical** dα/dT_r (verified against a "
        "central-difference oracle in the test suite)."
    ))

    # ---- Worked example: setup ------------------------------------------
    cells.append(md("## Worked example\n\nImports and two reference components."))
    cells.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "import vle._engine as e\n"
        "%matplotlib inline\n"
        "\n"
        "print('vle-thermo', e.version())\n"
        "\n"
        "N_PENTANE = dict(tc=469.7, pc=3370.0, omega=0.252)   # K, kPa, –\n"
        "METHANE = dict(tc=190.564, pc=4599.0, omega=0.0115)\n"
        "\n"
        "THREE_PARAM = {\n"
        "    'Schmidt-Wenzel': e.CubicEos.SchmidtWenzel,\n"
        "    'Patel-Teja': e.CubicEos.PatelTeja,\n"
        "    'Patel-Teja USB': e.CubicEos.PatelTejaUSB,\n"
        "}"
    ))

    # ---- α(Tr) plot ------------------------------------------------------
    cells.append(md(
        "### α(T_r): the three-parameter models vs Peng-Robinson\n"
        "\n"
        "Every α passes through 1 at the critical point ($T_r = 1$) — the "
        "EOS-specific prefactor is folded into $\\Omega_a$, so the bare α "
        "keeps the standard convention."
    ))
    cells.append(code(
        "trs = np.linspace(0.5, 1.5, 80)\n"
        "w = N_PENTANE['omega']\n"
        "plt.figure(figsize=(7, 4))\n"
        "for name, eos in {**THREE_PARAM, 'Peng-Robinson': e.CubicEos.PR1976}.items():\n"
        "    plt.plot(trs, [e.eos_alpha(eos, tr, w) for tr in trs], label=name)\n"
        "plt.axhline(1.0, ls=':', c='grey'); plt.axvline(1.0, ls=':', c='grey')\n"
        "plt.xlabel(r'Reduced temperature $T_r$'); plt.ylabel(r'$\\alpha(T_r)$')\n"
        "plt.title('α(T_r): three-parameter EOS vs PR (n-pentane)')\n"
        "plt.legend(); plt.tight_layout(); plt.show()\n"
        "\n"
        "# α(T_r = 1) = 1 for every three-parameter model.\n"
        "for eos in THREE_PARAM.values():\n"
        "    assert abs(e.eos_alpha(eos, 1.0, w) - 1.0) < 1e-12"
    ))

    # ---- Z + lnphi -------------------------------------------------------
    cells.append(md(
        "### Compressibility factor and fugacity coefficient\n"
        "\n"
        "Vapor and liquid roots and $\\ln\\varphi$ for n-pentane. The "
        "three-parameter $Z_c$ differs per model, which is exactly the point "
        "— it shifts the liquid root."
    ))
    cells.append(code(
        "tc, pc, w = N_PENTANE['tc'], N_PENTANE['pc'], N_PENTANE['omega']\n"
        "T_vap, P_vap = 400.0, 500.0      # K, kPa (vapor)\n"
        "T_liq, P_liq = 300.0, 2000.0     # K, kPa (compressed liquid)\n"
        "print(f\"{'EOS':<16}{'Z_vap':>9}{'Z_liq':>9}{'ln_phi_vap':>12}\")\n"
        "for name, eos in {**THREE_PARAM, 'Peng-Robinson': e.CubicEos.PR1976}.items():\n"
        "    zv = e.eos_z_factor(eos, T_vap, P_vap, tc, pc, w, 'vapor')\n"
        "    zl = e.eos_z_factor(eos, T_liq, P_liq, tc, pc, w, 'liquid')\n"
        "    lnp = e.eos_ln_phi_pure(eos, T_vap, P_vap, tc, pc, w, 'vapor')\n"
        "    print(f'{name:<16}{zv:>9.4f}{zl:>9.4f}{lnp:>12.4f}')\n"
        "\n"
        "# Ideal-gas limit sanity check: Z -> 1, ln_phi -> 0 as P -> 0.\n"
        "z0 = e.eos_z_factor(e.CubicEos.PatelTeja, 400.0, 1e-3, tc, pc, w, 'vapor')\n"
        "assert abs(z0 - 1.0) < 1e-4"
    ))

    # ---- Chao-Seader -----------------------------------------------------
    cells.append(md(
        "### Chao-Seader liquid fugacity\n"
        "\n"
        "Chao-Seader returns the liquid fugacity coefficient $\\nu = "
        "f_L/(xP)$ from a regular-solution + reduced-property correlation, "
        "with **distinct coefficient sets** for hydrogen and methane (their "
        "quantum/light-gas behavior breaks the normal-fluid fit). We return "
        "$\\ln\\nu$ to match `eos_ln_phi_pure`."
    ))
    cells.append(code(
        "cases = [\n"
        "    ('n-pentane', N_PENTANE, e.ChaoSeaderSpecies.Normal),\n"
        "    ('methane',   METHANE,   e.ChaoSeaderSpecies.Methane),\n"
        "]\n"
        "for label, comp, species in cases:\n"
        "    t = 0.7 * comp['tc']          # a representative reduced temperature\n"
        "    ln_nu = e.chao_seader_ln_phi(t, 500.0, comp['tc'], comp['pc'], comp['omega'], species)\n"
        "    print(f'{label:<10} ({str(species).split(chr(46))[-1]:<7}) ln nu = {ln_nu:+.4f}  ->  nu = {np.exp(ln_nu):.4f}')\n"
        "\n"
        "# Using the wrong coefficient set changes the answer materially:\n"
        "t = 0.7 * METHANE['tc']\n"
        "ln_methane = e.chao_seader_ln_phi(t, 500.0, METHANE['tc'], METHANE['pc'], METHANE['omega'], e.ChaoSeaderSpecies.Methane)\n"
        "ln_as_normal = e.chao_seader_ln_phi(t, 500.0, METHANE['tc'], METHANE['pc'], METHANE['omega'], e.ChaoSeaderSpecies.Normal)\n"
        "print(f'\\nmethane: dedicated set ln nu = {ln_methane:+.4f}, normal set ln nu = {ln_as_normal:+.4f}')\n"
        "assert abs(ln_methane - ln_as_normal) > 1e-3"
    ))

    # ---- Exercise 1 ------------------------------------------------------
    cells.append(md(
        "## Exercise 1 — Z vs pressure for Patel-Teja\n"
        "\n"
        "Plot the **vapor** compressibility factor of Patel-Teja n-pentane "
        "at $T = 450$ K over $P = 10 \\dots 2000$ kPa, and confirm $Z \\to 1$ "
        "as $P \\to 0$. Fill in the `# TODO`s."
    ))
    cells.append(code(
        "tc, pc, w = N_PENTANE['tc'], N_PENTANE['pc'], N_PENTANE['omega']\n"
        "pressures = np.linspace(10.0, 2000.0, 50)\n"
        "# TODO: build a list `zs` of vapor Z values at T = 450 K for each P.\n"
        "# zs = [e.eos_z_factor(...) for p in pressures]\n"
        "# TODO: plot zs vs pressures, and assert the low-P value is ~1.\n"
    ))
    cells.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "zs = [e.eos_z_factor(e.CubicEos.PatelTeja, 450.0, p, tc, pc, w, 'vapor') for p in pressures]\n"
        "plt.figure(figsize=(7, 4))\n"
        "plt.plot(pressures, zs)\n"
        "plt.xlabel('Pressure (kPa)'); plt.ylabel('Z (vapor)')\n"
        "plt.title('Patel-Teja vapor Z — n-pentane at 450 K'); plt.tight_layout(); plt.show()\n"
        "z_low = e.eos_z_factor(e.CubicEos.PatelTeja, 450.0, 1e-3, tc, pc, w, 'vapor')\n"
        "assert abs(z_low - 1.0) < 1e-4\n"
        "```\n"
        "</details>"
    ))

    # ---- Exercise 2 ------------------------------------------------------
    cells.append(md(
        "## Exercise 2 — Schmidt-Wenzel entropy across $T_r = 1$\n"
        "\n"
        "The Schmidt-Wenzel α has a slope kink at $T_r = 1$. Evaluate the "
        "**entropy departure** $S^R/R$ for Schmidt-Wenzel n-pentane (vapor) "
        "at $T_r = 0.95, 1.00, 1.05$ (i.e. $T = 0.95\\,T_c$, etc.) at "
        "$P = 500$ kPa and confirm every value is **finite** (the legacy "
        "returned NaN at $T_r = 1$)."
    ))
    cells.append(code(
        "import math\n"
        "tc, pc, w = N_PENTANE['tc'], N_PENTANE['pc'], N_PENTANE['omega']\n"
        "# TODO: for tr in (0.95, 1.00, 1.05), compute T = tr*tc and\n"
        "#       s = e.eos_s_departure_r(e.CubicEos.SchmidtWenzel, T, 500.0, tc, pc, w, 'vapor')\n"
        "#       then assert math.isfinite(s).\n"
    ))
    cells.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "for tr in (0.95, 1.00, 1.05):\n"
        "    s = e.eos_s_departure_r(e.CubicEos.SchmidtWenzel, tr * tc, 500.0, tc, pc, w, 'vapor')\n"
        "    print(f'Tr={tr:.2f}  S^R/R = {s:+.5f}')\n"
        "    assert math.isfinite(s)\n"
        "```\n"
        "</details>"
    ))

    # ---- References ------------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- **(4)** Da Silva, F. A.; Báez, L. *Desarrollo de un Paquete "
        "Computacional…*, Universidad Simón Bolívar, 1989 — origin of the "
        "Schmidt-Wenzel, Patel-Teja, and Chao-Seader implementations "
        "(`legacy/pascal/TERMOII.PAS`).\n"
        "- **(5)** Abbott, M. M. — generalized cubic-EOS form.\n"
        "- [Chapter II §2.3](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md) — cubic EOS theory.\n"
        "- [`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/docs/plans/MODERNIZATION_PLAN.md) — Phase 7 notes and the unified (U, W) three-parameter formulation.\n"
        "- Two-parameter α zoo: [`02b_alpha_zoo.ipynb`](02b_alpha_zoo.ipynb). "
        "Advanced saturation models (M7.4): "
        "[`02d_advanced_saturation.ipynb`](02d_advanced_saturation.ipynb)."
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
