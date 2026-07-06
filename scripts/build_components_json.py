#!/usr/bin/env python
"""Generate the bundled JSON component database (Milestone 10; extended M12.1,
M12.2).

Writes the SAME content to THREE places (single source of truth = this script):

- ``engine/data/components.json``         — the canonical copy embedded into
  the Rust crate via ``include_str!`` (M12.2, ``engine/src/db.rs``). It lives
  inside the crate directory so ``cargo package`` ships it to crates.io.
- ``python/src/vle/data/components.json`` — shipped inside the wheel, loaded
  by :mod:`vle.components` via ``importlib.resources``.
- ``notebooks/data/components.json``      — the copy the notebooks read, so a
  notebook works even without the package data (e.g. on a hub that mounts
  only the notebooks directory).

The engine and wheel copies are asserted byte-identical by
``python/tests/test_rust_db.py`` (a cheap drift guard), so they must never be
hand-edited — only regenerated from this script.

Run from the repo root with the ``vle`` conda env::

    ~/miniconda3/envs/vle/bin/python scripts/build_components_json.py

Data provenance
---------------
- ``tc``/``pc``/``omega``/``zc``/``vc``/``tb``/``mw``: DIPPR-backed values
  from the CalebBell/thermo library. The original 15 compounds were extracted
  2026-04-05 (matching ``python/src/vle/db/sql/seed_chapter4.sql``); the 9
  compounds added in Milestone 12.1 were sourced 2026-07-05 from
  ``thermo`` 0.6.0 / ``chemicals`` 1.5.2 and cross-checked against Poling,
  Prausnitz & O'Connell 5th ed. (reference (30)).
- ``psat_coeffs``: the engine's reduced Antoine form
  ``ln(Psat/Pc) = a1 − a2/(a3 + T)`` (T in K, P in kPa), converted EXACTLY
  from published Antoine constants ``log10(P[bar]) = A − B/(T + C)`` via::

      a1 = ln(10)·A + ln(100) − ln(Pc[kPa]);  a2 = ln(10)·B;  a3 = C

  The original 15 use NIST-WebBook constants; the M12.1 additions use the
  ``chemicals`` Antoine-Poling table (published as ``log10(P[Pa])``; the bar
  form used here is ``A_bar = A_Pa − 5``). For CO2 and H2S (no 1-atm liquid
  boiling point — CO2 sublimes) the coefficients are a two-point fit through
  documented (T, Psat) anchors with a3 = 0.
- ``cp_coeffs`` (M12.1): the **dimensionless ideal-gas Cp°/R polynomial in
  T [K]** matching the engine convention ``Cp°(T) = R·Σₖ aₖ·Tᵏ`` exactly
  (``engine/src/energy.rs`` ``ideal_cp``, R = 8.31451 kJ/(kmol·K)). Each
  compound's 5 coefficients are a degree-4 least-squares fit of the Poling
  ``POLING_POLY`` ideal-gas heat capacity (reference (30), via
  ``chemicals``) over ``cp_t_range``. Because POLING_POLY is itself a
  quartic, the fit reproduces it to machine precision; the per-compound
  Cp°(298.15 K) is pinned in ``python/tests/test_components_cp.py``.
- ``liquid_volume``: molar volume of the saturated liquid near 298 K in
  cm³/mol (CRC Handbook / ``thermo`` ``VolumeLiquid``) — used by
  Wilson / Poynting.

Every Antoine-derived entry is sanity-checked: Psat(Tb) must reproduce
101.325 kPa within 5% (extrapolation outside the stated range is allowed but
the boiling point must stay honest).
"""

from __future__ import annotations

import json
import math
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

LN10 = math.log(10.0)
LN100 = math.log(100.0)


def reduced_antoine(a: float, b: float, c: float, pc_kpa: float) -> list[float]:
    """Convert Antoine (log10, bar, K) to reduced form (ln, Psat/Pc, K)."""
    return [LN10 * a + LN100 - math.log(pc_kpa), LN10 * b, c]


