"""Build notebooks/02_pure_component.ipynb.

The Milestone 7.1 notebook tours the deployable pure-component layer:
the four core cubic EOS (PR, RKS, RK, VdW) shipping in v0.3.0, their
α(Tr) functions and analytical derivatives, Z-factor + fugacity +
departure properties, the Antoine saturation pressure, and the
truncated virial equation.

It exists for two audiences:

1. **Hub students** opening the file on the hosted JupyterLab — the
   first runnable example through `vle._engine.eos_*` they meet.
2. **Reviewers / future maintainers** — a place to land when a Chapter II
   §2.3 derivation in the research paper needs an answer in code form.

The variants deferred to M7.2 / M7.3 / M7.4 each have their own
placeholder notebook (`02b`, `02c`, `02d`) — see
``scripts/build_notebook_m07_placeholders.py``.

Follows CLAUDE.md *Notebook Conventions* (title + motivation → hub
sandbox notice → optional pip install → research-paper context →
what was built → worked examples → exercises → references).

Run:

    $ /Users/migueljackson/miniconda3/envs/vle/bin/python \\
        scripts/build_notebook_m07.py

The script executes the notebook end-to-end via NotebookClient before
saving, so an engine regression surfaces here rather than on the hub.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "02_pure_component.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Pure-Component Cubic EOS — Milestone 7 (v0.3.0)\n"
        "\n"
        "Every multicomponent flash, every bubble-point search, every "
        "distillation column simulation eventually decomposes into "
        "**pure-component questions**: at this T and P, what is the "
        "compressibility factor of n-pentane? What is its fugacity "
        "coefficient? How does its saturation pressure scale with "
        "temperature? Milestone 7.1 ships the deployable core that "
        "answers those questions — the four most-used cubic equations of "
        "state (Peng-Robinson, RKS, original RK, van der Waals), the "
        "Antoine vapor-pressure correlation, and the truncated virial "
        "equation."
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
        "## Context — Chapter II §2.3 (cubic EOS) in code form\n"
        "\n"
        "From [Chapter II §2.3](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md): "
        "every cubic equation of state in this library shares the **Abbott "
        "form**\n"
        "\n"
        "$$P = \\frac{R T}{V - b} - \\frac{a \\, \\alpha(T)}{V^2 + k_1 b V + k_2 b^2}$$\n"
        "\n"
        "where `a = Ω_a · R² T_c² / P_c`, `b = Ω_b · R T_c / P_c`, "
        "and `α(T)` is the variant-specific temperature dependence. The "
        "four core variants in this notebook differ only in those "
        "`(k_1, k_2, Ω_a, Ω_b, α)` choices:\n"
        "\n"
        "| EOS | `k_1` | `k_2` | `Ω_a` | `Ω_b` | α(T_r) |\n"
        "|---|---|---|---|---|---|\n"
        "| van der Waals (1870) | 0 | 0 | 27/64 | 1/8 | 1 |\n"
        "| Redlich-Kwong (1949) | 1 | 0 | 0.427480 | 0.086640 | T_r^(-1/2) |\n"
        "| Soave-RK (1972) | 1 | 0 | 0.427480 | 0.086640 | [1 + m(1 − √T_r)]², m = 0.48 + 1.574ω − 0.176ω² |\n"
        "| Peng-Robinson (1976) | 2 | -1 | 0.457236 | 0.077796 | [1 + κ(1 − √T_r)]², κ = 0.37464 + 1.54226ω − 0.26992ω² |\n"
        "\n"
        "Those four sets of constants live verbatim in "
        "[`engine/src/eos.rs`](https://github.com/miguelju/vle/blob/main/engine/src/eos.rs) "
        "(family_constants table from `legacy/vb6/McommonFunctions.bas:273`). "
        "The other 12 two-parameter α variants from VB6 (PRSV, polar "
        "Mathias-Naumann extensions, etc.) shipped in **M7.2** — see "
        "[`02b_alpha_zoo.ipynb`](02b_alpha_zoo.ipynb). The three OL-family "
        "variants (M7.4) and the three-parameter Pascal EOS "
        "(Schmidt-Wenzel, Patel-Teja; M7.3) are still to come — see "
        "[`02c_three_param_eos.ipynb`](02c_three_param_eos.ipynb)."
    ))

    # ---- What was built --------------------------------------------------
    cells.append(md(
        "## What was built in Milestone 7.1\n"
        "\n"
        "Ten public PyO3-bound functions, every one with an analytical "
        "derivative where one exists:\n"
        "\n"
        "Cubic EOS (`engine/src/eos.rs`):\n"
        "- `eos_family_constants(eos)` — returns `(k_1, k_2, Ω_a, Ω_b)`\n"
        "- `eos_alpha(eos, T_r, ω)` and `eos_d_alpha_d_tr(eos, T_r, ω)`\n"
        "- `eos_z_factor(eos, T, P, T_c, P_c, ω, phase)` — solves the cubic "
        "in Z and picks the liquid or vapor root\n"
        "- `eos_ln_phi_pure(...)` — pure-component fugacity coefficient ln φ\n"
        "- `eos_h_departure_rt(...)` — H^R/(RT), dimensionless\n"
        "- `eos_s_departure_r(...)` — S^R/R, dimensionless\n"
        "\n"
        "Saturation (`engine/src/saturation.rs`):\n"
        "- `antoine_psat(T, P_c, coeffs)` — Da Silva form ln(P/Pc) = a₁ − a₂/(a₃+T)\n"
        "- `antoine_d_psat_dt(...)` — analytical dP_sat/dT\n"
        "\n"
        "Virial (`engine/src/virial.rs`):\n"
        "- `virial_pitzer_b0(T_r)`, `virial_pitzer_b1(T_r)` — Pitzer correlation\n"
        "- `virial_b_pure(...)`, `virial_d_b_d_t_pure(...)`\n"
        "- `virial_z(...)`, `virial_ln_phi(...)`\n"
        "- `virial_h_dep_rt(...)`, `virial_s_dep_r(...)`\n"
        "- `virial_b_mix_py(...)`, `virial_ln_phi_mix(...)` — mixture (M7.6)"
    ))

    # ---- Setup cell -----------------------------------------------------
    cells.append(code(
        "import math\n"
        "\n"
        "import matplotlib.pyplot as plt\n"
        "import numpy as np\n"
        "\n"
        "from vle._engine import (\n"
        "    CubicEos,\n"
        "    antoine_d_psat_dt,\n"
        "    antoine_psat,\n"
        "    eos_alpha,\n"
        "    eos_d_alpha_d_tr,\n"
        "    eos_family_constants,\n"
        "    eos_h_departure_rt,\n"
        "    eos_ln_phi_pure,\n"
        "    eos_z_factor,\n"
        "    version,\n"
        "    virial_b_mix_py,\n"
        "    virial_b_pure,\n"
        "    virial_ln_phi,\n"
        "    virial_ln_phi_mix,\n"
        "    virial_pitzer_b0,\n"
        "    virial_pitzer_b1,\n"
        "    virial_z,\n"
        ")\n"
        "\n"
        "print(f'vle._engine version: {version()}')"
    ))

    # ── Component data ────────────────────────────────────────────────────
    cells.append(md(
        "### Component data\n"
        "\n"
        "Two reference components used throughout — methane (a small, "
        "near-spherical molecule with a tiny acentric factor) and "
        "n-pentane (an everyday hydrocarbon where ω = 0.252 actually "
        "matters). Numbers from NIST."
    ))
    cells.append(code(
        "METHANE = dict(name='methane', Tc=190.564, Pc=4599.0, omega=0.0115)\n"
        "N_PENTANE = dict(name='n-pentane', Tc=469.7, Pc=3370.0, omega=0.252)\n"
        "\n"
        "for c in (METHANE, N_PENTANE):\n"
        "    print(f\"{c['name']:>10s}: Tc={c['Tc']:.2f} K, Pc={c['Pc']:.0f} kPa, ω={c['omega']:.4f}\")"
    ))

    # ── Worked example 1: α(Tr) comparison ───────────────────────────────
    cells.append(md(
        "## 1. α(T_r) across the four core EOS for n-pentane\n"
        "\n"
        "Plot α as a function of reduced temperature for each variant. "
        "All four collapse to α=1 at the critical point (T_r=1); below "
        "T_c the curves diverge — RK falls off as T_r^(−1/2), while PR "
        "and RKS have the cleanest matched behavior near T_r=1."
    ))
    cells.append(code(
        "trs = np.linspace(0.4, 2.0, 200)\n"
        "fig, ax = plt.subplots(figsize=(7, 4.5))\n"
        "\n"
        "for eos, label in [\n"
        "    (CubicEos.VdW1870,  'VdW (1870) — α=1'),\n"
        "    (CubicEos.RK1949,   'RK (1949) — α=T_r^(-1/2)'),\n"
        "    (CubicEos.RKS1972,  'RKS (1972) — Soave'),\n"
        "    (CubicEos.PR1976,   'PR (1976) — Peng-Robinson'),\n"
        "]:\n"
        "    alphas = [eos_alpha(eos, float(tr), N_PENTANE['omega']) for tr in trs]\n"
        "    ax.plot(trs, alphas, label=label)\n"
        "\n"
        "ax.axvline(1.0, color='k', alpha=0.3, linestyle='--')\n"
        "ax.axhline(1.0, color='k', alpha=0.3, linestyle='--')\n"
        "ax.set_xlabel('Reduced temperature  T_r = T / T_c')\n"
        "ax.set_ylabel('α(T_r)')\n"
        "ax.set_title('α(T_r) for the four M7.1 core EOS — n-pentane (ω=0.252)')\n"
        "ax.legend()\n"
        "ax.grid(alpha=0.3)\n"
        "plt.show()"
    ))

    # ── Worked example 2: Z-factor isotherms ─────────────────────────────
    cells.append(md(
        "## 2. PR Z-factor isotherms for methane\n"
        "\n"
        "At supercritical T (300 K, T_r ≈ 1.57) the cubic in Z has one "
        "real root: Z is single-valued from low to high P. At "
        "near-critical T (180 K, T_r ≈ 0.94) the cubic has three real "
        "roots in a P window — the liquid Z (lowest), an unstable middle "
        "root, and the vapor Z. The isotherm splits at the saturation "
        "envelope."
    ))
    cells.append(code(
        "ps = np.linspace(100.0, 8000.0, 80)\n"
        "fig, ax = plt.subplots(figsize=(7, 4.5))\n"
        "\n"
        "for T, color in [(180.0, 'tab:blue'), (300.0, 'tab:red')]:\n"
        "    z_vap, z_liq = [], []\n"
        "    for p in ps:\n"
        "        try:\n"
        "            z_vap.append((p, eos_z_factor(CubicEos.PR1976, T, float(p),\n"
        "                                          METHANE['Tc'], METHANE['Pc'], METHANE['omega'], 'vapor')))\n"
        "        except RuntimeError:\n"
        "            pass\n"
        "        try:\n"
        "            z_liq.append((p, eos_z_factor(CubicEos.PR1976, T, float(p),\n"
        "                                          METHANE['Tc'], METHANE['Pc'], METHANE['omega'], 'liquid')))\n"
        "        except RuntimeError:\n"
        "            pass\n"
        "    if z_vap:\n"
        "        xs, ys = zip(*z_vap)\n"
        "        ax.plot(xs, ys, color=color, label=f'T={T} K (vapor)')\n"
        "    if z_liq:\n"
        "        xs, ys = zip(*z_liq)\n"
        "        # Only plot liquid branch where it differs from vapor (sub-critical).\n"
        "        xs, ys = list(xs), list(ys)\n"
        "        if max(ys) < 0.3:\n"
        "            ax.plot(xs, ys, '--', color=color, label=f'T={T} K (liquid)')\n"
        "\n"
        "ax.set_xlabel('Pressure  P  (kPa)')\n"
        "ax.set_ylabel('Compressibility factor  Z')\n"
        "ax.set_title('PR Z-factor isotherms for methane')\n"
        "ax.legend()\n"
        "ax.grid(alpha=0.3)\n"
        "plt.show()"
    ))

    # ── Worked example 3: Antoine + d/dT ─────────────────────────────────
    cells.append(md(
        "## 3. Antoine vapor pressure and its derivative\n"
        "\n"
        "The Antoine form in this library uses the Da Silva (1989) "
        "convention `ln(P_sat / P_c) = a_1 − a_2 / (a_3 + T)`, so the "
        "coefficients are tabulated against `P_c` not against 1 atm. "
        "The analytical `dP_sat/dT = P_sat · a_2 / (a_3 + T)²` matches "
        "central differences to floating-point precision."
    ))
    cells.append(code(
        "# Plausible coefficients for an n-pentane-like compound (illustration).\n"
        "Pc = N_PENTANE['Pc']\n"
        "coeffs = [9.0, 2900.0, -45.0]\n"
        "\n"
        "Ts = np.linspace(280.0, 460.0, 80)\n"
        "psats = [antoine_psat(float(T), Pc, coeffs) for T in Ts]\n"
        "\n"
        "fig, ax1 = plt.subplots(figsize=(7, 4.5))\n"
        "ax1.plot(Ts, psats, 'tab:blue', label='P_sat')\n"
        "ax1.set_xlabel('Temperature  T  (K)')\n"
        "ax1.set_ylabel('P_sat  (kPa)', color='tab:blue')\n"
        "ax1.tick_params(axis='y', labelcolor='tab:blue')\n"
        "ax1.set_yscale('log')\n"
        "\n"
        "ax2 = ax1.twinx()\n"
        "d_an = [antoine_d_psat_dt(float(T), Pc, coeffs) for T in Ts]\n"
        "ax2.plot(Ts, d_an, 'tab:red', label='dP_sat/dT (analytical)')\n"
        "ax2.set_ylabel('dP_sat/dT  (kPa/K)', color='tab:red')\n"
        "ax2.tick_params(axis='y', labelcolor='tab:red')\n"
        "\n"
        "plt.title('Antoine P_sat and analytical dP_sat/dT — illustrative coefficients')\n"
        "plt.show()"
    ))

    # ── Worked example 4: Virial ─────────────────────────────────────────
    cells.append(md(
        "## 4. Truncated virial — B(T) and the ideal-gas crossover\n"
        "\n"
        "Pitzer's correlation gives the second virial coefficient as "
        "`B · P_c / (R · T_c) = B⁰(T_r) + ω · B¹(T_r)` with\n"
        "\n"
        "$$B^0 = 0.083 - 0.422 / T_r^{1.6}, \\quad B^1 = 0.139 - 0.172 / T_r^{4.2}$$\n"
        "\n"
        "B(T) is negative at low T (attractive forces dominate, Z < 1) "
        "and crosses zero around T_r ≈ 2.7 for most fluids — the Boyle "
        "temperature, where the truncated virial reduces exactly to "
        "the ideal-gas law."
    ))
    cells.append(code(
        "Ts = np.linspace(100.0, 1200.0, 120)\n"
        "fig, ax = plt.subplots(figsize=(7, 4.5))\n"
        "\n"
        "for c, color in [(METHANE, 'tab:blue'), (N_PENTANE, 'tab:red')]:\n"
        "    Bs = [virial_b_pure(c['Tc'], c['Pc'], c['omega'], float(T)) for T in Ts]\n"
        "    ax.plot(Ts, Bs, color=color, label=c['name'])\n"
        "\n"
        "ax.axhline(0.0, color='k', alpha=0.3, linestyle='--')\n"
        "ax.set_xlabel('Temperature  T  (K)')\n"
        "ax.set_ylabel('Second virial coefficient  B(T)  (cm³/mol)')\n"
        "ax.set_title('Pitzer B(T) — Boyle temperature is where the curve crosses 0')\n"
        "ax.legend()\n"
        "ax.grid(alpha=0.3)\n"
        "plt.show()"
    ))

    # ── Worked example 5: Mixture virial ─────────────────────────────────
    cells.append(md(
        "## 5. Mixture virial — methane / n-pentane partial fugacity\n"
        "\n"
        "For a 60% methane + 40% n-pentane mixture at 350 K, 1000 kPa, "
        "the truncated virial gives a partial fugacity coefficient for "
        "each component via\n"
        "\n"
        "$$\\ln \\hat\\varphi_i = \\frac{P}{RT}\\left(2\\sum_j x_j B_{ij} - B_{\\text{mix}}\\right)$$\n"
        "\n"
        "with Lewis-Randall quadratic mixing for B(T, x). The methane "
        "partial ln(φ) ends up slightly *positive* (the small molecule "
        "is squeezed by the long-chain neighbor); the pentane partial "
        "ln(φ) is more negative than its pure value."
    ))
    cells.append(code(
        "tcs    = [METHANE['Tc'],    N_PENTANE['Tc']]\n"
        "pcs    = [METHANE['Pc'],    N_PENTANE['Pc']]\n"
        "omegas = [METHANE['omega'], N_PENTANE['omega']]\n"
        "xs     = [0.6, 0.4]\n"
        "T, P   = 350.0, 1000.0\n"
        "\n"
        "b_mix = virial_b_mix_py(tcs, pcs, omegas, xs, T)\n"
        "ln_phi_i = virial_ln_phi_mix(tcs, pcs, omegas, xs, T, P)\n"
        "ln_phi_pure_methane = virial_ln_phi(*[METHANE[k] for k in ('Tc', 'Pc', 'omega')], T, P)\n"
        "ln_phi_pure_pentane = virial_ln_phi(*[N_PENTANE[k] for k in ('Tc', 'Pc', 'omega')], T, P)\n"
        "\n"
        "print(f'B_mix = {b_mix:.2f} cm³/mol')\n"
        "print(f'ln(φ_methane,   pure)    = {ln_phi_pure_methane:+.4f}')\n"
        "print(f'ln(φ_methane,   mixture) = {ln_phi_i[0]:+.4f}')\n"
        "print(f'ln(φ_pentane,   pure)    = {ln_phi_pure_pentane:+.4f}')\n"
        "print(f'ln(φ_pentane,   mixture) = {ln_phi_i[1]:+.4f}')"
    ))

    # ── Exercises ────────────────────────────────────────────────────────
    cells.append(md(
        "## Exercises\n"
        "\n"
        "Two exercises that exercise the bindings end-to-end. Solutions "
        "are in collapsible `<details>` blocks at the bottom."
    ))

    cells.append(md(
        "### Exercise 1 — Compare PR and RKS for benzene at 350 K, 2000 kPa\n"
        "\n"
        "Benzene has T_c = 562.05 K, P_c = 4895 kPa, ω = 0.212. Compute "
        "the **liquid** Z, **vapor** Z, and pure-component ln(φ) for both "
        "PR1976 and RKS1972 at this T, P. Which EOS predicts a denser "
        "liquid (smaller Z_liquid)?"
    ))
    cells.append(code(
        "# TODO: compute z_l, z_v, ln_phi_l, ln_phi_v for both EOS and print them.\n"
        "# Hint: use the 'liquid' / 'vapor' phase strings; benzene is sub-critical at 350 K.\n"
        "BENZENE = dict(Tc=562.05, Pc=4895.0, omega=0.212)\n"
        "T, P = 350.0, 2000.0\n"
        "\n"
        "# ...\n"
    ))
    cells.append(md(
        "<details>\n"
        "<summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "for eos, label in [(CubicEos.PR1976, 'PR'), (CubicEos.RKS1972, 'RKS')]:\n"
        "    z_l = eos_z_factor(eos, T, P, **BENZENE, phase='liquid')\n"
        "    z_v = eos_z_factor(eos, T, P, **BENZENE, phase='vapor')\n"
        "    ln_phi_l = eos_ln_phi_pure(eos, T, P, **BENZENE, phase='liquid')\n"
        "    ln_phi_v = eos_ln_phi_pure(eos, T, P, **BENZENE, phase='vapor')\n"
        "    print(f'{label}: Z_l={z_l:.4f}, Z_v={z_v:.4f}, ln(φ_l)={ln_phi_l:+.4f}, ln(φ_v)={ln_phi_v:+.4f}')\n"
        "```\n"
        "\n"
        "PR predicts the smaller Z_liquid (denser liquid) — the classic PR "
        "vs RKS finding for sub-critical hydrocarbons.\n"
        "</details>"
    ))

    cells.append(md(
        "### Exercise 2 — Find the Boyle temperature of methane\n"
        "\n"
        "The Boyle temperature is the T at which B(T) = 0. Use the Brent "
        "root finder from M6 (`vle._engine.brent`) to find it for "
        "methane between 300 K and 1500 K."
    ))
    cells.append(code(
        "from vle._engine import brent\n"
        "\n"
        "# TODO: define a callable that returns virial_b_pure(...) for methane\n"
        "# at a given T, and pass it to brent() with a bracket that straddles\n"
        "# the Boyle temperature.\n"
        "\n"
        "# ...\n"
    ))
    cells.append(md(
        "<details>\n"
        "<summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "def b_of_t(t):\n"
        "    return virial_b_pure(METHANE['Tc'], METHANE['Pc'], METHANE['omega'], float(t))\n"
        "\n"
        "t_boyle = brent(b_of_t, 300.0, 1500.0, 1e-6)\n"
        "print(f'Methane Boyle temperature: T = {t_boyle:.2f} K  (T_r = {t_boyle / METHANE[\"Tc\"]:.3f})')\n"
        "```\n"
        "\n"
        "Around T ≈ 510 K, i.e. T_r ≈ 2.7 — matches the universal "
        "value predicted by the Pitzer correlation.\n"
        "</details>"
    ))

    # ---- References ------------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- **Research paper, Chapter II §2.3** — cubic EOS derivation and "
        "the Abbott family form. "
        "[`docs/en/research-paper/chapter-2-vle-theory.md`](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md)\n"
        "- **MODERNIZATION_PLAN.md — Phase 7 / 8 / 9** — phase-level "
        "scope notes for the pure-component layer. "
        "[`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md)\n"
        "- **Reference (4): Da Silva & Báez (1989)** — Antoine form "
        "and 3-parameter EOS source. `legacy/pascal/TERMOI.PAS` "
        "(Antoine) and `TERMOII.PAS` (3-param EOS, deferred to M7.3).\n"
        "- **Reference (5): Abbott (1989)** — cubic EOS family form. "
        "`legacy/vb6/McommonFunctions.bas:273` for the family-constants "
        "table.\n"
        "- **Reference (12): Poling, Prausnitz, O'Connell (2001)** — "
        "the discriminant robustness for the cubic solver (used by the "
        "Z-factor code path).\n"
        "- **Companion notebooks** (planned for later v0.x releases): "
        "[`02b_alpha_zoo.ipynb`](02b_alpha_zoo.ipynb) (M7.2, v0.4.0), "
        "[`02c_three_param_eos.ipynb`](02c_three_param_eos.ipynb) "
        "(M7.3, v0.5.0), and "
        "[`02d_advanced_saturation.ipynb`](02d_advanced_saturation.ipynb) "
        "(M7.4, v0.6.0)."
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
    client = NotebookClient(nb, timeout=120, kernel_name="python3")
    client.execute(cwd=str(NB_PATH.parent))
    NB_PATH.parent.mkdir(parents=True, exist_ok=True)
    NB_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    print(f"Wrote {NB_PATH}  ({len(nb.cells)} cells)")


if __name__ == "__main__":
    main()
