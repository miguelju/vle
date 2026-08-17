"""Refinery thermodynamics — the pure-state building blocks (Milestone 20).

The mixture-level entry points live on :class:`vle.System`:

- :meth:`vle.System.flash_free_water` — the water-decant flash for
  steam-stripped feeds;
- ``liquid_model="grayson_streed"`` / ``"bk10"`` — the refinery K-value
  methods, used by every flash and saturation call exactly like the cubic
  and activity liquids;
- :meth:`vle.System.lee_kesler_departure` /
  :meth:`vle.System.enthalpy_entropy_lee_kesler` — the Lee–Kesler enthalpy
  route;
- :meth:`vle.System.peneloux_shifts` /
  :meth:`vle.System.translated_molar_volume` /
  :meth:`vle.System.translated_density` — Peneloux volume translation.

This module exposes the underlying correlations for a single component or a
reduced state, so each can be used and taught on its own — the same split
:mod:`vle.petroleum` makes. Inputs accept plain floats in the engine's canonical
units (K, kPa absolute) or :mod:`pint` quantities.

Design record: ``docs/plans/engine/PETROLEUM_PSEUDOCOMPONENT_PLAN.md`` §2
(U4, U5); learning guide: ``docs/en/petroleum/README.md`` §11.
"""

from __future__ import annotations

from typing import Any, Union

from vle import _engine
from vle._engine import ChaoSeaderSpecies, CubicEos, RegularSolutionSet
from vle.units import to_canonical

__all__ = [
    "ChaoSeaderSpecies",
    "RegularSolutionSet",
    "lee_kesler_reduced",
    "regular_solution_ln_nu",
    "peneloux_shift",
]


def _coerce(value: Any, target: str) -> float:
    if hasattr(value, "magnitude") and hasattr(value, "units"):
        return to_canonical(float(value.magnitude), str(value.units), target)
    return float(value)


def _t(value: Any) -> float:
    return _coerce(value, "temperature")


def _p(value: Any) -> float:
    return _coerce(value, "pressure")


_EOS = {
    "pr": CubicEos.PR1976,
    "pr1976": CubicEos.PR1976,
    "srk": CubicEos.RKS1972,
    "rks": CubicEos.RKS1972,
    "rks1972": CubicEos.RKS1972,
}


def _eos(value: Union[str, CubicEos]) -> CubicEos:
    if isinstance(value, CubicEos):
        return value
    key = str(value).lower().replace("-", "").replace("_", "")
    if key in _EOS:
        return _EOS[key]
    try:
        return getattr(CubicEos, str(value))
    except AttributeError as exc:
        raise ValueError(f"unknown EOS {value!r}") from exc


def lee_kesler_reduced(tr: float, pr: float, omega: float, phase: str = "vapor") -> dict:
    """Lee–Kesler departure functions at reduced ``(Tr, Pr, ω)``.

    Returns a dict with ``z``, ``h_dep_rt`` = (H−H°)/(RT), ``s_dep_r`` = (S−S°)/R
    and ``ln_phi`` = ln(f/P), all dimensionless. ``phase`` is ``"vapor"`` or
    ``"liquid"`` — which root of the reduced BWR to take; above the critical
    isotherm both give the same answer.
    """
    z, h, s, ln_phi = _engine.refinery_lee_kesler_reduced(float(tr), float(pr), float(omega), phase)
    return {"z": z, "h_dep_rt": h, "s_dep_r": s, "ln_phi": ln_phi}


def regular_solution_ln_nu(
    t: Any,
    p: Any,
    tc: Any,
    pc: Any,
    omega: float,
    *,
    set: Union[str, RegularSolutionSet] = "grayson-streed",
    species: Union[str, ChaoSeaderSpecies] = "normal",
) -> float:
    """``ln ν`` — the Chao–Seader / Grayson–Streed pure-liquid fugacity coefficient.

    ``set`` picks the coefficient table: ``"grayson-streed"`` (1963, the
    refinery standard and the one every flash uses) or ``"chao-seader"``
    (1961 original). ``species`` is ``"normal"``, ``"hydrogen"`` or
    ``"methane"``. Temperatures in K, pressures in kPa (or unit-aware).
    """
    if not isinstance(set, RegularSolutionSet):
        key = str(set).lower().replace("-", "").replace("_", "").replace(" ", "")
        set = {
            "graysonstreed": RegularSolutionSet.GraysonStreed1963,
            "graysonstreed1963": RegularSolutionSet.GraysonStreed1963,
            "gs": RegularSolutionSet.GraysonStreed1963,
            "chaoseader": RegularSolutionSet.ChaoSeader1961,
            "chaoseader1961": RegularSolutionSet.ChaoSeader1961,
            "cs": RegularSolutionSet.ChaoSeader1961,
        }[key]
    if not isinstance(species, ChaoSeaderSpecies):
        species = {
            "normal": ChaoSeaderSpecies.Normal,
            "hydrogen": ChaoSeaderSpecies.Hydrogen,
            "h2": ChaoSeaderSpecies.Hydrogen,
            "methane": ChaoSeaderSpecies.Methane,
            "ch4": ChaoSeaderSpecies.Methane,
        }[str(species).lower()]
    return _engine.regular_solution_ln_nu(_t(t), _p(p), _t(tc), _p(pc), float(omega), set, species)


def peneloux_shift(eos: Union[str, CubicEos], tc: Any, pc: Any, omega: float, *, zra: float = 0.0) -> float:
    """Peneloux volume shift ``c`` in **cm³/mol** for one component under ``eos`` (SRK or PR).

    ``zra=0`` uses the ``0.29056 − 0.08775·ω`` Rackett correlation.
    """
    return _engine.refinery_peneloux_shift(_eos(eos), _t(tc), _p(pc), float(omega), float(zra))
