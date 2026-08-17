"""Petroleum characterization — a crude assay into pseudocomponents.

Everywhere else in this package you name a compound and get its measured
properties. Crude oil does not work that way: it is thousands of molecules
nobody has separated, and what a refinery has instead is an **assay** — a
distillation curve and a density. This module turns that assay into a list of
:class:`~vle.components.Component` objects a flash calculation can use, by
slicing the curve into narrow boiling **pseudocomponents** and correlating each
slice's critical properties from its boiling point and gravity.

Quick start
-----------
>>> from vle.petroleum import Assay
>>> assay = Assay(
...     fractions=[0.0, 0.1, 0.3, 0.5, 0.7, 0.9, 0.95],
...     temperatures=[310.0, 380.0, 460.0, 540.0, 620.0, 730.0, 790.0],
...     basis="tbp",
...     api_gravity=35.0,
... )
>>> cuts = assay.cuts(n=5)
>>> round(cuts[0]["tb"], 1)              # lightest cut's boiling point, K
370.1
>>> system, z = assay.to_system(n=12, eos="PR")
>>> res = system.flash_pt(500.0, 200.0, z)   # T in K, P in kPa
>>> 0.0 < res.beta < 1.0                     # a real two-phase split
True

Units
-----
Canonical engine units throughout: **temperature in K**, **pressure in kPa
absolute**, molecular weight in g/mol, volumes in cm³/mol. Scalar temperature
and pressure arguments also accept ``pint`` quantities or ``"<value> <unit>"``
strings (``"350 degC"``, ``"10 mmHg"``), resolved through the same registry the
rest of the package uses. Distillation-curve arrays are plain floats in K.

Accuracy
--------
These are **correlations**, not measurements, and the numbers they produce
inherit that. Against ten pure hydrocarbons the critical temperature is good to
about 1 %, critical pressure to 5 %, and molecular weight to 6 % — and a real
vacuum-residue cut sits well outside the range any of them were fitted on. See
the ``vle_thermo::petroleum`` Rust module docs for the per-correlation table.

See also
--------
``docs/en/petroleum/README.md`` — the learning guide: the domain from first
principles, every correlation written out with its published units, the
validation provenance, and the design decisions behind this API.

``notebooks/15_petroleum_characterization.ipynb`` — the milestone notebook,
which builds an assay, tabulates the cuts, plots the curves, and flashes the
result.
"""

from __future__ import annotations

from typing import Any, Iterable, Optional, Sequence

from vle import _engine
from vle.components import Component
from vle.units import Q_, ureg

__all__ = [
    "Assay",
    "watson_k",
    "api_from_sg",
    "sg_from_api",
    "average_boiling_points",
    "convert_curve",
    "cut_curve",
    "estimate",
    "acentric_factor",
    "ideal_gas_cp",
    "ideal_gas_cp_coeffs",
    "normal_boiling_point",
    "boiling_point_at_pressure",
    "vapor_pressure",
    "PROPERTY_METHODS",
    "DISTILLATION_BASES",
]

#: Critical-property correlation families accepted by ``method=``.
PROPERTY_METHODS = ("api", "riazi-daubert-1980", "kesler-lee", "twu")

#: Distillation-curve bases accepted by ``basis=``.
DISTILLATION_BASES = ("d86", "tbp", "d2887", "efv")


def _coerce(value: Any, target: str) -> float:
    """Coerce ``value`` to a float in unit ``target``.

    Same contract as :func:`vle.steam._coerce`: a plain number is assumed to be
    in the canonical unit already, a ``pint`` quantity is converted, and a
    ``"<value> <unit>"`` string is parsed (the only path that works for pint's
    offset units like ``degC`` and ``barg``).
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
    """Temperature to K."""
    return _coerce(value, "K")


def _p(value: Any) -> float:
    """Pressure to kPa absolute."""
    return _coerce(value, "kPa")


def _temps(values: Iterable[Any]) -> list[float]:
    """A distillation curve's temperatures to a list of K."""
    return [_t(v) for v in values]


