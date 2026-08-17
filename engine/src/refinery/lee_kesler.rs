//! Lee–Kesler (1975) three-parameter corresponding-states departure functions.
//!
//! # The idea
//!
//! Pitzer's corresponding-states principle says that fluids with the same
//! reduced temperature `Tr = T/Tc`, reduced pressure `Pr = P/Pc` and acentric
//! factor `ω` have the same reduced properties. Lee and Kesler made that
//! quantitative by fitting **two** modified Benedict–Webb–Rubin equations — one
//! to a *simple fluid* (`ω = 0`, argon/krypton/methane) and one to a *reference
//! fluid* (n-octane, `ω_r = 0.3978`) — and interpolating linearly in `ω`:
//!
//! ```text
//!   X(Tr, Pr, ω) = X⁽⁰⁾(Tr, Pr) + (ω / ω_r) · [X⁽ʳ⁾(Tr, Pr) − X⁽⁰⁾(Tr, Pr)]
//! ```
//!
//! for `X` any of `Z`, `(H − H°)/(RTc)`, `(S − S°)/R`, `ln(f/P)`. Each fluid's
//! reduced BWR is
//!
//! ```text
//!   Z = Pr·Vr/Tr = 1 + B/Vr + C/Vr² + D/Vr⁵ + c₄/(Tr³·Vr²)·(β + γ/Vr²)·exp(−γ/Vr²)
//!   B = b₁ − b₂/Tr − b₃/Tr² − b₄/Tr³      C = c₁ − c₂/Tr + c₃/Tr³      D = d₁ + d₂/Tr
//! ```
//!
//! with `Vr = Pc·V/(R·Tc)` (an *ideal* reduced volume, not `V/Vc`), solved for
//! `Vr` at the given `(Tr, Pr)`. The departure functions follow analytically:
//!
//! ```text
//!   (H − H°)/(R·Tc) = Tr·[ Z − 1 − (b₂ + 2b₃/Tr + 3b₄/Tr²)/(Tr·Vr)
//!                        − (c₂ − 3c₃/Tr²)/(2·Tr·Vr²) + d₂/(5·Tr·Vr⁵) + 3E ]
//!   (S − S°)/R      = ln Z − (b₁ + b₃/Tr² + 2b₄/Tr³)/Vr − (c₁ − 2c₃/Tr³)/(2Vr²)
//!                        − d₁/(5Vr⁵) + 2E
//!   ln(f/P)         = Z − 1 − ln Z + B/Vr + C/(2Vr²) + D/(5Vr⁵) + E
//!   E = c₄/(2·Tr³·γ) · [ β + 1 − (β + 1 + γ/Vr²)·exp(−γ/Vr²) ]
//! ```
//!
//! `S°` is the ideal gas at the **same T and P**, so `(S − S°)/R` carries the
//! `ln Z` and no `ln P` term.
//!
//! # Why a refinery wants this
//!
//! It is the API *Technical Data Book*'s recommended enthalpy method for
//! petroleum fractions (Procedure 7B4.7 area) and what refinery flowsheets have
//! been tuned against for fifty years. Compared with a cubic EOS departure it is
//! markedly better for heavy-liquid enthalpies far from the critical point, and
//! it does not care that a pseudocomponent has no `psat` fit — it needs `Tc`,
//! `Pc`, `ω`, which is exactly what [`crate::petroleum`] produces.
//!
//! # Mixtures
//!
//! Lee & Kesler's own pseudo-critical rules (their eq. 20–24; `η = 1`), with
//! Plöcker, Knapp & Prausnitz's `η = 0.25` exponent available because it is what
//! most refinery packages actually run:
//!
//! ```text
//!   Zc,ᵢ = 0.2905 − 0.085·ωᵢ          Vc,ᵢ = Zc,ᵢ·R·Tc,ᵢ/Pc,ᵢ
//!   Vc,ᵢⱼ = (Vc,ᵢ^⅓ + Vc,ⱼ^⅓)³/8      Tc,ᵢⱼ = √(Tc,ᵢ·Tc,ⱼ)
//!   Vc,ₘ = ΣᵢΣⱼ xᵢxⱼ·Vc,ᵢⱼ            Tc,ₘ = Vc,ₘ^(−η)·ΣᵢΣⱼ xᵢxⱼ·Vc,ᵢⱼ^η·Tc,ᵢⱼ
//!   ωₘ = Σᵢ xᵢωᵢ                      Pc,ₘ = (0.2905 − 0.085·ωₘ)·R·Tc,ₘ/Vc,ₘ
//! ```
//!
//! That double sum is the only O(N²) work; everything inside it is a handful of
//! multiplies on precomputed `Vc^⅓` and `√Tc`, so N = 300 pseudocomponents cost
//! ~90 000 flops — well under a microsecond per stage.
//!
//! # Solving for `Vr`
//!
//! The reduced BWR is not a polynomial, so there is no Cardano. Below the
//! critical isotherm it has three roots like a cubic (liquid, unstable, vapor);
//! above, one. `f(Vr) = Pr·Vr/Tr − Z(Vr)` runs from `−∞` at `Vr → 0⁺` to `+∞`,
//! so a bracket always exists. [`lee_kesler_reduced`] scans a log-spaced grid
//! for sign changes, takes the **smallest** bracket for the liquid and the
//! **largest** for the vapor, and finishes with Brent to 1e-12. If only one
//! bracket exists (single-phase region) both phase requests return that root,
//! which is the physically correct answer there. Cost: ~50 evaluations of `Z`,
//! ~2 µs per fluid — negligible next to the mixing rules for any real N.
//!
//! # References
//! - (37) Lee & Kesler, *AIChE J.* **1975**, *21*, 510 — equations and Table 1.
//! - (45) Plöcker, Knapp & Prausnitz, *Ind. Eng. Chem. Process Des. Dev.*
//!   **1978**, *17*, 324 — the `η = 0.25` mixing exponent.

