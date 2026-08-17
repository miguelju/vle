"""Wheel-level tests for the M19 petroleum-characterization API.

Exercises the whole assay → pseudocomponents → flash path through the built
wheel, plus every `petro_*` correlation binding. The numerical validation of
the correlations themselves lives in the Rust unit tests (against published
worked examples and the bundled component database); what these tests check is
that the *binding layer* passes the right numbers in the right units, and that
the Python-side conveniences — unit strings, API gravity, `to_system` — behave.
"""

from __future__ import annotations

import math

import pytest

from vle import System, petroleum
from vle.components import Component
from vle.petroleum import Assay

# A light sweet crude, TBP basis, 35 °API. Reused across most tests.
CURVE_X = [0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95]
CURVE_T = [310.0, 380.0, 460.0, 540.0, 620.0, 730.0, 790.0]


def crude(**kw) -> Assay:
    kw.setdefault("api_gravity", 35.0)
    return Assay(fractions=CURVE_X, temperatures=CURVE_T, basis="tbp", **kw)


# --------------------------------------------------------------------------
# The Assay class
# --------------------------------------------------------------------------


def test_characterize_returns_one_complete_record_per_cut():
    cuts = crude().cuts(n=8)
    assert len(cuts) == 8
    expected = {
        "index", "name", "fraction", "mole_fraction", "x_lower", "x_upper",
        "t_lower", "t_upper", "tb", "sg", "api_gravity", "watson_k", "mw",
        "tc", "pc", "vc", "zc", "omega", "cp_coeffs", "psat_coeffs", "zra",
        "liquid_volume", "solubility_param",  # the last one is M20's
    }
    assert set(cuts[0]) == expected
    assert math.isclose(sum(c["mole_fraction"] for c in cuts), 1.0, abs_tol=1e-12)
    assert math.isclose(sum(c["fraction"] for c in cuts), 1.0, abs_tol=1e-12)
    for c in cuts:
        assert len(c["cp_coeffs"]) == 5
        assert len(c["psat_coeffs"]) == 3
        assert c["tc"] > c["tb"] > 0.0


def test_properties_are_ordered_light_to_heavy():
    cuts = crude().cuts(n=15)
    for lo, hi in zip(cuts, cuts[1:]):
        assert hi["tb"] > lo["tb"]
        assert hi["mw"] > lo["mw"]
        assert hi["tc"] > lo["tc"]
        assert hi["sg"] > lo["sg"]
        assert hi["pc"] < lo["pc"]
        assert hi["omega"] > lo["omega"]


def test_api_gravity_and_bulk_sg_are_the_same_specification():
    by_api = crude(api_gravity=35.0).cuts(n=5)
    by_sg = crude(api_gravity=None, bulk_sg=petroleum.sg_from_api(35.0)).cuts(n=5)
    for a, b in zip(by_api, by_sg):
        assert math.isclose(a["tc"], b["tc"], rel_tol=1e-12)
        assert math.isclose(a["sg"], b["sg"], rel_tol=1e-12)


def test_constant_watson_k_blends_back_to_the_stated_gravity():
    # The design guarantee: characterizing an assay conserves its gravity
    # exactly, not approximately.
    for api in (25.0, 35.0, 45.0):
        cuts = crude(api_gravity=api).cuts(n=40)
        blended = sum(c["fraction"] * c["sg"] for c in cuts)
        assert math.isclose(blended, petroleum.sg_from_api(api), abs_tol=1e-12)
        # And Watson K really is constant across the barrel.
        kws = [c["watson_k"] for c in cuts]
        assert max(kws) - min(kws) < 1e-9


