#!/usr/bin/env bash
# publish-crate.sh — publish the Rust crates to crates.io.
#
# Publishes the dependency-free siblings first (vle-units, vle-steam), then
# vle-thermo (which depends on both — vle-units directly, vle-steam via the
# `steam` feature the wheel turns on). Runs `cargo publish --dry-run` by
# default; pass `--go` to actually upload.
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

# vle-steam is also dependency-free (pure f64), so it can go before vle-thermo,
# which references it via the optional `steam` feature. crates.io requires even
# optional dependencies to be published, so this must precede vle-thermo.
echo "--- vle-steam ---"
cargo publish -p vle-steam "${FLAGS[@]}"

echo "--- vle-thermo ---"
cargo publish -p vle-thermo "${FLAGS[@]}"

if [[ $GO -eq 1 ]]; then
    # POSIX character class [[:space:]] in place of \s — BSD sed on macOS
    # doesn't recognize \s, which silently leaves the line unchanged and
    # produces a literal "version = X.Y.Z" tag instead of the bare version.
    VERSION="$(grep -E '^version[[:space:]]*=' Cargo.toml | head -1 \
        | sed -E 's/version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')"
    echo ""
    echo "Published v${VERSION}. Verify at:"
    echo "  https://crates.io/crates/vle-units/${VERSION}"
    echo "  https://crates.io/crates/vle-steam/${VERSION}"
    echo "  https://crates.io/crates/vle-thermo/${VERSION}"
fi