use super::RefineryError;
use crate::eos::PhaseId;
use crate::numerics::root_finding::brent;
use crate::types::{Component, R_GAS};

/// The reference fluid's acentric factor (n-octane).
pub const OMEGA_REFERENCE: f64 = 0.3978;

/// One fluid's reduced-BWR constants (Lee & Kesler 1975, Table 1).
#[derive(Debug, Clone, Copy)]
struct Bwr {
    b1: f64,
    b2: f64,
    b3: f64,
    b4: f64,
    c1: f64,
    c2: f64,
    c3: f64,
    c4: f64,
    d1: f64,
    d2: f64,
    beta: f64,
    gamma: f64,
}

/// Simple fluid (ω = 0).
const SIMPLE: Bwr = Bwr {
    b1: 0.118_119_3,
    b2: 0.265_728,
    b3: 0.154_790,
    b4: 0.030_323,
    c1: 0.023_674_4,
    c2: 0.018_698_4,
    c3: 0.0,
    c4: 0.042_724,
    d1: 0.155_488e-4,
    d2: 0.623_689e-4,
    beta: 0.653_92,
    gamma: 0.060_167,
};

/// Reference fluid (n-octane, ω_r = 0.3978).
const REFERENCE: Bwr = Bwr {
    b1: 0.202_657_9,
    b2: 0.331_511,
    b3: 0.027_655,
    b4: 0.203_488,
    c1: 0.031_338_5,
    c2: 0.050_361_8,
    c3: 0.016_901,
    c4: 0.041_577,
    d1: 0.487_36e-4,
    d2: 0.074_033_6e-4,
    beta: 1.226,
    gamma: 0.037_54,
};

/// Reduced departure functions of one state, all **dimensionless**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LkDeparture {
    /// Compressibility factor `Z = PV/(RT)`.
    pub z: f64,
    /// Enthalpy departure `(H − H°)/(R·T)` — multiply by `R·T` for kJ/kmol.
    /// Negative for a real fluid below its ideal-gas enthalpy.
    pub h_dep_rt: f64,
    /// Entropy departure `(S − S°)/R` with `S°` the ideal gas at the **same
    /// T and P** — multiply by `R` for kJ/(kmol·K).
    pub s_dep_r: f64,
    /// `ln(f/P)`, the mixture (or pure) fugacity coefficient.
    pub ln_phi: f64,
}

/// `(Z, dZ/dVr)` of one fluid at `(Tr, Vr)`.
#[inline]
fn z_and_dz(k: &Bwr, tr: f64, vr: f64) -> (f64, f64) {
    let tr2 = tr * tr;
    let tr3 = tr2 * tr;
    let b = k.b1 - k.b2 / tr - k.b3 / tr2 - k.b4 / tr3;
    let c = k.c1 - k.c2 / tr + k.c3 / tr3;
    let d = k.d1 + k.d2 / tr;
    let iv = 1.0 / vr;
    let iv2 = iv * iv;
    let iv5 = iv2 * iv2 * iv;
    let u = iv2; // Vr⁻²
    let e = (-k.gamma * u).exp();
    let z = 1.0 + b * iv + c * iv2 + d * iv5 + k.c4 / tr3 * (k.beta * u + k.gamma * u * u) * e;
    // d/dVr of the exponential term via u = Vr⁻², du/dVr = −2Vr⁻³.
    let dexp_du =
        e * (k.beta + 2.0 * k.gamma * u - k.gamma * k.beta * u - k.gamma * k.gamma * u * u);
    let dz = -b * iv2 - 2.0 * c * iv2 * iv - 5.0 * d * iv5 * iv
        + k.c4 / tr3 * dexp_du * (-2.0 * iv2 * iv);
    (z, dz)
}

