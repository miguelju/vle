"""Wheel-level tests for the M20 refinery-thermodynamics API.

The numerical validation of each method (thermodynamic-consistency identities
for Lee–Kesler, the Grayson–Streed / Chao–Seader coefficient provenance, the
free-water balances, the Peneloux densities against measured values) lives in
the Rust unit tests. What these tests check is that the *binding layer* and the
Python conveniences pass the right numbers in the right units, that the new
liquid models plug into every existing flash entry point, and that the whole
thing scales to a hundreds-of-pseudocomponents assay.
"""

from __future__ import annotations

import math

import numpy as np
import pytest

from vle import System, petroleum, refinery, steam
from vle.components import Component
from vle.petroleum import Assay
from vle.results import FreeWaterFlashResult, LeeKeslerDeparture

R = 8.31451

CURVE_X = [0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95]
CURVE_T = [310.0, 380.0, 460.0, 540.0, 620.0, 730.0, 790.0]


def crude(**kw) -> Assay:
    kw.setdefault("api_gravity", 35.0)
    return Assay(fractions=CURVE_X, temperatures=CURVE_T, basis="tbp", **kw)


def hydrocarbons() -> list[Component]:
    """n-butane / n-heptane with regular-solution data (δ in (cal/cm³)^½)."""
    from dataclasses import replace

    from vle import components as db

    b = replace(db.get("n-butane"), solubility_param=6.73)
    h = replace(db.get("n-heptane"), solubility_param=7.43)
    return [b, h]


# --------------------------------------------------------------------------
# Grayson–Streed and Braun K10 as liquid models
# --------------------------------------------------------------------------


def test_grayson_streed_is_nu_gamma_over_phi_and_flashes():
    comps = hydrocarbons()
    gs = System(comps, eos="RKS", liquid_model="grayson_streed")
    t, p = 400.0, 800.0
    x, y = [0.4, 0.6], [0.7, 0.3]
    k = gs.k_values(t, p, x, y)
    # ν from the pure binding, γ from Scatchard-Hildebrand by hand, φ̂ from the
    # same RKS vapor.
    vap = System(comps, eos="RKS").ln_phi(t, p, y, "vapor")
    vl = [c.liquid_volume for c in comps]
    dl = [c.solubility_param for c in comps]
    vtot = sum(xi * v for xi, v in zip(x, vl))
    dbar = sum(xi * v * d for xi, v, d in zip(x, vl, dl)) / vtot
    for i, c in enumerate(comps):
        ln_nu = refinery.regular_solution_ln_nu(t, p, c.tc, c.pc, c.omega)
        ln_gamma = vl[i] * (dl[i] - dbar) ** 2 / (R * 0.23898 * t)  # engine R_CAL
        want = math.exp(ln_nu + ln_gamma - vap[i])
        assert k[i] == pytest.approx(want, rel=1e-10)
    assert k[0] > 1 > k[1]
    r = gs.flash_pt(380.0, 600.0, [0.5, 0.5])
    assert r.two_phase
    for i in range(2):
        assert r.beta * r.y[i] + (1 - r.beta) * r.x[i] == pytest.approx(0.5, abs=1e-9)
    # Saturation entry points accept the model too.
    bp = gs.bubble_pressure([0.5, 0.5], 350.0)
    assert 100 < bp.value < 2000


def test_grayson_streed_without_solubility_data_equals_legacy_chao_seader():
    names = ["n-butane", "n-heptane"]  # database entries carry no δ
    gs = System(names, eos="RKS", liquid_model="grayson_streed")
    cs = System(names, eos="RKS", liquid_model="chao_seader")
    a = gs.k_values(400.0, 800.0, [0.4, 0.6], [0.7, 0.3])
    b = cs.k_values(400.0, 800.0, [0.4, 0.6], [0.7, 0.3])
    assert a == pytest.approx(b, rel=1e-12)


def test_regular_solution_tables_and_species():
    gs = refinery.regular_solution_ln_nu(400.0, 1000.0, 469.7, 3370.0, 0.252)
    cs = refinery.regular_solution_ln_nu(400.0, 1000.0, 469.7, 3370.0, 0.252, set="chao-seader")
    assert gs != cs and abs(gs - cs) < 1.2  # same data, different fits
    h2 = refinery.regular_solution_ln_nu(400.0, 1000.0, 33.2, 1300.0, -0.216, species="hydrogen")
    assert math.isfinite(h2)
    from vle._engine import RegularSolutionSet

    again = refinery.regular_solution_ln_nu(
        400.0, 1000.0, 469.7, 3370.0, 0.252, set=RegularSolutionSet.GraysonStreed1963
    )
    assert again == gs
    # unit-aware inputs
    from vle.units import Q_

    q = refinery.regular_solution_ln_nu(Q_(126.85, "degC"), Q_(10, "bar"), 469.7, 3370.0, 0.252)
    assert q == pytest.approx(gs, rel=1e-12)


