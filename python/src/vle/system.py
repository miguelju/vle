"""High-level :class:`System` — the ergonomic front door to the VLE engine.

Everything below is a thin, *stateful* wrapper around the Rust
``vle._engine.System`` pyclass (built in Milestone 10, Track D of
``PERFORMANCE_PROPOSAL.md``). The Rust object owns the component list and
model selection once, at construction, so no per-call ``Component`` rebuild
ever happens again; this Python layer adds the three things a chemical
engineer actually wants at the keyboard:

1. **Name-based construction** — ``System(["methanol", "water"], ...)``
   pulls Tc/Pc/ω/Antoine/vl straight from the bundled database
   (:mod:`vle.components`) instead of making you paste critical constants.
2. **Friendly model names** — ``eos="PR"``, ``liquid_model="activity"``,
   ``activity="wilson"`` map to the engine's enum variants; you can still
   pass the exact :class:`vle._engine.CubicEos` / ``ActivityModel`` /
   ``MixingRule`` enum if you want a specific variant.
3. **Unit-aware inputs and named outputs** — temperatures/pressures accept a
   plain float (canonical **K** / **kPa abs**), a pint ``Quantity``
   (``Q_(25, "degC")``), or a ``"<value> <unit>"`` string; results come back
   as the dataclasses in :mod:`vle.results` rather than bare tuples.

**Canonical units** (see ``CLAUDE.md``): T in **K**, P in **kPa absolute**,
molar enthalpy in **kJ/kmol**, molar entropy in **kJ/(kmol·K)**. Every
method that takes or returns a physical quantity documents its units.
"""

from __future__ import annotations

from typing import Iterable, Sequence, Union

import numpy as np

from vle import _engine
from vle._engine import ActivityModel, CubicEos, MixingRule
from vle import components as _components
from vle.components import Component
from vle.results import (
    BatchFlashResult,
    BatchSaturationResult,
    BubbleResult,
    CriticalResult,
    DewResult,
    FlashResult,
    FreeWaterFlashResult,
    LeeKeslerDeparture,
)
from vle.units import to_canonical

# A temperature or pressure argument may arrive as a bare number (already in
# canonical units), a pint Quantity, or a "25 degC" / "3.5 barg" string.
Scalar = Union[float, str, "object"]

# ── Friendly-name → engine-enum maps ──────────────────────────────────────
#
# Users think "PR", not "PR1976". These alias tables translate the common
# shorthands to engine enum variants. The *exact* variant name (e.g.
# "PRSV1986") and a passed-in enum instance both still work — see `_eos`.

_EOS_ALIASES = {
    "pr": CubicEos.PR1976,
    "pr76": CubicEos.PR1976,
    "peng-robinson": CubicEos.PR1976,
    "pengrobinson": CubicEos.PR1976,
    "rks": CubicEos.RKS1972,
    "srk": CubicEos.RKS1972,
    "soave": CubicEos.RKS1972,
    "rk": CubicEos.RK1949,
    "redlich-kwong": CubicEos.RK1949,
    "vdw": CubicEos.VdW1870,
    "van-der-waals": CubicEos.VdW1870,
    "prsv": CubicEos.PRSV1986,
    "schmidt-wenzel": CubicEos.SchmidtWenzel,
    "patel-teja": CubicEos.PatelTeja,
}

_ACTIVITY_ALIASES = {
    "ideal": ActivityModel.IdealSolution,
    "wilson": ActivityModel.Wilson,
    "van_laar": ActivityModel.VanLaar,
    "vanlaar": ActivityModel.VanLaar,
    "van-laar": ActivityModel.VanLaar,
    "margules": ActivityModel.Margules,
    "scatchard_hildebrand": ActivityModel.ScatchardHildebrand,
    "scatchard-hildebrand": ActivityModel.ScatchardHildebrand,
    "nrtl": ActivityModel.Nrtl,
}

_MIXING_ALIASES = {
    "classical": MixingRule.Classical,
    "vdw": MixingRule.IVDW,
    "1vdw": MixingRule.IVDW,
    "2vdw": MixingRule.IIVDW,
    "wong-sandler": MixingRule.WongSandler,
    "wong_sandler": MixingRule.WongSandler,
    "mhv1": MixingRule.MHV1,
    "mhv2": MixingRule.MHV2,
    "huron-vidal": MixingRule.HuronVidalOriginal,
}


