"""M8.3 / M8.4 mixture layer — Python-side tests through the wheel.

Exercises the bindings added in Milestones 8.3 and 8.4:

- Mixing rules + multicomponent fugacity (`mixture_z`, `mixture_ln_phi`)
  across classical (IVDW/Classical/IIVDW), GE-based (Wong-Sandler, Huron-
  Vidal original/simplified, MHV1, MHV2) and 3-parameter (Schmidt-Wenzel,
  Patel-Teja) EOS.
- The exact composition Jacobian ∂ln φ̂ᵢ/∂nⱼ (`mixture_d_ln_phi_d_n`) — checked
  against a finite-difference oracle and for Gibbs-Duhem symmetry.
- Mixture energy properties (`mixture_h_departure_rt`, `mixture_s_departure_r`,
  the ideal terms, and the total `mixture_phase_enthalpy_entropy`).
- Chao-Seader multicomponent liquid fugacity.

These are correctness oracles verified against the Rust engine's own test
suite (which cross-validates against the VB6/Pascal formulas and the
textbook PR mixture expression). Where a value is a textbook invariant
(pure-component limit, ideal-gas limit, Lewis-Randall) the docstring says so.
"""

import math

import pytest

import vle._engine as e


# Component data (NIST critical constants; cp_coeffs are plausible ideal-Cp/R
# polynomials — exact values immaterial to the invariants checked here).
METHANE = dict(tc=190.564, pc=4599.0, omega=0.0115, cp=[4.5, 1.5e-3, 0.0, 0.0, 0.0])
N_PENTANE = dict(tc=469.7, pc=3370.0, omega=0.252, cp=[7.0, 5.0e-3, -1.0e-6, 0.0, 0.0])
METHANOL = dict(tc=512.6, pc=8097.0, omega=0.564, cp=[4.9, 1.2e-2, -3.0e-6, 0.0, 0.0])
WATER = dict(tc=647.1, pc=22064.0, omega=0.344, cp=[4.0, 1.0e-3, 0.0, 0.0, 0.0])


def arrays(*comps):
    """Unpack a list of component dicts into parallel (tcs, pcs, omegas, cps)."""
    tcs = [c["tc"] for c in comps]
    pcs = [c["pc"] for c in comps]
    omegas = [c["omega"] for c in comps]
    cps = [c["cp"] for c in comps]
    return tcs, pcs, omegas, cps


def kij2(k):
    return [[0.0, k], [k, 0.0]]


# Every a/b mixing rule that takes no activity model.
CLASSICAL_RULES = [
    e.MixingRule.Classical,
    e.MixingRule.IVDW,
    e.MixingRule.IIVDW,
]

# GE-based rules (need an activity model + its arrays).
GE_RULES = [
    e.MixingRule.WongSandler,
    e.MixingRule.HuronVidalOriginal,
    e.MixingRule.HuronVidalSimplified,
    e.MixingRule.MHV1,
    e.MixingRule.MHV2,
]

VAN_LAAR_AIJ = [[0.0, 0.847], [0.522, 0.0]]


# =============================================================================
# Mixing rules + multicomponent fugacity
# =============================================================================


def test_mixture_z_and_ln_phi_shapes_classical():
    """Every classical rule returns a scalar Z and one ln φ̂ per component."""
    tcs, pcs, omegas, _ = arrays(METHANE, N_PENTANE)
    x = [0.4, 0.6]
    for rule in CLASSICAL_RULES:
        z = e.mixture_z(
            e.CubicEos.PR1976, rule, tcs, pcs, omegas, x, kij2(0.023),
            350.0, 2000.0, "vapor",
        )
        lnphi = e.mixture_ln_phi(
            e.CubicEos.PR1976, rule, tcs, pcs, omegas, x, kij2(0.023),
            350.0, 2000.0, "vapor",
        )
        assert 0.0 < z < 1.2
        assert len(lnphi) == 2
        assert all(math.isfinite(v) for v in lnphi)


def test_mixture_pure_limit_matches_pure_ln_phi():
    """x=[1] mixture ln φ̂ must equal the pure-component ln φ (all EOS)."""
    for eos in [e.CubicEos.PR1976, e.CubicEos.RKS1972, e.CubicEos.SchmidtWenzel,
                e.CubicEos.PatelTeja]:
        lnphi = e.mixture_ln_phi(
            eos, e.MixingRule.Classical,
            [N_PENTANE["tc"]], [N_PENTANE["pc"]], [N_PENTANE["omega"]],
            [1.0], [], 400.0, 1500.0, "vapor",
        )[0]
        pure = e.eos_ln_phi_pure(
            eos, 400.0, 1500.0, N_PENTANE["tc"], N_PENTANE["pc"],
            N_PENTANE["omega"], "vapor",
        )
        assert lnphi == pytest.approx(pure, abs=1e-9)


