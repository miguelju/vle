#!/usr/bin/env python3
"""Write the push brief: what this push changes, and which documents describe it.

Called by ``hooks/claude-push-gate`` just before a ``git push`` from a Claude
Code session. The result, ``.claude/push-brief.md`` (gitignored), is the input
of the *agent* hook that runs beside the gate.

Why a file. An agent hook can read files but, as documented, cannot run
commands — so it cannot ask git what is being pushed. Before this brief
existed the agent could only compare ROADMAP.md with TODO.md, which is why it
passed pushes whose documentation was stale in both files at once: it never
knew what had changed. The brief gives it the change.

What is in it:

* the commits and files this push adds on top of ``origin/main``;
* for every changed area of the repo, the documents CLAUDE.md's *Release &
  Push Rules* say must describe that area, each marked ``touched`` or
  ``NOT touched`` by this push;
* every line the push adds to a Markdown file that reads like a completion,
  count or performance claim — the lines a reviewer has to check.

This script judges nothing. "NOT touched" is not an error: most code changes
do not affect most documents. Deciding whether an untouched document is now
wrong is the agent's job (and before that, the author's).

Usage:  push-brief.py <repo-root> [<command being run>]
"""

from __future__ import annotations

import datetime
import fnmatch
import pathlib
import re
import subprocess
import sys

# Area of the repo -> the documents that describe it. Taken from the numbered
# list under "Release & Push Rules" in CLAUDE.md; when that list changes, this
# table changes with it (hooks/test-claude-push-gate.sh checks that every
# document named here exists).
DOC_MAP: list[tuple[str, list[str], list[str]]] = [
    (
        "engine (Rust thermodynamics core)",
        ["engine/src/*", "engine/benches/*", "engine/examples/*", "engine/tests/*", "engine/Cargo.toml"],
        [
            "README.md",
            "engine/README.md",
            "ROADMAP.md",
            "TODO.md",
            "CLAUDE.md",
            "docs/plans/MODERNIZATION_PLAN.md",
            "docs/plans/engine/OPTIMIZATION_PLAN_PART1.md",
            "docs/plans/engine/OPTIMIZATION_PLAN_PART2.md",
        ],
    ),
    (
        "Python package and bindings",
        ["python/src/*", "python/tests/*", "python/pyproject.toml", "engine/src/py_bindings.rs"],
        ["python/README.md", "README.md", "ROADMAP.md", "TODO.md"],
    ),
    (
        "steam tables crate",
        ["steam/src/*", "steam/Cargo.toml"],
        ["steam/README.md", "README.md", "ROADMAP.md", "TODO.md"],
    ),
    (
        "units crate",
        ["units/src/*", "units/Cargo.toml"],
        ["docs/en/units/dimensional-analysis.md", "README.md"],
    ),
    (
        "WebAssembly wrapper",
        ["wasm/*", "scripts/build-wasm.sh"],
        ["docs/en/web/README.md", "distribution/README.md", "docs/plans/delivery/WEB_UI_PLAN.md", "ROADMAP.md", "TODO.md"],
    ),
    (
        "Swift / Kotlin wrappers",
        ["ffi/*", "swift/*", "kotlin/*", "scripts/build-ios.sh", "scripts/build-android.sh"],
        ["docs/en/ios/README.md", "docs/en/android/README.md", "distribution/README.md", "ROADMAP.md", "TODO.md"],
    ),
    (
        "notebooks",
        ["notebooks/*"],
        ["distribution/NOTEBOOKS.md", "ROADMAP.md", "TODO.md", "README.md"],
    ),
    (
        "release and publishing",
        [".github/workflows/*", "deploy/*", "Cargo.toml"],
        ["deploy/README.md", "PUBLISHING.md", "README.md", "python/README.md", "engine/README.md", "steam/README.md"],
    ),
    (
        "plans and audits",
        ["docs/plans/*"],
        ["docs/plans/README.md", "ROADMAP.md", "TODO.md"],
    ),
    (
        "hooks and Claude Code configuration",
        ["hooks/*", ".claude/settings.json"],
        ["CLAUDE.md"],
    ),
]