def _resolve(value, aliases, enum_cls, what):
    """Resolve a user model selector to its engine enum.

    Accepts (a) an enum instance already, (b) a friendly alias from
    ``aliases``, or (c) the exact variant name (case-insensitively matched
    against the enum's attributes). Raises ``ValueError`` otherwise with the
    list of accepted aliases.
    """
    if isinstance(value, enum_cls):
        return value
    if not isinstance(value, str):
        raise TypeError(f"{what} must be a str or {enum_cls.__name__}, got {type(value).__name__}")
    key = value.strip().lower()
    if key in aliases:
        return aliases[key]
    # Fall back to an exact variant-name match (e.g. "PRSV1986").
    for attr in dir(enum_cls):
        if attr.lower() == key and not attr.startswith("_"):
            return getattr(enum_cls, attr)
    raise ValueError(
        f"unknown {what} {value!r}; aliases: {', '.join(sorted(aliases))} "
        f"(or an exact {enum_cls.__name__} variant name)"
    )


def _as_temperature(value: Scalar) -> float:
    """Normalize a temperature input to canonical **K**."""
    return _as_quantity(value, "temperature")


def _as_pressure(value: Scalar) -> float:
    """Normalize a pressure input to canonical **kPa absolute**."""
    return _as_quantity(value, "pressure")


def _as_quantity(value: Scalar, quantity: str) -> float:
    """Coerce float / pint Quantity / '<v> <unit>' string to canonical units."""
    # Plain number: already canonical (K or kPa abs) by convention.
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return float(value)
    # "25 degC" / "3.5 barg" string → parse and convert.
    if isinstance(value, str):
        from vle.units import parse

        return parse(value, quantity)
    # pint Quantity (duck-typed: has .magnitude and .units) → to canonical.
    if hasattr(value, "magnitude") and hasattr(value, "units"):
        return to_canonical(float(value.magnitude), str(value.units), quantity)
    raise TypeError(
        f"expected a number, a pint Quantity, or a '<value> <unit>' string "
        f"for {quantity}, got {type(value).__name__}"
    )


def _as_array_canonical(values, quantity: str) -> np.ndarray:
    """Coerce an array-like (or pint Quantity array) to a canonical float64 array."""
    if hasattr(values, "magnitude") and hasattr(values, "units"):
        mag = np.asarray(values.magnitude, dtype=np.float64)
        unit = str(values.units)
        # Vectorized unit conversion via a single canonical scale where
        # possible; fall back to element-wise for offset units.
        return np.array([to_canonical(v, unit, quantity) for v in mag], dtype=np.float64)
    return np.ascontiguousarray(values, dtype=np.float64)


