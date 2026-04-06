#!/usr/bin/env python3
"""Extract thermodynamic component properties from the ``thermo`` library.

This script pulls critical properties, boiling points, molecular weights,
and dipole moments for a list of compounds and outputs them as SQL INSERT
statements compatible with ``data/schema.sql``.

Requirements:
    pip install thermo

Usage:
    # Extract Chapter IV validation compounds
    python scripts/extract_component_data.py --preset chapter4 > data/seed_chapter4_generated.sql

    # Extract common industrial compounds
    python scripts/extract_component_data.py --preset common > data/seed_common.sql

    # Extract specific compounds by name or CAS number
    python scripts/extract_component_data.py --compounds "ethylene" "74-85-1" "acetone"

    # Extract to JSON instead of SQL
    python scripts/extract_component_data.py --preset chapter4 --format json

    # Print a summary table (no file output)
    python scripts/extract_component_data.py --preset chapter4 --format table

All output values use VLE canonical units:
    - Temperature: K
    - Pressure: kPa (absolute)
    - Molar volume: cm3/mol
    - Molecular weight: g/mol
    - Dipole moment: Debye

Source: CalebBell/thermo library (wraps DIPPR 801, ChemSep, CoolProp).
See https://github.com/CalebBell/thermo
"""

import argparse
import json
import sys
from dataclasses import dataclass, asdict
from typing import Optional


# ---------------------------------------------------------------------------
# Compound presets
# ---------------------------------------------------------------------------

CHAPTER4_COMPOUNDS = [
    # (canonical_name, identifier_for_thermo, notes)
    ("methane",           "74-82-8",   "Chapter IV: critical points (4.1), bubble T (4.5)"),
    ("ethane",            "74-84-0",   "Chapter IV: critical points (4.1), bubble T (4.5)"),
    ("propane",           "74-98-6",   "Chapter IV: critical points (4.1), bubble T (4.5)"),
    ("n-butane",          "106-97-8",  "Chapter IV: critical points (4.1), isothermal flash (4.6), kij regression (4.7), bubble T (4.5)"),
    ("n-pentane",         "109-66-0",  "Chapter IV: critical points (4.1)"),
    ("carbon dioxide",    "124-38-9",  "Chapter IV: critical points (4.1), kij regression (4.7)"),
    ("hydrogen sulfide",  "7783-06-4", "Chapter IV: critical points (4.1)"),
    ("benzene",           "71-43-2",   "Chapter IV: adiabatic flash (4.2)"),
    ("cyclohexane",       "110-82-7",  "Chapter IV: adiabatic flash (4.2)"),
    ("methylcyclohexane", "108-87-2",  "Chapter IV: adiabatic flash (4.2)"),
    ("n-hexane",          "110-54-3",  "Chapter IV: adiabatic flash (4.2)"),
    ("n-heptane",         "142-82-5",  "Chapter IV: isothermal flash (4.6)"),
    ("methanol",          "67-56-1",   "Chapter IV: bubble P (4.3)"),
    ("water",             "7732-18-5", "Chapter IV: bubble P (4.3), dew point (4.4)"),
    ("2-propanol",        "67-63-0",   "Chapter IV: dew point (4.4)"),
]

COMMON_COMPOUNDS = [
    # Light gases
    ("hydrogen",          "1333-74-0",  "Light gas"),
    ("nitrogen",          "7727-37-9",  "Light gas"),
    ("oxygen",            "7782-44-7",  "Light gas"),
    ("carbon monoxide",   "630-08-0",   "Light gas"),
    ("argon",             "7440-37-1",  "Noble gas"),
    # Additional alkanes
    ("isobutane",         "75-28-5",    "C4 isomer"),
    ("isopentane",        "78-78-4",    "C5 isomer"),
    ("neopentane",        "463-82-1",   "C5 isomer"),
    ("n-octane",          "111-65-9",   "C8 alkane"),
    ("n-nonane",          "111-84-2",   "C9 alkane"),
    ("n-decane",          "124-18-5",   "C10 alkane"),
    # Alkenes
    ("ethylene",          "74-85-1",    "C2 alkene"),
    ("propylene",         "115-07-1",   "C3 alkene"),
    ("1-butene",          "106-98-9",   "C4 alkene"),
    # Aromatics
    ("toluene",           "108-88-3",   "Aromatic"),
    ("ethylbenzene",      "100-41-4",   "Aromatic"),
    ("o-xylene",          "95-47-6",    "Aromatic"),
    ("m-xylene",          "108-38-3",   "Aromatic"),
    ("p-xylene",          "106-42-3",   "Aromatic"),
    # Acid gases
    ("sulfur dioxide",    "7446-09-5",  "Acid gas"),
    # Alcohols
    ("ethanol",           "64-17-5",    "Alcohol"),
    ("1-propanol",        "71-23-8",    "Alcohol"),
    ("1-butanol",         "71-36-3",    "Alcohol"),
    # Other oxygenates
    ("acetone",           "67-64-1",    "Ketone"),
    ("diethyl ether",     "60-29-7",    "Ether"),
    ("acetic acid",       "64-19-7",    "Carboxylic acid"),
    # Nitrogen compounds
    ("ammonia",           "7664-41-7",  "Inorganic"),
    ("methylamine",       "74-89-5",    "Amine"),
    # Halogenated
    ("chloroform",        "67-66-3",    "Halogenated"),
    ("dichloromethane",   "75-09-2",    "Halogenated"),
]


