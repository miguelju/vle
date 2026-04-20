#!/usr/bin/env bash
# deploy.sh — pull latest main, rebuild images, restart the stack.
#
# Reads configuration from deploy/.env (copy deploy/.env.example first).
# Safe to rerun; idempotent.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
ENV_FILE="${REPO_ROOT}/deploy/.env"
COMPOSE_DIR="${REPO_ROOT}/deploy/compose"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "ERROR: ${ENV_FILE} not found." >&2
  echo "       Copy deploy/.env.example to deploy/.env and fill in values." >&2
  exit 1
fi

cd "${REPO_ROOT}"

echo "==> Updating source (git pull --ff-only origin main)"
git pull --ff-only origin main

echo "==> Regenerating notebooks/index.ipynb landing page"
# Only needs nbformat (pure-Python) — no full vle install required on the host.
if command -v python3 >/dev/null 2>&1; then
  INDEX_PY="python3"
else
  INDEX_PY="python"
fi
"${INDEX_PY}" scripts/build_index.py

echo "==> Building notebook image (profile=build-only)"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" --profile build-only build )

echo "==> Building hub image"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" build )

echo "==> Starting / updating stack"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" up -d )

echo "==> Current state:"
( cd "${COMPOSE_DIR}" \
  && docker compose --env-file "${ENV_FILE}" ps )

echo "==> Done."
