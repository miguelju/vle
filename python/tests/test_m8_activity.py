"""M8.1 mixture layer — activity coefficients + liquid molar volume.

Exercises the bindings added in Milestone 8.1, all called through the built
wheel (``vle._engine``):

- ``liquid_molar_volume`` — Rackett (Spencer-Danner) and Thomson/COSTALD.
- ``activity_ln_gamma`` — the five models of research-paper Table 2.3.
- ``activity_excess_gibbs`` / ``_enthalpy`` / ``_entropy`` — eqs (2.44)–(2.46).

The activity formulas are pinned against the closed forms in Table 2.3 and,
for the analytical Wilson excess enthalpy, against a finite-difference oracle
of ``-T²·d(Gᴱ/T)/dT`` — the same test-oracle pattern the Rust unit tests use.
"""

import math

import pytest

import vle._engine as e

# Solubility parameters δ in (cal/cm³)^0.5 and liquid volumes in cm³/mol are
# representative textbook values; the regression checks only need internal
# consistency, not literature accuracy.
WATER = dict(tc=647.3, pc=22120.0, zra=0.235)
ETHANOL = dict(tc=513.9, pc=6148.0, zra=0.248)


# =============================================================================
# Liquid molar volume
# =============================================================================


def test_rackett_water_near_18_cm3() -> None:
    """Liquid water is ~18 cm³/mol near room temperature."""
    v = e.liquid_molar_volume(e.VolumeModel.Rackett, WATER["tc"], WATER["pc"],
                              298.15, zra=WATER["zra"])
    assert 15.0 < v < 20.0


def test_rackett_expands_with_temperature() -> None:
    lo = e.liquid_molar_volume(e.VolumeModel.Rackett, WATER["tc"], WATER["pc"],
                               298.15, zra=WATER["zra"])
    hi = e.liquid_molar_volume(e.VolumeModel.Rackett, WATER["tc"], WATER["pc"],
                               330.0, zra=WATER["zra"])
    assert hi > lo


def test_thomson_is_fraction_of_characteristic_volume() -> None:
    """At Tr≈0.6 the COSTALD reduced volume is ~0.39, so V ≈ 0.39·V*."""
    v = e.liquid_molar_volume(e.VolumeModel.Thomson, 500.0, 4000.0, 300.0,
                              vstar=100.0, omega_srk=0.0)
    assert 30.0 < v < 45.0


# =============================================================================
# Activity coefficients — Table 2.3 closed forms
# =============================================================================


def test_ideal_solution_unity() -> None:
    x = [0.4, 0.6]
    aij = [[0.0, 0.0], [0.0, 0.0]]
    for i in range(2):
        assert e.activity_ln_gamma(e.ActivityModel.IdealSolution, i, x, aij) == 0.0
    assert e.activity_excess_gibbs(e.ActivityModel.IdealSolution, x, aij) == 0.0


def test_margules_matches_closed_form() -> None:
    x = [0.3, 0.7]
    a12, a21 = 0.5, 0.8
    aij = [[0.0, a12], [a21, 0.0]]
    g1 = e.activity_ln_gamma(e.ActivityModel.Margules, 0, x, aij)
    g2 = e.activity_ln_gamma(e.ActivityModel.Margules, 1, x, aij)
    assert g1 == pytest.approx(x[1] ** 2 * (a12 + 2 * (a21 - a12) * x[0]))
    assert g2 == pytest.approx(x[0] ** 2 * (a21 + 2 * (a12 - a21) * x[1]))


def test_van_laar_matches_closed_form() -> None:
    x = [0.35, 0.65]
    a12, a21 = 1.2, 0.9
    aij = [[0.0, a12], [a21, 0.0]]
    g1 = e.activity_ln_gamma(e.ActivityModel.VanLaar, 0, x, aij)
    r1 = 1.0 + (a12 * x[0]) / (a21 * x[1])
    assert g1 == pytest.approx(a12 / r1**2)


def test_wilson_unity_for_zero_interaction() -> None:
    x = [0.5, 0.5]
    aij = [[0.0, 0.0], [0.0, 0.0]]
    vl = [40.0, 40.0]
    for i in range(2):
        g = e.activity_ln_gamma(e.ActivityModel.Wilson, i, x, aij, vl=vl, t=320.0)
        assert g == pytest.approx(0.0, abs=1e-12)


