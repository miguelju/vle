//! Integration tests for the runtime registry.
//!
//! Covers all 7 VLE quantities × multiple alternate units (round-trip),
//! gauge pressure with configurable P_atm, custom unit / dimension extension,
//! and the absolute-vs-difference temperature parity rule.

use approx::assert_relative_eq;
use vle_units::{Dimension, DimensionVector, RegistryError, UnitRegistry};

const EPS: f64 = 1e-9;

#[test]
fn temperature_round_trip() {
    let r = UnitRegistry::with_vle_defaults();
    // Round-trip a range of values per unit. For K and °R only positive
    // values are physical (absolute scales); °C and °F admit negatives.
    let cases: &[(&str, &[f64])] = &[
        ("K", &[1.0, 100.0, 273.15, 1000.0]),
        ("degC", &[-40.0, 0.0, 25.0, 100.0, 1000.0]),
        ("degF", &[-40.0, 0.0, 32.0, 212.0, 1832.0]),
        ("degR", &[1.0, 491.67, 1000.0]),
    ];
    for (unit, vals) in cases {
        for &v in *vals {
            let canonical = r.to_canonical(v, unit).unwrap();
            assert!(canonical > 0.0, "{unit}={v} → K must be > 0 (got {canonical})");
            let back = r.from_canonical(canonical, unit).unwrap();
            assert_relative_eq!(back, v, epsilon = EPS);
        }
    }
}

#[test]
fn temperature_known_values() {
    let r = UnitRegistry::with_vle_defaults();
    assert_relative_eq!(r.to_canonical(25.0, "degC").unwrap(), 298.15, epsilon = EPS);
    assert_relative_eq!(r.to_canonical(32.0, "degF").unwrap(), 273.15, epsilon = EPS);
    assert_relative_eq!(r.to_canonical(491.67, "degR").unwrap(), 273.15, epsilon = 1e-6);
    // -40 °C == -40 °F (the famous crossover)
    assert_relative_eq!(
        r.to_canonical(-40.0, "degC").unwrap(),
        r.to_canonical(-40.0, "degF").unwrap(),
        epsilon = EPS
    );
}

#[test]
fn pressure_round_trip_and_known_values() {
    let r = UnitRegistry::with_vle_defaults();
    for unit in ["Pa", "kPa", "MPa", "bar", "atm", "psi", "mmHg", "torr"] {
        for v in [0.5, 1.0, 101.325, 5000.0] {
            let canonical = r.to_canonical(v, unit).unwrap();
            let back = r.from_canonical(canonical, unit).unwrap();
            assert_relative_eq!(back, v, epsilon = EPS);
        }
    }
    // 1 atm == 101.325 kPa
    assert_relative_eq!(r.to_canonical(1.0, "atm").unwrap(), 101.325, epsilon = EPS);
    // 14.7 psi ≈ 101.353 kPa
    assert_relative_eq!(
        r.to_canonical(14.7, "psi").unwrap(),
        14.7 * 6.894_757_293_168_36,
        epsilon = EPS
    );
}

#[test]
fn gauge_pressure_uses_registry_p_atm() {
    let mut r = UnitRegistry::with_vle_defaults();
    // Default standard atmosphere
    assert_relative_eq!(r.atmospheric_pressure_kpa(), 101.325, epsilon = EPS);

    // 2.5 barg @ standard atm = 351.325 kPa absolute
    assert_relative_eq!(r.to_canonical(2.5, "barg").unwrap(), 351.325, epsilon = 1e-6);

    // Switch atmosphere (e.g. plant at altitude)
    r.set_atmospheric_pressure(84.5).unwrap();
    assert_relative_eq!(r.to_canonical(2.5, "barg").unwrap(), 334.5, epsilon = 1e-6);

    // psig at altered atmosphere
    let p = r.to_canonical(10.0, "psig").unwrap();
    assert_relative_eq!(p, 10.0 * 6.894_757_293_168_36 + 84.5, epsilon = 1e-6);

    // kPag is just gauge + atm
    assert_relative_eq!(r.to_canonical(50.0, "kPag").unwrap(), 134.5, epsilon = 1e-6);
}

#[test]
fn gauge_rejects_non_positive_absolute() {
    let r = UnitRegistry::with_vle_defaults();
    // -110 kPag would be -8.675 kPa absolute → negative, rejected
    let err = r.to_canonical(-110.0, "kPag").unwrap_err();
    assert!(matches!(err, RegistryError::NonPositivePressure(_)));
    // Mild vacuum is allowed: -50 kPag = +51.325 kPa abs
    assert_relative_eq!(r.to_canonical(-50.0, "kPag").unwrap(), 51.325, epsilon = 1e-6);
}

