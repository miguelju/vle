//! Isothermal-flash cost over a whole P–T phase map — the measurement behind
//! `OPTIMIZATION_PLAN_PART1.md` §22.
//!
//! Five Peng–Robinson mixtures, each flashed at every cell of a 96 × 64
//! grid (6 144 flashes) that covers the single-phase regions, the two-phase
//! region and the critical neighbourhood. That is what a phase-map plot
//! does, and it is a different load from the criterion benches, which time
//! one state each: v0.20.0 was 1.2–2.2× slower than v0.19.0 here while no
//! isothermal bench moved.
//!
//! For each mixture it prints the best of three grid times, the time spent
//! in two-phase / single-phase / failed cells, the median and 99th-percentile
//! cell, and the mean time of 200 `critical_point` calls. With a directory
//! argument it also writes one CSV per mixture (T in K, P in kPa, cell time
//! in µs, kind 0 = error / 1 = one phase / 2 = two phases, β, iterations), so
//! two builds can be compared cell by cell and not only by the clock.
//!
//! ```text
//! cargo run --release --features component-db --example phase_map_timing -- /tmp/out
//! ```

use std::io::Write as _;
use std::time::Instant;
use vle_thermo::db;
use vle_thermo::eos::{CubicEos, LiquidModel, VaporModel};
use vle_thermo::flash::SystemSpec;
use vle_thermo::flash::critical::critical_point;
use vle_thermo::flash::isothermal::flash_isothermal;
use vle_thermo::mixing::MixingRule;
use vle_thermo::types::Component;

/// Components by name from the bundled database.
fn comps(names: &[&str]) -> Vec<Component> {
    names
        .iter()
        .map(|n| db::component(n).expect("db"))
        .collect()
}
/// One phase map: (label, components, feed z, top T in **K**, top P in **kPa**).
type Case = (&'static str, Vec<Component>, Vec<f64>, f64, f64);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dump = args.get(1).cloned();
    // (label, components, feed z, top temperature in K, top pressure in kPa).
    // Every grid runs from 130 K and 20 kPa. Methane-rich feeds are here on
    // purpose: a component above ≈ 74 % puts the whole single-phase region
    // inside the single-phase check's margin (§22).
    let cases: Vec<Case> = vec![
        (
            "c1c2",
            comps(&["methane", "ethane"]),
            vec![0.5, 0.5],
            290.0,
            9000.0,
        ),
        (
            "co2c4",
            comps(&["carbon dioxide", "n-butane"]),
            vec![0.6, 0.4],
            445.0,
            9000.0,
        ),
        (
            "c1c4",
            comps(&["methane", "n-butane"]),
            vec![0.7, 0.3],
            445.0,
            15000.0,
        ),
        (
            "c1c5",
            comps(&["methane", "n-pentane"]),
            vec![0.7, 0.3],
            445.0,
            21500.0,
        ),
        (
            "five",
            comps(&["methane", "ethane", "propane", "n-butane", "n-pentane"]),
            vec![0.75, 0.1, 0.07, 0.05, 0.03],
            445.0,
            15000.0,
        ),
    ];
    for (label, c, z, tmax, pmax) in &cases {
        let spec = SystemSpec {
            components: c,
            vapor: VaporModel::Cubic(CubicEos::PR1976),
            liquid: LiquidModel::Cubic(CubicEos::PR1976),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &[],
            alpha: &[],
            vl: &[],
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        let (nt, np) = (96usize, 64usize);
        let mut best = f64::MAX;
        let mut cells: Vec<(f64, f64, f64, i32, f64, usize)> = Vec::new();
        for _ in 0..3 {
            cells.clear();
            let start = Instant::now();
            for j in 0..np {
                let p = 20.0 + (pmax - 20.0) * j as f64 / (np - 1) as f64;
                for i in 0..nt {
                    let t = 130.0 + (tmax - 130.0) * i as f64 / (nt - 1) as f64;
                    let c0 = Instant::now();
                    let r = flash_isothermal(&spec, t, p, z, 1e-10, 200);
                    let us = c0.elapsed().as_secs_f64() * 1e6;
                    let (kind, beta, it) = match &r {
                        Ok(f) if f.two_phase => (2, f.beta, f.iterations),
                        Ok(f) => (1, f.beta, f.iterations),
                        Err(_) => (0, f64::NAN, 0),
                    };
                    cells.push((t, p, us, kind, beta, it));
                }
            }
            let ms = start.elapsed().as_secs_f64() * 1e3;
            if ms < best {
                best = ms;
            }
        }
        let sum = |k: i32| cells.iter().filter(|c| c.3 == k).map(|c| c.2).sum::<f64>() / 1e3;
        let cnt = |k: i32| cells.iter().filter(|c| c.3 == k).count();
        let mut ts: Vec<f64> = cells.iter().map(|c| c.2).collect();
        ts.sort_by(|a, b| a.total_cmp(b));
        let n = 200;
        let c0 = Instant::now();
        let mut cp = None;
        for _ in 0..n {
            cp = Some(critical_point(&spec, z, 0.0, 200));
        }
        let cus = c0.elapsed().as_secs_f64() * 1e6 / n as f64;
        println!(
            "{label:6} grid {best:7.1} ms | 2ph {} cells {:6.1} ms | 1ph {} cells {:6.1} ms | err {} {:5.1} ms | median {:5.1} us p99 {:6.1} us | crit {cus:6.1} us {:?}",
            cnt(2),
            sum(2),
            cnt(1),
            sum(1),
            cnt(0),
            sum(0),
            ts[ts.len() / 2],
            ts[ts.len() * 99 / 100],
            cp.unwrap().map(|s| (s.tc, s.pc)).ok()
        );
        if let Some(d) = &dump {
            let mut f = std::fs::File::create(format!("{d}/{label}.csv")).unwrap();
            for c in &cells {
                writeln!(
                    f,
                    "{:.3},{:.3},{:.2},{},{:.6},{}",
                    c.0, c.1, c.2, c.3, c.4, c.5
                )
                .unwrap();
            }
        }
    }
}