def test_gravity_curve_is_followed_and_lets_watson_k_vary():
    assay = Assay(
        fractions=CURVE_X,
        temperatures=CURVE_T,
        basis="tbp",
        sg_fractions=[0.0, 0.5, 1.0],
        sg_values=[0.70, 0.85, 0.98],
    )
    cuts = assay.cuts(n=10)
    for lo, hi in zip(cuts, cuts[1:]):
        assert hi["sg"] > lo["sg"]
    kws = [c["watson_k"] for c in cuts]
    assert max(kws) - min(kws) > 0.5


def test_every_property_method_produces_a_sane_assay():
    for method in petroleum.PROPERTY_METHODS:
        cuts = crude(method=method).cuts(n=6)
        assert len(cuts) == 6
        for c in cuts:
            assert 0.15 < c["zc"] < 0.35, (method, c["zc"])
            assert 0.0 < c["omega"] < 1.5, (method, c["omega"])
            assert c["pc"] > 101.325, (method, c["pc"])
        # The four families should broadly agree on the mid-barrel cut.
        assert 500.0 < cuts[3]["tc"] < 1000.0, method


def test_cutting_modes_differ_and_all_close_the_barrel():
    assay = crude()
    by_volume = assay.cuts(n=6)
    by_temperature = assay.cuts(n=6, equal_temperature=True)
    by_boundary = assay.cuts(boundaries=[420.0, 500.0, 590.0, 680.0])

    for cuts in (by_volume, by_temperature, by_boundary):
        assert math.isclose(sum(c["fraction"] for c in cuts), 1.0, abs_tol=1e-12)
    assert len(by_boundary) == 5

    # Equal-volume cuts are equal in volume; equal-temperature cuts are not.
    assert max(c["fraction"] for c in by_volume) - min(
        c["fraction"] for c in by_volume
    ) < 1e-12
    assert max(c["fraction"] for c in by_temperature) - min(
        c["fraction"] for c in by_temperature
    ) > 1e-3
    # ... and equal-temperature cuts have equal boiling ranges.
    widths = [c["t_upper"] - c["t_lower"] for c in by_temperature]
    assert max(widths) - min(widths) < 1e-6


def test_explicit_boundaries_land_exactly_where_asked():
    boundaries = [420.0, 500.0, 590.0]
    cuts = crude().cuts(boundaries=boundaries)
    for i, want in enumerate(boundaries):
        assert math.isclose(cuts[i]["t_upper"], want, abs_tol=1e-6)


def test_three_hundred_pseudocomponents():
    # The scale the whole petroleum track exists to reach.
    cuts = crude().cuts(n=300)
    assert len(cuts) == 300
    assert math.isclose(sum(c["mole_fraction"] for c in cuts), 1.0, abs_tol=1e-12)
    assert all(c["t_upper"] - c["t_lower"] < 15.0 for c in cuts)


def test_tbp_conversion_widens_a_d86_curve():
    d86 = Assay(
        fractions=CURVE_X,
        temperatures=[350.0, 380.0, 420.0, 460.0, 500.0, 550.0, 580.0],
        basis="d86",
        api_gravity=45.0,
    )
    x, t = d86.tbp_curve()
    assert x == CURVE_X
    # A real 15-plate column separates better than a single-stage flask.
    assert t[0] < 350.0
    assert t[-1] > 580.0


def test_watson_k_classifies_the_crude():
    kw = crude().watson_k()
    assert 10.0 < kw < 13.0
    # A lighter (more paraffinic) barrel of the same boiling range has a higher K.
    assert crude(api_gravity=50.0).watson_k() > crude(api_gravity=20.0).watson_k()


def test_repr_names_the_basis_and_gravity():
    r = repr(crude())
    assert "TBP" in r and "bulk_sg" in r


# --------------------------------------------------------------------------
# The bridge into the rest of the package
# --------------------------------------------------------------------------


