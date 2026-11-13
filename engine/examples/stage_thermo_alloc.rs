//! Heap traffic and wall time of the stage-thermodynamics call against
//! mixture size — the measurement behind `OPTIMIZATION_PLAN_PART2.md` §11
//! (should the dual path get a caller-owned workspace?).
//!
//! For each component count it prints, per `stage_thermo_into` call: the
//! number of heap allocations, the bytes requested, and the mean wall time.
//! The mixture buffers are `SmallVec<[D; 8]>`, so up to 8 components nothing
//! should touch the heap; above that every buffer spills.
//!
//! Run (release, it is a timing harness):
//! ```text
//! cargo run --release --example stage_thermo_alloc
//! cargo run --release --example stage_thermo_alloc -- --spin 40   # loop n = 40 for a profiler
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use vle_thermo::eos::{CubicEos, LiquidModel, VaporModel};
use vle_thermo::flash::{SystemSpec, stage_thermo_into};
use vle_thermo::mixing::MixingRule;
use vle_thermo::types::Component;

/// A global allocator that forwards to the system one and counts calls.
///
/// `#[global_allocator]` swaps the allocator for the whole program, which is
/// the only way to see allocations made deep inside the library.
struct Counting;
static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

// SAFETY: every method forwards unchanged to `System`, which upholds the
// `GlobalAlloc` contract; the counters are plain atomics.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// `n` synthetic n-alkane-like components (carbon numbers 3, 4, …).
///
/// Critical constants follow smooth trends in carbon number — close enough
/// to real alkanes that both cubic roots exist at the test state, and it
/// lets the harness go to any `n` without a database.
fn components(n: usize) -> Vec<Component> {
    (0..n)
        .map(|i| {
            let c = 3.0 + i as f64 * (if n > 20 { 20.0 / n as f64 } else { 1.0 });
            Component {
                name: format!("C{c:.2}"),
                tc: 190.0 + 175.0 * c.ln(),    // K
                pc: 5200.0 / (1.0 + 0.11 * c), // kPa
                omega: 0.05 + 0.045 * c,
                mw: 14.0 * c + 2.0,
                cp_coeffs: [2.0 + c, 2.0e-2 + 5.0e-3 * c, -1.0e-5, 0.0, 0.0],
                ..Component::default()
            }
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spin: Option<usize> = args
        .iter()
        .position(|a| a == "--spin")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok());

    let sizes: Vec<usize> = match spin {
        Some(n) => vec![n],
        None => vec![2, 4, 8, 9, 10, 16, 20, 40, 100, 300],
    };
    if spin.is_none() {
        println!(
            "{:>5} {:>12} {:>12} {:>12} {:>14}",
            "n", "allocs/call", "bytes/call", "ns/call", "ns/call/n"
        );
    }
    for n in sizes {
        let comps = components(n);
        let sys = SystemSpec {
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
        let norm = |v: Vec<f64>| -> Vec<f64> {
            let s: f64 = v.iter().sum();
            v.into_iter().map(|a| a / s).collect()
        };
        let x = norm((0..n).map(|i| 1.0 + i as f64).collect());
        let y = norm((0..n).map(|i| (n - i) as f64).collect());
        let (t, p, t_ref) = (400.0, 1000.0, 298.15); // K, kPa, K
        let mut k = vec![0.0; n];
        let mut dk = vec![0.0; n];
        let mut call = || {
            black_box(
                stage_thermo_into(black_box(&sys), t, p, &x, &y, t_ref, &[], &mut k, &mut dk)
                    .expect("stage thermo"),
            )
        };

        if spin.is_some() {
            // Loop for ~6 s so an external sampler has something to look at.
            let start = Instant::now();
            while start.elapsed().as_secs_f64() < 6.0 {
                for _ in 0..1000 {
                    call();
                }
            }
            return;
        }

        // Scale the call count so every size runs for a comparable time.
        let calls = (2_000_000 / (n * n).max(16)).max(200);
        for _ in 0..calls / 10 + 1 {
            call();
        }
        let (a0, b0) = (
            ALLOCS.load(Ordering::Relaxed),
            BYTES.load(Ordering::Relaxed),
        );
        let start = Instant::now();
        for _ in 0..calls {
            call();
        }
        let ns = start.elapsed().as_secs_f64() * 1e9 / calls as f64;
        let allocs = (ALLOCS.load(Ordering::Relaxed) - a0) as f64 / calls as f64;
        let bytes = (BYTES.load(Ordering::Relaxed) - b0) as f64 / calls as f64;
        println!(
            "{n:>5} {allocs:>12.1} {bytes:>12.0} {ns:>12.0} {:>14.1}",
            ns / n as f64
        );
    }
}