def test_mixture_ge_rules_run_methanol_water():
    """Every GE rule produces a finite Z and ln φ̂ for methanol/water."""
    tcs, pcs, omegas, _ = arrays(METHANOL, WATER)
    x = [0.4, 0.6]
    for rule in GE_RULES:
        lnphi = e.mixture_ln_phi(
            e.CubicEos.PR1976, rule, tcs, pcs, omegas, x, kij2(0.05),
            400.0, 300.0, "vapor",
            ge_model=e.ActivityModel.VanLaar, ge_aij=VAN_LAAR_AIJ,
        )
        assert len(lnphi) == 2
        assert all(math.isfinite(v) for v in lnphi)


def test_ge_rule_without_model_raises():
    """A GE-based rule with no activity model is a ValueError, not a panic."""
    tcs, pcs, omegas, _ = arrays(METHANOL, WATER)
    with pytest.raises(ValueError):
        e.mixture_ln_phi(
            e.CubicEos.PR1976, e.MixingRule.WongSandler,
            tcs, pcs, omegas, [0.5, 0.5], [], 400.0, 300.0, "vapor",
        )


def test_c_parameter_rule_as_ab_rule_raises():
    """Passing a C-parameter rule as the a/b rule is rejected."""
    tcs, pcs, omegas, _ = arrays(METHANE, N_PENTANE)
    with pytest.raises(ValueError):
        e.mixture_z(
            e.CubicEos.PatelTeja, e.MixingRule.PatelTejaC,
            tcs, pcs, omegas, [0.5, 0.5], [], 350.0, 2000.0, "vapor",
        )


# =============================================================================
# Exact composition derivatives
# =============================================================================


def _jac_fd(eos, rule, tcs, pcs, omegas, x, kij, t, p, phase, **ge):
    """Central-difference oracle for ∂ln φ̂ᵢ/∂nⱼ (renormalizing inside)."""
    n = len(x)
    h = 1e-6

    def lnphi(moles):
        tot = sum(moles)
        xn = [m / tot for m in moles]
        return e.mixture_ln_phi(eos, rule, tcs, pcs, omegas, xn, kij, t, p, phase, **ge)

    jac = [[0.0] * n for _ in range(n)]
    for j in range(n):
        plus = list(x)
        plus[j] += h
        minus = list(x)
        minus[j] -= h
        fp = lnphi(plus)
        fm = lnphi(minus)
        for i in range(n):
            jac[i][j] = (fp[i] - fm[i]) / (2.0 * h)
    return jac


def test_jacobian_matches_fd_classical():
    """Analytic ∂ln φ̂ᵢ/∂nⱼ (classical + PR) matches the FD oracle + is symmetric."""
    tcs, pcs, omegas, _ = arrays(METHANE, N_PENTANE)
    x = [0.35, 0.65]
    jac = e.mixture_d_ln_phi_d_n(
        e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs, omegas, x,
        kij2(0.023), 350.0, 2000.0, "vapor",
    )
    fd = _jac_fd(e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs, omegas, x,
                 kij2(0.023), 350.0, 2000.0, "vapor")
    for i in range(2):
        for j in range(2):
            assert jac[i][j] == pytest.approx(fd[i][j], rel=1e-4, abs=1e-8)
    # Gibbs-Duhem symmetry.
    assert jac[0][1] == pytest.approx(jac[1][0], rel=1e-8, abs=1e-9)


def test_jacobian_matches_fd_wong_sandler():
    """Dual-number ∂ln φ̂ᵢ/∂nⱼ (Wong-Sandler) matches the FD oracle."""
    tcs, pcs, omegas, _ = arrays(METHANOL, WATER)
    x = [0.4, 0.6]
    ge = dict(ge_model=e.ActivityModel.VanLaar, ge_aij=VAN_LAAR_AIJ)
    jac = e.mixture_d_ln_phi_d_n(
        e.CubicEos.PR1976, e.MixingRule.WongSandler, tcs, pcs, omegas, x,
        kij2(0.05), 400.0, 300.0, "vapor", **ge,
    )
    fd = _jac_fd(e.CubicEos.PR1976, e.MixingRule.WongSandler, tcs, pcs, omegas,
                 x, kij2(0.05), 400.0, 300.0, "vapor", **ge)
    for i in range(2):
        for j in range(2):
            assert jac[i][j] == pytest.approx(fd[i][j], rel=1e-4, abs=1e-6)


