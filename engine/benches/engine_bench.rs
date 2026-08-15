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
//! - `flash`      — Rachford-Rice + a binary isothermal flash (M9)
//! - `flash_multi` — multicomponent flash scaling at n = 2, 4, 6, 8, with
//!   the inner RR solve, one K-value evaluation, the whole driver, and the
//!   TPD stability test measured separately (the Part 1 audit baseline)
//! - `mixture`    — the mixture core opened up: `mixture_params` (pure +
//!   mixing rule) vs `z_mix` (+ cubic solve) vs `ln_phi_mix` (+ fugacity),
//!   plus the composition-Jacobian, activity-model, and virial paths
//!   (the Part 2 audit baseline)
//! - `derivatives` — dual T/P-derivative and Cp paths (M12.3/M12.4)
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
                &[],
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
                &[],
                black_box(&vl),
                &[],
                black_box(340.0),
            )
        })
    });
    g.finish();
}

/// Rachford-Rice inner solve (§F) and the full isothermal flash (§J) —
/// added in Milestone 9. The RR bench isolates the scalar Halley loop; the
/// flash bench measures the whole Wilson-init + GDEM-SS convergence for a
/// two-phase binary.
fn bench_flash(c: &mut Criterion) {
    use vle_thermo::eos::{LiquidModel, VaporModel};
    use vle_thermo::flash::SystemSpec;
    use vle_thermo::flash::isothermal::{flash_isothermal, rachford_rice};
    use vle_thermo::mixing::MixingRule;

    let mut g = c.benchmark_group("flash");
    g.bench_function("rachford_rice_ternary", |b| {
        let z = [0.3, 0.4, 0.3];
        let k = [3.0, 1.2, 0.4];
        b.iter(|| rachford_rice(black_box(&z), black_box(&k), 1e-12, 100).unwrap())
    });

    let comps = [
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            ..Component::default()
        },
        Component {
            name: "n-heptane".into(),
            tc: 540.2,
            pc: 2740.0,
            omega: 0.350,
            ..Component::default()
        },
    ];
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
    g.bench_function("isothermal_flash_rks_binary", |b| {
        b.iter(|| {
            flash_isothermal(
                black_box(&spec),
                black_box(420.0),
                black_box(1000.0),
                black_box(&[0.5, 0.5]),
                1e-10,
                200,
            )
            .unwrap()
        })
    });
    g.finish();
}

// ===========================================================================
// Multicomponent flash scaling (Part 1 of optimizations_audit.md).
//
// The audit's central claim is that Rachford-Rice is *not* the flash
// bottleneck — the surrounding thermodynamics (K-value evaluation, i.e.
// mixture fugacity) is. These benches make that testable rather than
// asserted: for the same mixture at the same state, `rachford_rice_n*`
// isolates the scalar inner solve, `k_values_n*` isolates one outer-loop
// thermodynamic evaluation, and `isothermal_n*` measures the whole driver.
// Comparing the three across n = 2, 4, 6, 8 shows where the time actually
// goes and how each layer scales with component count.
// ===========================================================================

