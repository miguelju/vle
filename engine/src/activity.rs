//! Activity coefficient models for liquid-phase non-ideality.
//!
//! Activity coefficients (γᵢ) quantify how much a component's behavior in a
//! liquid mixture deviates from ideal solution (Raoult's law). A value of γ = 1
//! means ideal behavior; γ > 1 indicates positive deviation (components "dislike"
//! each other); γ < 1 indicates negative deviation (favorable interactions).
//!
//! These models are used in the γ-φ approach to VLE, where the liquid phase is
//! described by activity coefficients and the vapor phase by an EOS. This is
//! preferred over the φ-φ (EOS-only) approach for highly non-ideal liquid
//! mixtures such as water + alcohol, where cubic EOS gives poor liquid predictions.
//!
//! Five of the six models are identical in both legacy codebases (VB6
//! `clsActivityMulticomp.cls` and Pascal `TERMOIII.PAS`). **NRTL** (Renon &
//! Prausnitz, 1968) has **no legacy counterpart** — it was added in Milestone 14
//! for the aqueous-associating / polar mixtures (ammonia–water and the
//! alcohol–water ladder) the downstream `stages-thermo` library needs. Each
//! model requires binary interaction parameters (Aij) fit to experimental VLE
//! data.
//!
//! # The `aij` matrix convention
//!
//! Every multicomponent function below takes an N×N `aij` slice-of-slices whose
//! meaning depends on the model (this mirrors the legacy `CoefAIJ` matrix):
//!
//! - **Wilson** — `aij[i][j] = λᵢⱼ − λᵢᵢ`, an interaction *energy* in
//!   **kJ/kmol**. The diagonal is unused (Λᵢᵢ ≡ 1).
//! - **NRTL** — `aij[i][j] = gᵢⱼ − gⱼⱼ`, an interaction *energy* in
//!   **kJ/kmol** (so `τᵢⱼ = aij[i][j]/(R·T)`). The diagonal is 0 (τᵢᵢ = 0).
//!   NRTL additionally needs the symmetric non-randomness matrix `alpha`
//!   (`αᵢⱼ = αⱼᵢ`, dimensionless, typically 0.2–0.47); every other model
//!   ignores `alpha`.
//! - **van Laar** — `aij[i][j]` are dimensionless, with `aij[i][i] = 0`.
//! - **Margules** — binary only: `aij[0][1] = A₁₂`, `aij[1][0] = A₂₁`,
//!   dimensionless.
//! - **Scatchard-Hildebrand** / **Ideal** — `aij` is ignored (Scatchard uses the
//!   solubility parameters `delta` and the liquid volumes `vl`).
//!
//! # Excess properties (eqs 2.44–2.46 of the research paper)
//!
//! `Gᴱ = RT Σᵢ xᵢ ln γᵢ`, `Hᴱ = −T² ∂(Gᴱ/T)/∂T`, `Sᴱ = (Hᴱ − Gᴱ)/T`. Per
//! CLAUDE.md the temperature derivative is **analytical**. For Wilson the only
//! temperature dependence is the Boltzmann factor in Λᵢⱼ, which differentiates to
//! a closed form; for **NRTL** the T-dependence enters through `τᵢⱼ = aij/(R·T)`
//! and `Gᵢⱼ = exp(−αᵢⱼτᵢⱼ)`, and the analytic `Hᴱ` comes for free from one
//! `num-dual` evaluation seeded on T (the same "generic value path carries exact
//! derivatives" trick the mixture core uses — FD survives only as a test oracle).
//! For Margules, van Laar and Scatchard-Hildebrand the legacy programs treat `Gᴱ`
//! as temperature-independent over the excess-property derivative (`Hᴱ = Gᴱ`,
//! `Sᴱ = 0`), and we reproduce that exactly.

// The generic full-vector helpers index parallel arrays (xₖ, Λₖⱼ) in nested
// `for k/j in 0..n` loops that mirror the local-composition sums; allow the
// range-loop lint here rather than obscure the activity-model formulas.
#![allow(clippy::needless_range_loop)]

/// Activity coefficient model for liquid-phase non-ideality.
///
/// Each model computes ln(γᵢ) from composition and binary parameters (Aij),
/// and provides analytical excess enthalpy HE (via dGE/dT) for enthalpy
/// departure calculations in adiabatic flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum ActivityModel {
    /// Ideal solution (γᵢ = 1 for all components at all compositions).
    /// No binary parameters needed. Used as a baseline or for nearly ideal mixtures.
    IdealSolution = 25,
    /// van Laar model. Two-suffix equation derived from van der Waals EOS.
    /// Parameters: A₁₂, A₂₁ (asymmetric). Good for simple non-polar mixtures.
    /// Cannot predict liquid-liquid immiscibility.
    VanLaar = 21,
    /// Wilson model. Local composition equation using molar volume ratios.
    /// Parameters: Λ₁₂, Λ₂₁ (related to energy differences). Good for
    /// polar/non-polar mixtures. Cannot predict liquid-liquid immiscibility.
    Wilson = 22,
    /// Scatchard-Hildebrand (regular solution theory). Based on solubility
    /// parameters (δᵢ) and liquid molar volumes (Vᵢᴸ). Semi-predictive —
    /// requires only pure-component data, no binary fitting needed.
    /// Limited to non-polar mixtures.
    ScatchardHildebrand = 23,
    /// Margules model. Two-suffix equation — the simplest empirical activity
    /// coefficient model. Parameters: A₁₂, A₂₁ (asymmetric). Useful for
    /// quick estimates but limited accuracy for strongly non-ideal systems.
    Margules = 24,
    /// NRTL (Non-Random Two-Liquid, Renon & Prausnitz 1968). Local-composition
    /// model with three binary knobs per pair (τ₁₂, τ₂₁ from the energies
    /// `aij[i][j] = gᵢⱼ − gⱼⱼ` in **kJ/kmol**, plus the symmetric
    /// non-randomness `αᵢⱼ`). The standard model for aqueous-associating and
    /// polar mixtures; unlike Wilson it can represent liquid-liquid splits.
    ///
    /// **No legacy counterpart** — the discriminant `37` is project-assigned:
    /// the legacy VB6 model-ID space packs `CubicEos` 0–20, activity 21–25,
    /// mixing rules 26–33, and project C-rules 34–36, so `37` is the first free
    /// value and can never collide with a legacy ID.
    Nrtl = 37,
}

