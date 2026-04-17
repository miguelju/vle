"""Seeding logic for the VLE component database.

Supports two modes:
1. Static SQL files (always available, no external dependencies)
2. Optional ``thermo`` library for on-demand seeding of any compound

All seeded values use canonical units:
- Temperature: **K**
- Pressure: **kPa** (absolute)
- Molar volume: **cm3/mol**
- Molecular weight: **g/mol**
"""

import os
from contextlib import contextmanager
from importlib import resources
from pathlib import Path
from typing import Iterator, Optional

from vle.db.connection import get_connection, seed_from_sql, get_db_path
from vle.db.models import ComponentRecord
from vle.db.queries import upsert_component


@contextmanager
def _resolve_seed_file(filename: str) -> Iterator[Path]:
    """Yield a filesystem Path to a bundled seed file.

    When ``VLE_SEED_DIR`` is set, seeds are read from that directory
    verbatim. Otherwise the file is loaded from the ``vle.db.sql`` package
    resource; ``importlib.resources.as_file`` materializes it to a temporary
    path for callers that need a real file path (e.g. ``seed_from_sql``).
    """
    env_dir = os.environ.get("VLE_SEED_DIR")
    if env_dir:
        candidate = Path(env_dir) / filename
        if not candidate.exists():
            raise FileNotFoundError(
                f"VLE_SEED_DIR is {env_dir} but {candidate} does not exist."
            )
        yield candidate
        return

    resource = resources.files("vle.db.sql").joinpath(filename)
    with resources.as_file(resource) as real_path:
        yield real_path


def seed_chapter4() -> int:
    """Seed the database with Chapter IV validation compounds.

    Runs the bundled ``seed_chapter4.sql`` (15 compounds + binary parameters
    + experimental VLE points) shipped inside the wheel. Override the source
    directory with the ``VLE_SEED_DIR`` environment variable.

    Returns:
        Total number of components in the database after seeding.
    """
    with _resolve_seed_file("seed_chapter4.sql") as sql_path:
        return seed_from_sql(sql_path)


def seed_common() -> int:
    """Seed the database with common industrial compounds.

    Executes ``seed_common.sql`` from ``VLE_SEED_DIR`` (or the packaged
    ``vle.db.sql`` resource) — ~50 compounds.

    Returns:
        Total number of components in the database after seeding.
    """
    with _resolve_seed_file("seed_common.sql") as sql_path:
        return seed_from_sql(sql_path)


def seed_from_thermo(compound_names: Optional[list[str]] = None) -> int:
    """Seed the database from the ``thermo`` Python library (optional dependency).

    The ``thermo`` library (by CalebBell) wraps DIPPR, ChemSep, and CoolProp
    data for ~70,000 compounds. Install with: ``pip install thermo``

    Args:
        compound_names: List of compound names or CAS numbers to seed.
            If None, seeds a default set of ~50 common industrial compounds.

    Returns:
        Number of compounds successfully seeded.

    Raises:
        ImportError: If ``thermo`` is not installed.
    """
    try:
        from thermo import Chemical
    except ImportError:
        raise ImportError(
            "The 'thermo' library is required for this seeding source. "
            "Install it with: pip install thermo\n"
            "Or use 'vle-db seed --source chapter4' for static seed data."
        )

    if compound_names is None:
        compound_names = _DEFAULT_THERMO_COMPOUNDS

    conn = get_connection()
    count = 0
    try:
        for name in compound_names:
            try:
                c = Chemical(name)
                comp = ComponentRecord(
                    name=c.name.lower() if c.name else name.lower(),
                    formula=c.formula,
                    cas_number=c.CAS,
                    mw=c.MW,
                    tc=c.Tc,
                    pc=c.Pc / 1000 if c.Pc else None,  # Pa -> kPa
                    w=c.omega,
                    zc=c.Zc,
                    vc=c.Vc * 1e6 if c.Vc else None,  # m3/mol -> cm3/mol
                    tb=c.Tb,
                    dipole_moment=c.dipole,
                    source="thermo/DIPPR",
                )
                upsert_component(comp, conn)
                count += 1
            except Exception as e:
                print(f"  Warning: Could not seed '{name}': {e}")
        return count
    finally:
        conn.close()


# Default compounds for thermo seeding (common industrial chemicals)
_DEFAULT_THERMO_COMPOUNDS = [
    # Light gases
    "hydrogen", "nitrogen", "oxygen", "carbon monoxide", "argon",
    # C1-C10 alkanes
    "methane", "ethane", "propane", "n-butane", "isobutane",
    "n-pentane", "isopentane", "neopentane", "n-hexane", "n-heptane",
    "n-octane", "n-nonane", "n-decane",
    # Alkenes
    "ethylene", "propylene", "1-butene",
    # Aromatics
    "benzene", "toluene", "ethylbenzene", "o-xylene", "m-xylene", "p-xylene",
    # Cycloalkanes
    "cyclohexane", "methylcyclohexane",
    # Acid gases
    "carbon dioxide", "hydrogen sulfide", "sulfur dioxide",
    # Alcohols
    "methanol", "ethanol", "1-propanol", "2-propanol", "1-butanol",
    # Other oxygenates
    "water", "acetone", "diethyl ether", "acetic acid",
    # Amines / Nitrogen compounds
    "ammonia", "methylamine",
    # Halogenated
    "chloroform", "dichloromethane",
]
