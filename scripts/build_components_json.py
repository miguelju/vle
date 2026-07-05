#!/usr/bin/env python
"""Generate the bundled JSON component database (Milestone 10).

Writes the SAME content to two places (single source of truth = this script):

- ``python/src/vle/data/components.json`` — shipped inside the wheel, loaded
  by :mod:`vle.components` via ``importlib.resources``.
- ``notebooks/data/components.json``      — the copy the notebooks read, so a
  notebook works even without the package data (e.g. on a hub that mounts
  only the notebooks directory).

Run from the repo root with the ``vle`` conda env::

    ~/miniconda3/envs/vle/bin/python scripts/build_components_json.py

Data provenance
---------------
- ``tc``/``pc``/``omega``/``zc``/``vc``/``tb``/``mw``: same DIPPR-backed
  values as the SQLite seed (``python/src/vle/db/sql/seed_chapter4.sql``,
  extracted from the CalebBell/thermo library 2026-04-05).
- ``psat_coeffs``: the engine's reduced Antoine form
  ``ln(Psat/Pc) = a1 − a2/(a3 + T)`` (T in K, P in kPa), converted EXACTLY
  from published NIST-WebBook Antoine constants
  ``log10(P[bar]) = A − B/(T + C)`` via::

      a1 = ln(10)·A + ln(100) − ln(Pc[kPa]);  a2 = ln(10)·B;  a3 = C

  For CO2 and H2S (no 1-atm liquid boiling point — CO2 sublimes) the
  coefficients are a two-point fit through documented (T, Psat) anchors
  with a3 = 0.
- ``liquid_volume``: molar volume of the saturated liquid near 298 K in
  cm³/mol (CRC Handbook values) — used by Wilson / Poynting.

Every Antoine-derived entry is sanity-checked: Psat(Tb) must reproduce
101.325 kPa within 5% (extrapolation outside the stated NIST range is
allowed but the boiling point must stay honest).
"""

from __future__ import annotations

import json
import math
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

LN10 = math.log(10.0)
LN100 = math.log(100.0)


def reduced_antoine(a: float, b: float, c: float, pc_kpa: float) -> list[float]:
    """Convert NIST Antoine (log10, bar, K) to reduced form (ln, Psat/Pc, K)."""
    return [LN10 * a + LN100 - math.log(pc_kpa), LN10 * b, c]


def two_point_fit(pc_kpa: float, t1: float, p1: float, t2: float, p2: float) -> list[float]:
    """Fit ``ln(P/Pc) = a1 − a2/T`` (a3 = 0) through two (T [K], Psat [kPa]) anchors."""
    y1 = math.log(p1 / pc_kpa)
    y2 = math.log(p2 / pc_kpa)
    a2 = (y2 - y1) / (1.0 / t1 - 1.0 / t2)
    a1 = y1 + a2 / t1
    return [a1, a2, 0.0]


# name: (formula, cas, mw, tc [K], pc [kPa], omega, zc, vc [cm3/mol], tb [K],
#        NIST Antoine (A, B, C) or None, liquid_volume [cm3/mol] or None)
RAW = {
    "methane": ("CH4", "74-82-8", 16.0425, 190.564, 4599.2, 0.01142, 0.28629, 98.628, 111.667,
                (3.9895, 443.028, -0.49), None),
    "ethane": ("C2H6", "74-84-0", 30.0690, 305.322, 4872.2, 0.0995, 0.27990, 145.839, 184.569,
               (3.93835, 659.739, -16.719), None),
    "propane": ("C3H8", "74-98-6", 44.0956, 369.89, 4251.2, 0.1521, 0.27646, 200.000, 231.036,
                (3.98292, 819.296, -24.417), None),
    "n-butane": ("C4H10", "106-97-8", 58.1222, 425.125, 3796.0, 0.201, 0.27377, 254.922, 272.660,
                 (4.35576, 1175.581, -2.071), 100.4),
    "n-pentane": ("C5H12", "109-66-0", 72.1488, 469.7, 3367.5, 0.251, 0.26863, 311.526, 309.209,
                  (3.9892, 1070.617, -40.454), 116.1),
    "carbon dioxide": ("CO2", "124-38-9", 44.0095, 304.1282, 7377.3, 0.22394, 0.27459, 94.118, 194.67,
                       None, None),
    "hydrogen sulfide": ("H2S", "7783-06-4", 34.0809, 373.1, 9000.0, 0.1005, 0.28471, 98.135, 212.855,
                         None, None),
    "benzene": ("C6H6", "71-43-2", 78.1118, 562.02, 4907.277, 0.211, 0.26920, 256.345, 353.219,
                (4.01814, 1203.835, -53.226), 89.4),
    "cyclohexane": ("C6H12", "110-82-7", 84.1595, 553.6, 4080.5, 0.2096, 0.27497, 310.174, 353.865,
                    (3.96988, 1203.526, -50.287), 108.7),
    "methylcyclohexane": ("C7H14", "108-87-2", 98.1861, 572.2, 3470.0, 0.234, 0.26815, 367.647, 374.010,
                          None, 128.3),
    "n-hexane": ("C6H14", "110-54-3", 86.1754, 507.82, 3044.1, 0.3, 0.26643, 369.549, 341.866,
                 (4.00266, 1171.53, -48.784), 131.6),
    "n-heptane": ("C7H16", "142-82-5", 100.2019, 540.2, 2735.73, 0.349, 0.26141, 429.185, 371.550,
                  (4.02832, 1268.636, -56.199), 147.5),
    "methanol": ("CH4O", "67-56-1", 32.0419, 513.38, 8215.85, 0.5625, 0.21909, 113.828, 337.632,
                 (5.20409, 1581.341, -33.50), 40.7),
    "water": ("H2O", "7732-18-5", 18.0153, 647.096, 22064.0, 0.3443, 0.22944, 55.948, 373.124,
              (4.6543, 1435.264, -64.848), 18.07),
    "2-propanol": ("C3H8O", "67-63-0", 60.0950, 508.3, 4764.0, 0.665, 0.25025, 222.000, 355.36,
                   (4.8610, 1357.427, -75.814), 76.92),
}

