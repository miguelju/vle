"""M7.1 pure-component layer — Python-side smoke tests.

Exercises the bindings added in Milestone 7.1:

- Cubic EOS family constants, α(Tr) + dα/dTr, Z-factor, ln(φ), H^R/RT, S^R/R
- Antoine saturation pressure + analytical dPsat/dT
- Virial equation (Pitzer): B⁰, B¹, B(T) pure and mixture, ln(φ) mixture

The variants deferred to M7.2 / M7.3 / M7.4 raise ``NotImplementedError``
when called. The tests for those gates are at the bottom of this file.

Reference numbers come from the VLE engine itself running on Miguel's
laptop on 2026-05-24. If you change the underlying engine code and these
numbers drift, update the assertions — they're correctness oracles, not
hard truths from the literature. Where a value can be sanity-checked
against textbook physics (ideal-gas limit, α(Tr=1)=1, etc.) the test
spells that out in the docstring.
"""

import math

import pytest

import vle._engine as e


# Convenient component data — chosen to span sub/super-critical and
# polar/non-polar in a small number of variants.

METHANE = dict(tc=190.564, pc=4599.0, omega=0.0115)       # mildly polar, very low ω
N_PENTANE = dict(tc=469.7, pc=3370.0, omega=0.252)        # textbook "ω-rich" alkane


# =============================================================================
# Cubic EOS — α and dα/dTr
# =============================================================================


def test_eos_family_constants_pr_rks_vdw() -> None:
    """The k1/k2/OmA/OmB tuples come straight from McommonFunctions.bas:273."""
    pr = e.eos_family_constants(e.CubicEos.PR1976)
    assert pr == pytest.approx((2.0, -1.0, 0.457235528921382, 0.0777960739038885))

    rks = e.eos_family_constants(e.CubicEos.RKS1972)
    assert rks == pytest.approx((1.0, 0.0, 0.427480233540341, 0.0866403499649577))

    vdw = e.eos_family_constants(e.CubicEos.VdW1870)
    # 27/64 and 1/8 are analytical fractions.
    assert vdw == pytest.approx((0.0, 0.0, 27 / 64, 1 / 8))


@pytest.mark.parametrize(
    "eos",
    [e.CubicEos.PR1976, e.CubicEos.RKS1972, e.CubicEos.RK1949, e.CubicEos.VdW1870],
)
def test_alpha_at_tr_one(eos) -> None:
    """α(Tr=1) = 1 for every variant in the deployable core."""
    a = e.eos_alpha(eos, 1.0, N_PENTANE["omega"])
    assert a == pytest.approx(1.0, abs=1e-12)


@pytest.mark.parametrize(
    "eos",
    [e.CubicEos.PR1976, e.CubicEos.RKS1972, e.CubicEos.RK1949, e.CubicEos.VdW1870],
)
@pytest.mark.parametrize("tr", [0.5, 0.85, 1.0, 1.5])
def test_analytical_d_alpha_matches_numerical(eos, tr) -> None:
    """The analytical derivative agrees with a central-difference oracle.

    Per CLAUDE.md "Algorithm Choices", analytical derivatives are the
    production code path and numerical ones exist only as test oracles.
    This is that oracle, evaluated through the PyO3 binding so the FFI
    boundary itself is part of the regression coverage.
    """
    h = 1e-6
    omega = N_PENTANE["omega"]
    analytical = e.eos_d_alpha_d_tr(eos, tr, omega)
    numerical = (
        e.eos_alpha(eos, tr + h, omega) - e.eos_alpha(eos, tr - h, omega)
    ) / (2.0 * h)
    if abs(analytical) < 1e-10:
        assert abs(analytical - numerical) < 1e-6
    else:
        assert numerical == pytest.approx(analytical, rel=1e-5)


# =============================================================================
# M7.2 — the remaining twelve two-parameter α variants
# =============================================================================

# A synthetic "polar" component carrying every fitted parameter the M7.2
# variants might read. The exact numbers don't matter — the oracle tests
# check internal consistency of α and its analytical derivative. g > 1
# keeps |1 − Tr|^(g−1) smooth at Tr = 1 for the ATmn exponential variants.
POLAR = dict(omega=0.344, zc=0.229, m=0.45, n=0.12, g=1.5, prsv_k1=0.07)