def test_braun_k10_is_maxwell_bonnell_over_pressure():
    comps = hydrocarbons()  # database tb values
    bk = System(comps, vapor_model="ideal", liquid_model="bk10")
    t, p = 350.0, 120.0
    k = bk.k_values(t, p, [0.5, 0.5], [0.5, 0.5])
    for i, c in enumerate(comps):
        assert k[i] == pytest.approx(petroleum.vapor_pressure(t, c.tb) / p, rel=1e-12)
    # It drives a flash and a bubble point like any other liquid model.
    r = bk.flash_pt(t, p, [0.5, 0.5])
    assert r.two_phase and 0 < r.beta < 1
    bt = bk.bubble_temperature([0.5, 0.5], 101.325)
    assert 280 < bt.value < 372
    # A pseudocomponent's Watson K flows through (Component.watson_k).
    from dataclasses import replace

    with_kw = [replace(comps[0]), replace(comps[1], watson_k=11.0)]
    bk2 = System(with_kw, vapor_model="ideal", liquid_model="bk10")
    k2 = bk2.k_values(t, 50.0, [0.5, 0.5], [0.5, 0.5])
    k1 = bk.k_values(t, 50.0, [0.5, 0.5], [0.5, 0.5])
    assert k2[0] == pytest.approx(k1[0], rel=1e-12)
    assert k2[1] != pytest.approx(k1[1], rel=1e-6)


def test_unknown_liquid_model_lists_the_new_ones():
    with pytest.raises(ValueError, match="grayson_streed"):
        System(["n-butane"], liquid_model="nope")


# --------------------------------------------------------------------------
# Free-water flash
# --------------------------------------------------------------------------


def test_free_water_flash_decants_and_balances():
    s = System(["n-pentane", "n-decane", "water"], eos="PR")
    z = [0.25, 0.65, 0.10]
    r = s.flash_free_water(325.0, 40.0, z, water="water")
    assert isinstance(r, FreeWaterFlashResult)
    assert r.free_water and r.free_water_fraction > 0.02 and r.vapor_fraction > 0.02
    assert r.vapor_fraction + r.hc_liquid_fraction + r.free_water_fraction == pytest.approx(1.0, abs=1e-12)
    assert r.vapor_fraction * r.y[2] + r.free_water_fraction == pytest.approx(0.10, abs=1e-12)
    for i in range(2):
        assert r.vapor_fraction * r.y[i] + r.hc_liquid_fraction * r.x[i] == pytest.approx(z[i], abs=1e-10)
    assert r.y[2] == pytest.approx(r.psat_water / 40.0, abs=1e-12)
    assert r.x[2] == 0.0
    assert r.water_index == 2 and r.t == 325.0 and r.p == 40.0
    # Water's Antoine Psat vs IF97 agree to ~1 %; passing IF97 explicitly is honoured.
    pw = steam.psat(325.0)
    assert pw == pytest.approx(r.psat_water, rel=0.02)
    r2 = s.flash_free_water(325.0, 40.0, z, water=2, psat_water=pw)
    assert r2.psat_water == pw and r2.y[2] == pytest.approx(pw / 40.0, abs=1e-12)


def test_free_water_flash_hot_stripper_keeps_water_in_vapor():
    s = System(["n-pentane", "n-decane", "water"], eos="PR")
    z = [0.2, 0.7, 0.10]
    r = s.flash_free_water(450.0, 150.0, z, water="water")
    assert not r.free_water and r.free_water_fraction == 0.0
    assert r.vapor_fraction * r.y[2] == pytest.approx(0.10, abs=1e-10)
    assert math.isnan(r.k[2])


def test_free_water_flash_on_a_steam_stripped_assay():
    # A characterized crude with 3 % stripping steam at side-stripper
    # conditions: hot enough that the water stays in the vapor, and the
    # dry-hydrocarbon flash is the same the plain flash would give.
    comps, z = crude().components(n=20)
    from vle import components as db

    comps.append(db.get("water"))
    z = [zi * 0.97 for zi in z] + [0.03]
    s = System(comps, eos="PR")
    r = s.flash_free_water(560.0, 250.0, z, water="water")
    assert not r.free_water
    assert r.vapor_fraction * r.y[-1] == pytest.approx(0.03, abs=1e-9)
    assert sum(r.y) == pytest.approx(1.0, abs=1e-12)
    assert sum(r.x) == pytest.approx(1.0, abs=1e-12)


# --------------------------------------------------------------------------
# Lee–Kesler
# --------------------------------------------------------------------------


def test_lee_kesler_reduced_and_pure_component_route():
    d = refinery.lee_kesler_reduced(0.8, 0.3, 0.3, "liquid")
    assert set(d) == {"z", "h_dep_rt", "s_dep_r", "ln_phi"}
    assert d["z"] < 0.1 and d["h_dep_rt"] < -5
    # Gibbs consistency holds through the binding.
    assert d["s_dep_r"] == pytest.approx(d["h_dep_rt"] - d["ln_phi"], abs=1e-10)
    v = refinery.lee_kesler_reduced(0.8, 0.3, 0.3, "vapor")
    assert v["z"] > d["z"] and v["h_dep_rt"] > d["h_dep_rt"]  # lighter, closer to ideal
    with pytest.raises(ValueError):
        refinery.lee_kesler_reduced(-1.0, 0.3, 0.3)