class Assay:
    """A crude assay, and the machinery to characterize it.

    Parameters
    ----------
    fractions
        Cumulative fraction distilled at each curve point, 0–1, strictly
        increasing. Volume fraction on every basis except ``"d2887"``, which is
        a chromatogram and therefore weight fraction.
    temperatures
        Curve temperature at each fraction. Floats are K; strings and ``pint``
        quantities are converted.
    basis
        Which apparatus produced the curve — one of :data:`DISTILLATION_BASES`.
        Converted to TBP internally before anything else happens, because every
        property correlation is written against true boiling points.
    bulk_sg, api_gravity
        The barrel's gravity, one or the other. Per-cut gravities are derived by
        holding the Watson characterization factor constant at whatever value
        makes the cuts blend back to this number exactly.
    sg_fractions, sg_values
        A measured gravity curve, as an alternative to a single bulk number.
        **Prefer this whenever the lab reported one** — it is strictly better
        information than an assumption of chemical uniformity.
    method
        Critical-property correlation family; one of :data:`PROPERTY_METHODS`.
        Defaults to ``"api"`` (Riazi–Daubert 1987), the API's own
        recommendation and the best all-rounder on the validation set.
    zc_method
        Critical-compressibility correlation, used only by ``"kesler-lee"``
        (the only family that publishes no critical volume of its own).
    name_prefix
        Generated component names are ``f"{name_prefix}-{i}"``, 1-based.

    Notes
    -----
    Every method that produces cuts takes the same three cutting arguments:

    ``n``
        Cut into this many pseudocomponents of equal volume.
    ``equal_temperature=True``
        Space those ``n`` cuts by boiling range instead of by volume.
    ``boundaries``
        Cut at these explicit temperatures instead — how you model *real*
        products (naphtha / kerosene / diesel / AGO), since the boundaries are
        the tower's own draw specifications.
    """

    def __init__(
        self,
        *,
        fractions: Sequence[float],
        temperatures: Sequence[Any],
        basis: str = "tbp",
        bulk_sg: Optional[float] = None,
        api_gravity: Optional[float] = None,
        sg_fractions: Optional[Sequence[float]] = None,
        sg_values: Optional[Sequence[float]] = None,
        method: str = "api",
        zc_method: str = "lee-kesler",
        name_prefix: str = "PC",
    ) -> None:
        if bulk_sg is not None and api_gravity is not None:
            raise ValueError("pass either `bulk_sg` or `api_gravity`, not both")
        if api_gravity is not None:
            bulk_sg = sg_from_api(api_gravity)

        self._assay = _engine.Assay(
            fractions=[float(x) for x in fractions],
            temperatures=_temps(temperatures),
            basis=basis,
            bulk_sg=bulk_sg,
            sg_fractions=None if sg_fractions is None else [float(x) for x in sg_fractions],
            sg_values=None if sg_values is None else [float(x) for x in sg_values],
            method=method,
            zc_method=zc_method,
            name_prefix=name_prefix,
        )
        self.basis = basis
        self.method = method

    # -- inspection ------------------------------------------------------

    def tbp_curve(self) -> tuple[list[float], list[float]]:
        """The assay converted to TBP, as ``(fractions, temperatures_in_K)``.

        Useful for plotting the conversion the pipeline does silently: a D86
        curve and its TBP equivalent on the same axes is the clearest picture
        of what fractionation efficiency actually means.
        """
        return self._assay.tbp_curve()

    def watson_k(self, n: int = 30, **cut) -> float:
        """The assay's Watson characterization factor (textbook definition).

        Roughly 12.5–13 for a paraffinic crude, 11–12 naphthenic, ~10 aromatic.
        """
        return self._assay.watson_k(n=n, **cut)

    # -- characterization -------------------------------------------------

    def cuts(
        self,
        n: Optional[int] = None,
        *,
        boundaries: Optional[Sequence[Any]] = None,
        equal_temperature: bool = False,
    ) -> list[dict]:
        """Characterize into pseudocomponents, as a list of dicts.

        One dict per cut, carrying where it came from (``fraction``,
        ``mole_fraction``, ``x_lower``/``x_upper``, ``t_lower``/``t_upper``),
        what it is (``tb``, ``sg``, ``api_gravity``, ``watson_k``, ``mw``,
        ``tc``, ``pc``, ``vc``, ``zc``, ``omega``) and what the engine will use
        (``cp_coeffs``, ``psat_coeffs``, ``zra``, ``liquid_volume``).

        Dicts rather than objects because this is a table: pass the list
        straight to ``pandas.DataFrame`` and you have the cut summary a
        refinery engineer expects to see.
        """
        return self._assay.characterize(
            n=n,
            boundaries=None if boundaries is None else _temps(boundaries),
            equal_temperature=equal_temperature,
        )

    def components(
        self,
        n: Optional[int] = None,
        *,
        boundaries: Optional[Sequence[Any]] = None,
        equal_temperature: bool = False,
    ) -> tuple[list[Component], list[float]]:
        """Characterize into ``(components, mole_fractions)``.

        The components are ordinary :class:`~vle.components.Component`
        instances — the same type :class:`vle.System` accepts for a named
        compound. Nothing downstream knows or cares that these were correlated
        rather than measured.
        """
        names, tc, pc, omega, mw, tb, psat, cp, z = self._assay.mixture(
            n=n,
            boundaries=None if boundaries is None else _temps(boundaries),
            equal_temperature=equal_temperature,
        )
        cuts = self.cuts(n=n, boundaries=boundaries, equal_temperature=equal_temperature)
        comps = [
            Component(
                name=names[i],
                tc=tc[i],
                pc=pc[i],
                omega=omega[i],
                mw=mw[i],
                tb=tb[i],
                zc=cuts[i]["zc"],
                vc=cuts[i]["vc"],
                psat_coeffs=list(psat[i]),
                cp_coeffs=list(cp[i]),
                liquid_volume=cuts[i]["liquid_volume"],
                # M20: what the refinery methods read per cut.
                solubility_param=cuts[i]["solubility_param"],
                watson_k=cuts[i]["watson_k"],
                zra=cuts[i]["zra"],
                psat_source=f"pseudo-Antoine anchored on Tb and Tc ({self.method})",
                cp_source=f"Kesler-Lee ideal-gas Cp° at K_W = {cuts[i]['watson_k']:.3f}",
            )
            for i in range(len(names))
        ]
        return comps, list(z)

    def to_system(
        self,
        n: Optional[int] = None,
        *,
        boundaries: Optional[Sequence[Any]] = None,
        equal_temperature: bool = False,
        **system_kwargs: Any,
    ) -> Any:
        """Characterize and build a ready-to-flash :class:`vle.System`.

        Extra keyword arguments go straight to :class:`vle.System` — ``eos``,
        ``mixing_rule``, ``kij`` and so on. Returns ``(system, z)`` so the feed
        composition travels with the system that describes it.

        A cubic EOS with classical mixing and an all-zero ``kij`` is the right
        default here, and not only for simplicity: with every ``kij`` zero the
        engine's mixing rule collapses to its O(N) form (Milestone 18), which
        is what makes a several-hundred-component assay tractable at all.
        """
        from vle import System  # imported here to avoid a circular import

        comps, z = self.components(
            n=n, boundaries=boundaries, equal_temperature=equal_temperature
        )
        return System(comps, **system_kwargs), z

    def __repr__(self) -> str:
        return repr(self._assay)


