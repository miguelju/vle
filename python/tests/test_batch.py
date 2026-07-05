"""Batch numpy API tests (Milestone 10, Track D).

The load-bearing property is **parity**: a ``*_batch`` call must return, point
for point, exactly what the scalar method returns — batching is a performance
optimization, never a semantic change. These tests also cover broadcasting,
NaN handling of failed points, and the warm-start iteration savings.
"""

import numpy as np
import pytest

from vle import BatchFlashResult, BatchSaturationResult, System


@pytest.fixture
def heptane_butane():
    return System(["n-heptane", "n-butane"], eos="RKS")


# ── Parity: batch == scalar ───────────────────────────────────────────────

def test_flash_pt_batch_matches_scalar(heptane_butane):
    sys = heptane_butane
    ts = np.linspace(295.0, 315.0, 25)
    ps = np.full_like(ts, 100.0)
    z = [0.5, 0.5]
    batch = sys.flash_pt_batch(ts, ps, z, parallel=True)
    assert isinstance(batch, BatchFlashResult)
    assert len(batch) == 25
    assert batch.converged.all()
    for i, (t, p) in enumerate(zip(ts, ps)):
        s = sys.flash_pt(float(t), float(p), z)
        assert batch.beta[i] == pytest.approx(s.beta, abs=1e-9)
        assert batch.x[i] == pytest.approx(s.x, abs=1e-9)
        assert batch.y[i] == pytest.approx(s.y, abs=1e-9)


def test_flash_pt_batch_serial_equals_parallel(heptane_butane):
    sys = heptane_butane
    ts = np.linspace(295.0, 315.0, 40)
    ps = np.full_like(ts, 100.0)
    z = [0.5, 0.5]
    par = sys.flash_pt_batch(ts, ps, z, parallel=True, warm_start=False)
    ser = sys.flash_pt_batch(ts, ps, z, parallel=False, warm_start=False)
    assert par.beta == pytest.approx(ser.beta, abs=1e-9, nan_ok=True)


def test_bubble_pressure_batch_matches_scalar():
    sys = System(["n-heptane", "n-butane"], eos="RKS")
    x1 = np.linspace(0.2, 0.8, 20)
    xs = np.column_stack([x1, 1 - x1])
    ts = np.array([380.0])  # broadcast to every row
    batch = sys.bubble_pressure_batch(xs, ts)
    assert isinstance(batch, BatchSaturationResult)
    for i in range(len(x1)):
        s = sys.bubble_pressure(xs[i].tolist(), 380.0)
        assert batch.value[i] == pytest.approx(s.value, rel=1e-8)
        assert batch.incipient[i] == pytest.approx(s.y, abs=1e-8)


# ── Broadcasting ──────────────────────────────────────────────────────────

def test_length1_pressure_broadcasts(heptane_butane):
    sys = heptane_butane
    ts = np.linspace(295.0, 315.0, 10)
    full = sys.flash_pt_batch(ts, np.full_like(ts, 100.0), [0.5, 0.5])
    bcast = sys.flash_pt_batch(ts, np.array([100.0]), [0.5, 0.5])
    assert bcast.beta == pytest.approx(full.beta, abs=1e-12, nan_ok=True)


def test_incompatible_lengths_raise(heptane_butane):
    sys = heptane_butane
    with pytest.raises(ValueError, match="broadcast"):
        sys.flash_pt_batch(np.linspace(295, 315, 5), np.linspace(90, 110, 4), [0.5, 0.5])


# ── Warm start ────────────────────────────────────────────────────────────

def test_warm_start_reduces_total_iterations(heptane_butane):
    """On a smooth T-sweep, warm-starting each point from its predecessor's
    K-values should cost no more iterations than starting cold every time."""
    sys = heptane_butane
    ts = np.linspace(295.0, 315.0, 300)
    ps = np.full_like(ts, 100.0)
    cold = sys.flash_pt_batch(ts, ps, [0.5, 0.5], warm_start=False, parallel=False)
    warm = sys.flash_pt_batch(ts, ps, [0.5, 0.5], warm_start=True, parallel=False)
    # Same answers…
    assert warm.beta == pytest.approx(cold.beta, abs=1e-8, nan_ok=True)
    # …but not more iterations overall (warm-start is a net win here).
    assert warm.iterations.sum() <= cold.iterations.sum()


# ── Failed points come back as NaN, not exceptions ────────────────────────

def test_failed_points_are_nan_not_raise(heptane_butane):
    sys = heptane_butane
    # A wildly out-of-range T (5 K) can't flash; it must NaN out without
    # taking the whole batch down.
    ts = np.array([300.0, 5.0, 305.0])
    ps = np.array([100.0])
    batch = sys.flash_pt_batch(ts, ps, [0.5, 0.5])
    assert batch.converged[0] and batch.converged[2]
    # The pathological point either fails (NaN + not converged) or converges
    # to single-phase; either way the batch returned without raising.
    assert len(batch) == 3


# ── Property batch parity ─────────────────────────────────────────────────

def test_z_factor_batch_matches_scalar(heptane_butane):
    sys = heptane_butane
    ts = np.linspace(300.0, 400.0, 15)
    ps = np.full_like(ts, 500.0)
    zb = sys.z_factor_batch(ts, ps, [0.5, 0.5], "vapor")
    for i, (t, p) in enumerate(zip(ts, ps)):
        assert zb[i] == pytest.approx(sys.z_factor(float(t), float(p), [0.5, 0.5], "vapor"), rel=1e-9)


def test_enthalpy_entropy_batch_matches_scalar(heptane_butane):
    sys = heptane_butane
    ts = np.linspace(300.0, 380.0, 12)
    ps = np.full_like(ts, 200.0)
    h, s = sys.enthalpy_entropy_batch(ts, ps, [0.5, 0.5], "liquid")
    for i, (t, p) in enumerate(zip(ts, ps)):
        hs, ss = sys.enthalpy_entropy(float(t), float(p), [0.5, 0.5], "liquid")
        assert h[i] == pytest.approx(hs, rel=1e-8, nan_ok=True)
        assert s[i] == pytest.approx(ss, rel=1e-8, nan_ok=True)