/// Eight n-alkanes spanning methane → n-octane, with critical constants from
/// the standard tables and reduced-Antoine `ln(P/Pc) = a1 − a2/(a3+T)` fits.
/// Subsets of this list drive every scaling bench below.
fn alkane_series() -> Vec<Component> {
    fn c(name: &str, tc: f64, pc: f64, omega: f64, a: [f64; 3], vl: f64) -> Component {
        Component {
            name: name.into(),
            tc,
            pc,
            omega,
            psat_coeffs: a.to_vec(),
            liquid_volume: vl,
            ..Component::default()
        }
    }
    vec![
        c("methane", 190.56, 4599.0, 0.011, [4.20, 900.0, -8.0], 37.9),
        c("ethane", 305.32, 4872.0, 0.099, [4.30, 1400.0, -15.0], 55.2),
        c(
            "propane",
            369.83,
            4248.0,
            0.152,
            [4.35, 1850.0, -25.0],
            74.9,
        ),
        c(
            "n-butane",
            425.12,
            3796.0,
            0.200,
            [4.35, 2277.0, -30.0],
            100.4,
        ),
        c(
            "n-pentane",
            469.7,
            3370.0,
            0.252,
            [4.30, 2600.0, -40.0],
            116.1,
        ),
        c(
            "n-hexane",
            507.6,
            3025.0,
            0.301,
            [4.15, 2800.0, -48.0],
            131.6,
        ),
        c(
            "n-heptane",
            540.2,
            2740.0,
            0.350,
            [4.02, 2911.0, -56.0],
            147.5,
        ),
        c(
            "n-octane",
            568.7,
            2490.0,
            0.399,
            [3.98, 3120.0, -63.0],
            163.5,
        ),
    ]
}

/// An `n`-component subset of [`alkane_series`], evenly spread light→heavy so
/// every size spans a realistic volatility range (a subset of only the light
/// end would flash to a single phase).
fn alkane_subset(n: usize) -> Vec<Component> {
    let all = alkane_series();
    (0..n).map(|i| all[i * (8 / n)].clone()).collect()
}

/// The scaling series' state point: 350 K / 2000 kPa is genuinely two-phase
/// for n = 2, 4, 6 and 8 (β ≈ 0.60, 0.35, 0.45, 0.21), so every size measures
/// the same *kind* of work — a converging two-phase split, not an early
/// single-phase bail-out.
const FLASH_T: f64 = 350.0;
const FLASH_P: f64 = 2000.0;

fn bench_flash_multi(c: &mut Criterion) {
    use vle_thermo::eos::{LiquidModel, VaporModel};
    use vle_thermo::flash::SystemSpec;
    use vle_thermo::flash::isothermal::{flash_isothermal, rachford_rice};
    use vle_thermo::flash::stability::stability_analysis;
    use vle_thermo::flash::{init::wilson_k_values, k_values};
    use vle_thermo::mixing::MixingRule;

    let mut g = c.benchmark_group("flash_multi");

    for n in [2usize, 4, 6, 8] {
        let comps = alkane_subset(n);
        let z: Vec<f64> = vec![1.0 / n as f64; n];
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

        // --- The scalar inner solve, in isolation. ---
        let k_wilson = wilson_k_values(&comps, FLASH_T, FLASH_P);
        g.bench_function(format!("rachford_rice_n{n}"), |b| {
            b.iter(|| rachford_rice(black_box(&z), black_box(&k_wilson), 1e-12, 200).unwrap())
        });

        // --- One outer-iteration thermodynamic evaluation, in isolation.
        // Split the feed at the Wilson K to get a representative (x, y).
        let beta = rachford_rice(&z, &k_wilson, 1e-12, 200).unwrap();
        let x: Vec<f64> = (0..n)
            .map(|i| z[i] / (1.0 + beta * (k_wilson[i] - 1.0)))
            .collect();
        let y: Vec<f64> = (0..n).map(|i| k_wilson[i] * x[i]).collect();
        g.bench_function(format!("k_values_n{n}"), |b| {
            b.iter(|| {
                k_values(
                    black_box(&spec),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    black_box(&y),
                )
                .unwrap()
            })
        });

        // --- The whole driver: Wilson init → RR → split → K → GDEM/SS. ---
        g.bench_function(format!("isothermal_n{n}"), |b| {
            b.iter(|| {
                flash_isothermal(
                    black_box(&spec),
                    black_box(FLASH_T),
                    black_box(FLASH_P),
                    black_box(&z),
                    1e-10,
                    300,
                )
                .unwrap()
            })
        });

        // --- TPD stability analysis (two Wilson-seeded trial phases). ---
        g.bench_function(format!("stability_n{n}"), |b| {
            b.iter(|| {
                stability_analysis(
                    black_box(&spec),
                    black_box(FLASH_T),
                    black_box(FLASH_P),
                    black_box(&z),
                    100,
                )
                .unwrap()
            })
        });
    }

    // --- γ-φ path: Wilson-activity liquid + ideal vapor, 4 components.
    // Exercises the modified-Raoult K assembly (Psat + φˢᵃᵗ + Poynting)
    // rather than the φ-φ mixture-fugacity path.
    {
        use vle_thermo::activity::ActivityModel;
        let comps = alkane_subset(4);
        let n = 4;
        let z: Vec<f64> = vec![0.25; n];
        // Mild non-ideality — enough that γ ≠ 1 without breaking convergence.
        let aij: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            0.0
                        } else {
                            120.0 * (j as f64 - i as f64)
                        }
                    })
                    .collect()
            })
            .collect();
        let vl: Vec<f64> = comps.iter().map(|c| c.liquid_volume).collect();
        let spec = SystemSpec {
            components: &comps,
            vapor: VaporModel::IdealGas,
            liquid: LiquidModel::Activity(ActivityModel::Wilson),
            mixing_rule: MixingRule::Classical,
            kij: &[],
            aij: &aij,
            alpha: &[],
            vl: &vl,
            delta: &[],
            sat_models: &[],
            ge_model: None,
        };
        g.bench_function("k_values_gamma_phi_n4", |b| {
            b.iter(|| {
                k_values(
                    black_box(&spec),
                    FLASH_T,
                    black_box(300.0),
                    black_box(&z),
                    black_box(&z),
                )
                .unwrap()
            })
        });
    }

    g.finish();
}

