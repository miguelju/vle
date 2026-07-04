//! Criterion benchmark suite — Milestone 8.2 (PERFORMANCE_PROPOSAL.md Track E).
//!
//! This is the baseline every later performance claim is measured against.
//! Groups mirror the engine's mathematical layers:
//!
//! - `alpha`      — α(Tr) dispatch cost across representative EOS variants
//! - `z_factor`   — cubic solve + root selection (the engine's hottest path)
//! - `ln_phi`     — pure-component fugacity coefficient (Z + attractive term)
//! - `saturation` — Antoine and Riedel Psat correlations
//! - `activity`   — Wilson ln γ for a 3-component mixture (the most
//!   expensive activity model: N² Boltzmann factors per component)
//!
//! Mixture-fugacity, Rachford-Rice, and flash benches are added by later
//! milestones as those layers land (M8.3 / M9 — see the roadmap).
//!
//! Run locally with `cargo bench -p vle-thermo`. The CI `bench` job runs
//! the same suite informationally (deltas reported, never blocking) — see
//! `.github/workflows/ci.yml`.
//!
//! Rust-idiom note for readers new to criterion: `black_box` tells the
//! optimizer "assume this value is observed", preventing LLVM from
//! constant-folding the whole benchmark body away (with LTO fat + a single
//! codegen unit, it absolutely would).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vle_thermo::eos::{self, CubicEos, PhaseId};
use vle_thermo::types::Component;
use vle_thermo::{activity, saturation};

/// n-pentane with the data every bench below needs (critical constants for
/// the EOS groups, a reduced-Antoine fit for the saturation group).
fn n_pentane() -> Component {
    Component {
        name: "n-pentane".into(),
        tc: 469.7,
        pc: 3370.0,
        omega: 0.252,
        tb: 309.2,
        psat_coeffs: vec![6.738, 3165.0, 0.0],
        liquid_volume: 116.0,
        ..Component::default()
    }
}

/// α(Tr) dispatch: one cheap variant (PR), one mid-tier (PRSV), and a
/// 3-parameter Pascal model (Patel-Teja). The spread shows how much of the
/// α cost is the enum dispatch vs the transcendental math.
fn bench_alpha(c: &mut Criterion) {
    let comp = n_pentane();
    let mut g = c.benchmark_group("alpha");
    for (label, eos_variant) in [
        ("pr1976", CubicEos::PR1976),
        ("prsv1986", CubicEos::PRSV1986),
        ("patel_teja", CubicEos::PatelTeja),
    ] {
        g.bench_function(label, |b| {
            b.iter(|| eos::alpha(black_box(eos_variant), black_box(0.85), black_box(&comp)))
        });
    }
    g.finish();
}

/// Z-factor: cubic coefficients + Cardano solve + phase root selection.
/// Sub-critical two-root regime (the common flash condition) for PR and
/// the 3-parameter Patel-Teja path.
fn bench_z_factor(c: &mut Criterion) {
    let comp = n_pentane();
    let mut g = c.benchmark_group("z_factor");
    g.bench_function("pr1976_vapor", |b| {
        b.iter(|| {
            eos::z_factor(
                black_box(CubicEos::PR1976),
                black_box(400.0),
                black_box(1500.0),
                black_box(&comp),
                PhaseId::Vapor,
            )
            .unwrap()
        })
    });
    g.bench_function("pr1976_liquid", |b| {
        b.iter(|| {
            eos::z_factor(
                black_box(CubicEos::PR1976),
                black_box(400.0),
                black_box(1500.0),
                black_box(&comp),
                PhaseId::Liquid,
            )
            .unwrap()
        })
    });
    g.bench_function("patel_teja_vapor", |b| {
        b.iter(|| {
            eos::z_factor(
                black_box(CubicEos::PatelTeja),
                black_box(400.0),
                black_box(1500.0),
                black_box(&comp),
                PhaseId::Vapor,
            )
            .unwrap()
        })
    });
    g.finish();
}

/// Pure-component ln φ — Z-factor plus the attractive-term integral.
fn bench_ln_phi(c: &mut Criterion) {
    let comp = n_pentane();
    let mut g = c.benchmark_group("ln_phi");
    g.bench_function("pr1976_vapor", |b| {
        b.iter(|| {
            eos::ln_phi_pure(
                black_box(CubicEos::PR1976),
                black_box(400.0),
                black_box(1500.0),
                black_box(&comp),
                PhaseId::Vapor,
            )
            .unwrap()
        })
    });
    g.finish();
}

/// Saturation pressure: the analytic Antoine hot path and the Riedel
/// correlation (representative of the M7.4 non-Antoine models).
fn bench_saturation(c: &mut Criterion) {
    let comp = n_pentane();
    let riedel = Component {
        // Riedel reads Tc/Pc/Tb only.
        psat_coeffs: vec![],
        ..n_pentane()
    };
    let mut g = c.benchmark_group("saturation");
    g.bench_function("antoine", |b| {
        b.iter(|| saturation::psat_antoine(black_box(&comp), black_box(350.0)).unwrap())
    });
    g.bench_function("riedel", |b| {
        b.iter(|| saturation::psat_riedel(black_box(&riedel), black_box(350.0)).unwrap())
    });
    g.finish();
}

/// Wilson ln γ for a ternary — N² Λ evaluations (each an exp) per call.
/// This is the activity-model cost that dominates GE-based mixing rules
/// (Wong-Sandler, MHV) in M8.3, so it needs a baseline now.
fn bench_activity(c: &mut Criterion) {
    let x = [0.3, 0.45, 0.25];
    // Wilson aij in kJ/kmol — magnitudes typical of the thesis systems.
    let aij = vec![
        vec![0.0, 1200.0, 800.0],
        vec![-300.0, 0.0, 650.0],
        vec![450.0, -150.0, 0.0],
    ];
    let vl = [90.0, 116.0, 130.0];
    let mut g = c.benchmark_group("activity");
    g.bench_function("wilson_ln_gamma_ternary", |b| {
        b.iter(|| {
            activity::ln_gamma(
                activity::ActivityModel::Wilson,
                black_box(0),
                black_box(&x),
                black_box(&aij),
                black_box(&vl),
                &[],
                black_box(340.0),
            )
        })
    });
    g.bench_function("wilson_excess_gibbs_ternary", |b| {
        b.iter(|| {
            activity::excess_gibbs(
                activity::ActivityModel::Wilson,
                black_box(&x),
                black_box(&aij),
                black_box(&vl),
                &[],
                black_box(340.0),
            )
        })
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_alpha,
    bench_z_factor,
    bench_ln_phi,
    bench_saturation,
    bench_activity
);
criterion_main!(benches);
