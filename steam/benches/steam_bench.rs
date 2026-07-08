//! Criterion benchmarks guarding the IAPWS-IF97 hot paths.
//!
//! One `(T, P)` state evaluation per region (region 3 pays the density
//! iteration; regions 1/2/5 are explicit) plus a two-phase PH flash and a
//! saturation-row query — the same regression-bench pattern the `vle-thermo`
//! engine uses. Run with `cargo bench -p vle-steam`.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vle_steam::{SteamState, sat_p};

fn bench_regions(c: &mut Criterion) {
    // Region 1 — compressed liquid (300 K, 3 MPa).
    c.bench_function("tp_region1", |b| {
        b.iter(|| SteamState::tp(black_box(300.0), black_box(3_000.0)).unwrap())
    });
    // Region 2 — superheated vapor (700 K, 3.5 kPa).
    c.bench_function("tp_region2", |b| {
        b.iter(|| SteamState::tp(black_box(700.0), black_box(3.5)).unwrap())
    });
    // Region 3 — near-critical (650 K, 25 MPa) — pays the Brent density solve.
    c.bench_function("tp_region3", |b| {
        b.iter(|| SteamState::tp(black_box(650.0), black_box(25_000.0)).unwrap())
    });
    // Region 5 — high-T steam (1500 K, 0.5 MPa).
    c.bench_function("tp_region5", |b| {
        b.iter(|| SteamState::tp(black_box(1500.0), black_box(500.0)).unwrap())
    });
}

fn bench_flash_and_sat(c: &mut Criterion) {
    // Two-phase PH flash at 1 MPa, h between h_f and h_g.
    c.bench_function("ph_flash_two_phase", |b| {
        b.iter(|| SteamState::ph(black_box(1_000.0), black_box(2_000.0)).unwrap())
    });
    // Single-phase PH flash — region-1 seed + Newton polish.
    c.bench_function("ph_flash_liquid", |b| {
        b.iter(|| SteamState::ph(black_box(5_000.0), black_box(500.0)).unwrap())
    });
    // Saturation-table row at 1 bar.
    c.bench_function("sat_p", |b| b.iter(|| sat_p(black_box(100.0)).unwrap()));
}

criterion_group!(benches, bench_regions, bench_flash_and_sat);
criterion_main!(benches);
