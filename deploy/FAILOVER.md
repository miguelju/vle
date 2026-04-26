# Warm-Standby Failover via Cloudflare Tunnel

This guide adds a second host as a **warm standby** to the VLE deployment so
the public URL stays reachable when the primary host goes down. It uses
**Cloudflare Tunnel's free same-tunnel replica feature** — no paid Cloudflare
Load Balancer, no external orchestration, no shared storage.

If you only run the stack on one host, you don't need any of this.

## What you get

- The public URL (`https://$DOMAIN/`) keeps resolving when the primary host
  dies, after a single command on a workstation that can SSH into the standby.
- ~2-3 minutes of perceived outage in the worst case (Cloudflare's connector
  health-check window plus operator reaction time).
- No DNS changes during failover. The named tunnel's connector list is the
  load balancer; Cloudflare drops dead connectors automatically.

## What you don't get

- **Hot/automatic failover.** This is *cold* standby — the standby's
  cloudflared connector is stopped in steady state. You run a one-line
  command (or the supplied script) to promote it.
- **Session preservation.** Per-user JupyterHub state (running kernels,
  unsaved work) lives on a single host's docker volumes. When that host
  dies, those users get a fresh `~/work/` on the standby and lose
  in-progress work. New logins after promotion are unaffected.
- **Active/active load balancing.** Cloudflare's free tier has no sticky
  sessions, so running both connectors simultaneously would round-robin
  users between two unsynchronised JupyterHub databases. Don't do it
  outside of brief testing windows.

For a deployment where these matter, you want a paid Cloudflare Load
Balancer (sticky sessions) plus shared storage (NFS, Tailscale ts.net
filer, S3-fuse, etc.) plus a synchronised JupyterHub state DB. That's a
different architecture.

## Architecture

```
                  Cloudflare edge
                         |
                Named tunnel (one ID,
                 one auth token)
                  /              \
       cloudflared              cloudflared
       (primary)                (standby — STOPPED in steady state)
            |                        |
       Traefik                  Traefik
            |                        |
       JupyterHub               JupyterHub
       (per-user                (per-user
        containers)              containers)
```

Both hosts run the same compose stack from this repo. Both have the same
Cloudflare origin certificate at the same path. Both have the same
`CLOUDFLARED_TUNNEL_TOKEN` value in `deploy/.env`. The CF dashboard's
Public Hostname routing for `$DOMAIN` is unchanged.

## Prerequisites

Both hosts must already be running the stack from this repo and pass
`deploy/scripts/deploy.sh`'s self-checks individually before you wire them
into a tunnel.

In addition:

1. **A named Cloudflare Tunnel** (not a quick tunnel) with `$DOMAIN` routed
   to it. If you already serve `$DOMAIN` through `cloudflared`, you have one
   — note the tunnel's connector token.
2. **The same Cloudflare origin certificate** at
   `~/traefik/certs/{cert.pem,key.pem}` on both hosts. Cloudflare Tunnel does
   not care about the cert (TLS terminates at the edge), but the connector
   forwards to Traefik over the internal docker network using the configured
   service URL — typically `http://traefik:80` — and Traefik routes by Host
   header. The cert is needed if you also serve direct LAN traffic on `:443`.
3. **The same `CLOUDFLARED_TUNNEL_TOKEN` in `deploy/.env` on both hosts.**
   This is what makes the standby register as a *replica* of the same
   tunnel, not a new tunnel.
4. **SSH access to the standby** from whatever workstation will run the
   promotion command, ideally via an alias in `~/.ssh/config`.

## Setup walkthrough

These steps assume `primary` is already serving production through the named
tunnel; you are adding `standby` as a new replica.

### 1. Match TLS identity

Copy the Cloudflare origin cert from primary to standby (paths are
operator-specific; the standard layout is `~/traefik/certs/`):

```sh
scp primary:~/traefik/certs/{cert.pem,key.pem} standby:/tmp/
ssh standby 'sudo install -m 644 /tmp/cert.pem ~/traefik/certs/cert.pem && \
             sudo install -m 600 /tmp/key.pem  ~/traefik/certs/key.pem  && \
             shred -u /tmp/cert.pem /tmp/key.pem && \
             docker compose -f ~/traefik/docker-compose.yml restart traefik'
```

Verify both hosts present the same issuer + dates:

```sh
openssl s_client -connect <host>:443 -servername $DOMAIN </dev/null 2>/dev/null \
  | openssl x509 -noout -issuer -dates
```

### 2. Set the tunnel token on the standby

Read it from primary, paste into standby's `deploy/.env`:

```sh
ssh primary 'grep ^CLOUDFLARED_TUNNEL_TOKEN ~/vle/deploy/.env'
ssh standby '$EDITOR ~/vle/deploy/.env'   # paste the same value
```

Standby's `.env` should also have the same `DOMAIN`, `AUTH_MODE`,
`CF_ACCESS_TEAM_DOMAIN`, and `JUPYTERHUB_ADMIN_EMAIL` as primary so the
JupyterHub config and Traefik labels match.

### 3. First-run on standby (registers, then stop)

```sh
ssh standby 'cd ~/vle && ./deploy/scripts/deploy.sh'
```

`deploy.sh` detects the populated token and activates the `tunnel` compose
profile, starting the cloudflared container. Confirm in the Cloudflare
Zero Trust dashboard → Networks → Tunnels → your tunnel that **2
connectors** are now listed (one per host), both Connected.

