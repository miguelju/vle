"""VLE Units of Measurement — Python wrapper around `pint`.

Mirrors the Rust ``vle_units`` crate (Milestone 3): same canonical units,
same gauge-pressure handling with a configurable atmospheric-pressure offset,
and the same extension API for user-defined units.

See ``docs/en/units/dimensional-analysis.md`` for the full design.

Canonical internal units (matching the Rust engine and legacy VB6/Pascal
codebases):

==================  =================================
Quantity            Canonical unit
==================  =================================
temperature         **K** (absolute)
temperature_diff    **K** (interval; scale-only)
pressure            **kPa** (absolute)
molar_energy        **kJ/kmol**
molar_entropy       **kJ/(kmol·K)**
molar_volume        **cm³/mol**
amount              **kmol**
==================  =================================
"""

from __future__ import annotations

from typing import Final

from pint import UnitRegistry

# Shared registry — exported so users can call any pint extension API.
ureg: Final = UnitRegistry()
Q_ = ureg.Quantity

# Canonical units per quantity. Keep aligned with `units/src/registry.rs`.
CANONICAL: Final[dict[str, str]] = {
    "temperature": "kelvin",
    "temperature_diff": "delta_degC",  # 1 ΔK == 1 Δ°C
    "pressure": "kilopascal",
    "molar_energy": "kJ / kmol",
    "molar_entropy": "kJ / (kmol * kelvin)",
    "molar_volume": "cm**3 / mol",
    "amount": "kmol",
}

# ── Atmospheric pressure (configurable, never hardcoded) ────────────────────
#
# Default is 1 standard atmosphere (101.325 kPa). Override via
# ``set_atmospheric_pressure`` for non-standard sites (e.g. plant at altitude).
_P_ATM_DEFAULT_KPA: Final = 101.325
_p_atm_kpa: float = _P_ATM_DEFAULT_KPA


def _install_gauge_units(p_atm_kpa: float) -> None:
    """(Re)register barg/psig/kPag with the given atmospheric offset.

    Pint stores affine-unit definitions immutably, so changing P_atm requires
    redefining the unit symbols. We use private aliases to allow rebinding.
    """
    # Use ``force_ndarray=False`` is N/A; instead we redefine via ureg.define.
    # Pint accepts ``offset`` in absolute reference-unit terms; for pressure
    # the reference is pascal, so we convert kPa → Pa for the offset literal.
    p_atm_pa = p_atm_kpa * 1000.0
    # Redefine if already present (pint raises on duplicate; catch and re-add)
    for name, scale_pa_per_unit in (
        ("kPag", 1000.0),
        ("barg", 100_000.0),
        ("psig", 6894.757_293_168_36),
    ):
        # Pint's affine syntax: ``name = scale * pascal; offset: P_atm_pa``
        defn = f"{name} = {scale_pa_per_unit} * pascal; offset: {p_atm_pa}"
        try:
            ureg.define(defn)
        except Exception:
            # Already defined: redefine via the private API. Pint ≥ 0.24
            # exposes ``UnitRegistry._units`` — pop and retry.
            ureg._units.pop(name, None)
            ureg.define(defn)


_install_gauge_units(_p_atm_kpa)


# Engineering units missing from pint's default definitions but present in
# the Rust ``vle_units`` registry. Add them so unit strings round-trip across
# both sides.
def _define_if_missing(definition: str) -> None:
    name = definition.split("=", 1)[0].strip()
    if name in ureg:
        return
    ureg.define(definition)


_define_if_missing("lbmol = 0.45359237 * kmol = pound_mole")
_define_if_missing("BTU_per_lbmol = 2.326 * (kJ / kmol)")


def get_atmospheric_pressure() -> float:
    """Return the current atmospheric pressure used for gauge conversions.

    Returns:
        Atmospheric pressure in **kPa**.
    """
    return _p_atm_kpa


def set_atmospheric_pressure(value: float, unit_str: str = "kPa") -> None:
    """Override atmospheric pressure for all subsequent gauge conversions.

    Default is 101.325 kPa (1 standard atm). Change for non-standard altitude
    or weather (e.g. ``set_atmospheric_pressure(84.5)`` at ~1500 m elevation).

    Args:
        value: Atmospheric pressure value (must be > 0).
        unit_str: Unit of ``value`` (default: **kPa**).

    Raises:
        ValueError: If the resulting P_atm is ≤ 0.
    """
    global _p_atm_kpa
    new_p_atm = Q_(value, unit_str).to("kPa").magnitude
    if new_p_atm <= 0:
        raise ValueError(f"Atmospheric pressure must be > 0; got {new_p_atm} kPa")
    _p_atm_kpa = float(new_p_atm)
    _install_gauge_units(_p_atm_kpa)


def to_canonical(value: float, unit_str: str, quantity: str) -> float:
    """Convert ``value`` (in ``unit_str``) to the canonical unit of ``quantity``.

    Gauge pressure units (barg, psig, kPag) are automatically converted to
    absolute kPa using the current atmospheric pressure.

    Args:
        value: Numeric value.
        unit_str: Source unit string (e.g. ``"degC"``, ``"barg"``, ``"cal/mol"``).
        quantity: Quantity name; one of the keys of :data:`CANONICAL`.

    Returns:
        Value in canonical units (e.g. K for temperature, absolute kPa for
        pressure).
    """
    if quantity not in CANONICAL:
        raise KeyError(f"unknown quantity: {quantity!r}")
    q = Q_(value, unit_str)
    return float(q.to(CANONICAL[quantity]).magnitude)


def from_canonical(value_canonical: float, unit_str: str, quantity: str) -> float:
    """Inverse of :func:`to_canonical`.

    Args:
        value_canonical: Value in canonical units of ``quantity``.
        unit_str: Target unit string.
        quantity: Quantity name; one of the keys of :data:`CANONICAL`.

    Returns:
        Value in ``unit_str``.
    """
    q = Q_(value_canonical, CANONICAL[quantity])
    return float(q.to(unit_str).magnitude)


def parse(s: str, quantity: str) -> float:
    """Parse ``"<value> <unit>"`` and return the canonical value of ``quantity``.

    Splits on the first whitespace and constructs ``Q_(value, unit)`` directly
    — this is the only construction path that works for offset units like
    ``degC`` and ``barg`` (pint's string form ``Q_("25 degC")`` raises an
    ``OffsetUnitCalculusError``).

    Example:
        ``parse("3.5 barg", "pressure")`` returns ``451.325`` (kPa absolute) at
        standard atmospheric pressure.
    """
    text = s.strip()
    try:
        val_str, unit_str = text.split(None, 1)
    except ValueError as exc:
        raise ValueError(f"expected '<value> <unit>', got {s!r}") from exc
    return to_canonical(float(val_str), unit_str, quantity)


__all__ = [
    "CANONICAL",
    "Q_",
    "from_canonical",
    "get_atmospheric_pressure",
    "parse",
    "set_atmospheric_pressure",
    "to_canonical",
    "ureg",
]