def test_components_are_ordinary_vle_components():
    comps, z = crude().components(n=6)
    assert len(comps) == len(z) == 6
    assert all(isinstance(c, Component) for c in comps)
    assert math.isclose(sum(z), 1.0, abs_tol=1e-12)
    for c in comps:
        assert c.name.startswith("PC-")
        assert len(c.psat_coeffs) == 3
        assert len(c.cp_coeffs) == 5
        assert c.mw > 0.0 and c.tb > 0.0 and c.vc > 0.0
        assert "pseudo-Antoine" in c.psat_source
        assert "Kesler-Lee" in c.cp_source


def test_name_prefix_is_honoured():
    comps, _ = crude(name_prefix="ARAB").components(n=3)
    assert [c.name for c in comps] == ["ARAB-1", "ARAB-2", "ARAB-3"]


def test_to_system_flashes_with_a_closed_mass_balance():
    # The acceptance test for the milestone: an assay drives a real flash with
    # no special casing anywhere.
    system, z = crude().to_system(n=12, eos="PR")
    assert isinstance(system, System)
    assert len(system) == 12
    res = system.flash_pt(500.0, 200.0, z)
    assert res.two_phase
    assert 0.0 < res.beta < 1.0
    for i in range(12):
        recombined = res.beta * res.y[i] + (1.0 - res.beta) * res.x[i]
        assert math.isclose(recombined, z[i], abs_tol=1e-12)
    # Light ends concentrate in the vapor.
    assert res.k[0] > res.k[-1]


def test_pseudocomponents_support_saturation_calculations():
    system, z = crude().to_system(n=8, eos="PR")
    bub = system.bubble_pressure(z, 500.0)
    dew = system.dew_pressure(z, 500.0)
    assert bub.value > dew.value > 0.0


def test_pseudocomponent_enthalpy_uses_the_correlated_heat_capacity():
    # Component.cp_coeffs must actually reach the enthalpy path — the defect
    # this wiring exists to avoid is a silent zero ideal-gas contribution.
    system, z = crude().to_system(n=6, eos="PR")
    cp = system.phase_cp(500.0, 200.0, z, "vapor")
    assert cp > 100.0  # kJ/(kmol·K); a heavy hydrocarbon mixture is O(500)
    h_lo, _ = system.enthalpy_entropy(450.0, 200.0, z, "vapor")
    h_hi, _ = system.enthalpy_entropy(550.0, 200.0, z, "vapor")
    assert h_hi > h_lo


# --------------------------------------------------------------------------
# Standalone correlations
# --------------------------------------------------------------------------


def test_watson_k_separates_paraffins_from_aromatics():
    assert 12.5 < petroleum.watson_k(371.55, 0.6882) < 13.0   # n-heptane
    assert 9.5 < petroleum.watson_k(353.219, 0.8829) < 10.0   # benzene


def test_api_and_sg_round_trip_through_water():
    assert math.isclose(petroleum.api_from_sg(1.0), 10.0, abs_tol=1e-12)
    assert math.isclose(petroleum.sg_from_api(10.0), 1.0, abs_tol=1e-12)
    for sg in (0.65, 0.85, 1.02):
        assert math.isclose(petroleum.sg_from_api(petroleum.api_from_sg(sg)), sg, rel_tol=1e-12)


def test_average_boiling_points_are_ordered():
    a = petroleum.average_boiling_points(450.0, 480.0, 505.0, 530.0, 565.0)
    assert a["wabp"] >= a["vabp"] >= a["cabp"] >= a["meabp"] >= a["mabp"]


def test_riazi_worked_example_survives_the_binding():
    # Riazi 2005 Example 3.3, D86 -> TBP by API Procedure 3A1.1. Published
    # answer in °C: 133.5 154.2 189.2 210.7 232.9 258.2. Checking it here as
    # well as in Rust is what proves the binding did not reorder or rescale
    # anything on the way through.
    x = [0.0, 0.1, 0.3, 0.5, 0.7, 0.9]
    d86 = [t + 273.15 for t in (165.6, 173.7, 193.3, 206.7, 222.8, 242.8)]
    tbp = petroleum.convert_curve(x, d86, "d86", "tbp")
    want = (133.5, 154.2, 189.2, 210.7, 232.9, 258.2)
    for got, expect in zip(tbp, want):
        assert abs((got - 273.15) - expect) < 0.15


