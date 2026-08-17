# Scripts

Utility scripts that support the repo but are not part of the shipped library.
Two families live here: the **data-extraction** scripts that generated the
component database, and the **external-model** scripts that call the OpenAI API
for a second opinion or an illustration.

The data-extraction scripts pull and validate thermodynamic component properties from public data sources. They were used to generate the static seed data in `python/src/vle/db/sql/seed_chapter4.sql` (shipped inside the wheel) and can be re-run to update or extend the dataset.

*(The `build_notebook_*.py` and `build_index.py` scripts, which generate the
notebooks under `notebooks/`, are not documented individually — each carries its
own module docstring.)*

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
Cross-validating 24 compounds (tolerance: 0.5%)
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

## External-model scripts

Two scripts call the OpenAI API. Both are **stdlib-only** — plain `urllib`, no
`openai` package — so neither installs anything into the `vle` conda
environment, and both read the API key from 1Password *at call time* through the
shared helper `_openai_key.py`. The key is never written to disk, echoed, or
exported into a long-lived environment variable.

Both write down what the external model actually produced. That is the point of
them: a record you can go back to, including where the model was wrong.

### `second_opinion.py`

Sends a prompt file to a reasoning model and records the answer verbatim, with a
provenance header carrying the model, the reasoning effort, the token counts and
the cost. Used for the external audit of the optimization plans — see
[`SECOND_OPINION_TRIAL.md`](../docs/plans/engine/SECOND_OPINION_TRIAL.md) and the
lessons in
[`OPTIMIZATION_AUDIT_HISTORY.md`](../docs/plans/engine/OPTIMIZATION_AUDIT_HISTORY.md).

```bash
~/miniconda3/envs/vle/bin/python scripts/second_opinion.py PROMPT.md \
    --model gpt-5.6-sol --effort xhigh --out RESPONSE.md
```

### `generate_image.py`

Generates a conceptual illustration with `gpt-image-2` and writes a sidecar
`<out>.json` recording the model, size, quality, the exact prompt, the token
usage, the cost, and any prompt revision the API reports.

```bash
~/miniconda3/envs/vle/bin/python scripts/generate_image.py \
    scripts/prompts/distillation-bases.md \
    --out docs/assets/distillation-bases.png \
    --size 2048x1024 --quality high
```

**Use it only for figures that carry no data.** An image model does not compute;
it draws something that looks like a computation. Every figure in this repo that
shows numbers is produced by matplotlib or plotly from the engine's own output,
and must stay that way. The one generated illustration currently in the docs —
the four distillation methods in
[`docs/en/petroleum/`](../docs/en/petroleum/README.md#a-note-on-the-illustration)
— is schematic, and that section explains what it took and what is still wrong
with it.

Prompts live in `scripts/prompts/`. Cost is token-based: roughly \$0.01 per
1536x1024 render at `--quality low`, \$0.08 at `medium`, \$0.15–0.32 at `high`.

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

# 4. The verified values were used to write python/src/vle/db/sql/seed_chapter4.sql
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