/// Departure functions of one fluid at a converged `(Tr, Vr)`.
fn departure(k: &Bwr, tr: f64, vr: f64) -> LkDeparture {
    let tr2 = tr * tr;
    let tr3 = tr2 * tr;
    let b = k.b1 - k.b2 / tr - k.b3 / tr2 - k.b4 / tr3;
    let c = k.c1 - k.c2 / tr + k.c3 / tr3;
    let d = k.d1 + k.d2 / tr;
    let iv = 1.0 / vr;
    let iv2 = iv * iv;
    let iv5 = iv2 * iv2 * iv;
    let g = k.gamma * iv2;
    let ex = (-g).exp();
    let z = 1.0 + b * iv + c * iv2 + d * iv5 + k.c4 / tr3 * iv2 * (k.beta + g) * ex;
    let e = k.c4 / (2.0 * tr3 * k.gamma) * (k.beta + 1.0 - (k.beta + 1.0 + g) * ex);
    let h_rtc = tr
        * (z - 1.0
            - (k.b2 + 2.0 * k.b3 / tr + 3.0 * k.b4 / tr2) / (tr * vr)
            - (k.c2 - 3.0 * k.c3 / tr2) / (2.0 * tr * vr * vr)
            + k.d2 / (5.0 * tr * vr * vr * vr * vr * vr)
            + 3.0 * e);
    let s_r = z.ln()
        - (k.b1 + k.b3 / tr2 + 2.0 * k.b4 / tr3) * iv
        - (k.c1 - 2.0 * k.c3 / tr3) / (2.0 * vr * vr)
        - k.d1 / (5.0 * vr * vr * vr * vr * vr)
        + 2.0 * e;
    let ln_phi = z - 1.0 - z.ln() + b * iv + c * iv2 / 2.0 + d * iv5 / 5.0 + e;
    LkDeparture {
        z,
        h_dep_rt: h_rtc / tr, // (H−H°)/(R·Tc) → (H−H°)/(R·T)
        s_dep_r: s_r,
        ln_phi,
    }
}

/// Solve `Pr·Vr/Tr = Z(Tr, Vr)` for `Vr` on one fluid, returning the liquid
/// (smallest) or vapor (largest) root. See the module docs for the strategy.
fn solve_vr(k: &Bwr, tr: f64, pr: f64, phase: PhaseId) -> Result<f64, RefineryError> {
    let f = |vr: f64| pr * vr / tr - z_and_dz(k, tr, vr).0;
    // Log-spaced scan from deep in the repulsive wall to well past the
    // ideal-gas volume. 0.02 is below any physical liquid Vr (Zc·V/Vc ≈ 0.1
    // at the triple point); the top end guarantees f > 0.
    let lo = 0.02_f64;
    let hi = (4.0 * tr / pr).max(4.0);
    const N: usize = 60;
    let step = (hi / lo).ln() / N as f64;
    let mut brackets: [(f64, f64); 8] = [(0.0, 0.0); 8];
    let mut nb = 0;
    let mut prev_v = lo;
    let mut prev_f = f(lo);
    for i in 1..=N {
        let v = lo * (step * i as f64).exp();
        let fv = f(v);
        if prev_f <= 0.0 && fv > 0.0 && nb < brackets.len() {
            brackets[nb] = (prev_v, v);
            nb += 1;
        }
        prev_v = v;
        prev_f = fv;
    }
    if nb == 0 {
        return Err(RefineryError::NoConvergence(format!(
            "Lee-Kesler found no reduced-volume root at Tr = {tr:.4}, Pr = {pr:.4}"
        )));
    }
    // A stable phase has (∂P/∂V)_T < 0, i.e. f increasing through the root —
    // which is exactly what a `−` → `+` sign change is. Smallest = liquid,
    // largest = vapor; both coincide when there is only one.
    let (a, b) = match phase {
        PhaseId::Liquid => brackets[0],
        PhaseId::Vapor => brackets[nb - 1],
    };
    let vr = brent(f, a, b, 1e-13, 200).map_err(|e| {
        RefineryError::NoConvergence(format!(
            "Lee-Kesler Vr solve failed at Tr = {tr:.4}, Pr = {pr:.4}: {e}"
        ))
    })?;
    // Two Newton polishes: Brent's xtol is absolute in Vr; for a liquid root
    // near 0.1 that is plenty, but the polish is free and makes the departure
    // functions reproducible to the last bit across platforms.
    let mut v = vr;
    for _ in 0..2 {
        let (z, dz) = z_and_dz(k, tr, v);
        let fv = pr * v / tr - z;
        let dfv = pr / tr - dz;
        if dfv.abs() > 0.0 {
            let nv = v - fv / dfv;
            if nv > 0.0 && nv.is_finite() {
                v = nv;
            }
        }
    }
    Ok(v)
}

