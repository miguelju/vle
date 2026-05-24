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

# -------- Compose profile selection --------
# The `cloudflared` service is gated behind the `tunnel` compose profile.
# Activate it only when CLOUDFLARED_TUNNEL_TOKEN is non-empty in deploy/.env,
# so hosts running behind another reverse-proxy / tunnel setup don't pull
# in (and crash-loop) an unconfigured cloudflared container.
COMPOSE_PROFILES=()
if grep -E '^CLOUDFLARED_TUNNEL_TOKEN=.+' "${ENV_FILE}" >/dev/null 2>&1; then
  COMPOSE_PROFILES+=(--profile tunnel)
  echo "==> CLOUDFLARED_TUNNEL_TOKEN is set — including the tunnel profile"
else
  echo "==> CLOUDFLARED_TUNNEL_TOKEN is empty — skipping cloudflared service"
fi

# -------- Source state --------
# This script no longer runs `git pull` itself. The expected callers are
# (a) the auto-deploy wrapper /usr/local/bin/vle-deploy, which checks out a
# specific tag before invoking us, or (b) a human operator who has already
# checked out the desired revision. Running git pull here would race with
# the wrapper's tag-checkout.
echo "==> Current HEAD: $(git rev-parse --short HEAD) — $(git log -1 --pretty=%s)"

# -------- Regenerate landing page --------
# notebooks/index.ipynb is committed, so this step is a "freshen on deploy"
# nicety — if the host lacks nbformat (e.g. fresh Ubuntu without
# python3-nbformat), warn and fall back to the committed copy.
echo "==> Regenerating notebooks/index.ipynb landing page"
if command -v python3 >/dev/null 2>&1; then
  INDEX_PY="python3"
else
  INDEX_PY="python"
fi
if ! "${INDEX_PY}" -c 'import nbformat' >/dev/null 2>&1; then
  echo "  ! nbformat not installed for ${INDEX_PY} — skipping regeneration"
  echo "    (using committed notebooks/index.ipynb; install with"
  echo "     'sudo apt-get install -y python3-nbformat' to enable refresh)"
elif ! "${INDEX_PY}" scripts/build_index.py; then
  echo "  ! build_index.py failed — falling back to committed index.ipynb"
fi

# -------- Stamp notebook version marker --------
# The per-user seed-user-home.sh (NOTEBOOK-VERSION-RESEED-v1 sentinel) compares
# /opt/vle/notebooks/.notebook-version inside the spawned container against
# ~/work/notebooks/.notebook-version in the user volume, and re-seeds every
# managed .ipynb on mismatch. The notebooks-only fast path
# (deploy-notebooks.sh) stamps this file too; we mirror that here so a full
# `deploy.sh` rebuild leaves no gap. Without this step the rebuilt image ships
# no version marker, the `if [[ -f ${BUNDLED_VER_FILE} ]]` guard in the seed
# script falls through, and existing users never receive new bundled notebooks.
# (Hit on the v0.3.1 deploy on 2026-05-24 — see deploy/local/deploy-notes/
# milestone-07.md "Outcome" section.)
VLE_TAG="$(git describe --tags --exact-match 2>/dev/null \
  || git rev-parse --short HEAD)"
echo "==> Stamping notebooks/.notebook-version = ${VLE_TAG}"
printf '%s\n' "${VLE_TAG}" > notebooks/.notebook-version

# -------- Build images --------
echo "==> Building notebook image (profile=build-only)${BUILD_FLAGS:+ $BUILD_FLAGS}"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" --profile build-only build ${BUILD_FLAGS} )

echo "==> Building hub image${BUILD_FLAGS:+ $BUILD_FLAGS}"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" "${COMPOSE_PROFILES[@]}" build ${BUILD_FLAGS} )

# -------- Force stale per-user containers to be recreated --------
# DockerSpawner reuses an existing per-user container across spawns by default
# (`remove=False`). After we rebuild `vle-notebook:latest`, those existing
# containers are still pinned to the *old* image — even a "Stop My Server" +
# "Start My Server" only restarts them, never recreates. So the new image
# (with updated seed script, .notebook-version marker, and notebook content)
# never gets used until each container is explicitly removed.
#
# We remove every container whose image is `vle-notebook:latest`. The user
# volumes (`vle-user-<email>`) are *not* touched — they live independently
# of the container lifecycle, so user-created notebooks survive. The next
# spawn for each affected user creates a fresh container from the new image
# and the seed-user-home.sh's version-marker re-seed picks up any new
# bundled notebooks.
echo "==> Force-removing stale per-user containers on vle-notebook:latest"
STALE_CONTAINERS="$(docker ps -aq --filter ancestor=vle-notebook:latest)"
if [[ -n "${STALE_CONTAINERS}" ]]; then
  # shellcheck disable=SC2086
  docker rm -f ${STALE_CONTAINERS} >/dev/null
  echo "  removed $(echo "${STALE_CONTAINERS}" | wc -w | tr -d ' ') container(s)"
else
  echo "  none found (fresh install or no users have spawned yet)"
fi

# -------- Restart stack --------
echo "==> Starting / updating stack"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" "${COMPOSE_PROFILES[@]}" up -d )

# Give the hub a moment to start before we query it
sleep 3

echo "==> Current state:"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" "${COMPOSE_PROFILES[@]}" ps )

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
#
# Read the real file path (not /usr/local/bin/before-notebook.d/10-…, which
# is a symlink — docker cp would tar the symlink itself, leaving the grep
# with empty input and a false-positive "stale" warning).
SEED_SCRIPT=""
SEED_CID="$(docker create vle-notebook:latest 2>/dev/null || true)"
if [[ -n "${SEED_CID}" ]]; then
  SEED_SCRIPT="$(docker cp "${SEED_CID}:/usr/local/bin/seed-user-home.sh" - 2>/dev/null \
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
