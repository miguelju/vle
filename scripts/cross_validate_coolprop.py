#!/usr/bin/env python3
"""Cross-validate component properties between thermo and CoolProp.

Compares Tc, Pc, and acentric factor (w) from the ``thermo`` library against
``CoolProp`` reference values for compounds available in both. Reports
percentage deviations and flags any that exceed a tolerance (default 0.5%).

Requirements:
    pip install thermo CoolProp

Usage:
    python scripts/cross_validate_coolprop.py

    # Custom tolerance (e.g., 1%)
    python scripts/cross_validate_coolprop.py --tolerance 1.0

    # Validate specific compounds
    python scripts/cross_validate_coolprop.py --compounds methane ethane water

All comparisons use VLE canonical units:
    - Temperature: K
    - Pressure: kPa (absolute)
"""

import argparse
import sys


# CoolProp fluid names differ from thermo names — this maps CAS to CoolProp ID
CAS_TO_COOLPROP = {
    "74-82-8":    "Methane",
    "74-84-0":    "Ethane",
    "74-98-6":    "Propane",
    "106-97-8":   "n-Butane",
    "109-66-0":   "n-Pentane",
    "110-54-3":   "n-Hexane",
    "142-82-5":   "n-Heptane",
    "111-65-9":   "n-Octane",
    "111-84-2":   "n-Nonane",
    "124-18-5":   "n-Decane",
    "75-28-5":    "IsoButane",
    "78-78-4":    "Isopentane",
    "463-82-1":   "Neopentane",
    "124-38-9":   "CarbonDioxide",
    "7783-06-4":  "HydrogenSulfide",
    "71-43-2":    "Benzene",
    "108-88-3":   "Toluene",
    "110-82-7":   "CycloHexane",
    "7732-18-5":  "Water",
    "67-56-1":    "Methanol",
    "64-17-5":    "Ethanol",
    "67-64-1":    "Acetone",
    "7664-41-7":  "Ammonia",
    "1333-74-0":  "Hydrogen",
    "7727-37-9":  "Nitrogen",
    "7782-44-7":  "Oxygen",
    "7440-37-1":  "Argon",
    "630-08-0":   "CarbonMonoxide",
    "7446-09-5":  "SulfurDioxide",
    "74-85-1":    "Ethylene",
    "115-07-1":   "Propylene",  # propene
    "67-63-0":    "n-Propanol",  # CoolProp doesn't have 2-propanol but has n-propanol
}

# Chapter IV compounds by CAS
CHAPTER4_CAS = [
    "74-82-8", "74-84-0", "74-98-6", "106-97-8", "109-66-0",
    "124-38-9", "7783-06-4", "71-43-2", "110-82-7", "108-87-2",
    "110-54-3", "142-82-5", "67-56-1", "7732-18-5", "67-63-0",
]


def validate_compound(cas: str, tolerance_pct: float) -> dict:
    """Compare thermo vs CoolProp for a single compound.

    Args:
        cas: CAS registry number.
        tolerance_pct: Maximum acceptable deviation in percent.

    Returns:
        Dict with comparison results, or None if compound not in CoolProp.
    """
    from thermo import Chemical
    import CoolProp.CoolProp as CP

    coolprop_name = CAS_TO_COOLPROP.get(cas)
    if not coolprop_name:
        return None

    c = Chemical(cas)
    thermo_name = c.name or cas

    # Thermo values (convert to canonical units)
    t_tc = c.Tc                              # K
    t_pc = c.Pc / 1000 if c.Pc else None     # Pa -> kPa
    t_w = c.omega

    # CoolProp values
    try:
        cp_tc = CP.PropsSI("Tcrit", coolprop_name)       # K
        cp_pc = CP.PropsSI("pcrit", coolprop_name) / 1000  # Pa -> kPa
        cp_w = CP.PropsSI("acentric", coolprop_name)
    except Exception as e:
        return {"name": thermo_name, "cas": cas, "error": str(e)}

    def pct_diff(a, b):
        if a is None or b is None or b == 0:
            return None
        return abs(a - b) / abs(b) * 100

    tc_diff = pct_diff(t_tc, cp_tc)
    pc_diff = pct_diff(t_pc, cp_pc)
    w_diff = pct_diff(t_w, cp_w)

    passed = all(
        d is not None and d < tolerance_pct
        for d in [tc_diff, pc_diff, w_diff]
    )

    return {
        "name": thermo_name,
        "cas": cas,
        "thermo_tc": t_tc, "coolprop_tc": cp_tc, "tc_diff_pct": tc_diff,
        "thermo_pc": t_pc, "coolprop_pc": cp_pc, "pc_diff_pct": pc_diff,
        "thermo_w": t_w, "coolprop_w": cp_w, "w_diff_pct": w_diff,
        "passed": passed,
    }