# ---------------------------------------------------------------------------
# Data extraction
# ---------------------------------------------------------------------------

@dataclass
class ComponentData:
    """Extracted component properties in VLE canonical units."""
    name: str
    formula: Optional[str] = None
    cas_number: Optional[str] = None
    mw: Optional[float] = None              # g/mol
    tc: Optional[float] = None              # K
    pc: Optional[float] = None              # kPa (absolute)
    w: Optional[float] = None               # acentric factor (dimensionless)
    zc: Optional[float] = None              # critical compressibility (dimensionless)
    vc: Optional[float] = None              # cm3/mol
    tb: Optional[float] = None              # K
    dipole_moment: Optional[float] = None   # Debye
    source: str = "thermo/DIPPR"
    notes: Optional[str] = None


def extract_component(identifier: str, canonical_name: Optional[str] = None,
                      notes: Optional[str] = None) -> ComponentData:
    """Extract properties for a single compound from the thermo library.

    Args:
        identifier: Compound name or CAS number (passed to ``thermo.Chemical``).
        canonical_name: Override name for the database (e.g., "n-butane" instead
            of thermo's default "butane"). If None, uses thermo's name in lowercase.
        notes: Optional notes to attach to the record.

    Returns:
        ComponentData with all available properties in canonical units.

    Unit conversions applied:
        - Pressure: Pa (thermo) -> kPa (VLE canonical) via / 1000
        - Volume: m3/mol (thermo) -> cm3/mol (VLE canonical) via * 1e6
        - Temperature: K (thermo) -> K (VLE canonical) — no conversion needed
        - Dipole: Debye (thermo) -> Debye (VLE canonical) — no conversion needed
    """
    from thermo import Chemical

    c = Chemical(identifier)

    name = canonical_name or (c.name.lower() if c.name else identifier.lower())

    return ComponentData(
        name=name,
        formula=c.formula,
        cas_number=c.CAS,
        mw=_round(c.MW, 4),
        tc=_round(c.Tc, 4),
        pc=_round(c.Pc / 1000, 4) if c.Pc else None,      # Pa -> kPa
        w=_round(c.omega, 6),
        zc=_round(c.Zc, 6),
        vc=_round(c.Vc * 1e6, 4) if c.Vc else None,       # m3/mol -> cm3/mol
        tb=_round(c.Tb, 4),
        dipole_moment=_round(c.dipole, 4),
        source="thermo/DIPPR",
        notes=notes,
    )


def _round(value, decimals):
    """Round a value, returning None if the input is None."""
    return round(value, decimals) if value is not None else None


def extract_preset(preset: str) -> list[ComponentData]:
    """Extract all compounds for a named preset.

    Args:
        preset: "chapter4" or "common".

    Returns:
        List of ComponentData records.
    """
    if preset == "chapter4":
        compound_list = CHAPTER4_COMPOUNDS
    elif preset == "common":
        compound_list = COMMON_COMPOUNDS
    else:
        raise ValueError(f"Unknown preset: {preset}. Use 'chapter4' or 'common'.")

    results = []
    for canonical_name, identifier, notes in compound_list:
        try:
            data = extract_component(identifier, canonical_name=canonical_name, notes=notes)
            results.append(data)
            print(f"  OK: {data.name:25s}  Tc={data.tc:>10}  Pc={data.pc:>10}  w={data.w}",
                  file=sys.stderr)
        except Exception as e:
            print(f"  ERROR: {canonical_name}: {e}", file=sys.stderr)
    return results


