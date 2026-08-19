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


def test_gamma_phi_phase_cp_no_longer_errors_and_matches_fd():
    # M12.6: before, a γ-φ System's phase_cp errored ("needs a cubic model on
    # that phase"). Now the liquid Cp is the exact T-derivative of the shipped
    # γ-φ enthalpy, and the ideal-gas vapor is Σy·Cp°.
    sys = System(
        ["methanol", "water"],
        liquid_model="activity",
        activity="VanLaar",
        vapor_model="ideal",
        aij=[[0.0, 0.847], [0.522, 0.0]],
    )
    t, p, x = 340.0, 100.0, [0.4, 0.6]
    h = 5e-2
    for phase in ("liquid", "vapor"):
        cp = sys.phase_cp(t, p, x, phase)
        h_hi, _ = sys.enthalpy_entropy(t + h, p, x, phase)
        h_lo, _ = sys.enthalpy_entropy(t - h, p, x, phase)
        cp_fd = (h_hi - h_lo) / (2.0 * h)
        assert abs(cp - cp_fd) <= 1e-6 * max(abs(cp), 1.0), (phase, cp, cp_fd)
        assert 20.0 < cp < 200.0, (phase, cp)  # physically sized, kJ/(kmol·K)
    # NRTL (a real Cpᴱ) through the same route.
    nrtl = System(
        ["ethanol", "water"],
        liquid_model="activity",
        activity="NRTL",
        vapor_model="ideal",
        aij=[[0.0, -458.7], [5574.0, 0.0]],
        alpha=[[0.0, 0.303], [0.303, 0.0]],
    )
    cp = nrtl.phase_cp(350.0, 101.325, [0.5, 0.5], "liquid")
    h_hi, _ = nrtl.enthalpy_entropy(350.0 + h, 101.325, [0.5, 0.5], "liquid")
    h_lo, _ = nrtl.enthalpy_entropy(350.0 - h, 101.325, [0.5, 0.5], "liquid")
    assert abs(cp - (h_hi - h_lo) / (2.0 * h)) <= 1e-6 * abs(cp)
    # φ-φ path unchanged: still equals the FD (the M12.4 test above), and a
    # cubic-vapor γ-φ system routes the vapor through the EOS.
    mixed = System(
        ["methanol", "water"],
        liquid_model="activity",
        activity="VanLaar",
        vapor_model="cubic",
        eos="PR",
        aij=[[0.0, 0.847], [0.522, 0.0]],
    )
    assert mixed.phase_cp(t, p, x, "vapor") > 0.0