# A line added to a Markdown file is listed for review if it looks like a
# claim about the state of the code: a ticked box, a status word, a count of
# tests or notebooks, a version, or a measurement.
CLAIM = re.compile(
    r"\[x\]|\b(done|shipped|complete[d]?|implemented|released|fixed|built|faster|slower)\b"
    r"|\b\d[\d ,.]*\s*(Rust|Python)?\s*(tests?|notebooks?|compounds?)\b"
    r"|\bv\d+\.\d+\.\d+\b|\d\s*(µs|ns|ms|%|×)",
    re.IGNORECASE,
)

MAX_CLAIM_LINES = 150
MAX_LINE_CHARS = 400


def git(root: pathlib.Path, *args: str) -> str:
    """Run a git command in ``root`` and return its stdout ('' on failure)."""
    r = subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ""


def matches(path: str, patterns: list[str]) -> bool:
    # fnmatch's `*` crosses `/`, so "engine/src/*" covers the whole subtree.
    return any(fnmatch.fnmatch(path, p) for p in patterns)


def build(root: pathlib.Path, command: str) -> str:
    head = git(root, "rev-parse", "HEAD").strip()
    branch = git(root, "rev-parse", "--abbrev-ref", "HEAD").strip()
    # What the push adds relative to main. On main itself (an admin hot-fix)
    # compare with what the remote already has.
    base = git(root, "merge-base", "HEAD", "origin/main").strip()
    if not base or base == head:
        base = git(root, "rev-parse", "@{u}").strip() or base

    out: list[str] = []
    out.append("# Push brief")
    out.append("")
    out.append(f"command: {command}")
    out.append(f"written: {datetime.datetime.now().isoformat(timespec='seconds')}")
    out.append(f"branch: {branch}")
    out.append(f"head: {head}")
    out.append(f"base: {base or '(none found)'}")
    out.append("")

    if not base or base == head:
        out.append("NOTHING TO REVIEW: this push adds no commits on top of its base.")
        return "\n".join(out) + "\n"

    rng = f"{base}..{head}"
    commits = git(root, "log", "--format=%h %s", rng).strip().splitlines()
    changed = [p for p in git(root, "diff", "--name-only", rng).splitlines() if p]

    out.append(f"## Commits ({len(commits)})")
    out.extend(f"- {c}" for c in commits[:40])
    out.append("")
    out.append(f"## Files changed ({len(changed)})")
    out.extend(f"- {p}" for p in changed[:200])
    out.append("")

    out.append("## Documents that describe the changed areas")
    out.append("")
    out.append("`NOT touched` is not an error by itself: check whether the document is")
    out.append("still true after this change.")
    out.append("")
    any_area = False
    for area, patterns, docs in DOC_MAP:
        hits = [p for p in changed if matches(p, patterns)]
        if not hits:
            continue
        any_area = True
        out.append(f"### {area} — {len(hits)} file(s) changed")
        for d in docs:
            if not (root / d).exists():
                state = "MISSING from the repo"
            elif d in changed:
                state = "touched"
            else:
                state = "NOT touched"
            out.append(f"- {d}: {state}")
        out.append("")
    if not any_area:
        out.append("(no mapped area changed — documentation-only or unmapped files)")
        out.append("")

    out.append("## Claim-like lines this push adds to Markdown files")
    out.append("")
    out.append("Each one asserts something about the code. Check it against the code or")
    out.append("against the same number elsewhere in the repo.")
    out.append("")
    shown = 0
    for path in changed:
        if not path.endswith(".md"):
            continue
        added = [
            line[1:].strip()
            for line in git(root, "diff", "--unified=0", rng, "--", path).splitlines()
            if line.startswith("+") and not line.startswith("+++")
        ]
        claims = [a for a in added if CLAIM.search(a)]
        if not claims:
            continue
        out.append(f"### {path}")
        for c in claims:
            if shown >= MAX_CLAIM_LINES:
                break
            out.append(f"- {c[:MAX_LINE_CHARS]}")
            shown += 1
        out.append("")
        if shown >= MAX_CLAIM_LINES:
            out.append(f"(truncated at {MAX_CLAIM_LINES} lines — read the documents for the rest)")
            break
    if shown == 0:
        out.append("(none)")
    return "\n".join(out) + "\n"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 1
    root = pathlib.Path(sys.argv[1]).resolve()
    command = sys.argv[2] if len(sys.argv) > 2 else ""
    brief = root / ".claude" / "push-brief.md"
    brief.parent.mkdir(exist_ok=True)
    brief.write_text(build(root, command), encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
