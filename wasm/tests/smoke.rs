//! Smoke tests through the **real JS↔wasm boundary** (WEB_UI_PLAN.md
//! verification ladder, step 2) — the analog of the M16 Kotlin/JNA smoke
//! tests. Run with:
//!
//! ```sh
//! wasm-pack test --node wasm
//! ```
//!
//! `wasm-bindgen-test` compiles this file to wasm and executes it inside
//! Node.js, so every call here crosses the same boundary a browser app
//! uses: JsValue in, plain JS objects / thrown Errors out. The host-side
//! unit tests (`cargo test -p vle-wasm`) cover the logic; these five cover
//! the *plumbing*.

#![cfg(target_arch = "wasm32")]

use serde_json::json;
use vle_wasm::{VleSystem, db_component, steam_sat_p, version};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

/// Build a JsValue from a serde_json literal — the test-side stand-in for
/// a JS object literal.
fn js(v: serde_json::Value) -> JsValue {
    serde_wasm_bindgen::to_value(&v).unwrap()
}

/// Read one numeric field out of a returned plain JS object.
fn get_f64(obj: &JsValue, key: &str) -> f64 {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .unwrap()
        .as_f64()
        .unwrap_or_else(|| panic!("field {key} is not a number"))
}

// 1. Liveness: if version() returns, the Rust→wasm→JS pipeline is alive.
#[wasm_bindgen_test]
fn version_string_crosses_the_boundary() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}

// 2. Component DB: the embedded catalogue is readable and lands as a
//    plain object with camelCase keys.
#[wasm_bindgen_test]
fn water_lookup_returns_a_plain_object() {
    let water = db_component("water".into()).expect("water is bundled");
    let tc = get_f64(&water, "tc");
    assert!((tc - 647.0).abs() < 1.0, "tc = {tc}");
}

// 3. Steam tables: the kitchen benchmark — water boils at ~373.12 K at
//    1 atm with a latent heat of ~2256.5 kJ/kg.
#[wasm_bindgen_test]
fn boiling_at_one_atmosphere() {
    let row = steam_sat_p(101.325).unwrap();
    let t = get_f64(&row, "t");
    let h_fg = get_f64(&row, "hFg");
    assert!((t - 373.12).abs() < 0.05, "t_sat = {t}");
    assert!((h_fg - 2256.5).abs() < 1.0, "hFg = {h_fg}");
}

// 4. The Chapter IV Table 4.10 flash, end-to-end through JsValue inputs
//    (object-form models, no options) and a plain-object result.
#[wasm_bindgen_test]
fn chapter_iv_flash_through_the_boundary() {
    let sys = VleSystem::from_db(
        vec!["n-heptane".into(), "n-butane".into()],
        js(json!({"kind": "cubic", "eos": "RKS1972"})),
        js(json!("RKS1972")), // bare-string form on the liquid side
        None,                 // mixingRule defaults to "classical"
        JsValue::UNDEFINED,   // options omitted
    )
    .unwrap();
    assert_eq!(sys.n_components(), 2);
    let r = sys.flash_tp(300.0, 100.0, &[0.5, 0.5]).unwrap();
    let beta = get_f64(&r, "beta");
    // Thesis Table 4.10: β = 0.19889 (±5% band; bundled-DB constants).
    assert!((beta - 0.19889).abs() / 0.19889 < 0.05, "beta = {beta}");
    let two_phase = js_sys::Reflect::get(&r, &JsValue::from_str("twoPhase"))
        .unwrap()
        .as_bool()
        .unwrap();
    assert!(two_phase);
}

// 5. Error mapping: a Rust Err must arrive as a thrown JS Error carrying
//    the category prefix in its message.
#[wasm_bindgen_test]
fn errors_arrive_as_js_errors_with_prefixes() {
    let sys = VleSystem::from_db(
        vec!["n-heptane".into(), "n-butane".into()],
        js(json!("idealGas")),
        js(json!("ideal")),
        None,
        JsValue::UNDEFINED,
    )
    .unwrap();
    // Wrong-length feed → "invalid input: …".
    let err: JsValue = sys.flash_tp(300.0, 100.0, &[1.0]).unwrap_err().into();
    let msg = js_sys::Error::from(err).message().as_string().unwrap();
    assert!(msg.starts_with("invalid input:"), "message = {msg}");
    // Unknown DB name → "component not found …". (`match` instead of
    // `unwrap_err`: the Ok type, VleSystem, has no Debug impl.)
    let err: JsValue = match VleSystem::from_db(
        vec!["unobtainium".into()],
        js(json!("idealGas")),
        js(json!("ideal")),
        None,
        JsValue::UNDEFINED,
    ) {
        Err(e) => e.into(),
        Ok(_) => panic!("expected a NotFound error"),
    };
    let msg = js_sys::Error::from(err).message().as_string().unwrap();
    assert!(msg.starts_with("component not found"), "message = {msg}");
}