#[test]
fn gauge_round_trip() {
    let r = UnitRegistry::with_vle_defaults();
    for unit in ["barg", "psig", "kPag"] {
        for v in [0.0, 1.0, 50.0, 200.0] {
            let canonical = r.to_canonical(v, unit).unwrap();
            let back = r.from_canonical(canonical, unit).unwrap();
            assert_relative_eq!(back, v, epsilon = 1e-9);
        }
    }
}

#[test]
fn molar_energy_round_trip() {
    let r = UnitRegistry::with_vle_defaults();
    for unit in ["kJ/kmol", "J/mol", "J/kmol", "kJ/mol", "cal/mol", "kcal/kmol", "BTU/lbmol"] {
        for v in [1.0, 1000.0, -250.5] {
            let canonical = r.to_canonical(v, unit).unwrap();
            let back = r.from_canonical(canonical, unit).unwrap();
            assert_relative_eq!(back, v, epsilon = 1e-9);
        }
    }
    // 1 cal/mol = 4.184 J/mol = 4.184 kJ/kmol
    assert_relative_eq!(r.to_canonical(1.0, "cal/mol").unwrap(), 4.184, epsilon = EPS);
}

#[test]
fn molar_entropy_and_volume_and_amount() {
    let r = UnitRegistry::with_vle_defaults();
    // Entropy: 1 cal/(mol·K) = 4.184 kJ/(kmol·K)
    assert_relative_eq!(
        r.to_canonical(1.0, "cal/(mol*K)").unwrap(),
        4.184,
        epsilon = EPS
    );
    // Volume: 1 L/mol = 1000 cm³/mol; 1 m³/kmol = 1 cm³/mol
    assert_relative_eq!(r.to_canonical(1.0, "L/mol").unwrap(), 1000.0, epsilon = EPS);
    assert_relative_eq!(r.to_canonical(1.0, "m^3/kmol").unwrap(), 1.0, epsilon = EPS);
    // Amount: 1 mol = 1e-3 kmol; 1 lbmol = 0.45359237 kmol
    assert_relative_eq!(r.to_canonical(1.0, "mol").unwrap(), 1e-3, epsilon = EPS);
    assert_relative_eq!(
        r.to_canonical(1.0, "lbmol").unwrap(),
        0.453_592_37,
        epsilon = EPS
    );
}

#[test]
fn temperature_diff_is_scale_only() {
    // Δ°C → ΔK is identity (both are 1 K per 1 Δ°C). Crucially: NO offset.
    let r = UnitRegistry::with_vle_defaults();
    assert_relative_eq!(r.to_canonical(60.0, "delta_degC").unwrap(), 60.0, epsilon = EPS);
    // Δ°F → ΔK is × 5/9 (no offset). 60 Δ°C = 108 Δ°F.
    assert_relative_eq!(
        r.to_canonical(108.0, "delta_degF").unwrap(),
        60.0,
        epsilon = 1e-9
    );
    // Contrast: absolute 60 °C = 333.15 K
    assert_relative_eq!(r.to_canonical(60.0, "degC").unwrap(), 333.15, epsilon = EPS);
}

#[test]
fn parse_string_form() {
    let r = UnitRegistry::with_vle_defaults();
    let q = r.parse("25 degC").unwrap();
    assert_eq!(q.dimension, "temperature");
    assert_relative_eq!(q.canonical, 298.15, epsilon = EPS);

    let q = r.parse("3.5 barg").unwrap();
    assert_eq!(q.dimension, "pressure");
    assert_relative_eq!(q.canonical, 451.325, epsilon = 1e-6);

    let formatted = r.format(&q, "kPa").unwrap();
    assert!(formatted.starts_with("451.325"));
}

#[test]
fn custom_unit_within_existing_dimension() {
    // mmH2O within Pressure
    let mut r = UnitRegistry::with_vle_defaults();
    r.define("mmH2O", Dimension::Pressure, 0.009_806_65, 0.0).unwrap();

    let canonical = r.to_canonical(1000.0, "mmH2O").unwrap();
    assert_relative_eq!(canonical, 9.806_65, epsilon = 1e-9);

    let back = r.from_canonical(canonical, "mmH2O").unwrap();
    assert_relative_eq!(back, 1000.0, epsilon = 1e-9);
}

