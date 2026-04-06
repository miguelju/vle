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

from pathlib import Path
from typing import Optional

from vle.db.connection import get_connection, seed_from_sql, get_db_path
from vle.db.models import ComponentRecord
from vle.db.queries import upsert_component

_DATA_DIR = Path(__file__).resolve().parents[4] / "data"


def seed_chapter4() -> int:
    """Seed the database with Chapter IV validation compounds.

    Executes ``data/seed_chapter4.sql`` which contains 15 compounds,
    binary interaction parameters, and experimental VLE data.

    Returns:
        Total number of components in the database after seeding.
    """
    sql_path = _DATA_DIR / "seed_chapter4.sql"
    return seed_from_sql(sql_path)


def seed_common() -> int:
    """Seed the database with common industrial compounds.

    Executes ``data/seed_common.sql`` (~50 compounds).

    Returns:
        Total number of components in the database after seeding.
    """
    sql_path = _DATA_DIR / "seed_common.sql"
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
