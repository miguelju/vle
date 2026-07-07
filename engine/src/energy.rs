//! Mixture energy properties — enthalpy and entropy (Milestone 8.4).
//!
//! Total molar H and S of a phase are the sum of three contributions
//! (research paper Eqs 2.35–2.36):
//!
//! ```text
//!   H(T,P,x) = Σ xᵢ·[Hᵢ° + ∫_{Tref}^T Cpᵢ° dT]           + H^R
//!   S(T,P,x) = Σ xᵢ·[Sᵢ° + ∫_{Tref}^T (Cpᵢ°/T) dT
//!                     − R·ln(P/Pref)]  − R·Σ xᵢ ln xᵢ    + S^R
//! ```
//!
//! - **Ideal-gas term** — the ideal heat-capacity integral from a reference
//!   state, plus the ideal mixing entropy `−R Σ xᵢ ln xᵢ`.
//! - **Residual (departure) term** `H^R`, `S^R` — the correction for real-
//!   fluid non-ideality, from the EOS. For the φ-φ path this comes from the
//!   generalized cubic; for the γ-φ (activity) path the liquid uses the
//!   excess-property route (condensation + Hᴱ/Sᴱ).
//!
//! ## Analytic temperature derivative (CLAUDE.md Algorithm Choices)
//!
//! The residual enthalpy needs `T·dA_mix/dT`. It is computed **analytically**
//! for every mixing rule — never by finite differences (those survive only
//! as the test oracle). The building blocks are all exact:
//!
//! - `dAᵢ/dT = Aᵢ·(α'ᵢ/αᵢ − 2/T)`, with `α'ᵢ = (dαᵢ/dTr)/Tcᵢ` (the analytic
//!   dα/dTr the EOS layer already provides for all 22 variants).
//! - `dBᵢ/dT = −Bᵢ/T` (Bᵢ ∝ 1/T exactly).
//! - For GE-based rules, `T·d(Gᴱ/RT)/dT = −Hᴱ/(RT)` — a clean consequence of
//!   `Hᴱ = Gᴱ − T·dGᴱ/dT`, and Hᴱ is the analytic excess enthalpy from the
//!   activity layer. So even Wong-Sandler / MHV get an exact departure
//!   derivative with no numerical stencil (the legacy Pascal used a 5-point
//!   stencil here — this is the modernization the plan calls for).
//!
//! ## Legacy fidelity
//!
//! - Ideal Cp integration matches Pascal `EntalpiaIdeal`/`EntropiaIdeal`
//!   (TERMOII.PAS:531-577); the engine's `cp_coeffs` are `Cpᵢ°/R` (see
//!   [`crate::types::Component::cp_coeffs`]).
//! - The residual form is Müller et al. (9), research paper Eqs 2.28–2.29,
//!   generalized to the (A, B, U, W) mixture core.
//! - Schmidt-Wenzel residual energy is finite here (the engine's guarded
//!   analytic dα/dTr — see `eos::d_alpha_d_tr`), where the legacy Pascal
//!   returned NaN (TERMOII.PAS:442-444). This is a documented improvement.
//!
//! # References
//! - (9) Müller et al. — departure functions
//! - (4) Da Silva & Báez (1989), legacy/pascal/TERMOII.PAS, TERMOIII.PAS

// Parallel-array numerical code (Aᵢ, Bᵢ, xᵢ indexed in lockstep across the
// ΣΣ mixing sums) — index loops mirror the math; allow the range-loop lint.
#![allow(clippy::needless_range_loop)]

use crate::activity::{ActivityModel, excess_enthalpy, excess_entropy};
use crate::eos::{EosState, PhaseId};
use crate::mixing::MixingRule;
use crate::mixture::{MixError, MixtureSpec, ln_phi_mix, z_mix};
use crate::types::{Component, R_GAS};

/// Ideal-gas heat capacity Cpᵢ°(T) for one component.
///
/// `Cpᵢ°(T) = R·(a₀ + a₁T + a₂T² + a₃T³ + a₄T⁴)` — the engine stores
/// `cp_coeffs` as `Cpᵢ°/R` (see [`Component::cp_coeffs`]).
///
/// # Arguments
/// * `t` — Temperature in **K**.
///
/// # Returns
/// Cpᵢ° in **kJ/(kmol·K)**.
pub fn ideal_cp(comp: &Component, t: f64) -> f64 {
    let c = &comp.cp_coeffs;
    R_GAS * (c[0] + t * (c[1] + t * (c[2] + t * (c[3] + t * c[4]))))
}

/// Ideal-gas enthalpy integral `∫_{Tref}^T Cpᵢ° dT` for one component.
///
/// Closed-form term-by-term integral of the polynomial (Pascal
/// `EntalpiaIdeal`, TERMOII.PAS:531-550):
/// `R·Σₖ aₖ·(T^{k+1} − Tref^{k+1})/(k+1)`.
///
/// # Returns
/// Enthalpy contribution in **kJ/kmol**.
pub fn ideal_enthalpy_integral(comp: &Component, t: f64, t_ref: f64) -> f64 {
    let c = &comp.cp_coeffs;
    let mut acc = 0.0;
    for (k, &ak) in c.iter().enumerate() {
        let p = (k + 1) as i32;
        acc += ak * (t.powi(p) - t_ref.powi(p)) / p as f64;
    }
    R_GAS * acc
}