def two_point_fit(pc_kpa: float, t1: float, p1: float, t2: float, p2: float) -> list[float]:
    """Fit ``ln(P/Pc) = a1 − a2/T`` (a3 = 0) through two (T [K], Psat [kPa]) anchors."""
    y1 = math.log(p1 / pc_kpa)
    y2 = math.log(p2 / pc_kpa)
    a2 = (y2 - y1) / (1.0 / t1 - 1.0 / t2)
    a1 = y1 + a2 / t1
    return [a1, a2, 0.0]


# name: (formula, cas, mw, tc [K], pc [kPa], omega, zc, vc [cm3/mol], tb [K],
#        Antoine (A, B, C) [log10, bar] or None, liquid_volume [cm3/mol] or None)
#
# The original 15 Chapter IV compounds (NIST-WebBook Antoine, bar form).
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

# Milestone 12.1 additions (9 compounds → 24 total): the classic
# distillation / absorber validation set the downstream stages-thermo library
# needs. Sourced 2026-07-05 from thermo 0.6.0 / chemicals 1.5.2; Antoine from
# the chemicals Antoine-Poling table (bar form = published Pa form − 5).
RAW_NEW = {
    "toluene": ("C7H8", "108-88-3", 92.1384, 591.750, 4126.30, 0.2657, 0.26465, 315.557, 383.746,
                (4.05043, 1327.62, -55.525), 106.86),
    "ethanol": ("C2H6O", "64-17-5", 46.0684, 514.710, 6268.00, 0.6460, 0.24699, 168.634, 351.570,
                (5.33675, 1648.22, -42.232), 58.68),
    "acetone": ("C3H6O", "67-64-1", 58.0791, 508.100, 4692.40, 0.3071, 0.23633, 212.766, 329.225,
                (4.21840, 1197.01, -45.09), 74.01),
    "chloroform": ("CHCl3", "67-66-3", 119.3776, 536.200, 5330.00, 0.2160, 0.29100, 244.000, 334.350,
                   (3.96288, 1106.904, -54.598), 80.5),
    "isobutane": ("C4H10", "75-28-5", 58.1222, 407.810, 3629.00, 0.1840, 0.27586, 257.748, 261.401,
                  (4.00272, 947.54, -24.28), 105.55),
    "isopentane": ("C5H12", "78-78-4", 72.1488, 460.350, 3378.00, 0.2274, 0.26981, 305.717, 300.976,
                   (3.92023, 1022.88, -39.69), 117.32),
    "n-octane": ("C8H18", "111-65-9", 114.2285, 568.740, 2483.59, 0.3980, 0.25860, 492.368, 398.794,
                 (4.05075, 1356.36, -63.515), 163.52),
    "n-nonane": ("C9H20", "111-84-2", 128.2551, 594.550, 2281.00, 0.4433, 0.25493, 552.486, 423.913,
                 (4.07356, 1438.03, -70.456), 179.61),
    "n-decane": ("C10H22", "124-18-5", 142.2817, 617.700, 2103.00, 0.4884, 0.24968, 609.756, 447.270,
                 (4.06853, 1495.17, -79.292), 195.84),
}

# Two-point (T [K], Psat [kPa]) anchors for the compounds without a usable
# liquid-range Antoine set (CO2 sublimes at 1 atm; H2S's set is spotty).
# Anchors: CRC Handbook saturated-vapor-pressure tables.
TWO_POINT = {
    "carbon dioxide": ((273.15, 3485.1), (298.15, 6434.0)),
    "hydrogen sulfide": ((212.855, 101.325), (273.15, 1030.0)),
}

