"""Tests for the M6.1 numerics bindings exposed via ``vle._engine``.

Each algorithm has its own Rust unit tests in engine/src/numerics/; the
tests below confirm the *binding layer* is wired correctly — the wheel
exposes the right names, values round-trip cleanly across the FFI
boundary, and errors raised in either language propagate properly.
"""

from __future__ import annotations

import math

import pytest

from vle._engine import (
    brent,
    halley,
    illinois,
    norm_l1,
    norm_l2,
    norm_linf,
    solve_cubic,
    sum_frac_residual,
)


# ─────────────────────────── solve_cubic ────────────────────────────


def test_solve_cubic_three_real_roots():
    # (x - 1)(x - 2)(x - 3) = x^3 - 6x^2 + 11x - 6
    roots = solve_cubic(1.0, -6.0, 11.0, -6.0)
    assert roots == pytest.approx([1.0, 2.0, 3.0], abs=1e-10)


def test_solve_cubic_one_real_root():
    # x^3 + x + 1 = 0 — discriminant > 0, one real root.
    roots = solve_cubic(1.0, 0.0, 1.0, 1.0)
    assert len(roots) == 1
    assert roots[0] == pytest.approx(-0.682_327_803_828_019_3, abs=1e-12)


def test_solve_cubic_rejects_not_cubic():
    with pytest.raises(ValueError, match="not a cubic"):
        solve_cubic(0.0, 1.0, 2.0, 3.0)


def test_solve_cubic_rejects_nan():
    with pytest.raises(ValueError, match="non-finite"):
        solve_cubic(1.0, float("nan"), 0.0, 0.0)


# ─────────────────────────── bracketed solvers ──────────────────────────


@pytest.mark.parametrize("solver", [brent, illinois])
def test_solver_finds_dottie_number(solver):
    """cos(x) - x = 0 → Dottie number ≈ 0.7390851332151607."""
    root = solver(lambda x: math.cos(x) - x, 0.0, 1.0, 1e-12, 100)
    assert root == pytest.approx(0.739_085_133_215_160_7, abs=1e-9)


@pytest.mark.parametrize("solver", [brent, illinois])
def test_solver_finds_wallis_root(solver):
    """x^3 - 2x - 5 = 0 → Wallis's polynomial root ≈ 2.0945514815..."""
    root = solver(lambda x: x**3 - 2.0 * x - 5.0, 2.0, 3.0, 1e-12, 100)
    assert root == pytest.approx(2.094_551_481_542_326_5, abs=1e-9)


@pytest.mark.parametrize("solver", [brent, illinois])
def test_solver_rejects_unbracketed_interval(solver):
    """Both endpoints same sign → RuntimeError (Rust RootError::NoBracket)."""
    with pytest.raises(RuntimeError, match="does not bracket"):
        solver(lambda x: x * x + 1.0, -1.0, 1.0, 1e-9, 50)


@pytest.mark.parametrize("solver", [brent, illinois])
def test_solver_propagates_python_callback_exception(solver):
    """A Python exception raised inside the callback must re-raise verbatim
    through the Rust solver, not get collapsed into a Rust error."""

    class CallbackBoom(Exception):
        pass

    def bad_f(x):
        if x < 0.5:
            return -1.0
        raise CallbackBoom("callback failed")

    with pytest.raises(CallbackBoom, match="callback failed"):
        solver(bad_f, 0.0, 1.0, 1e-9, 100)


def test_brent_uses_default_tolerances():
    """Default xtol=1e-9 and max_iter=100 should be enough for cos(x) - x."""
    root = brent(lambda x: math.cos(x) - x, 0.0, 1.0)
    assert root == pytest.approx(0.739_085_133_215_160_7, abs=1e-6)


# ─────────────────────────── Halley's method ────────────────────────────


def test_halley_finds_sqrt2():
    """x^2 - 2 = 0 → √2."""
    root = halley(lambda x: (x * x - 2.0, 2.0 * x, 2.0), 1.0, 1e-15, 50)
    assert root == pytest.approx(math.sqrt(2.0), abs=1e-15)


def test_halley_finds_dottie_number():
    """cos(x) - x = 0 with analytical derivatives."""
    root = halley(
        lambda x: (math.cos(x) - x, -math.sin(x) - 1.0, -math.cos(x)),
        0.5,
        1e-15,
        50,
    )
    assert root == pytest.approx(0.739_085_133_215_160_7, abs=1e-15)


def test_halley_detects_singular_step():
    """Denominator 2·f'² - f·f'' = 0 with f ≠ 0 → RuntimeError."""
    with pytest.raises(RuntimeError, match="Halley denominator"):
        halley(lambda _x: (2.0, 1.0, 1.0), 0.5, 1e-12, 50)


def test_halley_propagates_python_callback_exception():
    class CallbackBoom(Exception):
        pass

    def bad(x):
        raise CallbackBoom("derivs failed")

    with pytest.raises(CallbackBoom, match="derivs failed"):
        halley(bad, 1.0, 1e-12, 50)


# ─────────────────────────── utility helpers ────────────────────────────


def test_sum_frac_residual():
    assert sum_frac_residual([0.4, 0.6]) == pytest.approx(0.0, abs=1e-15)
    assert sum_frac_residual([0.3, 0.3, 0.3]) == pytest.approx(0.1, abs=1e-15)
    assert sum_frac_residual([]) == 1.0


def test_norms_basic():
    v = [3.0, -4.0]
    assert norm_l1(v) == pytest.approx(7.0)
    assert norm_l2(v) == pytest.approx(5.0)  # classic 3-4-5
    assert norm_linf(v) == pytest.approx(4.0)


def test_norms_empty():
    assert norm_l1([]) == 0.0
    assert norm_l2([]) == 0.0
    assert norm_linf([]) == 0.0
