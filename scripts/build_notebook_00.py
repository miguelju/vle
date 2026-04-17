"""Build notebooks/00_component_database.ipynb.

This generator emits a notebook that follows the CLAUDE.md *Notebook
Conventions* (title + motivation → research-paper context → what was built
→ worked example → user exercises → references). Re-run this script if
the notebook needs to be regenerated from a single source of truth.

    $ python scripts/build_notebook_00.py

The resulting notebook is expected to execute top-to-bottom in a fresh
kernel. The generator also executes it once at the end to catch cell
errors before committing.
"""

from __future__ import annotations

import sys
from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "00_component_database.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Component Property Database — Milestone 4\n"
        "\n"
        "Every thermodynamic calculation in this library starts with the same "
        "question: *what do we know about these compounds?* Critical properties, "
        "Antoine coefficients, binary interaction parameters — all of it has to "
        "come from somewhere, and the numbers you pick change the answer. This "
        "notebook is a guided tour of the SQLite **component property database** "
        "that ships with the `vle` package: how it is structured, what has been "
        "seeded from the research paper, and how to extend it."
    ))

    # ---- Research-paper context -----------------------------------------
    cells.append(md(
        "## Context from the research paper\n"
        "\n"
        "The database is seeded from three tables in "
        "[Chapter IV — Validation](../docs/en/research-paper/chapter-4-validation.md) "
        "of the thesis. Each table gives us one category of parameters we will "
        "need throughout later milestones.\n"
        "\n"
        "**Pure-component critical properties** come from Chapter IV §4.1, which "
        "reports four multicomponent critical-point mixtures and implicitly "
        "fixes the `Tc`, `Pc`, `ω` of every pure component that appears in any "
        "of them:\n"
        "\n"
        "> *\"Peng and Robinson reported results for critical point calculations "
        "of different mixtures. Table 4.1 contains the global compositions of "
        "the systems studied [...].\"* — [§4.1](../docs/en/research-paper/chapter-4-validation.md#41-critical-point-calculations)\n"
        "\n"
        "**Activity-model binary parameters** come from Chapter IV §4.3, which "
        "gives the van Laar parameters for methanol/water at 298 K:\n"
        "\n"
        "> *\"Using the van Laar model (whose parameters are shown in Table 4.5) "
        "for the description of the liquid phase and the ideal gas model for "
        "the vapor phase [...].\"* — [§4.3 / Table 4.5](../docs/en/research-paper/chapter-4-validation.md#43-bubble-point-pressure)\n"
        "\n"
        "**Experimental VLE data** for kij regression comes from Chapter IV §4.7 "
        "Table 4.11 — 14 P-x-y points for CO₂/n-butane at 357.57 K, of which we "
        "seed the 10 used by Da Silva & Báez:\n"
        "\n"
        "> *\"Table 4.11 presents the pressure versus composition data at "
        "constant temperature used for solving the stated problem [...].\"* — [§4.7 / Table 4.11](../docs/en/research-paper/chapter-4-validation.md#47-binary-interaction-parameter-calculation)\n"
        "\n"
        "Every value in the seeded database is traceable back to one of these "
        "tables or to DIPPR via the `thermo` library."
    ))

    # ---- What was built --------------------------------------------------
    cells.append(md(
        "## What was built in Milestone 4\n"
        "\n"
        "Milestone 4 adds four pieces that together make the thermodynamic "
        "parameters a *queryable resource* rather than a scatter of constants "
        "in code:\n"
        "\n"
        "1. **SQL schema** ([`python/src/vle/db/sql/schema.sql`](../python/src/vle/db/sql/schema.sql)) "
        "   — four tables: `components`, `kij_params`, `activity_params`, "
        "   `experimental_vle`, with foreign-key constraints and lookup "
        "   indexes. All numeric columns are stored in the canonical units "
        "   defined in `CLAUDE.md` (K, kPa absolute, cm³/mol, kJ/(kmol·K), g/mol). "
        "   The schema ships inside the installed `vle` wheel.\n"
        "2. **Python package** (`vle.db`) — `connection`, `models` (dataclasses), "
        "   `queries` (CRUD), and `seed` (static SQL files + optional `thermo` "
        "   library for ~70 k additional compounds).\n"
        "3. **CLI** (`vle-db` / `python -m vle.cli.main`) — `init`, `seed`, "
        "   `validate`, `show`, `list`, `export` subcommands.\n"
        "4. **Chapter IV seed** ([`python/src/vle/db/sql/seed_chapter4.sql`](../python/src/vle/db/sql/seed_chapter4.sql)) "
        "   — 15 compounds, the van Laar methanol/water pair, the PR kij for "
        "   CO₂/n-butane, and the 10 experimental VLE points used in Table 4.11.\n"
        "\n"
        "Everything in this notebook is a thin veneer over that package. If the "
        "cells below run, the whole pipeline is healthy."
    ))

    # ---- Setup -----------------------------------------------------------
    cells.append(md(
        "## Setup\n"
        "\n"
        "The cell below imports the `vle.db` functions we will use, creates the "
        "database from schema if it does not exist yet, and seeds the Chapter IV "
        "data. It is **idempotent** — running it a second time is a no-op."
    ))
    cells.append(code(
        "from pathlib import Path\n"
        "\n"
        "from vle.db.connection import get_db_path, init_db\n"
        "from vle.db.queries import (\n"
        "    get_activity_params,\n"
        "    get_component,\n"
        "    get_experimental_vle,\n"
        "    get_kij,\n"
        "    list_components,\n"
        "    search_components,\n"
        "    set_kij,\n"
        ")\n"
        "from vle.db.seed import seed_chapter4\n"
        "\n"
        "db_path = get_db_path()\n"
        "if not db_path.exists():\n"
        "    init_db()\n"
        "    print(f'Created empty database at {db_path}')\n"
        "\n"
        "count = seed_chapter4()\n"
        "print(f'Database path: {db_path}')\n"
        "print(f'Components in database: {count}')"
    ))

    # ---- Worked example: browse -----------------------------------------
    cells.append(md(
        "## Worked example\n"
        "\n"
        "We will walk through the four kinds of question the database answers, "
        "each one tied back to a Chapter IV table.\n"
        "\n"
        "### 1. *What compounds do we have?*"
    ))
    cells.append(code(
        "comps = list_components()\n"
        "print(f'{len(comps)} components in the database:\\n')\n"
        "print(f'{\"name\":<22s} {\"formula\":<10s} {\"Tc (K)\":>10s} {\"Pc (kPa)\":>10s} {\"ω\":>8s}')\n"
        "print('-' * 64)\n"
        "for c in comps:\n"
        "    print(f'{c.name:<22s} {c.formula or \"\":<10s} {c.tc:>10.2f} {c.pc:>10.2f} {c.w:>8.4f}')"
    ))

    # ---- Worked example: search -----------------------------------------
    cells.append(md(
        "### 2. *Is this compound in the database?*\n"
        "\n"
        "`search_components` does a case-insensitive partial match over name, "
        "formula and CAS number — useful when you do not remember the exact "
        "canonical name we used."
    ))
    cells.append(code(
        "for hit in search_components('butane'):\n"
        "    print(hit.summary())"
    ))

    # ---- Worked example: component details ------------------------------
    cells.append(md(
        "### 3. *What do we know about one compound?*\n"
        "\n"
        "`get_component` returns a fully typed `ComponentRecord` dataclass. Here "
        "we pull CO₂ — one of the two compounds in the §4.7 kij regression."
    ))
    cells.append(code(
        "co2 = get_component('carbon dioxide')\n"
        "print(f'Name:        {co2.name}')\n"
        "print(f'Formula:     {co2.formula}')\n"
        "print(f'CAS:         {co2.cas_number}')\n"
        "print(f'MW (g/mol):  {co2.mw}')\n"
        "print(f'Tc (K):      {co2.tc}')\n"
        "print(f'Pc (kPa):    {co2.pc}')\n"
        "print(f'ω:           {co2.w}')\n"
        "print(f'Source:      {co2.source}')"
    ))

    # ---- Worked example: kij --------------------------------------------
    cells.append(md(
        "### 4a. *Binary interaction parameter — Table 4.12*\n"
        "\n"
        "Chapter IV §4.7 reports a regressed PR kij of **0.1357** for "
        "CO₂/n-butane at 357.57 K. The value is seeded into `kij_params`."
    ))
    cells.append(code(
        "rec = get_kij('carbon dioxide', 'n-butane', eos_model='PR', temperature=357.57)\n"
        "print(f'{rec.comp1_name} / {rec.comp2_name}  (EOS: {rec.eos_model}, T = {rec.temperature} K)')\n"
        "print(f'kij = {rec.kij}   source: {rec.source}')"
    ))

    # ---- Worked example: activity params --------------------------------
    cells.append(md(
        "### 4b. *Activity-model parameters — Table 4.5*\n"
        "\n"
        "The van Laar parameters for methanol/water at 298 K (Λ₁₂ = 0.5853, "
        "Λ₂₁ = 0.3458) are seeded into `activity_params`. Later milestones will "
        "pass them straight into the activity-coefficient module."
    ))
    cells.append(code(
        "vl = get_activity_params('methanol', 'water', model='van_laar', temperature=298.0)\n"
        "print(f'{vl.comp1_name} / {vl.comp2_name} — {vl.model} at {vl.temperature} K')\n"
        "print(f'A12 = {vl.a12}')\n"
        "print(f'A21 = {vl.a21}')\n"
        "print(f'source: {vl.source}')"
    ))

    # ---- Worked example: experimental VLE + plot ------------------------
    cells.append(md(
        "### 4c. *Experimental VLE data — Table 4.11*\n"
        "\n"
        "The ten P-x-y points we use for the §4.7 kij regression are stored in "
        "`experimental_vle`. A P-x-y plot is the usual first sanity check."
    ))
    cells.append(code(
        "import matplotlib.pyplot as plt\n"
        "\n"
        "pts = get_experimental_vle('CO2/n-butane')\n"
        "x1 = [p.x1 for p in pts]\n"
        "y1 = [p.y1 for p in pts]\n"
        "P  = [p.pressure for p in pts]\n"
        "\n"
        "fig, ax = plt.subplots(figsize=(6, 4))\n"
        "ax.plot(x1, P, 'o-', label='P vs. x₁ (liquid)')\n"
        "ax.plot(y1, P, 's--', label='P vs. y₁ (vapor)')\n"
        "ax.set_xlabel('CO₂ mole fraction')\n"
        "ax.set_ylabel('P (kPa abs.)')\n"
        "ax.set_title('CO₂ / n-butane at 357.57 K — Table 4.11')\n"
        "ax.legend(); ax.grid(True, alpha=0.3)\n"
        "fig.tight_layout()\n"
        "plt.show()"
    ))

    # ---- Exercises intro -------------------------------------------------
    cells.append(md(
        "## Your turn — exercises\n"
        "\n"
        "Two short exercises to check you can read and write the database. "
        "Template cells with `# TODO:` markers are below; the solutions are "
        "in the collapsed block at the end of the notebook."
    ))

    # ---- Exercise 1 ------------------------------------------------------
    cells.append(md(
        "### Exercise 1 — Reduced temperature at the boiling point\n"
        "\n"
        "Pull `Tc` for each of the four compounds in the **Chapter IV §4.2 "
        "adiabatic flash** — benzene, cyclohexane, methylcyclohexane, n-hexane — "
        "and compute the reduced temperature $T_r = T / T_c$ at the flash outlet "
        "temperature $T = 394.263$ K (the answer from Table 4.4). Print a small "
        "table sorted by $T_r$ ascending.\n"
        "\n"
        "**Expected**: all four $T_r$ values should fall between roughly 0.69 "
        "and 0.78 — a sanity check that the flash is well below any critical "
        "point."
    ))
    cells.append(code(
        "# TODO: for each of the four compounds, look it up with get_component()\n"
        "# TODO: compute Tr = 394.263 / comp.tc\n"
        "# TODO: print the compound name and Tr, sorted ascending by Tr\n"
        "\n"
        "T_flash = 394.263  # K, from Table 4.4\n"
        "\n"
        "compounds = [\"benzene\", \"cyclohexane\", \"methylcyclohexane\", \"n-hexane\"]\n"
        "\n"
        "# your code here\n"
    ))

    # ---- Exercise 2 ------------------------------------------------------
    cells.append(md(
        "### Exercise 2 — Seed a fictitious kij and read it back\n"
        "\n"
        "Use `set_kij` to insert an RKS kij for methanol/water of `0.042` at "
        "298 K (purely illustrative — do not commit this value to the real "
        "database), then use `get_kij` to read it back and assert the round-trip "
        "is exact.\n"
        "\n"
        "Notice that you can call `set_kij('water', 'methanol', ...)` *or* "
        "`set_kij('methanol', 'water', ...)` and the query layer normalizes — "
        "verify that reading back with the opposite order also works."
    ))
    cells.append(code(
        "# TODO: set_kij(\"methanol\", \"water\", eos_model=\"RKS\", kij=0.042,\n"
        "#                temperature=298.0, source=\"notebook demo\")\n"
        "# TODO: read it back with get_kij and assert the value matches 0.042\n"
        "# TODO: read it back with the component pair REVERSED and assert it still works\n"
        "\n"
        "# your code here\n"
    ))

    # ---- Solutions (collapsed) ------------------------------------------
    cells.append(md(
        "### Solutions (expand to see)\n"
        "\n"
        "<details>\n"
        "<summary>Click to show Exercise 1 solution</summary>\n"
        "\n"
        "```python\n"
        "T_flash = 394.263\n"
        "rows = []\n"
        "for name in compounds:\n"
        "    c = get_component(name)\n"
        "    rows.append((name, T_flash / c.tc))\n"
        "rows.sort(key=lambda r: r[1])\n"
        "for name, tr in rows:\n"
        "    print(f'{name:<20s} Tr = {tr:.4f}')\n"
        "```\n"
        "\n"
        "</details>\n"
        "\n"
        "<details>\n"
        "<summary>Click to show Exercise 2 solution</summary>\n"
        "\n"
        "```python\n"
        "set_kij('methanol', 'water', eos_model='RKS',\n"
        "        kij=0.042, temperature=298.0, source='notebook demo')\n"
        "\n"
        "rec1 = get_kij('methanol', 'water', eos_model='RKS', temperature=298.0)\n"
        "rec2 = get_kij('water', 'methanol', eos_model='RKS', temperature=298.0)\n"
        "assert rec1.kij == rec2.kij == 0.042\n"
        "print('Round-trip OK in both orderings.')\n"
        "```\n"
        "\n"
        "</details>"
    ))

    # ---- References ------------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- **Research paper**: [Chapter IV — Validation](../docs/en/research-paper/chapter-4-validation.md), "
        "tables 4.1, 4.5, 4.11, 4.12.\n"
        "- **Parameter reference**: [`docs/en/parameters/parameter_reference.md`](../docs/en/parameters/parameter_reference.md) "
        "documents the canonical unit for every column in the schema.\n"
        "- **Modernization plan**: "
        "[`MODERNIZATION_PLAN.md`](../MODERNIZATION_PLAN.md) — Phase 4 describes "
        "this database as the input to every later phase.\n"
        "- **Original Pascal program** (Ref (4)): Da Silva, F. A.; Báez, L., *Ekilib* "
        "(1989) — source of the 10 experimental VLE points in Table 4.11 and the "
        "kij = 0.1357 regression result in Table 4.12.\n"
        "- **Orbey & Sandler** (Ref (21)): source of the van Laar methanol/water "
        "parameters in Table 4.5.\n"
        "- **DIPPR 801** (via the `thermo` library): source of critical "
        "properties for the 15 seeded compounds."
    ))

    nb.cells = cells
    nb.metadata = {
        "kernelspec": {"display_name": "Python 3", "language": "python", "name": "python3"},
        "language_info": {"name": "python", "pygments_lexer": "ipython3"},
    }
    return nb


def main() -> None:
    nb = build()
    NB_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    print(f"Wrote {NB_PATH}")

    # Execute to verify cells run top-to-bottom.
    client = NotebookClient(nb, timeout=120, kernel_name="python3", resources={"metadata": {"path": str(NB_PATH.parent)}})
    client.execute()
    NB_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    print(f"Executed + saved {NB_PATH}")


if __name__ == "__main__":
    main()
