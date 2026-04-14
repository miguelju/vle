"""Cross-implementation parity test.

Verifies that the Python ``vle.units`` wrapper produces the same canonical
values as the Rust ``vle_units`` crate for a fixed table of golden inputs.
The Rust expected values come from the integration tests in
``units/tests/conversions.rs`` (and were spot-checked against published
unit-conversion tables).
"""
from __future__ import annotations

import math

import pytest

from vle.units import set_atmospheric_pressure, to_canonical


@pytest.fixture(autouse=True)
def _reset_atm():
    set_atmospheric_pressure(101.325)
    yield
    set_atmospheric_pressure(101.325)


# Golden table: (value, unit, quantity, expected_canonical)
GOLDEN = [
    # Temperature (K)
    (25.0, "degC", "temperature", 298.15),
    (32.0, "degF", "temperature", 273.15),
    (491.67, "rankine", "temperature", 273.15),
    # Pressure absolute (kPa)
    (1.0, "atm", "pressure", 101.325),
    (14.7, "psi", "pressure", 14.7 * 6.89475729316836),
    (1.0, "bar", "pressure", 100.0),
    # Pressure gauge @ standard atm (kPa abs)
    (2.5, "barg", "pressure", 351.325),
    (50.0, "kPag", "pressure", 151.325),
    # Molar energy (kJ/kmol)
    (1.0, "cal/mol", "molar_energy", 4.184),
    # Molar entropy (kJ/(kmol·K))
    (1.0, "cal/(mol*kelvin)", "molar_entropy", 4.184),
    # Volume / amount
    (1.0, "L/mol", "molar_volume", 1000.0),
    (1.0, "lbmol", "amount", 0.45359237),
    # Temperature differences (scale-only)
    (60.0, "delta_degC", "temperature_diff", 60.0),
    (108.0, "delta_degF", "temperature_diff", 60.0),
]


@pytest.mark.parametrize("value,unit,quantity,expected", GOLDEN)
def test_python_matches_rust_golden(value, unit, quantity, expected):
    got = to_canonical(value, unit, quantity)
    assert math.isclose(got, expected, rel_tol=1e-9, abs_tol=1e-9), (
        f"{value} {unit} → {got} (expected {expected})"
    )
