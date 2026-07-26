//! Transport and interfacial properties — viscosity, thermal conductivity,
//! surface tension.
//!
//! IF97 (`R7-97`) covers only the *thermodynamic* surface. Viscosity, thermal
//! conductivity and surface tension are separate IAPWS releases, each with its
//! own reference constants and its own range of validity:
//!
//! | Property | Release | Form |
//! |---|---|---|
//! | Viscosity `μ` | **R12-08** | `μ = μ₀(T̄)·μ₁(T̄,ρ̄)·μ₂` |
//! | Thermal conductivity `λ` | **R15-11** | `λ = λ₀(T̄)·λ₁(T̄,ρ̄) + λ₂(T̄,ρ̄)` |
//! | Surface tension `σ` | **R1-76(2014)** | `σ = B τ^μ (1 + b τ)` |
//!
//! ## Industrial vs. scientific formulation
//!
//! Both transport releases define themselves against **IAPWS-95**, the
//! scientific formulation, and then give a separate *industrial* recommendation
//! that takes its thermodynamic inputs from **IF97** instead. This crate
//! implements the **industrial** one throughout, because IF97 is what it has:
//!
//! - R12-08 §3 — set `μ₂ = 1` and take the density from IF97.
//! - R15-11 §3 — take `c_p`, `κ = c_p/c_v` and `ζ = (∂ρ̄/∂p̄)_T̄` from IF97, and
//!   compute `ζ` at the reference temperature `T_R = 1.5·T*` from the explicit
//!   polynomial of its Eq. (25) rather than from an equation of state. That
//!   substitution matters practically: `T_R = 970.644 K` is above region 3's
//!   ceiling, so evaluating it directly would mean inverting region 2 for a
//!   density on every call.
//!
//! The two are *not* interchangeable, and this is why the acceptance tests
//! below assert against R15-11 Tables **7–9** (the industrial tables, keyed on
//! IF97 regions 1/2/3) rather than its Tables 4–5, which were generated with
//! IAPWS-95 and which an IF97-based implementation cannot reproduce exactly.
//!
//! ## Units
//!
//! Public functions return **SI**: `μ` in **Pa·s**, `λ` in **W/(m·K)**, `σ` in
//! **N/m**. The IAPWS releases work in µPa·s, mW/(m·K) and mN/m; that
//! conversion happens once, at the boundary, exactly as the kPa↔MPa conversion
//! does elsewhere in the crate.
//!
//! ## Two-phase states
//!
//! Transport properties are **per-phase**. There is no meaningful
//! quality-weighted viscosity of a boiling mixture — the two phases differ by
//! more than an order of magnitude and the mixture value depends on the flow
//! regime, which is not thermodynamics. So a two-phase state reports the
//! saturated-liquid and saturated-vapor values separately (see
//! [`crate::SatProps`]) and never an average.
//!
//! ## References
//!
//! - IAPWS. *Release on the IAPWS Formulation 2008 for the Viscosity of
//!   Ordinary Water Substance*; R12-08, 2008.
//! - IAPWS. *Release on the IAPWS Formulation 2011 for the Thermal
//!   Conductivity of Ordinary Water Substance*; R15-11, 2011.
//! - IAPWS. *Revised Release on Surface Tension of Ordinary Water Substance*;
//!   R1-76(2014), 2014.

use crate::regions::Region;
use crate::{RHO_C, SteamError, T_C};

// ── Reference constants shared by R12-08 and R15-11 ──────────────────────

/// Reference pressure `p*`, **kPa** (R12-08 Eq. 3 / R15-11 Eq. 2).
const P_STAR_KPA: f64 = 22_064.0;
/// Specific gas constant used by the transport releases, **kJ/(kg·K)**.
///
/// R15-11 Eq. (6) — this is the IAPWS-95 value, `0.46151805`, and it is
/// **not** IF97's `R = 0.461526` ([`crate::R`]). The dimensionless heat
/// capacity `c̄p = cp/R` in the critical enhancement must use this one.
const R_TRANSPORT: f64 = 0.461_518_05;

