//! Peneloux volume translation — a consistent liquid-density fix for cubic EOS.
//!
//! # The problem
//!
//! Two-parameter cubic EOS (SRK, PR) fix the critical compressibility at
//! `Zc = 1/3` (SRK) or `0.307` (PR), while real hydrocarbons sit at 0.25–0.29.
//! The consequence is that the *liquid* molar volume they predict is too large
//! — by ~10–20 % for SRK on a heavy cut, less for PR — and heavy-cut density is
//! exactly what a refinery column needs right for tray hydraulics and product
//! gravities.
//!
//! # The fix
//!
//! Péneloux, Rauzy & Fréze (1982) noticed that shifting every volume by a
//! **constant** `c` per component,
//!
//! ```text
//!   V_corrected = V_EOS − Σᵢ xᵢ·cᵢ
//! ```
//!
//! leaves the phase-equilibrium calculation **untouched** — the shift changes
//! every component's fugacity in both phases by the same factor
//! `exp(−cᵢP/RT)`, which cancels in `Kᵢ = φ̂ᵢᴸ/φ̂ᵢⱽ` — while fixing the liquid
//! density where it matters. So the whole flash layer of this crate is
//! unaffected; only volume-derived quantities (density, Poynting factors built
//! from EOS volumes) change. Their correlation for `cᵢ` from the Rackett
//! compressibility `Z_RA`:
//!
//! ```text
//!   SRK:  cᵢ = 0.40768·(R·Tc,ᵢ/Pc,ᵢ)·(0.29441 − Z_RA,ᵢ)
//!   PR:   cᵢ = 0.50033·(R·Tc,ᵢ/Pc,ᵢ)·(0.25969 − Z_RA,ᵢ)
//!   Z_RA,ᵢ = 0.29056 − 0.08775·ωᵢ    (if the component carries no measured Z_RA)
//! ```
//!
//! The PR constants are the standard adaptation of Peneloux's SRK form to the
//! PR critical compressibility (Pedersen & Christensen, *Phase Behavior of
//! Petroleum Reservoir Fluids*, eq. 4.42), and are the ones refinery packages
//! ship. The correction is exact at the saturated-liquid volume the Rackett
//! equation gives near `Tr = 0.7`, and degrades towards the critical point,
//! where no constant shift can help.
//!
//! # References
//! - (44) Péneloux, A.; Rauzy, E.; Fréze, R. *Fluid Phase Equilib.* **1982**,
//!   *8*, 7–23.

use super::RefineryError;
use crate::eos::{CubicEos, PhaseId, family_constants};
use crate::mixture::{MixtureSpec, z_mix};
use crate::types::{Component, R_GAS};

/// Rackett compressibility used by the shift: the component's `zra` if it has
/// one, else the acentric-factor correlation.
fn rackett_zra(comp: &Component) -> f64 {
    if comp.zra > 0.0 {
        comp.zra
    } else {
        0.29056 - 0.08775 * comp.omega
    }
}