def extract_by_names(names: list[str]) -> list[ComponentData]:
    """Extract compounds by name or CAS number.

    Args:
        names: List of compound names or CAS numbers.

    Returns:
        List of ComponentData records.
    """
    results = []
    for name in names:
        try:
            data = extract_component(name)
            results.append(data)
            print(f"  OK: {data.name:25s}  Tc={data.tc:>10}  Pc={data.pc:>10}  w={data.w}",
                  file=sys.stderr)
        except Exception as e:
            print(f"  ERROR: {name}: {e}", file=sys.stderr)
    return results


# ---------------------------------------------------------------------------
# Output formatters
# ---------------------------------------------------------------------------

def _sql_val(v) -> str:
    """Format a Python value for SQL."""
    if v is None:
        return "NULL"
    if isinstance(v, str):
        return "'" + v.replace("'", "''") + "'"
    return str(v)


def format_sql(components: list[ComponentData]) -> str:
    """Format extracted data as SQL INSERT statements.

    Output is compatible with data/schema.sql and can be executed directly
    against the VLE component database.
    """
    lines = [
        "-- Component data extracted from thermo library (DIPPR 801 backed)",
        f"-- Extracted on: {_today()}",
        "-- All values in canonical units: K, kPa (absolute), cm3/mol, g/mol, Debye",
        "--",
        "-- Usage: sqlite3 data/components.db < this_file.sql",
        "--   or:  PYTHONPATH=python/src python -m vle.cli.main init",
        "--        sqlite3 data/components.db < this_file.sql",
        "",
    ]

    for c in components:
        lines.append(
            f"INSERT OR IGNORE INTO components "
            f"(name, formula, cas_number, mw, tc, pc, w, zc, vc, tb, dipole_moment, source, notes) "
            f"VALUES ({_sql_val(c.name)}, {_sql_val(c.formula)}, {_sql_val(c.cas_number)}, "
            f"{_sql_val(c.mw)}, {_sql_val(c.tc)}, {_sql_val(c.pc)}, {_sql_val(c.w)}, "
            f"{_sql_val(c.zc)}, {_sql_val(c.vc)}, {_sql_val(c.tb)}, "
            f"{_sql_val(c.dipole_moment)}, {_sql_val(c.source)}, {_sql_val(c.notes)});"
        )

    return "\n".join(lines) + "\n"


def format_json(components: list[ComponentData]) -> str:
    """Format extracted data as JSON."""
    return json.dumps([asdict(c) for c in components], indent=2, default=str) + "\n"


def format_table(components: list[ComponentData]) -> str:
    """Format extracted data as a human-readable table."""
    header = f"{'Name':25s}  {'Formula':10s}  {'CAS':12s}  {'MW':>8s}  {'Tc (K)':>10s}  {'Pc (kPa)':>10s}  {'w':>8s}  {'Zc':>8s}  {'Tb (K)':>10s}"
    sep = "-" * len(header)
    lines = [header, sep]
    for c in components:
        lines.append(
            f"{c.name:25s}  {c.formula or '':10s}  {c.cas_number or '':12s}  "
            f"{c.mw or 0:8.2f}  {c.tc or 0:10.3f}  {c.pc or 0:10.3f}  "
            f"{c.w or 0:8.4f}  {c.zc or 0:8.4f}  {c.tb or 0:10.3f}"
        )
    lines.append(sep)
    lines.append(f"Total: {len(components)} compounds")
    return "\n".join(lines) + "\n"


def _today() -> str:
    from datetime import date
    return date.today().isoformat()


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="Extract thermodynamic properties from the thermo library.",
        epilog="Examples:\n"
               "  python scripts/extract_component_data.py --preset chapter4\n"
               "  python scripts/extract_component_data.py --preset common --format json\n"
               "  python scripts/extract_component_data.py --compounds ethylene acetone\n",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--preset", choices=["chapter4", "common"],
                       help="Extract a predefined set of compounds")
    group.add_argument("--compounds", nargs="+", metavar="NAME",
                       help="Extract specific compounds by name or CAS number")
    parser.add_argument("--format", choices=["sql", "json", "table"], default="sql",
                        help="Output format (default: sql)")

    args = parser.parse_args()

    # Check thermo is installed
    try:
        import thermo  # noqa: F401
    except ImportError:
        print("ERROR: The 'thermo' library is required.", file=sys.stderr)
        print("Install with: pip install thermo", file=sys.stderr)
        sys.exit(1)

    # Extract
    if args.preset:
        components = extract_preset(args.preset)
    else:
        components = extract_by_names(args.compounds)

    if not components:
        print("No compounds extracted.", file=sys.stderr)
        sys.exit(1)

    # Format and output
    formatters = {
        "sql": format_sql,
        "json": format_json,
        "table": format_table,
    }
    output = formatters[args.format](components)
    sys.stdout.write(output)


if __name__ == "__main__":
    main()