// ── R12-08: viscosity ────────────────────────────────────────────────────

/// Dilute-gas coefficients `Hᵢ` (R12-08 Table 1).
const VISC_H: [f64; 4] = [1.677_52, 2.204_62, 0.636_656_4, -0.241_605];

/// Finite-density coefficients `Hᵢⱼ` as `(i, j, Hᵢⱼ)` (R12-08 Table 2).
/// Entries omitted from that table are identically zero.
const VISC_HIJ: [(usize, usize, f64); 21] = [
    (0, 0, 5.200_94e-1),
    (1, 0, 8.508_95e-2),
    (2, 0, -1.083_74),
    (3, 0, -2.895_55e-1),
    (0, 1, 2.225_31e-1),
    (1, 1, 9.991_15e-1),
    (2, 1, 1.887_97),
    (3, 1, 1.266_13),
    (5, 1, 1.205_73e-1),
    (0, 2, -2.813_78e-1),
    (1, 2, -9.068_51e-1),
    (2, 2, -7.724_79e-1),
    (3, 2, -4.898_37e-1),
    (4, 2, -2.570_40e-1),
    (0, 3, 1.619_13e-1),
    (1, 3, 2.573_99e-1),
    (0, 4, -3.253_72e-2),
    (3, 4, 6.984_52e-2),
    (4, 5, 8.721_02e-3),
    (3, 6, -4.356_73e-3),
    (5, 6, -5.932_64e-4),
];

