#!/usr/bin/env bash
# publish-pypi.sh — publish the Python wheel + sdist to PyPI.
#
# Builds a wheel for the current platform and an sdist, then uploads both via
# maturin. Runs `maturin build` by default; pass `--go` to actually upload
# with `maturin publish`.
#
# Prereqs:
#   - Python 3.10+ and maturin installed: `pip install "maturin>=1.5,<2"`
#   - Rust toolchain (stable) installed: https://rustup.rs
#   - API token: export MATURIN_PYPI_TOKEN=pypi-...  (get from
#     https://pypi.org/manage/account/token/)
#   - Clean working tree, version bumped in workspace Cargo.toml and
#     python/pyproject.toml (they must match — see PUBLISHING.md).
#
# Note: this only builds the wheel for the host platform. Multi-platform
# wheel releases (linux x86_64/aarch64, macOS x86_64/arm64, windows x64)
# require a CI matrix — see PUBLISHING.md "Future: CI-driven releases".

set -euo pipefail

cd "$(dirname "$0")/../../python"

GO=0
for arg in "$@"; do
    case "$arg" in
        --go) GO=1 ;;
        -h|--help)
            sed -n '2,20p' "$0"
            exit 0
            ;;
        *)
            echo "unknown arg: $arg" >&2
            exit 2
            ;;
    esac
done

echo "== Building wheel + sdist =="
# Pin the Python interpreter explicitly so maturin uses the active env
# (typically conda or venv) instead of auto-discovering a stray system
# python that PyO3 may not yet support — for example homebrew's python@3.14
# on macOS, which pyo3 0.22 cannot build against. set -e above will abort
# if no python is on PATH, with a clearer error than maturin's downstream
# failure.
PYTHON_INTERP="$(command -v python)"
maturin build --release --sdist --out target/wheels --interpreter "$PYTHON_INTERP"

echo ""
echo "Built:"
ls -1 target/wheels/

if [[ $GO -eq 0 ]]; then
    echo ""
    echo "== DRY RUN (pass --go to publish for real) =="
    echo "Would upload the files above to PyPI as vle-thermo."
    exit 0
fi

if [[ -z "${MATURIN_PYPI_TOKEN:-}" ]]; then
    echo "ERROR: set MATURIN_PYPI_TOKEN=pypi-... before running with --go" >&2
    exit 1
fi

echo ""
echo "== Uploading to PyPI =="
maturin upload --skip-existing target/wheels/*

# POSIX character class [[:space:]] in place of \s — BSD sed on macOS
# doesn't recognize \s, which silently leaves the line unchanged.
VERSION="$(grep -E '^version[[:space:]]*=' pyproject.toml | head -1 \
    | sed -E 's/version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')"
echo ""
echo "Published v${VERSION}. Verify at:"
echo "  https://pypi.org/project/vle-thermo/${VERSION}/"