use crate::types::R_GAS;

/// Gas constant expressed in **cal/(mol·K)** (≈ 1.987), used by the
/// Scatchard-Hildebrand model where the solubility parameter is in
/// (cal/cm³)^0.5. `R_GAS · 0.23898` matches the legacy `R*0.23898` conversion.
const R_CAL: f64 = R_GAS * 0.23898;
/// Converts a cal/mol energy into the canonical **kJ/kmol** (= 1/0.23898).
const CAL_TO_KJ_PER_KMOL: f64 = 4.18445;

/// ln(γᵢ) for component `i` in a liquid mixture.
///
/// Ref (4): Da Silva & Báez (1989), `legacy/pascal/TERMOIII.PAS`; VB6
/// `legacy/vb6/clsActivityMulticomp.cls:74`. Formulas: research paper Table 2.3.
/// (NRTL has no legacy source — Renon & Prausnitz, 1968.)
///
/// # Arguments
/// * `model` — the activity model.
/// * `i` — 0-based component index.
/// * `x` — mole fractions (length N).
/// * `aij` — N×N interaction matrix; see the module docs for the per-model
///   convention. May be empty for `IdealSolution`/`ScatchardHildebrand`.
/// * `alpha` — N×N symmetric NRTL non-randomness matrix (dimensionless); used
///   only by `Nrtl`, ignored (may be empty) by every other model.
/// * `vl` — liquid molar volumes Vᵢᴸ in **cm³/mol** (Wilson, Scatchard); may be
///   empty for the other models.
/// * `delta` — solubility parameters δᵢ in **(cal/cm³)^0.5** (Scatchard only).
/// * `temperature` — **K**.
///
/// # Returns
/// ln(γᵢ), dimensionless. γᵢ = exp(result).
#[allow(clippy::too_many_arguments)]
pub fn ln_gamma(
    model: ActivityModel,
    i: usize,
    x: &[f64],
    aij: &[Vec<f64>],
    alpha: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let n = x.len();
    match model {
        ActivityModel::IdealSolution => 0.0,

        ActivityModel::ScatchardHildebrand => {
            // Volume-fraction-weighted average solubility parameter δ_mix.
            let v_tot: f64 = (0..n).map(|k| x[k] * vl[k]).sum();
            let delta_mix: f64 = (0..n).map(|k| x[k] * vl[k] * delta[k] / v_tot).sum();
            let d = delta[i] - delta_mix;
            // ln γᵢ = Vᵢ(δᵢ − δ_mix)² / (R_cal·T). R_cal keeps the cal-based δ²
            // consistent with the cm³/mol volume.
            vl[i] * d * d / (R_CAL * temperature)
        }

        ActivityModel::Margules => {
            // Two-suffix Margules — binary only (research paper Table 2.3).
            let (x1, x2) = (x[0], x[1]);
            let (a12, a21) = (aij[0][1], aij[1][0]);
            if i == 0 {
                x2 * x2 * (a12 + 2.0 * (a21 - a12) * x1)
            } else {
                x1 * x1 * (a21 + 2.0 * (a12 - a21) * x2)
            }
        }

        ActivityModel::VanLaar => {
            // Multicomponent van Laar (CoefAIJ diagonal = 0). For a binary this
            // reduces algebraically to the closed form in Table 2.3.
            let mut sum_ij = 0.0; // Σⱼ xⱼ Aᵢⱼ
            let mut sum_ji = 0.0; // Σⱼ xⱼ Aⱼᵢ
            for j in 0..n {
                sum_ij += x[j] * aij[i][j];
                sum_ji += x[j] * aij[j][i];
            }
            let denom = x[i] * sum_ij + (1.0 - x[i]) * sum_ji;
            if denom.abs() < f64::EPSILON {
                0.0
            } else {
                let ratio = sum_ji / denom;
                sum_ij * (1.0 - x[i]) * ratio * ratio
            }
        }

        ActivityModel::Wilson => {
            // ln γᵢ = 1 − ln(Σⱼ xⱼΛᵢⱼ) − Σₖ [xₖΛₖᵢ / Σⱼ(xⱼΛₖⱼ)].
            let denom_i: f64 = (0..n)
                .map(|j| x[j] * wilson_lambda(i, j, aij, vl, temperature))
                .sum();
            let nested: f64 = (0..n)
                .map(|k| {
                    let dk: f64 = (0..n)
                        .map(|j| x[j] * wilson_lambda(k, j, aij, vl, temperature))
                        .sum();
                    x[k] * wilson_lambda(k, i, aij, vl, temperature) / dk
                })
                .sum();
            1.0 - denom_i.ln() - nested
        }

        // NRTL (Renon & Prausnitz 1968), general multicomponent form:
        //   ln γᵢ = Cᵢ/Sᵢ + Σⱼ (xⱼ Gᵢⱼ / Sⱼ)·(τᵢⱼ − Cⱼ/Sⱼ)
        // with Sⱼ = Σₖ xₖ Gₖⱼ, Cⱼ = Σₖ xₖ τₖⱼ Gₖⱼ, τₖⱼ = aij[k][j]/(R·T),
        // Gₖⱼ = exp(−αₖⱼ τₖⱼ). The single-component path reuses the same
        // column sums (Sⱼ, Cⱼ). See `ln_gamma_all_generic` for the vector form.
        ActivityModel::Nrtl => {
            let tau = |k: usize, j: usize| aij[k][j] / (R_GAS * temperature);
            let g = |k: usize, j: usize| (-alpha[k][j] * tau(k, j)).exp();
            let (s, c) = nrtl_column_sums(x, &tau, &g);
            nrtl_ln_gamma_i(i, x, &tau, &g, &s, &c)
        }
    }
}