/// Viscosity from density and temperature, the form the release is written in.
///
/// # Arguments
/// * `rho` — Density in **kg/m³**.
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Dynamic viscosity in **µPa·s** (the release's own `μ*`), with the critical
/// enhancement `μ₂` set to 1 per the industrial recommendation of R12-08 §3.
///
/// Ref: IAPWS R12-08 Eqs. (10)–(12).
pub(crate) fn viscosity_bar(rho: f64, t: f64) -> f64 {
    let (t_bar, rho_bar) = (t / T_C, rho / RHO_C);
    // μ₀ = 100 √T̄ / Σ Hᵢ/T̄ⁱ  (Eq. 11)
    let mut denom = 0.0;
    let mut t_pow = 1.0;
    for h in VISC_H {
        denom += h / t_pow;
        t_pow *= t_bar;
    }
    let mu0 = 100.0 * t_bar.sqrt() / denom;
    // μ₁ = exp[ ρ̄ Σᵢ (1/T̄ − 1)ⁱ Σⱼ Hᵢⱼ (ρ̄ − 1)ʲ ]  (Eq. 12)
    let (a, b) = (1.0 / t_bar - 1.0, rho_bar - 1.0);
    let mut sum = 0.0;
    for (i, j, h) in VISC_HIJ {
        sum += h * a.powi(i as i32) * b.powi(j as i32);
    }
    mu0 * (rho_bar * sum).exp()
}

// ── R15-11: thermal conductivity ─────────────────────────────────────────

/// Dilute-gas coefficients `L_k` (R15-11 Table 1).
const TC_L: [f64; 5] = [
    2.443_221e-3,
    1.323_095e-2,
    6.770_357e-3,
    -3.454_586e-3,
    4.096_266e-4,
];

/// Finite-density coefficients `L_ij` (R15-11 Table 2), row `i`, column `j`.
const TC_LIJ: [[f64; 6]; 5] = [
    [
        1.603_973_57,
        -0.646_013_523,
        0.111_443_906,
        0.102_997_357,
        -0.050_412_363_4,
        0.006_098_592_58,
    ],
    [
        2.337_718_42,
        -2.788_437_78,
        1.536_161_67,
        -0.463_045_512,
        0.083_282_701_9,
        -0.007_192_012_45,
    ],
    [
        2.196_505_29,
        -4.545_807_85,
        3.557_772_44,
        -1.409_449_78,
        0.275_418_278,
        -0.020_593_881_6,
    ],
    [
        -1.210_513_78,
        1.608_129_89,
        -0.621_178_141,
        0.071_637_322_4,
        0.0,
        0.0,
    ],
    [
        -2.720_337_0,
        4.575_863_31,
        -3.183_692_45,
        1.116_834_8,
        -0.192_683_05,
        0.012_913_842,
    ],
];

/// Coefficients `A_ij` of the industrial `ζ(T_R, ρ̄)` polynomial
/// (R15-11 Table 6), indexed `[j][i]` — `j` selects the density range.
const TC_AIJ: [[f64; 6]; 5] = [
    [
        6.537_868_071_995_16,
        -5.611_499_549_233_48,
        3.396_241_673_613_25,
        -2.274_926_297_308_78,
        10.263_185_466_270_9,
        1.978_150_503_315_19,
    ],
    [
        6.527_177_592_817_99,
        -6.308_169_833_875_75,
        8.083_792_854_925_95,
        -9.822_405_101_976_03,
        12.135_841_379_139_5,
        -5.543_496_645_712_95,
    ],
    [
        5.355_005_298_961_24,
        -3.964_156_899_254_46,
        8.919_902_089_187_95,
        -12.033_872_950_579_0,
        9.194_948_651_943_02,
        -2.168_662_744_797_12,
    ],
    [
        1.552_259_599_066_81,
        0.464_621_290_821_181,
        8.932_373_748_614_79,
        -11.032_196_006_112_6,
        6.167_809_999_333_60,
        -0.965_458_722_086_812,
    ],
    [
        1.119_999_264_199_94,
        0.595_748_562_571_649,
        9.889_525_650_789_20,
        -10.325_505_114_704_0,
        4.668_612_944_574_14,
        -0.503_243_546_373_828,
    ],
];

/// Upper `ρ̄` bounds selecting the `j` range of [`TC_AIJ`] (R15-11 Eq. 26).
const TC_RHO_BANDS: [f64; 4] = [0.310_559_006, 0.776_397_516, 1.242_236_025, 1.863_354_037];

// Critical-region constants (R15-11 Table 3).
/// `Λ`, the critical-enhancement prefactor.
const TC_BIG_LAMBDA: f64 = 177.851_4;
/// `q_D⁻¹` in **nm** — the reference wave number's inverse.
const TC_QD_INV_NM: f64 = 0.40;
/// Critical exponent `ν`.
const TC_NU: f64 = 0.630;
/// Critical exponent `γ`.
const TC_GAMMA: f64 = 1.239;
/// Amplitude `ξ₀` in **nm**.
const TC_XI0_NM: f64 = 0.13;
/// Amplitude `Γ₀`.
const TC_GAMMA0: f64 = 0.06;
/// Reference temperature `T̄_R` for the compressibility difference.
const TC_TR_BAR: f64 = 1.5;
/// Ceiling applied to `ζ` and `c̄p` where IF97 misbehaves near the critical
/// point (R15-11 §3.4, footnote 2).
const TC_CLAMP: f64 = 1e13;

/// The dilute-gas factor `λ₀(T̄)` (R15-11 Eq. 16), dimensionless.
fn lambda0(t_bar: f64) -> f64 {
    let mut denom = 0.0;
    let mut t_pow = 1.0;
    for l in TC_L {
        denom += l / t_pow;
        t_pow *= t_bar;
    }
    t_bar.sqrt() / denom
}

/// The finite-density factor `λ₁(T̄, ρ̄)` (R15-11 Eq. 17), dimensionless.
fn lambda1(t_bar: f64, rho_bar: f64) -> f64 {
    let (a, b) = (1.0 / t_bar - 1.0, rho_bar - 1.0);
    let mut sum = 0.0;
    for (i, row) in TC_LIJ.iter().enumerate() {
        let mut inner = 0.0;
        for (j, l) in row.iter().enumerate() {
            inner += l * b.powi(j as i32);
        }
        sum += a.powi(i as i32) * inner;
    }
    (rho_bar * sum).exp()
}

/// `ζ(T̄_R, ρ̄)` from the industrial polynomial (R15-11 Eq. 25).
///
/// The scientific formulation evaluates the compressibility at
/// `T_R = 1.5·T* = 970.644 K` from IAPWS-95. This closed form replaces that,
/// which is what makes the industrial route self-contained — 970.644 K is
/// above region 3's ceiling, so an IF97 implementation would otherwise have to
/// invert region 2 for a density on every single call.
fn zeta_ref(rho_bar: f64) -> f64 {
    let j = TC_RHO_BANDS.iter().filter(|&&b| rho_bar > b).count();
    let mut denom = 0.0;
    for (i, a) in TC_AIJ[j].iter().enumerate() {
        denom += a * rho_bar.powi(i as i32);
    }
    1.0 / denom
}

/// The critical enhancement `λ₂` (R15-11 Eqs. 18–24), dimensionless.
///
/// # Arguments
/// * `t_bar`, `rho_bar` — reduced temperature and density.
/// * `cp_bar` — `c_p/R` with R15-11's own gas constant, already clamped.
/// * `kappa` — `c_p/c_v`.
/// * `zeta` — `(∂ρ̄/∂p̄)_T̄` at the state, already clamped.
/// * `mu_bar` — dimensionless viscosity `μ/μ*` at the state.
fn lambda2(t_bar: f64, rho_bar: f64, cp_bar: f64, kappa: f64, zeta: f64, mu_bar: f64) -> f64 {
    // Δχ = ρ̄ [ ζ(T̄,ρ̄) − ζ(T̄_R,ρ̄)·T̄_R/T̄ ]   (Eq. 23)
    let delta_chi = rho_bar * (zeta - zeta_ref(rho_bar) * TC_TR_BAR / t_bar);
    if delta_chi <= 0.0 {
        return 0.0; // Eq. (23) note: negative Δχ is set to zero
    }
    // ξ = ξ₀ (Δχ/Γ₀)^(ν/γ)   (Eq. 22), in nm
    let xi = TC_XI0_NM * (delta_chi / TC_GAMMA0).powf(TC_NU / TC_GAMMA);
    // y = q_D ξ   (Eq. 20)
    let y = xi / TC_QD_INV_NM;
    if y < 1.2e-7 {
        return 0.0; // Eq. (21): avoid truncation error for tiny y
    }
    // Z(y)   (Eq. 19)
    let inv_k = 1.0 / kappa;
    let term1 = (1.0 - inv_k) * y.atan() + inv_k * y;
    let term2 = 1.0 - (-1.0 / (1.0 / y + y * y / (3.0 * rho_bar * rho_bar))).exp();
    let z = 2.0 / (std::f64::consts::PI * y) * (term1 - term2);
    // λ₂ = Λ ρ̄ c̄p T̄ Z(y) / μ̄   (Eq. 18)
    TC_BIG_LAMBDA * rho_bar * cp_bar * t_bar * z / mu_bar
}

/// The three factors of `λ` at a fully-specified state, dimensionless
/// (i.e. in units of `λ* = 1 mW/(m·K)`).
///
/// Returning the breakdown rather than only the total is what lets the
/// acceptance tests assert against R15-11 Tables 7–9 term by term — those
/// tables publish `λ₀`, `λ₁` and `λ₂` separately, so a mistake in any one of
/// them is localized instead of being hidden in a total that happens to land
/// close.
pub(crate) fn thermal_conductivity_parts(
    t: f64,
    rho: f64,
    cp: f64,
    cv: f64,
    drho_dp: f64,
    enhanced: bool,
) -> (f64, f64, f64) {
    let (t_bar, rho_bar) = (t / T_C, rho / RHO_C);
    let l0 = lambda0(t_bar);
    let l1 = lambda1(t_bar, rho_bar);
    if !enhanced {
        return (l0, l1, 0.0);
    }
    // IF97 misbehaves for ζ and c̄p very close to the critical point; R15-11
    // §3.4 footnote 2 prescribes exactly this clamp.
    let clamp = |v: f64| {
        if !(0.0..=TC_CLAMP).contains(&v) {
            TC_CLAMP
        } else {
            v
        }
    };
    let zeta = clamp(drho_dp * P_STAR_KPA / RHO_C);
    let cp_bar = clamp(cp / R_TRANSPORT);
    let l2 = lambda2(t_bar, rho_bar, cp_bar, cp / cv, zeta, viscosity_bar(rho, t));
    (l0, l1, l2)
}

// ── R1-76(2014): surface tension ─────────────────────────────────────────

/// Surface tension of the liquid–vapor interface.
///
/// # Arguments
/// * `t` — Temperature in **K**, valid from the triple point to `Tc`
///   (and reasonably extrapolated down to about 248 K in the supercooled
///   region, per the 2014 revision).
///
/// # Returns
/// Surface tension in **N/m**.
///
/// `σ = B τ^μ (1 + b τ)` with `τ = 1 − T/Tc`, `B = 235.8 mN/m`, `b = −0.625`,
/// `μ = 1.256` (R1-76(2014)).
pub fn surface_tension(t: f64) -> Result<f64, SteamError> {
    if !(200.0..=T_C).contains(&t) {
        return Err(SteamError::OutOfSaturationRange(t));
    }
    let tau = 1.0 - t / T_C;
    // 235.8 mN/m → N/m at the boundary, per the crate's units rule.
    Ok(235.8e-3 * tau.powf(1.256) * (1.0 - 0.625 * tau))
}

// ── Region dispatch for the thermodynamic inputs ─────────────────────────

/// `(∂ρ/∂p)_T` at a single-phase state, **(kg/m³)/kPa**.
///
/// Analytic in both routes, never a difference quotient (the repo's standing
/// rule): the Gibbs regions differentiate `v = R·T·γ_π/p*` once more in `π`,
/// and region 3 inverts `∂p/∂ρ = R·T·(2δφ_δ + δ²φ_δδ)`.
pub(crate) fn drho_dp(t: f64, p_kpa: f64, region: Region) -> Result<f64, SteamError> {
    Ok(match region {
        Region::One => crate::region1::drho_dp(t, p_kpa),
        Region::Two => crate::region2::drho_dp(t, p_kpa),
        Region::Three => {
            let rho = crate::region3::density_tp(t, p_kpa)?;
            1.0 / crate::region3::dp_drho(rho, t)
        }
        Region::Five => crate::region5::drho_dp(t, p_kpa),
        Region::Saturated => return Err(SteamError::TwoPhase("(∂ρ/∂p) is per-phase")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_abs_diff_eq, assert_relative_eq};

    /// R12-08 **Table 4** — the industrial verification points (`μ₂ = 1`),
    /// all eleven, at the published precision.
    ///
    /// Asserted **absolutely** to half a unit in the last printed place: the
    /// table gives six decimals throughout, so `14.538324` carries only eight
    /// significant figures and a relative tolerance tight enough for
    /// `889.735100` would be asserting on digits the standard never printed.
    #[test]
    fn r12_08_table_4() {
        for (t, rho, expected) in [
            (298.15, 998.0, 889.735_100),
            (298.15, 1200.0, 1_437.649_467),
            (373.15, 1000.0, 307.883_622),
            (433.15, 1.0, 14.538_324),
            (433.15, 1000.0, 217.685_358),
            (873.15, 1.0, 32.619_287),
            (873.15, 100.0, 35.802_262),
            (873.15, 600.0, 77.430_195),
            (1173.15, 1.0, 44.217_245),
            (1173.15, 100.0, 47.640_433),
            (1173.15, 400.0, 64.154_608),
        ] {
            assert_abs_diff_eq!(viscosity_bar(rho, t), expected, epsilon = 5e-7);
        }
    }

    /// R15-11 **Table 4** — the `λ₂ = 0` points, which exercise `λ₀` and `λ₁`
    /// without the critical-enhancement machinery. The two liquid points at
    /// 298.15 K are exactly the case where `Δχ < 0`, so the release itself
    /// states `λ₂ = 0` there.
    #[test]
    fn r15_11_table_4() {
        for (t, rho, expected) in [
            (298.15, 0.0, 18.434_188_3),
            (298.15, 998.0, 607.712_868),
            (298.15, 1200.0, 799.038_144),
            (873.15, 0.0, 79.103_465_9),
        ] {
            let (l0, l1, _) = thermal_conductivity_parts(t, rho, 1.0, 1.0, 0.0, false);
            assert_relative_eq!(l0 * l1, expected, max_relative = 1e-8);
        }
    }

    /// The surface-tension equation against R1-76(2014) Table 1.
    #[test]
    fn r1_76_table_1() {
        // (t °C, σ_calc mN/m) — the release's own "calculated" column.
        for (t_c, expected) in [
            (0.01, 75.65),
            (20.0, 72.74),
            (50.0, 67.94),
            (100.0, 58.91),
            (150.0, 48.74),
            (200.0, 37.67),
            (250.0, 26.04),
            (300.0, 14.36),
            (350.0, 3.67),
        ] {
            let sigma = surface_tension(t_c + 273.15).unwrap() * 1e3; // N/m → mN/m
            assert_relative_eq!(sigma, expected, epsilon = 0.005);
        }
    }

    /// Sanity checks against values every engineer knows by heart, plus the
    /// qualitative behaviour a correct implementation must show. These catch
    /// a unit-conversion slip at the public boundary, which the dimensionless
    /// table tests above cannot see.
    #[test]
    fn liquid_water_at_room_conditions_is_familiar() {
        let st = crate::SteamState::tp(293.15, 101.325).unwrap();
        // Water at 20 °C: ~1.0 mPa·s (1 cP), ~0.598 W/(m·K), Pr ≈ 7.
        assert_relative_eq!(st.viscosity().unwrap(), 1.0016e-3, max_relative = 2e-3);
        assert_relative_eq!(
            st.thermal_conductivity().unwrap(),
            0.5984,
            max_relative = 2e-3
        );
        assert_relative_eq!(st.prandtl().unwrap(), 7.0, max_relative = 2e-2);
        // ν = μ/ρ ≈ 1.004e-6 m²/s — the textbook kinematic viscosity of water.
        assert_relative_eq!(
            st.kinematic_viscosity().unwrap(),
            1.004e-6,
            max_relative = 5e-3
        );
        // Surface tension at 20 °C is the classic 72.7 mN/m.
        assert_relative_eq!(
            surface_tension(293.15).unwrap(),
            72.74e-3,
            max_relative = 1e-3
        );
    }

    /// Across the saturation line the liquid is far more viscous and far more
    /// conductive than the vapor — the physical reason a quality-weighted
    /// average would be meaningless.
    #[test]
    fn saturated_phases_differ_by_orders_of_magnitude() {
        let sat = crate::sat_p(101.325).unwrap();
        let (mu_f, mu_g) = (sat.mu_f(), sat.mu_g());
        let (k_f, k_g) = (sat.k_f().unwrap(), sat.k_g().unwrap());
        assert!(mu_f > 20.0 * mu_g, "mu_f={mu_f:e} mu_g={mu_g:e}");
        assert!(k_f > 20.0 * k_g, "k_f={k_f:e} k_g={k_g:e}");
        // Saturated steam at 1 atm: ~12.3 µPa·s, ~0.0250 W/(m·K).
        assert_relative_eq!(mu_g, 12.3e-6, max_relative = 2e-2);
        assert_relative_eq!(k_g, 0.0250, max_relative = 3e-2);
        assert!(sat.sigma().unwrap() > 0.0);
    }

    /// σ must vanish at the critical point and be positive just below it.
    #[test]
    fn surface_tension_endpoints() {
        assert_relative_eq!(surface_tension(T_C).unwrap(), 0.0, epsilon = 1e-15);
        assert!(surface_tension(T_C - 0.01).unwrap() > 0.0);
        assert!(matches!(
            surface_tension(700.0),
            Err(SteamError::OutOfSaturationRange(_))
        ));
    }
}
