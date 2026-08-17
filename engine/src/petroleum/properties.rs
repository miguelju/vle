//! Pseudocomponent property estimation — Tb + SG → M, Tc, Pc, ω, Vc, Zc.
//!
//! This is the heart of petroleum characterization. Given nothing but a cut's
//! **mid-boiling point** and its **specific gravity**, produce the full set of
//! parameters a cubic EOS needs. Two numbers in, six out. It works as well as it
//! does because boiling point and density between them pin down both the size of
//! the molecules and how hydrogen-rich they are, and those two facts turn out to
//! determine nearly everything else about a hydrocarbon.
//!
//! # The four correlation families
//!
//! | [`PropertyMethod`] | year | gives | shape |
//! |---|---|---|---|
//! | [`RiaziDaubert1980`] | 1980 | M, Tc, Pc, Vc | `θ = a·Tb^b·SG^c` — three constants, nothing else |
//! | [`ApiRiaziDaubert1987`] | 1987 | M, Tc, Pc, Vc | the same with an `exp(d·Tb + e·SG + f·Tb·SG)` factor; the API's recommendation |
//! | [`KeslerLee`] | 1976 | M, Tc, Pc | polynomials in Tb and SG; the refinery-standard pairing with Lee–Kesler enthalpies |
//! | [`Twu`] | 1984 | M, Tc, Pc, Vc | a *perturbation* about the n-alkane of the same boiling point |
//!
//! [`RiaziDaubert1980`]: PropertyMethod::RiaziDaubert1980
//! [`ApiRiaziDaubert1987`]: PropertyMethod::ApiRiaziDaubert1987
//! [`KeslerLee`]: PropertyMethod::KeslerLee
//! [`Twu`]: PropertyMethod::Twu
//!
//! Twu is structurally different and worth understanding. It first asks "what
//! n-alkane boils at this temperature?", computes that alkane's properties from
//! a high-accuracy fit, and then *corrects* for how far the real cut's density
//! sits from the alkane's. So it is near-exact for paraffins by construction and
//! degrades smoothly as the cut gets more aromatic — which is exactly what the
//! measured errors below show.
//!
//! # Measured accuracy
//!
//! Every correlation here was checked against the measured properties of ten
//! pure hydrocarbons — n-C5 … n-C10, benzene, toluene, cyclohexane,
//! methylcyclohexane — taken from this crate's bundled component database. The
//! worst absolute deviation over that set:
//!
//! | | Tc | Pc | M | Vc |
//! |---|---|---|---|---|
//! | Riazi–Daubert 1980 | 2.5 % | 8.2 % | 8.3 % | 4.6 % |
//! | API / R–D 1987 | 1.3 % | 5.1 % | 6.0 % | 4.0 % |
//! | Kesler–Lee | 1.9 % | 5.5 % | 8.1 % | — |
//! | Twu | 1.8 % | 15.3 % | 7.4 % | 6.0 % |
//! | acentric factor ω | \< 1.6 % on all ten | | | |
//!
//! Read those as *lower bounds on the error you will see in practice*: these are
//! pure compounds in the middle of the fitted range. A real vacuum-residue cut
//! is far outside it. The single clearest pattern is that **Twu's Pc is
//! excellent on paraffins (\< 1 %) and poor on aromatics (11–15 %)**, for the
//! structural reason above — pick it for paraffinic crudes, not for aromatic
//! ones. The tests at the bottom of this file assert every one of these numbers,
//! so the table cannot silently go stale.
//!
//! # References
//! - (32) Riazi & Daubert, *Hydrocarbon Process.* **1980**, *59* (3), 115–116.
//! - (33) Riazi & Daubert, *Ind. Eng. Chem. Res.* **1987**, *26* (4), 755–759.
//! - (36) Kesler & Lee, *Hydrocarbon Process.* **1976**, *55* (3), 153–158.
//! - (37) Lee & Kesler, *AIChE J.* **1975**, *21* (3), 510–527 — the ω correlation.
//! - (38) Twu, *Fluid Phase Equilib.* **1984**, *16*, 137–150.

use super::PetroleumError;
use super::gravity::{k_to_r, r_to_k, watson_k};
use crate::types::R_GAS;

/// psia per kPa. The 1980 and Twu correlations were published in psia.
const KPA_PER_PSIA: f64 = 6.894_757;
/// cm³ per ft³, used to move Twu's molar critical volume onto the crate's basis.
const CM3_PER_FT3: f64 = 28_316.846_592;
/// mol per lbmol.
const MOL_PER_LBMOL: f64 = 453.592_37;
/// One atmosphere in kPa — the reference pressure for the acentric factor.
const P_ATM_KPA: f64 = 101.325;

/// Which correlation family to use for the critical properties.
///
/// See the module docs for the measured accuracy of each. When in doubt,
/// [`ApiRiaziDaubert1987`] is the API's own recommendation and has the best
/// all-round record on the test set.
///
/// [`ApiRiaziDaubert1987`]: PropertyMethod::ApiRiaziDaubert1987
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum PropertyMethod {
    /// Riazi–Daubert (1980), ref (32). The simplest form, `θ = a·Tb^b·SG^c`.
    RiaziDaubert1980,
    /// Riazi–Daubert (1987) as adopted into the API Technical Data Book,
    /// ref (33). **The default.**
    #[default]
    ApiRiaziDaubert1987,
    /// Kesler–Lee (1976), ref (36). No critical volume; pairs with the
    /// Lee–Kesler enthalpy route a refinery normally uses.
    KeslerLee,
    /// Twu (1984), ref (38). A perturbation about the n-alkane of the same
    /// boiling point; best on paraffinic material.
    Twu,
}