/// Column sums `Sⱼ = Σₖ xₖ Gₖⱼ` and `Cⱼ = Σₖ xₖ τₖⱼ Gₖⱼ` for NRTL (f64).
fn nrtl_column_sums(
    x: &[f64],
    tau: &dyn Fn(usize, usize) -> f64,
    g: &dyn Fn(usize, usize) -> f64,
) -> (Vec<f64>, Vec<f64>) {
    let n = x.len();
    let mut s = vec![0.0; n];
    let mut c = vec![0.0; n];
    for j in 0..n {
        for k in 0..n {
            let gkj = g(k, j);
            s[j] += x[k] * gkj;
            c[j] += x[k] * tau(k, j) * gkj;
        }
    }
    (s, c)
}

/// NRTL ln γᵢ from the precomputed column sums (f64).
fn nrtl_ln_gamma_i(
    i: usize,
    x: &[f64],
    tau: &dyn Fn(usize, usize) -> f64,
    g: &dyn Fn(usize, usize) -> f64,
    s: &[f64],
    c: &[f64],
) -> f64 {
    let n = x.len();
    let mut acc = c[i] / s[i];
    for j in 0..n {
        acc += x[j] * g(i, j) / s[j] * (tau(i, j) - c[j] / s[j]);
    }
    acc
}

/// Wilson coefficient Λᵢⱼ = (Vⱼᴸ/Vᵢᴸ)·exp(−aᵢⱼ/RT), with Λᵢᵢ ≡ 1.
///
/// `aij[i][j] = λᵢⱼ − λᵢᵢ` in **kJ/kmol**; `vl` in cm³/mol (the ratio is
/// dimensionless, so the volume units cancel); `t` in K.
fn wilson_lambda(i: usize, j: usize, aij: &[Vec<f64>], vl: &[f64], t: f64) -> f64 {
    if i == j {
        1.0
    } else {
        (vl[j] / vl[i]) * (-aij[i][j] / (R_GAS * t)).exp()
    }
}

/// Cached Wilson Λ matrix for one temperature (M8.2, PERFORMANCE_PROPOSAL
/// §C2). Λᵢⱼ depends only on (T, aij, vl) — NOT on composition — so a
/// flash iteration that updates x at fixed T can reuse one `WilsonCache`
/// across every γ evaluation instead of paying N² `exp` calls each time.
///
/// The matrix is stored flattened row-major (`lambda[i*n + j]`) in a
/// single contiguous buffer — cache-friendly and one allocation total.
#[derive(Debug, Clone)]
pub struct WilsonCache {
    n: usize,
    lambda: Vec<f64>,
}

impl WilsonCache {
    /// Precompute Λᵢⱼ for all pairs at temperature `t` (**K**).
    /// Arguments as in [`ln_gamma`] (`aij` in kJ/kmol, `vl` in cm³/mol).
    pub fn new(aij: &[Vec<f64>], vl: &[f64], t: f64) -> Self {
        let n = vl.len();
        let mut lambda = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                lambda[i * n + j] = wilson_lambda(i, j, aij, vl, t);
            }
        }
        Self { n, lambda }
    }

    /// Λᵢⱼ lookup. **Dimensionless.**
    #[inline]
    pub fn lambda(&self, i: usize, j: usize) -> f64 {
        self.lambda[i * self.n + j]
    }

    /// Number of components the cache was built for.
    #[inline]
    pub fn len(&self) -> usize {
        self.n
    }

    /// True when built for zero components (clippy convention: any type
    /// with `len` should offer `is_empty`).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// ln(γᵢ) for **all** components at composition `x`, reusing the
    /// cached Λ matrix.
    ///
    /// Writes into `out` (length n). Cost: one pass computing the row
    /// sums Sₖ = Σⱼ xⱼΛₖⱼ (O(N²)), then O(N²) for the nested term —
    /// versus O(N³) `exp`-heavy work for n calls to [`ln_gamma`]. This
    /// full-vector form is what the GE-based mixing rules (Wong-Sandler,
    /// MHV1/2 — M8.3) consume on their hot path.
    pub fn ln_gamma_all(&self, x: &[f64], out: &mut [f64]) {
        let n = self.n;
        // Sₖ = Σⱼ xⱼ·Λₖⱼ, one row-sum per component. The nested term
        // needs every Sₖ while filling out[i], so S gets its own scratch
        // buffer — a SmallVec, so for n ≤ 8 it lives on the stack and
        // this whole function is allocation-free (M8.2 §C3).
        let mut s: smallvec::SmallVec<[f64; 8]> = smallvec::smallvec![0.0; n];
        for (k, sk) in s.iter_mut().enumerate() {
            let mut acc = 0.0;
            for j in 0..n {
                acc += x[j] * self.lambda(k, j);
            }
            *sk = acc;
        }
        // ln γᵢ = 1 − ln(Sᵢ) − Σₖ xₖ·Λₖᵢ/Sₖ.
        for i in 0..n {
            let mut nested = 0.0;
            for k in 0..n {
                nested += x[k] * self.lambda(k, i) / s[k];
            }
            out[i] = 1.0 - s[i].ln() - nested;
        }
    }
}

