# Data Extraction Scripts

Utility scripts for extracting and validating thermodynamic component properties from public data sources. These scripts were used to generate the static seed data in `data/seed_chapter4.sql` and can be re-run to update or extend the dataset.

## Prerequisites

```bash
pip install thermo          # Required for extraction (~70,000 compounds via DIPPR)
pip install CoolProp        # Optional, for cross-validation (~100 high-accuracy fluids)
```

## Scripts

### `extract_component_data.py`

Extracts Tc, Pc, acentric factor (w), Zc, Vc, Tb, MW, and dipole moment from the `thermo` library (which wraps the DIPPR 801 database). Outputs SQL, JSON, or a summary table.

**All output values use VLE canonical units** (see `CLAUDE.md` for definitions):
- Temperature: **K**
- Pressure: **kPa** (absolute, never gauge)
- Molar volume: **cm3/mol**
- Molecular weight: **g/mol**
- Dipole moment: **Debye**

#### Usage

```bash
# Extract the 15 Chapter IV validation compounds as SQL
python scripts/extract_component_data.py --preset chapter4

# Extract ~30 common industrial compounds as SQL
python scripts/extract_component_data.py --preset common

# Extract specific compounds by name or CAS number
python scripts/extract_component_data.py --compounds ethylene "74-85-1" acetone

# Output as JSON instead of SQL
python scripts/extract_component_data.py --preset chapter4 --format json

# Output as a human-readable table
python scripts/extract_component_data.py --preset chapter4 --format table

# Save SQL to a file
python scripts/extract_component_data.py --preset chapter4 > data/seed_chapter4_generated.sql

# Load generated SQL into the database
PYTHONPATH=python/src python -m vle.cli.main init
sqlite3 data/components.db < data/seed_chapter4_generated.sql
```

#### Presets

| Preset | Compounds | Description |
|--------|-----------|-------------|
| `chapter4` | 15 | All compounds used in the thesis validation (Chapter IV) |
| `common` | ~30 | Common industrial chemicals (light gases, alkanes, aromatics, alcohols, etc.) |

#### Unit conversions applied

The `thermo` library returns values in SI base units. This script converts them to VLE canonical units:

| Property | thermo unit | VLE canonical | Conversion |
|----------|------------|---------------|------------|
| Pressure (Pc) | Pa | kPa | `/ 1000` |
| Molar volume (Vc) | m3/mol | cm3/mol | `* 1e6` |
| Temperature (Tc, Tb) | K | K | none |
| Acentric factor (w) | — | — | none |
| Dipole moment | Debye | Debye | none |

### `cross_validate_coolprop.py`

Compares Tc, Pc, and w from `thermo` against CoolProp reference values. Reports percentage deviations and flags any that exceed a configurable tolerance. Use this to verify that the extracted data is consistent across sources.

#### Usage

```bash
# Validate Chapter IV compounds (default tolerance: 0.5%)
python scripts/cross_validate_coolprop.py

# Custom tolerance
python scripts/cross_validate_coolprop.py --tolerance 1.0

# Validate specific compounds by CAS number
python scripts/cross_validate_coolprop.py --compounds 74-82-8 7732-18-5
```

#### Sample output

```
Cross-validating 15 compounds (tolerance: 0.5%)
Name                       Tc diff%  Pc diff%   w diff%    Status
----------------------------------------------------------------------
methane                       0.000     0.000     0.000      PASS
ethane                        0.000     0.000     0.000      PASS
water                         0.000     0.000     0.019      PASS
...
----------------------------------------------------------------------
Results: 12 passed, 0 failed, 3 skipped
```

Compounds are skipped if they are not available in CoolProp (e.g., methylcyclohexane).

## How the seed data was generated

The static seed files shipped with the project were created as follows:

```bash
# 1. Install thermo
pip install thermo

# 2. Extract Chapter IV compounds
python scripts/extract_component_data.py --preset chapter4 --format table
# (reviewed output manually to verify values are reasonable)

# 3. Cross-validate against CoolProp
pip install CoolProp
python scripts/cross_validate_coolprop.py
# (all available compounds passed within 0.5%)

# 4. The verified values were used to write data/seed_chapter4.sql
#    (the SQL file also includes binary params and experimental VLE data
#    that were manually transcribed from the thesis Chapter IV tables)
```

## Adding new compounds

To add a compound not in the presets:

```bash
# 1. Check if thermo has it
python scripts/extract_component_data.py --compounds "dimethyl ether" --format table

# 2. If OK, generate SQL and append to a seed file
python scripts/extract_component_data.py --compounds "dimethyl ether" >> data/seed_custom.sql

# 3. Load into the database
sqlite3 data/components.db < data/seed_custom.sql

# 4. Or use the CLI directly
PYTHONPATH=python/src python -m vle.cli.main seed --source thermo
```

## Data sources

| Library | Compounds | Backend | License |
|---------|-----------|---------|---------|
| [thermo](https://github.com/CalebBell/thermo) | ~70,000 | DIPPR 801, ChemSep, CoolProp | MIT |
| [CoolProp](http://www.coolprop.org/) | ~100 | High-accuracy reference EOS | MIT |
| [ChemSep](http://www.chemsep.org/) | ~400 | Open-source XML database | BSD |
