#!/usr/bin/env python3
"""Render the water P-v-T surface README hero (docs/assets/pvt_surface_hero.png).

Every point is computed by ``vle-steam`` (IAPWS-IF97, standard: IAPWS
R7-97(2012); textbook form: Wagner & Kretzschmar, *International Steam
Tables*, 3rd ed., 2019) through the batch ``vle.steam.properties`` API —
~36k state-point evaluations per render.

Companion to ``render_3d_hero.py`` and shares its visual language (GitHub-dark
background, plasma ramp, elev=22/azim=-58). The interactive, step-by-step
version of this figure lives in ``notebooks/14_pvt_surface.ipynb``.

Surface anatomy (the classic textbook "plastic model" of water):
  - single-phase sheet A — compressed liquid, continuing smoothly into the
    supercritical region above Tc;
  - single-phase sheet B — superheated vapor (T < Tc);
  - the two-phase dome — a *ruled* surface v = v_f + x*(v_g - v_f) swept in
    quality x from 0 to 1 along the saturation line (single-phase queries
    cannot reach inside the dome; the saturation backward equations can).
Sheets A/B use per-row pressure grids that END exactly on the saturation
line (boundary sample = v_f / v_g), so they meet the dome edge-to-edge with
no gap and no spurious "bridge" facets.

Run:  ~/miniconda3/envs/vle/bin/python scripts/render_pvt_hero.py
"""

from __future__ import annotations

from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from matplotlib import cm
from matplotlib.colors import Normalize
from matplotlib.lines import Line2D
from matplotlib.patches import Patch

import vle.steam as st

OUT = Path(__file__).resolve().parents[1] / "docs" / "assets" / "pvt_surface_hero.png"

BG = "#0d1117"  # GitHub dark background — the banner blends into the page
FG = "#c9d1d9"
ACCENT = "#00e5ff"    # critical point (same accent as the phase-envelope hero)
LIQ_EDGE = "#8dffb0"  # saturated-liquid boundary curve v_f(T)
VAP_EDGE = "#ffb46b"  # saturated-vapor boundary curve v_g(T)

T_TRIPLE, T_C, P_C = 273.16, 647.096, 22064.0  # K, K, kPa (IF97 constants)
P_MIN, P_MAX = 1.0, 100_000.0                  # kPa — the IF97 envelope
T_MAX = 1073.15                                # K — region 1/2/3 ceiling
K0 = 273.15                                    # K → °C offset for display

# ── saturation backbone (shared by both sheets and the dome) ─────────────
# Uniform coverage keeps the surface quads small (matplotlib's painter
# algorithm sorts whole polygons, so big translucent quads render as dark
# wedges); the extra geomspace points near Tc round off the dome's top.
t_sat = np.unique(np.concatenate([
    np.linspace(T_TRIPLE + 0.2, T_C - 0.05, 160),
    T_C - np.geomspace(40.0, 0.05, 50),
]))
sat = [st.saturation(T=float(t)) for t in t_sat]
p_sat = np.array([s.p for s in sat])
v_f = np.array([s.v_f for s in sat])
v_g = np.array([s.v_g for s in sat])

N_P = 60


