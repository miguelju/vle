"""Cross-implementation parity test.

Verifies that the Python ``vle.units`` wrapper produces the same canonical
values as the Rust ``vle_units`` crate for a fixed table of golden inputs.
The Rust expected values come from the integration tests in
``units/tests/conversions.rs`` (and were spot-checked against published
unit-conversion tables).
"""
from __future__ import annotations

import math
import sys

import pytest

# tomllib is stdlib on 3.11+; on 3.10 we fall back to the `tomli`
# backport (declared as a conditional dep in pyproject.toml).
if sys.version_info >= (3, 11):
    import tomllib
else:  # pragma: no cover — only exercised on 3.10
    import tomli as tomllib

from vle._engine import default_units_toml
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


# ── TOML-driven drift check ─────────────────────────────────────────────────
#
# The hand-curated GOLDEN table above catches arithmetic errors; the test below
# catches *missing rows* — any unit added to ``units/src/data/defaults.toml``
# without a matching pint mapping on the Python side fails immediately.

# Translation table: TOML name → pint-parseable name.
# • Exponent notation: TOML keeps the legacy ``^`` from VB6/Pascal; pint expects ``**``.
# • Δ-temperatures in Kelvin/Rankine: pint treats these as non-affine and uses
#   the absolute-unit name (``kelvin`` / ``rankine``) for differences too.
# • BTU/(lb-mol·R) variants: legacy Rust uses thermochemical-cal-based values
#   (2.326, 4.184); pint's native conversion uses IT BTU. We route through the
#   Python aliases registered in ``vle.units`` from the same TOML constants.
_TOML_TO_PINT: dict[str, str] = {
    "cm^3/mol": "cm**3/mol",
    "m^3/kmol": "m**3/kmol",
    "m^3/mol": "m**3/mol",
    "ft^3/lbmol": "ft**3/lbmol",
    "delta_K": "kelvin",
    "delta_degR": "rankine",
    "BTU/lbmol": "BTU_per_lbmol",
    "BTU/(lbmol*degR)": "BTU_per_lbmol_per_degR",
}

# TOML entries that knowingly disagree with pint's mathematically-correct
# answer. Add a row only with an explanatory comment — every entry here is a
# legacy-compatibility wart we are intentionally preserving.
_KNOWN_DIVERGENCES: dict[str, str] = {}


def _pint_name(toml_name: str) -> str:
    return _TOML_TO_PINT.get(toml_name, toml_name)


def test_every_unit_in_toml_matches_pint_conversion():
    catalog = tomllib.loads(default_units_toml())
    failures: list[str] = []
    for entry in catalog["unit"]:
        name = entry["name"]
        if name in _KNOWN_DIVERGENCES:
            continue
        dim = entry["dimension"]
        scale = float(entry["scale"])
        offset = float(entry.get("offset", 0.0))
        is_gauge = bool(entry.get("gauge", False))
        expected = 1.0 * scale + (101.325 if is_gauge else offset)
        try:
            actual = to_canonical(1.0, _pint_name(name), dim)
        except Exception as exc:
            failures.append(
                f"{name!r} ({dim}): pint raised {type(exc).__name__}: {exc}"
            )
            continue
        tol = max(abs(expected) * (1e-9 if is_gauge else 1e-12), 1e-12)
        if abs(actual - expected) > tol:
            failures.append(
                f"{name!r} ({dim}): pint→canonical = {actual!r}, "
                f"TOML expected = {expected!r}"
            )
    assert not failures, "Pint vs. TOML parity failures:\n  " + "\n  ".join(failures)


def test_gauge_units_track_atmospheric_pressure_changes():
    """A change to P_atm must shift every gauge conversion by the same delta."""
    set_atmospheric_pressure(101.325)
    assert math.isclose(to_canonical(0.0, "barg", "pressure"), 101.325, abs_tol=1e-9)

    set_atmospheric_pressure(84.5)  # ~1500 m elevation
    assert math.isclose(to_canonical(0.0, "barg", "pressure"), 84.5, abs_tol=1e-9)


def test_lbmol_and_btu_per_lbmol_scales_match_toml():
    """The TOML-derived `_define_if_missing` calls in vle.units must match catalog values."""
    by_name = {e["name"]: e for e in tomllib.loads(default_units_toml())["unit"]}
    assert to_canonical(1.0, "lbmol", "amount") == pytest.approx(
        by_name["lbmol"]["scale"], rel=1e-15
    )
    assert to_canonical(1.0, "BTU_per_lbmol", "molar_energy") == pytest.approx(
        by_name["BTU/lbmol"]["scale"], rel=1e-15
    )
