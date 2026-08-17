"""Result dataclasses for the high-level :class:`vle.System` API.

The Rust engine returns bare tuples (``(beta, x, y, k, iterations,
two_phase)`` and friends) because that is the cheapest thing to move across
the FFI boundary. These dataclasses give those numbers *names* on the Python
side, in the thesis's notation, so downstream code reads like the equations
it implements rather than like ``result[3]``.

Scalar results (:class:`FlashResult`, :class:`BubbleResult`,
:class:`DewResult`, :class:`CriticalResult`) describe a single state point.
:class:`BatchFlashResult` and :class:`BatchSaturationResult` wrap the numpy
arrays that the ``*_batch`` engine methods return — one row per state point,
plus a boolean ``converged`` mask so a caller can drop the points that
failed without the whole sweep raising.

**Units** (canonical engine units — see ``CLAUDE.md``):

- temperature ``t`` — **K**
- pressure ``p`` — **kPa** (absolute)
- molar enthalpy ``h`` — **kJ/kmol**
- molar entropy ``s`` — **kJ/(kmol·K)**
- ``x``/``y``/``z``/``k``/``beta`` are dimensionless (mole fractions and
  the vapor mole fraction β = V/F)
- critical ``vc`` — **m³/kmol** (matches the thesis critical-point tables)
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    # numpy is a hard runtime dependency, but importing the *type* only for
    # annotations keeps this module import-light and avoids a circular-ish
    # cost at load. The arrays themselves are plain ``numpy.ndarray``.
    import numpy as np


@dataclass(frozen=True)
class FlashResult:
    """Outcome of an isothermal (PT) or adiabatic (PH) flash.

    Attributes:
        beta: Vapor mole fraction β = V/F (dimensionless). 0 ⇒ bubble
            point, 1 ⇒ dew point, in-between ⇒ two-phase.
        x: Liquid-phase mole fractions (one per component).
        y: Vapor-phase mole fractions (one per component).
        k: Equilibrium ratios Kᵢ = yᵢ/xᵢ.
        two_phase: Whether the feed actually split into two phases at the
            given conditions (False ⇒ single-phase; x/y then coincide).
        iterations: Number of solver iterations taken.
        t: Temperature in **K** (the *solved* T for a PH flash; the input T
            for a PT flash, echoed back for convenience). None if not set.
        p: Pressure in **kPa** (absolute).
        enthalpy: Molar enthalpy in **kJ/kmol** (populated by PH flash).
    """

    beta: float
    x: list[float]
    y: list[float]
    k: list[float]
    two_phase: bool
    iterations: int = 0
    t: float | None = None
    p: float | None = None
    enthalpy: float | None = None


@dataclass(frozen=True)
class FreeWaterFlashResult:
    """Outcome of a free-water (water-decant) flash (M20).

    Phase fractions are moles per mole of total feed and sum to one.

    Attributes:
        vapor_fraction: Vapor moles per mole of feed (hydrocarbons + water vapor).
        hc_liquid_fraction: Hydrocarbon-liquid moles per mole of feed.
        free_water_fraction: Free-water (pure liquid water) moles per mole of
            feed; 0 when no second liquid forms.
        y: Vapor mole fractions, all components (water included).
        x: Hydrocarbon-liquid mole fractions; water's entry is 0 by construction.
        k: Dry-hydrocarbon K-values at the hydrocarbon partial pressure; water's
            entry is ``y_w`` when free water exists, else NaN.
        free_water: Whether a free-water phase is present.
        y_water: Vapor mole fraction of water actually used.
        psat_water: Water saturation pressure used, **kPa**.
        iterations: Total dry-flash iterations.
        t: Temperature in **K**. p: Pressure in **kPa** (absolute).
        water_index: Index of the water component.
    """

    vapor_fraction: float
    hc_liquid_fraction: float
    free_water_fraction: float
    y: list[float]
    x: list[float]
    k: list[float]
    free_water: bool
    y_water: float
    psat_water: float
    iterations: int = 0
    t: float | None = None
    p: float | None = None
    water_index: int | None = None


@dataclass(frozen=True)
class LeeKeslerDeparture:
    """Lee–Kesler reduced departure functions of one phase (M20), all dimensionless.

    Attributes:
        z: Compressibility factor.
        h_dep_rt: ``(H − H°)/(R·T)`` — multiply by ``R·T`` for kJ/kmol.
        s_dep_r: ``(S − S°)/R`` with S° the ideal gas at the same T and P.
        ln_phi: ``ln(f/P)``.
    """

    z: float
    h_dep_rt: float
    s_dep_r: float
    ln_phi: float


@dataclass(frozen=True)
class BubbleResult:
    """Bubble-point result: the first bubble of vapor off a liquid.

    Attributes:
        value: The solved variable — pressure in **kPa** for a bubble-*P*
            calculation, temperature in **K** for a bubble-*T* calculation.
        y: Incipient vapor mole fractions (the composition of that first
            bubble).
        k: Equilibrium ratios Kᵢ = yᵢ/xᵢ at the bubble point.
    """

    value: float
    y: list[float]
    k: list[float]


@dataclass(frozen=True)
class DewResult:
    """Dew-point result: the first drop of liquid out of a vapor.

    Attributes:
        value: The solved variable — pressure in **kPa** for a dew-*P*
            calculation, temperature in **K** for a dew-*T* calculation.
        x: Incipient liquid mole fractions (the composition of that first
            drop).
        k: Equilibrium ratios Kᵢ = yᵢ/xᵢ at the dew point.
    """

    value: float
    x: list[float]
    k: list[float]


@dataclass(frozen=True)
class CriticalResult:
    """Mixture critical point (Heidemann–Khalil).

    Attributes:
        tc: Critical temperature in **K**.
        pc: Critical pressure in **kPa** (absolute).
        vc: Critical molar volume in **m³/kmol** (thesis-table convention).
    """

    tc: float
    pc: float
    vc: float


@dataclass(frozen=True)
class BatchFlashResult:
    """Vectorized isothermal-flash result over m state points.

    Every attribute is a numpy array with the batch dimension first. The
    scalar per-point outputs (``beta``, ``iterations``, ``two_phase``,
    ``converged``) are length-m 1-D arrays; the per-component outputs
    (``x``, ``y``, ``k``) are m×n. Points that failed to converge come back
    as NaN rows with ``converged[i] == False`` — filter on that mask rather
    than assuming every row is valid.
    """

    beta: "np.ndarray"
    x: "np.ndarray"
    y: "np.ndarray"
    k: "np.ndarray"
    iterations: "np.ndarray"
    two_phase: "np.ndarray"
    converged: "np.ndarray"

    def __len__(self) -> int:
        return len(self.beta)


@dataclass(frozen=True)
class BatchSaturationResult:
    """Vectorized bubble/dew result over m state points.

    Attributes:
        value: Length-m array of solved pressures (**kPa**) or temperatures
            (**K**), depending on which saturation method produced it.
        incipient: m×n array of incipient-phase mole fractions (vapor y for
            bubble methods, liquid x for dew methods).
        k: m×n array of equilibrium ratios.
        converged: Length-m boolean mask (failed points are NaN).
    """

    value: "np.ndarray"
    incipient: "np.ndarray"
    k: "np.ndarray"
    converged: "np.ndarray"

    def __len__(self) -> int:
        return len(self.value)
