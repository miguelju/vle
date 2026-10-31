//! Flash-vs-stability grid sweep — the evidence harness behind
//! `OPTIMIZATION_PLAN_PART1.md` §10 and `OPTIMIZATION_NEXT_STEPS.md` Step 0.
//!
//! For each mixture, run the cold isothermal flash and the TPD stability
//! analysis at every point of a 60 × 60 T–P grid (150–600 K linear,
//! 10–80 000 kPa log-spaced) and tabulate where they agree, where they
//! disagree, and where the flash fails.
//!
//! Run with:
//! ```sh
//! cargo run --release --features component-db --example flash_stability_sweep
//! ```
//!
//! The §10 sweep that first produced the 55-failure-point figure was a
//! scratch harness that was never committed; this file is its reproducible
//! replacement, so the numbers recorded in the plan can be regenerated.

use vle_thermo::db::component;
use vle_thermo::eos::{CubicEos, LiquidModel, VaporModel};
use vle_thermo::flash::isothermal::{flash_isothermal, flash_isothermal_warm};
use vle_thermo::flash::stability::{Stability, stability_analysis};
use vle_thermo::flash::{FlashError, SystemSpec};
use vle_thermo::mixing::MixingRule;
use vle_thermo::types::Component;

/// Flash iteration cap and tolerance — the values the engine's own callers use.
const TOL: f64 = 1e-10;
const MAX_ITER: usize = 200;
const GRID: usize = 60;

/// One mixture to sweep: name, component list, feed and kij matrix.
struct Case {
    label: &'static str,
    comps: Vec<Component>,
    z: Vec<f64>,
    kij: Vec<Vec<f64>>,
}

fn comps(names: &[&str]) -> Vec<Component> {
    // `db::component` is the bundled Rust-side database (feature `component-db`).
    names.iter().map(|n| component(n).expect(n)).collect()
}

fn equimolar(n: usize) -> Vec<f64> {
    vec![1.0 / n as f64; n]
}

/// Hydrocarbon cases (classical mixing, kij = 0): the six-mixture shape of §10.
fn hydrocarbon_cases() -> Vec<Case> {
    let mk = |label, names: &[&str]| {
        let c = comps(names);
        let n = c.len();
        Case {
            label,
            comps: c,
            z: equimolar(n),
            kij: vec![],
        }
    };
    vec![
        mk("methane/ethane", &["methane", "ethane"]),
        mk("methane/n-pentane", &["methane", "n-pentane"]),
        mk("n-butane/n-heptane", &["n-butane", "n-heptane"]),
        mk(
            "C1/C3/nC5/nC7",
            &["methane", "propane", "n-pentane", "n-heptane"],
        ),
        mk(
            "C1-nC8 (6)",
            &[
                "methane", "ethane", "propane", "n-butane", "n-hexane", "n-octane",
            ],
        ),
        mk(
            "C3-nC10 (10, no methane)",
            &[
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
            ],
        ),
    ]
}

/// Liquid–liquid-prone cases: water + hydrocarbon on PR with a large kij, the
/// usual way a cubic EOS represents the water/hydrocarbon immiscibility gap.
/// `kij_w` ≈ 0.5 is the textbook order of magnitude for PR water–alkane.
fn water_cases() -> Vec<Case> {
    let mk = |label, names: &[&str], z: Vec<f64>, kij_w: f64| {
        let c = comps(names);
        let n = c.len();
        // Water is component 0; every water–HC pair gets kij_w, HC–HC 0.
        // (i == 0) != (j == 0) is true exactly for the water–HC pairs.
        let kij: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| if (i == 0) != (j == 0) { kij_w } else { 0.0 })
                    .collect()
            })
            .collect();
        Case {
            label,
            comps: c,
            z,
            kij,
        }
    };
    vec![
        mk(
            "water/n-hexane",
            &["water", "n-hexane"],
            vec![0.5, 0.5],
            0.5,
        ),
        mk(
            "water/n-decane",
            &["water", "n-decane"],
            vec![0.3, 0.7],
            0.5,
        ),
        mk(
            "water/propane/n-heptane",
            &["water", "propane", "n-heptane"],
            vec![0.3, 0.35, 0.35],
            0.5,
        ),
        mk("water/benzene", &["water", "benzene"], vec![0.5, 0.5], 0.3),
    ]
}

#[derive(Default)]
struct Tally {
    flash_single: usize,
    flash_single_stab_unstable: usize,
    /// Of the `missed` points, how many converge to a genuine two-phase split
    /// when the flash is re-run from the stability test's trial-phase K —
    /// i.e. how many are real misses rather than stability false alarms.
    missed_confirmed: usize,
    flash_two: usize,
    flash_two_stab_stable: usize,
    /// Fallback answers (iterations == MAX_ITER, single phase) — only
    /// non-zero once the stability-on-failure fallback is built.
    fallback: usize,
    fail: usize,
    fail_stab_stable: usize,
    fail_stab_unstable: usize,
    fail_stab_error: usize,
    stab_error: usize,
    /// (T, P) of every remaining flash failure, for the regression test.
    fail_points: Vec<(f64, f64)>,
}