def test_convert_curve_round_trips():
    tbp = petroleum.convert_curve(CURVE_X, CURVE_T, "tbp", "d86")
    back = petroleum.convert_curve(CURVE_X, tbp, "d86", "tbp")
    for a, b in zip(CURVE_T, back):
        assert math.isclose(a, b, abs_tol=1e-6)


def test_efv_conversion_needs_a_gravity():
    with pytest.raises(ValueError, match="specific gravity"):
        petroleum.convert_curve(CURVE_X, CURVE_T, "tbp", "efv")
    efv = petroleum.convert_curve(CURVE_X, CURVE_T, "tbp", "efv", sg=0.85)
    # No fractionation at all means the flattest curve of the four.
    assert (efv[-1] - efv[0]) < (CURVE_T[-1] - CURVE_T[0])


def test_cut_curve_without_characterizing():
    cuts = petroleum.cut_curve(CURVE_X, CURVE_T, n=4)
    assert len(cuts) == 4
    assert set(cuts[0]) == {
        "index", "fraction", "x_lower", "x_upper", "t_lower", "t_upper", "tb"
    }
    assert math.isclose(sum(c["fraction"] for c in cuts), 1.0, abs_tol=1e-12)


def test_estimate_matches_a_known_hydrocarbon():
    # n-decane: Tb 447.27 K, SG 0.7342, measured Tc 617.7 K, Pc 2103 kPa,
    # M 142.28, ω 0.4884. The correlations should land close on a compound
    # squarely inside their fitted range.
    p = petroleum.estimate(447.27, 0.7342)
    assert abs(p["tc"] - 617.7) / 617.7 < 0.02
    assert abs(p["pc"] - 2103.0) / 2103.0 < 0.06
    assert abs(p["mw"] - 142.28) / 142.28 < 0.06
    assert abs(p["omega"] - 0.4884) / 0.4884 < 0.10


def test_acentric_factor_from_measured_criticals():
    # Isolated from the critical-property correlations by feeding measured
    # Tc/Pc, so this tests only the ω correlation.
    w = petroleum.acentric_factor(447.27, 617.7, 2103.0, 0.7342)
    assert abs(w - 0.4884) / 0.4884 < 0.02


def test_ideal_gas_cp_is_hydrocarbon_like_and_rises():
    lo = petroleum.ideal_gas_cp(11.8, 350.0)
    hi = petroleum.ideal_gas_cp(11.8, 750.0)
    assert 1.0 < lo < hi < 5.0            # kJ/(kg·K)
    molar = petroleum.ideal_gas_cp(11.8, 750.0, mw=200.0)
    assert math.isclose(molar, hi * 200.0, rel_tol=1e-12)


def test_ideal_gas_cp_coeffs_reproduce_the_direct_call():
    a = petroleum.ideal_gas_cp_coeffs(11.9, 180.0)
    assert len(a) == 5 and a[3] == 0.0 and a[4] == 0.0
    R = 8.31451
    for t in (350.0, 600.0, 900.0):
        poly = sum(c * t**k for k, c in enumerate(a)) * R
        assert math.isclose(poly, petroleum.ideal_gas_cp(11.9, t, mw=180.0), rel_tol=1e-12)


def test_ideal_gas_cp_refuses_to_extrapolate():
    with pytest.raises(ValueError, match="Watson K"):
        petroleum.ideal_gas_cp(8.0, 500.0)


def test_maxwell_bonnell_directions_invert_each_other():
    for kw in (None, 11.0, 12.0, 12.8):
        for tb in (400.0, 600.0, 750.0):
            for p in (1.0, 10.0, 101.325, 500.0):
                t = petroleum.boiling_point_at_pressure(tb, p, kw)
                assert math.isclose(petroleum.normal_boiling_point(t, p, kw), tb, abs_tol=1e-8)


