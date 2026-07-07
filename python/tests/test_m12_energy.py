"""Wheel-level tests for the M12.4 energy API.

Exercises partial molar enthalpy, real-mixture Cp, and the γ-φ liquid
enthalpy/entropy dispatch through the built wheel.
"""

from __future__ import annotations

from vle import System


def test_partial_molar_enthalpy_euler_sum():
    # Σxᵢ·H̄ᵢ = H against the total enthalpy_entropy (φ-φ EOS liquid).
    sys = System(["methane", "n-pentane"], eos="PR", kij=0.02)
    t, p, x = 360.0, 1500.0, [0.4, 0.6]
    for phase in ("vapor", "liquid"):
        hbar = sys.partial_molar_enthalpy(t, p, x, phase)
        total = sum(xi * hi for xi, hi in zip(x, hbar))
        h, _s = sys.enthalpy_entropy(t, p, x, phase)
        assert abs(total - h) <= 1e-6 * max(abs(h), 1.0)


def test_phase_cp_matches_fd_and_ideal_limit():
    sys = System(["methane", "n-pentane"], eos="PR", kij=0.02)
    t, p, x = 360.0, 2000.0, [0.4, 0.6]
    cp = sys.phase_cp(t, p, x, "vapor")
    # FD of H(T).
    h = 1e-2
    h_hi, _ = sys.enthalpy_entropy(t + h, p, x, "vapor")
    h_lo, _ = sys.enthalpy_entropy(t - h, p, x, "vapor")
    cp_fd = (h_hi - h_lo) / (2.0 * h)
    assert abs(cp - cp_fd) <= 1e-4 * max(abs(cp), 1.0)
    # Ideal-gas limit at very low P.
    cp_low = sys.phase_cp(t, 1e-3, x, "vapor")
    assert cp_low > 0.0 and abs(cp_low - cp) > 0.0  # residual is nonzero at 2000 kPa


def test_gamma_phi_enthalpy_entropy_no_longer_errors():
    # Before M12.4 a γ-φ System's enthalpy_entropy errored (no cubic liquid
    # EOS). Now the liquid returns the ideal − condensation + excess assembly.
    sys = System(
        ["methanol", "water"],
        liquid_model="activity",
        activity="VanLaar",
        vapor_model="ideal",
        aij=[[0.0, 0.847], [0.522, 0.0]],
    )
    t, p, x = 340.0, 100.0, [0.4, 0.6]
    h_liq, s_liq = sys.enthalpy_entropy(t, p, x, "liquid")
    h_vap, s_vap = sys.enthalpy_entropy(t, p, x, "vapor")
    assert h_liq < h_vap  # condensation lowers the liquid enthalpy
    assert all(v == v for v in (h_liq, s_liq, h_vap, s_vap))  # not NaN