/// Ideal-gas entropy integral `∫_{Tref}^T (Cpᵢ°/T) dT` for one component.
///
/// `R·[a₀·ln(T/Tref) + Σ_{k≥1} aₖ·(Tᵏ − Trefᵏ)/k]` (Pascal `EntropiaIdeal`,
/// TERMOII.PAS:552-577). The `a₀` term integrates to a logarithm; the
/// higher terms to the power series.
///
/// # Returns
/// Entropy contribution in **kJ/(kmol·K)**.
pub fn ideal_entropy_integral(comp: &Component, t: f64, t_ref: f64) -> f64 {
    let c = &comp.cp_coeffs;
    let mut acc = c[0] * (t / t_ref).ln();
    for (k, &ak) in c.iter().enumerate().skip(1) {
        let p = k as i32;
        acc += ak * (t.powi(p) - t_ref.powi(p)) / p as f64;
    }
    R_GAS * acc
}

/// Ideal mixing entropy `−R·Σᵢ xᵢ·ln xᵢ`, in **kJ/(kmol·K)**.
///
/// Guards `xᵢ → 0` (the `xᵢ·ln xᵢ` term → 0) with a `1e-300` floor, matching
/// the legacy `Abs(x) > 1e-60` guard (TERMOII.PAS:574).
pub fn ideal_mixing_entropy(x: &[f64]) -> f64 {
    let s: f64 = x
        .iter()
        .filter(|&&xi| xi > 1e-300)
        .map(|&xi| xi * xi.ln())
        .sum();
    -R_GAS * s
}

/// Total ideal-gas molar enthalpy of a mixture relative to the reference
/// state, in **kJ/kmol**.
///
/// `Σᵢ xᵢ·[Hᵢ° + ∫_{Tref}^T Cpᵢ° dT]`. `h_ref` is the per-component
/// reference enthalpy Hᵢ° (pass `&[]` for all-zero, the usual convention).
pub fn ideal_enthalpy_mix(
    comps: &[Component],
    x: &[f64],
    t: f64,
    t_ref: f64,
    h_ref: &[f64],
) -> f64 {
    (0..comps.len())
        .map(|i| {
            let h0 = h_ref.get(i).copied().unwrap_or(0.0);
            x[i] * (h0 + ideal_enthalpy_integral(&comps[i], t, t_ref))
        })
        .sum()
}

/// Total ideal-gas molar entropy of a mixture relative to the reference
/// state, in **kJ/(kmol·K)**.
///
/// `Σᵢ xᵢ·[Sᵢ° + ∫(Cpᵢ°/T)dT − R·ln(P/Pref)] − R·Σ xᵢ ln xᵢ`.
pub fn ideal_entropy_mix(
    comps: &[Component],
    x: &[f64],
    t: f64,
    p: f64,
    t_ref: f64,
    p_ref: f64,
    s_ref: &[f64],
) -> f64 {
    let pressure_term = R_GAS * (p / p_ref).ln();
    let sum: f64 = (0..comps.len())
        .map(|i| {
            let s0 = s_ref.get(i).copied().unwrap_or(0.0);
            x[i] * (s0 + ideal_entropy_integral(&comps[i], t, t_ref) - pressure_term)
        })
        .sum();
    sum + ideal_mixing_entropy(x)
}

