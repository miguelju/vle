"""M9 flash / bubble / dew / stability — Python-side tests through the wheel.

Exercises the bindings added in Milestone 9:

- `rachford_rice` — the scalar vapor-fraction solve.
- `flash_pt` — isothermal flash (φ-φ EOS both phases, and γ-φ activity liquid).
- `bubble_pressure_py` / `bubble_temperature_py` / `dew_pressure_py` /
  `dew_temperature_py` — the four saturation-point solves.
- `flash_stability` — tangent-plane-distance stability analysis.
- `flash_k_values_py` — the K-value dispatch used to check equilibrium.

Correctness is checked via physics invariants (mass balance, the saturation
sum condition Σ Kx = 1 / Σ y/K = 1, the dew ≤ bubble pressure ordering) rather
than hard-coded numbers, so the tests stay robust to engine refinements.
"""

import math

import pytest

import vle._engine as e


# Component data: (tc [K], pc [kPa], omega, reduced-Antoine [a1,a2,a3]).
N_BUTANE = dict(tc=425.12, pc=3796.0, omega=0.200, psat=[4.35, 2277.0, -30.0], vl=96.5)
N_HEPTANE = dict(tc=540.2, pc=2740.0, omega=0.350, psat=[4.02, 2911.0, -56.0], vl=147.5)
METHANOL = dict(tc=512.6, pc=8097.0, omega=0.564, psat=[5.20, 3200.0, -35.0], vl=40.7)
WATER = dict(tc=647.1, pc=22064.0, omega=0.344, psat=[5.11, 3800.0, -46.0], vl=18.07)


def arrays(*comps):
    return (
        [c["tc"] for c in comps],
        [c["pc"] for c in comps],
        [c["omega"] for c in comps],
        [c["psat"] for c in comps],
        [c["vl"] for c in comps],
    )


# =============================================================================
# Rachford-Rice
# =============================================================================


def test_rachford_rice_hand_solution():
    """z=[0.5,0.5], K=[2,0.5] → β=0.5 (hand-solved)."""
    beta = e.rachford_rice([0.5, 0.5], [2.0, 0.5])
    assert beta == pytest.approx(0.5, abs=1e-10)


def test_rachford_rice_single_phase_raises():
    """All K>1 → no interior root → ValueError."""
    with pytest.raises(ValueError):
        e.rachford_rice([0.5, 0.5], [2.0, 1.5])


# =============================================================================
# Isothermal flash — φ-φ
# =============================================================================


def test_flash_pt_two_phase_mass_balance():
    """RKS both phases; β·y + (1−β)·x = z at the converged split."""
    tcs, pcs, om, psat, vl = arrays(N_BUTANE, N_HEPTANE)
    z = [0.5, 0.5]
    beta, x, y, k, iters, two_phase = e.flash_pt(
        tcs, pcs, om, z, 420.0, 1000.0,
        vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972,
    )
    assert two_phase
    assert 0.0 <= beta <= 1.0
    for i in range(2):
        assert beta * y[i] + (1 - beta) * x[i] == pytest.approx(z[i], abs=1e-8)
    # Equilibrium ratios consistent.
    for i in range(2):
        assert k[i] == pytest.approx(y[i] / x[i], rel=1e-6)


def test_flash_pt_single_phase_high_pressure():
    """Compressed to a single liquid at high P → two_phase False, β=0."""
    tcs, pcs, om, psat, vl = arrays(N_BUTANE, N_HEPTANE)
    beta, x, y, k, iters, two_phase = e.flash_pt(
        tcs, pcs, om, [0.5, 0.5], 350.0, 20000.0,
        vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972,
    )
    assert not two_phase
    assert beta == 0.0


def test_flash_pt_gamma_phi_runs():
    """γ-φ flash: Wilson liquid + ideal vapor, methanol/water."""
    tcs, pcs, om, psat, vl = arrays(METHANOL, WATER)
    aij = [[0.0, 1200.0], [-300.0, 0.0]]
    beta, x, y, k, iters, two_phase = e.flash_pt(
        tcs, pcs, om, [0.5, 0.5], 350.0, 80.0,
        vapor_kind="ideal", liquid_kind="activity",
        liquid_activity=e.ActivityModel.Wilson, aij=aij, vl=vl, psat_coeffs=psat,
    )
    if two_phase:
        for i in range(2):
            assert beta * y[i] + (1 - beta) * x[i] == pytest.approx(0.5, abs=1e-7)