fn check_reduced(tr: f64, pr: f64, omega: f64) -> Result<(), RefineryError> {
    if !(tr > 0.0 && tr.is_finite() && pr > 0.0 && pr.is_finite() && omega.is_finite()) {
        return Err(RefineryError::InvalidInput(format!(
            "Lee-Kesler needs Tr > 0, Pr > 0 and a finite ω, got Tr = {tr}, Pr = {pr}, ω = {omega}"
        )));
    }
    Ok(())
}

/// Lee–Kesler departure functions at reduced conditions — the core routine.
///
/// # Arguments
/// * `tr` — reduced temperature `T/Tc`, **dimensionless**.
/// * `pr` — reduced pressure `P/Pc`, **dimensionless**.
/// * `omega` — acentric factor, **dimensionless**.
/// * `phase` — which root of the reduced BWR to take.
///
/// # Returns
/// [`LkDeparture`] — `Z`, `(H − H°)/(RT)`, `(S − S°)/R`, `ln(f/P)`.
///
/// # Errors
/// [`RefineryError::InvalidInput`] on non-positive `Tr`/`Pr`;
/// [`RefineryError::NoConvergence`] if either fluid's `Vr` solve fails.
pub fn lee_kesler_reduced(
    tr: f64,
    pr: f64,
    omega: f64,
    phase: PhaseId,
) -> Result<LkDeparture, RefineryError> {
    check_reduced(tr, pr, omega)?;
    let v0 = solve_vr(&SIMPLE, tr, pr, phase)?;
    let d0 = departure(&SIMPLE, tr, v0);
    if omega == 0.0 {
        return Ok(d0);
    }
    let vr = solve_vr(&REFERENCE, tr, pr, phase)?;
    let dr = departure(&REFERENCE, tr, vr);
    let w = omega / OMEGA_REFERENCE;
    Ok(LkDeparture {
        z: d0.z + w * (dr.z - d0.z),
        h_dep_rt: d0.h_dep_rt + w * (dr.h_dep_rt - d0.h_dep_rt),
        s_dep_r: d0.s_dep_r + w * (dr.s_dep_r - d0.s_dep_r),
        ln_phi: d0.ln_phi + w * (dr.ln_phi - d0.ln_phi),
    })
}

/// Lee–Kesler departure functions of a **pure** component (or a single
/// pseudocomponent) at `t` (**K**) and `p` (**kPa absolute**).
///
/// Reads `tc` (**K**), `pc` (**kPa**) and `omega`. Returns [`LkDeparture`];
/// `h_dep_rt · R_GAS · t` is the enthalpy departure in **kJ/kmol**.
pub fn lee_kesler_departure(
    comp: &Component,
    t: f64,
    p: f64,
    phase: PhaseId,
) -> Result<LkDeparture, RefineryError> {
    if !(comp.tc > 0.0 && comp.pc > 0.0) {
        return Err(RefineryError::InvalidInput(format!(
            "component '{}' needs Tc > 0 and Pc > 0 (got {}, {})",
            comp.name, comp.tc, comp.pc
        )));
    }
    lee_kesler_reduced(t / comp.tc, p / comp.pc, comp.omega, phase)
}

/// Pseudo-critical constants of a mixture under the Lee–Kesler rules.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LkPseudoCritical {
    /// Pseudo-critical temperature, **K**.
    pub tc: f64,
    /// Pseudo-critical pressure, **kPa**.
    pub pc: f64,
    /// Mixture acentric factor `Σ xᵢωᵢ`, **dimensionless**.
    pub omega: f64,
}