/// Peneloux volume shift `cᵢ` for one component under `eos`.
///
/// # Arguments
/// * `eos` — a member of the SRK family (`RKS1972` & co.) or the PR family
///   (`PR1976` & co.), as classified by [`family_constants`].
/// * `comp` — reads `tc` (**K**), `pc` (**kPa**), `omega`, and `zra` if set.
///
/// # Returns
/// `cᵢ` in **cm³/mol** — subtract it from the EOS molar volume.
///
/// # Errors
/// [`RefineryError::Unsupported`] for a van der Waals-family or
/// three-parameter EOS (which have their own Zc handling);
/// [`RefineryError::InvalidInput`] if `Tc`/`Pc` are not positive.
pub fn peneloux_shift(eos: CubicEos, comp: &Component) -> Result<f64, RefineryError> {
    if !(comp.tc > 0.0 && comp.pc > 0.0) {
        return Err(RefineryError::InvalidInput(format!(
            "component '{}' needs Tc > 0 and Pc > 0 (got {}, {})",
            comp.name, comp.tc, comp.pc
        )));
    }
    let fc = family_constants(eos);
    // (coefficient, reference Zc) per family; k1 identifies the family the
    // same way `family_constants` does.
    let (k, z_ref) = if eos.is_three_parameter() {
        return Err(RefineryError::Unsupported(format!(
            "Peneloux shift is not defined for the three-parameter EOS {eos:?}"
        )));
    } else if fc.k1 == 1.0 {
        (0.40768, 0.29441) // SRK family
    } else if fc.k1 == 2.0 {
        (0.50033, 0.25969) // PR family
    } else {
        return Err(RefineryError::Unsupported(format!(
            "Peneloux shift is not defined for the van der Waals-family EOS {eos:?}"
        )));
    };
    // R·Tc/Pc in cm³/mol: R [kJ/(kmol·K)] · Tc [K] / Pc [kPa] = m³/kmol = 1000 cm³/mol.
    let rtc_pc = 1000.0 * R_GAS * comp.tc / comp.pc;
    Ok(k * rtc_pc * (z_ref - rackett_zra(comp)))
}

/// Mixture shift `Σᵢ xᵢ·cᵢ` in **cm³/mol**.
///
/// # Errors
/// As [`peneloux_shift`]; additionally [`RefineryError::InvalidInput`] on a
/// length mismatch.
pub fn peneloux_shift_mix(
    eos: CubicEos,
    components: &[Component],
    x: &[f64],
) -> Result<f64, RefineryError> {
    if components.len() != x.len() {
        return Err(RefineryError::InvalidInput(format!(
            "components={}, x={}",
            components.len(),
            x.len()
        )));
    }
    let mut c = 0.0;
    for (comp, &xi) in components.iter().zip(x) {
        if xi != 0.0 {
            c += xi * peneloux_shift(eos, comp)?;
        }
    }
    Ok(c)
}

