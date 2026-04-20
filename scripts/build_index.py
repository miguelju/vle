"""Generate ``notebooks/index.ipynb`` — the landing page for JupyterLab.

Scans every ``*.ipynb`` file under ``notebooks/`` (except the index itself),
pulls the first H1 title and the first paragraph after it from each notebook,
and writes a fresh ``index.ipynb`` with:

* A welcome header explaining what VLE is.
* A bulleted, clickable list of every discovered notebook.
* A short "Getting started" section covering how to open a notebook, where
  the persistent workspace lives, and where the seeded DB is.

Run from the repo root:

    $ python scripts/build_index.py

The output is deterministic — rerunning with no notebook changes produces a
byte-identical file (modulo the timestamp-free nbformat header). Wire this
into ``deploy/scripts/deploy.sh`` so the hub always ships a fresh index.
"""

from __future__ import annotations

import re
from pathlib import Path

import nbformat as nbf

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_DIR = REPO_ROOT / "notebooks"
INDEX_PATH = NB_DIR / "index.ipynb"


# ---------------------------------------------------------------------------
# Notebook metadata extraction
# ---------------------------------------------------------------------------

def _trim_blurb(text: str, max_chars: int = 200) -> str:
    """Return at most the first 1–2 sentences of ``text``, ≤ ``max_chars``.

    Splits on sentence boundaries (``. ! ?`` followed by whitespace). If even
    the first sentence is longer than ``max_chars``, truncates at the last
    word boundary so the blurb never ends mid-word.
    """
    text = " ".join(text.split()).strip()
    if not text:
        return ""

    # Strip simple markdown emphasis markers so "*word*" becomes "word".
    text = re.sub(r"\*([^*]+)\*", r"\1", text)

    sentences = re.split(r"(?<=[.!?])\s+", text)
    out = ""
    for sentence in sentences[:2]:
        candidate = f"{out} {sentence}".strip() if out else sentence
        if len(candidate) > max_chars and out:
            break
        out = candidate
        if len(out) >= max_chars:
            break
    if not out:
        out = text

    if len(out) > max_chars:
        cut = out.rfind(" ", 0, max_chars - 1)
        if cut < 0:
            cut = max_chars - 1
        out = out[:cut].rstrip(" ,;:—-") + "…"
    return out


def _clean_title(title: str) -> str:
    """Strip project-internal ``— Milestone N`` suffixes from an H1 title."""
    return re.sub(r"\s*[—-]\s*Milestone\s+\d+\s*$", "", title).strip()


def extract_title_and_blurb(path: Path) -> tuple[str, str]:
    """Return ``(H1 title, one-line blurb)`` for ``path``.

    Walks markdown cells until one contains an H1 (``# Title``); uses the
    next non-empty, non-heading paragraph as the blurb. Falls back to the
    filename stem if no H1 is found.
    """
    nb = nbf.read(path, as_version=4)
    for cell in nb.cells:
        if cell.cell_type != "markdown":
            continue
        lines = cell.source.splitlines()
        h1_idx = None
        title = None
        for i, raw in enumerate(lines):
            s = raw.strip()
            if s.startswith("# ") and not s.startswith("## "):
                title = s[2:].strip()
                h1_idx = i
                break
        if title is None:
            continue

        paragraph: list[str] = []
        started = False
        for raw in lines[h1_idx + 1 :]:
            s = raw.strip()
            if not s:
                if started:
                    break
                continue
            if s.startswith("#"):
                break
            paragraph.append(s)
            started = True
        return _clean_title(title), _trim_blurb(" ".join(paragraph))

    return path.stem.replace("_", " ").title(), ""


# ---------------------------------------------------------------------------
# Discovery
# ---------------------------------------------------------------------------

def discover_notebooks() -> list[Path]:
    """Return every user-facing notebook under ``notebooks/``, sorted.

    Skips the index itself and any notebook whose filename starts with ``_``
    (a convention for "private" work-in-progress notebooks).
    """
    found: list[Path] = []
    for nb in sorted(NB_DIR.glob("*.ipynb")):
        if nb.name == INDEX_PATH.name:
            continue
        if nb.name.startswith("_"):
            continue
        found.append(nb)
    return found


def format_link_prefix(name: str) -> str:
    """Return the leading digit run of a filename, or an empty string.

    ``"00_component_database"`` -> ``"00"``; ``"intro"`` -> ``""``.
    Used so entries render as "00 — Title" when a numeric prefix is present.
    """
    m = re.match(r"(\d+)", name)
    return m.group(1) if m else ""


