# Linux self-hosted runner (ephemeral Docker on Proxmox)

This is the recipe for standing up a **Linux x86_64 self-hosted GitHub
Actions runner** that destroys itself after every job. We run it as a
Docker container inside a Proxmox LXC; the container image
(`myoung34/github-runner:latest`) handles registration, ephemeral
lifecycle, and Docker-in-Docker (needed by cibuildwheel's manylinux
builds).

Designed for the lab; tagged `self-hosted, linux, x64` so the workflows
in `.github/workflows/` target it correctly.

## Why ephemeral?

Ephemeral = "container is created when a job starts, destroyed when
the job ends." Each CI job gets a clean slate. Benefits:

- **No state leakage** between jobs (a wedged build can't corrupt
  the next).
- **Safer for self-hosted hardware**: a compromised job dies with the
  container.
- **Zero maintenance**: there is no long-running service to update,
  patch, or restart.

The host LXC still needs occasional updates (Docker, kernel), but the
*runner* itself is created fresh per job.

## Prerequisites

- A Proxmox VE host with enough capacity for 4 vCPU / 8 GB RAM / 40 GB
  disk per concurrent runner (more if you want to run 2–3 in parallel).
- A network segment that allows outbound HTTPS to `github.com`,
  `pkg.cloudflare.com`, `pypi.org`, `crates.io`, GHCR, and the
  `quay.io/jupyter` registry. Inbound is not needed.
- A **GitHub Personal Access Token (classic)** with the `repo` scope.
  (Long-term, prefer a GitHub App token for finer-grained permissions;
  short-term, a PAT is simpler.) The PAT is used by the runner image
  to register itself with the repo, then refresh the registration
  token on each ephemeral spin-up. **Treat the PAT like a password**;
  store it only in the LXC's `.env` file, never in the repo.

## Create the LXC

In the Proxmox web UI:

1. **Create CT** (container).
2. **General**: hostname e.g. `vle-runner-01`, unprivileged = **off**
   (we need privileged so Docker can run).
3. **Template**: `ubuntu-24.04-standard`.
4. **Disks**: 40 GB root (more if you'll cache multiple wheel
   architectures).
5. **CPU**: 4 cores.
6. **Memory**: 8192 MB.
7. **Network**: pick the lab VLAN (the one with internet egress but
   restricted access to your other lab services).
8. **Confirm**, but **do not start yet**.

Edit `/etc/pve/lxc/<vmid>.conf` on the Proxmox host and add:

```
features: nesting=1,keyctl=1
```

These two flags let Docker run inside the LXC. Start the container
after this edit.

## Install Docker inside the LXC

SSH into the LXC and:

```sh
# Bootstrap Docker (official one-liner).
curl -fsSL https://get.docker.com | sh

# Optional: run docker without sudo as the lab user.
sudo usermod -aG docker $USER
newgrp docker

# Sanity check.
docker run --rm hello-world
```

## Launch the runner

Pick a runner name (per-container, anything unique to this LXC) and
launch the image. Replace `<PAT>` with the token from
"Prerequisites" above and `miguelju/vle` with the repo if you forked.

```sh
docker run -d --restart=unless-stopped \
    --name vle-runner-01 \
    -e RUNNER_NAME=vle-runner-01 \
    -e REPO_URL=https://github.com/miguelju/vle \
    -e ACCESS_TOKEN=<PAT> \
    -e EPHEMERAL=true \
    -e RUNNER_SCOPE=repo \
    -e LABELS=self-hosted,linux,x64 \
    -v /var/run/docker.sock:/var/run/docker.sock \
    myoung34/github-runner:latest
```

The `EPHEMERAL=true` flag is the load-bearing line: the image will
register the runner, wait for one job, run it, then deregister and
exit. Combined with `--restart=unless-stopped`, docker immediately
spawns a fresh container that registers again, ready for the next job.

`/var/run/docker.sock` is mounted so cibuildwheel can spawn the
manylinux container during the wheel build. This is the main attack
surface — read the **Security** section below.

