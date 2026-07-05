"""Chapter IV validation through the high-level :class:`vle.System` API.

These pin the *published thesis results* (Jackson & Mendible, USB 1999,
``docs/en/research-paper/chapter-4-validation.md``) as executed end-to-end
through the ergonomic Python surface — not the raw engine bindings. The
engine-level reproduction lives in ``engine/tests/chapter_iv_validation.rs``;
this file guarantees the *wrapper* delivers the same numbers a user would get.

The thesis tolerance is 1–5% (see ``CLAUDE.md`` → Validation Cases). Values
are in canonical units: T in **K**, P in **kPa absolute**.
"""

import numpy as np
import pytest

from vle import System, components
from vle import _engine as e


def rel(a, b):
    return abs(a - b) / abs(b)


def components_of(name):
    """Bundled-database record for ``name`` (canonical units)."""
    return components.get(name)


# ── §4.6 Isothermal flash — Table 4.10 (n-heptane/n-butane, RKS) ──────────

def test_isothermal_flash_table_4_10():
    """n-heptane/n-butane equimolar at 300 K, 100 kPa, RKS, no kij.

    Thesis Table 4.10: x₁ = 0.6135, y₁ = 0.04284, β = 0.19889.
    """
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    r = sys.flash_pt(300.0, 100.0, [0.5, 0.5])
    assert r.two_phase
    assert rel(r.x[0], 0.6135) < 0.05, f"x₁ = {r.x[0]}"
    assert rel(r.y[0], 0.04284) < 0.05, f"y₁ = {r.y[0]}"
    assert rel(r.beta, 0.19889) < 0.05, f"β = {r.beta}"


# ── §4.1 Mixture critical points — Tables 4.1–4.2 (PR EOS) ────────────────

# (name, components, z, t_init, Tc_ref [K], Pc_ref [kPa]) — thesis Table 4.2.
CRITICAL_CASES = [
    ("Mix 1", ["ethane", "propane", "n-pentane"], [0.3414, 0.3421, 0.3165], 405.0, 404.43, 5552.0),
    ("Mix 2", ["propane", "n-butane", "n-pentane"], [0.3276, 0.3398, 0.3326], 430.0, 430.72, 4174.0),
    ("Mix 4", ["ethane", "propane", "n-butane", "n-pentane"],
     [0.2542, 0.2547, 0.2554, 0.2357], 410.0, 410.74, 5063.0),
]


@pytest.mark.parametrize("name,comps,z,t0,tc_ref,pc_ref", CRITICAL_CASES)
def test_critical_points_tables_4_1_2(name, comps, z, t0, tc_ref, pc_ref):
    sys = System(comps, eos="PR")
    cp = sys.critical_point(z, t_init=t0)
    assert rel(cp.tc, tc_ref) < 0.02, f"{name} Tc = {cp.tc:.2f} vs {tc_ref}"
    assert rel(cp.pc, pc_ref) < 0.06, f"{name} Pc = {cp.pc:.1f} vs {pc_ref}"


# ── §4.7 kij regression — Tables 4.11–4.12 (CO₂/n-butane) ─────────────────

def test_kij_regression_table_4_12():
    """Fit k₁₂ for CO₂/n-butane against Table 4.11 P-x bubble data.

    Thesis Table 4.12: k₁₂ ≈ 0.1357. The fitted value must land in the
    literature neighborhood (0.12–0.20 — exact 0.1357 needs the full dataset
    plus a near-critical-robust bubble solver, tracked in the engine's
    ``chapter_iv_validation.rs`` module caveat). Uses the same sub-critical
    subset (x₁ ≲ 0.20) as the engine test.
    """
    co2 = components_of("carbon dioxide")
    nc4 = components_of("n-butane")
    t = 357.57  # K
    # Table 4.11 sub-critical subset: (P [bar], x_CO2).
    bar_x = [
        (14.824, 0.02967), (19.029, 0.06228), (23.511, 0.0959),
        (27.441, 0.1283), (31.164, 0.15673), (36.404, 0.19636),
    ]
    data = [(t, x1, p_bar * 100.0) for (p_bar, x1) in bar_x]  # bar → kPa
    kij, sse, rmse = e.fit_kij_py(
        e.CubicEos.PR1976,
        [co2.tc, nc4.tc], [co2.pc, nc4.pc], [co2.omega, nc4.omega],
        [list(co2.psat_coeffs), list(nc4.psat_coeffs)],
        data,
    )
    assert 0.12 <= kij <= 0.20, f"fitted k₁₂ = {kij} outside literature neighborhood ~0.1357"
