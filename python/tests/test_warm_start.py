"""Warm-started bubble/dew points (``t_guess`` / ``p_guess`` / ``k_guess``, v0.18.0).

A guess may change the cost of a saturation solve, never its answer: every
warm-started call here must return what the cold call returns, including when
the guess is poor, malformed, or the trivial solution ``K = 1``.
"""

import pytest

from vle import System


@pytest.fixture
def sys10():
    # Ten alkanes, RKS — the downstream staged-separation consumer's headline feed.
    return System(
        ["propane", "isobutane", "n-butane", "isopentane", "n-pentane",
         "n-hexane", "n-heptane", "n-octane", "n-nonane", "n-decane"],
        eos="RKS",
    )


Z10 = [0.1] * 10


def test_temperature_warm_start_matches_cold(sys10):
    for method in ("bubble_temperature", "dew_temperature"):
        cold = getattr(sys10, method)(Z10, 1000.0)
        k_near = [k * 1.05 for k in cold.k]
        for kwargs in (
            {"t_guess": cold.value + 0.5, "k_guess": k_near},
            {"t_guess": cold.value - 2.0},
            {"t_guess": "160 degC"},          # unit-aware guess
            {"t_guess": 50.0},                # absurd guess -> cold fallback
            {"t_guess": cold.value, "k_guess": [1.0, 2.0]},  # wrong length
        ):
            warm = getattr(sys10, method)(Z10, 1000.0, **kwargs)
            assert warm.value == pytest.approx(cold.value, rel=1e-6), (method, kwargs)


def test_pressure_warm_start_matches_cold(sys10):
    for method in ("bubble_pressure", "dew_pressure"):
        cold = getattr(sys10, method)(Z10, 400.0)
        for kwargs in (
            {"p_guess": cold.value * 1.02, "k_guess": cold.k},
            {"p_guess": "5 bar"},
            {"p_guess": cold.value, "k_guess": [float("nan")] * 10},
        ):
            warm = getattr(sys10, method)(Z10, 400.0, **kwargs)
            assert warm.value == pytest.approx(cold.value, rel=1e-6), (method, kwargs)


def test_warm_start_from_the_trivial_solution_is_rejected():
    # Methane/n-pentane at 350 K: the trivial "bubble pressure" K = 1 near
    # 27 MPa must not be accepted as a warm start; the real one is ~12.4 MPa.
    s = System(["methane", "n-pentane"], eos="RKS")
    cold = s.bubble_pressure([0.5, 0.5], 350.0)
    warm = s.bubble_pressure([0.5, 0.5], 350.0, p_guess=27_479.0, k_guess=[1.0, 1.0])
    assert cold.k[0] > 1.5
    assert warm.value == pytest.approx(cold.value, rel=1e-6)
