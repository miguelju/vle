#!/usr/bin/env bash
# deploy-notebooks.sh — fast-path notebook refresh, no image rebuild.
#
# Invoked from CI (or by hand) when only the notebooks/ directory needs to
# reach the hub — e.g., on every `v*` tag where the engine hasn't changed.
# The full image-rebuild path lives in `deploy.sh`; this script does a
# strict subset of its work:
#
#   1. Regenerate notebooks/index.ipynb via scripts/build_index.py.
#   2. Stamp notebooks/.notebook-version with the current git
#      tag (or short SHA fallback) — seed-user-home.sh re-seeds the
#      per-user notebooks volume when this value changes.
#   3. (No docker build, no docker compose up, no container restart.)
#
# Per-user containers spawned from now on bind-mount this notebooks
# directory read-only at /opt/vle/notebooks (see jupyterhub_config.py +
# the NOTEBOOK_HOST_PATH env var), so new spawns see the updated content
# immediately. Existing running per-user containers keep whatever they
# bind-mounted at spawn time — they pick up the change on their next
# spawn (which the idle-culler triggers within IDLE_TIMEOUT_MINUTES).
#
# Safe to rerun; idempotent. Usage:
#
#   deploy/scripts/deploy-notebooks.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

echo "==> Current HEAD: $(git rev-parse --short HEAD) — $(git log -1 --pretty=%s)"

# -------- Regenerate landing page -------------------------------------------
# notebooks/index.ipynb is committed but auto-generated; keep it in sync with
# whatever set of notebooks ships in this checkout.
echo "==> Regenerating notebooks/index.ipynb landing page"
if command -v python3 >/dev/null 2>&1; then
  INDEX_PY="python3"
else
  INDEX_PY="python"
fi
if ! "${INDEX_PY}" -c 'import nbformat' >/dev/null 2>&1; then
  echo "  ! nbformat not installed for ${INDEX_PY} — using committed copy."
  echo "    (install with 'sudo apt-get install -y python3-nbformat' to enable refresh)"
elif ! "${INDEX_PY}" scripts/build_index.py; then
  echo "  ! build_index.py failed — using committed copy."
fi

# -------- Stamp the version marker ------------------------------------------
# seed-user-home.sh reads /opt/vle/notebooks/.notebook-version on each spawn
# and re-seeds the user's ~/work/notebooks/ if the value here differs from
# what it stamped on last seed. The marker name is intentionally hidden by
# leading-dot so it doesn't clutter the user's JupyterLab file browser.
VERSION="$(git describe --tags --always --abbrev=8 2>/dev/null \
           || git rev-parse --short HEAD)"
echo "${VERSION}" > notebooks/.notebook-version
echo "==> Stamped notebooks/.notebook-version = ${VERSION}"

# -------- Verification ------------------------------------------------------
echo "==> Verifying"
WARNINGS=0
if [[ ! -f notebooks/index.ipynb ]]; then
  echo "  ✗ notebooks/index.ipynb missing"
  WARNINGS=$((WARNINGS + 1))
else
  echo "  ✓ notebooks/index.ipynb present"
fi
if [[ ! -f notebooks/.notebook-version ]]; then
  echo "  ✗ notebooks/.notebook-version missing"
  WARNINGS=$((WARNINGS + 1))
else
  echo "  ✓ notebooks/.notebook-version = $(cat notebooks/.notebook-version)"
fi

# Sanity: confirm the bind-mount path on the running hub matches our
# refresh target (if NOTEBOOK_HOST_PATH is configured). Otherwise the
# rsynced changes would never reach per-user containers.
if [[ -f "${REPO_ROOT}/deploy/.env" ]]; then
  CONFIGURED="$(grep -E '^NOTEBOOK_HOST_PATH=' "${REPO_ROOT}/deploy/.env" \
                | sed 's/^NOTEBOOK_HOST_PATH=//' | tr -d '"' || true)"
  ABS_REPO_NB="${REPO_ROOT}/notebooks"
  if [[ -z "${CONFIGURED}" ]]; then
    echo "  ! NOTEBOOK_HOST_PATH is empty in deploy/.env — bind-mount disabled."
    echo "    Per-user containers will still serve the image-bundled notebooks"
    echo "    until the next full deploy (deploy/scripts/deploy.sh)."
    WARNINGS=$((WARNINGS + 1))
  elif [[ "${CONFIGURED}" != "${ABS_REPO_NB}" ]]; then
    echo "  ! NOTEBOOK_HOST_PATH=${CONFIGURED} but this checkout is at ${ABS_REPO_NB}."
    echo "    The hub will serve notebooks from ${CONFIGURED}, not the freshly"
    echo "    stamped ones here. Either move the checkout or update the env."
    WARNINGS=$((WARNINGS + 1))
  else
    echo "  ✓ NOTEBOOK_HOST_PATH matches this checkout"
  fi
fi

echo
if [[ "${WARNINGS}" -eq 0 ]]; then
  echo "==> Done. New per-user spawns will pick up the refreshed notebooks."
  echo "    Existing user volumes re-seed on their next login."
else
  echo "==> Done with ${WARNINGS} warning(s). See above."
  exit 1
fi
