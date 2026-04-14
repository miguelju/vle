"""Tests for ``vle.units`` — parity with the Rust ``vle_units`` crate.

The same conversions are exercised on both sides; expected canonical values
are golden constants verified to match the Rust integration tests.
"""
from __future__ import annotations

import math

import pytest

from vle.units import (
    Q_,
    from_canonical,
    get_atmospheric_pressure,
    parse,
    set_atmospheric_pressure,
    to_canonical,
    ureg,
)

EPS = 1e-9


@pytest.fixture(autouse=True)
def _reset_atm():
    """Reset P_atm before every test so leakage across tests is impossible."""
    set_atmospheric_pressure(101.325)
    yield
    set_atmospheric_pressure(101.325)


# ── Temperature ─────────────────────────────────────────────────────────────


def test_temperature_known_values():
    assert math.isclose(to_canonical(25.0, "degC", "temperature"), 298.15, abs_tol=EPS)
    assert math.isclose(to_canonical(32.0, "degF", "temperature"), 273.15, abs_tol=EPS)
    # -40 °C == -40 °F
    assert math.isclose(
        to_canonical(-40.0, "degC", "temperature"),
        to_canonical(-40.0, "degF", "temperature"),
        abs_tol=EPS,
    )


@pytest.mark.parametrize("unit", ["kelvin", "degC", "degF", "rankine"])
@pytest.mark.parametrize("v", [200.0, 273.15, 300.0, 500.0])
def test_temperature_round_trip(unit, v):
    canonical = to_canonical(v, unit, "temperature")
    assert canonical > 0, f"{unit}={v} → K must be > 0 (got {canonical})"
    back = from_canonical(canonical, unit, "temperature")
    assert math.isclose(back, v, abs_tol=1e-9, rel_tol=1e-12)


# ── Pressure (absolute) ─────────────────────────────────────────────────────


@pytest.mark.parametrize(
    "unit,expected_kpa",
    [
        ("Pa", 0.001),
        ("kPa", 1.0),
        ("MPa", 1000.0),
        ("bar", 100.0),
        ("atm", 101.325),
        ("psi", 6.89475729316836),
        ("mmHg", 0.133322387415),
    ],
)
def test_pressure_known_values(unit, expected_kpa):
    assert math.isclose(to_canonical(1.0, unit, "pressure"), expected_kpa, rel_tol=1e-6)


# ── Gauge pressure with configurable P_atm ──────────────────────────────────


def test_gauge_default_atm():
    # 2.5 barg @ standard atm = 351.325 kPa absolute
    assert math.isclose(to_canonical(2.5, "barg", "pressure"), 351.325, abs_tol=1e-6)


def test_gauge_custom_atm():
    set_atmospheric_pressure(84.5)  # plant at altitude
    assert get_atmospheric_pressure() == 84.5
    assert math.isclose(to_canonical(2.5, "barg", "pressure"), 334.5, abs_tol=1e-6)
    assert math.isclose(
        to_canonical(10.0, "psig", "pressure"),
        10.0 * 6.89475729316836 + 84.5,
        abs_tol=1e-6,
    )
    assert math.isclose(to_canonical(50.0, "kPag", "pressure"), 134.5, abs_tol=1e-6)


def test_gauge_round_trip():
    for unit in ["barg", "psig", "kPag"]:
        for v in [0.0, 1.0, 50.0, 200.0]:
            canonical = to_canonical(v, unit, "pressure")
            back = from_canonical(canonical, unit, "pressure")
            assert math.isclose(back, v, abs_tol=1e-9)


def test_gauge_negative_within_vacuum_range():
    # -50 kPag = +51.325 kPa absolute (mild vacuum, allowed)
    assert math.isclose(to_canonical(-50.0, "kPag", "pressure"), 51.325, abs_tol=1e-6)


def test_set_atmospheric_pressure_rejects_non_positive():
    with pytest.raises(ValueError):
        set_atmospheric_pressure(0.0)
    with pytest.raises(ValueError):
        set_atmospheric_pressure(-5.0)