## Scale to multiple concurrent runners (optional)

Run more containers with different `RUNNER_NAME`s:

```sh
for n in 02 03; do
  docker run -d --restart=unless-stopped \
      --name vle-runner-$n \
      -e RUNNER_NAME=vle-runner-$n \
      -e REPO_URL=https://github.com/miguelju/vle \
      -e ACCESS_TOKEN=<PAT> \
      -e EPHEMERAL=true \
      -e RUNNER_SCOPE=repo \
      -e LABELS=self-hosted,linux,x64 \
      -v /var/run/docker.sock:/var/run/docker.sock \
      myoung34/github-runner:latest
done
```

Three runners can handle ~3 concurrent jobs (one wheel build + lint
+ test, for example) without queueing. Beyond that, watch CPU and
disk pressure.

## Verify

1. **GitHub side**: open **Settings → Actions → Runners** in your repo;
   you should see `vle-runner-01` (and any other names) listed as
   "Idle". The label set must include `self-hosted`, `linux`, `x64`.
2. **Trigger a test workflow**: from a branch, push a small commit
   touching any file. The `lint-rust` job runs on a hosted runner;
   `test-rust` and `build` (Linux x86_64) should land on your
   self-hosted container. Watch the Actions UI — the runner name
   appears in the job's "Set up job" step.
3. **After the job finishes**, the runner disappears from the Runners
   tab for a few seconds while the ephemeral container is recreated,
   then reappears as "Idle". This is normal.

## Security notes

- **Docker socket mount = root on host**. A malicious or compromised
  job could in principle escape the runner container via the docker
  socket. We accept this trade-off because (a) the fork-PR guard in
  `_build.yml` blocks PRs from forks from ever reaching this runner,
  (b) we don't run untrusted code paths, (c) ephemeral lifecycle
  blast-radius-limits any exploitation to a single job. Long-term, an
  isolated dockerd-in-docker setup eliminates the socket mount; we
  haven't needed it.
- **VLAN isolation**: keep the runner LXC on a network segment that
  cannot reach your other lab services. Outbound to internet is fine;
  inbound from internet should not be possible.
- **PAT scope**: the classic `repo`-scoped PAT can do quite a lot; if
  this runner is ever shared with a teammate, rotate to a GitHub App
  token with `Actions: write` + `Self-hosted runners: write` only.

## Maintenance (effectively zero)

- **Update the runner image** every month or two:
  ```sh
  docker pull myoung34/github-runner:latest
  docker rm -f vle-runner-01
  # re-run the `docker run` from above
  ```
  Ephemeral containers picking up the new image on their next cycle —
  no orchestration needed.
- **Patch the LXC** with `apt update && apt upgrade -y` on the LXC
  itself once a month.
- **No log rotation needed** — ephemeral containers don't accumulate
  logs locally; GitHub holds the run logs.

## Troubleshooting

- **Runner shows offline in GitHub**: `docker ps -a | grep github-runner`
  on the LXC. If the container has exited and isn't restarting,
  `docker logs <name>` will show the registration error (usually a
  bad PAT or the runner being orphaned in GitHub's database — go to
  Settings → Actions → Runners and delete the stale entry first).
- **`failed to start runner: docker socket permission`**: confirm
  `/var/run/docker.sock` is mounted and the runner image's UID maps
  to the host's `docker` group. The image handles this automatically
  in standard configurations; if you customised the host, you may
  need `-v /var/run/docker.sock:/var/run/docker.sock:rw` and
  `--group-add $(getent group docker | cut -d: -f3)`.
- **Disk full**: cibuildwheel containers can leave stale layers
  behind. `docker system prune -af` reclaims space; consider a cron
  every Sunday.

## See also

- [`../ci.md`](../ci.md) — overall CI architecture
- [`macos-setup.md`](macos-setup.md) — the persistent Mac mini runner
- [`myoung34/github-runner` on GitHub](https://github.com/myoung34/docker-github-actions-runner)
