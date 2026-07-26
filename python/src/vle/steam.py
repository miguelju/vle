"""Industrial steam tables — IAPWS-IF97 water/steam properties.

A thin, unit-aware Python surface over the Rust ``vle-steam`` crate (compiled
into ``vle._engine``). This is "VLE for water only": the single most-used
thermodynamic reference in process engineering — reboilers, condensers,
steam-header balances, flash-steam recovery, turbine calculations.

Quick start
-----------
>>> from vle import steam
>>> st = steam.Water(T="180 degC", x=1.0)      # saturated steam at 180 °C
>>> round(st.p, 1)                              # saturation pressure, kPa
1002.6
>>> sat = steam.saturation(P="10 bar")         # saturation row at 10 bar
>>> round(sat.h_fg, 1)                          # latent heat, kJ/kg
2014.4

All properties are **mass-basis**: enthalpy/internal energy in kJ/kg, entropy
and heat capacities in kJ/(kg·K), specific volume in m³/kg, density in kg/m³,
speed of sound in m/s. Inputs accept plain floats in the canonical units
(**T [K], P [kPa absolute]**), ``pint`` quantities, or unit strings such as
``"180 degC"`` and ``"10 barg"`` (gauge pressure is resolved through the same
:class:`~vle.units.UnitRegistry` the engine uses, honouring
:func:`~vle.units.set_atmospheric_pressure`).

The batch helpers :func:`properties`, :func:`ph_flash`, and :func:`sat_table`
take numpy arrays (canonical units) and return dicts of numpy arrays — the
"numpy for thermo" path, with the Rust kernels releasing the GIL and fanning
out across cores.
"""

from __future__ import annotations

from typing import Any

import numpy as np

from vle import _engine
from vle.units import Q_, ureg

__all__ = [
    "Water",
    "saturation",
    "properties",
    "ph_flash",
    "sat_table",
    "psat",
    "tsat",
    "latent_heat",
    "psat_derivative",
    "viscosity",
    "thermal_conductivity",
    "surface_tension",
    "transport",
    "SteamState",
    "SatState",
]

# Re-export the engine pyclasses so callers can type-check / introspect.
SteamState = _engine.SteamState
SatState = _engine.SatState


def _coerce(value: Any, target: str) -> float:
    """Coerce ``value`` to a float in unit ``target``.

    Accepts a plain number (assumed already in ``target``'s canonical unit), a
    ``pint`` quantity, or a ``"<value> <unit>"`` string (the only construction
    path that works for pint's offset units like ``degC`` and ``barg``).
    """
    if isinstance(value, str):
        parts = value.strip().split(None, 1)
        if len(parts) != 2:
            raise ValueError(f"expected '<value> <unit>', got {value!r}")
        mag, unit = parts
        return float(Q_(float(mag), unit).to(target).magnitude)
    if isinstance(value, ureg.Quantity):
        return float(value.to(target).magnitude)
    return float(value)


def _t(value: Any) -> float:
    return _coerce(value, "kelvin")


def _p(value: Any) -> float:
    return _coerce(value, "kilopascal")


def _h(value: Any) -> float:
    return _coerce(value, "kJ/kg")


def _s(value: Any) -> float:
    return _coerce(value, "kJ/(kg*K)")


def Water(
    *,
    T: Any = None,
    P: Any = None,
    x: float | None = None,
    h: Any = None,
    s: Any = None,
) -> SteamState:
    """Resolve a single water/steam state from a pair of properties.

    Exactly one valid pair of keyword arguments selects the mode:

    ======================  ===================================================
    Keywords                Mode
    ======================  ===================================================
    ``T, P``                temperature + pressure  → :meth:`SteamState.tp`
    ``T, x``                temperature + quality   → saturated mixture
    ``P, x``                pressure + quality      → saturated mixture
    ``P, h``                pressure + enthalpy     → PH (isenthalpic) flash
    ``P, s``                pressure + entropy      → PS (isentropic) flash
    ======================  ===================================================

    Args:
        T: Temperature (K, ``pint`` quantity, or string like ``"180 degC"``).
        P: Pressure (kPa absolute, quantity, or string like ``"10 barg"``).
        x: Vapor quality (mass fraction), dimensionless in ``[0, 1]``.
        h: Specific enthalpy (kJ/kg, quantity, or string).
        s: Specific entropy (kJ/(kg·K), quantity, or string).

    Returns:
        A :class:`SteamState` with the full mass-basis property set.

    Raises:
        ValueError: if the argument combination is not one of the modes above,
            or the resolved state is outside the IF97 validity range.
    """
    if T is not None and P is not None:
        return _engine.steam_tp(_t(T), _p(P))
    if T is not None and x is not None:
        return _engine.steam_tx(_t(T), float(x))
    if P is not None and x is not None:
        return _engine.steam_px(_p(P), float(x))
    if P is not None and h is not None:
        return _engine.steam_ph(_p(P), _h(h))
    if P is not None and s is not None:
        return _engine.steam_ps(_p(P), _s(s))
    raise ValueError(
        "Water() needs exactly one of the pairs (T,P), (T,x), (P,x), (P,h), (P,s)"
    )