fn sweep(case: &Case) -> Tally {
    let spec = SystemSpec {
        components: &case.comps,
        vapor: VaporModel::Cubic(CubicEos::PR1976),
        liquid: LiquidModel::Cubic(CubicEos::PR1976),
        mixing_rule: MixingRule::Classical,
        kij: &case.kij,
        aij: &[],
        alpha: &[],
        vl: &[],
        delta: &[],
        sat_models: &[],
        ge_model: None,
    };
    let mut t = Tally::default();
    for i in 0..GRID {
        let temp = 150.0 + 450.0 * i as f64 / (GRID - 1) as f64;
        for j in 0..GRID {
            let p = 10.0 * (8000.0_f64).powf(j as f64 / (GRID - 1) as f64);
            let flash = flash_isothermal(&spec, temp, p, &case.z, TOL, MAX_ITER);
            let stab = stability_analysis(&spec, temp, p, &case.z, MAX_ITER);
            if stab.is_err() {
                t.stab_error += 1;
            }
            let stable = matches!(stab, Ok(Stability::Stable));
            match flash {
                Ok(r) if r.two_phase => {
                    t.flash_two += 1;
                    if stable {
                        t.flash_two_stab_stable += 1;
                    }
                }
                Ok(r) => {
                    t.flash_single += 1;
                    if r.iterations == MAX_ITER {
                        t.fallback += 1;
                    }
                    if let Ok(Stability::Unstable { trial_k, .. }) = &stab {
                        t.flash_single_stab_unstable += 1;
                        let retry = flash_isothermal_warm(
                            &spec,
                            temp,
                            p,
                            &case.z,
                            Some(trial_k),
                            TOL,
                            MAX_ITER,
                        );
                        if matches!(retry, Ok(r) if r.two_phase) {
                            t.missed_confirmed += 1;
                        }
                    }
                }
                Err(FlashError::NoConvergence { .. }) | Err(_) => {
                    t.fail += 1;
                    t.fail_points.push((temp, p));
                    match stab {
                        Ok(Stability::Stable) => t.fail_stab_stable += 1,
                        Ok(Stability::Unstable { .. }) => t.fail_stab_unstable += 1,
                        Err(_) => t.fail_stab_error += 1,
                    }
                }
            }
        }
    }
    t
}

fn main() {
    let verbose = std::env::args().any(|a| a == "--points");
    for (group, cases) in [
        ("hydrocarbons (PR, kij = 0)", hydrocarbon_cases()),
        ("water/hydrocarbon (PR, kij_w)", water_cases()),
    ] {
        println!("\n== {group} ==");
        println!(
            "{:<26} {:>7} {:>7} {:>7} {:>7} {:>9} {:>6} {:>5} {:>6} {:>6} {:>6}",
            "mixture",
            "single",
            "missed",
            "m:split",
            "two",
            "spurious",
            "fallbk",
            "fail",
            "f:stab",
            "f:unst",
            "f:err"
        );
        let mut sum = Tally::default();
        let start = std::time::Instant::now();
        for case in &cases {
            let t = sweep(case);
            println!(
                "{:<26} {:>7} {:>7} {:>7} {:>7} {:>9} {:>6} {:>5} {:>6} {:>6} {:>6}",
                case.label,
                t.flash_single,
                t.flash_single_stab_unstable,
                t.missed_confirmed,
                t.flash_two,
                t.flash_two_stab_stable,
                t.fallback,
                t.fail,
                t.fail_stab_stable,
                t.fail_stab_unstable,
                t.fail_stab_error
            );
            if verbose {
                for (tt, pp) in &t.fail_points {
                    println!("    fail at T = {tt:.4} K, P = {pp:.4} kPa");
                }
            }
            sum.flash_single += t.flash_single;
            sum.flash_single_stab_unstable += t.flash_single_stab_unstable;
            sum.missed_confirmed += t.missed_confirmed;
            sum.flash_two += t.flash_two;
            sum.flash_two_stab_stable += t.flash_two_stab_stable;
            sum.fallback += t.fallback;
            sum.fail += t.fail;
            sum.fail_stab_stable += t.fail_stab_stable;
            sum.fail_stab_unstable += t.fail_stab_unstable;
            sum.fail_stab_error += t.fail_stab_error;
            sum.stab_error += t.stab_error;
        }
        println!(
            "{:<26} {:>7} {:>7} {:>7} {:>7} {:>9} {:>6} {:>5} {:>6} {:>6} {:>6}",
            "TOTAL",
            sum.flash_single,
            sum.flash_single_stab_unstable,
            sum.missed_confirmed,
            sum.flash_two,
            sum.flash_two_stab_stable,
            sum.fallback,
            sum.fail,
            sum.fail_stab_stable,
            sum.fail_stab_unstable,
            sum.fail_stab_error
        );
        println!(
            "  stability errors anywhere: {}; wall time {:.2?}",
            sum.stab_error,
            start.elapsed()
        );
    }
    println!(
        "\ncolumns: single = flash single-phase; missed = of those, stability says unstable;\n\
         m:split = of those, a flash re-run from the trial-phase K converges to a split;\n\
         two = flash two-phase; spurious = of those, stability says stable;\n\
         fallbk = single-phase answers produced by the stability-on-failure fallback;\n\
         fail = flash error; f:stab / f:unst / f:err = stability verdict at the failures."
    );
}