Now stop the standby's connector — cold standby steady state:

```sh
ssh standby 'cd ~/vle/deploy/compose && \
             docker compose --env-file ../.env --profile tunnel stop cloudflared'
```

The dashboard should drop to **1 connector** (primary). The container
remains on the standby (stopped); promotion brings it back up in seconds.

## Day-to-day operations

### Steady state

- Primary's `cloudflared` is `Up`.
- Standby's `cloudflared` exists but is `Stopped`.
- Cloudflare dashboard shows **1 connector**, status HEALTHY.
- All traffic flows through primary.

### Detecting a failure

- Cloudflare dashboard → Tunnels → your tunnel: **0 connectors**, or
  status DEGRADED/DOWN.
- `https://$DOMAIN/` returns a Cloudflare error page (1033, 1014, 502...).
- Whatever upstream monitoring you have (uptime checks, etc.) fires.

### Promotion (failover)

From a workstation with SSH access to the standby:

```sh
deploy/scripts/promote-standby.sh <ssh-target>
# or
STANDBY_HOST=<ssh-target> deploy/scripts/promote-standby.sh
```

The script SSHes in, runs `docker compose --profile tunnel up -d cloudflared`,
waits 5 seconds, prints the container state, and reminds you to verify in
the dashboard. Idempotent — safe to re-run if you're not sure.

Expect the public URL to recover within ~30-60 seconds of the script
finishing (Cloudflare needs a moment to add the new connector to its
routing tables).

### Restoring steady state

Once the primary is healthy again:

```sh
# Bring primary back into rotation
ssh <primary> 'cd ~/vle/deploy/compose && \
               docker compose --env-file ../.env --profile tunnel up -d cloudflared'

# Stop the standby so we're back to 1-connector cold standby
ssh <standby> 'cd ~/vle/deploy/compose && \
               docker compose --env-file ../.env --profile tunnel stop cloudflared'
```

Confirm dashboard shows **1 connector** (primary's). Public URL traffic now
flows through primary again.

### Planned primary maintenance

To take primary down without disrupting users, promote first, then stop
primary:

```sh
deploy/scripts/promote-standby.sh <ssh-target>
# (wait for dashboard to show standby's connector — usually <10s)
ssh <primary> 'cd ~/vle/deploy/compose && \
               docker compose --env-file ../.env --profile tunnel stop cloudflared'
# do whatever maintenance you need on primary, then reverse:
ssh <primary> 'cd ~/vle/deploy/compose && \
               docker compose --env-file ../.env --profile tunnel up -d cloudflared'
ssh <standby> 'cd ~/vle/deploy/compose && \
               docker compose --env-file ../.env --profile tunnel stop cloudflared'
```

## Common pitfalls

### "Too many redirects" on the public URL after promoting

The CF dashboard's Public Hostname for `$DOMAIN` typically forwards to
`http://traefik:80` (TLS terminates at the Cloudflare edge). If the standby's
Traefik has a global HTTP→HTTPS redirect on the `web` entrypoint
(`--entrypoints.web.http.redirections.entrypoint.to=websecure`), every
request from cloudflared is bounced back to `https://$DOMAIN/`, which the
client re-fetches, which re-redirects — infinite loop. The hub never sees
a request.

Fix: do not enable the global HTTP→HTTPS redirect on either host's Traefik.
The redirect is only useful for clients hitting Traefik directly on port 80
*outside* the tunnel; if you need that for LAN access, define a per-router
redirect middleware on the LAN-only routers instead of an entrypoint-wide
redirect.

### Promotion script reports "deploy/.env not found on standby"

The script defaults to `/home/ubuntu/vle` as the remote repo path. Override
with `SSH_REMOTE_PATH=/your/path deploy/scripts/promote-standby.sh ...` if
your standby uses a different layout.

### Only one connector ever appears in the dashboard

The two hosts have different `CLOUDFLARED_TUNNEL_TOKEN` values, so they're
talking to different tunnels (or one isn't talking at all). Re-copy from
primary and verify byte-for-byte:

```sh
ssh <primary> 'sha256sum <(grep ^CLOUDFLARED_TUNNEL_TOKEN ~/vle/deploy/.env)'
ssh <standby> 'sha256sum <(grep ^CLOUDFLARED_TUNNEL_TOKEN ~/vle/deploy/.env)'
```

### Origin cert expiry sneaks up

Cloudflare origin certs are issued by Cloudflare and last as long as you
ask (15 years is the maximum). Set a calendar reminder a few months
before expiry. Rotation = issue a new cert in the dashboard, replace the
two files on each host, `docker compose restart traefik` per host.

## Trade-off summary

| You want | Use this | Cost |
|---|---|---|
| Single host, no failover | Just `deploy/README.md` | $0, simplest |
| Cold standby on free CF tier | This doc | $0, ~2-3 min RTO, manual promote |
| Active/active with sticky sessions | CF Load Balancer + shared storage | Paid, complex |
| Multi-region with shared user state | Kubernetes + NFS + sync'd hub DB | Significantly more ops |

Cold standby is the right answer for homelab and small-team deployments
where ~3 minutes of downtime during a host failure is acceptable and the
ops cost of anything fancier is not.