/// ln(γᵢ) for **all** components at once.
///
/// Semantically identical to calling [`ln_gamma`] for each `i`, but the
/// Wilson branch routes through a [`WilsonCache`] so the Λ matrix is
/// built once (O(N²) `exp` calls) instead of once per component (O(N³)).
/// The GE-based mixing rules (M8.3) call this on their hot path.
///
/// Writes into `out` (length = number of components). Arguments and
/// units as in [`ln_gamma`].
#[allow(clippy::too_many_arguments)]
pub fn ln_gamma_all(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    alpha: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
    out: &mut [f64],
) {
    match model {
        ActivityModel::Wilson => {
            WilsonCache::new(aij, vl, temperature).ln_gamma_all(x, out);
        }
        _ => {
            for (i, o) in out.iter_mut().enumerate() {
                *o = ln_gamma(model, i, x, aij, alpha, vl, delta, temperature);
            }
        }
    }
}

// ===========================================================================
// Generic-scalar activity layer (M8.3, PERFORMANCE_PROPOSAL §B3).
//
// The GE-based mixing rules (Wong-Sandler, Huron-Vidal, MHV1/2) need ln γᵢ
// and Gᴱ/RT evaluated with *dual numbers* so the mixture derivative core can
// differentiate through them exactly. These functions are the same formulas
// as `ln_gamma`/`excess_gibbs` above, written once generic over the scalar
// type D: with D = f64 they compile to the plain arithmetic; with a dual
// type they carry exact derivatives along for free. Only the COMPOSITION is
// generic — temperature and the model parameters stay f64 (composition
// derivatives are what the flash Jacobians need).
// ===========================================================================

use num_dual::DualNum;

/// Wilson Λᵢⱼ generic over the scalar type, so a dual seeded on temperature
/// (M12.3) propagates the exact ∂Λᵢⱼ/∂T. Λᵢⱼ = (vⱼ/vᵢ)·exp(−aᵢⱼ/(R·T)); the
/// diagonal is 1 (a constant, zero derivative). `t` in **K**.
fn wilson_lambda_generic<D: DualNum<f64> + Copy>(
    i: usize,
    j: usize,
    aij: &[Vec<f64>],
    vl: &[f64],
    t: D,
) -> D {
    if i == j {
        D::from(1.0)
    } else {
        // exp(−aᵢⱼ/(R·T)) with T dual: (t·R).recip()·(−aᵢⱼ) inside exp.
        ((t * R_GAS).recip() * (-aij[i][j])).exp() * (vl[j] / vl[i])
    }
}

/// ln(γᵢ) for all components, generic over the scalar type of `x` **and**
/// temperature.
///
/// Same models/units as [`ln_gamma`]; writes into `out` (length n).
/// With `D = f64` this is equivalent to [`ln_gamma_all`]. Making `temperature`
/// generic (M12.3) lets a dual seeded on T flow through the T-dependent models
/// (Wilson's Λᵢⱼ(T), Scatchard's 1/RT, **NRTL's τᵢⱼ(T)/Gᵢⱼ(T)**) so the mixture
/// T-derivative dual path gets exact ∂lnγᵢ/∂T; the T-independent models (Van
/// Laar, Margules, Ideal) simply ignore the parameter.
#[allow(clippy::too_many_arguments)]
pub fn ln_gamma_all_generic<D: DualNum<f64> + Copy>(
    model: ActivityModel,
    x: &[D],
    aij: &[Vec<f64>],
    alpha: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: D,
    out: &mut [D],
) {
    let n = x.len();
    match model {
        ActivityModel::IdealSolution => {
            for o in out.iter_mut() {
                *o = D::from(0.0);
            }
        }

        ActivityModel::ScatchardHildebrand => {
            // δ_mix = Σ xₖvₖδₖ / Σ xₖvₖ (volume-fraction average).
            let mut v_tot = D::from(0.0);
            let mut num = D::from(0.0);
            for k in 0..n {
                v_tot += x[k] * vl[k];
                num += x[k] * (vl[k] * delta[k]);
            }
            let delta_mix = num / v_tot;
            for i in 0..n {
                let d = -delta_mix + delta[i];
                // vᵢ/(R_cal·T) with T dual: (T·R_cal).recip()·vᵢ.
                out[i] = d * d * ((temperature * R_CAL).recip() * vl[i]);
            }
        }

        ActivityModel::Margules => {
            // Binary-only (legacy convention, research paper Table 2.3).
            let (x1, x2) = (x[0], x[1]);
            let (a12, a21) = (aij[0][1], aij[1][0]);
            out[0] = x2 * x2 * (x1 * (2.0 * (a21 - a12)) + a12);
            out[1] = x1 * x1 * (x2 * (2.0 * (a12 - a21)) + a21);
        }

        ActivityModel::VanLaar => {
            for i in 0..n {
                let mut sum_ij = D::from(0.0); // Σⱼ xⱼ Aᵢⱼ
                let mut sum_ji = D::from(0.0); // Σⱼ xⱼ Aⱼᵢ
                for j in 0..n {
                    sum_ij += x[j] * aij[i][j];
                    sum_ji += x[j] * aij[j][i];
                }
                let one_minus_xi = -x[i] + 1.0;
                let denom = x[i] * sum_ij + one_minus_xi * sum_ji;
                if denom.re().abs() < f64::EPSILON {
                    out[i] = D::from(0.0);
                } else {
                    let ratio = sum_ji / denom;
                    out[i] = sum_ij * one_minus_xi * ratio * ratio;
                }
            }
        }

        ActivityModel::Wilson => {
            // Λ depends only on (T, aij, vl) — plain f64 even when x is dual.
            // Sₖ = Σⱼ xⱼ·Λₖⱼ computed once (the M8.2 cache structure).
            let mut s: smallvec::SmallVec<[D; 8]> = smallvec::smallvec![D::from(0.0); n];
            for k in 0..n {
                let mut acc = D::from(0.0);
                for j in 0..n {
                    acc += x[j] * wilson_lambda_generic(k, j, aij, vl, temperature);
                }
                s[k] = acc;
            }
            for i in 0..n {
                let mut nested = D::from(0.0);
                for k in 0..n {
                    nested += x[k] * wilson_lambda_generic(k, i, aij, vl, temperature) / s[k];
                }
                out[i] = -s[i].ln() - nested + 1.0;
            }
        }

        // NRTL, generic over composition AND temperature. τ and G carry the
        // T-dual, so `excess_enthalpy` gets its analytic Hᴱ from one seed on T.
        //   ln γᵢ = Cᵢ/Sᵢ + Σⱼ (xⱼ Gᵢⱼ / Sⱼ)·(τᵢⱼ − Cⱼ/Sⱼ)
        ActivityModel::Nrtl => {
            let rt = temperature * R_GAS;
            let tau = |k: usize, j: usize| -> D { rt.recip() * aij[k][j] };
            let g = |k: usize, j: usize| -> D { (tau(k, j) * (-alpha[k][j])).exp() };
            // Column sums Sⱼ = Σₖ xₖ Gₖⱼ, Cⱼ = Σₖ xₖ τₖⱼ Gₖⱼ.
            let mut s: smallvec::SmallVec<[D; 8]> = smallvec::smallvec![D::from(0.0); n];
            let mut c: smallvec::SmallVec<[D; 8]> = smallvec::smallvec![D::from(0.0); n];
            for j in 0..n {
                for k in 0..n {
                    let gkj = g(k, j);
                    s[j] += x[k] * gkj;
                    c[j] += x[k] * tau(k, j) * gkj;
                }
            }
            for i in 0..n {
                let mut acc = c[i] / s[i];
                for j in 0..n {
                    acc += x[j] * g(i, j) / s[j] * (tau(i, j) - c[j] / s[j]);
                }
                out[i] = acc;
            }
        }
    }
}

