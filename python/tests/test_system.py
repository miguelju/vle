"""Tests for the high-level :class:`vle.System` wrapper (Milestone 10)."""

import numpy as np
import pytest

from vle import (
    ActivityModel,
    BubbleResult,
    CriticalResult,
    CubicEos,
    DewResult,
    FlashResult,
    System,
)
from vle import _engine as e
from vle.units import Q_


# ── Construction ──────────────────────────────────────────────────────────

def test_construct_from_names():
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    assert sys.n_components == 2
    assert sys.names == ["n-heptane", "n-butane"]
    assert len(sys) == 2
    assert "n-heptane" in repr(sys)


def test_eos_alias_and_exact_name_and_enum_all_work():
    a = System(["benzene", "cyclohexane"], eos="PR")
    b = System(["benzene", "cyclohexane"], eos="PR1976")
    c = System(["benzene", "cyclohexane"], eos=CubicEos.PR1976)
    # All three resolve to the same EOS → identical scalar result.
    for s in (a, b, c):
        r = s.bubble_pressure([0.4, 0.6], 350.0)
        assert r.value == pytest.approx(a.bubble_pressure([0.4, 0.6], 350.0).value)


def test_unknown_eos_alias_raises():
    with pytest.raises(ValueError, match="unknown eos"):
        System(["water", "benzene"], eos="nope")


def test_from_arrays_matches_name_construction():
    named = System(["n-heptane", "n-butane"], eos="RKS")
    arr = System.from_arrays(
        tcs=[540.2, 425.125], pcs=[2735.73, 3796.0], omegas=[0.349, 0.201],
        names=["n-heptane", "n-butane"], eos="RKS",
    )
    r1 = named.flash_pt(300.0, 100.0, [0.5, 0.5])
    r2 = arr.flash_pt(300.0, 100.0, [0.5, 0.5])
    assert r1.beta == pytest.approx(r2.beta, abs=1e-9)


def test_scalar_kij_for_binary():
    sys = System(["carbon dioxide", "n-butane"], eos="PR", kij=0.1357)
    # Should build without error and flash.
    r = sys.flash_pt(320.0, 3000.0, [0.5, 0.5])
    assert isinstance(r, FlashResult)


def test_kij_wrong_shape_raises():
    with pytest.raises(ValueError, match="kij must be"):
        System(["water", "benzene"], kij=[[0.0, 0.1, 0.2]])


# ── Scalar methods + dataclasses ──────────────────────────────────────────

def test_flash_pt_returns_dataclass_and_mass_balance():
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    r = sys.flash_pt(300.0, 100.0, [0.5, 0.5])
    assert isinstance(r, FlashResult)
    assert r.two_phase
    assert 0.0 < r.beta < 1.0
    # β·y + (1−β)·x = z.
    for i in range(2):
        assert r.beta * r.y[i] + (1 - r.beta) * r.x[i] == pytest.approx(0.5, abs=1e-8)
    # k = y/x.
    for i in range(2):
        assert r.k[i] == pytest.approx(r.y[i] / r.x[i], rel=1e-6)
    assert r.t == 300.0 and r.p == 100.0


def test_high_level_matches_engine_free_function():
    """System.flash_pt must equal the raw engine binding for the same inputs."""
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    r = sys.flash_pt(300.0, 100.0, [0.5, 0.5])
    beta, x, y, k, iters, two = e.flash_pt(
        [540.2, 425.125], [2735.73, 3796.0], [0.349, 0.201], [0.5, 0.5],
        300.0, 100.0, vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972,
    )
    assert r.beta == pytest.approx(beta, abs=1e-12)
    assert r.x == pytest.approx(x, abs=1e-12)


def test_bubble_pressure_returns_bubble_result():
    sys = System(["methanol", "water"], eos="PR")
    r = sys.bubble_pressure([0.5, 0.5], 298.15)
    assert isinstance(r, BubbleResult)
    assert r.value > 0.0
    assert sum(r.y) == pytest.approx(1.0, abs=1e-9)


def test_dew_pressure_below_bubble_pressure():
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    z = [0.4, 0.6]
    bub = sys.bubble_pressure(z, 380.0)
    dew = sys.dew_pressure(z, 380.0)
    assert isinstance(dew, DewResult)
    assert dew.value <= bub.value + 1e-6


def test_critical_point_between_pure_criticals():
    sys = System(["methane", "ethane"], eos="PR")
    cp = sys.critical_point([0.5, 0.5])
    assert isinstance(cp, CriticalResult)
    assert 190.0 < cp.tc < 306.0  # between the two pure Tc's
    assert cp.pc > 0.0


# ── Unit-aware inputs ─────────────────────────────────────────────────────

def test_unit_aware_temperature_and_pressure():
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    base = sys.flash_pt(300.0, 100.0, [0.5, 0.5])
    # 300 K == 26.85 °C; 100 kPa via a string.
    q = sys.flash_pt(Q_(26.85, "degC"), "100 kPa", [0.5, 0.5])
    assert q.beta == pytest.approx(base.beta, abs=1e-6)


def test_activity_model_gamma_phi_construction():
    sys = System(
        ["methanol", "water"], vapor_model="cubic", liquid_model="activity",
        eos="PR", activity="wilson", aij=[[0.0, 1200.0], [-300.0, 0.0]],
    )
    r = sys.flash_pt(340.0, 100.0, [0.5, 0.5])
    assert isinstance(r, FlashResult)


def test_bubble_temperature_close_boiling_txy():
    """Regression (M10 fix): bubble_temperature must converge across the whole
    composition range for a close-boiling φ-φ pair (benzene/cyclohexane), where
    the true bubble T sits in the K≈1 band. Exercises the batch path the Txy
    plot helper uses."""
    import numpy as np

    sys = System(["benzene", "cyclohexane"], eos="RKS")
    x1 = np.linspace(1e-3, 1 - 1e-3, 25)
    xs = np.column_stack([x1, 1 - x1])
    res = sys.bubble_temperature_batch(xs, np.array([101.325]))
    assert np.isfinite(res.value).all(), "some close-boiling bubble-T points failed"
    # Both components boil ~353 K at 1 atm, so the whole curve sits in that band.
    assert np.all((res.value > 345.0) & (res.value < 362.0))


def test_properties_z_and_lnphi():
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    zv = sys.z_factor(300.0, 100.0, [0.5, 0.5], "vapor")
    zl = sys.z_factor(300.0, 100.0, [0.5, 0.5], "liquid")
    assert zv > zl  # vapor root is the larger Z
    lnphi = sys.ln_phi(300.0, 100.0, [0.5, 0.5], "vapor")
    assert len(lnphi) == 2