# ── Other quantities ────────────────────────────────────────────────────────


def test_molar_energy():
    # 1 cal/mol = 4.184 J/mol = 4.184 kJ/kmol
    assert math.isclose(to_canonical(1.0, "cal/mol", "molar_energy"), 4.184, abs_tol=EPS)
    assert math.isclose(to_canonical(1.0, "kJ/kmol", "molar_energy"), 1.0, abs_tol=EPS)


def test_molar_entropy():
    assert math.isclose(
        to_canonical(1.0, "cal/(mol*kelvin)", "molar_entropy"), 4.184, abs_tol=EPS
    )


def test_molar_volume_and_amount():
    assert math.isclose(to_canonical(1.0, "L/mol", "molar_volume"), 1000.0, abs_tol=EPS)
    assert math.isclose(to_canonical(1.0, "mol", "amount"), 1e-3, abs_tol=EPS)


def test_temperature_diff_is_scale_only():
    # Δ°C → ΔK is identity. Crucially: NO offset.
    assert math.isclose(
        to_canonical(60.0, "delta_degC", "temperature_diff"), 60.0, abs_tol=EPS
    )
    # 60 Δ°C = 108 Δ°F
    assert math.isclose(
        to_canonical(108.0, "delta_degF", "temperature_diff"), 60.0, abs_tol=1e-9
    )


# ── Parser & R constant ─────────────────────────────────────────────────────


def test_parse_string_form():
    assert math.isclose(parse("25 degC", "temperature"), 298.15, abs_tol=EPS)
    assert math.isclose(parse("3.5 barg", "pressure"), 451.325, abs_tol=1e-6)


def test_r_constant_round_trip():
    R_kJ_per_kmolK = 8.31451
    canonical = to_canonical(R_kJ_per_kmolK, "kJ/(kmol*kelvin)", "molar_entropy")
    assert math.isclose(canonical, R_kJ_per_kmolK, abs_tol=EPS)
    back = from_canonical(canonical, "J/(mol*kelvin)", "molar_entropy")
    assert math.isclose(back, R_kJ_per_kmolK, abs_tol=EPS)


# ── Dimension safety (runtime) ──────────────────────────────────────────────


def test_dimension_mismatch_raises():
    t = Q_(300, "kelvin")
    p = Q_(101.325, "kPa")
    with pytest.raises(Exception):
        # pint raises DimensionalityError; we catch broadly to avoid
        # importing pint internals into the test
        _ = t + p


def test_absolute_temperatures_cannot_be_added():
    t1 = Q_(25, "degC")
    t2 = Q_(85, "degC")
    with pytest.raises(Exception):
        _ = t1 + t2


def test_temperature_subtraction_yields_delta():
    t1 = Q_(25, "degC")
    t2 = Q_(85, "degC")
    dt = t2 - t1
    # 60 Δ°C → 108 Δ°F (scale-only; no offset)
    assert math.isclose(dt.to("delta_degF").magnitude, 108.0, abs_tol=1e-9)


# ── Custom user units (Scenario A & B) ──────────────────────────────────────


def test_custom_unit_within_existing_dimension():
    if "mmH2O" not in ureg:
        ureg.define("mmH2O = 9.80665 * pascal")
    # 1000 mmH2O = 9806.65 Pa = 9.80665 kPa
    canonical = to_canonical(1000.0, "mmH2O", "pressure")
    assert math.isclose(canonical, 9.80665, abs_tol=1e-9)


def test_custom_derived_quantity_via_pint():
    # pint composes derived units automatically; no registration needed
    h = 150 * ureg.watt / (ureg.meter**2 * ureg.kelvin)
    converted = h.to("BTU / (hour * foot**2 * degR)")
    # Expected ≈ 26.42 (USU-style HTC conversion factor)
    assert 25 < converted.magnitude < 28