# ---------------------------------------------------------------------------
# Notebook assembly
# ---------------------------------------------------------------------------

def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Welcome --------------------------------------------------------
    cells.append(md(
        "# Welcome to the VLE notebook hub\n"
        "\n"
        "This JupyterLab environment is a playground for the **VLE (Vapor–"
        "Liquid Equilibrium) calculator** — a thermodynamic library for "
        "multicomponent mixtures. It ships with cubic equations of state, "
        "activity-coefficient models, and flash algorithms that reproduce the "
        "validation cases from the original research paper.\n"
        "\n"
        "You do not need to run this page — it is just a landing index. "
        "**Click a notebook link below to open it.**"
    ))

    # ---- What is VLE ----------------------------------------------------
    cells.append(md(
        "## About this environment\n"
        "\n"
        "The Python package `vle` (with a Rust core via PyO3) exposes:\n"
        "\n"
        "- **Pure-component and mixture equations of state** — Peng-Robinson, "
        "  RKS, Schmidt-Wenzel, Patel-Teja, and more.\n"
        "- **Activity-coefficient models** — Wilson, van Laar, Margules, "
        "  Scatchard-Hildebrand, UNIFAC.\n"
        "- **Flash calculations** — bubble/dew T & P, isothermal and "
        "  adiabatic flash, mixture critical point.\n"
        "- **A component property database** seeded from the research paper's "
        "  Chapter IV validation cases (plus ~70 k DIPPR compounds via the "
        "  optional `thermo` library).\n"
        "- **A dimensional `units` library** so every calculation is done in "
        "  explicit physical units (K, kPa abs, cm³/mol, …).\n"
        "\n"
        "Everything is documented in the research paper translation under "
        "`docs/en/research-paper/`."
    ))

    # ---- Available notebooks -------------------------------------------
    nb_entries = discover_notebooks()
    parts: list[str] = [
        "## Notebooks",
        "",
        "Each notebook below is a guided, runnable walk-through of one "
        "piece of the library. Click to open it in a new tab.",
        "",
    ]
    if not nb_entries:
        parts.append(
            "_No notebooks discovered yet. Check back after the next "
            "milestone._"
        )
    else:
        for path in nb_entries:
            title, blurb = extract_title_and_blurb(path)
            prefix = format_link_prefix(path.stem)
            display = f"{prefix} — {title}" if prefix else title
            suffix = f" — {blurb}" if blurb else ""
            parts.append(f"- [{display}](./{path.name}){suffix}")
    cells.append(md("\n".join(parts)))

    # ---- Getting started ------------------------------------------------
    cells.append(md(
        "## Getting started\n"
        "\n"
        "**Opening a notebook** — click any link above, or use the file "
        "browser in the left sidebar to navigate `notebooks/`.\n"
        "\n"
        "**Your persistent workspace** is at `~/work/`. Anything you save "
        "there (edited notebooks, new files, plots) survives server "
        "restarts. Files written outside `~/work/` are **not** persistent.\n"
        "\n"
        "**The seeded component database** lives at "
        "`~/work/data/components.db`. It is pre-populated with the 15 "
        "Chapter IV validation compounds plus binary parameters and "
        "experimental VLE data. Open it from Python with:\n"
        "\n"
        "```python\n"
        "from vle.db import list_components, get_component\n"
        "for c in list_components()[:5]:\n"
        "    print(c.name, c.tc, c.pc)\n"
        "```\n"
        "\n"
        "**Re-seeding the database** — if you ever want a fresh copy:\n"
        "\n"
        "```bash\n"
        "rm ~/work/data/components.db\n"
        "vle-db init && vle-db seed --source chapter4\n"
        "```\n"
        "\n"
        "**Help & docs** — the research-paper translation under "
        "`docs/en/research-paper/` explains the theory behind every "
        "notebook. Parameter units are catalogued in "
        "`docs/en/parameters/parameter_reference.md`."
    ))

    nb.cells = cells
    nb.metadata = {
        "kernelspec": {
            "display_name": "Python 3",
            "language": "python",
            "name": "python3",
        },
        "language_info": {"name": "python", "pygments_lexer": "ipython3"},
    }
    return nb


def main() -> None:
    INDEX_PATH.parent.mkdir(parents=True, exist_ok=True)
    nb = build()
    INDEX_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    n = sum(1 for _ in discover_notebooks())
    print(f"Wrote {INDEX_PATH} ({n} notebooks indexed)")


if __name__ == "__main__":
    main()