# The twelve variants shipped in M7.2 (the OL family is M7.4, the 3-param
# Pascal EOS is M7.3).
M72_VARIANTS = [
    e.CubicEos.Berth1899,
    e.CubicEos.VdWAda1984,
    e.CubicEos.RKSGD1978,
    e.CubicEos.RKSL1997,
    e.CubicEos.RP1978,
    e.CubicEos.PRL1997,
    e.CubicEos.VdWVald1989,
    e.CubicEos.RKSmn1980,
    e.CubicEos.RKSATmn1995,
    e.CubicEos.PRATmng1997,
    e.CubicEos.PRMmn1989,
    e.CubicEos.PRSV1986,
]


@pytest.mark.parametrize("eos", M72_VARIANTS)
def test_m72_alpha_at_tr_one(eos) -> None:
    """α(Tr=1) = 1 for every M7.2 variant — they all reduce to the
    critical-point value at Tc by construction."""
    a = e.eos_alpha_ex(eos, 1.0, **POLAR)
    assert a == pytest.approx(1.0, abs=1e-12)


@pytest.mark.parametrize("eos", M72_VARIANTS)
@pytest.mark.parametrize("tr", [0.55, 0.7, 0.85, 1.1, 1.4])
def test_m72_analytical_d_alpha_matches_numerical(eos, tr) -> None:
    """Analytical dα/dTr agrees with a central-difference oracle, through
    the extended binding (so the full parameter set crosses the FFI)."""
    h = 1e-6
    analytical = e.eos_d_alpha_d_tr_ex(eos, tr, **POLAR)
    numerical = (
        e.eos_alpha_ex(eos, tr + h, **POLAR) - e.eos_alpha_ex(eos, tr - h, **POLAR)
    ) / (2.0 * h)
    if abs(analytical) < 1e-8:
        assert abs(analytical - numerical) < 1e-6
    else:
        assert numerical == pytest.approx(analytical, rel=1e-5)


def test_eos_alpha_ex_superset_of_eos_alpha() -> None:
    """For an ω-only variant, the extended binding with default extras
    must match the simple binding bit-for-bit."""
    for eos in [e.CubicEos.Berth1899, e.CubicEos.RKSGD1978, e.CubicEos.PR1976]:
        for tr in [0.6, 1.0, 1.5]:
            assert e.eos_alpha_ex(eos, tr, 0.252) == e.eos_alpha(eos, tr, 0.252)


def test_prsv_k1_is_additive_correction() -> None:
    """PRSV with K₁=0 collapses to the κ₀(ω) form; a nonzero K₁ shifts α."""
    w = 0.344
    a0 = e.eos_alpha_ex(e.CubicEos.PRSV1986, 0.6, w, prsv_k1=0.0)
    a1 = e.eos_alpha_ex(e.CubicEos.PRSV1986, 0.6, w, prsv_k1=0.07)
    # κ₀ form, evaluated independently.
    kappa0 = 0.378893 + 1.4897153 * w - 0.17131848 * w**2 + 0.0196554 * w**3
    inner = 1.0 + kappa0 * (1.0 - 0.6**0.5)
    assert a0 == pytest.approx(inner**2, abs=1e-12)
    assert a1 != pytest.approx(a0)


# =============================================================================
# Z-factor, ln(φ), H^R/RT, S^R/R
# =============================================================================


def test_z_factor_methane_supercritical_pr() -> None:
    """Methane at 300 K, 5 MPa (T > Tc) — one stable root, plausible Z ≈ 0.9."""
    z = e.eos_z_factor(
        e.CubicEos.PR1976, 300.0, 5000.0,
        METHANE["tc"], METHANE["pc"], METHANE["omega"], "vapor",
    )
    assert 0.85 < z < 1.0


def test_z_factor_n_pentane_two_phase_pr() -> None:
    """n-Pentane at 400 K (Tr ≈ 0.85), 1500 kPa → distinct liquid and vapor roots."""
    z_l = e.eos_z_factor(
        e.CubicEos.PR1976, 400.0, 1500.0,
        N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"], "liquid",
    )
    z_v = e.eos_z_factor(
        e.CubicEos.PR1976, 400.0, 1500.0,
        N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"], "vapor",
    )
    assert z_l < 0.1 < z_v
    assert z_v > 0.4


def test_ln_phi_ideal_gas_limit_all_core_eos() -> None:
    """At very low P, ln(φ) → 0 for every EOS — the ideal-gas limit."""
    for eos in [e.CubicEos.PR1976, e.CubicEos.RKS1972, e.CubicEos.RK1949, e.CubicEos.VdW1870]:
        ln_phi = e.eos_ln_phi_pure(
            eos, 300.0, 0.1,
            METHANE["tc"], METHANE["pc"], METHANE["omega"], "vapor",
        )
        assert abs(ln_phi) < 1e-3, f"{eos!r}: ln(φ) = {ln_phi}"


