#!/usr/bin/env bash
#
# Test harness for hooks/claude-push-gate.
#
# A gate that has never been run against something it must reject is not
# known to work (see CLAUDE.md, *Pre-commit doc-sync gate*). This feeds the
# gate the JSON Claude Code would send for a set of commands, against the
# real tree and against a deliberately stale copy of it, and checks the exit
# code of each: 0 = allowed, 2 = blocked.
#
# Run from anywhere:  hooks/test-claude-push-gate.sh

set -uo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
gate="$root/hooks/claude-push-gate"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# A stale copy: the plan documents and the gate, with one milestone header
# removed from TODO.md so ROADMAP.md and TODO.md no longer correspond.
stale="$tmp/stale"
mkdir -p "$stale/docs/plans" "$stale/hooks" "$stale/notebooks"
cp "$root/ROADMAP.md" "$root/TODO.md" "$root/README.md" "$stale/"
cp "$root/docs/plans/MODERNIZATION_PLAN.md" "$stale/docs/plans/"
cp "$root/hooks/pre-commit" "$stale/hooks/"
for nb in "$root"/notebooks/*.ipynb; do : > "$stale/notebooks/$(basename "$nb")"; done
mkdir -p "$stale/python" "$stale/distribution"
cp "$root/python/README.md" "$stale/python/"
cp "$root/distribution/NOTEBOOKS.md" "$stale/distribution/"
git -C "$stale" init -q
# An in-sync copy must pass before it is broken — otherwise a "blocked"
# result below could come from the copy being incomplete, not from the edit.
if ! (cd "$stale" && hooks/pre-commit >/dev/null 2>&1); then
    echo "SETUP FAILED: the unbroken copy does not pass hooks/pre-commit" >&2
    (cd "$stale" && hooks/pre-commit) >&2
    exit 1
fi
first_header="$(grep -n -m1 -E '^## .*Milestone [0-9]' "$stale/TODO.md" | cut -d: -f1)"
sed -i.bak "${first_header}d" "$stale/TODO.md"

# A repository that is not vle.
other="$tmp/other"
mkdir -p "$other" && git -C "$other" init -q

pass=0; fail=0
check() {  # check <expected exit> <label> <cwd> <command>
    local want="$1" label="$2" cwd="$3" cmd="$4" got
    printf '{"tool_name":"Bash","cwd":"%s","tool_input":{"command":"%s"}}' "$cwd" "$cmd" \
        | "$gate" >/dev/null 2>&1
    got=$?
    if [ "$got" = "$want" ]; then pass=$((pass + 1)); else
        fail=$((fail + 1)); echo "FAIL: $label — exit $got, expected $want"; fi
}

check 0 "not a push"                          "$root"  "git status"
check 0 "push word inside another command"    "$root"  "echo git push"
check 0 "push from the in-sync tree"          "$root"  "git push"
check 0 "push -u from the in-sync tree"       "$root"  "git push -u origin some-branch"
check 0 "push after other commands"           "$root"  "cargo fmt --check && git push 2>&1 | tail -3"
check 0 "push to another repo via cd"         "$root"  "cd $other && git push"
check 0 "push to another repo via git -C"     "$root"  "git -C $other push"
check 0 "push from a non-vle session"         "$other" "git push"
check 2 "push from a stale tree"              "$stale" "git push"
check 2 "push to a stale tree via cd"         "$root"  "cd $stale && git push origin main"
check 2 "push to a stale tree via git -C"     "$other" "git -C $stale push"
check 2 "stale tree, push after other steps"  "$stale" "git status; git push"
# Malformed input must not block: the gate cannot judge what it cannot parse.
printf 'not json' | "$gate" >/dev/null 2>&1
[ $? = 0 ] && pass=$((pass + 1)) || { fail=$((fail + 1)); echo "FAIL: malformed input blocked"; }

# The gate leaves a brief for the agent hook, stamped with the command.
rm -f "$root/.claude/push-brief.md"
printf '{"tool_name":"Bash","cwd":"%s","tool_input":{"command":"git push origin probe-branch"}}' "$root" \
    | "$gate" >/dev/null 2>&1
if grep -q '^command: git push origin probe-branch$' "$root/.claude/push-brief.md" 2>/dev/null \
    && grep -q '^head: [0-9a-f]\{40\}$' "$root/.claude/push-brief.md"; then pass=$((pass + 1)); else
    fail=$((fail + 1)); echo "FAIL: no usable push brief written"; fi

# Every document the brief can point the agent at must exist, and every
# document on CLAUDE.md's release list must be in the map — a renamed or
# newly listed document would otherwise drop out of review silently.
PY=""
for cand in "$HOME/miniconda3/envs/vle/bin/python" python3 python; do
    if command -v "$cand" >/dev/null 2>&1 || [ -x "$cand" ]; then PY="$cand"; break; fi
done
if "$PY" - "$root" <<'PYEOF'
import importlib.util, pathlib, re, sys
root = pathlib.Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("push_brief", root / "hooks" / "push-brief.py")
mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
mapped = {d for _, _, docs in mod.DOC_MAP for d in docs}
bad = sorted(d for d in mapped if not (root / d).exists())
# The numbered release list in CLAUDE.md: every backticked *.md path on it.
claude = (root / "CLAUDE.md").read_text(encoding="utf-8")
block = claude.split("## Release & Push Rules", 1)[1].split("**Package-page docs", 1)[0]
listed = set()
for name in re.findall(r"`([\w./-]+\.md)`", block):
    hits = [name] if (root / name).exists() else [str(p.relative_to(root)) for p in root.glob(f"docs/**/{name}")]
    listed.update(hits)
# CLAUDE.md is where the list lives; the legacy comparison and the research
# paper are not tied to a code area, so the release audit covers them.
exempt = {"PASCAL_VB6_COMPARISON.md"}
unmapped = sorted(d for d in listed - mapped - exempt if not d.startswith("docs/plans/engine/"))
for d in bad: print(f"FAIL: push-brief.py maps a missing document: {d}")
for d in unmapped: print(f"FAIL: on CLAUDE.md's release list but not in DOC_MAP: {d}")
sys.exit(1 if bad or unmapped else 0)
PYEOF
then pass=$((pass + 1)); else fail=$((fail + 1)); fi

if [ "$fail" = 0 ]; then echo "PASS: all $pass cases behaved as specified"; else
    echo "$fail of $((pass + fail)) cases failed"; exit 1; fi