def saturation(*, T: Any = None, P: Any = None) -> SatState:
    """Saturation-table row at a temperature *or* a pressure.

    Args:
        T: Temperature (K / quantity / string); returns the row at ``Psat(T)``.
        P: Pressure (kPa absolute / quantity / string); row at ``Tsat(P)``.

    Returns:
        A :class:`SatState` carrying ``v_f, v_g, h_f, h_g, h_fg, s_f, s_g,
        s_fg, u_f, u_g`` plus ``t`` and ``p``.
    """
    if (T is None) == (P is None):
        raise ValueError("saturation() needs exactly one of T or P")
    if T is not None:
        return _engine.steam_sat_t(_t(T))
    return _engine.steam_sat_p(_p(P))


def psat(T: Any) -> float:
    """Saturation pressure ``Psat(T)`` in **kPa absolute**."""
    return _engine.steam_psat(_t(T))


def tsat(P: Any) -> float:
    """Saturation temperature ``Tsat(P)`` in **K**."""
    return _engine.steam_tsat(_p(P))


def latent_heat(T: Any) -> float:
    """Latent heat of vaporization ``h_fg(T)`` in **kJ/kg**."""
    return _engine.steam_latent_heat(_t(T))


def psat_derivative(T: Any) -> float:
    """Analytic ``dPsat/dT`` in **kPa/K** (region-4 derivative)."""
    return _engine.steam_psat_derivative(_t(T))


def viscosity(T: Any, P: Any) -> float:
    """Dynamic viscosity at ``(T, P)`` in **Pa·s** (IAPWS R12-08).

    Args:
        T: Temperature — a pint quantity or a bare number in **K**.
        P: Pressure — a pint quantity (gauge units accepted) or a bare number
            in **kPa absolute**.

    Returns:
        Dynamic viscosity in **Pa·s**.

    Raises:
        ValueError: if the state is two-phase. Viscosity is a per-phase
            quantity; read ``mu_f``/``mu_g`` off :func:`saturation` instead.
    """
    return _engine.steam_viscosity(_t(T), _p(P))


def thermal_conductivity(T: Any, P: Any) -> float:
    """Thermal conductivity at ``(T, P)`` in **W/(m·K)** (IAPWS R15-11).

    Args:
        T: Temperature — a pint quantity or a bare number in **K**.
        P: Pressure — a pint quantity (gauge units accepted) or a bare number
            in **kPa absolute**.

    Returns:
        Thermal conductivity in **W/(m·K)**.

    Raises:
        ValueError: if the state is two-phase; use ``k_f``/``k_g`` from
            :func:`saturation` instead.
    """
    return _engine.steam_thermal_conductivity(_t(T), _p(P))


def surface_tension(T: Any) -> float:
    """Liquid–vapor surface tension at ``T`` in **N/m** (IAPWS R1-76(2014)).

    Args:
        T: Temperature — a pint quantity or a bare number in **K**, between
            the triple point and the critical temperature.

    Returns:
        Surface tension in **N/m** (multiply by 1000 for mN/m, the unit most
        tables print).
    """
    return _engine.steam_surface_tension(_t(T))


def _as_f64(a: Any) -> np.ndarray:
    """Contiguous float64 1-D view for the numpy batch kernels."""
    return np.ascontiguousarray(np.atleast_1d(np.asarray(a, dtype=np.float64)))


def properties(T: Any, P: Any) -> dict[str, np.ndarray]:
    """Batch ``(T, P) → properties`` over numpy arrays (canonical units).

    Length-1 arrays broadcast against the longer partner. Returns a dict of
    numpy arrays keyed ``t, p, v, rho, u, h, s, cp, cv, w, x`` (out-of-range
    points come back as ``NaN``).
    """
    return _engine.steam_tp_batch(_as_f64(T), _as_f64(P))


def ph_flash(P: Any, h: Any) -> dict[str, np.ndarray]:
    """Batch ``(P, h) → properties`` (PH flash) over numpy arrays.

    Same dict shape as :func:`properties`, with ``t`` the resolved temperature
    and ``x`` the quality (``NaN`` for single-phase points).
    """
    return _engine.steam_ph_batch(_as_f64(P), _as_f64(h))


def transport(T: Any, P: Any) -> dict[str, np.ndarray]:
    """Batch ``(T, P) → transport properties`` over numpy arrays.

    Length-1 arrays broadcast against the longer partner. Returns a dict of
    numpy arrays keyed ``t, p, mu, k, pr, nu, alpha`` — dynamic viscosity
    (**Pa·s**), thermal conductivity (**W/(m·K)**), Prandtl number
    (dimensionless), kinematic viscosity and thermal diffusivity (both
    **m²/s**).

    Two-phase and out-of-range points come back as ``NaN``: transport
    properties are per-phase, and there is no meaningful quality-weighted
    viscosity of a boiling mixture.

    This is a separate call from :func:`properties` rather than extra columns
    on it, because the R15-11 critical enhancement costs several times the
    thermodynamic surface underneath it.
    """
    return _engine.steam_transport_batch(_as_f64(T), _as_f64(P))


def sat_table(T: Any) -> dict[str, np.ndarray]:
    """Batch saturation rows over an array of temperatures (**K**).

    Returns a dict of numpy arrays keyed ``t, p, v_f, v_g, h_f, h_g, h_fg,
    s_f, s_g, s_fg, u_f, u_g`` — one printed steam-table page as columns.
    """
    return _engine.steam_sat_t_batch(_as_f64(T))