class System:
    """A persistent multicomponent VLE system.

    Construct from component names (looked up in the bundled database) and a
    model selection, then call flash / saturation / property methods on it.
    The heavy state lives in the Rust ``_engine.System``; this object is a
    typed, unit-aware facade.

    Args:
        components: Component names (strings, looked up in
            :mod:`vle.components`) or :class:`~vle.components.Component`
            objects, or explicit ``(Tc, Pc, ω)`` when combined with
            :meth:`from_arrays`.
        eos: EOS used for *both* phases unless ``vapor_eos`` / ``liquid_eos``
            override it. Alias ("PR", "RKS", …), exact variant name, or a
            :class:`~vle._engine.CubicEos` instance.
        vapor_model: ``"cubic"`` (default), ``"ideal"``, or ``"virial"``.
        liquid_model: ``"cubic"`` (default), ``"activity"``, ``"ideal"``,
            ``"chao_seader"``, or the M20 refinery methods ``"grayson_streed"``
            (``Kᵢ = νᵢγᵢ/φ̂ᵢⱽ``; needs ``solubility_param`` + ``liquid_volume``
            on the components for γ ≠ 1) and ``"bk10"`` (Braun K10 from
            Maxwell–Bonnell; needs ``tb`` on every component).
        activity: Activity model when ``liquid_model="activity"``; alias
            ("wilson", "van_laar", "nrtl", …) or an ``ActivityModel`` instance.
        mixing_rule: Mixing rule; alias ("classical", "wong-sandler", …) or a
            ``MixingRule`` instance. Default ``"classical"``.
        kij: Binary interaction parameters — an n×n matrix, or a single float
            for a binary (broadcast to a symmetric 2×2 with zero diagonal).
        aij: Activity-model interaction matrix (n×n), used by γ-models. For
            NRTL these are the interaction energies ``gᵢⱼ − gⱼⱼ`` in **kJ/kmol**.
        alpha: NRTL non-randomness matrix (n×n, symmetric, dimensionless; a
            single float broadcasts to a binary). Required for ``activity="nrtl"``,
            ignored by every other model.
        t_ref: Ideal-gas enthalpy/entropy reference temperature in **K**.
        p_ref: Ideal-gas reference pressure in **kPa absolute**.
    """

    def __init__(
        self,
        components: Sequence[Union[str, Component]],
        *,
        eos: Union[str, CubicEos] = "PR",
        vapor_model: str = "cubic",
        liquid_model: str = "cubic",
        vapor_eos: Union[str, CubicEos, None] = None,
        liquid_eos: Union[str, CubicEos, None] = None,
        activity: Union[str, ActivityModel, None] = None,
        mixing_rule: Union[str, MixingRule] = "classical",
        kij=None,
        aij=None,
        alpha=None,
        t_ref: float = 298.15,
        p_ref: float = 101.325,
    ) -> None:
        comps = [self._coerce_component(c) for c in components]
        if len(comps) == 0:
            raise ValueError("a System needs at least one component")
        self._comps = comps
        n = len(comps)

        # Resolve the shared / per-phase EOS.
        base_eos = _resolve(eos, _EOS_ALIASES, CubicEos, "eos")
        v_eos = _resolve(vapor_eos, _EOS_ALIASES, CubicEos, "vapor_eos") if vapor_eos else base_eos
        l_eos = _resolve(liquid_eos, _EOS_ALIASES, CubicEos, "liquid_eos") if liquid_eos else base_eos

        act = None
        if activity is not None:
            act = _resolve(activity, _ACTIVITY_ALIASES, ActivityModel, "activity")
        rule = _resolve(mixing_rule, _MIXING_ALIASES, MixingRule, "mixing_rule")

        self._engine = _engine.System(
            tcs=[c.tc for c in comps],
            pcs=[c.pc for c in comps],
            omegas=[c.omega for c in comps],
            vapor_kind=vapor_model,
            liquid_kind=liquid_model,
            vapor_eos=v_eos,
            liquid_eos=l_eos,
            liquid_activity=act,
            mixing_rule=rule,
            kij=self._square_matrix(kij, n, "kij"),
            aij=self._square_matrix(aij, n, "aij"),
            alpha=self._square_matrix(alpha, n, "alpha"),
            vl=[c.liquid_volume for c in comps],
            psat_coeffs=[list(c.psat_coeffs) for c in comps],
            # Ideal-gas Cp°/R polynomial (M12.1). Threading this fixes the
            # silent-zero ideal-Cp defect: without it, every enthalpy/entropy
            # of a DB-built system dropped the (usually dominant) ideal-gas
            # contribution. Empty rows (Components without Cp data) are left
            # for the engine to treat as zero.
            cp_coeffs=[list(c.cp_coeffs) for c in comps],
            tbs=[c.tb for c in comps],
            names=[c.name for c in comps],
            ge_model=act,
            t_ref=t_ref,
            p_ref=p_ref,
            # M20: regular-solution δ (Grayson-Streed), molecular weight
            # (Peneloux mass density), Watson K (Braun K10), Rackett Z_RA.
            delta=[c.solubility_param for c in comps],
            mws=[c.mw for c in comps],
            watson_ks=[c.watson_k for c in comps],
            zras=[c.zra for c in comps],
        )

    # ── Construction helpers ──────────────────────────────────────────────

    @staticmethod
    def _coerce_component(c: Union[str, Component]) -> Component:
        if isinstance(c, Component):
            return c
        if isinstance(c, str):
            return _components.get(c)
        raise TypeError(f"component must be a name or Component, got {type(c).__name__}")

    @staticmethod
    def _square_matrix(m, n: int, what: str) -> list[list[float]]:
        """Normalize a kij/aij argument to an n×n list-of-lists (or [] if None)."""
        if m is None:
            return []
        # A bare float for a binary → symmetric off-diagonal, zero diagonal.
        if isinstance(m, (int, float)) and not isinstance(m, bool):
            if n != 2:
                raise ValueError(f"scalar {what} only makes sense for a binary (n=2), got n={n}")
            return [[0.0, float(m)], [float(m), 0.0]]
        arr = np.asarray(m, dtype=np.float64)
        if arr.shape != (n, n):
            raise ValueError(f"{what} must be {n}×{n}, got shape {arr.shape}")
        return arr.tolist()

    @classmethod
    def from_arrays(
        cls,
        *,
        tcs: Sequence[float],
        pcs: Sequence[float],
        omegas: Sequence[float],
        names: Sequence[str] | None = None,
        psat_coeffs: Sequence[Sequence[float]] | None = None,
        vl: Sequence[float] | None = None,
        tbs: Sequence[float] | None = None,
        **kwargs,
    ) -> "System":
        """Build a System from explicit property arrays (no database lookup).

        Units: ``tcs`` in **K**, ``pcs`` in **kPa abs**, ``omegas``
        dimensionless, ``vl`` in **cm³/mol**, ``tbs`` in **K**. ``**kwargs``
        are passed through to :class:`System` (eos, mixing_rule, kij, …).
        """
        n = len(tcs)
        names = list(names) if names is not None else [f"comp{i + 1}" for i in range(n)]
        psat = list(psat_coeffs) if psat_coeffs is not None else [[] for _ in range(n)]
        vl_ = list(vl) if vl is not None else [0.0] * n
        tb_ = list(tbs) if tbs is not None else [0.0] * n
        comps = [
            Component(
                name=names[i], tc=tcs[i], pc=pcs[i], omega=omegas[i],
                psat_coeffs=list(psat[i]), liquid_volume=vl_[i], tb=tb_[i],
            )
            for i in range(n)
        ]
        return cls(comps, **kwargs)

    # ── Introspection ─────────────────────────────────────────────────────

    @property
    def n_components(self) -> int:
        """Number of components in the system."""
        return self._engine.n_components

    @property
    def names(self) -> list[str]:
        """Component names, in order."""
        return list(self._engine.names)

    @property
    def components(self) -> list[Component]:
        """The :class:`~vle.components.Component` records backing the system."""
        return list(self._comps)

    def __len__(self) -> int:
        return self.n_components

    def __repr__(self) -> str:
        return f"System({self.names!r})"

    # ── Scalar flash / saturation ─────────────────────────────────────────

    def flash_pt(
        self,
        t: Scalar,
        p: Scalar,
        z: Sequence[float],
        *,
        k_init: Sequence[float] | None = None,
        tol: float = 1e-10,
        max_iter: int = 200,
    ) -> FlashResult:
        """Isothermal (PT) flash of feed ``z`` at temperature ``t``, pressure ``p``.

        ``t`` in **K**, ``p`` in **kPa abs** (or unit-aware inputs). Returns a
        :class:`~vle.results.FlashResult`.
        """
        tk, pk = _as_temperature(t), _as_pressure(p)
        beta, x, y, k, iters, two = self._engine.flash_pt(
            tk, pk, list(z), None if k_init is None else list(k_init), tol, max_iter
        )
        return FlashResult(
            beta=beta, x=x, y=y, k=k, two_phase=two, iterations=iters, t=tk, p=pk
        )

    def flash_ph(
        self,
        p: Scalar,
        z: Sequence[float],
        h_feed: float,
        *,
        t_bracket: tuple[Scalar, Scalar],
        tol: float = 1e-4,
        max_iter: int = 200,
    ) -> FlashResult:
        """Adiabatic (PH) flash — cubic φ-φ systems only.

        ``p`` in **kPa abs**, ``h_feed`` in **kJ/kmol**, ``t_bracket`` the
        (lo, hi) temperature bracket in **K**. Returns a
        :class:`~vle.results.FlashResult` with the solved ``t`` and
        ``enthalpy`` populated.
        """
        pk = _as_pressure(p)
        t_lo, t_hi = _as_temperature(t_bracket[0]), _as_temperature(t_bracket[1])
        t, beta, x, y, h = self._engine.flash_ph(
            pk, list(z), h_feed, t_lo, t_hi, tol, max_iter
        )
        return FlashResult(
            beta=beta, x=x, y=y, k=[yi / xi if xi else float("nan") for xi, yi in zip(x, y)],
            two_phase=0.0 < beta < 1.0, t=t, p=pk, enthalpy=h,
        )

    def bubble_pressure(self, x: Sequence[float], t: Scalar, *, tol: float = 1e-9, max_iter: int = 200) -> BubbleResult:
        """Bubble pressure (**kPa**) at fixed T (**K**) and liquid ``x``."""
        val, y, k = self._engine.bubble_pressure(list(x), _as_temperature(t), tol, max_iter)
        return BubbleResult(value=val, y=y, k=k)

    def bubble_temperature(self, x: Sequence[float], p: Scalar, *, tol: float = 1e-9, max_iter: int = 200) -> BubbleResult:
        """Bubble temperature (**K**) at fixed P (**kPa**) and liquid ``x``."""
        val, y, k = self._engine.bubble_temperature(list(x), _as_pressure(p), tol, max_iter)
        return BubbleResult(value=val, y=y, k=k)

    def dew_pressure(self, y: Sequence[float], t: Scalar, *, tol: float = 1e-9, max_iter: int = 200) -> DewResult:
        """Dew pressure (**kPa**) at fixed T (**K**) and vapor ``y``."""
        val, x, k = self._engine.dew_pressure(list(y), _as_temperature(t), tol, max_iter)
        return DewResult(value=val, x=x, k=k)

    def dew_temperature(self, y: Sequence[float], p: Scalar, *, tol: float = 1e-9, max_iter: int = 200) -> DewResult:
        """Dew temperature (**K**) at fixed P (**kPa**) and vapor ``y``."""
        val, x, k = self._engine.dew_temperature(list(y), _as_pressure(p), tol, max_iter)
        return DewResult(value=val, x=x, k=k)

    def critical_point(self, z: Sequence[float], *, t_init: Scalar = 0.0, max_iter: int = 200) -> CriticalResult:
        """Mixture critical point (Heidemann–Khalil) — cubic + classical mixing.

        Returns Tc (**K**), Pc (**kPa**), Vc (**m³/kmol**). ``t_init=0`` uses
        the mole-fraction-average Tc as the initial guess.
        """
        ti = _as_temperature(t_init) if t_init else 0.0
        tc, pc, vc = self._engine.critical_point(list(z), ti, max_iter)
        return CriticalResult(tc=tc, pc=pc, vc=vc)

    def phase_envelope(self, z: Sequence[float], *, p_start: Scalar = 100.0, max_points: int = 60) -> list[tuple[float, float]]:
        """Trace the (T [K], P [kPa]) phase envelope at composition ``z``."""
        return self._engine.trace_envelope(list(z), _as_pressure(p_start), max_points)

    def stability(self, z: Sequence[float], t: Scalar, p: Scalar, *, max_iter: int = 100):
        """TPD stability test at ``(t, p)``. Returns ``(is_stable, trial_k, tpd)``."""
        return self._engine.stability(list(z), _as_temperature(t), _as_pressure(p), max_iter)

    # ── Scalar properties ─────────────────────────────────────────────────

    def k_values(self, t: Scalar, p: Scalar, x: Sequence[float], y: Sequence[float]) -> list[float]:
        """Equilibrium ratios Kᵢ = yᵢ/xᵢ at trial ``(t, p, x, y)``."""
        return self._engine.k_values(_as_temperature(t), _as_pressure(p), list(x), list(y))

    def z_factor(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> float:
        """Mixture compressibility factor Z of ``phase`` at ``(t, p, x)``."""
        return self._engine.z_factor(_as_temperature(t), _as_pressure(p), list(x), phase)

    def ln_phi(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> list[float]:
        """Partial fugacity coefficients ln φ̂ᵢ of ``phase`` at ``(t, p, x)``."""
        return self._engine.ln_phi(_as_temperature(t), _as_pressure(p), list(x), phase)

    def d_ln_phi_d_t(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> list[float]:
        """Exact ∂ln φ̂ᵢ/∂T of ``phase`` at ``(t, p, x)``, in **1/K** (M12.3).

        Dual-number AD through the temperature-generic fugacity core — exact to
        machine precision, not finite differences. ``phase`` needs a cubic model.
        """
        return self._engine.d_ln_phi_d_t(_as_temperature(t), _as_pressure(p), list(x), phase)

    def d_ln_phi_d_p(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> list[float]:
        """Exact ∂ln φ̂ᵢ/∂P of ``phase`` at ``(t, p, x)``, in **1/kPa** (M12.3).

        Dual-number AD through the pressure-generic fugacity core. ``phase``
        needs a cubic model.
        """
        return self._engine.d_ln_phi_d_p(_as_temperature(t), _as_pressure(p), list(x), phase)

    def k_values_with_derivs(
        self, t: Scalar, p: Scalar, x: Sequence[float], y: Sequence[float]
    ) -> tuple[list[float], list[float], list[float]]:
        """K-values and their exact T/P derivatives at ``(t, p, x, y)`` (M12.3).

        Returns ``(k, d_ln_k_d_t, d_ln_k_d_p)`` with ``k`` dimensionless,
        ``d_ln_k_d_t`` in **1/K** and ``d_ln_k_d_p`` in **1/kPa**. Dispatches on
        the System's liquid/vapor model exactly like :meth:`k_values`; the ``k``
        field is identical to that method's output.
        """
        return self._engine.k_values_with_derivs(
            _as_temperature(t), _as_pressure(p), list(x), list(y)
        )

    def enthalpy_entropy(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> tuple[float, float]:
        """Molar (H [kJ/kmol], S [kJ/(kmol·K)]) of ``phase`` vs the reference state.

        Routed through the SystemSpec-level dispatch (M12.4): a γ-φ (activity)
        liquid returns the ideal − condensation + excess assembly instead of
        erroring for lack of a cubic liquid EOS.
        """
        return self._engine.enthalpy_entropy(_as_temperature(t), _as_pressure(p), list(x), phase)


    # ── Refinery thermodynamics (M20) ─────────────────────────────────────

    def flash_free_water(
        self,
        t: Scalar,
        p: Scalar,
        z: Sequence[float],
        *,
        water: Union[int, str] = "water",
        psat_water: Scalar | None = None,
        tol: float = 1e-10,
        max_iter: int = 200,
    ) -> FreeWaterFlashResult:
        """Free-water (water-decant) flash of a steam-stripped feed (M20).

        Water is treated as immiscible with the hydrocarbon liquid: the vapor is
        saturated with water at ``Pˢᵃᵗ_w(T)`` whenever a free-water phase exists,
        and the hydrocarbons flash at their partial pressure ``P − y_w·P`` with
        this System's models. ``water`` is the water component's index or
        name; ``psat_water`` (kPa or unit-aware) overrides the water
        component's saturation model — pass an IF97 value from
        :mod:`vle.steam` when accuracy matters. Returns a
        :class:`~vle.results.FreeWaterFlashResult`.
        """
        idx = water if isinstance(water, int) else self.names.index(water)
        pw = None if psat_water is None else _as_pressure(psat_water)
        tk, pk = _as_temperature(t), _as_pressure(p)
        d = self._engine.flash_free_water(tk, pk, list(z), idx, pw, tol, max_iter)
        return FreeWaterFlashResult(t=tk, p=pk, water_index=idx, **d)

    def lee_kesler_pseudocritical(self, x: Sequence[float], *, eta: float = 0.25) -> tuple[float, float, float]:
        """Lee–Kesler pseudo-critical ``(Tc [K], Pc [kPa], ω)`` of composition ``x`` (M20).

        ``eta=1.0`` is Lee & Kesler's own mixing rule, ``0.25`` (default)
        Plöcker–Knapp–Prausnitz's, which most refinery packages run.
        """
        return self._engine.lee_kesler_pseudocritical(list(x), eta)

    def lee_kesler_departure(
        self, t: Scalar, p: Scalar, x: Sequence[float], phase: str, *, eta: float = 0.25
    ) -> LeeKeslerDeparture:
        """Lee–Kesler reduced departure functions of ``phase`` at ``(t, p, x)`` (M20):
        ``Z``, ``(H−H°)/(RT)``, ``(S−S°)/R`` and ``ln(f/P)``, all dimensionless.
        """
        z, h, s_, lnphi = self._engine.lee_kesler_departure(
            _as_temperature(t), _as_pressure(p), list(x), phase, eta
        )
        return LeeKeslerDeparture(z=z, h_dep_rt=h, s_dep_r=s_, ln_phi=lnphi)

    def enthalpy_entropy_lee_kesler(
        self, t: Scalar, p: Scalar, x: Sequence[float], phase: str, *, eta: float = 0.25
    ) -> tuple[float, float]:
        """Molar (H [kJ/kmol], S [kJ/(kmol·K)]) of ``phase`` with the **Lee–Kesler**
        residual in place of the EOS departure (M20). Same reference state as
        :meth:`enthalpy_entropy`, so the two routes are directly comparable —
        which is the point: it is the refinery-standard enthalpy method.
        """
        return self._engine.enthalpy_entropy_lee_kesler(
            _as_temperature(t), _as_pressure(p), list(x), phase, eta
        )

    def peneloux_shifts(self) -> list[float]:
        """Peneloux volume shifts ``cᵢ`` in **cm³/mol** under the liquid cubic EOS (M20)."""
        return list(self._engine.peneloux_shifts())

    def translated_molar_volume(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> float:
        """Volume-translated (Peneloux) molar volume of ``phase`` in **cm³/mol** (M20).

        ``V = Z_EOS·R·T/P − Σxᵢcᵢ``; the shift leaves every K-value untouched
        and fixes the heavy-liquid density a bare SRK/PR gets ~10 % light.
        """
        return self._engine.translated_molar_volume(_as_temperature(t), _as_pressure(p), list(x), phase)

    def translated_density(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> float:
        """Volume-translated mass density of ``phase`` in **kg/m³** (M20). Needs ``mw``."""
        return self._engine.translated_density(_as_temperature(t), _as_pressure(p), list(x), phase)

    def partial_molar_enthalpy(
        self, t: Scalar, p: Scalar, x: Sequence[float], phase: str
    ) -> list[float]:
        """Partial molar enthalpies H̄ᵢ of ``phase`` at ``(t, p, x)``, in **kJ/kmol** (M12.4).

        ``H̄ᵢ = h°ᵢ(T) − R·T²·∂ln φ̂ᵢ/∂T`` (exact identity over
        :meth:`d_ln_phi_d_t`); ``Σxᵢ·H̄ᵢ`` equals the total phase enthalpy.
        Needs a cubic model on ``phase``.
        """
        return self._engine.partial_molar_enthalpy(
            _as_temperature(t), _as_pressure(p), list(x), phase
        )

    def phase_cp(self, t: Scalar, p: Scalar, x: Sequence[float], phase: str) -> float:
        """Real-mixture isobaric heat capacity Cp of ``phase`` at ``(t, p, x)``,
        in **kJ/(kmol·K)** (M12.4).

        ``Cp = Σxᵢ·Cpᵢ°(T) + Cp^R``, the residual via a second-order dual
        through the temperature-generic fugacity core. Needs a cubic model on
        ``phase``.
        """
        return self._engine.phase_cp(_as_temperature(t), _as_pressure(p), list(x), phase)

    # ── Batch (numpy) methods ─────────────────────────────────────────────

    def flash_pt_batch(
        self,
        ts,
        ps,
        z: Sequence[float],
        *,
        warm_start: bool = True,
        parallel: bool = True,
        tol: float = 1e-10,
        max_iter: int = 200,
    ) -> BatchFlashResult:
        """Vectorized isothermal flash over paired ``(ts, ps)`` at fixed feed ``z``.

        ``ts`` in **K**, ``ps`` in **kPa abs** (array-likes or pint arrays;
        a length-1 array broadcasts). Failed points return NaN with
        ``converged=False``. Returns a :class:`~vle.results.BatchFlashResult`.
        """
        tsa = _as_array_canonical(ts, "temperature")
        psa = _as_array_canonical(ps, "pressure")
        beta, x, y, k, iters, two, conv = self._engine.flash_pt_batch(
            tsa, psa, list(z), warm_start, parallel, tol, max_iter
        )
        return BatchFlashResult(
            beta=beta, x=x, y=y, k=k, iterations=iters, two_phase=two, converged=conv
        )

    def bubble_pressure_batch(self, xs, ts, *, parallel: bool = True, tol: float = 1e-9, max_iter: int = 200) -> BatchSaturationResult:
        """Vectorized bubble pressure: liquid rows ``xs`` (m×n) at ``ts`` (**K**)."""
        val, inc, k, conv = self._engine.bubble_pressure_batch(
            np.ascontiguousarray(xs, dtype=np.float64),
            _as_array_canonical(ts, "temperature"), parallel, tol, max_iter,
        )
        return BatchSaturationResult(value=val, incipient=inc, k=k, converged=conv)

    def bubble_temperature_batch(self, xs, ps, *, parallel: bool = True, tol: float = 1e-9, max_iter: int = 200) -> BatchSaturationResult:
        """Vectorized bubble temperature: liquid rows ``xs`` (m×n) at ``ps`` (**kPa**)."""
        val, inc, k, conv = self._engine.bubble_temperature_batch(
            np.ascontiguousarray(xs, dtype=np.float64),
            _as_array_canonical(ps, "pressure"), parallel, tol, max_iter,
        )
        return BatchSaturationResult(value=val, incipient=inc, k=k, converged=conv)

    def dew_pressure_batch(self, ys, ts, *, parallel: bool = True, tol: float = 1e-9, max_iter: int = 200) -> BatchSaturationResult:
        """Vectorized dew pressure: vapor rows ``ys`` (m×n) at ``ts`` (**K**)."""
        val, inc, k, conv = self._engine.dew_pressure_batch(
            np.ascontiguousarray(ys, dtype=np.float64),
            _as_array_canonical(ts, "temperature"), parallel, tol, max_iter,
        )
        return BatchSaturationResult(value=val, incipient=inc, k=k, converged=conv)

    def dew_temperature_batch(self, ys, ps, *, parallel: bool = True, tol: float = 1e-9, max_iter: int = 200) -> BatchSaturationResult:
        """Vectorized dew temperature: vapor rows ``ys`` (m×n) at ``ps`` (**kPa**)."""
        val, inc, k, conv = self._engine.dew_temperature_batch(
            np.ascontiguousarray(ys, dtype=np.float64),
            _as_array_canonical(ps, "pressure"), parallel, tol, max_iter,
        )
        return BatchSaturationResult(value=val, incipient=inc, k=k, converged=conv)

    def z_factor_batch(self, ts, ps, x: Sequence[float], phase: str, *, parallel: bool = True) -> np.ndarray:
        """Vectorized Z of ``phase`` over paired ``(ts, ps)`` at fixed ``x``."""
        return self._engine.z_factor_batch(
            _as_array_canonical(ts, "temperature"), _as_array_canonical(ps, "pressure"),
            list(x), phase, parallel,
        )

    def ln_phi_batch(self, ts, ps, x: Sequence[float], phase: str, *, parallel: bool = True) -> np.ndarray:
        """Vectorized ln φ̂ᵢ of ``phase`` over paired ``(ts, ps)`` at fixed ``x`` (m×n out)."""
        return self._engine.ln_phi_batch(
            _as_array_canonical(ts, "temperature"), _as_array_canonical(ps, "pressure"),
            list(x), phase, parallel,
        )

    def enthalpy_entropy_batch(self, ts, ps, x: Sequence[float], phase: str, *, parallel: bool = True) -> tuple[np.ndarray, np.ndarray]:
        """Vectorized (H, S) of ``phase`` over paired ``(ts, ps)`` at fixed ``x``."""
        return self._engine.enthalpy_entropy_batch(
            _as_array_canonical(ts, "temperature"), _as_array_canonical(ps, "pressure"),
            list(x), phase, parallel,
        )