def test_h_dep_and_s_dep_self_consistent() -> None:
    """G^R/RT = H^R/RT - S^R/R for pure (Lewis-Randall identity)."""
    for eos in [e.CubicEos.PR1976, e.CubicEos.RKS1972]:
        ln_phi = e.eos_ln_phi_pure(
            eos, 400.0, 1500.0,
            N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"], "vapor",
        )
        h_dep = e.eos_h_departure_rt(
            eos, 400.0, 1500.0,
            N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"], "vapor",
        )
        s_dep = e.eos_s_departure_r(
            eos, 400.0, 1500.0,
            N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"], "vapor",
        )
        # Identity: G^R/(RT) = H^R/(RT) - S^R/R, and G^R/(RT) = ln(φ) for pure.
        assert ln_phi == pytest.approx(h_dep - s_dep, abs=1e-10)


# =============================================================================
# Antoine saturation
# =============================================================================


def test_antoine_psat_basic() -> None:
    """At the critical temperature the Antoine form reduces to Pc·exp(a1).

    With a1=0, a2=0, a3 arbitrary, we should recover Pc exactly.
    """
    pc = 22064.0  # water-like
    psat = e.antoine_psat(500.0, pc, [0.0, 0.0, 1.0])
    assert psat == pytest.approx(pc)


def test_antoine_d_psat_matches_numerical() -> None:
    """Closed-form dPsat/dT agrees with central differences for the Antoine form."""
    pc = 4599.0
    coeffs = [10.5, 4000.0, -20.0]
    t = 280.0
    analytical = e.antoine_d_psat_dt(t, pc, coeffs)
    h = 1e-3
    numerical = (e.antoine_psat(t + h, pc, coeffs) - e.antoine_psat(t - h, pc, coeffs)) / (2 * h)
    assert numerical == pytest.approx(analytical, rel=1e-7)


def test_antoine_bad_coeffs_raises_value_error() -> None:
    with pytest.raises(ValueError, match="3 Antoine coefficients"):
        e.antoine_psat(300.0, 1000.0, [1.0, 2.0])  # only 2 coeffs


# =============================================================================
# Virial — pure + mixture
# =============================================================================


def test_pitzer_b0_b1_at_critical() -> None:
    """B⁰(Tr=1) = 0.083 - 0.422 = -0.339; B¹(Tr=1) = 0.139 - 0.172 = -0.033."""
    assert e.virial_pitzer_b0(1.0) == pytest.approx(-0.339, abs=1e-6)
    assert e.virial_pitzer_b1(1.0) == pytest.approx(-0.033, abs=1e-6)


def test_virial_z_ideal_limit() -> None:
    """At very low P, Z → 1 for the truncated virial regardless of T."""
    z = e.virial_z(METHANE["tc"], METHANE["pc"], METHANE["omega"], 300.0, 0.1)
    assert z == pytest.approx(1.0, abs=1e-4)


def test_virial_d_b_d_t_matches_numerical() -> None:
    """Analytical dB/dT agrees with central differences."""
    tc, pc, omega = N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"]
    t = 350.0
    analytical = e.virial_d_b_d_t_pure(tc, pc, omega, t)
    h = 0.1
    numerical = (
        e.virial_b_pure(tc, pc, omega, t + h) - e.virial_b_pure(tc, pc, omega, t - h)
    ) / (2 * h)
    assert numerical == pytest.approx(analytical, rel=1e-4)


def test_virial_mix_consistency_with_pure() -> None:
    """For x_i = 1 (pure i), the mixture B_mix should equal B_i."""
    tc, pc, omega = METHANE["tc"], METHANE["pc"], METHANE["omega"]
    t = 350.0
    b_pure = e.virial_b_pure(tc, pc, omega, t)
    b_mix = e.virial_b_mix_py([tc, 500.0], [pc, 3000.0], [omega, 0.3], [1.0, 0.0], t)
    assert b_mix == pytest.approx(b_pure, rel=1e-10)