/// Lee–Kesler pseudo-critical `(Tc,ₘ, Pc,ₘ, ωₘ)` of a mixture — see the module
/// docs for the rules.
///
/// # Arguments
/// * `components` — reads `tc` (**K**), `pc` (**kPa**), `omega`.
/// * `x` — mole fractions (length N; need not be normalised — they are here).
/// * `eta` — the `Tc,ₘ` volume exponent: `1.0` for Lee & Kesler's original
///   rule, `0.25` for Plöcker–Knapp–Prausnitz.
///
/// # Errors
/// [`RefineryError::InvalidInput`] on a length mismatch, an empty mixture, a
/// non-positive `Tc`/`Pc`, or a composition that does not sum to a positive
/// number.
pub fn lee_kesler_pseudocritical(
    components: &[Component],
    x: &[f64],
    eta: f64,
) -> Result<LkPseudoCritical, RefineryError> {
    let n = components.len();
    if n == 0 || x.len() != n {
        return Err(RefineryError::InvalidInput(format!(
            "components={n}, x={}",
            x.len()
        )));
    }
    let sum: f64 = x.iter().sum();
    if !(sum > 0.0 && sum.is_finite()) || x.iter().any(|&xi| xi < 0.0 || !xi.is_finite()) {
        return Err(RefineryError::InvalidInput(format!(
            "mole fractions must be non-negative and sum to a positive number (sum = {sum})"
        )));
    }
    // Per-component precomputation — the O(N) part. Vc^⅓ and √Tc are all
    // the double loop needs.
    let mut cbrt_vc: smallvec::SmallVec<[f64; 16]> = smallvec::SmallVec::with_capacity(n);
    let mut sqrt_tc: smallvec::SmallVec<[f64; 16]> = smallvec::SmallVec::with_capacity(n);
    let mut omega_m = 0.0;
    for (c, &xi) in components.iter().zip(x) {
        if !(c.tc > 0.0 && c.pc > 0.0) {
            return Err(RefineryError::InvalidInput(format!(
                "component '{}' needs Tc > 0 and Pc > 0 (got {}, {})",
                c.name, c.tc, c.pc
            )));
        }
        let zc = 0.2905 - 0.085 * c.omega;
        let vc = zc * R_GAS * c.tc / c.pc; // m³/kmol — units cancel in Pc,ₘ
        cbrt_vc.push(vc.cbrt());
        sqrt_tc.push(c.tc.sqrt());
        omega_m += xi * c.omega;
    }
    omega_m /= sum;
    // The O(N²) double sum, symmetric so only i ≤ j is visited. `powf` in the
    // inner loop would dominate the whole routine (measured: ~0.3 ms at
    // N = 300), so the two exponents anyone uses get exact cheap forms and
    // only an unusual `eta` pays for a general power.
    let mut vcm = 0.0;
    let mut tnum = 0.0;
    let one = eta == 1.0;
    let quarter = eta == 0.25;
    for i in 0..n {
        let xi = x[i] / sum;
        if xi == 0.0 {
            continue;
        }
        for j in i..n {
            let xj = x[j] / sum;
            if xj == 0.0 {
                continue;
            }
            let s = cbrt_vc[i] + cbrt_vc[j];
            let vcij = s * s * s * 0.125;
            let tcij = sqrt_tc[i] * sqrt_tc[j];
            let w = if i == j { xi * xj } else { 2.0 * xi * xj };
            vcm += w * vcij;
            let vc_eta = if one {
                vcij
            } else if quarter {
                vcij.sqrt().sqrt()
            } else {
                vcij.powf(eta)
            };
            tnum += w * vc_eta * tcij;
        }
    }
    let tcm = tnum / (if one { vcm } else { vcm.powf(eta) });
    let zcm = 0.2905 - 0.085 * omega_m;
    let pcm = zcm * R_GAS * tcm / vcm;
    Ok(LkPseudoCritical {
        tc: tcm,
        pc: pcm,
        omega: omega_m,
    })
}