# -- module-level correlations -------------------------------------------
#
# Each is a thin pass-through to the Rust binding. They exist so the pieces are
# usable (and teachable) on their own, without building a whole Assay.


def watson_k(tb: Any, sg: float) -> float:
    """Watson (UOP) characterization factor from boiling point and gravity."""
    return _engine.petro_watson_k(_t(tb), float(sg))


def api_from_sg(sg: float) -> float:
    """API gravity (°API) from specific gravity at 60/60 °F."""
    return _engine.petro_api_from_sg(float(sg))


def sg_from_api(api: float) -> float:
    """Specific gravity at 60/60 °F from API gravity."""
    return _engine.petro_sg_from_api(float(api))


def average_boiling_points(d86_10: Any, d86_30: Any, d86_50: Any, d86_70: Any, d86_90: Any) -> dict:
    """The five average boiling points of a fraction from its ASTM D86 curve.

    Returns a dict keyed ``vabp``, ``wabp``, ``mabp``, ``cabp``, ``meabp``, all
    in K. They are not interchangeable — correlations name the one they want,
    and ``meabp`` is the usual default.
    """
    return _engine.petro_average_boiling_points(
        _t(d86_10), _t(d86_30), _t(d86_50), _t(d86_70), _t(d86_90)
    )


def convert_curve(
    fractions: Sequence[float],
    temperatures: Sequence[Any],
    from_basis: str,
    to_basis: str,
    sg: Optional[float] = None,
) -> list[float]:
    """Convert a distillation curve between bases. Returns temperatures in K.

    ``sg`` is needed only when the route touches EFV.
    """
    return _engine.petro_convert_curve(
        [float(x) for x in fractions], _temps(temperatures), from_basis, to_basis, sg
    )


