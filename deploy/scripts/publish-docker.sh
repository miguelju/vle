#!/usr/bin/env bash
# publish-docker.sh — build and push the standalone VLE image to GHCR.
#
# Produces a multi-arch (linux/amd64 + linux/arm64) image and pushes it as
# ghcr.io/miguelju/vle-thermo:<version> and :latest.
#
# Runs `docker buildx build --load` locally by default; pass `--go` to build
# for both platforms and push to GHCR.
#
# Prereqs:
#   - Docker 24+ with buildx enabled (`docker buildx version`)
#   - A buildx builder that supports multi-arch:
#       docker buildx create --name vle-builder --use
#       docker buildx inspect --bootstrap
#   - Logged in to GHCR with a PAT that has write:packages scope:
#       echo "$GITHUB_TOKEN" | docker login ghcr.io -u miguelju --password-stdin
#     (create the token at https://github.com/settings/tokens — classic,
#     with write:packages + read:packages.)

set -euo pipefail

cd "$(dirname "$0")/../.."

GO=0
for arg in "$@"; do
    case "$arg" in
        --go) GO=1 ;;
        -h|--help)
            sed -n '2,21p' "$0"
            exit 0
            ;;
        *)
            echo "unknown arg: $arg" >&2
            exit 2
            ;;
    esac
done

IMAGE="ghcr.io/miguelju/vle-thermo"
VERSION="$(grep -E '^version\s*=' Cargo.toml | head -1 | sed -E 's/version\s*=\s*"([^"]+)".*/\1/')"

if [[ -z "$VERSION" ]]; then
    echo "ERROR: could not read version from workspace Cargo.toml" >&2
    exit 1
fi

echo "== Image: ${IMAGE}:${VERSION} (+ :latest) =="

if [[ $GO -eq 0 ]]; then
    echo "== DRY RUN (pass --go to push multi-arch to GHCR) =="
    echo "Building linux/$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/') locally for smoke test..."
    docker buildx build \
        --file deploy/docker/Dockerfile.standalone \
        --tag "${IMAGE}:${VERSION}" \
        --tag "${IMAGE}:latest" \
        --load \
        .
    echo ""
    echo "Smoke-test locally with:"
    echo "  docker run --rm -p 8888:8888 ${IMAGE}:${VERSION}"
    exit 0
fi

echo "== Building + pushing multi-arch (amd64 + arm64) =="
docker buildx build \
    --file deploy/docker/Dockerfile.standalone \
    --platform linux/amd64,linux/arm64 \
    --tag "${IMAGE}:${VERSION}" \
    --tag "${IMAGE}:latest" \
    --push \
    .

echo ""
echo "Pushed. Verify at:"
echo "  https://github.com/miguelju/vle/pkgs/container/vle-thermo"
echo ""
echo "Try it:"
echo "  docker run --rm -p 8888:8888 ${IMAGE}:${VERSION}"