/// Lee–Kesler departure functions of a **mixture** at `t` (**K**), `p`
/// (**kPa absolute**), via the pseudo-critical rules with exponent `eta`
/// (see [`lee_kesler_pseudocritical`]).
///
/// The returned `h_dep_rt` times `R_GAS · t` is the mixture's residual
/// enthalpy in **kJ/kmol**; add [`crate::energy::ideal_enthalpy_mix`] for the
/// total.
pub fn lee_kesler_departure_mix(
    components: &[Component],
    x: &[f64],
    t: f64,
    p: f64,
    phase: PhaseId,
    eta: f64,
) -> Result<LkDeparture, RefineryError> {
    let pc = lee_kesler_pseudocritical(components, x, eta)?;
    lee_kesler_reduced(t / pc.tc, p / pc.pc, pc.omega, phase)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn methane() -> Component {
        Component {
            name: "methane".into(),
            tc: 190.564,
            pc: 4599.0,
            omega: 0.0115,
            ..Component::default()
        }
    }
    fn n_octane() -> Component {
        Component {
            name: "n-octane".into(),
            tc: 568.7,
            pc: 2490.0,
            omega: 0.3978,
            ..Component::default()
        }
    }
    fn n_decane() -> Component {
        Component {
            name: "n-decane".into(),
            tc: 617.7,
            pc: 2110.0,
            omega: 0.492,
            ..Component::default()
        }
    }

    // === The BWR fits themselves ==========================================

    #[test]
    fn low_pressure_limit_is_the_ideal_gas() {
        for k in [&SIMPLE, &REFERENCE] {
            for tr in [0.7, 1.0, 1.5, 3.0] {
                let vr = solve_vr(k, tr, 1e-6, PhaseId::Vapor).unwrap();
                let d = departure(k, tr, vr);
                assert!((d.z - 1.0).abs() < 1e-5, "Tr={tr}: Z={}", d.z);
                assert!(d.h_dep_rt.abs() < 1e-4, "Tr={tr}: H dep {}", d.h_dep_rt);
                assert!(d.s_dep_r.abs() < 1e-4, "Tr={tr}: S dep {}", d.s_dep_r);
                assert!(d.ln_phi.abs() < 1e-4, "Tr={tr}: ln φ {}", d.ln_phi);
            }
        }
    }

    #[test]
    fn second_virial_coefficient_matches_pitzer_within_a_few_percent() {
        // The B(Tr) of each fit is a second virial coefficient in reduced form
        // and must land near the Pitzer–Tsonopoulos B⁰/B¹ it was built to
        // reproduce. A transcription error in b₁…b₄ shows up here.
        let b = |k: &Bwr, tr: f64| k.b1 - k.b2 / tr - k.b3 / tr.powi(2) - k.b4 / tr.powi(3);
        let b0 = |tr: f64| 0.083 - 0.422 / tr.powf(1.6);
        let b1 = |tr: f64| 0.139 - 0.172 / tr.powf(4.2);
        for tr in [0.8, 1.0, 1.5, 2.0] {
            let simple = b(&SIMPLE, tr);
            let reference = b(&REFERENCE, tr);
            let want0 = b0(tr);
            let want_r = b0(tr) + OMEGA_REFERENCE * b1(tr);
            assert!(
                (simple - want0).abs() < 0.03,
                "Tr={tr}: B⁰ {simple} vs {want0}"
            );
            assert!(
                (reference - want_r).abs() < 0.04,
                "Tr={tr}: Bʳ {reference} vs {want_r}"
            );
        }
    }

    // === Thermodynamic consistency: the three departure formulas agree =====

    #[test]
    fn enthalpy_departure_is_the_temperature_derivative_of_the_fugacity() {
        // Exact identity: (H − H°)/(R·T) = −T·∂ln(f/P)/∂T at constant P, i.e.
        // in reduced form (H − H°)/(R·Tc) = −Tr²·∂ln φ/∂Tr |_Pr. Numerically
        // differentiating ln φ and comparing with the closed-form H tests every
        // b, c, d, β, γ in *both* formulas at once — they would have to be wrong
        // in a correlated way to pass.
        for k in [&SIMPLE, &REFERENCE] {
            for (tr, pr, phase) in [
                (0.7, 0.05, PhaseId::Vapor),
                (0.7, 0.05, PhaseId::Liquid),
                (0.9, 0.5, PhaseId::Vapor),
                (0.9, 0.5, PhaseId::Liquid),
                (1.2, 1.5, PhaseId::Vapor),
                (2.0, 5.0, PhaseId::Vapor),
            ] {
                let at = |t: f64| departure(k, t, solve_vr(k, t, pr, phase).unwrap());
                let d = at(tr);
                let h = 1e-5;
                let dlnphi = (at(tr + h).ln_phi - at(tr - h).ln_phi) / (2.0 * h);
                let want_h_rtc = -tr * tr * dlnphi;
                let got_h_rtc = d.h_dep_rt * tr;
                assert!(
                    (got_h_rtc - want_h_rtc).abs() < 2e-5,
                    "Tr={tr} Pr={pr} {phase:?}: H/RTc {got_h_rtc} vs −Tr²∂lnφ/∂Tr {want_h_rtc}"
                );
                // And Gibbs: (S − S°)/R = (H − H°)/(RT) − ln(f/P).
                let want_s = d.h_dep_rt - d.ln_phi;
                assert!(
                    (d.s_dep_r - want_s).abs() < 1e-10,
                    "Tr={tr} Pr={pr} {phase:?}: S/R {} vs H/RT − lnφ {want_s}",
                    d.s_dep_r
                );
            }
        }
    }

    #[test]
    fn root_selection_gives_a_dense_liquid_and_a_light_vapor_below_tc() {
        for k in [&SIMPLE, &REFERENCE] {
            let (tr, pr) = (0.8, 0.2);
            let vl = solve_vr(k, tr, pr, PhaseId::Liquid).unwrap();
            let vv = solve_vr(k, tr, pr, PhaseId::Vapor).unwrap();
            assert!(vl < 0.5, "liquid Vr {vl}");
            assert!(vv > 2.0, "vapor Vr {vv}");
            let zl = departure(k, tr, vl).z;
            let zv = departure(k, tr, vv).z;
            assert!(zl < 0.15 && zv > 0.8, "Zl={zl} Zv={zv}");
            // Both roots satisfy the equation of state.
            for v in [vl, vv] {
                assert!((pr * v / tr - z_and_dz(k, tr, v).0).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn above_the_critical_point_both_phases_return_the_same_root() {
        let a = lee_kesler_reduced(1.5, 2.0, 0.3, PhaseId::Liquid).unwrap();
        let b = lee_kesler_reduced(1.5, 2.0, 0.3, PhaseId::Vapor).unwrap();
        assert!((a.z - b.z).abs() < 1e-10);
        assert!((a.h_dep_rt - b.h_dep_rt).abs() < 1e-10);
    }

    #[test]
    fn analytic_dz_dvr_matches_finite_differences() {
        for k in [&SIMPLE, &REFERENCE] {
            for tr in [0.6, 1.0, 2.5] {
                for vr in [0.05, 0.3, 1.0, 10.0] {
                    let (_, dz) = z_and_dz(k, tr, vr);
                    let h = 1e-6 * vr;
                    let fd = (z_and_dz(k, tr, vr + h).0 - z_and_dz(k, tr, vr - h).0) / (2.0 * h);
                    assert!(
                        (dz - fd).abs() < 1e-6 * (1.0 + fd.abs()),
                        "Tr={tr} Vr={vr}: {dz} vs {fd}"
                    );
                }
            }
        }
    }

    // === Against the cubic EOS this crate already trusts ==================

    #[test]
    fn agrees_with_peng_robinson_for_a_near_simple_fluid_vapor() {
        // Methane (ω ≈ 0.01) is practically the simple fluid. At moderate
        // pressure the LK and PR vapor departures should agree to a couple of
        // percent — a coarse but independent check on the sign conventions and
        // the (H−H°)/(RTc) → (H−H°)/(RT) scaling.
        use crate::eos::{CubicEos, PhaseId};
        use crate::mixing::MixingRule;
        use crate::mixture::MixtureSpec;
        let comps = [methane()];
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &[],
            ge: None,
        };
        // (T, P, tolerance): the two models differ by 10–20 % on methane's
        // vapor enthalpy departure across this range (PR's B(T) is not
        // Pitzer's), so this is a sign-and-magnitude check, not a pin — the
        // pin is the thermodynamic-consistency test above.
        for (t, p, tol) in [
            (300.0, 300.0, 0.20),
            (250.0, 2000.0, 0.20),
            (400.0, 10_000.0, 0.20),
        ] {
            let lk = lee_kesler_departure(&comps[0], t, p, PhaseId::Vapor).unwrap();
            let pr_h =
                crate::energy::h_departure_rt_mix(&spec, t, p, &[1.0], PhaseId::Vapor).unwrap();
            let pr_z = crate::mixture::z_mix(&spec, t, p, &[1.0], PhaseId::Vapor).unwrap();
            assert!(
                (lk.h_dep_rt - pr_h).abs() < tol * pr_h.abs().max(0.02),
                "T={t} P={p}: LK H/RT {} vs PR {pr_h}",
                lk.h_dep_rt
            );
            assert!(
                (lk.z - pr_z).abs() < 0.02,
                "T={t} P={p}: LK Z {} vs PR {pr_z}",
                lk.z
            );
        }
    }

    #[test]
    fn heavy_liquid_enthalpy_departure_is_large_and_negative() {
        // n-decane liquid at 400 K, 500 kPa: ΔHvap ≈ 44 kJ/mol → (H−H°)/(RT)
        // around −13. Sanity band, not a pin.
        let d = lee_kesler_departure(&n_decane(), 400.0, 500.0, PhaseId::Liquid).unwrap();
        assert!(
            d.h_dep_rt < -10.0 && d.h_dep_rt > -16.0,
            "H/RT = {}",
            d.h_dep_rt
        );
        assert!(d.z < 0.1, "Z = {}", d.z);
    }

    #[test]
    fn reference_fluid_reproduces_itself() {
        // ω = ω_r must give the reference-fluid result exactly (w = 1).
        let d = lee_kesler_reduced(0.8, 0.3, OMEGA_REFERENCE, PhaseId::Liquid).unwrap();
        let vr = solve_vr(&REFERENCE, 0.8, 0.3, PhaseId::Liquid).unwrap();
        let r = departure(&REFERENCE, 0.8, vr);
        assert!((d.h_dep_rt - r.h_dep_rt).abs() < 1e-12);
    }

    // === Mixing rules =====================================================

    #[test]
    fn pseudocritical_of_a_pure_component_is_the_component() {
        let c = n_octane();
        let pc = lee_kesler_pseudocritical(std::slice::from_ref(&c), &[1.0], 1.0).unwrap();
        assert!((pc.tc - c.tc).abs() < 1e-9);
        assert!((pc.pc - c.pc).abs() < 1e-6, "{} vs {}", pc.pc, c.pc);
        assert!((pc.omega - c.omega).abs() < 1e-12);
    }

    #[test]
    fn pseudocritical_lies_between_the_pure_components_and_is_symmetric() {
        let comps = [methane(), n_decane()];
        for eta in [1.0, 0.25] {
            let a = lee_kesler_pseudocritical(&comps, &[0.3, 0.7], eta).unwrap();
            let b = lee_kesler_pseudocritical(&[n_decane(), methane()], &[0.7, 0.3], eta).unwrap();
            assert!((a.tc - b.tc).abs() < 1e-9 && (a.pc - b.pc).abs() < 1e-9);
            assert!(a.tc > comps[0].tc && a.tc < comps[1].tc, "Tc,m = {}", a.tc);
            assert!(a.pc > comps[1].pc && a.pc < comps[0].pc, "Pc,m = {}", a.pc);
        }
        // Un-normalised compositions are normalised, not rejected.
        let c = lee_kesler_pseudocritical(&comps, &[3.0, 7.0], 1.0).unwrap();
        let d = lee_kesler_pseudocritical(&comps, &[0.3, 0.7], 1.0).unwrap();
        assert!((c.tc - d.tc).abs() < 1e-9);
    }

    #[test]
    fn mixture_departure_runs_and_is_bounded_by_the_pure_ends() {
        let comps = [n_octane(), n_decane()];
        let m = lee_kesler_departure_mix(&comps, &[0.5, 0.5], 450.0, 300.0, PhaseId::Liquid, 0.25)
            .unwrap();
        let a = lee_kesler_departure(&comps[0], 450.0, 300.0, PhaseId::Liquid).unwrap();
        let b = lee_kesler_departure(&comps[1], 450.0, 300.0, PhaseId::Liquid).unwrap();
        let (lo, hi) = (a.h_dep_rt.min(b.h_dep_rt), a.h_dep_rt.max(b.h_dep_rt));
        assert!(
            m.h_dep_rt > lo - 0.5 && m.h_dep_rt < hi + 0.5,
            "{} not in [{lo}, {hi}]",
            m.h_dep_rt
        );
    }

    #[test]
    fn scales_to_hundreds_of_pseudocomponents() {
        // N = 300 must be routine: the double sum is the only O(N²) work.
        let comps: Vec<Component> = (0..300)
            .map(|i| Component {
                name: format!("PC-{i}"),
                tc: 400.0 + i as f64,
                pc: 3000.0 - 5.0 * i as f64,
                omega: 0.2 + 0.002 * i as f64,
                ..Component::default()
            })
            .collect();
        let x = vec![1.0 / 300.0; 300];
        let d = lee_kesler_departure_mix(&comps, &x, 600.0, 200.0, PhaseId::Vapor, 0.25).unwrap();
        assert!(d.h_dep_rt.is_finite() && d.z > 0.0);
    }

    // === Guard rails ======================================================

    #[test]
    fn rejects_bad_inputs() {
        assert!(lee_kesler_reduced(0.0, 1.0, 0.2, PhaseId::Vapor).is_err());
        assert!(lee_kesler_reduced(1.0, -1.0, 0.2, PhaseId::Vapor).is_err());
        assert!(lee_kesler_reduced(1.0, 1.0, f64::NAN, PhaseId::Vapor).is_err());
        assert!(lee_kesler_pseudocritical(&[], &[], 1.0).is_err());
        assert!(lee_kesler_pseudocritical(&[methane()], &[1.0, 0.0], 1.0).is_err());
        assert!(lee_kesler_pseudocritical(&[methane()], &[-1.0], 1.0).is_err());
        assert!(lee_kesler_departure(&Component::default(), 300.0, 100.0, PhaseId::Vapor).is_err());
    }
}
