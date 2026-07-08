"""VLE — Vapor-Liquid Equilibrium thermodynamic calculator.

High-level Python interface to the VLE engine (Rust core via PyO3).
Provides cubic equations of state, activity coefficient models, mixing rules,
and flash calculations for multicomponent vapor-liquid equilibrium.

Quick start
-----------
>>> from vle import System
>>> sys = System(["methanol", "water"], eos="PR",
...              liquid_model="activity", activity="wilson")
>>> res = sys.bubble_pressure([0.5, 0.5], 298.15)   # T in K
>>> round(res.value, 1)                              # bubble P in kPa
...

The engine enums (:class:`CubicEos`, :class:`ActivityModel`,
:class:`MixingRule`, :class:`PhaseId`) are re-exported for callers that want
to select an exact variant instead of a friendly alias.
"""

from __future__ import annotations

from vle._engine import ActivityModel, CubicEos, MixingRule, PhaseId
from vle.components import Component
from vle.results import (
    BatchFlashResult,
    BatchSaturationResult,
    BubbleResult,
    CriticalResult,
    DewResult,
    FlashResult,
)
from vle.system import System
from vle import steam

try:  # Prefer the installed distribution's version; fall back for source trees.
    from importlib.metadata import PackageNotFoundError, version

    __version__ = version("vle-thermo")
except (ImportError, PackageNotFoundError):  # pragma: no cover
    __version__ = "0.10.0"

__all__ = [
    "System",
    "Component",
    "FlashResult",
    "BubbleResult",
    "DewResult",
    "CriticalResult",
    "BatchFlashResult",
    "BatchSaturationResult",
    "CubicEos",
    "ActivityModel",
    "MixingRule",
    "PhaseId",
    "steam",
    "__version__",
]