def test_vacuum_lowers_the_boiling_point():
    # Why vacuum distillation exists: keep heavy oil below its cracking point.
    t = petroleum.boiling_point_at_pressure(600.0, "10 mmHg", 11.8)
    assert 450.0 < t < 520.0


def test_vapor_pressure_tracks_a_known_hydrocarbon():
    # n-decane, Tb 447.27 K, K_W 12.67. Its real vapor pressure at 400 K is
    # about 26 kPa; Maxwell-Bonnell should be in that neighbourhood.
    p = petroleum.vapor_pressure(400.0, 447.27, 12.67)
    assert 15.0 < p < 40.0
    # And it must rise with temperature.
    assert petroleum.vapor_pressure(430.0, 447.27, 12.67) > p


# --------------------------------------------------------------------------
# Unit handling and errors
# --------------------------------------------------------------------------


def test_unit_strings_are_accepted_wherever_a_scalar_is():
    assert math.isclose(
        petroleum.watson_k("98.4 degC", 0.6882),
        petroleum.watson_k(371.55, 0.6882),
        rel_tol=1e-6,
    )
    # The tolerance is 1e-7 rather than machine precision because the literal
    # on the right is a hand-rounded mmHg→kPa factor; pint carries more digits.
    assert math.isclose(
        petroleum.boiling_point_at_pressure(600.0, "10 mmHg", 11.8),
        petroleum.boiling_point_at_pressure(600.0, 10.0 / 7.500616827, 11.8),
        rel_tol=1e-7,
    )


def test_curve_temperatures_accept_unit_strings():
    a = Assay(
        fractions=[0.0, 0.5, 1.0],
        temperatures=["100 degC", "250 degC", "400 degC"],
        basis="tbp",
        api_gravity=35.0,
    )
    x, t = a.tbp_curve()
    assert math.isclose(t[0], 373.15, abs_tol=1e-9)
    assert math.isclose(t[2], 673.15, abs_tol=1e-9)


def test_boundaries_accept_unit_strings():
    cuts = crude().cuts(boundaries=["175 degC", "340 degC"])
    assert len(cuts) == 3
    assert math.isclose(cuts[0]["t_upper"], 448.15, abs_tol=1e-6)


def test_bad_input_raises_value_error_with_a_useful_message():
    with pytest.raises(ValueError, match="distillation basis"):
        Assay(fractions=[0.0, 1.0], temperatures=[300.0, 500.0], basis="d99", api_gravity=35.0)
    with pytest.raises(ValueError, match="gravity"):
        Assay(fractions=[0.0, 1.0], temperatures=[300.0, 500.0], basis="tbp")
    with pytest.raises(ValueError, match="not both"):
        Assay(
            fractions=[0.0, 1.0], temperatures=[300.0, 500.0], basis="tbp",
            api_gravity=35.0, bulk_sg=0.85,
        )
    with pytest.raises(ValueError, match="property method"):
        crude(method="nonesuch").cuts(n=3)
    with pytest.raises(ValueError, match="strictly increase"):
        Assay(fractions=[0.5, 0.1], temperatures=[300.0, 500.0], basis="tbp", api_gravity=35.0)
    with pytest.raises(ValueError):
        # A decreasing distillation curve is not physical.
        Assay(fractions=[0.1, 0.5], temperatures=[500.0, 300.0], basis="tbp", api_gravity=35.0)


def test_cut_specification_conflicts_are_rejected():
    a = crude()
    with pytest.raises(ValueError, match="not both"):
        a.cuts(n=5, boundaries=[500.0])
    with pytest.raises(ValueError, match="`n`"):
        a.cuts()
    with pytest.raises(ValueError, match="outside the curve"):
        a.cuts(boundaries=[100.0])
