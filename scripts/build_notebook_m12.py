#!/usr/bin/env python3
"""Generate ``notebooks/11_derivatives_and_database.ipynb`` — the Milestone 12
notebook (the downstream derivative & database release, vle-thermo 0.9.0).

Covers the four gaps M12 closed for downstream staged-separation work:
the expanded 24-compound database with ideal-gas Cp°, the exact temperature
and pressure derivatives of fugacity and K-values (dual-number AD), the
real-mixture heat capacity, and partial molar enthalpy. Structure follows
CLAUDE.md *Notebook Conventions*; generated deterministically and executed
top-to-bottom in a fresh kernel before saving.

Ref (4): Da Silva & Báez (1989). Derivative identities: Michelsen & Mollerup
(26); dual-number AD: Rehner & Bauer (27); property data: Poling, Prausnitz &
O'Connell 5th ed. (30).
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "11_derivatives_and_database.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Derivatives & the Component Database — Milestone 12\n"
        "\n"
        "Equilibrium is only half the story. A downstream **staged-separation** "
        "library (distillation columns, absorbers) needs more than *where* "
        "phases split — it needs how fast the split **moves** when temperature "
        "or pressure change, and how much **heat** each stage carries. This "
        "milestone (vle-thermo 0.9.0) adds the machinery for that:\n"
        "\n"
        "1. an expanded **24-compound database** carrying ideal-gas $C_p^\\circ$ "
        "coefficients, now readable from Rust *and* Python;\n"
        "2. **exact** temperature/pressure derivatives of fugacity and "
        "K-values — $\\partial\\ln\\hat\\varphi_i/\\partial T$, "
        "$\\partial\\ln K_i/\\partial T$, … — by dual-number automatic "
        "differentiation, never finite differences;\n"
        "3. **real-mixture heat capacity** $C_p$ and **partial molar "
        "enthalpy** $\\bar H_i$.\n"
        "\n"
        "Derivative identities follow Michelsen & Mollerup (26); the dual-number "
        "AD follows Rehner & Bauer (27); property data is Poling, Prausnitz & "
        "O'Connell 5th ed. (30)."
    ))

    # ---- Optional upgrade cell (CLAUDE.md §2b) --------------------------
    cells.append(md(
        "## Setup (optional)\n"
        "\n"
        "The cell below is **commented out by default**. Uncomment it if you "
        "want to use the latest `vle-thermo` released on PyPI instead of "
        "whatever version is currently installed in your kernel."
    ))
    cells.append(code(
        "# Optional: pull the latest vle-thermo from PyPI.\n"
        "# Uncomment if you want the newest released version instead of\n"
        "# whatever is currently in your kernel.\n"
        "# %pip install --upgrade vle-thermo"
    ))

    # ---- Research-paper context -----------------------------------------
    cells.append(md(
        "## Context from the research paper\n"
        "\n"
        "The K-value is the ratio the whole VLE calculation turns on. From "
        "[Chapter II §2.3](../docs/en/research-paper/chapter-2-vle-theory.md) "
        "the two-model framework writes it two ways:\n"
        "\n"
        "- **φ-φ** (equation of state on both phases): "
        "$K_i = \\hat\\varphi_i^{\\,L} / \\hat\\varphi_i^{\\,V}$.\n"
        "- **γ-φ** (activity-model liquid): modified Raoult, "
        "$K_i = \\dfrac{\\gamma_i\\, P_i^{\\text{sat}}\\, \\varphi_i^{\\text{sat}}\\, "
        "\\text{POY}_i}{\\hat\\varphi_i^{\\,V}\\, P}$.\n"
        "\n"
        "Both are built from the fugacity coefficient $\\hat\\varphi_i$. So the "
        "*derivatives* of $K_i$ decompose the same way — and the exact residual "
        "enthalpy follows from the Gibbs–Helmholtz relation\n"
        "\n"
        "$$\\sum_i x_i\\,\\frac{\\partial \\ln\\hat\\varphi_i}{\\partial T} "
        "= -\\frac{H^R}{RT^2},$$\n"
        "\n"
        "which this notebook uses as a correctness check. The heat-capacity and "
        "enthalpy definitions are the departure-function framework of the same "
        "chapter."
    ))

    # ---- What was built -------------------------------------------------
    cells.append(md(
        "## What was built in this milestone\n"
        "\n"
        "In the Rust engine (all exposed through `vle.System` and "
        "`vle.components`):\n"
        "\n"
        "- **`vle.components`** — 24 compounds with `cp_coeffs` "
        "($C_p^\\circ/R$ polynomial), plus a byte-identical Rust-side copy "
        "(`vle_thermo::db`).\n"
        "- **`System.d_ln_phi_d_t` / `d_ln_phi_d_p`** — exact "
        "$\\partial\\ln\\hat\\varphi_i/\\partial T$ [1/K], "
        "$\\partial\\ln\\hat\\varphi_i/\\partial P$ [1/kPa].\n"
        "- **`System.k_values_with_derivs`** — $(K_i,\\ "
        "\\partial\\ln K_i/\\partial T,\\ \\partial\\ln K_i/\\partial P)$.\n"
        "- **`System.phase_cp`** — real-mixture $C_p$ [kJ/(kmol·K)].\n"
        "- **`System.partial_molar_enthalpy`** — $\\bar H_i$ [kJ/kmol], with "
        "$\\sum_i x_i \\bar H_i = H$.\n"
        "\n"
        "The derivatives are computed by evaluating the *same* fugacity code "
        "with dual numbers, so they are exact to machine precision (27)."
    ))

    cells.append(code(
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "%matplotlib inline\n"
        "\n"
        "from vle import System, components\n"
        "from vle.units import Q_\n"
        "\n"
        "print('bundled compounds:', len(components.available()))"
    ))

    # ---- Worked example 1: DB tour --------------------------------------
    cells.append(md(
        "## 1. A tour of the 24-compound database\n"
        "\n"
        "The database grew from 15 to 24 compounds and now ships ideal-gas "
        "heat-capacity coefficients. The `cp_coeffs` are the **dimensionless** "
        "$C_p^\\circ/R = \\sum_k a_k T^k$ polynomial (T in K), matching the "
        "engine convention exactly. Below we read toluene — the "
        "benzene–toluene pair is *the* teaching binary for distillation."
    ))
    cells.append(code(
        "tol = components.get('toluene')\n"
        "print(f'toluene  Tc={tol.tc} K  Pc={tol.pc} kPa  omega={tol.omega}')\n"
        "print('cp_coeffs (Cp0/R poly):', tol.cp_coeffs)\n"
        "\n"
        "# Evaluate Cp0(298.15 K) from the polynomial and compare to literature\n"
        "R = 8.31451  # kJ/(kmol.K)\n"
        "T0 = 298.15\n"
        "cp0 = R * sum(a * T0**k for k, a in enumerate(tol.cp_coeffs))\n"
        "# kJ/(kmol.K) is numerically identical to J/(mol.K).\n"
        "print(f'Cp0(298.15 K) = {cp0:.2f} kJ/(kmol.K)  (~103.7 J/(mol.K) lit.)')\n"
        "assert abs(cp0 - 103.7) / 103.7 < 0.02  # within 2%"
    ))

    # ---- Worked example 2: K(T) and its exact tangent -------------------
    cells.append(md(
        "## 2. K(T) and its exact tangent — the money shot\n"
        "\n"
        "For a benzene–toluene liquid we plot $\\ln K_{\\text{benzene}}$ over a "
        "temperature band, then draw the **tangent** predicted by the exact "
        "derivative $\\partial\\ln K/\\partial T$ at the mid-point. If the "
        "derivative is right, the tangent kisses the curve."
    ))
    cells.append(code(
        "sys = System(['benzene', 'toluene'], eos='PR')\n"
        "p = 101.325  # kPa\n"
        "x = [0.5, 0.5]\n"
        "y = [0.7, 0.3]  # trial vapor\n"
        "\n"
        "Ts = np.linspace(360.0, 380.0, 41)\n"
        "lnK0 = np.array([np.log(sys.k_values(T, p, x, y)[0]) for T in Ts])\n"
        "\n"
        "# Exact derivative at the mid-point.\n"
        "Tmid = 370.0\n"
        "k, dlnk_dt, dlnk_dp = sys.k_values_with_derivs(Tmid, p, x, y)\n"
        "lnKmid = np.log(k[0])\n"
        "tangent = lnKmid + dlnk_dt[0] * (Ts - Tmid)\n"
        "\n"
        "fig, ax = plt.subplots(figsize=(6, 4))\n"
        "ax.plot(Ts, lnK0, label='ln K(benzene)', lw=2)\n"
        "ax.plot(Ts, tangent, '--', label='exact tangent at 370 K')\n"
        "ax.plot([Tmid], [lnKmid], 'o', color='crimson')\n"
        "ax.set_xlabel('T [K]'); ax.set_ylabel('ln K'); ax.legend()\n"
        "ax.set_title('K(T) with the exact d(lnK)/dT tangent')\n"
        "plt.show()\n"
        "\n"
        "# Pin: the exact derivative must match a central difference.\n"
        "h = 0.5\n"
        "fd = (np.log(sys.k_values(Tmid + h, p, x, y)[0])\n"
        "      - np.log(sys.k_values(Tmid - h, p, x, y)[0])) / (2 * h)\n"
        "print(f'exact d(lnK)/dT = {dlnk_dt[0]:.6e}   FD = {fd:.6e}')\n"
        "assert abs(dlnk_dt[0] - fd) < 1e-5 * abs(dlnk_dt[0])"
    ))

    # ---- Worked example 3: real-mixture Cp ------------------------------
    cells.append(md(
        "## 3. Real-mixture heat capacity vs temperature\n"
        "\n"
        "`phase_cp` returns $C_p = \\sum_i x_i C_{p,i}^\\circ + C_p^R$: the "
        "ideal-gas mixture value plus the residual from the EOS (computed with "
        "a *second-order* dual). We compare the vapor's real $C_p$ against the "
        "ideal-gas part over a temperature sweep at a moderate pressure."
    ))
    cells.append(code(
        "sysv = System(['benzene', 'toluene'], eos='PR')\n"
        "xv = [0.5, 0.5]\n"
        "P = 500.0  # kPa\n"
        "Ts = np.linspace(360.0, 460.0, 41)\n"
        "cp_real = np.array([sysv.phase_cp(T, P, xv, 'vapor') for T in Ts])\n"
        "\n"
        "# Ideal-gas part from the bundled cp_coeffs.\n"
        "def cp_ideal(T):\n"
        "    tot = 0.0\n"
        "    for name, xi in zip(['benzene', 'toluene'], xv):\n"
        "        c = components.get(name).cp_coeffs\n"
        "        tot += xi * R * sum(a * T**k for k, a in enumerate(c))\n"
        "    return tot\n"
        "cp_id = np.array([cp_ideal(T) for T in Ts])\n"
        "\n"
        "fig, ax = plt.subplots(figsize=(6, 4))\n"
        "ax.plot(Ts, cp_real, label='real Cp (PR, 500 kPa)', lw=2)\n"
        "ax.plot(Ts, cp_id, '--', label='ideal-gas Cp')\n"
        "ax.set_xlabel('T [K]'); ax.set_ylabel('Cp [kJ/(kmol.K)]'); ax.legend()\n"
        "ax.set_title('Real vs ideal-gas mixture heat capacity')\n"
        "plt.show()\n"
        "\n"
        "# The residual Cp^R = real - ideal is positive here (attractive EOS).\n"
        "print('Cp^R at 400 K:', sysv.phase_cp(400.0, P, xv, 'vapor') - cp_ideal(400.0))"
    ))

    # ---- Worked example 4: partial molar enthalpy + Euler ---------------
    cells.append(md(
        "## 4. Partial molar enthalpy and the Euler check\n"
        "\n"
        "The partial molar enthalpies must sum back to the total: "
        "$\\sum_i x_i \\bar H_i = H$. This is Euler's theorem for the "
        "homogeneous-degree-1 enthalpy, and it is a hard invariant — we assert "
        "it below and draw the per-component bars."
    ))
    cells.append(code(
        "sysh = System(['benzene', 'toluene'], eos='PR')\n"
        "T, P = 380.0, 300.0\n"
        "xh = [0.4, 0.6]\n"
        "hbar = sysh.partial_molar_enthalpy(T, P, xh, 'vapor')\n"
        "H_total, _S = sysh.enthalpy_entropy(T, P, xh, 'vapor')\n"
        "euler = sum(xi * hi for xi, hi in zip(xh, hbar))\n"
        "print(f'sum x_i Hbar_i = {euler:.3f}   H = {H_total:.3f} kJ/kmol')\n"
        "assert abs(euler - H_total) < 1e-6 * abs(H_total)\n"
        "\n"
        "fig, ax = plt.subplots(figsize=(5, 4))\n"
        "ax.bar(['benzene', 'toluene'], hbar, color=['#4C72B0', '#DD8452'])\n"
        "ax.axhline(H_total, ls='--', color='gray', label='total H')\n"
        "ax.set_ylabel('partial molar enthalpy [kJ/kmol]'); ax.legend()\n"
        "ax.set_title('Partial molar enthalpies (Euler sum = H)')\n"
        "plt.show()"
    ))

    # ---- Exercises ------------------------------------------------------
    cells.append(md(
        "## Exercises\n"
        "\n"
        "### Exercise 1 — the pressure derivative identity\n"
        "\n"
        "The composition-summed pressure derivative obeys an exact identity:\n"
        "\n"
        "$$\\sum_i x_i \\frac{\\partial \\ln\\hat\\varphi_i}{\\partial P} "
        "= \\frac{Z - 1}{P}.$$\n"
        "\n"
        "Verify it for the benzene–toluene vapor at (400 K, 800 kPa, "
        "x = [0.5, 0.5]) using `d_ln_phi_d_p` and `z_factor`."
    ))
    cells.append(code(
        "# TODO: build the System, evaluate d_ln_phi_d_p and z_factor,\n"
        "# and check the identity holds to ~1e-9.\n"
        "sys_ex = System(['benzene', 'toluene'], eos='PR')\n"
        "# dp = sys_ex.d_ln_phi_d_p(...)\n"
        "# z = sys_ex.z_factor(...)\n"
        "# lhs = ...; rhs = ...\n"
    ))
    cells.append(md(
        "<details>\n<summary>Solution</summary>\n\n"
        "```python\n"
        "sys_ex = System(['benzene', 'toluene'], eos='PR')\n"
        "T, P, x = 400.0, 800.0, [0.5, 0.5]\n"
        "dp = sys_ex.d_ln_phi_d_p(T, P, x, 'vapor')\n"
        "z = sys_ex.z_factor(T, P, x, 'vapor')\n"
        "lhs = sum(xi * di for xi, di in zip(x, dp))\n"
        "rhs = (z - 1.0) / P\n"
        "print(lhs, rhs)\n"
        "assert abs(lhs - rhs) < 1e-9 * max(abs(rhs), 1e-9)\n"
        "```\n"
        "</details>"
    ))
    cells.append(md(
        "### Exercise 2 — condensation lowers the liquid enthalpy\n"
        "\n"
        "Build a **γ-φ** methanol/water System (`liquid_model='activity'`, "
        "`activity='VanLaar'`, `vapor_model='ideal'`, "
        "`aij=[[0.0, 0.85], [0.52, 0.0]]`). Show that the liquid enthalpy is "
        "**below** the vapor enthalpy at (340 K, 100 kPa, x = [0.4, 0.6]) — the "
        "latent heat of condensation. (Before M12.4 this call errored, because "
        "the liquid has no cubic EOS.)"
    ))
    cells.append(code(
        "# TODO: build the gamma-phi System and compare liquid vs vapor enthalpy.\n"
        "# sys_gp = System(['methanol', 'water'], liquid_model='activity', ...)\n"
    ))
    cells.append(md(
        "<details>\n<summary>Solution</summary>\n\n"
        "```python\n"
        "sys_gp = System(['methanol', 'water'], liquid_model='activity',\n"
        "                activity='VanLaar', vapor_model='ideal',\n"
        "                aij=[[0.0, 0.85], [0.52, 0.0]])\n"
        "T, P, x = 340.0, 100.0, [0.4, 0.6]\n"
        "h_liq, _ = sys_gp.enthalpy_entropy(T, P, x, 'liquid')\n"
        "h_vap, _ = sys_gp.enthalpy_entropy(T, P, x, 'vapor')\n"
        "print('H_liquid =', h_liq, '  H_vapor =', h_vap)\n"
        "assert h_liq < h_vap\n"
        "```\n"
        "</details>"
    ))

    # ---- References -----------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- Research paper: "
        "[Chapter II — VLE theory](../docs/en/research-paper/chapter-2-vle-theory.md) "
        "(K-values, departure functions).\n"
        "- Parameter reference: "
        "[parameter_reference.md](../docs/en/parameters/parameter_reference.md) "
        "(the $C_p^\\circ/R$ polynomials).\n"
        "- Plan of record: `DERIVATIVE_RELEASE_PLAN.md` (Milestone 12 design, "
        "the T/P-generic core and dual-number strategy).\n"
        "- (26) Michelsen & Mollerup — derivative identities; (27) Rehner & "
        "Bauer — dual-number AD; (30) Poling, Prausnitz & O'Connell — property "
        "data; (4) Da Silva & Báez (1989) — condensation-enthalpy path.\n"
        "\n"
        "**Why this milestone exists:** a downstream staged-separation library "
        "consumes exactly these derivatives (stage-to-stage K sensitivity) and "
        "energy properties (reboiler/condenser duties). Shipping them here — "
        "exact and tested — means that library never has to finite-difference "
        "the engine."
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
