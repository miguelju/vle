//! Length-conversion CLI — a small, complete demo of `vle-units`.
//!
//! The crate ships with thermodynamic units (K, kPa, kJ/kmol, …) but not
//! length, so this example shows the full extension flow:
//!
//! 1. Register a brand-new dimension (`length`) with its SI exponent vector
//! 2. Bulk-register a handful of length units against it (m, in, ft, mi, …)
//! 3. Parse a `"<value> <unit>"` string from `argv[1]`
//! 4. Convert to the unit named in `argv[2]`
//!
//! Run from the repo root:
//!
//! ```sh
//! cargo run -p vle-units --example length_convert -- "1 mile" m
//! # → 1 mile = 1609.344 m
//!
//! cargo run -p vle-units --example length_convert -- "100 yd" ft
//! # → 100 yd = 300 ft
//! ```

use std::env;
use std::process::ExitCode;

use vle_units::{DimensionVector, UnitRegistry};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Start from the VLE defaults — not required for length, but mirrors
    // real usage where a single registry handles every unit the program
    // will see (thermo + custom).
    let mut reg = UnitRegistry::with_vle_defaults();

    // SI exponent vector is (L, M, T, I, Θ, N, J). Length is L^1.
    reg.define_dimension("length", DimensionVector::new([1, 0, 0, 0, 0, 0, 0]))?;

    // `scale` = meters per 1 of this unit. `offset = 0.0` for everything
    // that isn't an affine unit (like °C). Aliases are just extra rows
    // pointing at the same scale.
    for (name, meters_per_unit) in [
        // SI prefixes
        ("m", 1.0),
        ("meter", 1.0),
        ("mm", 1.0e-3),
        ("cm", 1.0e-2),
        ("km", 1.0e3),
        // US customary (exact, by 1959 international agreement)
        ("in", 0.0254),
        ("inch", 0.0254),
        ("ft", 0.3048), // 12 in
        ("foot", 0.3048),
        ("yd", 0.9144), // 3 ft
        ("yard", 0.9144),
        ("mi", 1609.344), // 5280 ft
        ("mile", 1609.344),
        // Marine
        ("nmi", 1852.0), // international nautical mile, by definition
    ] {
        reg.define_with_dimension(name, "length", meters_per_unit, 0.0)?;
    }

    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!(r#"usage: length_convert "<value> <from-unit>" <to-unit>"#);
        eprintln!(r#"example: length_convert "1 mile" m"#);
        return Err("expected exactly 2 arguments".into());
    }

    // `parse` splits on whitespace and converts to the dimension's canonical
    // unit (meters for length). `from_canonical` projects that back into
    // whatever target unit the caller asked for.
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