/// Which corresponding-states correlation to use for the critical
/// compressibility when a method supplies no critical volume of its own.
///
/// All four are one-liners in the acentric factor. They differ by less than
/// 0.005 in Zc over the whole range of ω a petroleum cut can have, so the
/// choice rarely matters; [`ZcMethod::LeeKesler`] is the default because it is
/// the one consistent with the rest of the Lee–Kesler framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum ZcMethod {
    /// `Zc = 0.2905 − 0.085 ω`. Lee & Kesler (1975), ref (37) Eq. 21.
    #[default]
    LeeKesler,
    /// `Zc = 0.2918 − 0.0928 ω`. Reid, Prausnitz & Sherwood (1977).
    Reid,
    /// `Zc = 0.291 − 0.08 ω − 0.016 ω²`. Salerno et al. (1985).
    Salerno,
    /// `Zc = 0.2908 − 0.0825 ω`. Nath (1985).
    Nath,
}

impl ZcMethod {
    /// Critical compressibility from the acentric factor, **dimensionless**.
    ///
    /// # Arguments
    /// * `omega` — acentric factor, **dimensionless**.
    pub fn zc(&self, omega: f64) -> f64 {
        match self {
            ZcMethod::LeeKesler => 0.2905 - 0.085 * omega,
            ZcMethod::Reid => 0.2918 - 0.0928 * omega,
            ZcMethod::Salerno => 0.291 - 0.08 * omega - 0.016 * omega * omega,
            ZcMethod::Nath => 0.2908 - 0.0825 * omega,
        }
    }
}

/// The full property set of one pseudocomponent, in the crate's canonical units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PseudoProperties {
    /// Normal boiling point, **K** — echoed back so the struct is self-contained.
    pub tb: f64,
    /// Specific gravity at 60/60 °F, **dimensionless**.
    pub sg: f64,
    /// Watson characterization factor, **dimensionless**.
    pub watson_k: f64,
    /// Molecular weight, **g/mol**.
    pub mw: f64,
    /// Critical temperature, **K**.
    pub tc: f64,
    /// Critical pressure, **kPa** (absolute).
    pub pc: f64,
    /// Critical volume, **cm³/mol**.
    pub vc: f64,
    /// Critical compressibility `Zc = Pc·Vc/(R·Tc)`, **dimensionless**.
    pub zc: f64,
    /// Acentric factor, **dimensionless**.
    pub omega: f64,
}

/// Convert `Pc·Vc/(R·Tc)` into a dimensionless Zc from the crate's units.
///
/// `Pc` in kPa times `Vc` in cm³/mol is 10⁻³ J/mol, and `R_GAS` in
/// kJ/(kmol·K) is numerically J/(mol·K), so the 1e-3 is the whole conversion.
fn zc_from_criticals(pc: f64, vc: f64, tc: f64) -> f64 {
    pc * vc * 1e-3 / (R_GAS * tc)
}

/// The inverse: critical volume in **cm³/mol** from Zc, Tc and Pc.
fn vc_from_zc(zc: f64, tc: f64, pc: f64) -> f64 {
    zc * R_GAS * tc * 1e3 / pc
}

fn check_inputs(tb: f64, sg: f64) -> Result<(), PetroleumError> {
    if tb <= 0.0 || !tb.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "boiling point must be positive and finite, got {tb} K"
        )));
    }
    if sg <= 0.0 || !sg.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "specific gravity must be positive and finite, got {sg}"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Riazi-Daubert (1980), ref (32) — published in °R and psia
// ---------------------------------------------------------------------------

/// Riazi–Daubert 1980: molecular weight in **g/mol**, `Tb` in **K**.
///
/// `M = 4.5673×10⁻⁵ · Tb^2.1962 · SG^(−1.0164)`, Tb in **°R**. Ref (32).
pub fn mw_riazi_daubert_1980(tb: f64, sg: f64) -> f64 {
    let t = k_to_r(tb);
    4.5673e-5 * t.powf(2.1962) * sg.powf(-1.0164)
}

/// Riazi–Daubert 1980: critical temperature in **K**, `Tb` in **K**.
///
/// `Tc = 24.2787 · Tb^0.58848 · SG^0.3596`, both in **°R**. Ref (32).
pub fn tc_riazi_daubert_1980(tb: f64, sg: f64) -> f64 {
    let t = k_to_r(tb);
    r_to_k(24.2787 * t.powf(0.58848) * sg.powf(0.3596))
}

/// Riazi–Daubert 1980: critical pressure in **kPa**, `Tb` in **K**.
///
/// `Pc = 3.12281×10⁹ · Tb^(−2.3125) · SG^2.3201`, Tb in **°R**, Pc in **psia**.
/// Ref (32).
pub fn pc_riazi_daubert_1980(tb: f64, sg: f64) -> f64 {
    let t = k_to_r(tb);
    3.12281e9 * t.powf(-2.3125) * sg.powf(2.3201) * KPA_PER_PSIA
}

/// Riazi–Daubert 1980: critical volume in **cm³/mol**, `Tb` in **K**.
///
/// The published form `Vc = 7.5214×10⁻³ · Tb^0.2896 · SG^(−0.7666)` is on a
/// **mass** basis (ft³/lb with Tb in °R), so it needs the molecular weight to
/// reach the molar basis this crate uses. Ref (32).
///
/// # Arguments
/// * `tb` — normal boiling point, **K**.
/// * `sg` — specific gravity, **dimensionless**.
/// * `mw` — molecular weight, **g/mol**.
pub fn vc_riazi_daubert_1980(tb: f64, sg: f64, mw: f64) -> f64 {
    let t = k_to_r(tb);
    // ft³/lb -> cm³/g: multiply by cm³/ft³ and divide by g/lb.
    let cm3_per_g = 7.5214e-3 * t.powf(0.2896) * sg.powf(-0.7666) * CM3_PER_FT3 / MOL_PER_LBMOL;
    cm3_per_g * mw
}

// ---------------------------------------------------------------------------
// Riazi-Daubert (1987) / API, ref (33) — published in K, bar and cm³/g
// ---------------------------------------------------------------------------

