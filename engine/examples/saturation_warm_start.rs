//! Warm-start payoff of the saturation-temperature solvers — the harness
//! behind the tables in `OPTIMIZATION_PLAN_PART1.md` §11 and §14.
//!
//! Mixture: the 10-component light-hydrocarbon RKS system of the downstream
//! column library's benchmark (propane … n-decane, equimolar) at 1000 kPa.
//! For each guess quality — the converged answer nudged by ΔT in **K** and a
//! relative ΔK on every K-value, as a neighboring column stage would supply
//! it — the harness times `bubble_temperature_from` / `dew_temperature_from`
//! (mean of `--calls` calls, default 3000) against the cold solve, and checks
//! that every warm answer equals the cold one.
//!
//! ```text
//! cargo run --release -p vle-thermo --features component-db \
//!     --example saturation_warm_start [-- --calls N]
//! ```
//!
//! Release build only: a debug build measures the debug allocator.

use std::hint::black_box;
use std::time::Instant;

use vle_thermo::db;
use vle_thermo::eos::{CubicEos, LiquidModel, VaporModel};
use vle_thermo::flash::FlashError;
use vle_thermo::flash::SystemSpec;
use vle_thermo::flash::bubble::{SaturationResult, bubble_temperature, bubble_temperature_from};
use vle_thermo::flash::dew::{dew_temperature, dew_temperature_from};
use vle_thermo::mixing::MixingRule;
use vle_thermo::types::Component;

const P: f64 = 1000.0; // kPa absolute
const TOL: f64 = 1e-9;
const MAX_ITER: usize = 200;

type Cold = fn(&SystemSpec, f64, &[f64], f64, usize) -> Result<SaturationResult, FlashError>;
type Warm = fn(
    &SystemSpec,
    f64,
    &[f64],
    f64,
    Option<&[f64]>,
    f64,
    usize,
) -> Result<SaturationResult, FlashError>;

/// Mean wall time of `f` over `calls` calls, in **µs**.
fn time_us(calls: usize, mut f: impl FnMut()) -> f64 {
    // A short warm-up so the first timed call does not pay for cold caches.
    for _ in 0..calls / 10 + 1 {
        f();
    }
    let start = Instant::now();
    for _ in 0..calls {
        f();
    }
    start.elapsed().as_secs_f64() * 1e6 / calls as f64
}

fn main() {
    let calls = std::env::args()
        .skip_while(|a| a != "--calls")
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000usize);

    let names = [
        "propane",
        "isobutane",
        "n-butane",
        "isopentane",
        "n-pentane",
        "n-hexane",
        "n-heptane",
        "n-octane",
        "n-nonane",
        "n-decane",
    ];
    let comps: Vec<Component> = names
        .iter()
        .map(|n| db::component(n).expect("component in the bundled database"))
        .collect();
    let spec = SystemSpec {
        components: &comps,
        vapor: VaporModel::Cubic(CubicEos::RKS1972),
        liquid: LiquidModel::Cubic(CubicEos::RKS1972),
        mixing_rule: MixingRule::Classical,
        kij: &[],
        aij: &[],
        alpha: &[],
        vl: &[],
        delta: &[],
        sat_models: &[],
        ge_model: None,
    };
    let z = vec![0.1; names.len()];

    // (ΔT in K, relative ΔK) — the rows of the §11 table.
    let guesses = [
        (0.0, 0.0),
        (0.05, 0.002),
        (0.2, 0.01),
        (0.5, 0.05),
        (2.0, 0.10),
    ];
    let solvers: [(&str, Cold, Warm); 2] = [
        ("bubble T", bubble_temperature, bubble_temperature_from),
        ("dew T", dew_temperature, dew_temperature_from),
    ];

    println!("10-component RKS alkanes, {P} kPa, tol {TOL:e}, mean of {calls} calls (µs)");
    println!(
        "{:<16} {:>10} {:>10}",
        "guess (ΔT, ΔK)", "bubble T", "dew T"
    );
    let mut rows: Vec<(String, [f64; 2])> = vec![("cold (Wilson)".into(), [0.0; 2])];
    for (dt, dk) in guesses {
        rows.push((format!("{dt} K, {} %", dk * 100.0), [0.0; 2]));
    }
    for (col, (what, cold_fn, warm_fn)) in solvers.iter().enumerate() {
        let cold = cold_fn(&spec, P, &z, TOL, MAX_ITER).expect("cold solve");
        rows[0].1[col] = time_us(calls, || {
            black_box(cold_fn(&spec, P, black_box(&z), TOL, MAX_ITER).unwrap());
        });
        for (row, (dt, dk)) in guesses.iter().enumerate() {
            let t0 = cold.value + dt;
            let k0: Vec<f64> = cold.k.iter().map(|k| k * (1.0 + dk)).collect();
            let warm = warm_fn(&spec, P, &z, t0, Some(&k0), TOL, MAX_ITER).expect("warm solve");
            assert!(
                (warm.value - cold.value).abs() < 1e-6,
                "{what}: warm {} vs cold {}",
                warm.value,
                cold.value
            );
            rows[row + 1].1[col] = time_us(calls, || {
                black_box(warm_fn(&spec, P, black_box(&z), t0, Some(&k0), TOL, MAX_ITER).unwrap());
            });
        }
    }
    for (label, [b, d]) in rows {
        println!("{label:<16} {b:>10.2} {d:>10.2}");
    }
}
