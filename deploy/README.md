# Deployment

This directory contains the Docker-based deployment for the VLE JupyterHub
service. It is intended to be run on a Linux host (ARM64 targeted) behind a
pre-existing Traefik reverse proxy, with upstream authentication handled by
Cloudflare Access or an equivalent header-setting gateway.

## Layout

```
deploy/
├── README.md              this file
├── FAILOVER.md            optional warm-standby setup using a shared CF tunnel
├── .env.example           template — copy to .env and fill in
├── compose/
│   ├── docker-compose.yml
│   └── jupyterhub_config.py
├── docker/
│   ├── Dockerfile.jupyterhub   hub image (ARM64)
│   └── Dockerfile.notebook     per-user image (ARM64)
└── scripts/
    ├── deploy.sh                pull + rebuild + restart
    └── promote-standby.sh       promote a warm standby (see FAILOVER.md)
```

Files under `deploy/local/` and `deploy/.env` are gitignored — use them for
operator-specific notes and real secrets.

## Prerequisites

On the deployment host:

1. **Docker Engine** with the Compose v2 plugin (`docker compose ...`).
2. **An external Docker network** for the reverse proxy, e.g.
   ```
   docker network create web
   ```
   The name must match `TRAEFIK_NETWORK` in `.env`.
3. **A running reverse proxy** attached to that network, terminating TLS and
   forwarding HTTP requests for `$DOMAIN` to the JupyterHub container. Traefik
   v3 with standard host-routing labels is assumed; the compose file emits the
   matching labels automatically.
4. **An upstream authenticator** (e.g. Cloudflare Access) that injects an
   authenticated email into the `Cf-Access-Authenticated-User-Email` header.
   Your reverse proxy must not pass through a client-supplied copy of that
   header. For local development, set `AUTH_MODE=dummy` to bypass this.
5. **ARM64 host** — the Dockerfiles pin `--platform=linux/arm64`. On an
   x86_64 host you can build cross-platform with `docker buildx`, or edit the
   `FROM` lines to drop the platform pin.

Optionally, if you do not want to expose any public ports on the host, set
`CLOUDFLARED_TUNNEL_TOKEN` in `.env` and the stack will also run a
`cloudflared` container that terminates a Cloudflare Tunnel and forwards to
Traefik on the internal `web` network. Leave the token blank if you terminate
TLS/ingress yourself (e.g. via a public port on Traefik).

For a second host as warm standby on the same Cloudflare Tunnel — so the
public URL stays reachable when the primary goes down — see
[`FAILOVER.md`](FAILOVER.md). Free Cloudflare tier, no Load Balancer
required, ~2-3 min cold-standby RTO.

## First-time setup

```sh
# 1. Clone the repo and enter it
git clone https://github.com/YOUR/vle.git
cd vle

# 2. Create .env from the template and edit it
cp deploy/.env.example deploy/.env
$EDITOR deploy/.env

# 3. Build both images (hub + notebook)
cd deploy/compose
docker compose --env-file ../.env --profile build-only build
docker compose --env-file ../.env build

# 4. Start the hub
docker compose --env-file ../.env up -d
```

Verify with `docker compose --env-file ../.env ps` and `docker compose logs -f jupyterhub`.

## Updates

Use the convenience script:

```sh
deploy/scripts/deploy.sh              # fast, uses Docker layer cache
deploy/scripts/deploy.sh --no-cache   # full rebuild from scratch
```

It pulls `origin/main`, rebuilds both images, restarts the stack, and then
runs a short self-check (hub running, config sentinels present in the built
images). If `--no-cache` is passed, both images are rebuilt from scratch —
slower but guaranteed-fresh. Use it after big changes or if the previous
deploy looks like it is running stale code.

## Local development (no reverse proxy)

```sh
# In deploy/.env
AUTH_MODE=dummy
DOMAIN=localhost
```

Then expose the hub port directly instead of relying on Traefik, e.g. by
adding `ports: ["8000:8000"]` to the `jupyterhub` service in a compose
override file. The dummy authenticator accepts any username/password — never
use it on a public host.

## Troubleshooting

**`network web not found`** — create it with `docker network create $TRAEFIK_NETWORK`.

**Hub starts but browser hangs on `/hub/login`** — upstream auth isn't
reaching the hub. Confirm the reverse proxy is forwarding the
`Cf-Access-Authenticated-User-Email` header. You can exec into the hub and
`curl -I http://localhost:8000/hub/login` to inspect behavior without the
proxy in the loop.

**`Error response from daemon: No such image: vle-notebook:latest`** — the
single-user image wasn't built. Run
`docker compose --env-file ../.env --profile build-only build` from
`deploy/compose/`.

**User containers never get culled** — check the `idle-culler` service in
`docker compose logs jupyterhub`. Confirm `IDLE_TIMEOUT_MINUTES` is set.

**Bundled notebooks or `components.db` not appearing in `~/work/`** — the
first-start hook at `/usr/local/bin/before-notebook.d/10-seed-user-home.sh`
only copies from `/opt/vle/` when the target paths do not already exist. If
you expect a fresh copy after rebuilding the image, delete the user's volume
(`docker volume rm vle_vle-user-<email>`) and re-spawn.

**DockerSpawner can't reach the hub** — the hub and spawned containers must
share a Docker network. The compose file attaches the hub to `$TRAEFIK_NETWORK`
and DockerSpawner uses the same network name; if you change one, change both.

**Resource limits look ignored** — `CONTAINER_MEM_LIMIT` and
`CONTAINER_CPU_LIMIT` apply to *spawned* user containers, not the hub itself.
Inspect a live user container with `docker inspect` to confirm.
