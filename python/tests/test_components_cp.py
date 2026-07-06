"""Milestone 12.1 — component-DB expansion + ideal-gas Cp coefficients.

Covers the G1/G2 downstream gaps:

* the DB grew from 15 to 24 compounds (the distillation/absorber set);
* every compound now carries a dimensionless ``cp_coeffs`` (Cp°/R polynomial)
  that is actually threaded through :class:`vle.System` into the engine, so a
  DB-built system's enthalpy includes the (usually dominant) ideal-gas part.

Units throughout: T in **K**, P in **kPa** absolute, molar enthalpy in
**kJ/kmol**. ``R_GAS`` = 8.31451 kJ/(kmol·K) matches ``engine::types::R_GAS``.
"""

import math

import pytest

from vle import components
from vle.components import Component
from vle.system import System

R_GAS = 8.31451  # kJ/(kmol·K) == J/(mol·K)

# The nine compounds added in M12.1 (15 → 24).
NEW_COMPOUNDS = [
    "toluene", "ethanol", "acetone", "chloroform", "isobutane",
    "isopentane", "n-octane", "n-nonane", "n-decane",
]

# Pinned ideal-gas Cp°(298.15 K) in **J/(mol·K)** (numerically == kJ/(kmol·K)),
# from the Poling POLING_POLY correlation (ref (30), via CalebBell/chemicals),
# the same source the cp_coeffs were fit to. Cross-checked against Poling's
# tabulated 298.15 K Cpg column. 1% tolerance guards a unit-conversion slip.
CP298_LIT = {
    "methane": 35.778, "ethane": 52.574, "propane": 73.762, "n-butane": 98.815,
    "n-pentane": 120.309, "carbon dioxide": 37.022, "hydrogen sulfide": 34.083,
    "benzene": 82.129, "cyclohexane": 106.289, "methylcyclohexane": 136.001,
    "n-hexane": 143.043, "n-heptane": 165.807, "methanol": 44.218,
    "water": 33.518, "2-propanol": 89.585, "toluene": 103.613, "ethanol": 65.383,
    "acetone": 74.700, "chloroform": 65.426, "isobutane": 96.955,
    "isopentane": 118.970, "n-octane": 188.461, "n-nonane": 211.142,
    "n-decane": 233.868,
}


def _cp_over_r(coeffs: list[float], t: float) -> float:
    """Dimensionless Cp°/R = Σₖ aₖ·Tᵏ (engine convention, T in K)."""
    return sum(a * t**k for k, a in enumerate(coeffs))


def _ideal_enthalpy_integral(coeffs: list[float], t: float, t_ref: float) -> float:
    """∫_{t_ref}^{t} Cp° dT = R·Σₖ aₖ·(T^{k+1} − Tref^{k+1})/(k+1), kJ/kmol."""
    return R_GAS * sum(
        a * (t ** (k + 1) - t_ref ** (k + 1)) / (k + 1) for k, a in enumerate(coeffs)
    )


# ── DB expansion (G1) ──────────────────────────────────────────────────────


def test_db_has_24_compounds_including_the_new_nine():
    names = components.available()
    assert len(names) == 24
    for nm in NEW_COMPOUNDS:
        assert nm in names, f"{nm} missing from the bundled DB"


def test_new_compound_properties_are_sane():
    """Spot-check the headline distillation binary partner, toluene."""
    tol = components.get("toluene")
    assert tol.formula == "C7H8"
    assert tol.tc == pytest.approx(591.75, rel=1e-3)  # K
    assert tol.pc == pytest.approx(4126.3, rel=1e-3)  # kPa
    assert 0.0 < tol.omega < 1.0


