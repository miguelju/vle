#!/usr/bin/env python
"""Batch numpy API benchmark — Milestone 10 (PERFORMANCE_PROPOSAL.md Track D).

Measures what the batch API buys over the scalar bindings it wraps: paying
the Python↔Rust boundary once per *array* instead of once per *state point*,
releasing the GIL, and fanning out across cores with rayon. Reported as
ns/point and speedup vs a tight Python scalar loop over the same points.

This is the M10 counterpart to ``bench_ffi_boundary.py`` (which measured the
scalar per-call floor the batch API is compared against). Informational only
— never fails the build.

Run inside the ``vle`` conda environment after ``maturin develop``:

    ~/miniconda3/envs/vle/bin/python scripts/bench_batch_api.py
"""

from __future__ import annotations

import time

import numpy as np

from vle import System

M = 20_000  # state points per sweep
REPEATS = 5


def best_ns_per_point(fn, m: int = M) -> float:
    """Best-of-REPEATS wall time of ``fn`` in nanoseconds per state point."""
    samples = []
    for _ in range(REPEATS):
        t0 = time.perf_counter()
        fn()
        samples.append((time.perf_counter() - t0) / m * 1e9)
    return min(samples)


def main() -> None:
    import vle

    print(f"Batch API benchmark — {M} points/sweep, best of {REPEATS}")
    print(f"engine version: {vle.__version__}   cores: {__import__('os').cpu_count()}")
    print("-" * 78)

    sys = System(["n-heptane", "n-butane"], eos="RKS")
    z = [0.5, 0.5]
    ts = np.linspace(290.0, 340.0, M)
    ps = np.full_like(ts, 100.0)
    x = [0.5, 0.5]

    # ── Z-factor: cheap property, dominated by boundary cost when scalar ──
    def z_scalar():
        f = sys.z_factor
        for t, p in zip(ts, ps):
            f(float(t), float(p), x, "vapor")

    z_sc = best_ns_per_point(z_scalar)
    z_bp = best_ns_per_point(lambda: sys.z_factor_batch(ts, ps, x, "vapor", parallel=True))
    z_bs = best_ns_per_point(lambda: sys.z_factor_batch(ts, ps, x, "vapor", parallel=False))
    print(f"z_factor  scalar loop      {z_sc:10.1f} ns/pt")
    print(f"z_factor  batch (serial)   {z_bs:10.1f} ns/pt   {z_sc / z_bs:6.1f}× vs scalar")
    print(f"z_factor  batch (parallel) {z_bp:10.1f} ns/pt   {z_sc / z_bp:6.1f}× vs scalar")
    print("-" * 78)

    # ── Isothermal flash: heavy kernel, real parallel + warm-start payoff ──
    def flash_scalar():
        f = sys.flash_pt
        for t, p in zip(ts, ps):
            f(float(t), float(p), z)

    f_sc = best_ns_per_point(flash_scalar)
    f_bs = best_ns_per_point(lambda: sys.flash_pt_batch(ts, ps, z, parallel=False, warm_start=False))
    f_bw = best_ns_per_point(lambda: sys.flash_pt_batch(ts, ps, z, parallel=False, warm_start=True))
    f_bp = best_ns_per_point(lambda: sys.flash_pt_batch(ts, ps, z, parallel=True, warm_start=True))
    print(f"flash_pt  scalar loop           {f_sc:10.1f} ns/pt")
    print(f"flash_pt  batch (serial, cold)  {f_bs:10.1f} ns/pt   {f_sc / f_bs:6.1f}× vs scalar")
    print(f"flash_pt  batch (serial, warm)  {f_bw:10.1f} ns/pt   {f_sc / f_bw:6.1f}× vs scalar")
    print(f"flash_pt  batch (parallel,warm) {f_bp:10.1f} ns/pt   {f_sc / f_bp:6.1f}× vs scalar")
    print("-" * 78)

    # Iteration savings from warm-start on the smooth sweep.
    cold = sys.flash_pt_batch(ts, ps, z, parallel=False, warm_start=False)
    warm = sys.flash_pt_batch(ts, ps, z, parallel=False, warm_start=True)
    print(f"warm-start iteration savings: {cold.iterations.sum()} → {warm.iterations.sum()} "
          f"({100 * (1 - warm.iterations.sum() / cold.iterations.sum()):.1f}% fewer)")


if __name__ == "__main__":
    main()