/// Dimensionless residual (departure) enthalpy `H^R/(R·T)` of a mixture
/// from the EOS, for the requested phase.
///
/// Generalized-cubic form (Müller (9), reducing to the pure expression at
/// x = [1]):
/// ```text
///   H^R/(RT) = (Z − 1) + A_mix·Ĩ·(τ − 1),   τ = T·(dA_mix/dT)/A_mix
/// ```
/// with `A_mix·Ĩ` the attractive term and `τ` the analytic temperature
/// derivative built by [`t_dln_a_dt_mix`]. Written via the total residual
/// Gibbs energy `G^R/(RT) = Σ xᵢ ln φ̂ᵢ` so the entropy follows from
/// Lewis-Randall without a separate B-derivative.
///
/// # Errors
/// Propagates [`MixError`] from the mixture layer (bad combination,
/// no root for phase, …).
pub fn h_departure_rt_mix(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<f64, MixError> {
    let (a_mix, itilde, z) = mix_attractive_pieces(spec, t, p, x, phase)?;
    // τ − 1 where τ = T·(dA_mix/dT)/A_mix + 2 (the +2 is a_mix ∝ A_mix·T²);
    // equivalently H = (Z−1) + A·Ĩ·(t_dln_A + 1), t_dln_A = T·dln A_mix/dT.
    let t_dln_a = t_dln_a_dt_mix(spec, t, p, x)?;
    Ok((z - 1.0) + a_mix * itilde * (t_dln_a + 1.0))
}

/// Dimensionless residual (departure) entropy `S^R/R` of a mixture from the
/// EOS, for the requested phase.
///
/// From Lewis-Randall: `S^R/R = H^R/(RT) − G^R/(RT)`, with the total
/// residual Gibbs `G^R/(RT) = Σᵢ xᵢ·ln φ̂ᵢ`.
///
/// # Errors
/// Propagates [`MixError`].
pub fn s_departure_r_mix(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<f64, MixError> {
    let h_rt = h_departure_rt_mix(spec, t, p, x, phase)?;
    let g_rt: f64 = ln_phi_mix(spec, t, p, x, phase)?
        .iter()
        .zip(x)
        .map(|(lnphi, xi)| xi * lnphi)
        .sum();
    Ok(h_rt - g_rt)
}

/// `(A_mix, Ĩ, Z)` at (T, P, x, phase) — the attractive-term pieces the
/// departure enthalpy needs. Shares the mixture-parameter computation with
/// the fugacity path.
fn mix_attractive_pieces(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<(f64, f64, f64), MixError> {
    let pars = crate::mixture::mixture_params::<f64>(spec, t, p, x)?;
    let z = z_mix(spec, t, p, x, phase)?;
    let (a, u, w) = (pars.big_a, pars.u, pars.w);
    // Ĩ(Z; U, W): the generalized attractive integral (same three branches
    // as the EOS layer). g = A·Ĩ; recover Ĩ = g/A.
    let g = crate::eos::attractive_term_uw(z, a, u, w);
    let itilde = if a.abs() < 1e-300 { 0.0 } else { g / a };
    Ok((a, itilde, z))
}

/// `T·d(ln A_mix)/dT` (dimensionless) — the analytic logarithmic
/// temperature derivative of the mixture attractive parameter, per rule.
///
/// Building blocks (all analytic): per component
/// `T·d(ln Aᵢ)/dT = Tr·(dαᵢ/dTr)/αᵢ − 2` and `T·d(ln Bᵢ)/dT = −1`; for the
/// GE-based rules `T·d(Gᴱ/RT)/dT = −Hᴱ/(RT)` (from `Hᴱ = Gᴱ − T dGᴱ/dT`).
pub fn t_dln_a_dt_mix(spec: &MixtureSpec, t: f64, p: f64, x: &[f64]) -> Result<f64, MixError> {
    let n = x.len();
    // Per-component dimensionless Aᵢ, Bᵢ and their T·d(ln·)/dT factors.
    let mut ai = vec![0.0; n];
    let mut bi = vec![0.0; n];
    let mut t_dln_ai = vec![0.0; n]; // T·d(ln Aᵢ)/dT
    for i in 0..n {
        let comp = &spec.components[i];
        let st = EosState::new(spec.eos, t, p, comp);
        ai[i] = st.big_a;
        bi[i] = st.big_b;
        // Tr·(dαᵢ/dTr)/αᵢ − 2.
        let alpha_prime_over_alpha = st.tr * st.d_alpha_d_tr / st.alpha;
        t_dln_ai[i] = alpha_prime_over_alpha - 2.0;
    }
    let kij_at = |i: usize, j: usize| {
        if spec.kij.is_empty() {
            0.0
        } else {
            spec.kij[i][j]
        }
    };

    // T·dA_mix/dT (dimensionless), then divide by A_mix at the end.
    let (a_mix, t_da_mix): (f64, f64) = match spec.rule {
        // Classical quadratic families: A = ΣΣ xᵢxⱼ(1−kmᵢⱼ)√(AᵢAⱼ), with
        // kmᵢⱼ = kᵢⱼ (Classical/IVDW) or the composition-weighted km (IIVDW).
        // Only Aᵢ(T) carries T-dependence, so
        //   T·dA/dT = ΣΣ xᵢxⱼ(1−km)√(AᵢAⱼ)·½(T dlnAᵢ + T dlnAⱼ).
        MixingRule::Classical | MixingRule::IVDW | MixingRule::IIVDW => {
            let iivdw = spec.rule == MixingRule::IIVDW;
            let mut a_mix = 0.0;
            let mut t_da = 0.0;
            for i in 0..n {
                for j in 0..n {
                    let km = if iivdw {
                        x[i] * kij_at(i, j) + x[j] * kij_at(j, i)
                    } else {
                        kij_at(i, j)
                    };
                    let aij = (1.0 - km) * (ai[i] * ai[j]).sqrt();
                    let xx = x[i] * x[j];
                    a_mix += xx * aij;
                    t_da += xx * aij * 0.5 * (t_dln_ai[i] + t_dln_ai[j]);
                }
            }
            (a_mix, t_da)
        }

        // GE-based rules: A_mix = B·α_mix. B = Σxᵢ Bᵢ (linear), so
        // T·dB/dT = Σxᵢ·(−Bᵢ) = −B. α_mix carries the αᵢ = Aᵢ/Bᵢ terms
        // (T·d(ln αᵢ)/dT = T dlnAᵢ + 1) and the Gᴱ term
        // (T·d(Gᴱ/RT)/dT = −Hᴱ/RT). MHV2 handles the quadratic α below.
        MixingRule::HuronVidalOriginal
        | MixingRule::HuronVidalSimplified
        | MixingRule::MHV1
        | MixingRule::MHV2 => {
            let ge = spec
                .ge
                .ok_or_else(|| MixError::Unsupported("GE rule requires GeSpec".into()))?;
            // B and T·dB/dT.
            let b: f64 = (0..n).map(|i| x[i] * bi[i]).sum();
            let t_db = -b; // Σxᵢ·(−Bᵢ)
            // Σxᵢ αᵢ and its T-derivative.
            let mut alpha_sum = 0.0;
            let mut t_dalpha_sum = 0.0;
            for i in 0..n {
                let alpha_i = ai[i] / bi[i];
                alpha_sum += x[i] * alpha_i;
                // T·d αᵢ/dT = αᵢ·(T dlnAᵢ + 1) (since T dlnBᵢ = −1).
                t_dalpha_sum += x[i] * alpha_i * (t_dln_ai[i] + 1.0);
            }
            // Gᴱ/RT and T·d(Gᴱ/RT)/dT = −Hᴱ/RT.
            let g_rt = excess_gibbs_rt(ge.model, x, ge.aij, ge.vl, ge.delta, t);
            let he = consistent_excess_enthalpy(ge.model, x, ge.aij, ge.vl, ge.delta, t);
            let t_dg_rt = -he / (R_GAS * t);
            // Σxᵢ ln(B/Bᵢ) and its T-derivative: T·d ln(B/Bᵢ)/dT =
            // T dlnB − T dlnBᵢ = (−1) − (−1) = 0. So the b-log term is
            // temperature-independent — its T-derivative vanishes.
            match spec.rule {
                MixingRule::HuronVidalOriginal => {
                    let c = crate::mixture::hv_c_constant(spec.eos);
                    let alpha_mix = alpha_sum + g_rt / c;
                    let t_dalpha_mix = t_dalpha_sum + t_dg_rt / c;
                    let a_mix = b * alpha_mix;
                    // T·dA/dT = T dB/dT·α + B·T dα/dT.
                    (a_mix, t_db * alpha_mix + b * t_dalpha_mix)
                }
                MixingRule::HuronVidalSimplified | MixingRule::MHV1 => {
                    let c = if spec.rule == MixingRule::MHV1 {
                        -0.593
                    } else {
                        crate::mixture::hv_c_constant(spec.eos)
                    };
                    let blog: f64 = (0..n).map(|i| x[i] * (b / bi[i]).ln()).sum();
                    let alpha_mix = alpha_sum + (g_rt + blog) / c;
                    // b-log term T-derivative is 0 (see above).
                    let t_dalpha_mix = t_dalpha_sum + t_dg_rt / c;
                    let a_mix = b * alpha_mix;
                    (a_mix, t_db * alpha_mix + b * t_dalpha_mix)
                }
                MixingRule::MHV2 => {
                    // q₂α² + q₁α = Σxᵢ(q₁αᵢ+q₂αᵢ²) + Gᴱ/RT + Σxᵢln(B/Bᵢ).
                    // Differentiate implicitly: (q₁+2q₂α)·dα/dT = d(rhs)/dT.
                    let (q1, q2) = (-0.478, -0.0047);
                    let blog: f64 = (0..n).map(|i| x[i] * (b / bi[i]).ln()).sum();
                    let mut rhs = g_rt + blog;
                    for i in 0..n {
                        let alpha_i = ai[i] / bi[i];
                        rhs += x[i] * (q1 * alpha_i + q2 * alpha_i * alpha_i);
                    }
                    let disc = (q1 * q1 + 4.0 * q2 * rhs).sqrt();
                    let r1 = (-q1 + disc) / (2.0 * q2);
                    let r2 = (-q1 - disc) / (2.0 * q2);
                    let alpha_mix = if r1 >= r2 { r1 } else { r2 };
                    // T·d(rhs)/dT: Gᴱ term (−Hᴱ/RT), b-log (0), and the
                    // Σxᵢ(q₁αᵢ+q₂αᵢ²) term.
                    let mut t_drhs = t_dg_rt;
                    for i in 0..n {
                        let alpha_i = ai[i] / bi[i];
                        let t_dalpha_i = alpha_i * (t_dln_ai[i] + 1.0);
                        t_drhs += x[i] * (q1 + 2.0 * q2 * alpha_i) * t_dalpha_i;
                    }
                    let t_dalpha_mix = t_drhs / (q1 + 2.0 * q2 * alpha_mix);
                    let a_mix = b * alpha_mix;
                    (a_mix, t_db * alpha_mix + b * t_dalpha_mix)
                }
                _ => unreachable!(),
            }
        }

        // Wong-Sandler: B = Q/(1−D), A = B·D, both nonlinear in T. Full
        // analytic chain through Q(T), D(T).
        MixingRule::WongSandler => {
            let ge = spec
                .ge
                .ok_or_else(|| MixError::Unsupported("WS requires GeSpec".into()))?;
            let c_star = crate::mixture::hv_c_constant(spec.eos);
            // Q = ΣΣ xᵢxⱼ·½[(Bᵢ−Aᵢ)+(Bⱼ−Aⱼ)](1−kᵢⱼ).
            // T·dQ/dT: T d(Bᵢ−Aᵢ)/dT = −Bᵢ − Aᵢ·T dlnAᵢ.
            let mut q = 0.0;
            let mut t_dq = 0.0;
            for i in 0..n {
                for j in 0..n {
                    let bij = 0.5 * ((bi[i] - ai[i]) + (bi[j] - ai[j])) * (1.0 - kij_at(i, j));
                    let t_dbij = 0.5
                        * ((-bi[i] - ai[i] * t_dln_ai[i]) + (-bi[j] - ai[j] * t_dln_ai[j]))
                        * (1.0 - kij_at(i, j));
                    let xx = x[i] * x[j];
                    q += xx * bij;
                    t_dq += xx * t_dbij;
                }
            }
            // D = Σxᵢαᵢ + Gᴱ/(c*·RT); T·dD/dT = Σxᵢαᵢ(T dlnAᵢ+1) − Hᴱ/(c*RT).
            let mut d = 0.0;
            let mut t_dd = 0.0;
            for i in 0..n {
                let alpha_i = ai[i] / bi[i];
                d += x[i] * alpha_i;
                t_dd += x[i] * alpha_i * (t_dln_ai[i] + 1.0);
            }
            let g_rt = excess_gibbs_rt(ge.model, x, ge.aij, ge.vl, ge.delta, t);
            let he = consistent_excess_enthalpy(ge.model, x, ge.aij, ge.vl, ge.delta, t);
            d += g_rt / c_star;
            t_dd += (-he / (R_GAS * t)) / c_star;
            let one_minus_d = 1.0 - d;
            let b = q / one_minus_d;
            // T·dB/dT = [T dQ·(1−D) + Q·T dD]/(1−D)².
            let t_db = (t_dq * one_minus_d + q * t_dd) / (one_minus_d * one_minus_d);
            let a_mix = b * d;
            // T·dA/dT = T dB·D + B·T dD.
            (a_mix, t_db * d + b * t_dd)
        }

        MixingRule::PatelTejaC | MixingRule::PatelTejaUSBC | MixingRule::SchmidtWenzelC => {
            return Err(MixError::Unsupported(
                "C-parameter rule is implied by the 3-parameter EOS; pass Classical".into(),
            ));
        }
    };

    if a_mix.abs() < 1e-300 {
        return Ok(0.0);
    }
    Ok(t_da_mix / a_mix)
}

/// Dimensionless excess Gibbs energy Gᴱ/(R·T) for the coupled activity
/// model. Thin f64 wrapper over the activity helper.
fn excess_gibbs_rt(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    t: f64,
) -> f64 {
    crate::activity::excess_gibbs(model, x, aij, vl, delta, t) / (R_GAS * t)
}

/// Excess enthalpy **consistent with the temperature dependence of γ as the
/// engine actually implements it** — the quantity that makes
/// `T·d(Gᴱ/RT)/dT = −Hᴱ_consistent/(RT)` exact against `mixture_params`.
///
/// This differs from [`crate::activity::excess_enthalpy`] for Margules and
/// van Laar only. Those models' `ln γ` use **dimensionless, temperature-
/// independent** `aij` here, so the `Gᴱ/RT = Σ xᵢ ln γᵢ` that feeds the
/// mixing rule is genuinely T-independent → its consistent excess enthalpy
/// is **0**. The legacy `Hᴱ = Gᴱ` convention (activity.rs) instead assumes
/// the `aij` are energies; that convention is right for the γ-φ *liquid*
/// enthalpy path but would be inconsistent with the EOS-departure T-
/// derivative, where what matters is how `Gᴱ(T)` actually varies in the
/// code. Wilson and Scatchard-Hildebrand already carry the correct T-
/// dependence in `ln γ`, so their `excess_enthalpy` is used directly.
fn consistent_excess_enthalpy(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    t: f64,
) -> f64 {
    match model {
        ActivityModel::Margules | ActivityModel::VanLaar => 0.0,
        _ => excess_enthalpy(model, x, aij, vl, delta, t),
    }
}

/// One phase's total molar enthalpy and entropy (ideal + residual),
/// relative to a shared ideal-gas reference state.
///
/// This is the φ-φ (EOS) path: both H^R and S^R come from the mixture EOS
/// departure. `t_ref`, `p_ref` define the reference; `h_ref`/`s_ref` are
/// per-component Hᵢ°/Sᵢ° (pass `&[]` for the usual all-zero convention).
///
/// # Returns
/// `(H, S)` — enthalpy in **kJ/kmol**, entropy in **kJ/(kmol·K)**.
///
/// # Errors
/// Propagates [`MixError`].
#[allow(clippy::too_many_arguments)]
pub fn phase_enthalpy_entropy(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
    t_ref: f64,
    p_ref: f64,
    h_ref: &[f64],
    s_ref: &[f64],
) -> Result<(f64, f64), MixError> {
    let h_ideal = ideal_enthalpy_mix(spec.components, x, t, t_ref, h_ref);
    let s_ideal = ideal_entropy_mix(spec.components, x, t, p, t_ref, p_ref, s_ref);
    let h_res = h_departure_rt_mix(spec, t, p, x, phase)? * R_GAS * t;
    let s_res = s_departure_r_mix(spec, t, p, x, phase)? * R_GAS;
    Ok((h_ideal + h_res, s_ideal + s_res))
}

/// Partial molar enthalpy H̄ᵢ of every component in one phase, in **kJ/kmol**
/// (M12.4).
///
/// Built from the exact identity `H̄ᵢ = h°ᵢ(T) − R·T²·∂ln φ̂ᵢ/∂T` — the
/// per-component ideal-gas enthalpy plus the residual partial molar enthalpy
/// `H̄ᵢ^R = −R·T²·∂ln φ̂ᵢ/∂T` (26) Michelsen & Mollerup. No new differentiation
/// machinery: it rides on M12.3's exact [`crate::mixture::d_ln_phi_d_t`].
///
/// The composition-weighted sum equals the total phase enthalpy
/// `Σᵢ xᵢ·H̄ᵢ = H` (Euler), used as a test invariant.
///
/// * `t` in **K**, `p` in **kPa absolute**; `t_ref` the ideal-gas reference
///   temperature in **K**; `h_ref` the per-component Hᵢ° (pass `&[]` for the
///   all-zero convention).
#[allow(clippy::too_many_arguments)]
pub fn partial_molar_enthalpy(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
    t_ref: f64,
    h_ref: &[f64],
) -> Result<Vec<f64>, MixError> {
    let d_ln_phi_dt = crate::mixture::d_ln_phi_d_t(spec, t, p, x, phase)?;
    let rt2 = R_GAS * t * t;
    Ok((0..x.len())
        .map(|i| {
            let h0 = h_ref.get(i).copied().unwrap_or(0.0);
            h0 + ideal_enthalpy_integral(&spec.components[i], t, t_ref) - rt2 * d_ln_phi_dt[i]
        })
        .collect())
}

/// Real-mixture isobaric heat capacity Cp of one phase, in **kJ/(kmol·K)**
/// (M12.4).
///
/// `Cp = Σᵢ xᵢ·Cpᵢ°(T) + Cp^R`, the ideal-gas mixture heat capacity plus the
/// residual [`crate::mixture::residual_cp`] (a second-order dual through the
/// T-generic fugacity core). Reduces to the ideal-gas value as the residual
/// vanishes (P → 0), a test invariant.
///
/// * `t` in **K**, `p` in **kPa absolute**, `x` mole fractions.
pub fn phase_cp(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<f64, MixError> {
    let cp_ideal: f64 = (0..x.len())
        .map(|i| x[i] * ideal_cp(&spec.components[i], t))
        .sum();
    let cp_res = crate::mixture::residual_cp(spec, t, p, x, phase)?;
    Ok(cp_ideal + cp_res)
}

/// Excess enthalpy Hᴱ and entropy Sᴱ of the liquid mixture (the γ-φ path's
/// non-ideal contribution), re-exported at mixture level for the activity-
/// coefficient liquid model. Units: Hᴱ in **kJ/kmol**, Sᴱ in **kJ/(kmol·K)**.
pub fn excess_h_s(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    t: f64,
) -> (f64, f64) {
    (
        excess_enthalpy(model, x, aij, vl, delta, t),
        excess_entropy(model, x, aij, vl, delta, t),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos::CubicEos;
    use crate::mixture::GeSpec;

    fn methane() -> Component {
        Component {
            name: "methane".into(),
            tc: 190.564,
            pc: 4599.0,
            omega: 0.0115,
            // Plausible ideal-Cp/R polynomial (order of magnitude of a
            // light hydrocarbon; exact values immaterial to these tests).
            cp_coeffs: [4.5, 1.5e-3, 0.0, 0.0, 0.0],
            ..Component::default()
        }
    }

    fn n_pentane() -> Component {
        Component {
            name: "n-pentane".into(),
            tc: 469.7,
            pc: 3370.0,
            omega: 0.252,
            cp_coeffs: [7.0, 5.0e-3, -1.0e-6, 0.0, 0.0],
            ..Component::default()
        }
    }

    fn methanol() -> Component {
        Component {
            name: "methanol".into(),
            tc: 512.6,
            pc: 8097.0,
            omega: 0.564,
            liquid_volume: 40.7,
            cp_coeffs: [4.9, 1.2e-2, -3.0e-6, 0.0, 0.0],
            ..Component::default()
        }
    }

    fn water() -> Component {
        Component {
            name: "water".into(),
            tc: 647.1,
            pc: 22064.0,
            omega: 0.344,
            liquid_volume: 18.07,
            cp_coeffs: [4.0, 1.0e-3, 0.0, 0.0, 0.0],
            ..Component::default()
        }
    }

    fn kij2(k: f64) -> Vec<Vec<f64>> {
        vec![vec![0.0, k], vec![k, 0.0]]
    }

    // -----------------------------------------------------------------
    // Ideal-gas terms.
    // -----------------------------------------------------------------

    #[test]
    fn ideal_cp_and_integral_consistent() {
        // ∫_{Tref}^T Cp dT differentiated numerically must return Cp(T).
        let c = n_pentane();
        let t = 400.0;
        let h = 1e-3;
        let d_int = (ideal_enthalpy_integral(&c, t + h, 298.15)
            - ideal_enthalpy_integral(&c, t - h, 298.15))
            / (2.0 * h);
        assert!(
            (d_int - ideal_cp(&c, t)).abs() < 1e-4,
            "{d_int} vs {}",
            ideal_cp(&c, t)
        );
    }

    #[test]
    fn ideal_entropy_integral_is_cp_over_t() {
        // d/dT ∫(Cp/T)dT = Cp/T.
        let c = methanol();
        let t = 350.0;
        let h = 1e-3;
        let d_int = (ideal_entropy_integral(&c, t + h, 298.15)
            - ideal_entropy_integral(&c, t - h, 298.15))
            / (2.0 * h);
        assert!((d_int - ideal_cp(&c, t) / t).abs() < 1e-6);
    }

    #[test]
    fn ideal_mixing_entropy_binary() {
        // −R(x ln x + (1−x) ln(1−x)); max at x = 0.5 → R·ln 2.
        let s = ideal_mixing_entropy(&[0.5, 0.5]);
        assert!((s - R_GAS * std::f64::consts::LN_2).abs() < 1e-10);
        // Pure limit → 0.
        assert!(ideal_mixing_entropy(&[1.0, 0.0]).abs() < 1e-12);
    }

    // -----------------------------------------------------------------
    // Departure — analytic T-derivative vs numerical oracle.
    // -----------------------------------------------------------------

    /// Central-difference oracle for T·d(ln A_mix)/dT (test-only).
    fn t_dln_a_fd(spec: &MixtureSpec, t: f64, p: f64, x: &[f64]) -> f64 {
        let h = t * 1e-6;
        let a = |tt: f64| {
            crate::mixture::mixture_params::<f64>(spec, tt, p, x)
                .unwrap()
                .big_a
        };
        let (ap, am) = (a(t + h), a(t - h));
        t * (ap.ln() - am.ln()) / (2.0 * h)
    }

    #[test]
    fn analytic_t_derivative_matches_oracle_classical_and_3param() {
        let comps = vec![methane(), n_pentane()];
        let kij = kij2(0.023);
        let x = [0.4, 0.6];
        for eos in [
            CubicEos::PR1976,
            CubicEos::RKS1972,
            CubicEos::SchmidtWenzel,
            CubicEos::PatelTeja,
        ] {
            for rule in [MixingRule::Classical, MixingRule::IVDW, MixingRule::IIVDW] {
                if eos.is_three_parameter() && rule == MixingRule::IIVDW {
                    continue; // 3-param uses classical A; skip the km variant
                }
                let spec = MixtureSpec {
                    eos,
                    rule,
                    components: &comps,
                    kij: &kij,
                    ge: None,
                };
                let analytic = t_dln_a_dt_mix(&spec, 350.0, 2000.0, &x).unwrap();
                let oracle = t_dln_a_fd(&spec, 350.0, 2000.0, &x);
                assert!(
                    (analytic - oracle).abs() < 1e-6 * oracle.abs().max(1.0),
                    "{eos:?}/{rule:?}: analytic {analytic} vs oracle {oracle}"
                );
            }
        }
    }

    #[test]
    fn analytic_t_derivative_matches_oracle_ge_rules() {
        let comps = vec![methanol(), water()];
        let aij = vec![vec![0.0, 0.847], vec![0.522, 0.0]];
        let vl = [40.7, 18.07];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            vl: &vl,
            delta: &[],
        };
        let x = [0.4, 0.6];
        for rule in [
            MixingRule::HuronVidalOriginal,
            MixingRule::HuronVidalSimplified,
            MixingRule::MHV1,
            MixingRule::MHV2,
            MixingRule::WongSandler,
        ] {
            let spec = MixtureSpec {
                eos: CubicEos::PR1976,
                rule,
                components: &comps,
                kij: &kij2(0.05),
                ge: Some(ge),
            };
            let analytic = t_dln_a_dt_mix(&spec, 400.0, 800.0, &x).unwrap();
            let oracle = t_dln_a_fd(&spec, 400.0, 800.0, &x);
            assert!(
                (analytic - oracle).abs() < 1e-5 * oracle.abs().max(1.0),
                "{rule:?}: analytic {analytic} vs oracle {oracle}"
            );
        }
    }

    #[test]
    fn departure_reduces_to_pure() {
        // x = [1]: mixture H^R/S^R must equal the pure EOS departure.
        let comps = [n_pentane()];
        for eos in [CubicEos::PR1976, CubicEos::RKS1972, CubicEos::PatelTeja] {
            let spec = MixtureSpec {
                eos,
                rule: MixingRule::Classical,
                components: &comps,
                kij: &[],
                ge: None,
            };
            let h = h_departure_rt_mix(&spec, 400.0, 2000.0, &[1.0], PhaseId::Vapor).unwrap();
            let s = s_departure_r_mix(&spec, 400.0, 2000.0, &[1.0], PhaseId::Vapor).unwrap();
            let hp =
                crate::eos::h_departure_rt(eos, 400.0, 2000.0, &comps[0], PhaseId::Vapor).unwrap();
            let sp =
                crate::eos::s_departure_r(eos, 400.0, 2000.0, &comps[0], PhaseId::Vapor).unwrap();
            assert!((h - hp).abs() < 1e-9, "{eos:?} H^R: {h} vs {hp}");
            assert!((s - sp).abs() < 1e-9, "{eos:?} S^R: {s} vs {sp}");
        }
    }

    #[test]
    fn departure_lewis_randall_consistency() {
        // S^R/R = H^R/RT − G^R/RT must hold by construction for a binary.
        let comps = vec![methane(), n_pentane()];
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::IVDW,
            components: &comps,
            kij: &kij2(0.023),
            ge: None,
        };
        let x = [0.4, 0.6];
        let h = h_departure_rt_mix(&spec, 350.0, 2000.0, &x, PhaseId::Vapor).unwrap();
        let s = s_departure_r_mix(&spec, 350.0, 2000.0, &x, PhaseId::Vapor).unwrap();
        let g: f64 = ln_phi_mix(&spec, 350.0, 2000.0, &x, PhaseId::Vapor)
            .unwrap()
            .iter()
            .zip(&x)
            .map(|(l, xi)| xi * l)
            .sum();
        assert!((s - (h - g)).abs() < 1e-12);
    }

    #[test]
    fn departure_h_via_direct_energy_oracle() {
        // Independent check of H^R/RT against the thermodynamic identity
        //   H^R/RT = −T·(∂(G^R/RT)/∂T)_P + ... actually use the residual
        // enthalpy from H^R = −R T² ∂(G^R/RT)/∂T |_{P,x}:
        //   H^R/(RT) = −T·∂(G^R/RT)/∂T.
        let comps = vec![methane(), n_pentane()];
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::IVDW,
            components: &comps,
            kij: &kij2(0.023),
            ge: None,
        };
        let x = [0.35, 0.65];
        let (p, phase) = (2000.0, PhaseId::Vapor);
        let g_rt = |t: f64| -> f64 {
            ln_phi_mix(&spec, t, p, &x, phase)
                .unwrap()
                .iter()
                .zip(&x)
                .map(|(l, xi)| xi * l)
                .sum()
        };
        let t = 350.0;
        let h = 1e-2;
        let dgrt_dt = (g_rt(t + h) - g_rt(t - h)) / (2.0 * h);
        let h_rt_oracle = -t * dgrt_dt;
        let h_rt = h_departure_rt_mix(&spec, t, p, &x, phase).unwrap();
        assert!(
            (h_rt - h_rt_oracle).abs() < 1e-5 * h_rt_oracle.abs().max(1.0),
            "analytic {h_rt} vs G-derivative oracle {h_rt_oracle}"
        );
    }

    #[test]
    fn full_phase_enthalpy_entropy_runs() {
        // Smoke: total H/S assemble and stay finite for a real binary.
        let comps = vec![methane(), n_pentane()];
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::IVDW,
            components: &comps,
            kij: &kij2(0.023),
            ge: None,
        };
        let x = [0.4, 0.6];
        let (h, s) = phase_enthalpy_entropy(
            &spec,
            350.0,
            2000.0,
            &x,
            PhaseId::Vapor,
            298.15,
            101.325,
            &[],
            &[],
        )
        .unwrap();
        assert!(h.is_finite() && s.is_finite(), "H={h} S={s}");
    }

    #[test]
    fn excess_h_s_matches_activity_layer() {
        let comps = [methanol(), water()];
        let _ = &comps;
        let aij = vec![vec![0.0, 0.847], vec![0.522, 0.0]];
        let vl = [40.7, 18.07];
        let x = [0.4, 0.6];
        let (he, se) = excess_h_s(ActivityModel::Wilson, &x, &aij, &vl, &[], 340.0);
        assert!(
            (he - excess_enthalpy(ActivityModel::Wilson, &x, &aij, &vl, &[], 340.0)).abs() < 1e-12
        );
        assert!(
            (se - excess_entropy(ActivityModel::Wilson, &x, &aij, &vl, &[], 340.0)).abs() < 1e-12
        );
    }

    // -----------------------------------------------------------------
    // M12.4: partial molar enthalpy, real Cp.
    // -----------------------------------------------------------------

    #[test]
    fn partial_molar_enthalpy_euler_sum_equals_total() {
        // Σxᵢ·H̄ᵢ = H (Euler): partial_molar_enthalpy summed must equal the
        // total phase enthalpy from phase_enthalpy_entropy. Both are analytic
        // (H̄ᵢ via d_ln_phi_d_t, H via the departure enthalpy) and consistent
        // for the classical + HV/MHV rules (WS excluded — see the mixture-layer
        // Gibbs–Helmholtz note).
        let comps = vec![methane(), n_pentane()];
        let aij = vec![vec![0.0, 0.4], vec![0.4, 0.0]];
        let vl = [37.0, 115.0];
        let ge = GeSpec {
            model: ActivityModel::VanLaar,
            aij: &aij,
            vl: &vl,
            delta: &[],
        };
        let specs = [
            MixtureSpec {
                eos: CubicEos::PR1976,
                rule: MixingRule::Classical,
                components: &comps,
                kij: &kij2(0.02),
                ge: None,
            },
            MixtureSpec {
                eos: CubicEos::PR1976,
                rule: MixingRule::MHV1,
                components: &comps,
                kij: &kij2(0.02),
                ge: Some(ge),
            },
        ];
        let (t, p, x) = (360.0, 1500.0, [0.4, 0.6]);
        for spec in &specs {
            for phase in [PhaseId::Vapor, PhaseId::Liquid] {
                let Ok(hbar) = partial_molar_enthalpy(spec, t, p, &x, phase, 298.15, &[]) else {
                    continue;
                };
                let sum: f64 = (0..2).map(|i| x[i] * hbar[i]).sum();
                let (h, _) =
                    phase_enthalpy_entropy(spec, t, p, &x, phase, 298.15, 101.325, &[], &[])
                        .unwrap();
                assert!(
                    (sum - h).abs() <= 1e-6 * h.abs().max(1.0),
                    "{:?} {phase:?}: Σx·H̄={sum} vs H={h}",
                    spec.rule
                );
            }
        }
    }

    #[test]
    fn phase_cp_matches_fd_of_enthalpy_and_ideal_limit() {
        // Cp = dH/dT: the analytic phase_cp must match a central difference of
        // the total enthalpy (oracle). And as P → 0 the residual vanishes, so
        // Cp → Σxᵢ·Cpᵢ°.
        let comps = vec![methane(), n_pentane()];
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &kij2(0.02),
            ge: None,
        };
        let (p, x) = (2000.0, [0.4, 0.6]);
        let t = 360.0;
        let cp = phase_cp(&spec, t, p, &x, PhaseId::Vapor).unwrap();
        // FD oracle of H(T).
        let h = 1e-2;
        let (h_hi, _) = phase_enthalpy_entropy(
            &spec,
            t + h,
            p,
            &x,
            PhaseId::Vapor,
            298.15,
            101.325,
            &[],
            &[],
        )
        .unwrap();
        let (h_lo, _) = phase_enthalpy_entropy(
            &spec,
            t - h,
            p,
            &x,
            PhaseId::Vapor,
            298.15,
            101.325,
            &[],
            &[],
        )
        .unwrap();
        let cp_fd = (h_hi - h_lo) / (2.0 * h);
        assert!(
            (cp - cp_fd).abs() <= 1e-4 * cp.abs().max(1.0),
            "Cp={cp} vs FD={cp_fd}"
        );
        // Ideal-gas limit at very low P.
        let cp_lowp = phase_cp(&spec, t, 1e-3, &x, PhaseId::Vapor).unwrap();
        let cp_ideal: f64 = (0..2).map(|i| x[i] * ideal_cp(&comps[i], t)).sum();
        assert!(
            (cp_lowp - cp_ideal).abs() <= 1e-3 * cp_ideal.abs().max(1.0),
            "low-P Cp={cp_lowp} vs ideal={cp_ideal}"
        );
    }

    #[test]
    fn gamma_phi_liquid_enthalpy_ideal_minus_condensation() {
        // γ-φ liquid enthalpy via the SystemSpec dispatch: hand-assemble
        // ideal − condensation + excess and compare. Uses the flash-layer
        // entry point (needs a SystemSpec).
        use crate::eos::{LiquidModel, VaporModel};
        use crate::flash::{SystemSpec, phase_enthalpy_entropy as sys_hs};
        let comps = [methanol(), water()];
        let aij = vec![vec![0.0, 0.847], vec![0.522, 0.0]];
        let vl = [40.7, 18.07];
        // methanol/water need psat coeffs for the condensation term.
        let mut a = methanol();
        a.psat_coeffs = vec![5.20, 3200.0, -35.0];
        let mut b = water();
        b.psat_coeffs = vec![5.11, 3800.0, -46.0];
        let comps = [a, b];
        let _ = &comps;
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::VanLaar),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let (t, p, x) = (340.0, 100.0, [0.4, 0.6]);
        let (h, _s) = sys_hs(&spec, t, p, &x, PhaseId::Liquid, 298.15, 101.325, &[], &[]).unwrap();
        // Hand assembly.
        const R: f64 = 8.31451;
        let h_ideal = ideal_enthalpy_mix(&comps, &x, t, 298.15, &[]);
        let mut h_cond = 0.0;
        for i in 0..2 {
            let psat_i = crate::saturation::psat(comps[i].sat_model, &comps[i], t).unwrap();
            let dpsat = crate::saturation::d_psat_dt(comps[i].sat_model, &comps[i], t).unwrap();
            h_cond += x[i] * R * t * t * dpsat / psat_i;
        }
        let (he, _) = excess_h_s(ActivityModel::VanLaar, &x, &aij, &vl, &[], t);
        let expect = h_ideal - h_cond + he;
        assert!(
            (h - expect).abs() <= 1e-9 * expect.abs().max(1.0),
            "γ-φ liquid H={h} vs hand={expect}"
        );
        // The liquid must be well below the ideal-gas enthalpy (condensation).
        assert!(
            h < h_ideal,
            "liquid H={h} should be below ideal gas {h_ideal}"
        );
    }
}