def sheet(rows: list) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Stack per-row (v, T, P) samples into parametric plot_surface grids.

    Axes are display units: log10(v [m³/kg]), T [°C], log10(P [kPa]).
    """
    V = np.array([r[0] for r in rows])
    T = np.array([r[1] for r in rows])
    P = np.array([r[2] for r in rows])
    return np.log10(V), T - K0, np.log10(P)


def props_v(t_row: np.ndarray, p_row: np.ndarray) -> np.ndarray:
    """Specific volume v(T, P) in m³/kg via the batch IF97 kernel."""
    return st.properties(T=t_row, P=p_row)["v"]


# Sheet A — compressed liquid, each row ending exactly at (Psat, v_f);
# above Tc the rows span the full pressure range (continuous fluid).
rows_a = []
for i, t in enumerate(t_sat):
    p_row = np.geomspace(P_MAX, p_sat[i] * (1 + 1e-6), N_P)
    v_row = props_v(np.full(N_P, t), p_row)
    rows_a.append((np.append(v_row, v_f[i]), np.full(N_P + 1, t),
                   np.append(p_row, p_sat[i])))
for t in np.linspace(T_C + 0.5, T_MAX, 36):
    p_row = np.geomspace(P_MAX, P_MIN, N_P + 1)
    rows_a.append((props_v(np.full(N_P + 1, t), p_row), np.full(N_P + 1, t), p_row))
XA, YA, ZA = sheet(rows_a)

# Sheet B — superheated vapor, each row starting exactly at (Psat, v_g).
rows_b = []
for i, t in enumerate(t_sat):
    p_row = np.geomspace(p_sat[i] * (1 - 1e-6), P_MIN, N_P)
    v_row = props_v(np.full(N_P, t), p_row)
    rows_b.append((np.insert(v_row, 0, v_g[i]), np.full(N_P + 1, t),
                   np.insert(p_row, 0, p_sat[i])))
XB, YB, ZB = sheet(rows_b)

# The dome — ruled in quality x between the two boundary curves.
x_q = np.linspace(0.0, 1.0, 40)
VD = v_f[:, None] + x_q[None, :] * (v_g - v_f)[:, None]
XD = np.log10(VD)
YD = np.broadcast_to((t_sat - K0)[:, None], VD.shape)
ZD = np.broadcast_to(np.log10(p_sat)[:, None], VD.shape)

n_pts = XA.size + XB.size + XD.size
assert not (np.isnan(XA).any() or np.isnan(XB).any() or np.isnan(XD).any())
print(f"IF97 state points on the surface: {n_pts}")

# ── render ───────────────────────────────────────────────────────────────
fig = plt.figure(figsize=(12, 8), facecolor=BG)
ax = fig.add_subplot(111, projection="3d", facecolor=BG)
ax.set_facecolor(BG)
for pane in (ax.xaxis, ax.yaxis, ax.zaxis):
    pane.set_pane_color((1, 1, 1, 0.02))
    pane.label.set_color(FG)
    pane.set_tick_params(colors=FG, labelsize=8)
ax.grid(False)

# One sequential job: surface color encodes temperature (plasma, the repo's
# established ramp), shared by all three sheets. The dome reuses the same
# ramp darkened ~35% so it reads as "the interior" — identity is carried by
# shade + the boundary curves, not by a competing hue.
norm = Normalize(vmin=T_TRIPLE - K0, vmax=T_MAX - K0)
kw = dict(rstride=1, cstride=1, linewidth=0, antialiased=True, shade=False)
ax.plot_surface(XA, YA, ZA, facecolors=cm.plasma(norm(YA)), alpha=0.85, **kw)
ax.plot_surface(XB, YB, ZB, facecolors=cm.plasma(norm(YB)), alpha=0.85, **kw)
dome_rgba = cm.plasma(norm(YD))
dome_rgba[..., :3] *= 0.65
ax.plot_surface(XD, YD, ZD, facecolors=dome_rgba, alpha=1.0, **kw)

# Identity overlays: the saturation boundary curves + the critical point.
ax.plot(np.log10(v_f), t_sat - K0, np.log10(p_sat), color=LIQ_EDGE, lw=1.6,
        alpha=0.95)
ax.plot(np.log10(v_g), t_sat - K0, np.log10(p_sat), color=VAP_EDGE, lw=1.6,
        alpha=0.95)
vc = st.Water(T=T_C - 0.2, P=P_C * 0.999).v  # ≈ critical volume
ax.scatter([np.log10(vc)], [T_C - K0], [np.log10(P_C)], color=ACCENT, s=110,
           marker="*", zorder=11, depthshade=False)

ax.set_xlabel("specific volume v (m$^3$/kg)", color=FG, fontsize=9, labelpad=8)
ax.set_ylabel("T (°C)", color=FG, fontsize=9, labelpad=8)
ax.set_zlabel("P (kPa)", color=FG, fontsize=9, labelpad=6)
ax.set_xticks([-3, -2, -1, 0, 1, 2])
ax.set_xticklabels(["0.001", "0.01", "0.1", "1", "10", "100"])
ax.set_zticks([0, 1, 2, 3, 4, 5])
ax.set_zticklabels(["1", "10", "10$^2$", "10$^3$", "10$^4$", "10$^5$"])
ax.view_init(elev=22, azim=-58)
ax.set_title(
    "Water P–v–T surface — every point from vle-steam (IAPWS-IF97)",
    color=FG, fontsize=11, pad=12,
)

# Full legend (proxy artists — plot_surface has no direct legend support).
legend_items = [
    (Patch(facecolor=cm.plasma(0.65), alpha=0.85), "single-phase sheets (liquid wall · superheat · supercritical)"),
    (Patch(facecolor=np.array(cm.plasma(0.35)) * [0.65, 0.65, 0.65, 1.0]), "two-phase dome (ruled in quality $x$: $v = v_f + x\\,(v_g - v_f)$)"),
    (Line2D([], [], color=LIQ_EDGE, lw=1.6), "saturated liquid $v_f(T)$"),
    (Line2D([], [], color=VAP_EDGE, lw=1.6), "saturated vapor $v_g(T)$"),
    (Line2D([], [], color=ACCENT, marker="*", ms=11, lw=0), "critical point (373.95 °C, 22.064 MPa)"),
]
ax.legend([h for h, _ in legend_items], [t for _, t in legend_items],
          loc="upper left", fontsize=8, facecolor=BG, edgecolor="none",
          labelcolor=FG)

sm = cm.ScalarMappable(norm=norm, cmap="plasma")
cb = fig.colorbar(sm, ax=ax, shrink=0.55, pad=0.06)
cb.set_label("temperature (°C)", color=FG, fontsize=9)
cb.ax.tick_params(colors=FG, labelsize=8)
cb.outline.set_visible(False)

OUT.parent.mkdir(parents=True, exist_ok=True)
fig.savefig(OUT, dpi=170, facecolor=BG, bbox_inches="tight")
print(f"saved {OUT}")