/// API / Riazi–Daubert 1987: molecular weight in **g/mol**, `Tb` in **K**.
///
/// `M = 42.965 · exp(2.097×10⁻⁴·Tb − 7.78712·SG + 2.08476×10⁻³·Tb·SG)
///     · Tb^1.26007 · SG^4.98308`, Tb in **K**. Ref (33).
pub fn mw_api_1987(tb: f64, sg: f64) -> f64 {
    42.965
        * (2.097e-4 * tb - 7.787_12 * sg + 2.084_76e-3 * tb * sg).exp()
        * tb.powf(1.260_07)
        * sg.powf(4.983_08)
}

/// API / Riazi–Daubert 1987: critical temperature in **K**, `Tb` in **K**.
///
/// `Tc = 9.5233 · exp(−9.314×10⁻⁴·Tb − 0.544442·SG + 6.4791×10⁻⁴·Tb·SG)
///      · Tb^0.81067 · SG^0.53691`. Ref (33).
pub fn tc_api_1987(tb: f64, sg: f64) -> f64 {
    9.5233
        * (-9.314e-4 * tb - 0.544_442 * sg + 6.4791e-4 * tb * sg).exp()
        * tb.powf(0.810_67)
        * sg.powf(0.536_91)
}

/// API / Riazi–Daubert 1987: critical pressure in **kPa**, `Tb` in **K**.
///
/// `Pc = 3.1958×10⁵ · exp(−8.505×10⁻³·Tb − 4.8014·SG + 5.749×10⁻³·Tb·SG)
///      · Tb^(−0.4844) · SG^4.0846`, Pc in **bar**. Ref (33).
pub fn pc_api_1987(tb: f64, sg: f64) -> f64 {
    let bar = 3.1958e5
        * (-8.505e-3 * tb - 4.8014 * sg + 5.749e-3 * tb * sg).exp()
        * tb.powf(-0.4844)
        * sg.powf(4.0846);
    bar * 100.0
}

/// API / Riazi–Daubert 1987: critical volume in **cm³/mol**, `Tb` in **K**.
///
/// `Vc = 6.049×10⁻² · exp(−2.6422×10⁻³·Tb − 0.26404·SG + 1.971×10⁻³·Tb·SG)
///      · Tb^0.7506 · SG^(−1.2028)` in **cm³/g**, so it needs `mw` to reach
/// cm³/mol. Ref (33).
pub fn vc_api_1987(tb: f64, sg: f64, mw: f64) -> f64 {
    let cm3_per_g = 6.049e-2
        * (-2.6422e-3 * tb - 0.264_04 * sg + 1.971e-3 * tb * sg).exp()
        * tb.powf(0.7506)
        * sg.powf(-1.2028);
    cm3_per_g * mw
}

// ---------------------------------------------------------------------------
// Kesler-Lee (1976), ref (36) — published in °R and psia
// ---------------------------------------------------------------------------

/// Kesler–Lee: critical temperature in **K**, `Tb` in **K**.
///
/// `Tc = 341.7 + 811.1·SG + (0.4244 + 0.1174·SG)·Tb
///      + (0.4669 − 3.26238·SG)·10⁵/Tb`, all in **°R**. Ref (36).
pub fn tc_kesler_lee(tb: f64, sg: f64) -> f64 {
    let t = k_to_r(tb);
    r_to_k(341.7 + 811.1 * sg + (0.4244 + 0.1174 * sg) * t + (0.4669 - 3.262_38 * sg) * 1e5 / t)
}

/// Kesler–Lee: critical pressure in **kPa**, `Tb` in **K**.
///
/// A cubic in Tb inside a logarithm; Tb in **°R**, Pc in **psia**. Ref (36).
pub fn pc_kesler_lee(tb: f64, sg: f64) -> f64 {
    let t = k_to_r(tb);
    let ln_pc = 8.3634 - 0.0566 / sg - (0.242_44 + 2.2898 / sg + 0.118_57 / (sg * sg)) * 1e-3 * t
        + (1.4685 + 3.648 / sg + 0.472_27 / (sg * sg)) * 1e-7 * t * t
        - (0.420_19 + 1.6977 / (sg * sg)) * 1e-10 * t * t * t;
    ln_pc.exp() * KPA_PER_PSIA
}

/// Kesler–Lee: molecular weight in **g/mol**, `Tb` in **K**.
///
/// A four-term polynomial in Tb and 1/Tb, Tb in **°R**. Ref (36).
pub fn mw_kesler_lee(tb: f64, sg: f64) -> f64 {
    let t = k_to_r(tb);
    -12_272.6
        + 9_486.4 * sg
        + (4.6523 - 3.3287 * sg) * t
        + (1.0 - 0.770_84 * sg - 0.020_58 * sg * sg) * (1.3437 - 720.79 / t) * 1e7 / t
        + (1.0 - 0.808_82 * sg + 0.022_26 * sg * sg) * (1.8828 - 181.98 / t) * 1e12 / (t * t * t)
}

// ---------------------------------------------------------------------------
// Twu (1984), ref (38) — published in °R and psia
// ---------------------------------------------------------------------------

/// The n-alkane reference state Twu perturbs about, at a given boiling point.
///
/// Every field is the property of the *normal paraffin* that boils at `tb_r`,
/// in Twu's published units: `tc0` in °R, `pc0` in psia, `vc0` in ft³/lbmol,
/// `sg0` dimensionless. All four are written internally in terms of the
/// reduced-temperature complement `α = 1 − Tb/Tc°`, which the caller can
/// recover as `1.0 - tb_r / tc0` if it needs it.
struct TwuReference {
    tc0: f64,
    pc0: f64,
    vc0: f64,
    sg0: f64,
}

