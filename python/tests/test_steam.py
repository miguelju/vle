"""Tests for the IAPWS-IF97 steam tables (``vle.steam``), through the wheel.

Ground truth is the R7-97(2012) computer-program verification tables; the
Python layer is checked for unit handling (pint quantities, gauge pressure),
mode dispatch, batch-vs-scalar agreement, and two-phase quality logic.
"""

from __future__ import annotations

import math

import numpy as np
import pytest

from vle import steam
from vle.units import Q_, set_atmospheric_pressure


# ── Region verification points (R7-97 Tables 5, 15, 33, 42) ───────────────


def test_region1_table5():
    st = steam.Water(T=300.0, P=3000.0)  # 3 MPa
    assert st.region == "1"
    assert st.phase == "liquid"
    assert st.v == pytest.approx(0.100215168e-2, rel=1e-8)
    assert st.h == pytest.approx(0.115331273e3, rel=1e-8)
    assert st.s == pytest.approx(0.392294792, rel=1e-8)
    assert st.w == pytest.approx(0.150773921e4, rel=1e-8)


def test_region2_table15():
    st = steam.Water(T=700.0, P=30000.0)  # 30 MPa
    assert st.h == pytest.approx(0.263149474e4, rel=1e-8)
    assert st.cp == pytest.approx(0.103505092e2, rel=1e-8)  # the cp that caught the bug


def test_region5_table42():
    st = steam.Water(T=1500.0, P=500.0)  # 0.5 MPa
    assert st.region == "5"
    assert st.h == pytest.approx(0.521976855e4, rel=1e-8)


# ── Saturation line (R7-97 Tables 35, 36) ─────────────────────────────────


def test_psat_tsat():
    assert steam.psat(300.0) == pytest.approx(3.53658941, rel=1e-8)  # kPa
    assert steam.tsat(100.0) == pytest.approx(372.755919, rel=1e-8)  # K


def test_saturation_row_atmospheric():
    sat = steam.saturation(P=101.325)  # 1 atm
    assert sat.t == pytest.approx(373.1243, rel=1e-5)
    # Handbook: h_fg ≈ 2256.5 kJ/kg, h_g ≈ 2675.6 kJ/kg at 1 atm.
    assert sat.h_fg == pytest.approx(2256.5, rel=1e-3)
    assert sat.h_g == pytest.approx(2675.6, rel=1e-3)


# ── Unit handling: pint quantities, strings, gauge pressure ───────────────


def test_string_and_quantity_inputs_agree():
    a = steam.Water(T="180 degC", x=1.0)
    b = steam.Water(T=180 + 273.15, x=1.0)
    assert a.p == pytest.approx(b.p, rel=1e-12)
    assert a.h == pytest.approx(b.h, rel=1e-12)


def test_gauge_pressure():
    # 10 barg at standard atm = 10 bar + 1.01325 bar ≈ 1101.325 kPa absolute.
    st = steam.Water(P="10 barg", x=0.0)
    assert st.p == pytest.approx(1101.325, rel=1e-9)


def test_gauge_pressure_respects_atmospheric_override():
    try:
        set_atmospheric_pressure(90.0)  # e.g. altitude
        st = steam.Water(P="1 barg", x=0.0)
        assert st.p == pytest.approx(100.0 + 90.0, rel=1e-9)
    finally:
        set_atmospheric_pressure(101.325)


# ── Mode dispatch & quality logic ─────────────────────────────────────────


def test_tx_two_phase_quality():
    st = steam.Water(T=400.0, x=0.5)
    assert st.phase == "two-phase"
    assert st.x == pytest.approx(0.5, rel=1e-12)
    sat = steam.saturation(T=400.0)
    assert st.h == pytest.approx(sat.h_f + 0.5 * sat.h_fg, rel=1e-12)


def test_ph_flash_two_phase():
    sat = steam.saturation(P=1000.0)
    hmid = 0.5 * (sat.h_f + sat.h_g)
    st = steam.Water(P=1000.0, h=hmid)
    assert st.phase == "two-phase"
    assert st.x == pytest.approx(0.5, rel=1e-9)


def test_ps_flash_round_trip_single_phase():
    ref = steam.Water(T=600.0, P=500.0)  # superheated vapor
    st = steam.Water(P=500.0, s=ref.s)
    assert st.t == pytest.approx(600.0, rel=1e-6)


def test_bad_argument_combo_raises():
    with pytest.raises(ValueError):
        steam.Water(T=300.0)  # only one property
    with pytest.raises(ValueError):
        steam.Water(h=100.0, s=1.0)  # unsupported pair


def test_out_of_range_raises():
    with pytest.raises(ValueError):
        steam.Water(T=200.0, P=1000.0)  # below the triple point


# ── Batch API: agreement with the scalar path ─────────────────────────────


def test_tp_batch_matches_scalar():
    ts = np.array([300.0, 500.0, 700.0, 1500.0])
    ps = np.array([3000.0, 3000.0, 30000.0, 500.0])
    res = steam.properties(ts, ps)
    for i, (t, p) in enumerate(zip(ts, ps)):
        st = steam.Water(T=float(t), P=float(p))
        assert res["h"][i] == pytest.approx(st.h, rel=1e-12)
        assert res["s"][i] == pytest.approx(st.s, rel=1e-12)
        assert res["v"][i] == pytest.approx(st.v, rel=1e-12)