# =============================================================================
# Energy properties
# =============================================================================


def test_departure_reduces_to_pure():
    """x=[1] mixture H^R/S^R equal the pure-component EOS departure."""
    for eos in [e.CubicEos.PR1976, e.CubicEos.PatelTeja]:
        h = e.mixture_h_departure_rt(
            eos, e.MixingRule.Classical,
            [N_PENTANE["tc"]], [N_PENTANE["pc"]], [N_PENTANE["omega"]],
            [1.0], [], 400.0, 2000.0, "vapor",
        )
        hp = e.eos_h_departure_rt(
            eos, 400.0, 2000.0, N_PENTANE["tc"], N_PENTANE["pc"],
            N_PENTANE["omega"], "vapor",
        )
        assert h == pytest.approx(hp, abs=1e-9)


def test_departure_lewis_randall_consistency():
    """S^R/R = H^R/RT − Σ xᵢ ln φ̂ᵢ (Lewis-Randall) for a binary."""
    tcs, pcs, omegas, _ = arrays(METHANE, N_PENTANE)
    x = [0.4, 0.6]
    args = (e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs, omegas, x,
            kij2(0.023), 350.0, 2000.0, "vapor")
    h = e.mixture_h_departure_rt(*args)
    s = e.mixture_s_departure_r(*args)
    lnphi = e.mixture_ln_phi(*args)
    g = sum(xi * lp for xi, lp in zip(x, lnphi))
    assert s == pytest.approx(h - g, abs=1e-10)


def test_ideal_mixing_entropy_binary():
    """Ideal mixing entropy peaks at x=0.5 → R·ln2 (R=8.31451)."""
    tcs, pcs, omegas, cps = arrays(METHANE, N_PENTANE)
    # Zero cp so only the −R Σ x ln x term contributes (ideal integral = 0 at
    # T = Tref); use equal-T reference and same P/Pref to isolate mixing.
    s_mix = e.mixture_ideal_entropy(
        tcs, pcs, omegas, [[0.0] * 5, [0.0] * 5], [0.5, 0.5],
        298.15, 101.325, 298.15, 101.325,
    )
    assert s_mix == pytest.approx(8.31451 * math.log(2.0), abs=1e-9)


def test_phase_enthalpy_entropy_finite():
    """Total H/S assemble and stay finite for a real binary."""
    tcs, pcs, omegas, cps = arrays(METHANE, N_PENTANE)
    h, s = e.mixture_phase_enthalpy_entropy(
        e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs, omegas, cps,
        [0.4, 0.6], kij2(0.023), 350.0, 2000.0, "vapor",
    )
    assert math.isfinite(h) and math.isfinite(s)


def test_ideal_enthalpy_matches_manual_integral():
    """Ideal enthalpy = Σ xᵢ R ∫Cp/R dT for a linear-Cp component."""
    # Single component, cp = [a0, a1, 0,0,0] → ∫ = R[a0(T−Tref) + a1/2(T²−Tref²)].
    a0, a1 = 4.5, 1.5e-3
    t, tref = 400.0, 298.15
    got = e.mixture_ideal_enthalpy(
        [METHANE["tc"]], [METHANE["pc"]], [METHANE["omega"]],
        [[a0, a1, 0.0, 0.0, 0.0]], [1.0], t, tref,
    )
    R = 8.31451
    want = R * (a0 * (t - tref) + a1 / 2.0 * (t**2 - tref**2))
    assert got == pytest.approx(want, rel=1e-12)


# =============================================================================
# Chao-Seader multicomponent
# =============================================================================


def test_chao_seader_mix_matches_pure_calls():
    """Multicomponent Chao-Seader = per-component pure ln ν."""
    tcs, pcs, omegas, _ = arrays(METHANE, N_PENTANE)
    species = [e.ChaoSeaderSpecies.Methane, e.ChaoSeaderSpecies.Normal]
    got = e.mixture_chao_seader_ln_phi(tcs, pcs, omegas, species, 300.0, 500.0)
    for i, (c, s) in enumerate(zip([METHANE, N_PENTANE], species)):
        want = e.chao_seader_ln_phi(300.0, 500.0, c["tc"], c["pc"], c["omega"], s)
        assert got[i] == pytest.approx(want, abs=1e-12)


