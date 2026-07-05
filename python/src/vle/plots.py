"""Plotting helpers — Pxy, Txy, and phase-envelope diagrams (matplotlib).

These are convenience wrappers that drive the :class:`vle.System` *batch*
API to trace a diagram and hand back a matplotlib ``Axes`` you can further
style or save. matplotlib is an **optional** dependency (the ``plot`` extra:
``pip install vle-thermo[plot]``); importing this module without it raises a
clear install hint rather than a bare ``ModuleNotFoundError``.

All temperatures are **K**, pressures **kPa absolute** — the canonical engine
units. Composition axes are the mole fraction of component 0 (the first
component you passed to :class:`~vle.System`).
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Sequence

import numpy as np

if TYPE_CHECKING:  # only for type hints; no import cost at runtime
    from matplotlib.axes import Axes

    from vle.system import System


def _require_mpl():
    """Import matplotlib.pyplot or raise a helpful install message."""
    try:
        import matplotlib.pyplot as plt

        return plt
    except ImportError as exc:  # pragma: no cover - exercised only without mpl
        raise ImportError(
            "plotting needs matplotlib — install it with "
            "`pip install vle-thermo[plot]` (or `pip install matplotlib`)"
        ) from exc


def _binary_grid(n_points: int) -> np.ndarray:
    """A length-``n_points`` grid of x1 in (0, 1), endpoints nudged inward.

    Pure-component endpoints (x1 = 0 or 1) make several saturation solvers
    degenerate, so we start just inside [0, 1]. The caller plots the curve,
    not the singular endpoints.
    """
    return np.linspace(1e-4, 1.0 - 1e-4, n_points)


def _binary_matrix(x1: np.ndarray) -> np.ndarray:
    """Stack a binary mole-fraction grid into an m×2 composition matrix."""
    return np.column_stack([x1, 1.0 - x1])


def pxy_diagram(
    system: "System",
    t,
    *,
    n_points: int = 51,
    ax: "Axes | None" = None,
    label_prefix: str = "",
    **plot_kwargs,
) -> "Axes":
    """Pressure–composition (P-x-y) diagram of a binary at fixed temperature.

    Traces the bubble curve (P vs liquid x₁) and the dew curve (P vs vapor
    y₁) by sweeping composition through the batch saturation API.

    Args:
        system: A binary :class:`~vle.System` (exactly 2 components).
        t: Temperature in **K** (or a unit-aware value the System accepts).
        n_points: Number of composition points per curve.
        ax: Existing matplotlib Axes to draw on (a new figure is made if
            None).
        label_prefix: Prefix for the legend labels (useful when overlaying).
        **plot_kwargs: Forwarded to both ``ax.plot`` calls.

    Returns:
        The matplotlib ``Axes`` with bubble + dew curves drawn.
    """
    if system.n_components != 2:
        raise ValueError("pxy_diagram requires a binary system (2 components)")
    plt = _require_mpl()
    if ax is None:
        _, ax = plt.subplots()

    x1 = _binary_grid(n_points)
    bub = system.bubble_pressure_batch(_binary_matrix(x1), np.array([float(_k(system, t))]))
    # Dew curve: sweep the *vapor* composition y1 over the same grid.
    dew = system.dew_pressure_batch(_binary_matrix(x1), np.array([float(_k(system, t))]))

    ax.plot(x1, bub.value, label=f"{label_prefix}bubble (P–x₁)", **plot_kwargs)
    ax.plot(x1, dew.value, label=f"{label_prefix}dew (P–y₁)", **plot_kwargs)
    ax.set_xlabel(f"mole fraction {system.names[0]}")
    ax.set_ylabel("pressure / kPa")
    ax.set_title(f"P-x-y: {system.names[0]}–{system.names[1]}")
    ax.legend()
    return ax


def txy_diagram(
    system: "System",
    p,
    *,
    n_points: int = 51,
    ax: "Axes | None" = None,
    label_prefix: str = "",
    **plot_kwargs,
) -> "Axes":
    """Temperature–composition (T-x-y) diagram of a binary at fixed pressure.

    Traces the bubble curve (T vs liquid x₁) and dew curve (T vs vapor y₁)
    via the batch saturation API.

    Args:
        system: A binary :class:`~vle.System`.
        p: Pressure in **kPa absolute** (or unit-aware).
        n_points, ax, label_prefix, **plot_kwargs: As in :func:`pxy_diagram`.

    Returns:
        The matplotlib ``Axes``.
    """
    if system.n_components != 2:
        raise ValueError("txy_diagram requires a binary system (2 components)")
    plt = _require_mpl()
    if ax is None:
        _, ax = plt.subplots()

    x1 = _binary_grid(n_points)
    pk = float(_kp(system, p))
    bub = system.bubble_temperature_batch(_binary_matrix(x1), np.array([pk]))
    dew = system.dew_temperature_batch(_binary_matrix(x1), np.array([pk]))

    ax.plot(x1, bub.value, label=f"{label_prefix}bubble (T–x₁)", **plot_kwargs)
    ax.plot(x1, dew.value, label=f"{label_prefix}dew (T–y₁)", **plot_kwargs)
    ax.set_xlabel(f"mole fraction {system.names[0]}")
    ax.set_ylabel("temperature / K")
    ax.set_title(f"T-x-y: {system.names[0]}–{system.names[1]} at {pk:.1f} kPa")
    ax.legend()
    return ax


def phase_envelope(
    system: "System",
    z: Sequence[float],
    *,
    p_start=100.0,
    max_points: int = 60,
    ax: "Axes | None" = None,
    label: str | None = None,
    **plot_kwargs,
) -> "Axes":
    """Plot the (T, P) phase envelope at fixed feed composition ``z``.

    Uses the engine's Michelsen continuation tracer (``System.phase_envelope``).

    Args:
        system: The :class:`~vle.System`.
        z: Feed mole fractions.
        p_start: Starting pressure in **kPa** for the trace.
        max_points: Maximum number of envelope points.
        ax: Existing Axes (new figure if None).
        label: Legend label for the curve.
        **plot_kwargs: Forwarded to ``ax.plot``.

    Returns:
        The matplotlib ``Axes`` (T on x-axis in K, P on y-axis in kPa).
    """
    plt = _require_mpl()
    if ax is None:
        _, ax = plt.subplots()

    pts = system.phase_envelope(z, p_start=p_start, max_points=max_points)
    ts = [t for t, _ in pts]
    ps = [p for _, p in pts]
    ax.plot(ts, ps, label=label, **plot_kwargs)
    ax.set_xlabel("temperature / K")
    ax.set_ylabel("pressure / kPa")
    ax.set_title(f"Phase envelope: {'–'.join(system.names)}")
    if label:
        ax.legend()
    return ax


# Tiny shims so the diagram helpers accept unit-aware t/p without importing
# the System's private normalizers. Round-trips a scalar through the System's
# own coercion by calling the public conversion path in vle.units.
def _k(system: "System", t) -> float:
    from vle.system import _as_temperature

    return _as_temperature(t)


def _kp(system: "System", p) -> float:
    from vle.system import _as_pressure

    return _as_pressure(p)
