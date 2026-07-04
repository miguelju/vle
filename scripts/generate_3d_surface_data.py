#!/usr/bin/env python
"""Generate the pre-computed 3-D surface datasets for notebook 09.

Two datasets, written as small CSVs under ``notebooks/data/`` and committed to
the repo so the notebook (and the README hero image) render instantly without
re-running the thermodynamics:

1. **Phase-envelope dome** — methane/ethane (PR, classical mixing).
   For each overall composition z1, the full P–T phase envelope from
   ``trace_envelope_py`` (bubble→critical→dew, traced through the critical
   point), plus the critical locus from ``critical_point_py``.
   → ``phase_dome_ch4_c2h6.csv``   (z1, T [K], P [kPa])
   → ``critical_locus_ch4_c2h6.csv`` (z1, Tc [K], Pc [kPa])

2. **P–x–y "sail"** — methanol/water (γ-φ: van Laar liquid, Table 4.5
   parameters, ideal vapor). For each temperature, the bubble curve
   P(x1) and the paired dew curve P(y1).
   → ``pxy_sail_meoh_h2o.csv``  (T [K], x1, y1, P [kPa])

Run inside the ``vle`` conda environment after ``maturin develop``:

    ~/miniconda3/envs/vle/bin/python scripts/generate_3d_surface_data.py

Regenerating is only needed if the engine's thermodynamics change; the CSVs
are the source of truth for rendering.
"""

from __future__ import annotations

import csv
from pathlib import Path

import vle._engine as e

REPO_ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = REPO_ROOT / "notebooks" / "data"


# ---------------------------------------------------------------------------
# Dataset 1 — methane/ethane phase-envelope dome + critical locus (PR).
# ---------------------------------------------------------------------------

CH4_C2H6 = dict(
    tcs=[190.564, 305.32],
    pcs=[4599.0, 4872.0],
    omegas=[0.0115, 0.0995],
)


def generate_dome() -> tuple[list[tuple[float, float, float]], list[tuple[float, float, float]]]:
    """Trace envelopes + critical locus across the composition sweep.

    Returns (envelope_rows, critical_rows). Methane-rich compositions need a
    higher seed pressure (the envelope is steep near methane's low Tc); very
    methane-rich (z1 > 0.85) is skipped — the continuation becomes
    near-degenerate there and the surface is visually complete without it.
    """
    envelope_rows: list[tuple[float, float, float]] = []
    critical_rows: list[tuple[float, float, float]] = []
    z1_sweep = [0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45,
                0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80, 0.85]
    for z1 in z1_sweep:
        z = [z1, 1.0 - z1]
        # Seed pressure: higher on the methane-rich side.
        p_starts = [200.0, 500.0, 1000.0] if z1 < 0.6 else [500.0, 1000.0, 1500.0]
        pts = None
        for p_start in p_starts:
            try:
                pts = e.trace_envelope_py(
                    e.CubicEos.PR1976,
                    CH4_C2H6["tcs"], CH4_C2H6["pcs"], CH4_C2H6["omegas"],
                    z, p_start=p_start, max_points=70,
                )
                break
            except (RuntimeError, ValueError):
                continue
        if pts is None:
            print(f"  z1={z1}: envelope failed at every seed — skipped")
            continue
        for (t, p) in pts:
            envelope_rows.append((z1, t, p))
        # Critical locus point at this composition.
        t_init = z1 * CH4_C2H6["tcs"][0] + (1 - z1) * CH4_C2H6["tcs"][1]
        try:
            tc, pc, _ = e.critical_point_py(
                e.CubicEos.PR1976,
                CH4_C2H6["tcs"], CH4_C2H6["pcs"], CH4_C2H6["omegas"],
                z, t_init=t_init,
            )
            critical_rows.append((z1, tc, pc))
        except (RuntimeError, ValueError) as ex:
            print(f"  z1={z1}: critical point failed ({str(ex)[:50]}) — skipped")
        print(f"  z1={z1}: {len(pts)} envelope points")
    return envelope_rows, critical_rows


# ---------------------------------------------------------------------------
# Dataset 2 — methanol/water P–x–y sail (van Laar γ-φ, Table 4.5 params).
# ---------------------------------------------------------------------------

MEOH_H2O = dict(
    tcs=[512.6, 647.1],
    pcs=[8097.0, 22064.0],
    omegas=[0.564, 0.344],
    # Reduced-Antoine ln(P/Pc) = a1 - a2/(a3 + T), fit to the pure Psat's.
    psat=[[7.493, 3603.0, -34.29], [6.240, 3803.0, -46.00]],
    # van Laar parameters (research-paper Table 4.5).
    aij=[[0.0, 0.5853], [0.3458, 0.0]],
)


def generate_sail() -> list[tuple[float, float, float, float]]:
    """Bubble curves P(x1) with paired vapor y1 across a temperature sweep."""
    rows: list[tuple[float, float, float, float]] = []
    t_sweep = [298.0, 308.0, 318.0, 328.0, 338.0, 348.0, 358.0, 368.0, 378.0]
    x_sweep = [round(0.02 + 0.04 * i, 2) for i in range(25)]  # 0.02 .. 0.98
    for t in t_sweep:
        n_ok = 0
        for x1 in x_sweep:
            try:
                p, y, _ = e.bubble_pressure_py(
                    MEOH_H2O["tcs"], MEOH_H2O["pcs"], MEOH_H2O["omegas"],
                    [x1, 1.0 - x1], t,
                    vapor_kind="ideal", liquid_kind="activity",
                    liquid_activity=e.ActivityModel.VanLaar,
                    aij=MEOH_H2O["aij"], psat_coeffs=MEOH_H2O["psat"],
                    tol=1e-10,
                )
                rows.append((t, x1, y[0], p))
                n_ok += 1
            except (RuntimeError, ValueError):
                continue
        print(f"  T={t}: {n_ok} bubble points")
    return rows


def write_csv(path: Path, header: list[str], rows: list[tuple]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(header)
        for row in rows:
            w.writerow([f"{v:.6g}" for v in row])
    print(f"wrote {path.relative_to(REPO_ROOT)}  ({len(rows)} rows)")


def main() -> None:
    print("Generating methane/ethane phase-envelope dome (PR)...")
    env_rows, crit_rows = generate_dome()
    write_csv(DATA_DIR / "phase_dome_ch4_c2h6.csv", ["z1", "T_K", "P_kPa"], env_rows)
    write_csv(DATA_DIR / "critical_locus_ch4_c2h6.csv", ["z1", "Tc_K", "Pc_kPa"], crit_rows)

    print("Generating methanol/water P-x-y sail (van Laar)...")
    sail_rows = generate_sail()
    write_csv(DATA_DIR / "pxy_sail_meoh_h2o.csv", ["T_K", "x1", "y1", "P_kPa"], sail_rows)


if __name__ == "__main__":
    main()