#[test]
fn custom_gauge_unit_tracks_p_atm() {
    let mut r = UnitRegistry::with_vle_defaults();
    r.define_gauge("mmH2Og", 0.009_806_65).unwrap();
    // 500 mmH2Og at standard atm = 4.903325 + 101.325 = 106.228325 kPa
    let p = r.to_canonical(500.0, "mmH2Og").unwrap();
    assert_relative_eq!(p, 500.0 * 0.009_806_65 + 101.325, epsilon = 1e-9);
    // Change atmosphere
    r.set_atmospheric_pressure(90.0).unwrap();
    let p2 = r.to_canonical(500.0, "mmH2Og").unwrap();
    assert_relative_eq!(p2, 500.0 * 0.009_806_65 + 90.0, epsilon = 1e-9);
}

#[test]
fn custom_dimension_round_trip() {
    let mut r = UnitRegistry::with_vle_defaults();
    // Heat transfer coefficient: W/(m²·K) = kg·s⁻³·K⁻¹
    r.define_dimension(
        "heat_transfer_coefficient",
        DimensionVector::new([0, 1, -3, 0, -1, 0, 0]),
    )
    .unwrap();

    r.define_with_dimension("W_per_m2K", "heat_transfer_coefficient", 1.0, 0.0)
        .unwrap();
    r.define_with_dimension("BTU_per_hr_ft2_R", "heat_transfer_coefficient", 5.678_263, 0.0)
        .unwrap();

    let v = r.to_canonical(26.4, "BTU_per_hr_ft2_R").unwrap();
    assert_relative_eq!(v, 26.4 * 5.678_263, epsilon = 1e-9);
    let back = r.from_canonical(v, "BTU_per_hr_ft2_R").unwrap();
    assert_relative_eq!(back, 26.4, epsilon = 1e-9);
}

#[test]
fn redefining_unit_errors() {
    let mut r = UnitRegistry::with_vle_defaults();
    let err = r.define("kPa", Dimension::Pressure, 1.0, 0.0).unwrap_err();
    assert!(matches!(err, RegistryError::AlreadyDefined(_)));
}

#[test]
fn unknown_unit_errors() {
    let r = UnitRegistry::with_vle_defaults();
    let err = r.to_canonical(1.0, "foo").unwrap_err();
    assert!(matches!(err, RegistryError::UnknownUnit(_)));
}

#[test]
fn bad_atmospheric_pressure_rejected() {
    let mut r = UnitRegistry::with_vle_defaults();
    assert!(r.set_atmospheric_pressure(0.0).is_err());
    assert!(r.set_atmospheric_pressure(-5.0).is_err());
}

#[test]
fn toml_loader() {
    let mut r = UnitRegistry::with_vle_defaults();
    let toml_text = r#"
        [[dimension]]
        name = "heat_transfer_coefficient"
        exponents = [0, 1, -3, 0, -1, 0, 0]

        [[unit]]
        name = "mmH2O"
        dimension = "pressure"
        scale = 0.00980665

        [[unit]]
        name = "atg"
        dimension = "pressure"
        scale = 101.325
        gauge = true

        [[unit]]
        name = "W_per_m2K"
        dimension = "heat_transfer_coefficient"
        scale = 1.0
    "#;
    r.load_from_toml_str(toml_text).unwrap();

    assert_relative_eq!(
        r.to_canonical(1.0, "mmH2O").unwrap(),
        0.009_806_65,
        epsilon = 1e-12
    );
    // 1 atg = 101.325 + 101.325 = 202.65 kPa absolute
    assert_relative_eq!(r.to_canonical(1.0, "atg").unwrap(), 202.65, epsilon = 1e-9);
    assert_relative_eq!(r.to_canonical(50.0, "W_per_m2K").unwrap(), 50.0, epsilon = EPS);
}

#[test]
fn r_constant_round_trip() {
    // Legacy R = 8.31451 kJ/(kmol·K) must round-trip unchanged.
    let r = UnitRegistry::with_vle_defaults();
    let canonical = r.to_canonical(8.31451, "kJ/(kmol*K)").unwrap();
    assert_relative_eq!(canonical, 8.31451, epsilon = EPS);
    let back = r.from_canonical(canonical, "J/(mol*K)").unwrap();
    assert_relative_eq!(back, 8.31451, epsilon = EPS);
}