# Ideal-gas heat capacity: dimensionless Cp°/R = Σₖ aₖ·Tᵏ (k = 0..4, T in K),
# the exact engine convention (engine/src/energy.rs `ideal_cp`). Each row is a
# degree-4 fit of the Poling POLING_POLY correlation (ref (30), via chemicals)
# over cp_t_range; POLING_POLY is a quartic so the fit is machine-exact.
# Regenerate with scratchpad sourcing when adding compounds. Do NOT round these
# (a4 ≈ 1e-11 would vanish); psat_coeffs rounding does not apply to Cp.
# value: (coeffs[a0..a4], [t_min, t_max] K)
CP = {
    "methane": ([4.567973968366664, -0.008974948854223229, 3.630979308042808e-05, -3.40698058455023e-08, 1.0909937827250705e-11], [200.0, 1000.0]),
    "ethane": ([4.1779761908572635, -0.0044269747718827974, 5.659967745393061e-05, -6.650962097987483e-08, 2.4869858273485e-11], [200.0, 1000.0]),
    "propane": ([3.846978077124928, 0.0051309707600015655, 6.010965745151554e-05, -7.89295502020979e-08, 3.078982453721771e-11], [200.0, 1000.0]),
    "n-butane": ([5.546968389345415, 0.005535968452030655, 8.056954085623982e-05, -1.0570939759107725e-07, 4.133976441599786e-11], [200.0, 1000.0]),
    "n-pentane": ([7.55395695206698, -0.0003679979028814063, 0.00011845932493273191, -1.4938914867213207e-07, 5.752967215414526e-11], [200.0, 1000.0]),
    "carbon dioxide": ([3.258981427956865, 0.0013559922725709835, 1.5019914405619353e-05, -2.3739864713009296e-08, 1.0559939821793491e-11], [200.0, 1000.0]),
    "hydrogen sulfide": ([4.265975689372168, -0.003437980407890703, 1.3189924834228668e-05, -1.3309924150385436e-08, 4.879972190374233e-12], [200.0, 1000.0]),
    "benzene": ([3.5509797639383125, -0.0061839647592780355, 0.00014364918138263418, -1.9806887125971656e-07, 8.233953076955149e-11], [200.0, 1000.0]),
    "cyclohexane": ([4.034977005770514, -0.004432974737690908, 0.00016833904068188385, -2.077488160963606e-07, 7.74595585791775e-11], [200.0, 1000.0]),
    "methylcyclohexane": ([3.1479820605119238, 0.01843789492748277, 0.00013623922360995544, -1.8792892904447214e-07, 7.363958034818791e-11], [200.0, 1000.0]),
    "n-hexane": ([8.83094967483494, -0.000165999054017394, 0.00014301918497281133, -1.831389563412154e-07, 7.123959402505293e-11], [200.0, 1000.0]),
    "n-heptane": ([9.633945098783862, 0.0041559763162275, 0.00015493911704438188, -2.0065885650010049e-07, 7.769955721149113e-11], [200.0, 1000.0]),
    "methanol": ([4.713973136357351, -0.006985960188925149, 4.210976002800361e-05, -4.442974680703384e-08, 1.534991252504992e-11], [200.0, 1000.0]),
    "water": ([4.394974954240682, -0.004185976145267869, 1.4049919933352348e-05, -1.563991087242916e-08, 6.319963984255215e-12], [200.0, 1000.0]),
    "2-propanol": ([3.333981000554831, 0.018852892562525196, 3.6439792339597634e-05, -6.114965152487325e-08, 2.5429855082216247e-11], [200.0, 1000.0]),
    "toluene": ([3.865977968849799, 0.0035579797240465165, 0.00013355923888245565, -1.8658893668072264e-07, 7.689956177044637e-11], [200.0, 1000.0]),
    "ethanol": ([4.3959749485420065, 0.0006279964212199487, 5.545968395044154e-05, -7.023959972374689e-08, 2.6849846990071243e-11], [200.0, 1000.0]),
    "acetone": ([5.125970788495517, 0.0015109913892733377, 5.730967340785786e-05, -7.176959100474516e-08, 2.7279844539632807e-11], [200.0, 1000.0]),
    "chloroform": ([2.388986385820476, 0.026217850591645313, -3.144982077607901e-05, 1.8569894175255358e-08, -4.229975894524996e-12], [200.0, 1000.0]),
    "isobutane": ([3.350980903677012, 0.017882898090258392, 5.476968788253942e-05, -8.099953840580042e-08, 3.242981519135933e-11], [200.0, 1000.0]),
    "isopentane": ([1.9589888362587693, 0.03819078236118458, 2.433986129379175e-05, -5.174970509259413e-08, 2.1649876623278537e-11], [200.0, 1000.0]),
    "n-octane": ([10.823938317338229, 0.004982971603407559, 0.00017750898842486308, -2.3136868149321465e-07, 8.979948825729673e-11], [200.0, 1000.0]),
    "n-nonane": ([12.151930749472886, 0.004574973928474323, 0.00020415883655467348, -2.677684740607602e-07, 1.0464940363169358e-10], [200.0, 1000.0]),
    "n-decane": ([13.466923255690485, 0.00413897641310502, 0.00023126868206308312, -3.047682632090871e-07, 1.1969931786635066e-10], [200.0, 1000.0]),
}