# =============================================================================
# Bubble / dew
# =============================================================================


def test_bubble_pressure_saturation_condition():
    tcs, pcs, om, psat, vl = arrays(N_BUTANE, N_HEPTANE)
    x = [0.4, 0.6]
    p, y, k = e.bubble_pressure_py(
        tcs, pcs, om, x, 400.0,
        vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972, tol=1e-10,
    )
    k_chk = e.flash_k_values_py(
        tcs, pcs, om, x, y, 400.0, p,
        vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972,
    )
    s = sum(k_chk[i] * x[i] for i in range(2))
    assert s == pytest.approx(1.0, abs=1e-7)
    assert sum(y) == pytest.approx(1.0, abs=1e-9)


def test_dew_below_bubble_pressure():
    """Dew pressure ≤ bubble pressure for the same composition and T."""
    tcs, pcs, om, psat, vl = arrays(N_BUTANE, N_HEPTANE)
    z = [0.5, 0.5]
    bub_p, _, _ = e.bubble_pressure_py(
        tcs, pcs, om, z, 400.0, vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972,
    )
    dew_p, _, _ = e.dew_pressure_py(
        tcs, pcs, om, z, 400.0, vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972,
    )
    assert dew_p <= bub_p + 1e-6


def test_bubble_temperature_in_range():
    tcs, pcs, om, psat, vl = arrays(N_BUTANE, N_HEPTANE)
    x = [0.4, 0.6]
    t, y, k = e.bubble_temperature_py(
        tcs, pcs, om, x, 1000.0, vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972, tol=1e-9,
    )
    assert 250.0 < t < 600.0
    k_chk = e.flash_k_values_py(
        tcs, pcs, om, x, y, t, 1000.0, vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.RKS1972, liquid_eos=e.CubicEos.RKS1972,
    )
    assert sum(k_chk[i] * x[i] for i in range(2)) == pytest.approx(1.0, abs=1e-5)


def test_dew_temperature_gamma_phi():
    """γ-φ dew temperature (Wilson liquid + ideal vapor)."""
    tcs, pcs, om, psat, vl = arrays(METHANOL, WATER)
    aij = [[0.0, 1100.0], [-250.0, 0.0]]
    y = [0.5, 0.5]
    t, x, k = e.dew_temperature_py(
        tcs, pcs, om, y, 101.325,
        vapor_kind="ideal", liquid_kind="activity",
        liquid_activity=e.ActivityModel.Wilson, aij=aij, vl=vl, psat_coeffs=psat, tol=1e-8,
    )
    k_chk = e.flash_k_values_py(
        tcs, pcs, om, x, y, t, 101.325,
        vapor_kind="ideal", liquid_kind="activity",
        liquid_activity=e.ActivityModel.Wilson, aij=aij, vl=vl, psat_coeffs=psat,
    )
    assert sum(y[i] / k_chk[i] for i in range(2)) == pytest.approx(1.0, abs=1e-4)


# =============================================================================
# Stability
# =============================================================================


def test_stability_two_phase_feed_unstable():
    tcs, pcs, om, psat, vl = arrays(N_BUTANE, N_HEPTANE)
    # 420 K / 1000 kPa is inside the two-phase region.
    stable, trial_k, tpd = e.flash_stability(
        tcs, pcs, om, [0.5, 0.5], 420.0, 1000.0, e.CubicEos.RKS1972,
    )
    assert not stable
    assert tpd < 0.0
    assert len(trial_k) == 2


def test_stability_single_phase_stable():
    tcs, pcs, om, psat, vl = arrays(N_BUTANE, N_HEPTANE)
    stable, trial_k, tpd = e.flash_stability(
        tcs, pcs, om, [0.5, 0.5], 350.0, 20000.0, e.CubicEos.RKS1972,
    )
    assert stable
    assert trial_k == []


# =============================================================================
# Critical point / adiabatic / regression (M9 tail)
# =============================================================================


def test_critical_point_pure_recovers_tc_pc():
    """Pure ethane critical point recovers its Tc, Pc under PR."""
    tc, pc, vc = e.critical_point_py(
        e.CubicEos.PR1976, [305.32], [4872.0], [0.0995], [1.0], t_init=305.0,
    )
    assert abs(tc - 305.32) < 1.0
    assert abs(pc - 4872.0) / 4872.0 < 0.02
    assert vc > 0.0