@pytest.mark.parametrize("name", sorted(CP298_LIT))
def test_psat_reproduces_boiling_point(name):
    """Psat(Tb) ≈ 1 atm — tight (1%) for the freshly-sourced M12.1 additions,
    5% for the legacy 15 (their original validation bound)."""
    if name == "carbon dioxide":
        pytest.skip("CO2 sublimes at 1 atm — no normal boiling point")
    c = components.get(name)
    if not c.psat_coeffs:
        pytest.skip(f"{name} has no vapor-pressure correlation")
    a1, a2, a3 = c.psat_coeffs
    psat_tb = c.pc * math.exp(a1 - a2 / (a3 + c.tb))  # kPa
    # The 9 new compounds were sourced to <0.7%; the legacy 15 to <5%.
    tol = 0.01 if name in NEW_COMPOUNDS else 0.05
    assert psat_tb == pytest.approx(101.325, rel=tol)


# ── Cp coefficients (G2) ───────────────────────────────────────────────────


@pytest.mark.parametrize("name", sorted(CP298_LIT))
def test_cp_coeffs_present_and_well_formed(name):
    c = components.get(name)
    assert len(c.cp_coeffs) == 5, f"{name} cp_coeffs must be a 5-term polynomial"
    assert len(c.cp_t_range) == 2 and c.cp_t_range[0] < c.cp_t_range[1]
    assert c.cp_source, f"{name} missing cp_source provenance"


@pytest.mark.parametrize("name,cp_lit", sorted(CP298_LIT.items()))
def test_cp_at_298_matches_literature(name, cp_lit):
    """Cp°(298.15 K) from the shipped polynomial within 1% of the pinned value.

    A failure here is almost always a *data* bug (Cp/R vs Cp, J vs cal, or a
    T-vs-T/1000 polynomial variable) — treat it as such before loosening tol.
    """
    c = components.get(name)
    cp = R_GAS * _cp_over_r(c.cp_coeffs, 298.15)  # kJ/(kmol·K) == J/(mol·K)
    assert cp == pytest.approx(cp_lit, rel=0.01)


# ── End-to-end: benzene–toluene, and the silent-zero-Cp fix ────────────────


def test_benzene_toluene_bubble_temperature_smoke():
    """The McCabe–Thiele teaching binary: x=0.5 bubble-T at 1 atm is sensible.

    Guards the new toluene Antoine data end-to-end through a PR flash. Benzene
    boils at ~353 K and toluene at ~384 K, so an equimolar bubble point must
    land between them.
    """
    sys = System(["benzene", "toluene"], eos="PR")
    res = sys.bubble_temperature([0.5, 0.5], 101.325)
    assert 355.0 < res.value < 370.0


def test_ideal_cp_is_threaded_into_the_engine():
    """Regression for the silent-zero ideal-Cp defect (G2).

    A DB-built system must now carry its ideal-gas Cp into the engine, so the
    vapor enthalpy at T ≠ t_ref is dominated by ∫Cp°dT. The same system built
    from bare critical constants (no Cp) sees only the small EOS departure —
    the two must differ by the ideal integral, and the DB system must match it.
    """
    benzene = components.get("benzene")
    t, p, t_ref = 400.0, 50.0, 298.15  # low P → departure is small

    sys_db = System(["benzene"], eos="PR")
    h_db, _ = sys_db.enthalpy_entropy(t, p, [1.0], "vapor")

    # Same component, but stripped of Cp data (from_arrays carries no cp_coeffs).
    sys_nocp = System.from_arrays(
        tcs=[benzene.tc], pcs=[benzene.pc], omegas=[benzene.omega],
        names=["benzene"], psat_coeffs=[list(benzene.psat_coeffs)],
    )
    h_nocp, _ = sys_nocp.enthalpy_entropy(t, p, [1.0], "vapor")

    ideal_integral = _ideal_enthalpy_integral(benzene.cp_coeffs, t, t_ref)

    # The DB system's enthalpy is the ideal integral plus a small departure.
    assert h_db == pytest.approx(ideal_integral, rel=0.02)
    # And it is emphatically nonzero / much larger than the Cp-less version.
    assert ideal_integral > 5000.0  # kJ/kmol — clearly nonzero
    assert abs(h_db - h_nocp) == pytest.approx(ideal_integral, rel=0.02)