def test_virial_ln_phi_mix_sums_correctly() -> None:
    """Σ xᵢ·ln(φᵢ_mix) should be close to a 'mixture average' ln(φ) under quadratic mixing.

    Specifically, with the Lewis-Randall quadratic mixing rule:
    Σ xᵢ·ln(φᵢ_mix) = (P/RT) · B_mix.
    """
    tcs = [METHANE["tc"], N_PENTANE["tc"]]
    pcs = [METHANE["pc"], N_PENTANE["pc"]]
    ws = [METHANE["omega"], N_PENTANE["omega"]]
    xs = [0.6, 0.4]
    t, p = 350.0, 1000.0
    ln_phi_i = e.virial_ln_phi_mix(tcs, pcs, ws, xs, t, p)
    weighted = sum(x * lp for x, lp in zip(xs, ln_phi_i))
    b_mix = e.virial_b_mix_py(tcs, pcs, ws, xs, t)
    # (P/RT) factor with our 1e-3 unit shift — see virial.rs.
    R = 8.31451
    expected = b_mix * p / (1000.0 * R * t)
    assert weighted == pytest.approx(expected, rel=1e-10)


# =============================================================================
# Deferred-variant gates
# =============================================================================


# =============================================================================
# M7.3 — three-parameter EOS (Schmidt-Wenzel, Patel-Teja, PT-USB) + Chao-Seader
# Ref (4): Da Silva & Báez (1989), legacy/pascal/TERMOII.PAS.
# =============================================================================

THREE_PARAM = [e.CubicEos.SchmidtWenzel, e.CubicEos.PatelTeja, e.CubicEos.PatelTejaUSB]


@pytest.mark.parametrize("eos", THREE_PARAM)
def test_three_param_alpha_unity_at_critical(eos) -> None:
    """α(Tr=1)=1 — the EOS prefactor is folded into Ω_a."""
    assert e.eos_alpha(eos, 1.0, N_PENTANE["omega"]) == pytest.approx(1.0, abs=1e-12)


@pytest.mark.parametrize("eos", THREE_PARAM)
@pytest.mark.parametrize("tr", [0.5, 0.7, 0.9, 1.2, 1.5])
def test_three_param_d_alpha_matches_numerical(eos, tr) -> None:
    """Analytical dα/dTr vs central difference, away from the SW Tr=1 kink."""
    h = 1e-6
    w = N_PENTANE["omega"]
    analytical = e.eos_d_alpha_d_tr(eos, tr, w)
    numerical = (e.eos_alpha(eos, tr + h, w) - e.eos_alpha(eos, tr - h, w)) / (2 * h)
    if abs(analytical) < 1e-10:
        assert abs(analytical - numerical) < 1e-6
    else:
        assert numerical == pytest.approx(analytical, rel=1e-5)


@pytest.mark.parametrize("eos", THREE_PARAM)
def test_three_param_ideal_gas_limit(eos) -> None:
    """As P→0, Z→1 and ln φ→0 through the Python bindings."""
    tc, pc, w = N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"]
    z = e.eos_z_factor(eos, 400.0, 1e-3, tc, pc, w, "vapor")
    assert z == pytest.approx(1.0, abs=1e-4)
    lnphi = e.eos_ln_phi_pure(eos, 400.0, 1e-3, tc, pc, w, "vapor")
    assert abs(lnphi) < 1e-4


