"""Wheel-level tests for the M12.3 T/P derivative API.

Exercises the fugacity and K-value temperature/pressure derivatives through
the built wheel (both the free ``vle.System`` wrappers and the underlying
``_engine`` bindings), cross-checking the exact dual-AD results against
central-difference finite differences of the value functions. Includes a γ-φ
(activity-model liquid) case, per the plan's test matrix.
"""

from __future__ import annotations

from vle import System


def _fd(f, arg, h):
    """Central difference of scalar->list function ``f`` at ``arg``."""
    hi = f(arg + h)
    lo = f(arg - h)
    return [(a - b) / (2.0 * h) for a, b in zip(hi, lo)]


def test_d_ln_phi_d_t_and_d_p_match_fd_phi_phi():
    sys = System(["n-butane", "n-heptane"], eos="PR")
    t, p, x = 400.0, 500.0, [0.35, 0.65]
    for phase in ("vapor", "liquid"):
        dt = sys.d_ln_phi_d_t(t, p, x, phase)
        fd_t = _fd(lambda tt: sys.ln_phi(tt, p, x, phase), t, 1e-3)
        for a, b in zip(dt, fd_t):
            assert abs(a - b) <= 1e-6 * max(abs(a), 1e-6) + 1e-9

        dp = sys.d_ln_phi_d_p(t, p, x, phase)
        fd_p = _fd(lambda pp: sys.ln_phi(t, pp, x, phase), p, 1e-1)
        for a, b in zip(dp, fd_p):
            assert abs(a - b) <= 1e-6 * max(abs(a), 1e-6) + 1e-12


def test_volumetric_identity_through_wheel():
    # Σxᵢ ∂lnφ̂ᵢ/∂P = (Z−1)/P — a pure value-path identity.
    sys = System(["n-butane", "n-heptane"], eos="PR")
    t, p, x = 400.0, 500.0, [0.35, 0.65]
    for phase in ("vapor", "liquid"):
        dp = sys.d_ln_phi_d_p(t, p, x, phase)
        z = sys.z_factor(t, p, x, phase)
        lhs = sum(xi * di for xi, di in zip(x, dp))
        rhs = (z - 1.0) / p
        assert abs(lhs - rhs) <= 1e-9 * max(abs(rhs), 1e-9) + 1e-14


def test_k_values_with_derivs_phi_phi_match_fd_and_parity():
    sys = System(["n-butane", "n-heptane"], eos="RKS")
    t, p, x, y = 400.0, 500.0, [0.3, 0.7], [0.6, 0.4]
    k, dkt, dkp = sys.k_values_with_derivs(t, p, x, y)
    # K parity with k_values.
    k_ref = sys.k_values(t, p, x, y)
    for a, b in zip(k, k_ref):
        assert a == b
    # FD of ln K.
    import math

    fd_t = _fd(lambda tt: [math.log(v) for v in sys.k_values(tt, p, x, y)], t, 1e-3)
    fd_p = _fd(lambda pp: [math.log(v) for v in sys.k_values(t, pp, x, y)], p, 1e-2)
    for a, b in zip(dkt, fd_t):
        assert abs(a - b) <= 1e-6 * max(abs(a), 1e-6) + 1e-9
    for a, b in zip(dkp, fd_p):
        assert abs(a - b) <= 1e-6 * max(abs(a), 1e-6) + 1e-12


def test_k_values_with_derivs_gamma_phi_match_fd():
    # γ-φ: Wilson liquid + ideal-gas vapor (modified Raoult) through the wheel.
    sys = System(
        ["methanol", "water"],
        liquid_model="activity",
        activity="Wilson",
        vapor_model="ideal",
        aij=[[0.0, 1200.0], [-300.0, 0.0]],
    )
    import math

    t, p, x, y = 340.0, 100.0, [0.4, 0.6], [0.5, 0.5]
    k, dkt, dkp = sys.k_values_with_derivs(t, p, x, y)
    assert all(v > 0.0 for v in k)
    fd_t = _fd(lambda tt: [math.log(v) for v in sys.k_values(tt, p, x, y)], t, 1e-3)
    fd_p = _fd(lambda pp: [math.log(v) for v in sys.k_values(t, pp, x, y)], p, 1e-2)
    for a, b in zip(dkt, fd_t):
        assert abs(a - b) <= 1e-5 * max(abs(a), 1e-6) + 1e-8
    for a, b in zip(dkp, fd_p):
        assert abs(a - b) <= 1e-6 * max(abs(a), 1e-6) + 1e-12
