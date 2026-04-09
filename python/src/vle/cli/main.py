"""VLE Component Database CLI.

Lightweight command-line tool for initializing, seeding, and validating
the component property database. For interactive browsing and editing,
use the Jupyter notebook: ``notebooks/00_component_database.ipynb``.

Usage::

    python -m vle.cli.main init
    python -m vle.cli.main seed --source chapter4
    python -m vle.cli.main validate chapter4
    python -m vle.cli.main show methane
    python -m vle.cli.main list
    python -m vle.cli.main export --format json
"""

import argparse
import json
import sys
from pathlib import Path


def cmd_init(args: argparse.Namespace) -> None:
    """Create the database from schema."""
    from vle.db.connection import init_db
    db_path = init_db()
    print(f"Database created at {db_path}")


def cmd_seed(args: argparse.Namespace) -> None:
    """Seed the database from a data source."""
    from vle.db.connection import init_db, get_db_path

    # Ensure DB exists
    if not get_db_path().exists():
        init_db()

    source = args.source
    if source == "chapter4":
        from vle.db.seed import seed_chapter4
        count = seed_chapter4()
        print(f"Seeded Chapter IV data. Total components: {count}")
    elif source == "common":
        from vle.db.seed import seed_common
        count = seed_common()
        print(f"Seeded common compounds. Total components: {count}")
    elif source == "thermo":
        from vle.db.seed import seed_from_thermo
        count = seed_from_thermo()
        print(f"Seeded {count} compounds from thermo library.")
    else:
        print(f"Unknown source: {source}. Use 'chapter4', 'common', or 'thermo'.", file=sys.stderr)
        sys.exit(1)


def cmd_validate(args: argparse.Namespace) -> None:
    """Validate that required data is present."""
    from vle.db.queries import get_component, get_activity_params, get_experimental_vle, get_kij

    target = args.target
    if target != "chapter4":
        print(f"Unknown validation target: {target}. Use 'chapter4'.", file=sys.stderr)
        sys.exit(1)

    chapter4_compounds = [
        "methane", "ethane", "propane", "n-butane", "n-pentane",
        "carbon dioxide", "hydrogen sulfide",
        "benzene", "cyclohexane", "methylcyclohexane", "n-hexane", "n-heptane",
        "methanol", "water", "2-propanol",
    ]

    errors = []
    for name in chapter4_compounds:
        comp = get_component(name)
        if comp is None:
            errors.append(f"MISSING: {name}")
        elif not comp.has_critical_props():
            errors.append(f"INCOMPLETE: {name} (missing Tc, Pc, or w)")
        else:
            print(f"  OK: {comp.summary()}")

    # Check binary params
    vl_params = get_activity_params("methanol", "water", "van_laar", temperature=298.0)
    if vl_params is None:
        errors.append("MISSING: van Laar params for methanol/water at 298 K (Table 4.5)")
    else:
        print(f"  OK: van Laar methanol/water A12={vl_params.a12} A21={vl_params.a21}")

    # Check kij regression result
    kij_rec = get_kij("carbon dioxide", "n-butane", "PR", temperature=357.57)
    if kij_rec is None:
        errors.append("MISSING: kij for CO2/n-butane PR at 357.57 K (Table 4.12)")
    else:
        print(f"  OK: kij CO2/n-butane PR = {kij_rec.kij} at {kij_rec.temperature} K")

    # Check experimental data
    exp_data = get_experimental_vle("CO2/n-butane")
    if len(exp_data) < 10:
        errors.append(f"INCOMPLETE: CO2/n-butane experimental data ({len(exp_data)} points, expected >= 10)")
    else:
        print(f"  OK: CO2/n-butane experimental data ({len(exp_data)} points)")

    if errors:
        print(f"\nValidation FAILED with {len(errors)} error(s):")
        for e in errors:
            print(f"  {e}")
        sys.exit(1)
    else:
        print("\nValidation PASSED: all Chapter IV data present.")


