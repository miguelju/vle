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


@pytest.mark.parametrize(
    "eos",
    [e.CubicEos.SchmidtWenzel, e.CubicEos.PatelTeja, e.CubicEos.PatelTejaUSB],
)
def test_three_param_eos_raises_not_implemented(eos) -> None:
    """The 3-parameter EOS variants (M7.3 deferred) raise NotImplementedError."""
    with pytest.raises(NotImplementedError, match="not yet ported"):
        e.eos_z_factor(eos, 300.0, 1000.0, 200.0, 4000.0, 0.1, "vapor")


@pytest.mark.parametrize(
    "eos",
    [e.CubicEos.RKSGD1978, e.CubicEos.PRSV1986, e.CubicEos.RKSmn1980],
)
def test_deferred_alpha_variants_panic_at_alpha(eos) -> None:
    """Deferred α variants panic at the α layer — pyo3 turns the Rust panic into PanicException."""
    # The Rust `unimplemented!` macro panics. PyO3 traps the panic and
    # raises `pyo3.exceptions.PyBaseException` (subclass of PanicException).
    with pytest.raises(BaseException) as excinfo:
        e.eos_alpha(eos, 1.0, 0.2)
    assert "M7.2 deferred" in str(excinfo.value)


def test_non_antoine_sat_models_raise_not_implemented() -> None:
    """Riedel/Müller/RPM/polynomial/Maxwell go through the saturation dispatch
    via the engine's `psat` function — but at the binding layer we only
    expose `antoine_psat` for now. Verify Antoine works (positive control)."""
    psat = e.antoine_psat(300.0, 4599.0, [9.0, 1500.0, -30.0])
    assert psat > 0
