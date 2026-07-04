#!/usr/bin/env python3
"""Generate ``notebooks/09_3d_phase_surfaces.ipynb`` — the 3-D showcase.

Renders two 3-D phase surfaces from the pre-computed CSVs in
``notebooks/data/`` (committed to the repo, so the notebook never re-runs the
thermodynamics by default) and includes the full generation code behind a
``REGENERATE`` guard:

1. The methane/ethane **phase-envelope dome** with its critical locus
   (``trace_envelope_py`` + ``critical_point_py``).
2. The methanol/water **P–x–y sail** vs temperature (γ-φ bubble/dew from
   ``bubble_pressure_py``).

Structure follows CLAUDE.md *Notebook Conventions*. Executed top-to-bottom in
a fresh kernel before saving.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "09_3d_phase_surfaces.ipynb"

SANDBOX_NOTICE = (
    "> 💾 **Notebook sandbox notice — only applies if you're running this "
    "notebook on a shared JupyterLab someone set up for you.** If you were given "
    "a URL to a shared JupyterLab environment, treat it as an *educational "
    "sandbox*: edits you make to this notebook won't survive a container "
    "restart, the bundled `vle-thermo` version may lag PyPI, and any "
    "`pip install` you run inside this container is ephemeral (it vanishes "
    "when your session is culled). For real work, install `vle-thermo` in "
    "your own Jupyter environment with `pip install vle-thermo` and run the "
    "notebook there — see the [project README](https://github.com/miguelju/vle/blob/main/README.md). "
    "**If you opened this notebook in your own Jupyter, you can ignore this "
    "notice.**"
)
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
        "# 3-D Phase Surfaces — the Showcase Notebook\n"
        "\n"
        "Two classic thermodynamic surfaces, every point computed by the "
        "`vle-thermo` Rust engine:\n"
        "\n"
        "1. The **phase-envelope dome** of methane/ethane — the P–T "
        "saturation boundary at each composition, stacked into a 3-D dome, "
        "with the **critical locus** riding the ridge between the two pure "
        "critical points. Traced by the Michelsen continuation solver, which "
        "walks *through* each mixture's critical point.\n"
        "2. The **P–x–y \"sail\"** of methanol/water — the bubble and dew "
        "surfaces vs temperature from the γ-φ (van Laar) path, using the "
        "research paper's Table 4.5 parameters.\n"
        "\n"
        "The surface points are **pre-computed and stored in "
        "`notebooks/data/*.csv`** (committed to the repo), so this notebook "
        "renders instantly without re-running the thermodynamics. The full "
        "generation code is included below behind a `REGENERATE` switch."
    ))
    c.append(md(SANDBOX_NOTICE))
    c.append(md(
        "## Setup (optional)\n\nThe cell below is **commented out by default**. "
        "Uncomment it to pull the latest `vle-thermo` from PyPI."
    ))
    c.append(code(SETUP_CELL))

    c.append(md(
        "## Context\n"
        "\n"
        "A binary mixture's saturation boundary is a *loop* in the P–T plane "
        "at each fixed composition — bubble branch and dew branch meeting at "
        "the mixture critical point (see "
        "[Chapter II](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md) "
        "and the [Chapter IV validation](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md)). "
        "Sweep the composition and those loops sweep out a **dome** whose "
        "ridge is the critical locus. The thesis's differential stepping "
        "could not cross the critical point; the modern engine's continuation "
        "solver (`trace_envelope_py`, Michelsen (24)) passes through it "
        "smoothly, which is what makes this surface drawable at all.\n"
        "\n"
        "For a *sub*-critical polar system, the natural 3-D object is instead "
        "the P–x–y **sail**: the bubble surface $P(T, x_1)$ and dew surface "
        "$P(T, y_1)$, joined at the pure-component edges."
    ))

    c.append(md(
        "## Load the pre-computed surfaces\n"
        "\n"
        "Data files (small CSVs, committed):\n"
        "- `data/phase_dome_ch4_c2h6.csv` — (z₁, T, P) envelope points, 17 compositions × ~70 points\n"
        "- `data/critical_locus_ch4_c2h6.csv` — (z₁, Tc, Pc) critical locus\n"
        "- `data/pxy_sail_meoh_h2o.csv` — (T, x₁, y₁, P) bubble/dew points, 9 isotherms × 25 compositions"
    ))
    c.append(code(
        "import csv\n"
        "from pathlib import Path\n"
        "\n"
        "import numpy as np\n"
        "import matplotlib.pyplot as plt\n"
        "from matplotlib import cm\n"
        "%matplotlib inline\n"
        "\n"
        "def load(name):\n"
        "    with (Path('data') / name).open() as fh:\n"
        "        rows = list(csv.reader(fh))\n"
        "    header, body = rows[0], np.array(rows[1:], dtype=float)\n"
        "    return {h: body[:, i] for i, h in enumerate(header)}\n"
        "\n"
        "dome = load('phase_dome_ch4_c2h6.csv')\n"
        "crit = load('critical_locus_ch4_c2h6.csv')\n"
        "sail = load('pxy_sail_meoh_h2o.csv')\n"
        "print(f\"dome: {len(dome['z1'])} pts   critical locus: {len(crit['z1'])} pts   sail: {len(sail['T_K'])} pts\")"
    ))

    c.append(md(
        "## Surface 1 — the phase-envelope dome (methane/ethane, PR)\n"
        "\n"
        "Each colored curve is one composition's full phase envelope; the "
        "cyan ridge is the critical locus connecting pure ethane's critical "
        "point (305.3 K, 4.87 MPa) toward pure methane's (190.6 K, 4.60 MPa). "
        "Everything under the dome is two-phase."
    ))
    c.append(code(
        "fig = plt.figure(figsize=(10, 7))\n"
        "ax = fig.add_subplot(projection='3d')\n"
        "z1, t, p = dome['z1'], dome['T_K'], dome['P_kPa']\n"
        "norm = plt.Normalize(p.min(), p.max())\n"
        "for z_val in np.unique(z1):\n"
        "    m = z1 == z_val\n"
        "    ax.plot(z1[m], t[m], p[m] / 1000.0, lw=1.3,\n"
        "            color=cm.plasma(norm(p[m].mean())), alpha=0.95)\n"
        "ax.plot_trisurf(z1, t, p / 1000.0, cmap='plasma', alpha=0.2, linewidth=0)\n"
        "order = np.argsort(crit['z1'])\n"
        "ax.plot(crit['z1'][order], crit['Tc_K'][order], crit['Pc_kPa'][order] / 1000.0,\n"
        "        color='#00b8d4', lw=3, zorder=10, label='critical locus')\n"
        "ax.set_xlabel('methane mole fraction')\n"
        "ax.set_ylabel('T (K)')\n"
        "ax.set_zlabel('P (MPa)')\n"
        "ax.set_title('Phase-envelope dome — methane/ethane (PR)')\n"
        "ax.view_init(elev=22, azim=-58)\n"
        "ax.legend()\n"
        "plt.show()"
    ))
    c.append(code(
        "# Sanity pins: the critical locus must sit between the pure critical\n"
        "# temperatures, and the dome's maximum pressure must exceed both pure\n"
        "# critical pressures (the classic mixture Pc maximum).\n"
        "assert crit['Tc_K'].min() > 190.5 and crit['Tc_K'].max() < 305.4\n"
        "assert p.max() / 1000.0 > 4.9, 'dome should rise above both pure Pc values'\n"
        "print('dome sanity checks pass')"
    ))

    c.append(md(
        "## Surface 2 — the P–x–y sail (methanol/water, van Laar γ-φ)\n"
        "\n"
        "The **bubble surface** (viridis, over the liquid composition $x_1$) "
        "and **dew surface** (magma, over the vapor composition $y_1$) share "
        "the same pressures — each bubble point's vapor lands on the dew "
        "surface. The gap between them at fixed (T, P) is the two-phase "
        "region; the surfaces close at the pure edges."
    ))
    c.append(code(
        "fig = plt.figure(figsize=(10, 7))\n"
        "ax = fig.add_subplot(projection='3d')\n"
        "ts, x1, y1, ps = sail['T_K'], sail['x1'], sail['y1'], sail['P_kPa']\n"
        "ax.plot_trisurf(x1, ts, ps, cmap='viridis', alpha=0.75, linewidth=0)\n"
        "ax.plot_trisurf(y1, ts, ps, cmap='magma', alpha=0.45, linewidth=0)\n"
        "for t_val in np.unique(ts):\n"
        "    m = ts == t_val\n"
        "    ax.plot(x1[m], ts[m], ps[m], color='green', lw=0.8, alpha=0.8)\n"
        "    ax.plot(y1[m], ts[m], ps[m], color='darkorange', lw=0.8, alpha=0.8)\n"
        "ax.set_xlabel('methanol mole fraction (x bubble / y dew)')\n"
        "ax.set_ylabel('T (K)')\n"
        "ax.set_zlabel('P (kPa)')\n"
        "ax.set_title('P–x–y sail — methanol/water (van Laar)')\n"
        "ax.view_init(elev=20, azim=-50)\n"
        "plt.show()"
    ))
    c.append(code(
        "# Sanity pins: at every isotherm the bubble pressure rises with the\n"
        "# methanol (more volatile) fraction, and y1 >= x1 throughout.\n"
        "for t_val in np.unique(ts):\n"
        "    m = ts == t_val\n"
        "    assert np.all(np.diff(ps[m]) > 0), f'P(x1) not monotone at T={t_val}'\n"
        "    assert np.all(y1[m] >= x1[m] - 1e-9), f'y1 < x1 at T={t_val}'\n"
        "print('sail sanity checks pass')"
    ))

    c.append(md(
        "## Regenerating the data (optional)\n"
        "\n"
        "The cell below contains the **complete generation code** — the same "
        "logic as `scripts/generate_3d_surface_data.py`. It is guarded by "
        "`REGENERATE = False` so the notebook renders from the committed CSVs "
        "by default; flip the switch to recompute every point with the live "
        "engine (a couple of minutes). The dome traces 17 phase envelopes "
        "with `trace_envelope_py` + 17 critical points with "
        "`critical_point_py`; the sail solves 225 γ-φ bubble points with "
        "`bubble_pressure_py`."
    ))
    c.append(code(
        "REGENERATE = False  # flip to True to recompute the CSVs with the live engine\n"
        "\n"
        "if REGENERATE:\n"
        "    import vle._engine as e\n"
        "\n"
        "    # ---- Surface 1: methane/ethane dome + critical locus (PR) ----\n"
        "    tcs, pcs, om = [190.564, 305.32], [4599.0, 4872.0], [0.0115, 0.0995]\n"
        "    env_rows, crit_rows = [], []\n"
        "    for z1v in [round(0.05 * k, 2) for k in range(1, 18)]:  # 0.05 .. 0.85\n"
        "        z = [z1v, 1 - z1v]\n"
        "        pts = None\n"
        "        for p_start in ([200.0, 500.0, 1000.0] if z1v < 0.6 else [500.0, 1000.0, 1500.0]):\n"
        "            try:\n"
        "                pts = e.trace_envelope_py(e.CubicEos.PR1976, tcs, pcs, om, z,\n"
        "                                          p_start=p_start, max_points=70)\n"
        "                break\n"
        "            except (RuntimeError, ValueError):\n"
        "                continue\n"
        "        if pts is None:\n"
        "            continue\n"
        "        env_rows += [(z1v, tt, pp) for (tt, pp) in pts]\n"
        "        tc_i, pc_i, _ = e.critical_point_py(\n"
        "            e.CubicEos.PR1976, tcs, pcs, om, z,\n"
        "            t_init=z1v * tcs[0] + (1 - z1v) * tcs[1])\n"
        "        crit_rows.append((z1v, tc_i, pc_i))\n"
        "\n"
        "    # ---- Surface 2: methanol/water P-x-y sail (van Laar) ----\n"
        "    tcs2, pcs2, om2 = [512.6, 647.1], [8097.0, 22064.0], [0.564, 0.344]\n"
        "    psat2 = [[7.493, 3603.0, -34.29], [6.240, 3803.0, -46.00]]\n"
        "    aij2 = [[0.0, 0.5853], [0.3458, 0.0]]  # Table 4.5\n"
        "    sail_rows = []\n"
        "    for tv in [298.0 + 10.0 * k for k in range(9)]:\n"
        "        for x1v in [round(0.02 + 0.04 * i, 2) for i in range(25)]:\n"
        "            pv, yv, _ = e.bubble_pressure_py(\n"
        "                tcs2, pcs2, om2, [x1v, 1 - x1v], tv,\n"
        "                vapor_kind='ideal', liquid_kind='activity',\n"
        "                liquid_activity=e.ActivityModel.VanLaar,\n"
        "                aij=aij2, psat_coeffs=psat2, tol=1e-10)\n"
        "            sail_rows.append((tv, x1v, yv[0], pv))\n"
        "\n"
        "    # ---- Write the CSVs the plotting cells read ----\n"
        "    def write(name, header, rows):\n"
        "        with (Path('data') / name).open('w', newline='') as fh:\n"
        "            w = csv.writer(fh)\n"
        "            w.writerow(header)\n"
        "            w.writerows([[f'{v:.6g}' for v in r] for r in rows])\n"
        "        print(f'wrote data/{name} ({len(rows)} rows)')\n"
        "\n"
        "    write('phase_dome_ch4_c2h6.csv', ['z1', 'T_K', 'P_kPa'], env_rows)\n"
        "    write('critical_locus_ch4_c2h6.csv', ['z1', 'Tc_K', 'Pc_kPa'], crit_rows)\n"
        "    write('pxy_sail_meoh_h2o.csv', ['T_K', 'x1', 'y1', 'P_kPa'], sail_rows)\n"
        "else:\n"
        "    print('using the committed CSVs (set REGENERATE = True to recompute)')"
    ))

    c.append(md(
        "### Interactive version (optional)\n"
        "\n"
        "For a rotate/zoom version when running live, uncomment the Plotly "
        "cell below (`pip install plotly` if needed). Note GitHub does *not* "
        "render Plotly output in saved notebooks — the matplotlib figures "
        "above are the always-visible renders."
    ))
    c.append(code(
        "# %pip install plotly\n"
        "# import plotly.graph_objects as go\n"
        "# fig = go.Figure()\n"
        "# for z_val in np.unique(dome['z1']):\n"
        "#     m = dome['z1'] == z_val\n"
        "#     fig.add_trace(go.Scatter3d(x=dome['z1'][m], y=dome['T_K'][m],\n"
        "#         z=dome['P_kPa'][m] / 1000.0, mode='lines', showlegend=False))\n"
        "# fig.add_trace(go.Scatter3d(x=crit['z1'], y=crit['Tc_K'],\n"
        "#     z=crit['Pc_kPa'] / 1000.0, mode='lines+markers', name='critical locus',\n"
        "#     line=dict(color='cyan', width=6)))\n"
        "# fig.update_layout(scene=dict(xaxis_title='z1 methane',\n"
        "#     yaxis_title='T (K)', zaxis_title='P (MPa)'), template='plotly_dark')\n"
        "# fig.show()"
    ))

    c.append(md(
        "## Exercise 1 — a different pair\n"
        "\n"
        "Regenerate the dome for **ethane/propane** (Tc = 305.32/369.83 K, "
        "Pc = 4872/4248 kPa, ω = 0.0995/0.1523) using the `REGENERATE` code "
        "above as a template. How does the critical locus differ from "
        "methane/ethane's? (Hint: this pair is closer-boiling.)"
    ))
    c.append(code(
        "# TODO: adapt the dome-generation loop to ethane/propane and plot it\n"
        "# with the same plotting code as Surface 1.\n"
    ))
    c.append(md(
        "<details><summary>Solution sketch</summary>\n"
        "\n"
        "Replace `tcs, pcs, om` with the ethane/propane constants and rerun "
        "the loop (seeds of 200–500 kPa work across the whole sweep — the "
        "pair is closer-boiling, so no methane-like steep side). The critical "
        "locus hugs the straight line between the pure critical points much "
        "more closely, and the dome's pressure maximum barely exceeds the "
        "pure-component Pc values — near-ideal mixtures make gentle domes.\n"
        "</details>"
    ))

    c.append(md(
        "## Exercise 2 — slice the sail\n"
        "\n"
        "Extract the 338 K isotherm from the sail data and plot the classic "
        "2-D P–x–y diagram (P vs x₁ and P vs y₁ on one axes). Confirm it "
        "matches the corresponding slice of the 3-D figure."
    ))
    c.append(code(
        "# TODO: mask sail['T_K'] == 338.0 and plot P vs x1 and P vs y1.\n"
    ))
    c.append(md(
        "<details><summary>Solution</summary>\n"
        "\n"
        "```python\n"
        "m = sail['T_K'] == 338.0\n"
        "plt.plot(sail['x1'][m], sail['P_kPa'][m], label='bubble P(x1)')\n"
        "plt.plot(sail['y1'][m], sail['P_kPa'][m], label='dew P(y1)')\n"
        "plt.xlabel('methanol mole fraction'); plt.ylabel('P (kPa)')\n"
        "plt.title('methanol/water at 338 K'); plt.legend(); plt.grid(True)\n"
        "plt.show()\n"
        "```\n"
        "</details>"
    ))

    c.append(md(
        "## References\n"
        "\n"
        "- (24) Michelsen (1980) — phase-envelope continuation; engine: `engine/src/flash/envelope.rs`.\n"
        "- (16) Heidemann & Khalil (1980) — the critical locus; engine: `engine/src/flash/critical.rs`.\n"
        "- Bubble/dew γ-φ path: [notebook 04](04_bubble_dew_point.ipynb) and `engine/src/flash/bubble.rs`.\n"
        "- Data generation: `scripts/generate_3d_surface_data.py`; hero image: `scripts/render_3d_hero.py`.\n"
        "- Research paper: [Chapter II](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-2-vle-theory.md), [Chapter IV](https://github.com/miguelju/vle/blob/main/docs/en/research-paper/chapter-4-validation.md).\n"
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