def main():
    # --- Parse command-line arguments ---
    # argparse builds a CLI interface automatically: --help, type checking, defaults.
    parser = argparse.ArgumentParser(
        description="Cross-validate thermo vs CoolProp component properties.",
    )
    # --tolerance: max allowed % deviation before a compound is flagged as FAIL.
    parser.add_argument("--tolerance", type=float, default=0.5,
                        help="Maximum acceptable deviation in %% (default: 0.5)")
    # --compounds: optional list of CAS numbers; if omitted, uses the default
    # Chapter IV compounds from the research paper.
    # nargs="*" means "zero or more values", so the user can pass multiple CAS numbers.
    parser.add_argument("--compounds", nargs="*", metavar="CAS",
                        help="CAS numbers to validate (default: Chapter IV compounds)")
    args = parser.parse_args()

    # --- Check that required third-party libraries are installed ---
    # `thermo` and `CoolProp` are external packages that provide reference
    # property data. If they're missing, we print a helpful install message.
    # `# noqa: F401` tells the linter to ignore the "imported but unused"
    # warning — we're only importing here to check availability.
    try:
        import thermo  # noqa: F401
        import CoolProp  # noqa: F401
    except ImportError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        print("Install with: pip install thermo CoolProp", file=sys.stderr)
        sys.exit(1)

    # Use user-supplied CAS numbers, or fall back to the default list.
    cas_list = args.compounds or CHAPTER4_CAS

    # --- Print table header ---
    print(f"Cross-validating {len(cas_list)} compounds (tolerance: {args.tolerance}%)")
    print(f"{'Name':25s}  {'Tc diff%':>8s}  {'Pc diff%':>8s}  {'w diff%':>8s}  {'Status':>8s}")
    print("-" * 70)

    # --- Loop over each compound and validate ---
    pass_count = 0
    skip_count = 0
    fail_count = 0

    for cas in cas_list:
        result = validate_compound(cas, args.tolerance)

        # result is None when the compound couldn't be found in either library.
        if result is None:
            print(f"{'(CAS: ' + cas + ')':25s}  {'—':>8s}  {'—':>8s}  {'—':>8s}  {'SKIP':>8s}")
            skip_count += 1
            continue

        # result contains "error" when the lookup succeeded but comparison failed.
        if "error" in result:
            print(f"{result['name']:25s}  ERROR: {result['error']}")
            fail_count += 1
            continue

        # Format the % deviations for Tc, Pc, and acentric factor (w).
        # :.3f means 3 decimal places (e.g., "0.123"). If a value is None
        # (property not available), show a dash instead.
        status = "PASS" if result["passed"] else "FAIL"
        tc_str = f"{result['tc_diff_pct']:.3f}" if result['tc_diff_pct'] is not None else "—"
        pc_str = f"{result['pc_diff_pct']:.3f}" if result['pc_diff_pct'] is not None else "—"
        w_str = f"{result['w_diff_pct']:.3f}" if result['w_diff_pct'] is not None else "—"

        print(f"{result['name']:25s}  {tc_str:>8s}  {pc_str:>8s}  {w_str:>8s}  {status:>8s}")

        if result["passed"]:
            pass_count += 1
        else:
            fail_count += 1

    # --- Print summary and exit ---
    print("-" * 70)
    print(f"Results: {pass_count} passed, {fail_count} failed, {skip_count} skipped")

    # Exit with code 1 (error) if any compound failed validation.
    # This lets CI pipelines detect failures automatically.
    if fail_count > 0:
        sys.exit(1)


# Standard Python idiom: only run main() when this file is executed directly
# (e.g., `python cross_validate_coolprop.py`), not when it's imported as a
# module by another script.
if __name__ == "__main__":
    main()
