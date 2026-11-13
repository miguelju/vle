"""Wheel-level tests for ``System.stage_thermo`` and
``System.k_values_with_t_derivs`` (OPTIMIZATION_PLAN_PART2.md §10).

``stage_thermo`` must return exactly what the separate calls return —
``k_values_with_derivs`` for K and ∂ln K/∂T, ``enthalpy_entropy`` for H and
``phase_cp`` for Cp — for the single-pass φ-φ route and for the assembled
γ-φ route.
"""

from __future__ import annotations

import pytest

from vle import StageThermoResult, System


def _close(a: float, b: float, tol: float = 1e-10) -> bool:
    """Relative agreement with an absolute floor for values near zero."""
    return abs(a - b) <= tol * max(abs(b), 1.0)


# One φ-φ system per EOS, and a γ-φ (Wilson liquid, ideal-gas vapor)
# system that takes the assembled route.
SYSTEMS = [
    ("PR", lambda: System(["n-butane", "n-heptane"], eos="PR")),
    ("RKS", lambda: System(["propane", "n-pentane", "n-octane"], eos="RKS")),
    (
        "Wilson",
        lambda: System(
            ["methanol", "water"],
            liquid_model="activity",
            activity="Wilson",
            vapor_model="ideal",
            aij=[[0.0, 1200.0], [-300.0, 0.0]],
        ),
    ),
]


@pytest.mark.parametrize("label,make", SYSTEMS, ids=[s[0] for s in SYSTEMS])
def test_stage_thermo_equals_the_separate_calls(label, make):
    sys = make()
    n = len(sys.components)
    # A liquid weighted to the heavy end and a vapor weighted to the light end.
    x = [(i + 1.0) for i in range(n)]
    y = [(n - i) * 1.0 for i in range(n)]
    x = [v / sum(x) for v in x]
    y = [v / sum(y) for v in y]
    t, p = (340.0, 100.0) if label == "Wilson" else (400.0, 500.0)

    st = sys.stage_thermo(t, p, x, y)
    assert isinstance(st, StageThermoResult)

    k, dkt, _dkp = sys.k_values_with_derivs(t, p, x, y)
    for a, b in zip(st.k, k):
        assert _close(a, b)
    for a, b in zip(st.d_ln_k_d_t, dkt):
        assert abs(a - b) <= 1e-10 * max(abs(b), 1e-3)
    assert _close(st.h_liquid, sys.enthalpy_entropy(t, p, x, "liquid")[0])
    assert _close(st.h_vapor, sys.enthalpy_entropy(t, p, y, "vapor")[0])
    assert _close(st.cp_liquid, sys.phase_cp(t, p, x, "liquid"))
    assert _close(st.cp_vapor, sys.phase_cp(t, p, y, "vapor"))
    # Physical sanity, so the comparison above cannot pass on two zeros.
    assert st.h_vapor > st.h_liquid
    assert st.cp_liquid > 0.0 and st.cp_vapor > 0.0


def test_k_values_with_t_derivs_equals_the_full_call():
    sys = System(["n-butane", "n-heptane"], eos="PR")
    t, p, x, y = 400.0, 500.0, [0.35, 0.65], [0.8, 0.2]
    k, dkt, _dkp = sys.k_values_with_derivs(t, p, x, y)
    k2, dkt2 = sys.k_values_with_t_derivs(t, p, x, y)
    assert k2 == k
    assert dkt2 == dkt


def test_stage_thermo_accepts_unit_strings():
    sys = System(["n-butane", "n-heptane"], eos="PR")
    x, y = [0.35, 0.65], [0.8, 0.2]
    a = sys.stage_thermo(400.0, 500.0, x, y)
    b = sys.stage_thermo("400 K", "5 bar", x, y)
    assert _close(a.h_liquid, b.h_liquid)
    assert a.k == pytest.approx(b.k, rel=1e-12)


def test_stage_thermo_rejects_wrong_length():
    sys = System(["n-butane", "n-heptane"], eos="PR")
    with pytest.raises(Exception):
        sys.stage_thermo(400.0, 500.0, [1.0], [0.8, 0.2])
