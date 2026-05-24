"""Tests for the M6 numerics bindings exposed via ``vle._engine``.

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
    broyden,
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


# ─────────────────────────── Broyden (M6.2) ─────────────────────────────


def test_broyden_solves_2x2_polynomial():
    """x² + y² = 2, x·y = 1 → root (1, 1)."""

    def f(v):
        x, y = v
        return [x * x + y * y - 2.0, x * y - 1.0]

    root = broyden(f, [0.5, 1.5], xtol=1e-14, ftol=1e-14)
    assert root[0] == pytest.approx(1.0, abs=1e-6)
    assert root[1] == pytest.approx(1.0, abs=1e-6)


def test_broyden_solves_linear_system():
    """Linear system → Broyden's rank-1 update converges in ~1 step."""

    # A·x = b → F(x) = A·x − b. A = [[2,1,0],[1,3,1],[0,1,2]], b = [3,5,3]
    # Solution: x = [1, 1, 1].
    def f(v):
        return [
            2.0 * v[0] + v[1] - 3.0,
            v[0] + 3.0 * v[1] + v[2] - 5.0,
            v[1] + 2.0 * v[2] - 3.0,
        ]

    root = broyden(f, [0.0, 0.0, 0.0])
    for r in root:
        assert r == pytest.approx(1.0, abs=1e-7)


def test_broyden_uses_default_tolerances():
    """Defaults (xtol=ftol=1e-8, max_iter=100, refresh_every=5) work for
    the 2x2 polynomial system without any explicit kwargs."""

    def f(v):
        x, y = v
        return [x * x + y * y - 2.0, x * y - 1.0]

    root = broyden(f, [0.5, 1.5])
    # Looser tol because default ftol=1e-8 admits ~5e-5 offset on x near (1,1)
    assert root[0] == pytest.approx(1.0, abs=1e-3)
    assert root[1] == pytest.approx(1.0, abs=1e-3)


def test_broyden_detects_dimension_mismatch():
    """F returning the wrong length → ValueError, not silent corruption."""

    def f(_v):
        return [1.0, 2.0, 3.0]  # length 3 from a 2-element x0

    with pytest.raises(ValueError, match="length 3, expected 2"):
        broyden(f, [0.0, 0.0])


def test_broyden_detects_singular_jacobian():
    """Initial Jacobian rank-deficient → RuntimeError mentioning singular."""

    def f(v):
        s = v[0] + v[1]
        return [s - 1.0, 2.0 * s - 2.0]  # second eq is 2× the first

    with pytest.raises(RuntimeError, match="singular"):
        broyden(f, [0.0, 0.0])


def test_broyden_reports_non_convergence():
    """Tight iteration cap on a non-trivial problem → RuntimeError."""

    def f(v):
        x, y = v
        return [math.exp(x) + y - 2.0, x + math.exp(y) - 2.0]

    with pytest.raises(RuntimeError, match="did not converge"):
        broyden(f, [2.0, 2.0], max_iter=3)


def test_broyden_propagates_python_callback_exception():
    """Exception raised inside the residual function re-raises verbatim."""

    class CallbackBoom(Exception):
        pass

    def bad(_v):
        raise CallbackBoom("residual evaluation failed")

    with pytest.raises(CallbackBoom, match="residual evaluation failed"):
        broyden(bad, [0.5, 0.5])


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
