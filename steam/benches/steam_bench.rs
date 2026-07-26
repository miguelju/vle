//! Criterion benchmarks guarding the IAPWS-IF97 hot paths.
//!
//! Groups mirror the standard's own structure, and each one is deliberately
//! **multi-point**:
//!
//! - `region`     — forward `(T, P)` evaluation, several points per region
//!   including near the region boundaries where the cost profile changes
//! - `boundary`   — region selection and the B23 boundary equations, which run
//!   before every forward evaluation
//! - `saturation` — the region-4 line: `psat`, `tsat`, its derivative, and the
//!   full saturated-property rows
//! - `inverse`    — the backward/iterative entry points (`ph`, `ps`, `tx`,
//!   `px`), split by which phase the input lands in, since they take
//!   different code paths
//! - `sweep`      — the same work amortized over a *range* of states
//!
//! **Why multi-point matters.** A single benchmark per region measures one
//! state, and an optimizer — human or agent — will happily make that state
//! faster while degrading its neighbours. Region 3 in particular pays a
//! bracketed density iteration whose cost depends strongly on where in the
//! region you land, so one sample there says almost nothing. The `sweep` group
//! exists for the same reason from the other direction: it is the shape a real
//! caller actually has (a table, a plot, a `vle.steam` batch call), and it
//! catches per-call overhead that a hot single-point loop hides.
//!
//! Units, since IF97 mixes them and the crate's public API does not:
//! `SteamState::tp` and `tsat` take **kPa**; `region_of` and the `b23_*`
//! boundary functions take **MPa**. Every point below was verified to land in
//! its intended region before being pinned here.
//!
//! Run with `cargo bench -p vle-steam`. To compare a change:
//! ```text
//! cargo bench -p vle-steam --bench steam_bench -- --save-baseline before
//! # ...edit...
//! cargo bench -p vle-steam --bench steam_bench -- --baseline before
//! ```
//!
//! Rust-idiom note for readers new to criterion: `black_box` tells the
//! optimizer "assume this value is observed", preventing LLVM from
//! constant-folding the whole benchmark body away — with the release profile's
//! fat LTO it absolutely would.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vle_steam::{
    SteamState, latent_heat, psat, psat_derivative, region_of, regions, sat_p, sat_t, tsat,
};

/// `(label, T in K, P in kPa)` — forward evaluation points, several per region.
/// Each was checked with `region_of` to confirm it lands where the label says.
const TP_POINTS: &[(&str, f64, f64)] = &[
    // Region 1 — compressed liquid. Explicit Gibbs equation, no iteration.
    ("r1_cold", 300.0, 3_000.0),
    ("r1_warm_highp", 500.0, 80_000.0),
    ("r1_near_boundary", 620.0, 100_000.0),
    // Region 2 — superheated vapour. Ideal + residual parts; the low-pressure
    // point is the cheapest state the standard has, the near-B23 point the
    // most expensive in this region.
    ("r2_low_p", 700.0, 3.5),
    ("r2_moderate", 700.0, 3_000.0),
    ("r2_near_b23", 800.0, 40_000.0),
    // Region 3 — the expensive one: Helmholtz in (rho, T), so a (T, P) query
    // costs a bracketed density solve. Three points because the iteration
    // count varies with where you are relative to the critical point.
    ("r3_near_critical", 650.0, 25_000.0),
    ("r3_dense", 650.0, 40_000.0),
    ("r3_hot", 750.0, 78_000.0),
    // Region 5 — high-temperature steam. Shortest coefficient tables.
    ("r5_high_t", 1500.0, 500.0),
    ("r5_high_t_high_p", 2000.0, 30_000.0),
];

/// Forward `(T, P)` evaluation across every region.
fn bench_region(c: &mut Criterion) {
    let mut g = c.benchmark_group("region");
    for (label, t, p) in TP_POINTS {
        g.bench_function(*label, |b| {
            b.iter(|| SteamState::tp(black_box(*t), black_box(*p)).unwrap())
        });
    }
    g.finish();
}

