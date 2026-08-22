//! Bundled component database — Rust-side pure-component property lookup (M12.2).
//!
//! This is the Rust mirror of the Python loader in
//! `python/src/vle/components.py`. Where the wheel reads
//! `vle/data/components.json` through `importlib.resources`, the crate embeds
//! the **same** JSON at compile time via [`include_str!`] and parses it lazily
//! on first use. Downstream Rust consumers (e.g. a column solver) can then turn
//! a component *name* into the [`Component`] the engine needs without
//! hand-building the struct — closing gap **G3** of `DERIVATIVE_RELEASE_PLAN.md`.
//!
//! Feature-gated behind **`component-db`** (off by default) so the core crate
//! stays dependency-free; the `python` feature turns it on so the wheel can
//! expose the lookups (`db_component`, `db_available`) for testing the Rust DB
//! through Python.
//!
//! # Units
//!
//! Every field is in the canonical engine units (see `CLAUDE.md`): `tc` in
//! **K**, `pc` in **kPa** (absolute), `vc` in **cm³/mol**, `tb` in **K**, `mw`
//! in **g/mol**, `liquid_volume` in **cm³/mol**, `omega`/`zc` dimensionless,
//! `psat_coeffs` the reduced-Antoine list `ln(Psat/Pc) = a₁ − a₂/(a₃ + T)` (T in
//! **K**, P in **kPa**), and `cp_coeffs` the dimensionless ideal-gas Cp°/R
//! polynomial `Cp°(T)/R = Σₖ aₖ·Tᵏ` (T in **K**).
//!
//! # Fields the JSON does not carry
//!
//! The bundled catalog stores only the constants the engine's EOS / activity /
//! energy paths read. The mapped [`Component`] therefore leaves the following at
//! their [`Component::default`] (`0.0`) values: `dipole_moment`,
//! `solubility_param`, `zra`, `omega_srk`, `m_polar`, `n_polar`, `g_polar`, and
//! `prsv_k1`. `sat_model` is set to [`SatPressureModel::Antoine`] to match the
//! shipped `psat_coeffs`. A caller needing the polar/PRSV parameters must fill
//! them in explicitly (they are not part of the bundled data set).
//!
//! Property data provenance: Poling, Prausnitz & O'Connell, 5th ed. (30) — see
//! `scripts/build_components_json.py`, the single generator for all three JSON
//! copies (engine, wheel, notebooks).

use crate::saturation::SatPressureModel;
use crate::types::Component;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// The canonical JSON catalog, embedded into the binary at compile time.
///
/// `include_str!` resolves relative to *this source file*, so the path reaches
/// `engine/data/components.json` — the copy that lives inside the crate
/// directory precisely so `cargo package` ships it to crates.io.
const COMPONENTS_JSON: &str = include_str!("../data/components.json");

/// One raw JSON record. This is a data-transfer object (DTO) that mirrors the
/// JSON key names exactly; it exists only so `serde` can deserialize into it,
/// after which [`to_component`] maps it onto the engine's [`Component`].
///
/// `#[serde(default)]` on the whole struct means any key the JSON omits for a
/// given compound (e.g. `liquid_volume` for CO₂) falls back to the field's
/// `Default` rather than erroring — matching the Python loader, which likewise
/// ignores missing optional keys.
#[derive(Deserialize)]
#[serde(default)]
struct RawComponent {
    formula: String,
    cas: String,
    mw: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    zc: f64,
    vc: f64,
    tb: f64,
    psat_coeffs: Vec<f64>,
    cp_coeffs: Vec<f64>,
    liquid_volume: f64,
}

// `Default` lets `#[serde(default)]` fill absent fields. Deriving it is enough
// (all fields are `Default`), but we spell out that `formula`/`cas` default to
// empty strings and every number to `0.0`.
impl Default for RawComponent {
    fn default() -> Self {
        Self {
            formula: String::new(),
            cas: String::new(),
            mw: 0.0,
            tc: 0.0,
            pc: 0.0,
            omega: 0.0,
            zc: 0.0,
            vc: 0.0,
            tb: 0.0,
            psat_coeffs: Vec::new(),
            cp_coeffs: Vec::new(),
            liquid_volume: 0.0,
        }
    }
}

/// Top-level shape of `components.json`: a `_meta` block (ignored here) plus the
/// `compounds` map keyed by canonical name. `serde` drops the unlisted `_meta`
/// key automatically, so we only declare the field we consume.
#[derive(Deserialize)]
struct RawDb {
    compounds: HashMap<String, RawComponent>,
}

/// Map one JSON record onto the engine [`Component`]. `name` is the JSON map key
/// (the canonical, original-case name).
fn to_component(name: &str, raw: &RawComponent) -> Component {
    // The JSON `cp_coeffs` is a 5-entry list; the engine stores a fixed-size
    // `[f64; 5]`. Copy element-by-element so a malformed list can't panic — a
    // short list leaves the tail at 0.0, a long one is truncated. The generated
    // data always has exactly 5, so in practice this is a straight copy.
    let mut cp_coeffs = [0.0f64; 5];
    for (slot, &v) in cp_coeffs.iter_mut().zip(raw.cp_coeffs.iter()) {
        *slot = v;
    }

    Component {
        name: name.to_string(),
        tc: raw.tc,
        pc: raw.pc,
        vc: raw.vc,
        zc: raw.zc,
        omega: raw.omega,
        tb: raw.tb,
        mw: raw.mw,
        cp_coeffs,
        psat_coeffs: raw.psat_coeffs.clone(),
        liquid_volume: raw.liquid_volume,
        // The bundled catalog ships Antoine `psat_coeffs`, so the saturation
        // model is Antoine. Everything not listed here stays at Default (0.0) —
        // see the module doc comment for the exhaustive list.
        sat_model: SatPressureModel::Antoine,
        ..Component::default()
    }
}

