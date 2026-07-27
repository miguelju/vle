#!/usr/bin/env python3
"""Test harness for check 5 of hooks/pre-commit (the "<N> notebooks" claim gate).

CLAUDE.md requires that this check be run against deliberately broken trees
before it is changed:

    If a check produces a false positive, **fix the check**, don't bypass it.
    Both false-positive classes it currently guards against ... were found by
    running the gate against deliberately broken trees — do the same before
    changing it.

This script makes that cheap. It **extracts the live regexes and constants from
`hooks/pre-commit` itself** rather than restating them, so the harness cannot
silently drift away from the thing it is testing — if the hook's pattern
changes, these cases are re-run against the new pattern.

Run it after any edit to check 5::

    ~/miniconda3/envs/vle/bin/python hooks/test-notebook-count-check.py

Exit status is 0 when every case behaves as specified, 1 otherwise.
"""

from __future__ import annotations

import pathlib
import re
import sys

HOOK = pathlib.Path(__file__).resolve().parent / "pre-commit"

# The count the hook computes for a claim to be judged against. Fixed here so
# the cases stay meaningful regardless of how many notebooks are on disk today.
N = 19


def load_from_hook() -> dict:
    """Pull check 5's regexes and constants out of the hook source.

    The hook is a bash script wrapping a Python heredoc, so the definitions are
    extracted textually and exec'd in an empty namespace. Keeping one source of
    truth matters more here than elegance: a copy-pasted regex in this file
    would pass its own tests while the hook shipped something else.
    """
    src = HOOK.read_text(encoding="utf-8")
    wanted = ("COUNT", "SUBSET_ADJ", "SUBSET_VERB", "HISTORICAL", "HISTORICAL_TRAILING")
    ns: dict = {"re": re}
    for name in wanted:
        # Match `NAME = ...` through to the line before the next definition or
        # a dedent, capturing multi-line re.compile(...) / set literals.
        m = re.search(
            rf"^\s*{name} = (.+?)(?=\n\s*(?:#|[A-Z_]+ =|for |if |\Z))",
            src,
            re.S | re.M,
        )
        if not m:
            sys.exit(f"FATAL: could not extract {name} from {HOOK}")
        exec(f"{name} = {m.group(1).strip()}", ns)  # noqa: S102 - trusted local file
    missing = [w for w in wanted if w not in ns]
    if missing:
        sys.exit(f"FATAL: extraction produced no value for {missing}")
    return ns


def verdict(text: str, env: dict) -> list[str]:
    """Reproduce check 5's decision for one document, returning its complaints."""
    out = []
    for m in env["COUNT"].finditer(text):
        if int(m.group(1)) == N:
            continue
        if env["HISTORICAL"].search(text[max(0, m.start() - 80) : m.end()]):
            continue
        if env["HISTORICAL_TRAILING"].search(text[m.end() : m.end() + 40]):
            continue
        if (m.group(2) or "").lower() in env["SUBSET_ADJ"]:
            continue
        if env["SUBSET_VERB"].search(text[max(0, m.start() - 30) : m.start()]):
            continue
        out.append(" ".join(m.group(0).split()))
    return out


# (description, document text, should the gate flag it?)
CASES: list[tuple[str, str, bool]] = [
    # --- correct claims must never fire -------------------------------------
    ("correct, bare noun", "the 19 notebooks ship in the repo", False),
    ("correct + 'executable'", "all 19 executable notebooks run top-to-bottom", False),
    ("correct + 'Jupyter'", "ships 19 Jupyter notebooks today", False),
    # --- wrong claims must fire ---------------------------------------------
    ("WRONG, bare noun", "the 20 notebooks ship", True),
    ("WRONG + 'executable' (original guard)", "all 20 executable notebooks", True),
    ("WRONG + 'Jupyter' (THE BLIND SPOT)", "It also ships 20 Jupyter notebooks", True),
    ("WRONG + arbitrary adjective", "the 22 teaching notebooks", True),
    ("WRONG, line-wrapped across a newline", "20 executable\nnotebooks", True),
    # --- false-positive class 1: version strings ----------------------------
    ("version string, singular noun", "Create v0.3.0 notebook for M7", False),
    ("decimal before the noun", "v12.20 notebooks", False),
    ("digits inside a longer number", "in 2026 notebooks grew", False),
    ("singular noun never matches", "the 20 Jupyter notebook", False),
    # --- false-positive class 2: past-tense, milestone-scoped records -------
    ("historical 'then-' prefix", "the then-15-notebook collection", False),
    ("historical marker BEFORE the claim", "at M11 there were 15 notebooks", False),
    ("historical marker AFTER the claim", "the 15 notebooks that existed at M11", False),
    (
        "a wrong claim is NOT excused by a later milestone ref",
        "the 20 notebooks, all of which shipped at M13",
        True,
    ),
    # --- false-positive class 3: prose gaps (from widening the slot) --------
    ("two-word prose gap", "7 systems in notebooks are shown", False),
    ("longer prose gap", "4 cases used by the notebooks", False),
    ("punctuation in the gap", "(7 systems), notebooks follow", False),
    # --- false-positive class 4: subset counts (from widening the slot) -----
    ("subset adjective", "Create 3 placeholder notebooks advertising M7.2", False),
    ("subset via creation verb", "added 2 notebooks this milestone", False),
    ("subset verb through markdown bold", "**Create 3 notebooks** (~0.5h)", False),
    ("subset adjective 'new'", "the 4 new notebooks land in v0.9.0", False),
]


def main() -> int:
    env = load_from_hook()
    print(f"check 5 harness — regexes loaded from {HOOK}")
    print(f"asserted notebook count N = {N}\n")
    print(f"{'result':7s} {'expect':7s}  case")
    print("-" * 76)
    bad = 0
    for desc, text, should_fail in CASES:
        got = bool(verdict(text, env))
        ok = got == should_fail
        bad += not ok
        print(
            f"{'FLAG' if got else 'pass':7s} {'FLAG' if should_fail else 'pass':7s}  "
            f"{'OK ' if ok else '>>> MISMATCH'} {desc}"
        )
    print("-" * 76)
    if bad:
        print(f"FAIL: {bad} of {len(CASES)} cases did not behave as specified")
        return 1
    print(f"PASS: all {len(CASES)} cases behaved as specified")
    return 0


if __name__ == "__main__":
    sys.exit(main())