def cut_curve(
    fractions: Sequence[float],
    temperatures: Sequence[Any],
    n: Optional[int] = None,
    *,
    boundaries: Optional[Sequence[Any]] = None,
    equal_temperature: bool = False,
) -> list[dict]:
    """Slice a **TBP** curve into cuts, without characterizing them."""
    return _engine.petro_cut_curve(
        [float(x) for x in fractions],
        _temps(temperatures),
        n=n,
        boundaries=None if boundaries is None else _temps(boundaries),
        equal_temperature=equal_temperature,
    )


def estimate(tb: Any, sg: float, method: str = "api", zc_method: str = "lee-kesler") -> dict:
    """Estimate one pseudocomponent's properties from boiling point and gravity.

    Returns a dict keyed ``tb``, ``sg``, ``watson_k``, ``mw``, ``tc``, ``pc``,
    ``vc``, ``zc``, ``omega`` in canonical units.
    """
    return _engine.petro_estimate(_t(tb), float(sg), method=method, zc_method=zc_method)


def acentric_factor(tb: Any, tc: Any, pc: Any, sg: float) -> float:
    """Lee–Kesler / Kesler–Lee acentric factor of a petroleum fraction."""
    return _engine.petro_acentric_factor(_t(tb), _t(tc), _p(pc), float(sg))


def ideal_gas_cp(watson_k_value: float, t: Any, mw: Optional[float] = None) -> float:
    """Ideal-gas Cp° of a fraction.

    Returns kJ/(kg·K) by default, or kJ/(kmol·K) if ``mw`` is given.
    """
    if mw is None:
        return _engine.petro_ideal_gas_cp_mass(float(watson_k_value), _t(t))
    return _engine.petro_ideal_gas_cp_molar(float(watson_k_value), float(mw), _t(t))


def ideal_gas_cp_coeffs(watson_k_value: float, mw: float) -> list[float]:
    """Ideal-gas ``Cp°/R = Σₖ aₖ Tᵏ`` coefficients (T in K) for a fraction.

    The same five-entry form :attr:`vle.components.Component.cp_coeffs` uses,
    so a correlated fraction drops into the enthalpy path unchanged.
    """
    return _engine.petro_ideal_gas_cp_coeffs(float(watson_k_value), float(mw))


def normal_boiling_point(t: Any, p: Any, watson_k_value: Optional[float] = None) -> float:
    """Maxwell–Bonnell atmospheric equivalent temperature, K.

    Converts a boiling point observed under vacuum into the normal boiling
    point every characterization correlation actually wants — the step that
    makes ASTM D1160 and D2892 vacuum data usable.
    """
    return _engine.petro_normal_boiling_point(_t(t), _p(p), watson_k_value)


def boiling_point_at_pressure(tb: Any, p: Any, watson_k_value: Optional[float] = None) -> float:
    """Maxwell–Bonnell boiling temperature at pressure ``p``, K."""
    return _engine.petro_boiling_point_at_pressure(_t(tb), _p(p), watson_k_value)


def vapor_pressure(t: Any, tb: Any, watson_k_value: Optional[float] = None) -> float:
    """Maxwell–Bonnell vapor pressure of a fraction at temperature ``t``, kPa."""
    return _engine.petro_vapor_pressure(_t(t), _t(tb), watson_k_value)
