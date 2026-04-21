#!/usr/bin/env bash
# deploy.sh — pull latest main, rebuild images, restart the stack.
#
# Reads configuration from deploy/.env (copy deploy/.env.example first).
# Safe to rerun; idempotent.
#
# Usage:
#   deploy.sh              Fast deploy, uses Docker layer cache.
#   deploy.sh --no-cache   Full rebuild from scratch (slower, always correct).
#                          Use this after big changes or if the last deploy
#                          seems to be running stale code.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
ENV_FILE="${REPO_ROOT}/deploy/.env"
COMPOSE_DIR="${REPO_ROOT}/deploy/compose"

# -------- Argument parsing --------
BUILD_FLAGS=""
if [[ "${1:-}" == "--no-cache" ]]; then
  BUILD_FLAGS="--no-cache"
  echo "==> Full rebuild requested (--no-cache) — this will take longer"
fi

# -------- Preflight --------
if [[ ! -f "${ENV_FILE}" ]]; then
  echo "ERROR: ${ENV_FILE} not found." >&2
  echo "       Copy deploy/.env.example to deploy/.env and fill in values." >&2
  exit 1
fi

cd "${REPO_ROOT}"

# -------- Source update --------
echo "==> Updating source (git pull --ff-only origin main)"
git pull --ff-only origin main

echo "==> Current HEAD: $(git rev-parse --short HEAD) — $(git log -1 --pretty=%s)"

# -------- Regenerate landing page --------
echo "==> Regenerating notebooks/index.ipynb landing page"
if command -v python3 >/dev/null 2>&1; then
  INDEX_PY="python3"
else
  INDEX_PY="python"
fi
"${INDEX_PY}" scripts/build_index.py

# -------- Build images --------
echo "==> Building notebook image (profile=build-only)${BUILD_FLAGS:+ $BUILD_FLAGS}"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" --profile build-only build ${BUILD_FLAGS} )

echo "==> Building hub image${BUILD_FLAGS:+ $BUILD_FLAGS}"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" build ${BUILD_FLAGS} )

# -------- Restart stack --------
echo "==> Starting / updating stack"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" up -d )

# Give the hub a moment to start before we query it
sleep 3

echo "==> Current state:"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" ps )

# -------- Verification --------
echo "==> Verifying deployment"

WARNINGS=0

# Check 1: hub container is running
if ! docker ps --format '{{.Names}}' | grep -q '^vle-jupyterhub$'; then
  echo "  ✗ vle-jupyterhub container is not running"
  WARNINGS=$((WARNINGS + 1))
else
  echo "  ✓ vle-jupyterhub is running"
fi

# Check 2: hub config has the latest fixes (sentinel lines that must be present)
if docker exec vle-jupyterhub grep -q "_parse_mem_limit" /srv/jupyterhub/jupyterhub_config.py 2>/dev/null; then
  echo "  ✓ jupyterhub_config.py has _parse_mem_limit (mem_limit string parser)"
else
  echo "  ✗ jupyterhub_config.py is stale (missing _parse_mem_limit) — rerun with --no-cache"
  WARNINGS=$((WARNINGS + 1))
fi

# Check 3: notebook image has the subshell fix in seed-user-home.sh.
# Extract the file via `docker create` + `docker cp` so we don't spin up a
# container just to cat one file. The sentinel 'SUBSHELL-WRAPPER-v1' is
# placed inside the seed script itself and survives reformatting, unlike a
# regex on the subshell's opening paren.
SEED_SCRIPT=""
SEED_CID="$(docker create vle-notebook:latest 2>/dev/null || true)"
if [[ -n "${SEED_CID}" ]]; then
  SEED_SCRIPT="$(docker cp "${SEED_CID}:/usr/local/bin/before-notebook.d/10-seed-user-home.sh" - 2>/dev/null \
    | tar -xO 2>/dev/null || true)"
  docker rm "${SEED_CID}" >/dev/null 2>&1 || true
fi

if grep -q 'SUBSHELL-WRAPPER-v1' <<<"${SEED_SCRIPT}"; then
  echo "  ✓ seed-user-home.sh has the subshell wrapper (start.sh safe)"
else
  echo "  ✗ seed-user-home.sh is stale (missing SUBSHELL-WRAPPER-v1 sentinel) — rerun with --no-cache"
  WARNINGS=$((WARNINGS + 1))
fi

# Check 4: notebook image exists and is tagged
if docker image inspect vle-notebook:latest >/dev/null 2>&1; then
  echo "  ✓ vle-notebook:latest image exists"
else
  echo "  ✗ vle-notebook:latest image missing"
  WARNINGS=$((WARNINGS + 1))
fi

echo
if [[ "${WARNINGS}" -eq 0 ]]; then
  echo "==> Done. All checks passed."
else
  echo "==> Done with ${WARNINGS} warning(s). See above."
  echo "    If 'stale' warnings appeared, rerun: deploy/scripts/deploy.sh --no-cache"
  exit 1
fi
