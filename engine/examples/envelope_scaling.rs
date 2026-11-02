//! Phase-envelope tracing cost against mixture size — the measurement behind
//! `OPTIMIZATION_PLAN_PART1.md` §15 (analytic envelope Jacobian).
//!
//! Traces four envelopes and prints, for each: the number of points, the mean
//! wall time per trace, and the start/end of the trace. With `--dump <dir>`
//! it also writes every traced point (T, P, K) to `<dir>/<case>.csv`, so the
//! same harness run on two commits shows whether a change moved the answers,
//! not only the time.
//!
//! Run (release, it is a timing harness):
//! ```text
//! cargo run --release --example envelope_scaling -- --calls 200 --dump /tmp/env
//! ```
//!
//! The Jacobian's forward-difference cost was `2(n+2)` min-Gibbs evaluations
//! per Newton iteration, so the gap between the old and new method should
//! widen with n — which is why this goes up to 10 components rather than
//! stopping at the binary the criterion bench uses.

use std::io::Write as _;
use std::time::Instant;

use vle_thermo::db;
use vle_thermo::eos::{CubicEos, LiquidModel, VaporModel};
use vle_thermo::flash::SystemSpec;
use vle_thermo::flash::envelope::{EnvelopePoint, trace_envelope};
use vle_thermo::mixing::MixingRule;
use vle_thermo::types::Component;

/// Mean wall time of `f` over `calls` calls, in **µs**.
fn time_us(calls: usize, mut f: impl FnMut()) -> f64 {
    for _ in 0..calls / 10 + 1 {
        f();
    }
    let start = Instant::now();
    for _ in 0..calls {
        f();
    }
    start.elapsed().as_secs_f64() * 1e6 / calls as f64
}

/// A classical-mixing cubic system on `eos`, kij = 0.
fn cubic(components: &[Component], eos: CubicEos) -> SystemSpec<'_> {
    SystemSpec {
        components,
        vapor: VaporModel::Cubic(eos),
        liquid: LiquidModel::Cubic(eos),
        mixing_rule: MixingRule::Classical,
        kij: &[],
        aij: &[],
        alpha: &[],
        vl: &[],
        delta: &[],
        sat_models: &[],
        ge_model: None,
    }
}

fn components(names: &[&str]) -> Vec<Component> {
    names
        .iter()
        .map(|n| db::component(n).expect("component in the bundled database"))
        .collect()
}

/// One envelope to trace: (label, components, EOS, z, p_start in kPa,
/// max points).
type Case<'a> = (&'a str, &'a [Component], CubicEos, Vec<f64>, f64, usize);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| args.iter().skip_while(|a| *a != name).nth(1).cloned();
    let calls: usize = flag("--calls").and_then(|v| v.parse().ok()).unwrap_or(200);
    let dump = flag("--dump");

    let c1c2 = components(&["methane", "ethane"]);
    let c4c7 = components(&["n-butane", "n-heptane"]);
    let five = components(&["methane", "ethane", "propane", "n-butane", "n-pentane"]);
    let ten = components(&[
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
    ]);

    // (label, components, EOS, z, p_start kPa, max points)
    let cases: Vec<Case> = vec![
        (
            "c1c2_pr_40",
            &c1c2,
            CubicEos::PR1976,
            vec![0.5, 0.5],
            300.0,
            40,
        ),
        (
            "c1c2_pr_loop",
            &c1c2,
            CubicEos::PR1976,
            vec![0.5, 0.5],
            100.0,
            400,
        ),
        (
            "c4c7_pr_loop",
            &c4c7,
            CubicEos::PR1976,
            vec![0.5, 0.5],
            100.0,
            400,
        ),
        (
            "five_pr_loop",
            &five,
            CubicEos::PR1976,
            vec![0.5, 0.2, 0.15, 0.1, 0.05],
            100.0,
            400,
        ),
        (
            "ten_rks_loop",
            &ten,
            CubicEos::RKS1972,
            vec![0.1; 10],
            100.0,
            400,
        ),
    ];

    println!("mean of {calls} traces per case (µs)");
    println!(
        "{:<14} {:>3} {:>6} {:>12} {:>10} {:>12} {:>12}",
        "case", "n", "points", "µs/trace", "µs/point", "first T,P", "last T,P"
    );
    for (label, comps, eos, z, p_start, max_points) in &cases {
        let spec = cubic(comps, *eos);
        let env: Vec<EnvelopePoint> =
            trace_envelope(&spec, z, *p_start, *max_points).expect("envelope traces");
        let us = time_us(calls, || {
            std::hint::black_box(trace_envelope(&spec, z, *p_start, *max_points).unwrap());
        });
        let (a, b) = (env.first().unwrap(), env.last().unwrap());
        println!(
            "{:<14} {:>3} {:>6} {:>12.1} {:>10.2} {:>5.1},{:>6.0} {:>5.1},{:>6.0}",
            label,
            z.len(),
            env.len(),
            us,
            us / env.len() as f64,
            a.t,
            a.p,
            b.t,
            b.p
        );
        if let Some(dir) = &dump {
            std::fs::create_dir_all(dir).expect("dump dir");
            let mut f = std::fs::File::create(format!("{dir}/{label}.csv")).expect("dump file");
            for pt in &env {
                let ks: Vec<String> = pt.k.iter().map(|k| format!("{k:.15e}")).collect();
                writeln!(f, "{:.15e},{:.15e},{}", pt.t, pt.p, ks.join(",")).unwrap();
            }
        }
    }
}
