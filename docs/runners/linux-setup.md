# Linux self-hosted runner (ephemeral Docker on Proxmox)

This is the recipe for standing up a **Linux x86_64 self-hosted GitHub
Actions runner** that destroys itself after every job. We run it as a
Docker container inside a Proxmox LXC; the container image
(`myoung34/github-runner:latest`) handles registration, ephemeral
lifecycle, and Docker-in-Docker (needed by cibuildwheel's manylinux
builds).

Designed for the lab; tagged `self-hosted, linux, x64` so the workflows
in `.github/workflows/` target it correctly.

## Build-only runner

This runner exists solely to **build wheels and run Rust tests** for vle CI
(`_build.yml` / `ci.yml`, jobs tagged `self-hosted, linux, x64`). It no longer
participates in any deploy step — the JupyterHub deployment moved to the
`homelab-iac` repo, which uses its own runner. So this LXC needs only outbound
HTTPS; it has **no Tailscale, no `--network host`, and no deploy access** to any
lab host.

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
  `pypi.org`, and `crates.io`. Inbound is not needed.
- A **GitHub Personal Access Token (classic)** with the `repo` scope.
  (Long-term, prefer a GitHub App token for finer-grained permissions;
  short-term, a PAT is simpler.) The PAT is used by the runner image
  to register itself with the repo, then refresh the registration
  token on each ephemeral spin-up. **Treat the PAT like a password**;
  store it only in the LXC's `.env` file or 1Password, never in the repo.

## Create the LXC

In the Proxmox web UI:

1. **Create CT** (container).
2. **General**: hostname e.g. `vle-runner`, unprivileged = **off**
   (we need privileged so Docker can run).
3. **Template**: `ubuntu-24.04-standard`.
4. **Disks**: 40 GB root (more if you'll cache multiple wheel
   architectures).
5. **CPU**: 4 cores.
6. **Memory**: 8192 MB.
7. **Network**: pick the lab VLAN (the one with internet egress but
   restricted access to your other lab services).
8. **DNS**: set a working resolver (your lab DNS, or `1.1.1.1`). The
   container needs outbound name resolution for `apt`, `curl`, and the
   GitHub/PyPI/crates endpoints.
9. **Confirm**, but **do not start yet**.

Edit `/etc/pve/lxc/<vmid>.conf` on the Proxmox host and append all
**three** lines below:

```
features: nesting=1,keyctl=1
lxc.apparmor.profile: unconfined
lxc.mount.entry: /dev/null sys/module/apparmor/parameters/enabled none bind,create=file 0 0
```

What each line is for, in order:

1. `features: nesting=1,keyctl=1` — lets dockerd set up cgroups and
   apt's gnupg work inside the LXC.
2. `lxc.apparmor.profile: unconfined` — strips the host's
   `lxc-container-default-cgns` AppArmor profile from the LXC so the
   container's userspace isn't fighting the host's AppArmor namespace.
3. `lxc.mount.entry: /dev/null sys/module/apparmor/parameters/enabled
   ... 0 0` — bind-mounts `/dev/null` over the AppArmor-enabled flag
   that Docker reads. Without this, modern `runc` (1.2.7+/1.3.2+)
   tries to load the `docker-default` AppArmor profile, fails because
   the LXC has no policy-admin rights on the host's AppArmor
   namespace, and refuses to start any container. This is the
   workaround for the
   [CVE-2025-52881 ecosystem regression](https://github.com/opencontainers/runc/issues/4968).
   If you are on `lxc-pve >= 6.0.5-2`, this line is harmless but
   redundant (the upstream fix is shipped).

Start the container after the edit.

### Bootstrap quirk on Ubuntu 24.04

One surprise before you can run anything inside the CT:

- **The Ubuntu 24.04 LXC template ships without `curl`.** The Docker
  installer relies on curl. With DNS set at creation (step 8 of
  *Create the LXC*), `apt` resolves fine, so just run
  `apt install -y curl` first.

## Install Docker inside the LXC

SSH into the LXC and:

```sh
# Bootstrap Docker (official one-liner).
curl -fsSL https://get.docker.com | sh

# Sanity check — should print "Hello from Docker!"
docker run --rm hello-world
```

If `hello-world` fails with
`apparmor_parser: Access denied. You need policy admin privileges`,
the AppArmor lines in `/etc/pve/lxc/<vmid>.conf` weren't picked up.
Stop the CT from the Proxmox host (`pct stop <vmid>`), confirm the two
AppArmor lines are present in the conf, then `pct start <vmid>` and
retry. A reboot from inside the CT is not enough — the LXC config is
only read on `pct start`.

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

This build-only runner uses Docker's default bridge network — it needs
only outbound HTTPS to GitHub/PyPI/crates and makes no inbound or deploy
connections to lab hosts.

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
   From your local machine:
   ```sh
   gh api repos/<owner>/<repo>/actions/runners --jq '.runners[] | {name, status, labels: [.labels[].name]}'
   ```
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
- **AppArmor unconfined**: the LXC runs without AppArmor confinement
  from the host. The kernel module is still loaded, but the LXC has
  no profile applied. The real isolation boundary for this runner is
  the LXC itself (a privileged container on its own Proxmox CT), not
  AppArmor. This is the standard pattern for Docker-in-LXC on
  Proxmox and is widely deployed; just don't expect AppArmor to save
  you from a runtime escape.
- **VLAN isolation**: keep the runner LXC on a network segment that
  cannot reach your other lab services. Outbound to internet is fine;
  inbound from internet should not be possible. (This is a build-only
  runner — it makes no inbound or deploy connections to lab hosts.)
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
  itself once a month. After a kernel update on the Proxmox host,
  re-check that all five `/etc/pve/lxc/<vmid>.conf` lines are still
  in place — a `pct restore` from backup can drop them.
- **No log rotation needed** — ephemeral containers don't accumulate
  logs locally; GitHub holds the run logs.

## Troubleshooting

- **`apparmor_parser: Access denied`** when running any docker
  container: the AppArmor lines in `/etc/pve/lxc/<vmid>.conf` aren't
  taking effect. Re-check that both lines are present and stop/start
  the CT from the Proxmox host (`pct stop <vmid> && pct start <vmid>`).
  A reboot from inside the CT is not enough.
- **`apt update` hangs on DNS at first boot**: `/etc/resolv.conf` is
  empty because no DNS was configured at LXC creation. Set a resolver
  in the CT's network config (step 8 of *Create the LXC*), or write
  `nameserver 1.1.1.1` to `/etc/resolv.conf`.
- **`curl: command not found`** right after LXC creation: the Ubuntu
  24.04 template doesn't include curl. `apt install -y curl` first.
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
- [opencontainers/runc#4968](https://github.com/opencontainers/runc/issues/4968) — CVE-2025-52881, the runc/AppArmor regression that the `/dev/null` bind-mount workaround addresses
