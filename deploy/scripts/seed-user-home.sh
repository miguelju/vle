#!/usr/bin/env bash
# seed-user-home.sh — copy bundled VLE notebooks + DB into ~/work on first start.
#
# DockerSpawner mounts a persistent named volume at ~/work. The very first time
# a user logs in, that volume is empty. This hook populates it with the copies
# baked into /opt/vle at image build time. On every subsequent start it is a
# no-op, so users can edit their own copies freely.

set -euo pipefail

WORK="${HOME}/work"
mkdir -p "${WORK}"

# Notebooks.
if [[ ! -d "${WORK}/notebooks" ]] && [[ -d /opt/vle/notebooks ]]; then
  cp -r /opt/vle/notebooks "${WORK}/notebooks"
fi

# Component database.
if [[ ! -f "${WORK}/data/components.db" ]] && [[ -f /opt/vle/data/components.db ]]; then
  mkdir -p "${WORK}/data"
  cp /opt/vle/data/components.db "${WORK}/data/components.db"
fi