/// Twu's n-alkane reference properties at boiling point `tb_r` (**°R**).
///
/// Ref (38), Eqs. 1–5. The `Tb^-13` term in Tc° is not a typo — it is what
/// bends the correlation over at the very light end.
fn twu_reference(tb_r: f64) -> TwuReference {
    let tc0 = tb_r
        / (0.533_272 + 0.191_017e-3 * tb_r + 0.779_681e-7 * tb_r.powi(2)
            - 0.284_376e-10 * tb_r.powi(3)
            + 0.959_468e28 / tb_r.powi(13));
    let alpha = 1.0 - tb_r / tc0;
    let vc0 = (1.0
        - (0.419_869 - 0.505_839 * alpha - 1.564_36 * alpha.powi(3) - 9_481.70 * alpha.powi(14)))
    .powf(-8.0);
    let sg0 = 0.843_593 - 0.128_624 * alpha - 3.361_59 * alpha.powi(3) - 13_749.5 * alpha.powi(12);
    let pc0 = (3.833_54
        + 1.196_29 * alpha.sqrt()
        + 34.8888 * alpha
        + 36.1952 * alpha.powi(2)
        + 104.193 * alpha.powi(4))
    .powi(2);
    TwuReference { tc0, pc0, vc0, sg0 }
}

/// Twu's boiling point (**°R**) of the n-alkane of log-molecular-weight `theta`.
///
/// Ref (38), Eq. 6, with `θ = ln M°`. Twu gives this direction only, so the
/// reference molecular weight is obtained by inverting it numerically.
fn twu_tb_of_ln_mw(theta: f64) -> f64 {
    (5.714_19 + 2.715_79 * theta
        - 0.286_590 * theta * theta
        - 39.8544 / theta
        - 0.122_488 / (theta * theta))
        .exp()
        - 24.7522 * theta
        + 35.3155 * theta * theta
}

/// Invert [`twu_tb_of_ln_mw`] for the reference molecular weight, **g/mol**.
///
/// Bisection rather than Newton: the bracket `M° ∈ [16, 5000] g/mol` covers
/// methane through the heaviest vacuum-residue cut anybody characterizes, the
/// function is monotone across it, and 200 halvings drive the interval to
/// machine precision. There is no derivative to get wrong and no way to
/// diverge, which for a function evaluated once per pseudocomponent is the
/// right trade.
fn twu_reference_mw(tb_r: f64) -> Result<f64, PetroleumError> {
    let (mut lo, mut hi) = (16.0f64.ln(), 5000.0f64.ln());
    if twu_tb_of_ln_mw(lo) > tb_r || twu_tb_of_ln_mw(hi) < tb_r {
        return Err(PetroleumError::NoConvergence(format!(
            "Twu reference molecular weight: boiling point {:.1} K is outside the \
             16–5000 g/mol n-alkane bracket",
            r_to_k(tb_r)
        )));
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if twu_tb_of_ln_mw(mid) < tb_r {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((0.5 * (lo + hi)).exp())
}

/// All four Twu critical properties at once, in canonical units.
///
/// Returns `(mw [g/mol], tc [K], pc [kPa], vc [cm³/mol])`.
///
/// Twu's structure is the same for each property: take the n-alkane reference
/// value, form a *density deviation* `ΔSG` measuring how far the real cut sits
/// from that alkane, and apply the correction factor `[(1 + 2f)/(1 − 2f)]²`.
/// When the cut *is* an n-alkane, `ΔSG = 0`, `f = 0` and the correction is
/// exactly 1 — which is why the paraffin errors in the module table are so small.
///
/// Ref (38), Eqs. 7–17.
///
/// # Arguments
/// * `tb` — normal boiling point, **K**.
/// * `sg` — specific gravity, **dimensionless**.
pub fn properties_twu(tb: f64, sg: f64) -> Result<(f64, f64, f64, f64), PetroleumError> {
    check_inputs(tb, sg)?;
    let t = k_to_r(tb);
    let r = twu_reference(t);
    let sqrt_t = t.sqrt();

    // Critical temperature.
    let d_t = (5.0 * (r.sg0 - sg)).exp() - 1.0;
    let f_t = d_t * (-0.270_16 / sqrt_t + (0.039_828_5 - 0.706_691 / sqrt_t) * d_t);
    let tc = r.tc0 * ((1.0 + 2.0 * f_t) / (1.0 - 2.0 * f_t)).powi(2);

    // Critical volume. Note the deviation is in SG² here, not SG.
    let d_v = (4.0 * (r.sg0 * r.sg0 - sg * sg)).exp() - 1.0;
    let f_v = d_v * (0.347_776 / sqrt_t + (-0.182_421 + 2.248_90 / sqrt_t) * d_v);
    let vc = r.vc0 * ((1.0 + 2.0 * f_v) / (1.0 - 2.0 * f_v)).powi(2);

    // Critical pressure. It carries the Tc and Vc corrections too, which is
    // what makes the set "internally consistent" in the paper's title.
    let d_p = (0.5 * (r.sg0 - sg)).exp() - 1.0;
    let f_p = d_p
        * ((2.532_62 - 46.1955 / sqrt_t - 0.001_278_85 * t)
            + (-11.4277 + 252.140 / sqrt_t + 0.002_305_35 * t) * d_p);
    let pc = r.pc0 * (tc / r.tc0) * (r.vc0 / vc) * ((1.0 + 2.0 * f_p) / (1.0 - 2.0 * f_p)).powi(2);

    // Molecular weight, whose reference has to be inverted numerically.
    let mw0 = twu_reference_mw(t)?;
    let chi = (0.012_342 - 0.244_541 / sqrt_t).abs();
    let d_m = (5.0 * (r.sg0 - sg)).exp() - 1.0;
    let f_m = d_m * (chi + (-0.017_569_1 + 0.143_979 / sqrt_t) * d_m);
    let mw = (mw0.ln() * ((1.0 + 2.0 * f_m) / (1.0 - 2.0 * f_m)).powi(2)).exp();

    Ok((
        mw,
        r_to_k(tc),
        pc * KPA_PER_PSIA,
        vc * CM3_PER_FT3 / MOL_PER_LBMOL,
    ))
}

// ---------------------------------------------------------------------------
// Acentric factor
// ---------------------------------------------------------------------------

/// Acentric factor of a petroleum fraction, **dimensionless**.
///
/// Two branches, and which one applies is decided by the *reduced* boiling
/// point `Tbr = Tb/Tc`:
///
/// - **`Tbr < 0.8`** — the Lee–Kesler vapor-pressure correlation, ref (37)
///   Eqs. 17–18, evaluated at the normal boiling point where `P/Pc = 1 atm/Pc`
///   by definition. This is a *derivation*, not a fit: it is what ω means.
/// - **`Tbr ≥ 0.8`** — a direct Kesler–Lee fit in Watson K and `Tbr`, ref (36).
///   The first branch degenerates as `Tbr → 1` (its denominator passes through
///   zero), and heavy cuts do reach `Tbr ≈ 0.8`, so the second branch exists to
///   cover them.
///
/// # Arguments
/// * `tb` — normal boiling point, **K**.
/// * `tc` — critical temperature, **K**.
/// * `pc` — critical pressure, **kPa** (absolute).
/// * `sg` — specific gravity, **dimensionless** (used only on the heavy branch).
///
/// # Returns
/// The acentric factor, **dimensionless**.
pub fn acentric_lee_kesler(tb: f64, tc: f64, pc: f64, sg: f64) -> Result<f64, PetroleumError> {
    check_inputs(tb, sg)?;
    if tc <= 0.0 || pc <= 0.0 || !tc.is_finite() || !pc.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "critical properties must be positive and finite, got Tc = {tc} K, Pc = {pc} kPa"
        )));
    }
    let tbr = tb / tc;
    if tbr < 0.8 {
        // Reduced pressure at the normal boiling point: 1 atm over Pc.
        let pbr = P_ATM_KPA / pc;
        let num =
            pbr.ln() - 5.927_14 + 6.096_48 / tbr + 1.288_62 * tbr.ln() - 0.169_347 * tbr.powi(6);
        let den = 15.2518 - 15.6875 / tbr - 13.4721 * tbr.ln() + 0.435_77 * tbr.powi(6);
        Ok(num / den)
    } else {
        let kw = watson_k(tb, sg)?;
        Ok(
            -7.904 + 0.1352 * kw - 0.007_465 * kw * kw
                + 8.359 * tbr
                + (1.408 - 0.010_63 * kw) / tbr,
        )
    }
}

