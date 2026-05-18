#!/usr/bin/env bash
# publish-crate.sh — publish the Rust crates to crates.io.
#
# Publishes vle-units first (vle-thermo depends on it) and then vle-thermo.
# Runs `cargo publish --dry-run` by default; pass `--go` to actually upload.
#
# Prereqs:
#   - Logged in: `cargo login <TOKEN>` (get from https://crates.io/settings/tokens)
#   - Clean working tree (no uncommitted changes to published files)
#   - Version bumped in the workspace root Cargo.toml (see PUBLISHING.md)

set -euo pipefail

cd "$(dirname "$0")/../.."

GO=0
for arg in "$@"; do
    case "$arg" in
        --go) GO=1 ;;
        -h|--help)
            sed -n '2,11p' "$0"
            exit 0
            ;;
        *)
            echo "unknown arg: $arg" >&2
            exit 2
            ;;
    esac
done

FLAGS=()
if [[ $GO -eq 0 ]]; then
    FLAGS+=(--dry-run)
    echo "== DRY RUN (pass --go to publish for real) =="
else
    echo "== PUBLISHING to crates.io =="
fi

# vle-units has no workspace siblings as runtime dependencies, so it goes first.
# Once it is on crates.io the vle-thermo upload can resolve it if we later add
# a dependency edge between them.
echo "--- vle-units ---"
cargo publish -p vle-units "${FLAGS[@]}"

echo "--- vle-thermo ---"
cargo publish -p vle-thermo "${FLAGS[@]}"

if [[ $GO -eq 1 ]]; then
    VERSION="$(grep -E '^version\s*=' Cargo.toml | head -1 | sed -E 's/version\s*=\s*"([^"]+)".*/\1/')"
    echo ""
    echo "Published v${VERSION}. Verify at:"
    echo "  https://crates.io/crates/vle-units/${VERSION}"
    echo "  https://crates.io/crates/vle-thermo/${VERSION}"
fi