/// Parse + cache the catalog on first use.
///
/// `OnceLock` gives a thread-safe, allocate-once cache with no `unsafe` and no
/// external `lazy_static`/`once_cell` dependency: the first caller parses the
/// JSON and every later caller reuses the same `HashMap`. Keys are the
/// **lower-cased** names, so lookup is case-insensitive; each `Component` keeps
/// its original-case name in `Component::name`.
fn db() -> &'static HashMap<String, Component> {
    static DB: OnceLock<HashMap<String, Component>> = OnceLock::new();
    DB.get_or_init(|| {
        // The embedded JSON is generated + committed, so a parse failure is a
        // build-data bug, not a runtime input error: `expect` turns it into a
        // clear panic instead of silently returning an empty database.
        let raw: RawDb =
            serde_json::from_str(COMPONENTS_JSON).expect("bundled components.json must parse");
        raw.compounds
            .iter()
            .map(|(name, rc)| (name.to_lowercase(), to_component(name, rc)))
            .collect()
    })
}

/// Look up a bundled component by name (case-insensitive).
///
/// Name normalization mirrors `vle.components.get` exactly — leading/trailing
/// whitespace is trimmed and the name is lower-cased before lookup — so the
/// Rust and Python databases accept the same strings. There are no extra
/// aliases: `component("Water")`, `component(" water ")`, and
/// `component("water")` all resolve, but common shorthands the Python loader
/// does not accept (e.g. `"H2O"`) are misses here too.
///
/// Returns a cloned [`Component`] in canonical engine units (see the module
/// docs), or `None` if no component with that name is bundled.
pub fn component(name: &str) -> Option<Component> {
    db().get(&name.trim().to_lowercase()).cloned()
}

/// Names of all bundled components, sorted (original casing preserved).
///
/// Matches `vle.components.available()`: the canonical names as they appear in
/// the JSON, alphabetically ordered.
pub fn available() -> Vec<String> {
    let mut names: Vec<String> = db().values().map(|c| c.name.clone()).collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_25_compounds_parse() {
        // The M12.1 database held 24 compounds; M14 added ammonia (→ 25);
        // every one must map cleanly.
        assert_eq!(available().len(), 25);
    }

    #[test]
    fn lookup_hit_case_and_whitespace_insensitive() {
        // Same record regardless of casing or surrounding whitespace.
        let a = component("water").unwrap();
        let b = component("Water").unwrap();
        let c = component("  WATER  ").unwrap();
        assert_eq!(a.name, "water");
        assert_eq!(a.tc, b.tc);
        assert_eq!(a.tc, c.tc);
    }

    #[test]
    fn lookup_miss_returns_none() {
        // Unknown name and a shorthand the Python loader also rejects.
        assert!(component("unobtainium").is_none());
        assert!(component("H2O").is_none());
    }

    #[test]
    fn spot_check_legacy_compound_vs_json_literals() {
        // benzene — one of the original 15. Values are the literals in
        // engine/data/components.json (Poling 5th ed. (30) via the generator).
        let benzene = component("benzene").unwrap();
        assert_eq!(benzene.tc, 562.02);
        assert_eq!(benzene.pc, 4907.277);
        assert_eq!(benzene.omega, 0.211);
    }

    #[test]
    fn spot_check_new_compound_vs_json_literals() {
        // toluene — added in M12.1, the McCabe–Thiele teaching binary partner.
        let toluene = component("toluene").unwrap();
        assert_eq!(toluene.tc, 591.75);
        assert_eq!(toluene.pc, 4126.3);
        assert_eq!(toluene.omega, 0.2657);
    }

    #[test]
    fn spot_check_ammonia_vs_json_literals() {
        // ammonia — added in M14 for the NH₃–H₂O method. Values are the
        // literals in engine/data/components.json (thermo 0.6.0 / chemicals
        // 1.5.2 via the generator).
        let nh3 = component("ammonia").unwrap();
        assert_eq!(nh3.tc, 405.56);
        assert_eq!(nh3.pc, 11363.4);
        assert_eq!(nh3.omega, 0.256);
        assert_eq!(nh3.mw, 17.03052);
        assert_eq!(nh3.psat_coeffs, vec![5.595032, 2132.497737, -32.98]);
        assert_eq!(nh3.liquid_volume, 28.24);
        assert_eq!(nh3.sat_model, SatPressureModel::Antoine);
        assert!(nh3.cp_coeffs[0] != 0.0);
    }

    #[test]
    fn mapped_fields_are_populated_and_defaults_left_zero() {
        let toluene = component("toluene").unwrap();
        // Populated from JSON.
        assert_eq!(toluene.psat_coeffs, vec![5.606494, 3056.958021, -55.525]);
        assert_eq!(toluene.cp_coeffs.len(), 5);
        assert!(toluene.cp_coeffs[0] != 0.0);
        assert_eq!(toluene.sat_model, SatPressureModel::Antoine);
        // Not carried by the JSON — left at Component::default() (0.0).
        assert_eq!(toluene.dipole_moment, 0.0);
        assert_eq!(toluene.zra, 0.0);
        assert_eq!(toluene.prsv_k1, 0.0);
    }

    #[test]
    fn available_is_sorted() {
        let names = available();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