// ===========================================================================
// Mixture-core scaling (Part 2 of optimizations_audit.md).
//
// Part 1 established that ~70 % of an isothermal flash is the K-value
// evaluation, i.e. two `ln_phi_mix` calls. This group opens that box:
// `mixture_params` isolates the pure-component + mixing-rule half from the
// cubic-solve + fugacity half, so the split between "work that depends only
// on (T, P)" and "work that depends on composition" is visible. That split is
// the whole premise of audit Part 2 §1.
// ===========================================================================

fn bench_mixture_core(c: &mut Criterion) {
    use vle_thermo::activity::{ActivityModel, ln_gamma_all};
    use vle_thermo::mixing::MixingRule;
    use vle_thermo::mixture::{
        GeSpec, MixtureSpec, d_ln_phi_d_n, ln_phi_mix, mixture_params, z_mix,
    };

    let mut g = c.benchmark_group("mixture");

    for n in [2usize, 4, 8] {
        let comps = alkane_subset(n);
        let x: Vec<f64> = vec![1.0 / n as f64; n];
        // A realistic dense kij — the `Vec<Vec<f64>>` layout audit Part 2 §3
        // targets. An empty matrix would short-circuit the lookup and hide it.
        let kij: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            0.0
                        } else {
                            0.01 * (i as f64 - j as f64).abs()
                        }
                    })
                    .collect()
            })
            .collect();
        let spec = MixtureSpec {
            eos: CubicEos::RKS1972,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &kij,
            ge: None,
        };

        // Pure params + mixing rule only — no cubic solve, no fugacity.
        g.bench_function(format!("mixture_params_n{n}"), |b| {
            b.iter(|| {
                mixture_params::<f64>(black_box(&spec), FLASH_T, FLASH_P, black_box(&x)).unwrap()
            })
        });
        // + the cubic solve.
        g.bench_function(format!("z_mix_n{n}"), |b| {
            b.iter(|| {
                z_mix(
                    black_box(&spec),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });
        // + the full fugacity assembly (this is what `k_values` calls twice).
        g.bench_function(format!("ln_phi_mix_n{n}"), |b| {
            b.iter(|| {
                ln_phi_mix(
                    black_box(&spec),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });
    }

    // --- Composition Jacobian: the analytic classical path vs the
    // dual-number path the exotic GE rules take (audit Part 2 §6). ---
    {
        let n = 4;
        let comps = alkane_subset(n);
        let x: Vec<f64> = vec![0.25; n];
        let classical = MixtureSpec {
            eos: CubicEos::RKS1972,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &[],
            ge: None,
        };
        let aij: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            0.0
                        } else {
                            120.0 * (j as f64 - i as f64)
                        }
                    })
                    .collect()
            })
            .collect();
        let vl: Vec<f64> = comps.iter().map(|c| c.liquid_volume).collect();
        let ws = MixtureSpec {
            eos: CubicEos::RKS1972,
            rule: MixingRule::WongSandler,
            components: &comps,
            kij: &[],
            ge: Some(GeSpec {
                model: ActivityModel::Wilson,
                aij: &aij,
                alpha: &[],
                vl: &vl,
                delta: &[],
            }),
        };
        g.bench_function("d_ln_phi_d_n_classical_n4", |b| {
            b.iter(|| {
                d_ln_phi_d_n(
                    black_box(&classical),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });
        g.bench_function("d_ln_phi_d_n_wong_sandler_n4", |b| {
            b.iter(|| {
                d_ln_phi_d_n(
                    black_box(&ws),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });

        // --- Activity models: Wilson rebuilds (and heap-allocates) its Λ
        // matrix per call; NRTL takes the per-component O(n³) path. Both are
        // audit Part 2 §5. ---
        let alpha: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..n).map(|j| if i == j { 0.0 } else { 0.3 }).collect())
            .collect();
        let mut out = vec![0.0; n];
        g.bench_function("ln_gamma_all_wilson_n4", |b| {
            b.iter(|| {
                ln_gamma_all(
                    ActivityModel::Wilson,
                    black_box(&x),
                    black_box(&aij),
                    &[],
                    black_box(&vl),
                    &[],
                    FLASH_T,
                    black_box(&mut out),
                )
            })
        });
        g.bench_function("ln_gamma_all_nrtl_n4", |b| {
            b.iter(|| {
                ln_gamma_all(
                    ActivityModel::Nrtl,
                    black_box(&x),
                    black_box(&aij),
                    black_box(&alpha),
                    black_box(&vl),
                    &[],
                    FLASH_T,
                    black_box(&mut out),
                )
            })
        });
    }

    // --- Virial mixture fugacity: rebuilds a `Vec<Vec<f64>>` Bᵢⱼ per call
    // (audit Part 2 §9). ---
    {
        let comps = alkane_subset(4);
        let x = [0.25, 0.25, 0.25, 0.25];
        g.bench_function("virial_ln_phi_mix_n4", |b| {
            b.iter(|| {
                vle_thermo::virial::ln_phi_mix_virial(
                    black_box(&comps),
                    black_box(&x),
                    FLASH_T,
                    black_box(200.0),
                )
                .unwrap()
            })
        });
    }

    g.finish();
}

/// M12.3/M12.4 derivative + energy benches: the dual T/P-derivative and
/// second-order Cp paths vs the plain value calls (targets in
/// DERIVATIVE_RELEASE_PLAN.md §M12.5 — `k_values_with_derivs` ≤ a small
/// multiple of `k_values`).
fn bench_derivatives(c: &mut Criterion) {
    use vle_thermo::energy::phase_cp;
    use vle_thermo::eos::{LiquidModel, VaporModel};
    use vle_thermo::flash::{SystemSpec, k_values, k_values_with_derivs};
    use vle_thermo::mixing::MixingRule;
    use vle_thermo::mixture::MixtureSpec;

    let comps = [
        Component {
            name: "n-butane".into(),
            tc: 425.12,
            pc: 3796.0,
            omega: 0.200,
            psat_coeffs: vec![4.35, 2277.0, -30.0],
            cp_coeffs: [4.5, 2.0e-2, 0.0, 0.0, 0.0],
            ..Component::default()
        },
        Component {
            name: "n-heptane".into(),
            tc: 540.2,
            pc: 2740.0,
            omega: 0.350,
            psat_coeffs: vec![4.02, 2911.0, -56.0],
            cp_coeffs: [7.0, 3.0e-2, 0.0, 0.0, 0.0],
            ..Component::default()
        },
    ];
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
    let mix = MixtureSpec {
        eos: CubicEos::RKS1972,
        rule: MixingRule::Classical,
        components: &comps,
        kij: &[],
        ge: None,
    };
    let (t, p, x, y) = (400.0, 500.0, [0.3, 0.7], [0.6, 0.4]);

    let mut g = c.benchmark_group("derivatives");
    g.bench_function("k_values_binary", |b| {
        b.iter(|| k_values(black_box(&sys), t, p, black_box(&x), black_box(&y)).unwrap())
    });
    g.bench_function("k_values_with_derivs_binary", |b| {
        b.iter(|| {
            k_values_with_derivs(black_box(&sys), t, p, black_box(&x), black_box(&y)).unwrap()
        })
    });
    g.bench_function("phase_cp_binary", |b| {
        b.iter(|| {
            phase_cp(
                black_box(&mix),
                t,
                p,
                black_box(&x),
                vle_thermo::eos::PhaseId::Vapor,
            )
            .unwrap()
        })
    });
    g.finish();
}

/// Milestone 18 — does the mixture core scale linearly in N?
///
/// This is the measurement the milestone lives or dies by. Each size is run
/// three ways so the comparison is like-for-like on identical numbers:
///
/// - `dense_zeros` — an N×N matrix of zeros, forcing the general `O(N²)`
///   double loop. This is what every size cost before Milestone 18.
/// - `factorized` — the same mixture with an **empty** `kij`, taking the
///   `O(N)` collapse.
/// - `sparse` — a realistic assay pattern (three light gases interacting with
///   everything, the rest structurally zero) through the cached `O(N + nnz)`
///   path.
///
/// Read it as a ratio across N, not as absolute times: `dense_zeros` should
/// grow ~N² while `factorized` grows ~N.
fn bench_mixture_scaling(c: &mut Criterion) {
    use vle_thermo::mixing::MixingRule;
    use vle_thermo::mixture::{
        MixtureSpec, MixtureWorkspace, TpCache, d_ln_phi_d_n, d_ln_phi_d_n_apply, ln_phi_mix,
        ln_phi_mix_cached_into, ln_phi_mix_cached_ws_into,
    };

    let mut g = c.benchmark_group("mixture_scaling");
    // 300 components at O(N²) is slow enough that the default 100 samples is
    // a poor trade; the variance here is tiny compared with the effect size.
    g.sample_size(20);

    for n in [10usize, 50, 100, 300] {
        // N distinct pseudo-alkanes — the shape a TBP cut produces.
        let comps: Vec<Component> = (0..n)
            .map(|i| {
                let f = 1.0 + 0.021 * i as f64;
                Component {
                    name: format!("pseudo{i}"),
                    tc: 190.0 * f,
                    pc: 4600.0 / f,
                    omega: 0.011 + 0.006 * i as f64,
                    psat_coeffs: vec![4.2, 900.0 * f, -8.0],
                    liquid_volume: 37.9 * f,
                    ..Component::default()
                }
            })
            .collect();
        let x: Vec<f64> = vec![1.0 / n as f64; n];
        let zeros: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
        // Three light gases against every hydrocarbon — a real assay's
        // sparsity, not a uniformly random fill.
        let mut sparse_kij = vec![vec![0.0; n]; n];
        for gidx in 0..3.min(n) {
            for j in 0..n {
                if gidx != j {
                    // Symmetric fill; written as explicit index pairs because
                    // the pattern *is* a matrix coordinate, not a traversal.
                    let (lo, hi) = (gidx.min(j), gidx.max(j));
                    sparse_kij[lo][hi] = 0.08;
                    sparse_kij[hi][lo] = 0.08;
                }
            }
        }

        let base = MixtureSpec {
            eos: CubicEos::PR1976,
            rule: MixingRule::Classical,
            components: &comps,
            kij: &[],
            ge: None,
        };
        let dense = MixtureSpec {
            kij: &zeros,
            ..base
        };
        let sparse = MixtureSpec {
            kij: &sparse_kij,
            ..base
        };

        g.bench_function(format!("ln_phi_dense_zeros_n{n}"), |b| {
            b.iter(|| {
                ln_phi_mix(
                    black_box(&dense),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });
        g.bench_function(format!("ln_phi_factorized_n{n}"), |b| {
            b.iter(|| {
                ln_phi_mix(
                    black_box(&base),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });

        // Cached: the column-solve pattern — build the (T, P) state and the
        // kij index once, then sweep composition.
        let cache_sparse = TpCache::new(&sparse, FLASH_T, FLASH_P).unwrap();
        let mut out = vec![0.0; n];
        g.bench_function(format!("ln_phi_cached_sparse_n{n}"), |b| {
            b.iter(|| {
                ln_phi_mix_cached_into(
                    black_box(&sparse),
                    black_box(&cache_sparse),
                    black_box(&x),
                    PhaseId::Liquid,
                    &mut out,
                )
                .unwrap()
            })
        });
        // Composition Jacobian: forming the N×N block vs applying it as a sum
        // of rank-1 terms. Forming is O(N²) in time *and* memory; applying is
        // O(N). At N = 300 forming it means 90 000 doubles a Newton step
        // never actually needs.
        let v: Vec<f64> = (0..n).map(|k| (k as f64 + 1.0).sin()).collect();
        let mut jv = vec![0.0; n];
        g.bench_function(format!("jacobian_formed_n{n}"), |b| {
            b.iter(|| {
                d_ln_phi_d_n(
                    black_box(&base),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                )
                .unwrap()
            })
        });
        g.bench_function(format!("jacobian_applied_n{n}"), |b| {
            b.iter(|| {
                d_ln_phi_d_n_apply(
                    black_box(&base),
                    FLASH_T,
                    FLASH_P,
                    black_box(&x),
                    PhaseId::Liquid,
                    black_box(&v),
                    &mut jv,
                )
                .unwrap()
            })
        });

        // Milestone 18 (U6): same cached path, caller-owned buffers. The
        // delta between these two is exactly the per-call allocation cost.
        let cache_ws = TpCache::new(&base, FLASH_T, FLASH_P).unwrap();
        let mut ws = MixtureWorkspace::new();
        g.bench_function(format!("ln_phi_cached_ws_n{n}"), |b| {
            b.iter(|| {
                ln_phi_mix_cached_ws_into(
                    black_box(&base),
                    black_box(&cache_ws),
                    &mut ws,
                    black_box(&x),
                    PhaseId::Liquid,
                    &mut out,
                )
                .unwrap()
            })
        });

        let cache_zero = TpCache::new(&base, FLASH_T, FLASH_P).unwrap();
        g.bench_function(format!("ln_phi_cached_factorized_n{n}"), |b| {
            b.iter(|| {
                ln_phi_mix_cached_into(
                    black_box(&base),
                    black_box(&cache_zero),
                    black_box(&x),
                    PhaseId::Liquid,
                    &mut out,
                )
                .unwrap()
            })
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_alpha,
    bench_z_factor,
    bench_ln_phi,
    bench_saturation,
    bench_activity,
    bench_flash,
    bench_flash_multi,
    bench_mixture_core,
    bench_mixture_scaling,
    bench_derivatives
);
criterion_main!(benches);