# Two-point (T [K], Psat [kPa]) anchors for the compounds without a usable
# liquid-range NIST Antoine set (CO2 sublimes at 1 atm; H2S's set is spotty).
# Anchors: CRC Handbook saturated-vapor-pressure tables.
TWO_POINT = {
    "carbon dioxide": ((273.15, 3485.1), (298.15, 6434.0)),
    "hydrogen sulfide": ((212.855, 101.325), (273.15, 1030.0)),
}


def build() -> dict:
    compounds = {}
    for name, (formula, cas, mw, tc, pc, omega, zc, vc, tb, antoine, vliq) in RAW.items():
        if antoine is not None:
            coeffs = reduced_antoine(*antoine, pc)
            psat_source = f"NIST WebBook Antoine (log10, bar) A={antoine[0]}, B={antoine[1]}, C={antoine[2]}, converted exactly"
        elif name in TWO_POINT:
            (t1, p1), (t2, p2) = TWO_POINT[name]
            coeffs = two_point_fit(pc, t1, p1, t2, p2)
            psat_source = f"two-point Clausius-Clapeyron fit through ({t1} K, {p1} kPa) and ({t2} K, {p2} kPa)"
        else:
            # Acentric-factor-anchored fit: exact through Psat(Tb) = 1 atm
            # and the ω definition point Psat(0.7·Tc) = Pc·10^(−1−ω).
            coeffs = two_point_fit(pc, tb, 101.325, 0.7 * tc, pc * 10.0 ** (-1.0 - omega))
            psat_source = "two-point fit through Psat(Tb) = 101.325 kPa and the acentric-factor definition Psat(0.7 Tc) = Pc*10^(-1-w)"

        # Sanity: the reduced Antoine must reproduce ~1 atm at Tb (CO2's
        # "boiling point" is a sublimation T — skip it there).
        if antoine is not None:
            psat_tb = pc * math.exp(coeffs[0] - coeffs[1] / (coeffs[2] + tb))
            err = abs(psat_tb - 101.325) / 101.325
            assert err < 0.05, f"{name}: Psat(Tb={tb}) = {psat_tb:.2f} kPa (err {err:.1%})"

        entry = {
            "formula": formula,
            "cas": cas,
            "mw": mw,
            "tc": tc,
            "pc": pc,
            "omega": omega,
            "zc": zc,
            "vc": vc,
            "tb": tb,
            "psat_coeffs": [round(v, 6) for v in coeffs],
            "psat_source": psat_source,
        }
        if vliq is not None:
            entry["liquid_volume"] = vliq
        compounds[name] = entry
    return {
        "_meta": {
            "description": "vle-thermo bundled component database (Milestone 10)",
            "units": {
                "tc": "K", "pc": "kPa (absolute)", "vc": "cm3/mol", "tb": "K",
                "mw": "g/mol", "liquid_volume": "cm3/mol at ~298 K",
                "psat_coeffs": "reduced Antoine ln(Psat/Pc) = a1 - a2/(a3 + T), T in K",
            },
            "generated_by": "scripts/build_components_json.py (do not edit by hand)",
        },
        "compounds": compounds,
    }


def main() -> None:
    db = build()
    for rel in ("python/src/vle/data/components.json", "notebooks/data/components.json"):
        path = REPO / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(db, indent=2) + "\n")
        print(f"wrote {path} ({len(db['compounds'])} compounds)")


if __name__ == "__main__":
    main()
