#!/usr/bin/env python
"""FFI boundary benchmark — Milestone 8.2 (PERFORMANCE_PROPOSAL.md Track E).

Times the *per-call overhead* of crossing the Python -> Rust boundary with
the current scalar bindings. This number is the baseline the Milestone 10
batch numpy API is measured against: the batch API's whole point is to pay
the boundary cost once per *array* instead of once per *state point*.

What is measured
----------------
For each representative binding we time a tight scalar loop over N state
points and report ns/call. We also time the cheapest possible no-op-ish
binding (``version()``) to isolate pure FFI dispatch cost from the
thermodynamic math, and a pure-Python arithmetic loop as the floor.

Run inside the ``vle`` conda environment, after ``maturin develop``:

    ~/miniconda3/envs/vle/bin/python scripts/bench_ffi_boundary.py

Output is plain text, one line per benchmark — commit-friendly for pasting
into PERFORMANCE_PROPOSAL.md progress notes or CI logs. This script never
fails the build: it is informational only.
"""

from __future__ import annotations

import statistics
import time

from vle import _engine

N_CALLS = 100_000
REPEATS = 5

# n-pentane constants used across the engine test-suite.
TC, PC, OMEGA = 469.7, 3370.0, 0.252


def bench(label: str, fn) -> float:
    """Time ``fn`` (a zero-arg callable running N_CALLS iterations) REPEATS
    times and report the best ns/call (min over repeats — standard practice
    for microbenchmarks: the minimum is the least noise-contaminated)."""
    samples = []
    for _ in range(REPEATS):
        t0 = time.perf_counter()
        fn()
        samples.append((time.perf_counter() - t0) / N_CALLS * 1e9)
    best = min(samples)
    spread = statistics.pstdev(samples)
    print(f"{label:<38} {best:10.1f} ns/call   (±{spread:.1f} over {REPEATS} runs)")
    return best


def loop_python_floor():
    # Pure-Python arithmetic floor: what a loop iteration costs with no FFI.
    acc = 0.0
    for i in range(N_CALLS):
        acc += 0.85 * i
    return acc


def loop_version():
    f = _engine.version
    for _ in range(N_CALLS):
        f()


def loop_alpha():
    f = _engine.eos_alpha
    eos = _engine.CubicEos.PR1976
    for _ in range(N_CALLS):
        f(eos, 0.85, OMEGA)


def loop_z_factor():
    f = _engine.eos_z_factor
    eos = _engine.CubicEos.PR1976
    for _ in range(N_CALLS):
        f(eos, 400.0, 1500.0, TC, PC, OMEGA, "vapor")


def loop_ln_phi():
    f = _engine.eos_ln_phi_pure
    eos = _engine.CubicEos.PR1976
    for _ in range(N_CALLS):
        f(eos, 400.0, 1500.0, TC, PC, OMEGA, "vapor")


def main() -> None:
    print(f"FFI boundary benchmark — {N_CALLS} calls/loop, best of {REPEATS}")
    print(f"engine version: {_engine.version()}")
    print("-" * 78)
    floor = bench("pure-python loop (floor)", loop_python_floor)
    version = bench("_engine.version() (FFI dispatch)", loop_version)
    alpha = bench("eos_alpha (PR1976)", loop_alpha)
    z = bench("eos_z_factor (PR1976, vapor)", loop_z_factor)
    lnphi = bench("eos_ln_phi_pure (PR1976, vapor)", loop_ln_phi)
    print("-" * 78)
    # The interpretation line the M10 batch API cares about: how much of a
    # scalar z_factor call is boundary overhead vs actual math. criterion
    # measures the pure-Rust z_factor cost; the difference is the FFI tax.
    print(f"FFI dispatch overhead (version - python floor): {version - floor:8.1f} ns")
    print(f"z_factor scalar call:                           {z:8.1f} ns")
    print(f"ln_phi scalar call:                             {lnphi:8.1f} ns")
    print(f"alpha scalar call:                              {alpha:8.1f} ns")


if __name__ == "__main__":
    main()