def test_critical_point_binary_between_pures():
    tc, pc, vc = e.critical_point_py(
        e.CubicEos.PR1976, [190.564, 305.32], [4599.0, 4872.0],
        [0.0115, 0.0995], [0.5, 0.5], t_init=250.0,
    )
    assert 190.564 < tc < 305.32
    assert pc > 0.0


def test_adiabatic_flash_round_trip():
    """Compute a stream enthalpy at a known T*, then recover T* from it."""
    tcs = [469.7, 617.7]
    pcs = [3370.0, 2110.0]
    om = [0.252, 0.4884]
    cp = [[1.5, 4.0e-2, -1.2e-5, 0.0, 0.0], [2.0, 8.0e-2, -2.4e-5, 0.0, 0.0]]
    z = [0.5, 0.5]
    p, t_star = 500.0, 450.0
    # Phase split at T* via isothermal flash, then the weighted stream enthalpy.
    beta, x, y, k, _, two_phase = e.flash_pt(
        tcs, pcs, om, z, t_star, p, vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=e.CubicEos.PR1976, liquid_eos=e.CubicEos.PR1976,
    )
    assert two_phase
    hL, _ = e.mixture_phase_enthalpy_entropy(
        e.CubicEos.PR1976, e.MixingRule.Classical, tcs, pcs, om, cp, x, [],
        t_star, p, "liquid",
    )
    hV, _ = e.mixture_phase_enthalpy_entropy(
        e.CubicEos.PR1976, e.MixingRule.Classical, tcs, pcs, om, cp, y, [],
        t_star, p, "vapor",
    )
    h_feed = beta * hV + (1 - beta) * hL
    t, betaA, xA, yA, hA = e.flash_adiabatic_py(
        e.CubicEos.PR1976, tcs, pcs, om, cp, z, p, h_feed, 420.0, 480.0,
    )
    assert abs(t - t_star) < 0.1, f"recovered T={t} vs {t_star}"


def test_fit_kij_recovers_known_value():
    """Fit kij to synthetic data generated at k*=0.13 (CO2/n-butane, PR)."""
    tcs = [304.13, 425.12]
    pcs = [7377.0, 3796.0]
    om = [0.2239, 0.200]
    psat = [[4.86, 1147.0, -8.0], [4.35, 2277.0, -30.0]]
    k_true = 0.13
    data = []
    for x1 in [0.2, 0.4, 0.5, 0.6, 0.8]:
        p, _, _ = e.bubble_pressure_py(
            tcs, pcs, om, [x1, 1 - x1], 310.0,
            vapor_kind="cubic", liquid_kind="cubic",
            vapor_eos=e.CubicEos.PR1976, liquid_eos=e.CubicEos.PR1976,
            kij=[[0.0, k_true], [k_true, 0.0]], psat_coeffs=psat, tol=1e-9,
        )
        data.append((310.0, x1, p))
    kij, sse, rmse = e.fit_kij_py(
        e.CubicEos.PR1976, tcs, pcs, om, psat, data, k_lo=-0.05, k_hi=0.3,
    )
    assert abs(kij - k_true) < 1e-3
    assert rmse < 1e-2


def test_fit_aij_recovers_van_laar():
    """Fit van Laar (A12, A21) to synthetic bubble-pressure data."""
    tcs = [512.6, 647.1]
    pcs = [8097.0, 22064.0]
    om = [0.564, 0.344]
    psat = [[5.20, 3200.0, -35.0], [5.11, 3800.0, -46.0]]
    a12t, a21t = 0.85, 0.52
    data = []
    for x1 in [0.2, 0.35, 0.5, 0.65, 0.8]:
        p, _, _ = e.bubble_pressure_py(
            tcs, pcs, om, [x1, 1 - x1], 298.15,
            vapor_kind="ideal", liquid_kind="activity",
            liquid_activity=e.ActivityModel.VanLaar,
            aij=[[0.0, a12t], [a21t, 0.0]], psat_coeffs=psat, tol=1e-9,
        )
        data.append((298.15, x1, p))
    a12, a21, sse, rmse, iters = e.fit_aij_py(
        e.ActivityModel.VanLaar, tcs, pcs, om, psat, data, 0.3, 0.3,
    )
    assert abs(a12 - a12t) < 5e-3 and abs(a21 - a21t) < 5e-3
    assert rmse < 1e-2