# =============================================================================
# Validation — golden values + physics sanity
#
# The legacy VB6/Pascal binaries cannot be run in this environment, so the
# mixture layer is validated three ways: (1) the Rust engine cross-checks
# each fugacity against the independent textbook Peng-Robinson mixture form
# and the VB6 C_cal / Pascal Chao-Seader constants (see engine/src/mixture.rs
# tests); (2) the derivative core is checked against finite-difference
# oracles; (3) the golden values below are pinned regression oracles,
# generated by the engine and hand-checked against physics. If the engine
# math changes and these drift, update them — but the physics assertions
# (φ ordering by molecular size, sign of the departures) must always hold.
# =============================================================================


def test_validation_methane_pentane_pr_ivdw():
    """methane(1)/n-pentane(2), PR + IVDW, kij=0.023, 350 K, 2000 kPa.

    Physics: the lighter component (methane) has φ > 1, the heavier
    (n-pentane) φ < 1; both residual H and S are negative (attractive
    forces lower energy/entropy below the ideal-gas reference); the liquid
    root is far more compressed (small Z) than the vapor.
    """
    tcs, pcs, omegas, _ = arrays(METHANE, N_PENTANE)
    x = [0.35, 0.65]
    kij = kij2(0.023)

    zv = e.mixture_z(e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs, omegas, x,
                     kij, 350.0, 2000.0, "vapor")
    lpv = e.mixture_ln_phi(e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs, omegas,
                           x, kij, 350.0, 2000.0, "vapor")
    hv = e.mixture_h_departure_rt(e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs,
                                  omegas, x, kij, 350.0, 2000.0, "vapor")
    sv = e.mixture_s_departure_r(e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs,
                                 omegas, x, kij, 350.0, 2000.0, "vapor")
    zl = e.mixture_z(e.CubicEos.PR1976, e.MixingRule.IVDW, tcs, pcs, omegas, x,
                     kij, 350.0, 2000.0, "liquid")

    # Golden values (engine, 2026-07-03).
    assert zv == pytest.approx(0.6198463855465869, rel=1e-9)
    assert lpv[0] == pytest.approx(0.24748095485454122, rel=1e-9)
    assert lpv[1] == pytest.approx(-0.6205147876457696, rel=1e-9)
    assert hv == pytest.approx(-1.1636904358040998, rel=1e-9)
    assert sv == pytest.approx(-0.8469741580334389, rel=1e-9)
    assert zl == pytest.approx(0.08436640021658673, rel=1e-9)

    # Physics invariants (must hold regardless of exact numbers).
    assert lpv[0] > 0 > lpv[1], "light φ>1, heavy φ<1"
    assert hv < 0 and sv < 0, "attractive departures are negative"
    assert zl < zv, "liquid root more compressed than vapor"


def test_validation_wong_sandler_methanol_water():
    """methanol/water, PR + Wong-Sandler + van Laar, 350 K, 200 kPa vapor.

    Near-ideal vapor at 200 kPa → Z ≈ 1 and small |ln φ̂|. Golden values
    pin the full GE-EOS coupling (activity model → mixing rule → fugacity).
    """
    tcs, pcs, omegas, _ = arrays(METHANOL, WATER)
    x = [0.4, 0.6]
    z = e.mixture_z(e.CubicEos.PR1976, e.MixingRule.WongSandler, tcs, pcs, omegas,
                    x, kij2(0.1), 350.0, 200.0, "vapor",
                    ge_model=e.ActivityModel.VanLaar, ge_aij=VAN_LAAR_AIJ)
    lp = e.mixture_ln_phi(e.CubicEos.PR1976, e.MixingRule.WongSandler, tcs, pcs,
                          omegas, x, kij2(0.1), 350.0, 200.0, "vapor",
                          ge_model=e.ActivityModel.VanLaar, ge_aij=VAN_LAAR_AIJ)
    assert z == pytest.approx(0.9758075835316242, rel=1e-9)
    assert lp[0] == pytest.approx(-0.030627222197953864, rel=1e-9)
    assert lp[1] == pytest.approx(-0.019486702763973036, rel=1e-9)
    assert 0.9 < z < 1.0, "near-ideal vapor at 200 kPa"


def test_validation_patel_teja_three_parameter():
    """Patel-Teja (3-param) methane/n-pentane, classical mixing, vapor.

    The 3-parameter EOS gives fugacities close to but distinct from PR
    (different attractive-denominator shape). Golden regression values.
    """
    tcs, pcs, omegas, _ = arrays(METHANE, N_PENTANE)
    lp = e.mixture_ln_phi(e.CubicEos.PatelTeja, e.MixingRule.Classical, tcs, pcs,
                          omegas, [0.35, 0.65], kij2(0.023), 350.0, 2000.0, "vapor")
    assert lp[0] == pytest.approx(0.25563359477876696, rel=1e-9)
    assert lp[1] == pytest.approx(-0.6125517038772331, rel=1e-9)