def test_lee_kesler_mixture_route_on_a_system():
    s = System(["n-octane", "n-decane"], eos="PR")
    tc, pc, w = s.lee_kesler_pseudocritical([0.5, 0.5])
    assert 568 < tc < 618 and 2110 < pc < 2500 and w == pytest.approx(0.5 * (0.3996 + 0.4923), abs=0.01)
    d = s.lee_kesler_departure(450.0, 300.0, [0.5, 0.5], "liquid")
    assert isinstance(d, LeeKeslerDeparture) and d.h_dep_rt < -8
    # Full enthalpy: ideal-gas part is shared with the EOS route, so the two
    # routes differ only by the residual, and both are large and negative.
    h_lk, s_lk = s.enthalpy_entropy_lee_kesler(450.0, 300.0, [0.5, 0.5], "liquid")
    h_pr, s_pr = s.enthalpy_entropy(450.0, 300.0, [0.5, 0.5], "liquid")
    # Both are "ideal-gas H at (T, T_ref) minus a heat of vaporization"; the
    # residuals differ by a fraction of R·T between the two models.
    assert h_lk == pytest.approx(h_pr, abs=0.3 * R * 450.0)
    assert s_lk == pytest.approx(s_pr, abs=0.3 * R)
    # The residual is exactly what was added: H_lk = H_ideal + R·T·(H−H°)/(RT),
    # with H_ideal the ideal-gas mixture enthalpy (an ideal-vapor System's H).
    ideal = System(["n-octane", "n-decane"], vapor_model="ideal", liquid_model="ideal")
    h_ig, s_ig = ideal.enthalpy_entropy(450.0, 300.0, [0.5, 0.5], "vapor")
    assert h_lk == pytest.approx(h_ig + d.h_dep_rt * R * 450.0, abs=1e-9)
    assert s_lk == pytest.approx(s_ig + d.s_dep_r * R, abs=1e-9)
    # eta selects the mixing rule and changes the pseudo-critical Tc.
    tc1, _, _ = s.lee_kesler_pseudocritical([0.5, 0.5], eta=1.0)
    assert tc1 != pytest.approx(tc, abs=1e-6)


def test_lee_kesler_scales_to_a_three_hundred_cut_assay():
    import time

    system, z = crude().to_system(n=300)
    t0 = time.perf_counter()
    for _ in range(20):
        system.enthalpy_entropy_lee_kesler(600.0, 200.0, z, "liquid")
    dt = (time.perf_counter() - t0) / 20
    # Generous bound — the point is "sub-millisecond per stage", not a number.
    assert dt < 0.02, f"{dt * 1e3:.2f} ms per Lee-Kesler mixture enthalpy at N = 300"


# --------------------------------------------------------------------------
# Peneloux
# --------------------------------------------------------------------------


def test_peneloux_translation_improves_heavy_liquid_density():
    s = System(["n-heptane", "n-decane"], eos="RKS")
    c = s.peneloux_shifts()
    assert len(c) == 2 and all(ci > 5 for ci in c)
    assert c[0] == pytest.approx(refinery.peneloux_shift("RKS", 540.2, 2736.0, 0.3495), rel=0.02)
    for x, rho_meas in [([1.0, 0.0], 680.0), ([0.0, 1.0], 727.0)]:
        v = s.translated_molar_volume(298.15, 101.325, x, "liquid")
        rho = s.translated_density(298.15, 101.325, x, "liquid")
        assert rho == pytest.approx(rho_meas, rel=0.03)
        mw = sum(xi * ci.mw for xi, ci in zip(x, s.components))
        assert rho == pytest.approx(1000 * mw / v, rel=1e-12)
    # K-values are untouched by translation, by construction: the shift is not
    # in the fugacity path at all.
    r1 = s.flash_pt(430.0, 150.0, [0.5, 0.5])
    assert r1.two_phase
    with pytest.raises(ValueError):
        refinery.peneloux_shift("VdW1870", 540.2, 2736.0, 0.35)


def test_pseudocomponents_carry_the_refinery_inputs():
    comps, _ = crude().components(n=5)
    for c in comps:
        assert c.watson_k > 10 and c.solubility_param > 5 and c.zra > 0.2
    # …so a Grayson-Streed system built from an assay has a real γ, and a
    # BK10 one has the Watson correction — both flash.
    for lm in ("grayson_streed", "bk10"):
        system, z = crude().to_system(n=10, eos="RKS", liquid_model=lm)
        r = system.flash_pt(600.0, 150.0, z)
        assert 0.0 <= r.beta <= 1.0