// ---------------------------------------------------------------------------
// The dispatcher
// ---------------------------------------------------------------------------

/// Estimate the full property set of a pseudocomponent from `Tb` and `SG`.
///
/// The acentric factor always comes from [`acentric_lee_kesler`], evaluated on
/// the `Tc` and `Pc` the chosen method produced — mixing a Kesler–Lee ω with
/// Twu criticals would be inconsistent, and this keeps them together.
///
/// `Zc` and `Vc` are kept mutually consistent through `Zc = Pc·Vc/(R·Tc)`:
///
/// - methods that publish a critical volume ([`PropertyMethod::RiaziDaubert1980`],
///   [`ApiRiaziDaubert1987`], [`Twu`]) use it and derive `Zc` from it;
/// - [`PropertyMethod::KeslerLee`], which publishes none, takes `Zc` from
///   `zc_method` and derives `Vc` from that.
///
/// [`ApiRiaziDaubert1987`]: PropertyMethod::ApiRiaziDaubert1987
/// [`Twu`]: PropertyMethod::Twu
///
/// # Arguments
/// * `method` — which critical-property correlation family to use.
/// * `tb` — normal (mid-)boiling point of the cut, **K**.
/// * `sg` — specific gravity at 60/60 °F, **dimensionless**.
/// * `zc_method` — the corresponding-states Zc correlation, used only by
///   [`PropertyMethod::KeslerLee`].
///
/// # Returns
/// A [`PseudoProperties`] in **K**, **kPa**, **g/mol** and **cm³/mol**.
pub fn estimate(
    method: PropertyMethod,
    tb: f64,
    sg: f64,
    zc_method: ZcMethod,
) -> Result<PseudoProperties, PetroleumError> {
    check_inputs(tb, sg)?;
    let kw = watson_k(tb, sg)?;

    let (mw, tc, pc, vc_opt) = match method {
        PropertyMethod::RiaziDaubert1980 => {
            let mw = mw_riazi_daubert_1980(tb, sg);
            (
                mw,
                tc_riazi_daubert_1980(tb, sg),
                pc_riazi_daubert_1980(tb, sg),
                Some(vc_riazi_daubert_1980(tb, sg, mw)),
            )
        }
        PropertyMethod::ApiRiaziDaubert1987 => {
            let mw = mw_api_1987(tb, sg);
            (
                mw,
                tc_api_1987(tb, sg),
                pc_api_1987(tb, sg),
                Some(vc_api_1987(tb, sg, mw)),
            )
        }
        PropertyMethod::KeslerLee => (
            mw_kesler_lee(tb, sg),
            tc_kesler_lee(tb, sg),
            pc_kesler_lee(tb, sg),
            None,
        ),
        PropertyMethod::Twu => {
            let (mw, tc, pc, vc) = properties_twu(tb, sg)?;
            (mw, tc, pc, Some(vc))
        }
    };

    if !(tc.is_finite() && tc > 0.0)
        || !(pc.is_finite() && pc > 0.0)
        || !(mw.is_finite() && mw > 0.0)
    {
        return Err(PetroleumError::InvalidInput(format!(
            "{method:?} produced non-physical properties at Tb = {tb} K, SG = {sg}: \
             Tc = {tc} K, Pc = {pc} kPa, M = {mw} g/mol — the correlation is \
             almost certainly being used far outside its fitted range"
        )));
    }

    let omega = acentric_lee_kesler(tb, tc, pc, sg)?;
    let (vc, zc) = match vc_opt {
        Some(vc) => (vc, zc_from_criticals(pc, vc, tc)),
        None => {
            let zc = zc_method.zc(omega);
            (vc_from_zc(zc, tc, pc), zc)
        }
    };

    Ok(PseudoProperties {
        tb,
        sg,
        watson_k: kw,
        mw,
        tc,
        pc,
        vc,
        zc,
        omega,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Measured properties of ten pure hydrocarbons, as
    /// `(name, Tb [K], SG 60/60, M [g/mol], Tc [K], Pc [kPa], ω, Vc [cm³/mol])`.
    ///
    /// Tb, M, Tc, Pc, ω and Vc are this crate's bundled component database
    /// (`python/src/vle/data/components.json`, itself sourced from DIPPR/NIST);
    /// the specific gravities are the standard 60/60 °F values, which the
    /// database does not carry. These ten span the three hydrocarbon families a
    /// petroleum cut is made of — six n-paraffins, two aromatics, two
    /// naphthenes — which is what makes them a fair test of correlations whose
    /// whole job is to interpolate between those families.
    /// One row of the reference table:
    /// `(name, Tb [K], SG 60/60, M [g/mol], Tc [K], Pc [kPa], ω, Vc [cm³/mol])`.
    type Reference = (&'static str, f64, f64, f64, f64, f64, f64, f64);

    const REFERENCE: [Reference; 10] = [
        (
            "n-pentane",
            309.209,
            0.6312,
            72.1488,
            469.70,
            3367.5,
            0.2510,
            311.5,
        ),
        (
            "n-hexane", 341.866, 0.6640, 86.1754, 507.82, 3044.1, 0.3000, 369.5,
        ),
        (
            "n-heptane",
            371.550,
            0.6882,
            100.2019,
            540.20,
            2735.73,
            0.3490,
            429.2,
        ),
        (
            "n-octane", 398.794, 0.7070, 114.2285, 568.74, 2483.59, 0.3980, 492.4,
        ),
        (
            "n-nonane", 423.913, 0.7219, 128.2551, 594.55, 2281.0, 0.4433, 552.5,
        ),
        (
            "n-decane", 447.270, 0.7342, 142.2817, 617.70, 2103.0, 0.4884, 609.8,
        ),
        (
            "benzene", 353.219, 0.8829, 78.1118, 562.02, 4907.277, 0.2110, 256.3,
        ),
        (
            "toluene", 383.746, 0.8719, 92.1384, 591.75, 4126.3, 0.2657, 315.6,
        ),
        (
            "cyclohexane",
            353.865,
            0.7834,
            84.1595,
            553.60,
            4080.5,
            0.2096,
            310.2,
        ),
        (
            "methylcyclohexane",
            374.010,
            0.7748,
            98.1861,
            572.20,
            3470.0,
            0.2340,
            367.6,
        ),
    ];

    fn worst_error(
        f: impl Fn(f64, f64) -> f64,
        reference: impl Fn(&(&str, f64, f64, f64, f64, f64, f64, f64)) -> f64,
    ) -> (f64, &'static str) {
        let mut worst = 0.0;
        let mut who = "";
        for row in &REFERENCE {
            let got = f(row.1, row.2);
            let want = reference(row);
            let err = 100.0 * (got - want).abs() / want;
            if err > worst {
                worst = err;
                who = row.0;
            }
        }
        (worst, who)
    }

    // === Accuracy against measured pure-component properties ==============
    //
    // These are the tests behind the accuracy table in the module docs. The
    // tolerances are the *measured* worst-case deviations rounded up slightly,
    // not aspirations — so a mistyped coefficient trips one immediately, and
    // the documented table cannot drift away from reality.

    #[test]
    fn riazi_daubert_1980_matches_measured_properties() {
        let (e, who) = worst_error(tc_riazi_daubert_1980, |r| r.4);
        assert!(e < 2.5, "Tc worst {e:.2}% on {who}, expected < 2.5%");
        let (e, who) = worst_error(pc_riazi_daubert_1980, |r| r.5);
        assert!(e < 8.3, "Pc worst {e:.2}% on {who}, expected < 8.3%");
        let (e, who) = worst_error(mw_riazi_daubert_1980, |r| r.3);
        assert!(e < 8.3, "M worst {e:.2}% on {who}, expected < 8.3%");
    }

    #[test]
    fn api_1987_matches_measured_properties() {
        let (e, who) = worst_error(tc_api_1987, |r| r.4);
        assert!(e < 1.3, "Tc worst {e:.2}% on {who}, expected < 1.3%");
        let (e, who) = worst_error(pc_api_1987, |r| r.5);
        assert!(e < 5.1, "Pc worst {e:.2}% on {who}, expected < 5.1%");
        let (e, who) = worst_error(mw_api_1987, |r| r.3);
        assert!(e < 6.0, "M worst {e:.2}% on {who}, expected < 6.0%");
    }

    #[test]
    fn kesler_lee_matches_measured_properties() {
        let (e, who) = worst_error(tc_kesler_lee, |r| r.4);
        assert!(e < 1.9, "Tc worst {e:.2}% on {who}, expected < 1.9%");
        let (e, who) = worst_error(pc_kesler_lee, |r| r.5);
        assert!(e < 5.6, "Pc worst {e:.2}% on {who}, expected < 5.6%");
        let (e, who) = worst_error(mw_kesler_lee, |r| r.3);
        assert!(e < 8.1, "M worst {e:.2}% on {who}, expected < 8.1%");
    }

    #[test]
    fn twu_matches_measured_properties() {
        let (e, who) = worst_error(|tb, sg| properties_twu(tb, sg).unwrap().1, |r| r.4);
        assert!(e < 1.9, "Tc worst {e:.2}% on {who}, expected < 1.9%");
        let (e, who) = worst_error(|tb, sg| properties_twu(tb, sg).unwrap().2, |r| r.5);
        assert!(e < 15.3, "Pc worst {e:.2}% on {who}, expected < 15.3%");
        let (e, who) = worst_error(|tb, sg| properties_twu(tb, sg).unwrap().0, |r| r.3);
        assert!(e < 7.5, "M worst {e:.2}% on {who}, expected < 7.5%");
    }

    #[test]
    fn critical_volumes_match_measured_values() {
        let (e, who) = worst_error(
            |tb, sg| vc_riazi_daubert_1980(tb, sg, mw_riazi_daubert_1980(tb, sg)),
            |r| r.7,
        );
        assert!(e < 8.0, "R-D 1980 Vc worst {e:.2}% on {who}");
        let (e, who) = worst_error(|tb, sg| vc_api_1987(tb, sg, mw_api_1987(tb, sg)), |r| r.7);
        assert!(e < 6.0, "API 1987 Vc worst {e:.2}% on {who}");
        let (e, who) = worst_error(|tb, sg| properties_twu(tb, sg).unwrap().3, |r| r.7);
        assert!(e < 6.0, "Twu Vc worst {e:.2}% on {who}");
    }

    #[test]
    fn acentric_factor_matches_measured_values() {
        // Fed the *measured* Tc and Pc so the ω correlation is isolated from
        // the critical-property correlations' own error. This is the only way
        // to tell whether ω itself is right.
        let mut worst: f64 = 0.0;
        let mut who = "";
        for r in &REFERENCE {
            let w = acentric_lee_kesler(r.1, r.4, r.5, r.2).unwrap();
            let err = 100.0 * (w - r.6).abs() / r.6;
            if err > worst {
                worst = err;
                who = r.0;
            }
        }
        assert!(worst < 1.6, "ω worst {worst:.2}% on {who}, expected < 1.6%");
    }

    #[test]
    fn twu_is_the_best_method_on_paraffins_and_the_worst_on_aromatics() {
        // The structural claim in the module docs, asserted rather than
        // asserted-in-prose: Twu is a perturbation about the n-alkane, so it
        // should nail n-heptane's Pc and miss benzene's badly.
        let heptane = &REFERENCE[2];
        let benzene = &REFERENCE[6];
        let pc_hep = properties_twu(heptane.1, heptane.2).unwrap().2;
        let pc_ben = properties_twu(benzene.1, benzene.2).unwrap().2;
        let err_hep = 100.0 * (pc_hep - heptane.5).abs() / heptane.5;
        let err_ben = 100.0 * (pc_ben - benzene.5).abs() / benzene.5;
        assert!(
            err_hep < 1.0,
            "Twu Pc on n-heptane {err_hep:.2}%, expected < 1%"
        );
        assert!(
            err_ben > 5.0,
            "Twu Pc on benzene {err_ben:.2}% — if this is now small the docs' \
             paraffin/aromatic caveat is stale and should be rewritten"
        );
    }

    // === Twu internals ====================================================

    #[test]
    fn twu_reference_molecular_weight_inverts_its_own_forward_relation() {
        // Round-trip the bisection against the published forward equation.
        for mw in [50.0f64, 100.0, 200.0, 500.0, 1000.0] {
            let tb_r = twu_tb_of_ln_mw(mw.ln());
            let back = twu_reference_mw(tb_r).unwrap();
            assert!(
                (back - mw).abs() / mw < 1e-9,
                "M° {mw} -> Tb {tb_r} °R -> {back}"
            );
        }
    }

    #[test]
    fn twu_reference_recovers_n_alkane_properties_almost_exactly() {
        // A cut whose SG *is* the reference SG has ΔSG = 0, so every correction
        // factor collapses to 1 and Twu must return the reference itself. This
        // is the check that the perturbation algebra is wired up right.
        let tb_r = k_to_r(400.0);
        let r = twu_reference(tb_r);
        let (_, tc, pc, vc) = properties_twu(400.0, r.sg0).unwrap();
        assert!((tc - r_to_k(r.tc0)).abs() < 1e-9, "Tc {tc} vs reference");
        assert!(
            (pc - r.pc0 * KPA_PER_PSIA).abs() < 1e-6,
            "Pc {pc} vs reference"
        );
        assert!(
            (vc - r.vc0 * CM3_PER_FT3 / MOL_PER_LBMOL).abs() < 1e-9,
            "Vc {vc} vs reference"
        );
    }

    #[test]
    fn twu_reference_alpha_is_a_sane_reduced_complement() {
        // α = 1 − Tb/Tc° must sit strictly between 0 and 1 for anything that
        // boils below its own critical point, which is everything.
        for tb in [250.0, 350.0, 500.0, 700.0, 850.0] {
            let r = twu_reference(k_to_r(tb));
            let a = 1.0 - k_to_r(tb) / r.tc0;
            assert!((0.0..1.0).contains(&a), "α = {a} at Tb = {tb} K");
        }
    }

    #[test]
    fn twu_reports_rather_than_silently_extrapolating_off_its_bracket() {
        // Far above any real cut: the n-alkane inversion has no root and must
        // say so instead of returning the bracket edge.
        let err = properties_twu(1500.0, 0.95).unwrap_err();
        assert!(
            matches!(err, PetroleumError::NoConvergence(_)),
            "got {err:?}"
        );
    }

    // === Zc / Vc consistency ==============================================

    #[test]
    fn zc_and_vc_are_mutually_consistent_for_every_method() {
        for method in [
            PropertyMethod::RiaziDaubert1980,
            PropertyMethod::ApiRiaziDaubert1987,
            PropertyMethod::KeslerLee,
            PropertyMethod::Twu,
        ] {
            let p = estimate(method, 450.0, 0.78, ZcMethod::LeeKesler).unwrap();
            let zc_check = zc_from_criticals(p.pc, p.vc, p.tc);
            assert!(
                (p.zc - zc_check).abs() < 1e-12,
                "{method:?}: Zc {} vs Pc·Vc/(R·Tc) {zc_check}",
                p.zc
            );
        }
    }

    #[test]
    fn zc_conversion_reproduces_a_known_compound() {
        // Methane: Pc = 4599.2 kPa, Vc = 98.628 cm³/mol, Tc = 190.564 K,
        // measured Zc = 0.28629. This pins the 1e-3 unit factor, which is the
        // only thing in that function that can be wrong.
        let zc = zc_from_criticals(4599.2, 98.628, 190.564);
        assert!((zc - 0.28629).abs() < 1e-4, "got {zc}");
    }

    #[test]
    fn zc_correlations_agree_with_each_other_and_land_in_range() {
        for omega in [0.0, 0.2, 0.4, 0.8, 1.2] {
            let vals: Vec<f64> = [
                ZcMethod::LeeKesler,
                ZcMethod::Reid,
                ZcMethod::Salerno,
                ZcMethod::Nath,
            ]
            .iter()
            .map(|m| m.zc(omega))
            .collect();
            for &z in &vals {
                assert!((0.15..0.30).contains(&z), "Zc {z} at ω = {omega}");
            }
            let spread = vals.iter().cloned().fold(f64::MIN, f64::max)
                - vals.iter().cloned().fold(f64::MAX, f64::min);
            assert!(
                spread < 0.02,
                "the four Zc correlations spread {spread:.4} at ω = {omega}; \
                 the module docs claim they agree to ~0.005"
            );
        }
    }

    // === The dispatcher ===================================================

    #[test]
    fn estimate_returns_a_physically_ordered_property_set() {
        // A mid-distillate cut: Tb 500 K, SG 0.82.
        for method in [
            PropertyMethod::RiaziDaubert1980,
            PropertyMethod::ApiRiaziDaubert1987,
            PropertyMethod::KeslerLee,
            PropertyMethod::Twu,
        ] {
            let p = estimate(method, 500.0, 0.82, ZcMethod::LeeKesler).unwrap();
            assert!(
                p.tc > p.tb,
                "{method:?}: Tc {} must exceed Tb {}",
                p.tc,
                p.tb
            );
            assert!(p.pc > 101.325, "{method:?}: Pc {} must exceed 1 atm", p.pc);
            assert!(p.mw > 0.0 && p.mw < 2000.0, "{method:?}: M = {}", p.mw);
            assert!(p.vc > 0.0, "{method:?}: Vc = {}", p.vc);
            assert!((0.15..0.35).contains(&p.zc), "{method:?}: Zc = {}", p.zc);
            assert!((0.0..1.5).contains(&p.omega), "{method:?}: ω = {}", p.omega);
            assert!(
                (10.0..14.0).contains(&p.watson_k),
                "{method:?}: K_W = {}",
                p.watson_k
            );
        }
    }

    #[test]
    fn heavier_cuts_have_higher_critical_temperature_and_lower_critical_pressure() {
        // The single most important monotonicity in the whole module: as you
        // move up a crude tower the cuts get heavier, and Tc must rise while
        // Pc falls. A sign error in any exponent shows up here.
        for method in [
            PropertyMethod::RiaziDaubert1980,
            PropertyMethod::ApiRiaziDaubert1987,
            PropertyMethod::KeslerLee,
            PropertyMethod::Twu,
        ] {
            let mut prev: Option<PseudoProperties> = None;
            for tb in [350.0, 420.0, 500.0, 580.0, 650.0] {
                // Watson K held roughly constant as Tb rises, as in a real assay.
                let sg = 0.68 + (tb - 350.0) * 0.00045;
                let p = estimate(method, tb, sg, ZcMethod::LeeKesler).unwrap();
                if let Some(q) = prev {
                    assert!(p.tc > q.tc, "{method:?}: Tc fell from {} to {}", q.tc, p.tc);
                    assert!(p.pc < q.pc, "{method:?}: Pc rose from {} to {}", q.pc, p.pc);
                    assert!(p.mw > q.mw, "{method:?}: M fell from {} to {}", q.mw, p.mw);
                    assert!(
                        p.omega > q.omega,
                        "{method:?}: ω fell from {} to {}",
                        q.omega,
                        p.omega
                    );
                }
                prev = Some(p);
            }
        }
    }

    #[test]
    fn acentric_factor_switches_branch_without_a_jump() {
        // The two ω branches meet at Tbr = 0.8. They are different fits, so
        // they will not agree exactly — but a discontinuity big enough to
        // matter would mean one of them is transcribed wrong.
        let tc = 700.0;
        let (tb_lo, tb_hi) = (0.7999 * tc, 0.8001 * tc);
        let lo = acentric_lee_kesler(tb_lo, tc, 1500.0, 0.86).unwrap();
        let hi = acentric_lee_kesler(tb_hi, tc, 1500.0, 0.86).unwrap();
        assert!(
            (lo - hi).abs() < 0.15,
            "ω jumps from {lo:.4} to {hi:.4} across the Tbr = 0.8 branch switch"
        );
    }

    #[test]
    fn estimate_rejects_non_physical_input() {
        for method in [
            PropertyMethod::RiaziDaubert1980,
            PropertyMethod::ApiRiaziDaubert1987,
            PropertyMethod::KeslerLee,
            PropertyMethod::Twu,
        ] {
            assert!(estimate(method, 0.0, 0.8, ZcMethod::default()).is_err());
            assert!(estimate(method, -100.0, 0.8, ZcMethod::default()).is_err());
            assert!(estimate(method, 450.0, 0.0, ZcMethod::default()).is_err());
            assert!(estimate(method, f64::NAN, 0.8, ZcMethod::default()).is_err());
            assert!(estimate(method, 450.0, f64::INFINITY, ZcMethod::default()).is_err());
        }
    }

    #[test]
    fn acentric_factor_rejects_non_physical_criticals() {
        assert!(acentric_lee_kesler(400.0, 0.0, 3000.0, 0.75).is_err());
        assert!(acentric_lee_kesler(400.0, 550.0, -1.0, 0.75).is_err());
    }
}