def cmd_show(args: argparse.Namespace) -> None:
    """Show all properties for a component."""
    from vle.db.queries import get_component

    comp = get_component(args.name)
    if comp is None:
        print(f"Component not found: {args.name}", file=sys.stderr)
        sys.exit(1)

    # Format spec `:>25s` means: right-align (>) the string (s) in a
    # field 25 characters wide, padding with spaces on the left. This
    # makes all the labels line up neatly in a column, e.g.:
    #                      Name: methane
    #                   Formula: CH4
    #                        MW: 16.04
    print(f"{'Name':>25s}: {comp.name}")
    print(f"{'Formula':>25s}: {comp.formula or '—'}")
    print(f"{'CAS':>25s}: {comp.cas_number or '—'}")
    print(f"{'MW (g/mol)':>25s}: {comp.mw}")
    print(f"{'--- Critical Properties':>25s}  {'---':>10s}")
    print(f"{'Tc (K)':>25s}: {comp.tc}")
    print(f"{'Pc (kPa abs)':>25s}: {comp.pc}")
    print(f"{'w (acentric)':>25s}: {comp.w}")
    print(f"{'Zc':>25s}: {comp.zc}")
    print(f"{'Vc (cm3/mol)':>25s}: {comp.vc}")
    print(f"{'--- Boiling Point':>25s}  {'---':>10s}")
    print(f"{'Tb (K)':>25s}: {comp.tb}")
    print(f"{'--- Other':>25s}  {'---':>10s}")
    print(f"{'Dipole (Debye)':>25s}: {comp.dipole_moment}")
    print(f"{'Source':>25s}: {comp.source or '—'}")
    if comp.notes:
        print(f"{'Notes':>25s}: {comp.notes}")


def cmd_list(args: argparse.Namespace) -> None:
    """List all components."""
    from vle.db.queries import list_components

    comps = list_components()
    if not comps:
        print("No components in database. Run 'vle-db seed --source chapter4' first.")
        return

    # Table header using Python f-string format specifiers:
    #   :>3s  — right-align (>), 3 chars wide, string (s)  →  "  #"
    #   :25s  — left-align (default for strings), 25 chars wide  →  "Name                     "
    #   :10s  — left-align, 10 chars wide  →  "Formula   "
    #   :>10s — right-align, 10 chars wide →  "    Tc (K)"
    #   :>8s  — right-align, 8 chars wide  →  "       w"
    # Left-align is the default for strings; > overrides it to right-align.
    # Numbers (Tc, Pc, w) are right-aligned so decimal points will line up
    # with the data rows below.
    print(f"{'#':>3s}  {'Name':25s}  {'Formula':10s}  {'Tc (K)':>10s}  {'Pc (kPa)':>10s}  {'w':>8s}")
    print("-" * 75)
    for i, c in enumerate(comps, 1):
        tc_str = f"{c.tc:.2f}" if c.tc else "—"
        pc_str = f"{c.pc:.2f}" if c.pc else "—"
        w_str = f"{c.w:.4f}" if c.w else "—"
        print(f"{i:3d}  {c.name:25s}  {c.formula or '':10s}  {tc_str:>10s}  {pc_str:>10s}  {w_str:>8s}")
    print(f"\nTotal: {len(comps)} components")


def cmd_export(args: argparse.Namespace) -> None:
    """Export database to JSON."""
    from vle.db.queries import list_components
    from dataclasses import asdict

    comps = list_components()
    data = [asdict(c) for c in comps]
    print(json.dumps(data, indent=2, default=str))


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="vle-db",
        description="VLE Component Property Database Manager",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # init
    subparsers.add_parser("init", help="Create database from schema")

    # seed
    seed_parser = subparsers.add_parser("seed", help="Seed database with component data")
    seed_parser.add_argument("--source", required=True, choices=["chapter4", "common", "thermo"],
                            help="Data source: chapter4 (static), common (static), thermo (optional dep)")

    # validate
    validate_parser = subparsers.add_parser("validate", help="Validate required data is present")
    validate_parser.add_argument("target", choices=["chapter4"], help="Validation target")

    # show
    show_parser = subparsers.add_parser("show", help="Show component properties")
    show_parser.add_argument("name", help="Component name")

    # list
    subparsers.add_parser("list", help="List all components")

    # export
    export_parser = subparsers.add_parser("export", help="Export database to JSON")
    export_parser.add_argument("--format", default="json", choices=["json"], help="Export format")

    args = parser.parse_args()
    cmd_map = {
        "init": cmd_init,
        "seed": cmd_seed,
        "validate": cmd_validate,
        "show": cmd_show,
        "list": cmd_list,
        "export": cmd_export,
    }
    cmd_map[args.command](args)


if __name__ == "__main__":
    main()
