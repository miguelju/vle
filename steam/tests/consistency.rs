//! Thermodynamic-consistency tests through the public `vle-steam` API.
//!
//! These need no external data — they assert internal identities that any
//! correct thermodynamic surface must satisfy, so they catch cross-region and
//! cross-property mistakes the single-region verification tables cannot.

use approx::assert_relative_eq;
use vle_steam::{SteamState, latent_heat, psat, psat_derivative, sat_t, tsat};

/// `h = u + p·v` must hold at every single-phase point (`p` in kPa, `v` in
/// m³/kg ⇒ `p·v` in kJ/kg).
#[test]
fn h_equals_u_plus_pv_all_regions() {
    let points = [
        (300.0, 3_000.0),   // region 1 (liquid)
        (500.0, 3_000.0),   // region 1
        (700.0, 3.5),       // region 2 (low-P vapor)
        (700.0, 30_000.0),  // supercritical / region 2 near B23
        (650.0, 25_000.0),  // region 3 (near-critical)
        (1500.0, 500.0),    // region 5 (high-T)
        (1500.0, 30_000.0), // region 5
    ];
    for (t, p) in points {
        let st = SteamState::tp(t, p).unwrap();
        assert_relative_eq!(st.h, st.u + p * st.v, max_relative = 1e-9);
    }
}

/// Clausius–Clapeyron: `h_fg = Tsat · v_fg · dPsat/dT` (with `dPsat/dT` the
/// analytic region-4 derivative). Ties region 4 to regions 1/2 and checks the
/// latent-heat/volume/slope triangle for mutual consistency.
#[test]
fn clausius_clapeyron_holds() {
    for &t in &[300.0, 350.0, 450.0, 550.0, 600.0] {
        let sat = sat_t(t).unwrap();
        let v_fg = sat.v_g - sat.v_f;
        let dpdt = psat_derivative(t).unwrap(); // kPa/K
        let cc = t * v_fg * dpdt; // K · m³/kg · kPa/K = kJ/kg
        assert_relative_eq!(sat.h_fg, cc, max_relative = 2e-3);
    }
}

/// `ph(P, h(tp(T,P))) → T` and `ps(P, s(tp(T,P))) → T` round-trips for
/// single-phase states across regions.
#[test]
fn ph_ps_round_trips() {
    let points = [
        (320.0, 5_000.0),  // liquid (region 1)
        (500.0, 10_000.0), // liquid (region 1)
        (800.0, 1_000.0),  // vapor (region 2)
        (600.0, 500.0),    // vapor (region 2)
        (1500.0, 2_000.0), // high-T (region 5)
    ];
    for (t, p) in points {
        let st = SteamState::tp(t, p).unwrap();
        let back_h = SteamState::ph(p, st.h).unwrap();
        assert_relative_eq!(back_h.t, t, max_relative = 1e-6);
        let back_s = SteamState::ps(p, st.s).unwrap();
        assert_relative_eq!(back_s.t, t, max_relative = 1e-6);
    }
}

/// `psat`/`tsat` invert each other, and `latent_heat` matches the sat row.
#[test]
fn saturation_inverse_and_latent_heat() {
    for &t in &[300.0, 400.0, 500.0, 600.0] {
        let p = psat(t).unwrap();
        assert_relative_eq!(tsat(p).unwrap(), t, max_relative = 1e-9);
        assert_relative_eq!(
            latent_heat(t).unwrap(),
            sat_t(t).unwrap().h_fg,
            max_relative = 1e-12
        );
    }
}

/// A two-phase PH flash recovers the quality used to build the state.
#[test]
fn two_phase_quality_round_trip() {
    // 10 bar (1000 kPa) wet steam at x = 0.7.
    let st = SteamState::px(1_000.0, 0.7).unwrap();
    let back = SteamState::ph(1_000.0, st.h).unwrap();
    assert_relative_eq!(back.x.unwrap(), 0.7, max_relative = 1e-9);
    assert_relative_eq!(back.t, st.t, max_relative = 1e-12);
}