/// Region selection and the B23 boundary. `region_of` runs before every
/// forward evaluation, so a regression here taxes the whole crate — worth
/// measuring on its own rather than only inside `region`.
fn bench_boundary(c: &mut Criterion) {
    let mut g = c.benchmark_group("boundary");
    // One probe per region, in MPa.
    for (label, t, p_mpa) in [
        ("region_of_r1", 300.0, 3.0),
        ("region_of_r2", 700.0, 3.0),
        ("region_of_r3", 650.0, 25.0),
        ("region_of_r5", 1500.0, 0.5),
    ] {
        g.bench_function(label, |b| {
            b.iter(|| region_of(black_box(t), black_box(p_mpa)))
        });
    }
    g.bench_function("b23_p", |b| b.iter(|| regions::b23_p(black_box(650.0))));
    g.bench_function("b23_t", |b| b.iter(|| regions::b23_t(black_box(25.0))));
    g.finish();
}

/// The region-4 saturation line. `psat` is an explicit quartic solve; `tsat`
/// is its inverse; the `sat_*` rows evaluate both bounding phases, so they
/// cost roughly a region-1 plus a region-2 evaluation on top.
fn bench_saturation(c: &mut Criterion) {
    let mut g = c.benchmark_group("saturation");
    for (label, t) in [("psat_400k", 400.0), ("psat_600k", 600.0)] {
        g.bench_function(label, |b| b.iter(|| psat(black_box(t)).unwrap()));
    }
    for (label, p) in [("tsat_100kpa", 100.0), ("tsat_10mpa", 10_000.0)] {
        g.bench_function(label, |b| b.iter(|| tsat(black_box(p)).unwrap()));
    }
    g.bench_function("psat_derivative", |b| {
        b.iter(|| psat_derivative(black_box(500.0)).unwrap())
    });
    // Full saturated-property rows — both phases plus the derived quantities.
    g.bench_function("sat_t", |b| b.iter(|| sat_t(black_box(400.0)).unwrap()));
    g.bench_function("sat_p_1bar", |b| {
        b.iter(|| sat_p(black_box(100.0)).unwrap())
    });
    g.bench_function("sat_p_10mpa", |b| {
        b.iter(|| sat_p(black_box(10_000.0)).unwrap())
    });
    g.bench_function("latent_heat", |b| {
        b.iter(|| latent_heat(black_box(400.0)).unwrap())
    });
    g.finish();
}

/// The inverse / iterative entry points. Split by resulting phase because each
/// takes a different path: the two-phase branch resolves on the saturation
/// line, while the single-phase branches run a backward-equation seed followed
/// by a Newton polish. Benchmarking only the two-phase case — as the previous
/// suite did — leaves the expensive single-phase paths unguarded.
fn bench_inverse(c: &mut Criterion) {
    let mut g = c.benchmark_group("inverse");
    for (label, p, h) in [
        ("ph_two_phase", 1_000.0, 2_000.0),
        ("ph_liquid", 5_000.0, 500.0),
        ("ph_vapor", 1_000.0, 3_200.0),
        // Region 3. The audit that produced the region-2 backward `T(p,h)`
        // deferred the region-3 one *because the suite had no region-3 PH
        // point to measure it against* — the PH sweep runs at 1 MPa and cannot
        // enter region 3. These two close that hole: both land above the B23
        // line, on either side of the critical density, so a future region-3
        // backward equation has a before/after workload to be judged on.
        ("ph_r3_liquid_side", 25_000.0, 1_800.0),
        ("ph_r3_vapor_side", 23_000.0, 2_400.0),
    ] {
        g.bench_function(label, |b| {
            b.iter(|| SteamState::ph(black_box(p), black_box(h)).unwrap())
        });
    }
    for (label, p, s) in [
        ("ps_two_phase", 1_000.0, 5.0),
        ("ps_liquid", 5_000.0, 1.5),
        ("ps_vapor", 1_000.0, 7.5),
    ] {
        g.bench_function(label, |b| {
            b.iter(|| SteamState::ps(black_box(p), black_box(s)).unwrap())
        });
    }
    // Quality-specified states — cheapest of the inverse family (the
    // saturation line is known analytically), so they act as the floor the
    // others are measured against.
    g.bench_function("tx", |b| {
        b.iter(|| SteamState::tx(black_box(400.0), black_box(0.5)).unwrap())
    });
    g.bench_function("px", |b| {
        b.iter(|| SteamState::px(black_box(1_000.0), black_box(0.5)).unwrap())
    });
    g.finish();
}

