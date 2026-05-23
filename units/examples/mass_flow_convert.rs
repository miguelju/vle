//! Mass-flow CLI — second extension demo for `vle-units`.
//!
//! Like `length_convert`, this shows the full extension flow but for a
//! *derived* dimension (mass ÷ time, M¹·T⁻¹). It also illustrates a more
//! interesting catalog: SI metric, US customary, and the engineering
//! shorthand you see on heat-balance sheets and refinery PFDs.
//!
//! 1. Register the `mass_flow` dimension with SI exponent vector
//!    `(0, 1, -1, 0, 0, 0, 0)` — M¹·T⁻¹.
//! 2. Bulk-register units against it, with kg/s as canonical.
//! 3. Parse `"<value> <unit>"` from `argv[1]` and convert to `argv[2]`.
//!
//! Run from the repo root:
//!
//! ```sh
//! cargo run -p vle-units --example mass_flow_convert -- "10 kg/s" lb/h
//! # → 10 kg/s = 79366.41438655593 lb/h
//!
//! cargo run -p vle-units --example mass_flow_convert -- "1 MMlb/day" t/h
//! # → 1 MMlb/day = 18.89968208333333 t/h
//!
//! cargo run -p vle-units --example mass_flow_convert -- "500 klb/h" kg/s
//! # → 500 klb/h = 62.99894027777778 kg/s
//! ```

use std::env;
use std::process::ExitCode;

use vle_units::{DimensionVector, UnitRegistry};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut reg = UnitRegistry::with_vle_defaults();

    // Mass flow = mass ÷ time. SI exponent vector is (L, M, T, I, Θ, N, J),
    // so M¹·T⁻¹ → [0, 1, -1, 0, 0, 0, 0]. The negative time exponent is
    // what distinguishes a *flow* (kg/s) from an *amount* (kg).
    reg.define_dimension("mass_flow", DimensionVector::new([0, 1, -1, 0, 0, 0, 0]))?;

    // Conversion constants (kept as `const` so the table below stays readable):
    //   1 lb        = 0.45359237 kg  (exact, 1959 international agreement)
    //   1 short ton = 2000 lb        (US ton, distinct from UK long ton)
    //   1 oz        = 1/16 lb        (avoirdupois)
    // Time: 60 s = 1 min, 3600 s = 1 h, 86400 s = 1 day.
    const LB_KG: f64 = 0.453_592_37;
    const STON_KG: f64 = 907.184_74; // 2000 lb
    const OZ_KG: f64 = 0.028_349_523_125; // lb / 16

    // `scale` is canonical units per 1 of this unit, i.e. kg/s per unit.
    // `offset = 0.0` for everything — mass flow has no affine units.
    for (name, kg_per_s_per_unit) in [
        // ── SI metric ─────────────────────────────────────────────────────
        ("kg/s", 1.0),
        ("g/s", 1.0e-3),
        ("kg/min", 1.0 / 60.0),
        ("kg/h", 1.0 / 3600.0),
        ("kg/hr", 1.0 / 3600.0),
        ("kg/day", 1.0 / 86_400.0),
        ("g/h", 1.0e-3 / 3600.0),
        ("g/min", 1.0e-3 / 60.0),
        ("t/h", 1000.0 / 3600.0), // metric tonne (1000 kg), distinct from US ton
        ("t/day", 1000.0 / 86_400.0),
        // ── US customary / imperial ───────────────────────────────────────
        ("lb/s", LB_KG),
        ("lb/min", LB_KG / 60.0),
        ("lb/h", LB_KG / 3600.0),
        ("lb/hr", LB_KG / 3600.0),
        ("lb/day", LB_KG / 86_400.0),
        ("oz/s", OZ_KG),
        ("oz/h", OZ_KG / 3600.0),
        ("ston/h", STON_KG / 3600.0), // short ton per hour (US)
        ("ston/day", STON_KG / 86_400.0),
        // ── Engineering shorthand (steam balances, refinery PFDs) ─────────
        ("klb/h", 1000.0 * LB_KG / 3600.0), // kilopound per hour
        ("klb/hr", 1000.0 * LB_KG / 3600.0),
        ("MMlb/day", 1.0e6 * LB_KG / 86_400.0), // million pounds per day
    ] {
        reg.define_with_dimension(name, "mass_flow", kg_per_s_per_unit, 0.0)?;
    }

    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!(r#"usage: mass_flow_convert "<value> <from-unit>" <to-unit>"#);
        eprintln!(r#"example: mass_flow_convert "10 kg/s" lb/h"#);
        return Err("expected exactly 2 arguments".into());
    }

    let q = reg.parse(&args[0])?;
    let value_in_target = reg.from_canonical(q.canonical, &args[1])?;

    println!("{} = {} {}", args[0], value_in_target, args[1]);
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
