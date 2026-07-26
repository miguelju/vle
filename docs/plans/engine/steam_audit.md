# `vle-steam` performance audit

Date: 2026-07-25  
Benchmark: `cargo bench -p vle-steam --bench steam_bench` (Criterion 100-sample estimates)  
Correctness: the existing 34 tests plus one new six-point R7-97 region-2 backward-equation test.

## Untouched baseline

The baseline was saved before source edits with:

```text
cargo bench -p vle-steam --bench steam_bench -- --save-baseline before
```

Values below are Criterion point estimates (the center of the reported confidence interval).

| Benchmark | Baseline |
|---|---:|
| region/r1_cold | 254.95 ns |
| region/r1_warm_highp | 258.91 ns |
| region/r1_near_boundary | 262.76 ns |
| region/r2_low_p | 361.17 ns |
| region/r2_moderate | 366.52 ns |
| region/r2_near_b23 | 363.39 ns |
| region/r3_near_critical | 4.4954 µs |
| region/r3_dense | 5.0997 µs |
| region/r3_hot | 4.3723 µs |
| region/r5_high_t | 19.509 ns |
| region/r5_high_t_high_p | 19.510 ns |
| boundary/region_of_r1 | 3.3104 ns |
| boundary/region_of_r2 | 1.7315 ns |
| boundary/region_of_r3 | 1.7383 ns |
| boundary/region_of_r5 | 1.4918 ns |
| boundary/b23_p | 415.14 ps |
| boundary/b23_t | 785.52 ps |
| saturation/psat_400k | 2.2230 ns |
| saturation/psat_600k | 2.2213 ns |
| saturation/tsat_100kpa | 9.2617 ns |
| saturation/tsat_10mpa | 9.2479 ns |
| saturation/psat_derivative | 4.6883 ns |
| saturation/sat_t | 668.56 ns |
| saturation/sat_p_1bar | 705.46 ns |
| saturation/sat_p_10mpa | 702.36 ns |
| saturation/latent_heat | 826.21 ns |
| inverse/ph_two_phase | 690.34 ns |
| inverse/ph_liquid | 1.7921 µs |
| inverse/ph_vapor | 20.038 µs |
| inverse/ps_two_phase | 684.60 ns |
| inverse/ps_liquid | 1.8230 µs |
| inverse/ps_vapor | 5.2007 µs |
| inverse/tx | 667.92 ns |
| inverse/px | 692.36 ns |
| sweep/region1_200pts | 54.191 µs |
| sweep/region2_200pts | 76.826 µs |
| sweep/region3_50pts | 241.79 µs |
| sweep/sat_table_100rows | 68.258 µs |
| sweep/ph_flash_200pts | 333.53 µs |

## Where the time was

`inverse/ph_vapor` was the dominant single inverse benchmark at 20.038 µs:
11.2 times `ph_liquid` and 3.85 times `ps_vapor`. Inspection matched the
measurement: region-2 PH flashes fell through to a bracketed Brent solve and
therefore evaluated the 52-term forward region-2 surface repeatedly.

Candidates ranked by measured-path payoff:

1. Region-2 backward `T(p,h)` (2a/2b/2c), targeting the 20.038 µs hotspot.
2. Avoid duplicate final forward evaluations in PH Newton paths, targeting both
   PH single-phase cases and the 333.53 µs mixed sweep.
3. Region-3 backward `T(p,h)`. Region-3 forward calls cost 4.37–5.10 µs, but the
   supplied PH benchmarks contain no region-3 point (the PH sweep is at 1 MPa),
   so this had lower benchmark-supported payoff than the two changes above.
4. Boundary and saturation primitives. Their measured nanosecond costs made
   them poor candidates for this audit.

## Accepted change 1: IF97 region-2 backward `T(p,h)`

Implemented the official 2a/2b/2c coefficient tables and subregion dispatch,
then retained a forward-equation Newton polish using `dh/dT|p = cp`. Thus the
backward polynomial supplies speed while the existing forward equation remains
the final accuracy authority. Added the six R7-97 Tables 21–23 verification
points; all 35 tests passed.

| Benchmark | Before | After | Delta |
|---|---:|---:|---:|
| inverse/ph_vapor | 20.038 µs | 2.2586 µs | **−88.67%** |
| sweep/ph_flash_200pts | 333.53 µs | 281.53 µs | **−15.80%** |
| inverse/ph_liquid (control) | 1.7921 µs | 1.8249 µs | +2.07% |

The full comparison showed measurement drift: untouched region-3 forward cases
moved about +9%, while most saturation primitives stayed within 1% and other
controls were mixed. The 88.67% PH-vapor reduction is far beyond that drift.
No unrelated benchmark movement is claimed as an optimization.

## Accepted change 2: reuse converged forward properties

Changed `t_from_ph` to return the already-computed `Props` at Newton
convergence, avoiding one duplicate region forward evaluation during
`SteamState` assembly. Tests remained green.

| Benchmark | Before (change 1) | After | Incremental delta | Versus original |
|---|---:|---:|---:|---:|
| inverse/ph_liquid | 1.8249 µs | 1.5673 µs | **−14.1%** | −12.5% |
| inverse/ph_vapor | 2.2586 µs | 1.9679 µs | **−12.9%** | **−90.18%** |
| sweep/ph_flash_200pts | 281.53 µs | 221.48 µs | **−21.3%** | **−33.60%** |

## Rejected changes

### Remove converged-point region validation

Tested removing the final `region_of` checks from the region-1 and region-2
Newton convergence branches. Correctness stayed green, but a back-to-back
measurement against the immediately preceding retained build was mixed and
below the 2–3% noise threshold:

| Benchmark | Retained build | Experiment | Delta |
|---|---:|---:|---:|
| inverse/ph_liquid | 1.5673 µs | 1.5371 µs | −1.93% |
| inverse/ph_vapor | 1.9679 µs | 1.9752 µs | +0.37% |
| sweep/ph_flash_200pts | 221.48 µs | 218.28 µs | −1.44% |

This was rejected and reverted: there was no measurable overall improvement,
and retaining the checks preserves a useful boundary guard.

## Region-3 backward-equation decision

Region-3 backward `T(p,h)` was investigated but not implemented in this pass.
The current harness measures expensive region-3 forward `(T,p)` density solves
(4.37–5.10 µs each), but it does not contain a region-3 PH flash, and its
1-MPa PH sweep cannot enter region 3. Implementing the separate supplementary
region-3 backward release without a before/after workload would violate the
measurement-first rule. A future audit should first add representative
region-3 PH points and save a new untouched baseline; only then should the
larger region-3 coefficient implementation be judged.

## Final result

The retained changes reduce the principal hotspot, `inverse/ph_vapor`, from
20.038 µs to 1.9679 µs (about **10.18× faster**, **−90.18%**) and the mixed
200-point PH sweep from 333.53 µs to 221.48 µs (**−33.60%**), while retaining
forward-equation accuracy and all R7-97 acceptance tests.