def test_tp_batch_broadcasts_length_one():
    res = steam.properties(np.array([300.0, 400.0, 500.0]), np.array([5000.0]))
    assert res["h"].shape == (3,)
    for i, t in enumerate([300.0, 400.0, 500.0]):
        assert res["h"][i] == pytest.approx(steam.Water(T=t, P=5000.0).h, rel=1e-12)


def test_ph_batch_matches_scalar():
    ps = np.array([1000.0, 5000.0])
    hs = np.array([500.0, 3000.0])
    res = steam.ph_flash(ps, hs)
    for i, (p, h) in enumerate(zip(ps, hs)):
        st = steam.Water(P=float(p), h=float(h))
        assert res["t"][i] == pytest.approx(st.t, rel=1e-9)


def test_sat_table_matches_scalar():
    ts = np.array([300.0, 400.0, 500.0, 600.0])
    tab = steam.sat_table(ts)
    for i, t in enumerate(ts):
        sat = steam.saturation(T=float(t))
        assert tab["p"][i] == pytest.approx(sat.p, rel=1e-12)
        assert tab["h_fg"][i] == pytest.approx(sat.h_fg, rel=1e-12)


# ── Consistency: Clausius–Clapeyron through the Python surface ─────────────


def test_clausius_clapeyron():
    for t in (300.0, 450.0, 550.0):
        sat = steam.saturation(T=t)
        v_fg = sat.v_g - sat.v_f
        dpdt = steam.psat_derivative(t)  # kPa/K
        assert sat.h_fg == pytest.approx(t * v_fg * dpdt, rel=2e-3)


# ── M13.7: transport properties (IAPWS R12-08 / R15-11 / R1-76) ───────────


def test_viscosity_against_r12_08_table_4():
    """R12-08 Table 4 verification points, reached through the wheel.

    The release tabulates µ against (T, ρ); here we go in through (T, P) at
    states whose density lands on those points, so this exercises the whole
    Python → PyO3 → IF97 → R12-08 path rather than the correlation alone.
    """
    # 20 °C liquid water: the ISO reference value the release reproduces.
    assert steam.viscosity(293.15, 101.325) == pytest.approx(1.0016e-3, rel=2e-3)
    # Superheated steam at 873.15 K is dominated by the dilute-gas term.
    assert steam.viscosity(873.15, 100.0) == pytest.approx(32.65e-6, rel=5e-3)


def test_thermal_conductivity_and_derived_groups():
    k = steam.thermal_conductivity(293.15, 101.325)
    assert k == pytest.approx(0.5984, rel=2e-3)
    st = steam.Water(T=293.15, P=101.325)
    assert st.k == pytest.approx(k, rel=1e-12)
    # Pr = cp·µ/k, with cp in kJ/(kg·K) → J/(kg·K).
    assert st.pr == pytest.approx(st.cp * 1e3 * st.mu / st.k, rel=1e-12)
    assert st.nu == pytest.approx(st.mu / st.rho, rel=1e-12)
    assert st.alpha == pytest.approx(st.k / (st.rho * st.cp * 1e3), rel=1e-12)
    # Water at room temperature: Pr ≈ 7.
    assert st.pr == pytest.approx(7.0, rel=2e-2)


def test_surface_tension_table():
    # R1-76(2014) Table 1, "calculated" column, in mN/m.
    for t_c, expected in ((0.01, 75.65), (20.0, 72.74), (100.0, 58.91), (300.0, 14.36)):
        assert steam.surface_tension(t_c + 273.15) * 1e3 == pytest.approx(
            expected, abs=0.005
        )


def test_transport_units_and_gauge_pressure():
    """Unit strings and gauge pressure must work here as everywhere else."""
    mu_q = steam.viscosity(Q_(20, "degC"), Q_(1, "atm"))
    assert mu_q == pytest.approx(steam.viscosity(293.15, 101.325), rel=1e-12)
    # 0 barg is 1 atm absolute by the registry's default atmospheric pressure.
    assert steam.thermal_conductivity(Q_(20, "degC"), Q_(0, "barg")) == pytest.approx(
        steam.thermal_conductivity(293.15, 101.325), rel=1e-6
    )


def test_transport_is_per_phase_not_averaged():
    """A two-phase state must report NaN, and the row must carry both phases."""
    tsat = steam.tsat(101.325)
    st = steam.Water(T=tsat, P=101.325)
    assert math.isnan(st.mu)
    assert math.isnan(st.k)
    sat = steam.saturation(P=101.325)
    # Saturated liquid is ~23× more viscous and ~27× more conductive.
    assert sat.mu_f > 20 * sat.mu_g
    assert sat.k_f > 20 * sat.k_g
    assert sat.mu_g == pytest.approx(12.3e-6, rel=2e-2)
    assert sat.sigma == pytest.approx(58.9e-3, rel=1e-2)


def test_transport_batch_matches_scalar():
    ts = np.array([300.0, 400.0, 500.0, 800.0])
    tab = steam.transport(ts, 101.325)
    assert sorted(tab) == ["alpha", "k", "mu", "nu", "p", "pr", "t"]
    for i, t in enumerate(ts):
        assert tab["mu"][i] == pytest.approx(steam.viscosity(float(t), 101.325), rel=1e-12)
        assert tab["k"][i] == pytest.approx(
            steam.thermal_conductivity(float(t), 101.325), rel=1e-12
        )


def test_transport_batch_marks_two_phase_nan():
    tsat = steam.tsat(101.325)
    tab = steam.transport(np.array([300.0, tsat, 500.0]), 101.325)
    assert not math.isnan(tab["mu"][0])
    assert math.isnan(tab["mu"][1])
    assert not math.isnan(tab["mu"][2])