/// Volume-translated molar volume of a phase, **cm³/mol**:
/// `V = 1000·Z_EOS·R·T/P − Σᵢ xᵢ·cᵢ`.
///
/// `t` in **K**, `p` in **kPa absolute**, `x` mole fractions summing to 1.
///
/// # Errors
/// [`RefineryError::Unsupported`] if `spec.eos` has no Peneloux shift;
/// [`RefineryError::InvalidInput`] if the mixture layer rejects the state.
pub fn translated_molar_volume(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<f64, RefineryError> {
    let c = peneloux_shift_mix(spec.eos, spec.components, x)?;
    let z = z_mix(spec, t, p, x, phase).map_err(|e| RefineryError::InvalidInput(e.to_string()))?;
    Ok(1000.0 * z * R_GAS * t / p - c)
}

/// Volume-translated **mass** density of a phase, **kg/m³**:
/// `ρ = Σᵢ xᵢ·Mᵢ / V` with `V` from [`translated_molar_volume`].
///
/// Reads `Component::mw` (**g/mol**). `t` in **K**, `p` in **kPa absolute**.
///
/// # Errors
/// As [`translated_molar_volume`]; [`RefineryError::InvalidInput`] if any
/// component with `xᵢ > 0` has no molecular weight.
pub fn translated_liquid_density(
    spec: &MixtureSpec,
    t: f64,
    p: f64,
    x: &[f64],
    phase: PhaseId,
) -> Result<f64, RefineryError> {
    let v = translated_molar_volume(spec, t, p, x, phase)?;
    let mut mw = 0.0;
    for (comp, &xi) in spec.components.iter().zip(x) {
        if xi > 0.0 && comp.mw <= 0.0 {
            return Err(RefineryError::InvalidInput(format!(
                "component '{}' has no molecular weight",
                comp.name
            )));
        }
        mw += xi * comp.mw;
    }
    // g/mol ÷ cm³/mol = g/cm³ = 1000 kg/m³.
    Ok(1000.0 * mw / v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mixing::MixingRule;

    fn n_decane() -> Component {
        Component {
            name: "n-decane".into(),
            tc: 617.7,
            pc: 2110.0,
            omega: 0.492,
            mw: 142.28,
            ..Component::default()
        }
    }
    fn n_heptane() -> Component {
        Component {
            name: "n-heptane".into(),
            tc: 540.2,
            pc: 2740.0,
            omega: 0.350,
            mw: 100.20,
            ..Component::default()
        }
    }

    #[test]
    fn shift_is_positive_and_of_the_expected_size_for_srk() {
        // Peneloux quotes c ≈ 10–20 cm³/mol for C7–C10 under SRK.
        let c = peneloux_shift(CubicEos::RKS1972, &n_heptane()).unwrap();
        assert!(c > 5.0 && c < 25.0, "c = {c} cm³/mol");
        let c10 = peneloux_shift(CubicEos::RKS1972, &n_decane()).unwrap();
        assert!(c10 > c, "shift should grow with size: {c10} vs {c}");
    }

    #[test]
    fn a_measured_zra_overrides_the_correlation() {
        let mut c = n_heptane();
        let base = peneloux_shift(CubicEos::PR1976, &c).unwrap();
        c.zra = 0.26;
        let with = peneloux_shift(CubicEos::PR1976, &c).unwrap();
        assert!(with != base);
        // and matches the formula outright
        let want = 0.50033 * 1000.0 * R_GAS * c.tc / c.pc * (0.25969 - 0.26);
        assert!((with - want).abs() < 1e-9);
    }

    #[test]
    fn translated_srk_liquid_density_of_n_heptane_is_close_to_measured() {
        // n-heptane at 298 K: ρ ≈ 680 kg/m³. Untranslated SRK is ~10 % light;
        // translated must land within 3 %.
        let comps = [n_heptane()];
        let spec = MixtureSpec {
            eos: CubicEos::RKS1972,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &[],
            ge: None,
        };
        let rho =
            translated_liquid_density(&spec, 298.15, 101.325, &[1.0], PhaseId::Liquid).unwrap();
        assert!((rho - 680.0).abs() / 680.0 < 0.03, "ρ = {rho} kg/m³");
        // and it is denser than the raw EOS
        let z = z_mix(&spec, 298.15, 101.325, &[1.0], PhaseId::Liquid).unwrap();
        let rho_raw = 1000.0 * 100.20 / (1000.0 * z * R_GAS * 298.15 / 101.325);
        assert!(rho > rho_raw, "{rho} vs raw {rho_raw}");
    }

    #[test]
    fn translated_pr_liquid_density_of_n_decane_is_close_to_measured() {
        // n-decane at 298 K: ρ ≈ 727 kg/m³.
        let comps = [n_decane()];
        let spec = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &[],
            ge: None,
        };
        let rho =
            translated_liquid_density(&spec, 298.15, 101.325, &[1.0], PhaseId::Liquid).unwrap();
        assert!((rho - 727.0).abs() / 727.0 < 0.03, "ρ = {rho} kg/m³");
    }

    #[test]
    fn mixture_shift_is_the_mole_fraction_average() {
        let comps = [n_heptane(), n_decane()];
        let a = peneloux_shift(CubicEos::PR1976, &comps[0]).unwrap();
        let b = peneloux_shift(CubicEos::PR1976, &comps[1]).unwrap();
        let m = peneloux_shift_mix(CubicEos::PR1976, &comps, &[0.25, 0.75]).unwrap();
        assert!((m - (0.25 * a + 0.75 * b)).abs() < 1e-12);
    }

    #[test]
    fn unsupported_families_are_reported() {
        assert!(peneloux_shift(CubicEos::VdW1870, &n_heptane()).is_err());
        assert!(peneloux_shift(CubicEos::PatelTeja, &n_heptane()).is_err());
        assert!(peneloux_shift(CubicEos::PR1976, &Component::default()).is_err());
        assert!(peneloux_shift_mix(CubicEos::PR1976, &[n_heptane()], &[0.5, 0.5]).is_err());
    }
}