/// Dimensionless excess Gibbs energy Gᴱ/(R·T) = Σᵢ xᵢ ln γᵢ, generic over
/// the scalar type of `x`. The GE-based mixing rules consume this form
/// directly (they always pair Gᴱ with an RT divisor).
#[allow(clippy::too_many_arguments)]
pub fn excess_gibbs_rt_generic<D: DualNum<f64> + Copy>(
    model: ActivityModel,
    x: &[D],
    aij: &[Vec<f64>],
    alpha: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: D,
) -> D {
    let n = x.len();
    let mut lng: smallvec::SmallVec<[D; 8]> = smallvec::smallvec![D::from(0.0); n];
    ln_gamma_all_generic(model, x, aij, alpha, vl, delta, temperature, &mut lng);
    let mut acc = D::from(0.0);
    for i in 0..n {
        acc += x[i] * lng[i];
    }
    acc
}

/// Excess Gibbs energy Gᴱ = RT Σᵢ xᵢ ln γᵢ.
///
/// Research paper eq (2.44). Arguments as in [`ln_gamma`].
///
/// # Returns
/// Gᴱ in **kJ/kmol**.
pub fn excess_gibbs(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    alpha: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let n = x.len();
    let sum: f64 = (0..n)
        .map(|i| x[i] * ln_gamma(model, i, x, aij, alpha, vl, delta, temperature))
        .sum();
    R_GAS * temperature * sum
}

/// Excess enthalpy Hᴱ = −T²·∂(Gᴱ/T)/∂T, evaluated **analytically**.
///
/// Research paper eq (2.45). For Wilson the derivative runs through the
/// Boltzmann factor in Λᵢⱼ and gives a closed form; for **NRTL** it comes from
/// one `num-dual` evaluation seeded on T (τ/G carry the T-dependence — exact,
/// not finite differences). For Margules, van Laar and Scatchard-Hildebrand the
/// legacy treatment is Hᴱ = Gᴱ (the parameters carry the model's temperature
/// dependence implicitly), which this reproduces. Ideal solutions have Hᴱ = 0.
///
/// # Returns
/// Hᴱ in **kJ/kmol**.
pub fn excess_enthalpy(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    alpha: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let n = x.len();
    match model {
        ActivityModel::IdealSolution => 0.0,

        // Hᴱ = Σⱼ xⱼ Vⱼ(δⱼ − δ_mix)²  (in cal/mol), converted to kJ/kmol.
        // This equals Gᴱ; SE is therefore 0.
        ActivityModel::ScatchardHildebrand => {
            let v_tot: f64 = (0..n).map(|k| x[k] * vl[k]).sum();
            let delta_mix: f64 = (0..n).map(|k| x[k] * vl[k] * delta[k] / v_tot).sum();
            let sum: f64 = (0..n)
                .map(|j| {
                    let d = delta[j] - delta_mix;
                    x[j] * vl[j] * d * d
                })
                .sum();
            sum * CAL_TO_KJ_PER_KMOL
        }

        // Legacy convention: Hᴱ = Gᴱ for these two (SE = 0).
        ActivityModel::Margules | ActivityModel::VanLaar => {
            excess_gibbs(model, x, aij, alpha, vl, delta, temperature)
        }

        // Wilson, analytical: Hᴱ = Σⱼ xⱼ·(Σₖ xₖ aⱼₖ Λⱼₖ)/(Σₖ xₖ Λⱼₖ), with the
        // k = j term dropped (aⱼⱼ = 0, Λⱼⱼ = 1). aⱼₖ is already in kJ/kmol.
        ActivityModel::Wilson => (0..n)
            .map(|j| {
                let mut up = 0.0;
                let mut down = 0.0;
                for k in 0..n {
                    let lam = wilson_lambda(j, k, aij, vl, temperature);
                    down += x[k] * lam;
                    if k != j {
                        up += x[k] * aij[j][k] * lam;
                    }
                }
                x[j] * up / down
            })
            .sum(),

        // NRTL, analytic via dual: Gᴱ = RT·g_rt with g_rt = Σxᵢlnγᵢ, so
        // Gᴱ/T = R·g_rt and Hᴱ = −T²·d(Gᴱ/T)/dT = −T²·R·dg_rt/dT. One
        // first-order dual seeded on T gives dg_rt/dT exactly.
        ActivityModel::Nrtl => {
            use num_dual::Dual64;
            let xd: smallvec::SmallVec<[Dual64; 8]> =
                x.iter().map(|&xi| Dual64::from(xi)).collect();
            let td = Dual64::new(temperature, 1.0);
            let g_rt = excess_gibbs_rt_generic(model, &xd, aij, alpha, vl, delta, td);
            -temperature * temperature * R_GAS * g_rt.eps
        }
    }
}

