"""Tests for the bundled JSON component database (:mod:`vle.components`)."""

import math

import pytest

from vle import components
from vle.components import Component


def test_available_lists_known_compounds():
    names = components.available()
    assert "water" in names
    assert "benzene" in names
    assert "n-butane" in names
    # Sorted and de-duplicated.
    assert names == sorted(set(names))


def test_get_returns_component_with_canonical_units():
    water = components.get("water")
    assert isinstance(water, Component)
    assert water.name == "water"
    assert water.formula == "H2O"
    assert water.tc == pytest.approx(647.096)  # K
    assert water.pc == pytest.approx(22064.0)  # kPa
    assert 0.0 < water.omega < 1.0


def test_get_is_case_insensitive_and_strips():
    a = components.get("N-Butane")
    b = components.get("  n-butane ")
    assert a.name == b.name == "n-butane"


def test_get_unknown_raises_keyerror_listing_options():
    with pytest.raises(KeyError) as exc:
        components.get("unobtainium")
    # The message should help the user by listing what *is* available.
    assert "water" in str(exc.value)


def test_reduced_antoine_reproduces_boiling_point():
    """Psat(Tb) ≈ 1 atm for a component with a shipped Antoine set."""
    benzene = components.get("benzene")
    a1, a2, a3 = benzene.psat_coeffs
    psat_tb = benzene.pc * math.exp(a1 - a2 / (a3 + benzene.tb))  # kPa
    assert psat_tb == pytest.approx(101.325, rel=0.05)


def test_search_matches_name_formula_cas():
    hits = {c.name for c in components.search("butane")}
    assert "n-butane" in hits
    # Formula search.
    assert any(c.formula == "CO2" for c in components.search("CO2"))


def test_all_components_roundtrip_count():
    assert len(components.all_components()) == len(components.available())