@pytest.mark.parametrize("eos", THREE_PARAM)
def test_three_param_entropy_consistency(eos) -> None:
    """S^R/R = H^R/RT − ln φ (Lewis-Randall) through the bindings."""
    args = (eos, 400.0, 500.0, N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"], "vapor")
    s = e.eos_s_departure_r(*args)
    h = e.eos_h_departure_rt(*args)
    g = e.eos_ln_phi_pure(*args)
    assert math.isfinite(s) and s == pytest.approx(h - g, abs=1e-9)


def test_schmidt_wenzel_tr1_entropy_finite() -> None:
    """Faithful + guarded: SW entropy at Tr=1 is finite (legacy gave NaN)."""
    tc, pc, w = N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"]
    s = e.eos_s_departure_r(e.CubicEos.SchmidtWenzel, tc, 500.0, tc, pc, w, "vapor")
    assert math.isfinite(s)


def test_chao_seader_pure_fugacity() -> None:
    """Chao-Seader ln(ν) is finite for each coefficient set + matched component."""
    ln_normal = e.chao_seader_ln_phi(
        0.7 * N_PENTANE["tc"], 500.0, N_PENTANE["tc"], N_PENTANE["pc"], N_PENTANE["omega"],
        e.ChaoSeaderSpecies.Normal,
    )
    assert math.isfinite(ln_normal) and abs(ln_normal) < 50.0
    ln_methane = e.chao_seader_ln_phi(
        0.7 * METHANE["tc"], 500.0, METHANE["tc"], METHANE["pc"], METHANE["omega"],
        e.ChaoSeaderSpecies.Methane,
    )
    ln_hydrogen = e.chao_seader_ln_phi(
        0.7 * 33.2, 500.0, 33.2, 1300.0, -0.216, e.ChaoSeaderSpecies.Hydrogen,
    )
    assert math.isfinite(ln_methane) and math.isfinite(ln_hydrogen)


# =============================================================================
# M7.4 — advanced saturation models + OL-family α (binding layer).
# Ref (4): Da Silva & Báez (1989); OL family: Olivera et al. (1998).
# =============================================================================

# n-pentane with the saturation data the M7.4 models need.
PENTANE_SAT = dict(tc=469.7, pc=3370.0, omega=0.252, tb=309.2)
PENTANE_ANTOINE = [6.738, 3165.0, 0.0]  # reduced Antoine ln(P/Pc)=a1−a2/(a3+T)
CORRELATIONS = [e.SatPressureModel.Riedel, e.SatPressureModel.Muller, e.SatPressureModel.RPM]
OL_FAMILY = [e.CubicEos.VdWOL1998, e.CubicEos.RKOL1998, e.CubicEos.PROL1998]


@pytest.mark.parametrize("model", CORRELATIONS)
def test_sat_correlations_subcritical(model) -> None:
    ps = e.sat_psat(model, 350.0, PENTANE_SAT["tc"], PENTANE_SAT["pc"], PENTANE_SAT["omega"], PENTANE_SAT["tb"])
    assert 0.0 < ps < PENTANE_SAT["pc"]
    pr = e.sat_reduced_psat(model, 350.0, PENTANE_SAT["tc"], PENTANE_SAT["pc"], PENTANE_SAT["omega"], PENTANE_SAT["tb"])
    assert 0.0 < pr < 1.0


@pytest.mark.parametrize("model", CORRELATIONS)
def test_sat_correlations_hit_one_atm_at_tb(model) -> None:
    ps = e.sat_psat(model, PENTANE_SAT["tb"], PENTANE_SAT["tc"], PENTANE_SAT["pc"], PENTANE_SAT["omega"], PENTANE_SAT["tb"])
    assert abs(ps - 101.325) / 101.325 < 0.05


def test_sat_maxwell_and_boiling_point() -> None:
    pm = e.sat_maxwell(e.CubicEos.PR1976, 350.0, PENTANE_SAT["tc"], PENTANE_SAT["pc"], PENTANE_SAT["omega"], PENTANE_ANTOINE)
    assert math.isfinite(pm) and pm > 0.0
    # Antoine boiling-point round trip: Psat(Tb(P)) == P.
    p = 200.0
    tb = e.boiling_temperature(e.SatPressureModel.Antoine, p, PENTANE_SAT["tc"], PENTANE_SAT["pc"], 0.0, 0.0, PENTANE_ANTOINE)
    assert abs(e.antoine_psat(tb, PENTANE_SAT["pc"], PENTANE_ANTOINE) - p) / p < 1e-6


def test_poynting_factor() -> None:
    assert e.poynting_factor(500.0, 500.0, 350.0, 116.0) == pytest.approx(1.0, abs=1e-12)
    assert e.poynting_factor(2000.0, 500.0, 350.0, 116.0) > 1.0


@pytest.mark.parametrize("eos", OL_FAMILY)
def test_ol_alpha_finite_and_unity_band(eos) -> None:
    """OL α via the dedicated binding (needs saturation data); finite + positive."""
    a = e.eos_alpha_ol(eos, 0.8, PENTANE_SAT["tc"], PENTANE_SAT["pc"], PENTANE_SAT["omega"],
                       e.SatPressureModel.Antoine, 0.0, PENTANE_ANTOINE)
    assert math.isfinite(a) and a > 0.0


@pytest.mark.parametrize("eos", OL_FAMILY)
def test_ol_d_alpha_matches_numerical(eos) -> None:
    """Analytical OL dα/dTr (Antoine sat model → analytical) vs central diff."""
    h = 1e-6
    args = (PENTANE_SAT["tc"], PENTANE_SAT["pc"], PENTANE_SAT["omega"], e.SatPressureModel.Antoine, 0.0, PENTANE_ANTOINE)
    tr = 0.8
    analytical = e.eos_d_alpha_d_tr_ol(eos, tr, *args)
    numerical = (e.eos_alpha_ol(eos, tr + h, *args) - e.eos_alpha_ol(eos, tr - h, *args)) / (2 * h)
    assert numerical == pytest.approx(analytical, rel=1e-4)


def test_antoine_still_works() -> None:
    """Positive control: the M7.1 Antoine binding is unchanged."""
    assert e.antoine_psat(300.0, 4599.0, [9.0, 1500.0, -30.0]) > 0
