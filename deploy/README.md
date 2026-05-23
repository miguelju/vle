# Distribution & Deployment

This repository is open-source educational software, and **how the code
reaches its users** is itself part of what it teaches. Source code travels
into other people's environments through a small handful of channels —
crates.io, PyPI, container registries, self-hosted services — and each one
exists for a different kind of user. This directory hosts one of them: a
self-hostable, multi-user JupyterHub stack.

**Important framing.** The Docker-based JupyterHub deployment described here
is **not the final intended distribution channel** for `vle-thermo`. Most
users should install via `pip` or `cargo` (see below). The hub stack is here
because it is a useful **educational reference** — a fully worked example of
how to run a shared JupyterHub for *any* Python coursework, not just this
thermodynamics project. Swap the notebook Dockerfile for your own and you
have a turnkey classroom hub.

## How `vle-thermo` reaches users

| Audience | Channel | Install / launch |
|---|---|---|
| Python users (most readers) | **PyPI** | `pip install vle-thermo` |
| Learners working through the bundled notebooks | **GitHub** ([`notebooks/`](https://github.com/miguelju/vle/tree/main/notebooks)) | clone the repo, `pip install "vle-thermo[plot]" jupyterlab`, then open in your preferred notebook viewer — no special kernel needed |
| Rust developers embedding the engine | **crates.io** | `cargo add vle-thermo` &nbsp; `cargo add vle-units` |
| Self-hosters wanting a one-container demo | **Standalone Dockerfile** (in this repo) | `docker build -f deploy/docker/Dockerfile.standalone -t vle-standalone .` then `docker run --rm -p 8888:8888 vle-standalone` |
| Educators hosting a class / lab | **This directory** | self-hosted JupyterHub (see [Quick start](#quick-start-jupyterhub-on-your-own-machine)) |

The five channels are layered: each one trades a little more setup work for
more capability. PyPI is "I want to call the library from a Python script";
the JupyterHub stack is "I want twenty students to all have a working
environment in their browser without installing anything on their laptops."

### crates.io — Rust source distribution

[crates.io](https://crates.io) is the official public registry for Rust
libraries (analogous to PyPI for Python or npm for JavaScript). Two crates
from this project live there:

- **[`vle-thermo`](https://crates.io/crates/vle-thermo)** — the numerical
  engine (cubic EOS solvers, activity models, flash algorithms).
- **[`vle-units`](https://crates.io/crates/vle-units)** — the unit registry
  and dimensional-analysis layer; usable standalone in any Rust project
  that needs thermodynamic-flavored units (see [`units/README.md`](../units/README.md)).

Publishing is one command per crate: `cargo publish`. crates.io stores
*source code*; when a downstream user runs `cargo add vle-thermo` and then
`cargo build`, their own machine compiles the source into a native library.
That works because every Rust developer has a Rust toolchain on hand — for
Rust users, "source distribution + local compile" is the natural shape.

### PyPI — pre-compiled binary wheels

[PyPI](https://pypi.org) (the Python Package Index) is the standard
registry `pip` reads from. The Python package
**[`vle-thermo`](https://pypi.org/project/vle-thermo/)** lives there.

Python users don't typically have a Rust toolchain, so PyPI wouldn't accept
the source-distribution model. Instead, this project ships **pre-compiled
binary wheels** — one per (operating system, CPU architecture) combination:
Linux x86_64, Linux aarch64, macOS arm64, Windows x86_64. Each wheel
embeds a pre-built native extension module, so `pip install vle-thermo`
lands ready-to-import code on the user's machine without any compiler
involvement.

The wheels are produced by **[`maturin`](https://maturin.rs)**, the
build tool that packages PyO3-using Rust crates into Python wheels. CI
runs [`cibuildwheel`](https://cibuildwheel.pypa.io) once per
(OS, arch) combination, calling maturin inside each container, and uploads
the resulting matrix of wheels to PyPI. The full mechanics — what PyO3
generates, what maturin does step-by-step, how a function call traverses
the FFI boundary — live in [`python/README.md`](../python/README.md), in
the section "How the Python package wraps Rust".

The takeaway for the distribution story: the **same Rust source** is what
gets published to crates.io and what gets compiled into the PyPI wheels.
Two different ecosystems, one source of truth, no parallel implementations
to keep in sync.

### Notebooks from the repo (work through them locally)

If you'd rather learn from the bundled notebooks on your own machine —
in JupyterLab, classic Jupyter Notebook, VS Code's Jupyter extension, or
any other client — pull them straight from the
[`notebooks/`](https://github.com/miguelju/vle/tree/main/notebooks)
directory:

```sh
git clone https://github.com/miguelju/vle.git
cd vle
pip install "vle-thermo[plot]" jupyterlab    # or `jupyter` / VS Code Jupyter
jupyter lab notebooks/                        # opens the notebook folder
```

**Kernel:** nothing custom is required. The notebooks declare their
kernel as `python3` (the standard `ipykernel` name), so any Jupyter
install with a Python 3.10+ environment that has `vle-thermo` installed
will run them. The `import vle` statements pick up whatever interpreter
your notebook client has selected — for VS Code users, that's the
"Select Kernel" picker; for `jupyter lab`, it's whichever environment
you ran the command from. There's nothing `vle`-specific about the
kernel itself.

> The `vle-thermo[plot]` extra pulls in `matplotlib`, which the
> notebooks use for P-x-y plots. Skip the `[plot]` if you only care
> about the numerical cells.

### Standalone Docker image (build-it-yourself, optional GHCR publish)

For people who just want to *try* the notebooks without setting up a
Python environment, this repo ships a self-contained Dockerfile at
[`deploy/docker/Dockerfile.standalone`](docker/Dockerfile.standalone)
that bundles `jupyter/scipy-notebook` + `vle-thermo` + the example
notebooks. Build and run it locally:

```sh
docker build -f deploy/docker/Dockerfile.standalone -t vle-standalone .
docker run --rm -p 8888:8888 vle-standalone
```

If you want to host the image yourself so others can `docker pull` it
without cloning the repo, [`deploy/scripts/publish-docker.sh`](scripts/publish-docker.sh)
publishes it as `ghcr.io/<your-user>/vle-thermo` (multi-arch
linux/amd64 + linux/arm64). It needs a buildx builder and a GitHub PAT
with `write:packages` scope — see the comment header at the top of the
script for the prerequisites and a dry-run / `--go` flag.

> **Note:** as of this writing the upstream `ghcr.io/miguelju/vle-thermo`
> image is **not** automatically built by CI. The scaffolding above is
> what's maintained; the public image is not. Treat the standalone
> Dockerfile as the source of truth and rebuild from it.

## Why JupyterHub is useful in educational settings

Jupyter notebooks have become a standard medium for teaching numerical
sciences and engineering. The pedagogical case rests on a few features
that are awkward in traditional formats:

- **Code, prose, math, and plots live in one document.** Students see the
  derivation → implementation → result in one continuous narrative,
  instead of toggling between a textbook, a code editor, and a plotting
  window.
- **Every result is re-executable.** Change a parameter, re-run a cell,
  see the new plot. Every worked example becomes a sandbox.
- **Notebooks climb [Bloom's taxonomy](https://en.wikipedia.org/wiki/Bloom%27s_taxonomy)
  naturally.** A single notebook can progress from "remember the
  definition" through "apply the equation" to "design a new flash
  configuration", because earlier cells set up the state the later ones
  build on.

The canonical reference is
[**Teaching and Learning with Jupyter**](https://jupyter4edu.github.io/jupyter-edu-book/)
by Lorena A. Barba and 14 co-authors — a free open book written
collaboratively by STEM educators using Jupyter in their own classrooms,
with a dedicated chapter on [why Jupyter notebooks for teaching](https://jupyter4edu.github.io/jupyter-edu-book/why-we-use-jupyter-notebooks.html).

For thermodynamics and chemical engineering specifically:

- [**Using Jupyter Notebooks to Climb Bloom's Taxonomy in Thermodynamics**](https://peer.asee.org/work-in-progress-using-jupyter-notebooks-to-climb-bloom-s-taxonomy-in-thermodynamics)
  (ASEE PEER, Weber et al.) — direct evidence from a sophomore-level
  Mechanical Engineering thermodynamics course.
- [**PyTherm applied thermodynamics**](https://chemistry.coe.edu/piper/posts/pytherm-applied-thermo/)
  (Coe College's PIPER — Physical Chemistry teaching resource collection).
- [**MatSciEd/Thermodynamics**](https://github.com/MatSciEd/Thermodynamics)
  — a public collection of teaching notebooks for unary and binary phase
  diagrams.
- [**Jupyter Notebooks for advanced topics in Fluid Mechanics**](https://onlinelibrary.wiley.com/doi/full/10.1002/cae.22619)
  (Castilla, *Computer Applications in Engineering Education*, 2023).

**JupyterHub** is what turns a single-user Jupyter installation into a
*multi-user* service. Each student gets their own isolated kernel and
home directory; the instructor brings up one host, points students at one
URL, and every student has a working Python environment without
touching their laptop's setup. That property — every student lands in
the same environment, on day one, no install instructions to debug — is
the operational win for teaching.

The Docker Compose stack in this directory is one packaged way to set
that up. Replace the per-user notebook image with one tailored to your
own course (any Python packages, any notebooks) and you have a
JupyterHub for your class.

**Operational note: the hub is not rebuilt on every `vle-thermo` PyPI
release.** The bundled engine version may lag PyPI by days or weeks —
this is intentional. The hub exists to teach the *workflow*, not to
serve as a production-pinned environment. Each milestone notebook
includes an optional `%pip install --upgrade vle-thermo` cell
(commented out by default) so a user can opt in to the latest version
mid-session; for sustained work, the recommended path is to install
`vle-thermo` in your own Jupyter environment (see the
[distribution table](#how-vle-thermo-reaches-users) above) where the
install is permanent and the version is under your control.

**Concretely, the CI deploy splits into two modes** (`.github/workflows/release.yml`,
job `deploy-sandbox`):

- **Auto on every `v*` tag — notebooks-only fast path.** The host runs
  `deploy/scripts/deploy-notebooks.sh`: regenerate `index.ipynb`, stamp
  `notebooks/.notebook-version` with the tag, done. No docker build, no
  stack restart. Per-user containers bind-mount the host's notebooks
  directory read-only (see `NOTEBOOK_HOST_PATH` in `.env.example`), so
  new spawns pick up the fresh content immediately; existing user
  volumes re-seed their managed notebooks on next login via the version
  marker.
- **Manual `workflow_dispatch` with `full_deploy=true` — image rebuild.**
  The host runs `deploy/scripts/deploy.sh`: rebuild both images (notebook
  + hub) and restart the stack. Use this for engine bumps, Dockerfile
  changes, or base-image rotations.

End-user impact: notebook content updates show up within ~15 seconds of
a release; engine updates require a manual click. The two modes share
the same SSH wrapper (`vle-deploy`) and force-command lockdown — they
differ only by the `notebooks:` prefix the CI sends. See
`deploy/local/auto-deploy/README.md` for the wrapper details.

## Quick start: JupyterHub on your own machine

This walks through running the hub locally with **OpenID Connect** auth
(via a self-hosted provider like [Pocket ID](https://pocket-id.org)) on
a single Linux host — laptop, lab workstation, or small VPS. Three
prerequisites:

1. **Docker Engine** with the Compose v2 plugin (`docker compose ...`).
   Install via your distro packages or [Docker's official instructions](https://docs.docker.com/engine/install/).
2. **An OIDC provider you control.** Pocket ID is the simplest option
   (self-hostable, ~10 MB container). You can also use Google,
   Microsoft, Authelia, Authentik, Keycloak, or anything else that
   speaks OIDC.
3. **An ARM64 host** (the bundled Dockerfiles pin `linux/arm64`). On
   x86_64 you can build with `docker buildx --platform linux/arm64` or
   edit the `FROM` lines in `deploy/docker/Dockerfile.*` to drop the
   pin — see [Prerequisites](#prerequisites) for details.

### 1. Clone and configure

```sh
git clone https://github.com/miguelju/vle.git
cd vle
cp deploy/.env.example deploy/.env
$EDITOR deploy/.env
```

Set in `.env`:

```sh
AUTH_MODE=oidc
DOMAIN=hub.example.local            # the hostname users will visit
OIDC_ISSUER=https://id.example.local/
OIDC_CLIENT_ID=vle-jupyterhub
OIDC_CLIENT_SECRET=...
OAUTH_CALLBACK_URL=https://hub.example.local/hub/oauth_callback
```

Register the `OAUTH_CALLBACK_URL` as an allowed redirect URI in your
OIDC provider's admin UI **before** the first login attempt.

### 2. Create the proxy network and build images

```sh
docker network create web

cd deploy/compose
docker compose --env-file ../.env --profile build-only build   # notebook image
docker compose --env-file ../.env build                         # hub image
```

### 3. Start the hub

```sh
docker compose --env-file ../.env up -d
docker compose --env-file ../.env logs -f jupyterhub
```

Visit `https://hub.example.local`, sign in through your OIDC provider,
and you should land on JupyterLab in a per-user container. Each user
gets their own home directory backed by a Docker named volume.

> **For a quick test without any auth or OIDC,** set `AUTH_MODE=dummy`
> and `DOMAIN=localhost` and expose port 8000 directly. The dummy
> authenticator accepts any username/password — fine for trying things
> on your laptop, never for a public host.

### Using it for non-`vle-thermo` Python work

The stack is intentionally generic. To make it a hub for a different
course or project:

1. Edit `deploy/docker/Dockerfile.notebook` — change the `pip install`
   lines to install whatever Python packages your course needs (or
   start `FROM` a different Jupyter base image).
2. Optionally edit the seed hook at
   `/usr/local/bin/before-notebook.d/10-seed-user-home.sh` to drop
   different starter notebooks into each user's home on first login.
3. Rebuild and restart:
   `docker compose --env-file ../.env --profile build-only build && docker compose --env-file ../.env up -d`.

Everything else (auth, spawning, culling, resource limits, reverse
proxying) works unchanged.

## About the auth and ingress choices

This stack uses three pieces of infrastructure that aren't strictly part
of JupyterHub but show up everywhere in the docs above. Worth a few
sentences each, because they're the kind of thing a reader will
otherwise google — and in every case the choice this stack made is **one
of many reasonable options**, not the right answer.

### OpenID Connect (OIDC)

OIDC is a thin standard layered on top of OAuth 2.0 that lets an
application delegate "who is this user?" to a separate *identity
provider* (IdP). The flow is: the user clicks "log in", the hub
redirects them to the IdP, the IdP authenticates the user however it
likes (password, passkey, 2FA, hardware key, SSO from yet another
provider), and hands back a cryptographically-signed token saying "this
is `alice@example.com`, valid for the next hour." The hub never sees
the user's password.

**Why it's useful for JupyterHub specifically:** the hub becomes a
"resource server" — it cares about *which* identified user is asking,
but it has no responsibility for password storage, password resets,
MFA, or account lockout. Those are the IdP's problem. For an educator
deploying a hub, this also means SSO with whatever the institution
already uses (Google Workspace, Microsoft Entra ID, an institutional
SSO portal).

**Other ways to do the same job:**

- **LDAP** — older on-prem directory protocol (Active Directory, OpenLDAP).
- **SAML** — enterprise federation; older but still common.
- **Header-based auth in front of the hub** — Cloudflare Access,
  [OAuth2 Proxy](https://oauth2-proxy.github.io/oauth2-proxy/),
  nginx's `auth_request`. The proxy authenticates and injects a header
  like `X-Forwarded-User`; the hub trusts the header. This is the
  `AUTH_MODE=cloudflare` legacy path.
- **The hub's own user database** (`PAMAuthenticator` against the host,
  or `LocalAuthenticator` with a password file). Simplest, no external
  service, but you're back to managing passwords.
- **GitHub OAuth, Google OAuth, etc.** — also OAuth 2.0 / OIDC under the
  hood, just hosted by a specific provider.

### Cloudflare Tunnel

A Cloudflare Tunnel is an **outbound-only** persistent connection from
your host to Cloudflare's edge network, established by a small daemon
(`cloudflared`) running alongside your service. Public traffic for your
domain arrives at Cloudflare and is routed *through* the tunnel to your
local service. The crucial property: **you don't open any inbound
ports** on your host. You can host from behind CGNAT, a restrictive home
router, a corporate firewall, or even mobile internet — Cloudflare
provides the public address.

**Why it's useful here:** removes the "I need a static public IP and
port forwarding on my router" problem that traditionally blocks people
from self-hosting. Free for personal use.

**Other ways to do the same job:**

- **Port forwarding on your router** — the classic approach. Requires a
  static public IP (or dynamic DNS) and admin access to the router.
- **A small VPS with a public IP** — DigitalOcean, Hetzner, Oracle's
  always-free tier. The hub runs on the VPS directly.
- **[Tailscale Funnel](https://tailscale.com/kb/1223/funnel)** — same
  outbound-only model, different vendor.
- **[ngrok](https://ngrok.com/), [frp](https://github.com/fatedier/frp),
  [Inlets](https://inlets.dev/)** — other tunnel implementations with
  different tradeoffs (free tier limits, self-hosted relay vs. SaaS).

### Traefik

[Traefik](https://traefik.io/traefik/) is the **reverse proxy** in this
stack — it terminates TLS, reads its routing rules from Docker container
labels, and forwards HTTP requests for `$DOMAIN` to the JupyterHub
container on the internal `web` network.

**Why it's useful here:** Traefik discovers containers automatically
from labels in `docker-compose.yml`, so adding a new service is a
matter of slapping on a few labels rather than editing a config file.
It also handles Let's Encrypt certificate provisioning on its own.

**Other ways to do the same job:**

- **[Caddy](https://caddyserver.com/)** — auto-HTTPS by default,
  arguably the simplest config format.
- **nginx** — the default for most of the web; manual config, very
  battle-tested.
- **HAProxy** — popular when you need fine-grained TCP/HTTP routing or
  high-throughput load balancing.
- **No proxy at all** — if your host has a single service on port 443
  and you're terminating TLS at Cloudflare (or via a tunnel), you can
  skip this layer entirely.

### The bottom line

JupyterHub itself only cares that *some* authenticator answers
"yes this user is X" and *some* networking gets requests to it. The
specific stack here — OIDC + Cloudflare Tunnel + Traefik — is one
coherent set of choices that works well together on a small Linux host.
Swap any of them for the equivalent of your choice and the rest still
runs.

## Prerequisites (production hosting)

For a real multi-user deployment behind a domain name, you'll typically
want everything from the quick start plus:

1. **A reverse proxy** terminating TLS in front of the hub. The compose
   file emits Traefik v3 host-routing labels automatically; substitute
   Caddy/nginx if you prefer (you'd edit `docker-compose.yml` accordingly).
2. **An external Docker network** for the proxy:
   ```sh
   docker network create web
   ```
   The name must match `TRAEFIK_NETWORK` in `.env`.
3. **An authenticator.** Pick one of:
   - `AUTH_MODE=oidc` (recommended) — covered in the quick start above.
   - `AUTH_MODE=cloudflare` (legacy) — an upstream gateway (Cloudflare
     Access) injects an authenticated email into the
     `Cf-Access-Authenticated-User-Email` header. Your reverse proxy
     **must not** pass through a client-supplied copy of that header.
   - `AUTH_MODE=dummy` (local development only) — no auth.
4. **ARM64 host** — the Dockerfiles pin `--platform=linux/arm64`. On an
   x86_64 host you can build cross-platform with `docker buildx`, or
   edit the `FROM` lines to drop the pin.

Optionally, if you do not want to expose any public ports on the host,
set `CLOUDFLARED_TUNNEL_TOKEN` in `.env` and the stack will also run a
`cloudflared` container that terminates a Cloudflare Tunnel and
forwards to the proxy on the internal `web` network.

For a second host as warm standby on the same Cloudflare Tunnel — so
the public URL stays reachable when the primary goes down — see
[`FAILOVER.md`](FAILOVER.md). Free Cloudflare tier, no Load Balancer
required, ~2-3 min cold-standby RTO.

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

Files under `deploy/local/` and `deploy/.env` are gitignored — use
them for operator-specific notes and real secrets.

## Updates

```sh
deploy/scripts/deploy.sh              # fast, uses Docker layer cache
deploy/scripts/deploy.sh --no-cache   # full rebuild from scratch
```

The script pulls `origin/main`, rebuilds both images, restarts the
stack, and then runs a short self-check (hub running, config sentinels
present in the built images). Use `--no-cache` after big changes or if
the previous deploy looks like it is running stale code.

## Troubleshooting

**`network web not found`** — create it with
`docker network create $TRAEFIK_NETWORK`.

**Hub starts but browser hangs on `/hub/login`** —
- `AUTH_MODE=cloudflare`: upstream auth isn't reaching the hub. Confirm
  the reverse proxy is forwarding the
  `Cf-Access-Authenticated-User-Email` header. You can exec into the
  hub and `curl -I http://localhost:8000/hub/login` to inspect behavior
  without the proxy in the loop.
- `AUTH_MODE=oidc`: the OIDC handshake is failing. Check
  `docker compose logs jupyterhub` for the redirect URL and the
  upstream response. Most common causes are an `OAUTH_CALLBACK_URL`
  that does not exactly match the redirect URI registered with the
  provider, an `OIDC_ISSUER` that returns 404 on `/authorize`, or
  wrong client credentials.

**`Error response from daemon: No such image: vle-notebook:latest`** —
the single-user image wasn't built. Run
`docker compose --env-file ../.env --profile build-only build` from
`deploy/compose/`.

**User containers never get culled** — check the `idle-culler` service
in `docker compose logs jupyterhub`. Confirm `IDLE_TIMEOUT_MINUTES` is
set.

**Bundled notebooks or `components.db` not appearing in `~/work/`** —
the first-start hook at
`/usr/local/bin/before-notebook.d/10-seed-user-home.sh` only copies
from `/opt/vle/` when the target paths do not already exist. If you
expect a fresh copy after rebuilding the image, delete the user's
volume (`docker volume rm vle_vle-user-<email>`) and re-spawn.

**DockerSpawner can't reach the hub** — the hub and spawned containers
must share a Docker network. The compose file attaches the hub to
`$TRAEFIK_NETWORK` and DockerSpawner uses the same network name; if
you change one, change both.

**Resource limits look ignored** — `CONTAINER_MEM_LIMIT` and
`CONTAINER_CPU_LIMIT` apply to *spawned* user containers, not the hub
itself. Inspect a live user container with `docker inspect` to confirm.

## Sources

Pedagogical references on Jupyter for teaching numerical sciences and
thermodynamics:

- [Teaching and Learning with Jupyter](https://jupyter4edu.github.io/jupyter-edu-book/) — Barba et al. (2019), free open book.
- [Why we use Jupyter notebooks](https://jupyter4edu.github.io/jupyter-edu-book/why-we-use-jupyter-notebooks.html) — chapter 2 of the above.
- [Using Jupyter Notebooks to Climb Bloom's Taxonomy in Thermodynamics](https://peer.asee.org/work-in-progress-using-jupyter-notebooks-to-climb-bloom-s-taxonomy-in-thermodynamics) — ASEE PEER, Weber et al.
- [PyTherm applied thermodynamics](https://chemistry.coe.edu/piper/posts/pytherm-applied-thermo/) — Coe College PIPER.
- [MatSciEd/Thermodynamics](https://github.com/MatSciEd/Thermodynamics) — open collection of materials-thermodynamics teaching notebooks.
- [Jupyter Notebooks for the study of advanced topics in Fluid Mechanics](https://onlinelibrary.wiley.com/doi/full/10.1002/cae.22619) — Castilla, *Computer Applications in Engineering Education*, 2023.
