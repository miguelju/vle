#!/usr/bin/env python
"""Render the README hero image from the pre-computed 3-D surface CSVs.

Produces ``docs/assets/phase_surfaces_hero.png`` — a dark-themed banner with
the methane/ethane phase-envelope dome (left) and the methanol/water P–x–y
sail (right), both drawn purely from ``notebooks/data/*.csv`` (no engine
calls). Rerun only if the datasets or the styling change:

    ~/miniconda3/envs/vle/bin/python scripts/render_3d_hero.py
"""

from __future__ import annotations

import csv
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from matplotlib import cm

REPO_ROOT = Path(__file__).resolve().parents[1]
DATA = REPO_ROOT / "notebooks" / "data"
OUT = REPO_ROOT / "docs" / "assets" / "phase_surfaces_hero.png"

BG = "#0d1117"  # GitHub dark background — the banner blends into the page
FG = "#c9d1d9"


def load(name: str) -> dict[str, np.ndarray]:
    with (DATA / name).open() as fh:
        rows = list(csv.reader(fh))
    header, body = rows[0], np.array(rows[1:], dtype=float)
    return {h: body[:, i] for i, h in enumerate(header)}


def style_axis(ax) -> None:
    ax.set_facecolor(BG)
    for pane in (ax.xaxis, ax.yaxis, ax.zaxis):
        pane.set_pane_color((1, 1, 1, 0.02))
        pane.label.set_color(FG)
        pane.set_tick_params(colors=FG, labelsize=7)
    ax.grid(False)


def main() -> None:
    dome = load("phase_dome_ch4_c2h6.csv")
    crit = load("critical_locus_ch4_c2h6.csv")
    sail = load("pxy_sail_meoh_h2o.csv")

    fig = plt.figure(figsize=(14, 6), facecolor=BG)

    # ---- Left: the phase-envelope dome --------------------------------
    ax1 = fig.add_subplot(1, 2, 1, projection="3d")
    style_axis(ax1)
    z1, t, p = dome["z1"], dome["T_K"], dome["P_kPa"]
    # Envelope curves per composition, colored by pressure (plasma).
    norm = plt.Normalize(p.min(), p.max())
    for z_val in np.unique(z1):
        m = z1 == z_val
        ax1.plot(
            z1[m], t[m], p[m] / 1000.0,
            lw=1.4,
            color=cm.plasma(norm(p[m].mean())),
            alpha=0.95,
        )
    # Translucent surface underneath for the "dome" body.
    ax1.plot_trisurf(
        z1, t, p / 1000.0, cmap="plasma", alpha=0.22, linewidth=0.0, antialiased=True
    )
    # Critical locus — the glowing ridge.
    order = np.argsort(crit["z1"])
    ax1.plot(
        crit["z1"][order], crit["Tc_K"][order], crit["Pc_kPa"][order] / 1000.0,
        color="#00e5ff", lw=3.0, alpha=1.0, zorder=10, label="critical locus",
    )
    ax1.scatter(
        crit["z1"][order], crit["Tc_K"][order], crit["Pc_kPa"][order] / 1000.0,
        color="#00e5ff", s=12, zorder=11,
    )
    ax1.set_xlabel("methane mole fraction", color=FG, fontsize=8, labelpad=6)
    ax1.set_ylabel("T (K)", color=FG, fontsize=8, labelpad=6)
    ax1.set_zlabel("P (MPa)", color=FG, fontsize=8, labelpad=4)
    ax1.set_title(
        "Phase-envelope dome — methane/ethane (PR)\nMichelsen continuation through the critical locus",
        color=FG, fontsize=10, pad=10,
    )
    ax1.view_init(elev=22, azim=-58)
    ax1.legend(loc="upper left", fontsize=8, facecolor=BG, edgecolor="none",
               labelcolor=FG)

    # ---- Right: the P-x-y sail -----------------------------------------
    ax2 = fig.add_subplot(1, 2, 2, projection="3d")
    style_axis(ax2)
    ts, x1, y1, ps = sail["T_K"], sail["x1"], sail["y1"], sail["P_kPa"]
    # Bubble surface (over x1) and dew surface (over y1) as trisurfs.
    ax2.plot_trisurf(x1, ts, ps, cmap="viridis", alpha=0.75, linewidth=0.0,
                     antialiased=True)
    ax2.plot_trisurf(y1, ts, ps, cmap="magma", alpha=0.45, linewidth=0.0,
                     antialiased=True)
    # Isotherm rims to sharpen the sail edges.
    for t_val in np.unique(ts):
        m = ts == t_val
        ax2.plot(x1[m], ts[m], ps[m], color="#8dffb0", lw=0.9, alpha=0.9)
        ax2.plot(y1[m], ts[m], ps[m], color="#ffb46b", lw=0.9, alpha=0.9)
    ax2.set_xlabel("methanol mole fraction (x bubble / y dew)", color=FG,
                   fontsize=8, labelpad=6)
    ax2.set_ylabel("T (K)", color=FG, fontsize=8, labelpad=6)
    ax2.set_zlabel("P (kPa)", color=FG, fontsize=8, labelpad=4)
    ax2.set_title(
        "P–x–y sail — methanol/water (van Laar γ-φ)\nbubble (green rims) and dew (orange rims) surfaces",
        color=FG, fontsize=10, pad=10,
    )
    ax2.view_init(elev=20, azim=-50)

    fig.suptitle(
        "vle-thermo — every point below computed by the Rust engine",
        color=FG, fontsize=12, y=0.98,
    )
    fig.tight_layout(rect=(0, 0, 1, 0.96))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(OUT, dpi=170, facecolor=BG, bbox_inches="tight")
    size_kb = OUT.stat().st_size / 1024
    print(f"wrote {OUT.relative_to(REPO_ROOT)}  ({size_kb:.0f} KB)")


if __name__ == "__main__":
    main()