/// Amortized cost over a *range* of states — the shape a real caller has
/// (a steam table, a T–s plot, a `vle.steam` batch call).
///
/// This is the group that catches an "optimization" which wins at one pinned
/// state and loses across the surrounding surface, and it is where per-call
/// setup overhead becomes visible instead of being hidden by a hot loop
/// hammering identical inputs.
fn bench_sweep(c: &mut Criterion) {
    let mut g = c.benchmark_group("sweep");

    // 200 compressed-liquid states along an isobar.
    let r1: Vec<(f64, f64)> = (0..200)
        .map(|i| (300.0 + i as f64 * 1.5, 20_000.0))
        .collect();
    g.bench_function("region1_200pts", |b| {
        b.iter(|| {
            for (t, p) in black_box(&r1) {
                black_box(SteamState::tp(*t, *p).unwrap());
            }
        })
    });

    // 200 superheated-vapour states.
    let r2: Vec<(f64, f64)> = (0..200)
        .map(|i| (600.0 + i as f64 * 2.0, 1_000.0))
        .collect();
    g.bench_function("region2_200pts", |b| {
        b.iter(|| {
            for (t, p) in black_box(&r2) {
                black_box(SteamState::tp(*t, *p).unwrap());
            }
        })
    });

    // 50 region-3 states — fewer points because each pays the density solve.
    // If the ρ-iteration is ever replaced or warm-started, this is the number
    // that should move most.
    let r3: Vec<(f64, f64)> = (0..50)
        .map(|i| (630.0 + i as f64 * 1.5, 25_000.0 + i as f64 * 400.0))
        .collect();
    g.bench_function("region3_50pts", |b| {
        b.iter(|| {
            for (t, p) in black_box(&r3) {
                black_box(SteamState::tp(*t, *p).unwrap());
            }
        })
    });

    // A saturation table: 100 rows, both phases each.
    let sat: Vec<f64> = (0..100).map(|i| 280.0 + i as f64 * 3.0).collect();
    g.bench_function("sat_table_100rows", |b| {
        b.iter(|| {
            for t in black_box(&sat) {
                black_box(sat_t(*t).unwrap());
            }
        })
    });

    // 200 PH flashes spanning subcooled → two-phase → superheated, so the
    // sweep crosses every branch of the inverse dispatch rather than staying
    // in one.
    let ph: Vec<(f64, f64)> = (0..200)
        .map(|i| (1_000.0, 200.0 + i as f64 * 16.0))
        .collect();
    g.bench_function("ph_flash_200pts", |b| {
        b.iter(|| {
            for (p, h) in black_box(&ph) {
                black_box(SteamState::ph(*p, *h).unwrap());
            }
        })
    });

    g.finish();
}

/// Transport properties (M13.7). Split by whether the R15-11 **critical
/// enhancement** contributes: `λ₂` costs several times the `λ₀·λ₁` product it
/// is added to, because it needs `(∂ρ/∂p)_T` at the state *and* the
/// reference-temperature polynomial on top of a full property evaluation.
/// Benchmarking only a far-from-critical point would hide that entirely — and
/// it is exactly why the batch kernel is a separate call from `properties`.
fn bench_transport(c: &mut Criterion) {
    let mut g = c.benchmark_group("transport");
    for (label, t, p) in [
        ("viscosity_liquid", 293.15, 101.325),
        ("viscosity_vapor", 873.15, 1_000.0),
    ] {
        g.bench_function(label, |b| {
            b.iter(|| vle_steam::viscosity(black_box(t), black_box(p)).unwrap())
        });
    }
    for (label, t, p) in [
        // Region 1, far from critical: Δχ ≤ 0, so λ₂ short-circuits to zero.
        ("conductivity_liquid", 293.15, 101.325),
        // Region 2 dilute vapor — enhancement present but negligible.
        ("conductivity_vapor", 873.15, 1_000.0),
        // Region 1 near the dome, where λ₂ is ~2.6% of λ and fully evaluated.
        ("conductivity_enhanced", 620.0, 20_000.0),
        // Region 3 just above Tc — the most expensive path in the crate.
        ("conductivity_near_critical", 647.35, 22_000.0),
    ] {
        g.bench_function(label, |b| {
            b.iter(|| vle_steam::thermal_conductivity(black_box(t), black_box(p)).unwrap())
        });
    }
    g.bench_function("surface_tension", |b| {
        b.iter(|| vle_steam::surface_tension(black_box(400.0)).unwrap())
    });
    g.bench_function("prandtl", |b| {
        b.iter(|| {
            SteamState::tp(black_box(293.15), black_box(101.325))
                .unwrap()
                .prandtl()
                .unwrap()
        })
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_region,
    bench_boundary,
    bench_saturation,
    bench_inverse,
    bench_sweep,
    bench_transport
);
criterion_main!(benches);