/// Excess entropy Sᴱ = (Hᴱ − Gᴱ)/T.
///
/// Research paper eq (2.46), rearranged from Gᴱ = Hᴱ − T·Sᴱ. Arguments as in
/// [`ln_gamma`].
///
/// # Returns
/// Sᴱ in **kJ/(kmol·K)**.
pub fn excess_entropy(
    model: ActivityModel,
    x: &[f64],
    aij: &[Vec<f64>],
    alpha: &[Vec<f64>],
    vl: &[f64],
    delta: &[f64],
    temperature: f64,
) -> f64 {
    let he = excess_enthalpy(model, x, aij, alpha, vl, delta, temperature);
    let ge = excess_gibbs(model, x, aij, alpha, vl, delta, temperature);
    (he - ge) / temperature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminant_values_match_legacy() {
        assert_eq!(ActivityModel::VanLaar as i32, 21);
        assert_eq!(ActivityModel::Wilson as i32, 22);
        assert_eq!(ActivityModel::ScatchardHildebrand as i32, 23);
        assert_eq!(ActivityModel::Margules as i32, 24);
        assert_eq!(ActivityModel::IdealSolution as i32, 25);
        // NRTL has no legacy counterpart; 37 is the first free ID above the
        // legacy space (mixing rules end at 33, project C-rules at 36).
        assert_eq!(ActivityModel::Nrtl as i32, 37);
    }

    // A zeroed N×N matrix helper.
    fn zeros(n: usize) -> Vec<Vec<f64>> {
        vec![vec![0.0; n]; n]
    }

    #[test]
    fn ideal_solution_is_unity_and_zero_excess() {
        let x = [0.4, 0.6];
        let aij = zeros(2);
        for i in 0..2 {
            assert_eq!(
                ln_gamma(
                    ActivityModel::IdealSolution,
                    i,
                    &x,
                    &aij,
                    &[],
                    &[],
                    &[],
                    300.0
                ),
                0.0
            );
        }
        assert_eq!(
            excess_gibbs(ActivityModel::IdealSolution, &x, &aij, &[], &[], &[], 300.0),
            0.0
        );
        assert_eq!(
            excess_enthalpy(ActivityModel::IdealSolution, &x, &aij, &[], &[], &[], 300.0),
            0.0
        );
    }

    #[test]
    fn margules_matches_table_2_3_closed_form() {
        // ln γ₁ = x₂²[A₁₂ + 2(A₂₁−A₁₂)x₁]; ln γ₂ = x₁²[A₂₁ + 2(A₁₂−A₂₁)x₂].
        let x = [0.3, 0.7];
        let mut aij = zeros(2);
        aij[0][1] = 0.5; // A₁₂
        aij[1][0] = 0.8; // A₂₁
        let g1 = ln_gamma(ActivityModel::Margules, 0, &x, &aij, &[], &[], &[], 300.0);
        let g2 = ln_gamma(ActivityModel::Margules, 1, &x, &aij, &[], &[], &[], 300.0);
        let e1 = x[1] * x[1] * (0.5 + 2.0 * (0.8 - 0.5) * x[0]);
        let e2 = x[0] * x[0] * (0.8 + 2.0 * (0.5 - 0.8) * x[1]);
        assert!((g1 - e1).abs() < 1e-12);
        assert!((g2 - e2).abs() < 1e-12);
    }

    #[test]
    fn van_laar_reduces_to_binary_closed_form() {
        // Table 2.3: ln γ₁ = A₁₂/[1 + (A₁₂x₁)/(A₂₁x₂)]².
        let x = [0.35, 0.65];
        let (a12, a21) = (1.2, 0.9);
        let mut aij = zeros(2);
        aij[0][1] = a12;
        aij[1][0] = a21;
        let g1 = ln_gamma(ActivityModel::VanLaar, 0, &x, &aij, &[], &[], &[], 300.0);
        let g2 = ln_gamma(ActivityModel::VanLaar, 1, &x, &aij, &[], &[], &[], 300.0);
        let r1 = 1.0 + (a12 * x[0]) / (a21 * x[1]);
        let r2 = 1.0 + (a21 * x[1]) / (a12 * x[0]);
        assert!((g1 - a12 / (r1 * r1)).abs() < 1e-12);
        assert!((g2 - a21 / (r2 * r2)).abs() < 1e-12);
    }

    #[test]
    fn wilson_gamma_goes_to_one_for_zero_interaction() {
        // With aᵢⱼ = 0 and equal volumes, Λᵢⱼ = 1 and ln γᵢ = 0.
        let x = [0.5, 0.5];
        let aij = zeros(2);
        let vl = [40.0, 40.0];
        for i in 0..2 {
            let g = ln_gamma(ActivityModel::Wilson, i, &x, &aij, &[], &vl, &[], 320.0);
            assert!(g.abs() < 1e-12, "ln γ should vanish, got {g}");
        }
    }

    #[test]
    fn scatchard_gamma_unity_when_solubility_params_equal() {
        // If δ₁ = δ₂ then δ_mix = δ and every (δᵢ − δ_mix) = 0 → γ = 1.
        let x = [0.3, 0.7];
        let vl = [75.0, 110.0];
        let delta = [9.0, 9.0];
        for i in 0..2 {
            let g = ln_gamma(
                ActivityModel::ScatchardHildebrand,
                i,
                &x,
                &zeros(2),
                &[],
                &vl,
                &delta,
                300.0,
            );
            assert!(g.abs() < 1e-12, "got {g}");
        }
    }

    #[test]
    fn gibbs_duhem_excess_gibbs_is_symmetric_endpoints() {
        // Gᴱ must vanish at the pure-component limits for every model.
        let aij = {
            let mut a = zeros(2);
            a[0][1] = 800.0;
            a[1][0] = 1200.0;
            a
        };
        let alpha = {
            let mut a = zeros(2);
            a[0][1] = 0.3;
            a[1][0] = 0.3;
            a
        };
        let vl = [58.0, 92.0];
        let delta = [7.4, 9.2];
        for model in [
            ActivityModel::Margules,
            ActivityModel::VanLaar,
            ActivityModel::Wilson,
            ActivityModel::ScatchardHildebrand,
            ActivityModel::Nrtl,
        ] {
            for x in [[1.0 - 1e-9, 1e-9], [1e-9, 1.0 - 1e-9]] {
                // Gᴱ → 0 like x_min as a component vanishes; with x_min = 1e-9 the
                // residual is ~1e-3 kJ/kmol, vs. the hundreds–thousands a broken
                // endpoint would leave — so a generous bound still catches bugs.
                let ge = excess_gibbs(model, &x, &aij, &alpha, &vl, &delta, 313.15);
                assert!(ge.abs() < 1e-1, "{model:?}: Gᴱ at pure limit = {ge}");
            }
        }
    }

    #[test]
    fn wilson_excess_enthalpy_matches_numerical_oracle() {
        // Verify the analytical Hᴱ against −T²·d(Gᴱ/T)/dT computed by central
        // difference. This is the test-oracle pattern CLAUDE.md mandates: the
        // analytical form is the shipping code, the numerical form only checks it.
        let x = [0.4, 0.6];
        let mut aij = zeros(2);
        aij[0][1] = 1500.0; // λ₁₂ − λ₁₁  [kJ/kmol]
        aij[1][0] = 2600.0; // λ₂₁ − λ₂₂
        let vl = [74.0, 18.0]; // e.g. 2-propanol / water
        let t = 333.15;
        let h = 1e-2;
        let g_over_t =
            |tt: f64| excess_gibbs(ActivityModel::Wilson, &x, &aij, &[], &vl, &[], tt) / tt;
        let d = (g_over_t(t + h) - g_over_t(t - h)) / (2.0 * h);
        let he_num = -t * t * d;
        let he_ana = excess_enthalpy(ActivityModel::Wilson, &x, &aij, &[], &vl, &[], t);
        assert!(
            (he_ana - he_num).abs() < 1e-2 * he_num.abs().max(1.0),
            "analytical {he_ana} vs numerical {he_num}"
        );
    }

    #[test]
    fn nrtl_excess_enthalpy_matches_numerical_oracle() {
        // Same test-oracle pattern as Wilson: the analytic Hᴱ (dual-number
        // dg_rt/dT) must match a central-difference −T²·d(Gᴱ/T)/dT. NRTL's
        // T-dependence lives in τᵢⱼ = aij/(R·T) and Gᵢⱼ = exp(−αᵢⱼτᵢⱼ).
        let x = [0.35, 0.65];
        let mut aij = zeros(2);
        aij[0][1] = 2400.0; // g₁₂ − g₂₂ [kJ/kmol]
        aij[1][0] = -1100.0; // g₂₁ − g₁₁
        let mut alpha = zeros(2);
        alpha[0][1] = 0.3;
        alpha[1][0] = 0.3;
        let t = 320.0;
        let h = 1e-2;
        let g_over_t =
            |tt: f64| excess_gibbs(ActivityModel::Nrtl, &x, &aij, &alpha, &[], &[], tt) / tt;
        let d = (g_over_t(t + h) - g_over_t(t - h)) / (2.0 * h);
        let he_num = -t * t * d;
        let he_ana = excess_enthalpy(ActivityModel::Nrtl, &x, &aij, &alpha, &[], &[], t);
        assert!(
            (he_ana - he_num).abs() < 1e-4 * he_num.abs().max(1.0),
            "analytical {he_ana} vs numerical {he_num}"
        );
        // A real (nonzero) heat of mixing — this is the whole point of NRTL.
        assert!(he_ana.abs() > 1.0, "expected nonzero Hᴱ, got {he_ana}");
    }

    #[test]
    fn nrtl_matches_binary_closed_form() {
        // Closed-form binary reduction (Renon & Prausnitz):
        //   ln γ₁ = x₂²[ τ₂₁ (G₂₁/(x₁+x₂G₂₁))² + τ₁₂ G₁₂/(x₂+x₁G₁₂)² ]
        //   ln γ₂ = x₁²[ τ₁₂ (G₁₂/(x₂+x₁G₁₂))² + τ₂₁ G₂₁/(x₁+x₂G₂₁)² ]
        let x = [0.4, 0.6];
        let t = 330.0;
        let mut aij = zeros(2);
        aij[0][1] = 1800.0;
        aij[1][0] = 900.0;
        let mut alpha = zeros(2);
        alpha[0][1] = 0.25;
        alpha[1][0] = 0.25;
        let tau12 = aij[0][1] / (R_GAS * t);
        let tau21 = aij[1][0] / (R_GAS * t);
        let g12 = (-alpha[0][1] * tau12).exp();
        let g21 = (-alpha[1][0] * tau21).exp();
        let (x1, x2) = (x[0], x[1]);
        let d21 = x1 + x2 * g21;
        let d12 = x2 + x1 * g12;
        let e1 = x2 * x2 * (tau21 * (g21 / d21).powi(2) + tau12 * g12 / (d12 * d12));
        let e2 = x1 * x1 * (tau12 * (g12 / d12).powi(2) + tau21 * g21 / (d21 * d21));
        let g1 = ln_gamma(ActivityModel::Nrtl, 0, &x, &aij, &alpha, &[], &[], t);
        let g2 = ln_gamma(ActivityModel::Nrtl, 1, &x, &aij, &alpha, &[], &[], t);
        assert!((g1 - e1).abs() < 1e-12, "ln γ₁ {g1} vs {e1}");
        assert!((g2 - e2).abs() < 1e-12, "ln γ₂ {g2} vs {e2}");
    }

    #[test]
    fn margules_van_laar_excess_enthalpy_equals_gibbs() {
        // Legacy convention: Hᴱ = Gᴱ (so Sᴱ = 0) for these two models.
        let x = [0.45, 0.55];
        let mut aij = zeros(2);
        aij[0][1] = 0.7;
        aij[1][0] = 1.1;
        for model in [ActivityModel::Margules, ActivityModel::VanLaar] {
            let ge = excess_gibbs(model, &x, &aij, &[], &[], &[], 298.15);
            let he = excess_enthalpy(model, &x, &aij, &[], &[], &[], 298.15);
            let se = excess_entropy(model, &x, &aij, &[], &[], &[], 298.15);
            assert!((he - ge).abs() < 1e-9);
            assert!(se.abs() < 1e-9);
        }
    }

    // -----------------------------------------------------------------
    // M8.2 — WilsonCache / ln_gamma_all consistency with the per-i path.
    // -----------------------------------------------------------------

    #[test]
    fn wilson_cache_matches_per_component_ln_gamma() {
        // Ternary with asymmetric aij — the cached full-vector path must
        // reproduce the per-component reference to machine precision.
        let x = [0.3, 0.45, 0.25];
        let aij = vec![
            vec![0.0, 1200.0, 800.0],
            vec![-300.0, 0.0, 650.0],
            vec![450.0, -150.0, 0.0],
        ];
        let vl = [90.0, 116.0, 130.0];
        let t = 340.0;
        let cache = WilsonCache::new(&aij, &vl, t);
        let mut got = [0.0; 3];
        cache.ln_gamma_all(&x, &mut got);
        for i in 0..3 {
            let want = ln_gamma(ActivityModel::Wilson, i, &x, &aij, &[], &vl, &[], t);
            assert!(
                (got[i] - want).abs() < 1e-14,
                "component {i}: cached={} reference={}",
                got[i],
                want
            );
        }
    }

    #[test]
    fn ln_gamma_all_matches_per_component_for_every_model() {
        // The dispatching full-vector helper must agree with per-i calls
        // for every model (binary case — Margules/van Laar are binary-only).
        let x = [0.4, 0.6];
        let aij = vec![vec![0.0, 950.0], vec![620.0, 0.0]];
        let alpha = vec![vec![0.0, 0.3], vec![0.3, 0.0]];
        let vl = [90.0, 116.0];
        let delta = [7.5, 8.2];
        let t = 330.0;
        for model in [
            ActivityModel::IdealSolution,
            ActivityModel::Margules,
            ActivityModel::VanLaar,
            ActivityModel::Wilson,
            ActivityModel::ScatchardHildebrand,
            ActivityModel::Nrtl,
        ] {
            let mut got = [0.0; 2];
            ln_gamma_all(model, &x, &aij, &alpha, &vl, &delta, t, &mut got);
            for i in 0..2 {
                let want = ln_gamma(model, i, &x, &aij, &alpha, &vl, &delta, t);
                assert!(
                    (got[i] - want).abs() < 1e-14,
                    "{model:?} component {i}: {} vs {}",
                    got[i],
                    want
                );
            }
        }
    }

    #[test]
    fn nrtl_ternary_generic_matches_f64_per_component() {
        // The generic (dual-capable) vector path must agree with the f64
        // per-component `ln_gamma` on a ternary — exercises the general
        // multicomponent form beyond the binary reduction.
        let x = [0.25, 0.35, 0.40];
        let aij = vec![
            vec![0.0, 1800.0, -600.0],
            vec![900.0, 0.0, 1500.0],
            vec![400.0, -250.0, 0.0],
        ];
        let alpha = vec![
            vec![0.0, 0.30, 0.20],
            vec![0.30, 0.0, 0.47],
            vec![0.20, 0.47, 0.0],
        ];
        let t = 345.0;
        let mut got = [0.0f64; 3];
        ln_gamma_all_generic(ActivityModel::Nrtl, &x, &aij, &alpha, &[], &[], t, &mut got);
        for i in 0..3 {
            let want = ln_gamma(ActivityModel::Nrtl, i, &x, &aij, &alpha, &[], &[], t);
            assert!(
                (got[i] - want).abs() < 1e-12,
                "component {i}: generic={} f64={}",
                got[i],
                want
            );
        }
    }
}