def test_scatchard_unity_when_deltas_equal() -> None:
    x = [0.3, 0.7]
    vl = [75.0, 110.0]
    delta = [9.0, 9.0]
    for i in range(2):
        g = e.activity_ln_gamma(e.ActivityModel.ScatchardHildebrand, i, x,
                                [[0.0, 0.0], [0.0, 0.0]], vl=vl, delta=delta, t=300.0)
        assert g == pytest.approx(0.0, abs=1e-12)


def test_positive_deviation_raises_activity() -> None:
    """Van Laar with positive A gives γ>1 (positive deviation from Raoult)."""
    x = [0.5, 0.5]
    aij = [[0.0, 1.0], [1.2, 0.0]]
    g0 = e.activity_ln_gamma(e.ActivityModel.VanLaar, 0, x, aij)
    assert g0 > 0.0


# =============================================================================
# Excess properties (eqs 2.44–2.46)
# =============================================================================


def test_excess_gibbs_vanishes_at_pure_limits() -> None:
    aij = [[0.0, 800.0], [1200.0, 0.0]]
    vl = [58.0, 92.0]
    delta = [7.4, 9.2]
    for model in (e.ActivityModel.Margules, e.ActivityModel.VanLaar,
                  e.ActivityModel.Wilson, e.ActivityModel.ScatchardHildebrand):
        for x in ([1.0 - 1e-9, 1e-9], [1e-9, 1.0 - 1e-9]):
            ge = e.activity_excess_gibbs(model, x, aij, vl=vl, delta=delta, t=313.15)
            assert abs(ge) < 1e-1


def test_wilson_excess_enthalpy_matches_numerical_oracle() -> None:
    """Analytical Hᴱ vs central-difference of −T²·d(Gᴱ/T)/dT."""
    x = [0.4, 0.6]
    aij = [[0.0, 1500.0], [2600.0, 0.0]]   # λᵢⱼ − λᵢᵢ in kJ/kmol
    vl = [74.0, 18.0]
    t = 333.15
    h = 1e-2

    def g_over_t(tt: float) -> float:
        return e.activity_excess_gibbs(e.ActivityModel.Wilson, x, aij, vl=vl, t=tt) / tt

    he_num = -t * t * (g_over_t(t + h) - g_over_t(t - h)) / (2 * h)
    he_ana = e.activity_excess_enthalpy(e.ActivityModel.Wilson, x, aij, vl=vl, t=t)
    assert he_ana == pytest.approx(he_num, rel=1e-2)


def test_margules_van_laar_enthalpy_equals_gibbs() -> None:
    """Legacy convention: Hᴱ = Gᴱ, Sᴱ = 0 for Margules and van Laar."""
    x = [0.45, 0.55]
    aij = [[0.0, 0.7], [1.1, 0.0]]
    for model in (e.ActivityModel.Margules, e.ActivityModel.VanLaar):
        ge = e.activity_excess_gibbs(model, x, aij, t=298.15)
        he = e.activity_excess_enthalpy(model, x, aij, t=298.15)
        se = e.activity_excess_entropy(model, x, aij, t=298.15)
        assert he == pytest.approx(ge)
        assert se == pytest.approx(0.0, abs=1e-9)


def test_gibbs_duhem_consistency_binary() -> None:
    """At fixed T,P a binary must satisfy x₁ d(lnγ₁) + x₂ d(lnγ₂) = 0.

    Check it numerically for Wilson by finite-differencing ln γ in x₁.
    """
    aij = [[0.0, 1500.0], [2600.0, 0.0]]
    vl = [74.0, 18.0]
    t = 330.0
    x1 = 0.4
    h = 1e-6

    def lng(i: int, a: float) -> float:
        return e.activity_ln_gamma(e.ActivityModel.Wilson, i, [a, 1 - a], aij, vl=vl, t=t)

    d1 = (lng(0, x1 + h) - lng(0, x1 - h)) / (2 * h)
    d2 = (lng(1, x1 + h) - lng(1, x1 - h)) / (2 * h)
    assert x1 * d1 + (1 - x1) * d2 == pytest.approx(0.0, abs=1e-4)
