#!/usr/bin/env bash
# promote-standby.sh — bring up the warm standby host's cloudflared connector.
#
# Use when the primary host has gone down and the Cloudflare-fronted production
# URL is unreachable. The standby must already have:
#   - The VLE compose stack deployed under SSH_REMOTE_PATH (default /home/ubuntu/vle)
#   - CLOUDFLARED_TUNNEL_TOKEN populated in deploy/.env (same token as the primary)
#   - The cloudflared container created at least once (i.e. a prior
#     `docker compose --profile tunnel up -d cloudflared` followed by `stop`)
#
# Cold-standby steady state is "primary cloudflared running, standby
# cloudflared stopped". This script flips standby ON; restore steady state
# afterwards by starting the primary again and stopping the standby (see
# deploy/local/runbook-failover.md for the operator-specific commands).
#
# Usage:
#   promote-standby.sh <ssh-target>
#   STANDBY_HOST=<ssh-target> promote-standby.sh
#   SSH_REMOTE_PATH=/srv/vle promote-standby.sh <ssh-target>   # custom layout

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: promote-standby.sh [<ssh-target>]

Arguments:
  <ssh-target>    SSH alias or user@host for the standby (positional wins
                  over STANDBY_HOST env var).

Environment:
  STANDBY_HOST     Default SSH target if no positional arg is given.
  SSH_REMOTE_PATH  Remote path to the cloned VLE repo on the standby
                   (default: /home/ubuntu/vle).
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

TARGET="${1:-${STANDBY_HOST:-}}"
REMOTE_PATH="${SSH_REMOTE_PATH:-/home/ubuntu/vle}"

if [[ -z "${TARGET}" ]]; then
  echo "ERROR: no SSH target provided." >&2
  echo "       Pass it as a positional argument or set STANDBY_HOST." >&2
  echo >&2
  usage >&2
  exit 1
fi

echo "==> Promoting standby at '${TARGET}' (remote VLE repo: ${REMOTE_PATH})"

# Sanity-check the remote layout up front so we fail loudly if the path is wrong.
if ! ssh "${TARGET}" "test -f ${REMOTE_PATH}/deploy/compose/docker-compose.yml \
                   && test -f ${REMOTE_PATH}/deploy/.env"; then
  echo "ERROR: ${REMOTE_PATH}/deploy/{compose/docker-compose.yml,.env} not found on ${TARGET}." >&2
  echo "       Override the path with SSH_REMOTE_PATH if the repo lives elsewhere." >&2
  exit 2
fi

echo "==> Starting cloudflared via docker compose"
ssh "${TARGET}" "cd ${REMOTE_PATH}/deploy/compose \
  && docker compose --env-file ../.env --profile tunnel up -d cloudflared"

echo "==> Waiting 5s for the connector to register with Cloudflare's edge..."
sleep 5

echo "==> Standby cloudflared state:"
ssh "${TARGET}" "cd ${REMOTE_PATH}/deploy/compose \
  && docker compose --env-file ../.env --profile tunnel ps cloudflared"

cat <<EOF

==> Done. Next steps:
    1. Open the Cloudflare Zero Trust dashboard -> Networks -> Tunnels and
       confirm the standby connector appears (Connected, healthy).
    2. Refresh your production URL from a non-LAN device. The page should
       load through the standby host now.
    3. Once the primary host is back up, restore steady state:
         primary:   docker compose --env-file ../.env --profile tunnel up -d cloudflared
         standby:   ssh ${TARGET} 'cd ${REMOTE_PATH}/deploy/compose && \\
                      docker compose --env-file ../.env --profile tunnel stop cloudflared'
EOF