CP_SOURCE = (
    "Poling, Prausnitz & O'Connell 5th ed. (30) ideal-gas Cp°/R (POLING_POLY "
    "via CalebBell/chemicals); degree-4 fit in T [K] over cp_t_range"
)


def build() -> dict:
    compounds = {}
    # The 15 originals first (stable order), then the 9 M12.1 additions.
    for name, row in {**RAW, **RAW_NEW}.items():
        formula, cas, mw, tc, pc, omega, zc, vc, tb, antoine, vliq = row
        is_new = name in RAW_NEW
        if antoine is not None:
            coeffs = reduced_antoine(*antoine, pc)
            if is_new:
                psat_source = (
                    f"chemicals Antoine-Poling (log10, bar) A={antoine[0]}, "
                    f"B={antoine[1]}, C={antoine[2]}, converted exactly"
                )
            else:
                psat_source = (
                    f"NIST WebBook Antoine (log10, bar) A={antoine[0]}, "
                    f"B={antoine[1]}, C={antoine[2]}, converted exactly"
                )
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
        # Ideal-gas Cp°/R polynomial (M12.1) — full precision, never rounded.
        cp_coeffs, cp_t_range = CP[name]
        entry["cp_coeffs"] = cp_coeffs
        entry["cp_t_range"] = cp_t_range
        entry["cp_source"] = CP_SOURCE
        if vliq is not None:
            entry["liquid_volume"] = vliq
        compounds[name] = entry
    return {
        "_meta": {
            "description": "vle-thermo bundled component database (Milestone 10; Cp added M12.1)",
            "units": {
                "tc": "K", "pc": "kPa (absolute)", "vc": "cm3/mol", "tb": "K",
                "mw": "g/mol", "liquid_volume": "cm3/mol at ~298 K",
                "psat_coeffs": "reduced Antoine ln(Psat/Pc) = a1 - a2/(a3 + T), T in K",
                "cp_coeffs": "dimensionless ideal-gas Cp/R = sum a_k*T^k (k=0..4), T in K",
                "cp_t_range": "[t_min, t_max] K validity window of the Cp fit",
            },
            "generated_by": "scripts/build_components_json.py (do not edit by hand)",
        },
        "compounds": compounds,
    }


def main() -> None:
    db = build()
    for rel in (
        "engine/data/components.json",
        "python/src/vle/data/components.json",
        "notebooks/data/components.json",
    ):
        path = REPO / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(db, indent=2) + "\n")
        print(f"wrote {path} ({len(db['compounds'])} compounds)")


if __name__ == "__main__":
    main()
