"""JupyterHub configuration for the VLE deployment.

Everything environment-specific (resource limits, auth mode, network name,
admin email, notebook image tag) is read from environment variables. The hub
container receives those from ../.env via docker-compose.

Auth modes
----------
``AUTH_MODE`` selects the authenticator:

* ``cloudflare`` (default, production): trust the
  ``Cf-Access-Authenticated-User-Email`` request header that Cloudflare Access
  sets on every request after it has authenticated the user. JupyterHub then
  does no login UI of its own — the upstream proxy is the sole auth gate.
  **This mode is only safe behind a reverse proxy that strips any
  client-supplied copy of that header.**

* ``dummy`` (local development): ``DummyAuthenticator`` accepts any username
  with any password. Never enable this when the hub is reachable from the
  public internet.
"""

from __future__ import annotations

import os
import sys

from dockerspawner import DockerSpawner
from jupyterhub.auth import Authenticator, DummyAuthenticator
from jupyterhub.handlers import BaseHandler
from tornado import web

c = get_config()  # noqa: F821 — provided by JupyterHub at load time


# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------

AUTH_MODE = os.environ.get("AUTH_MODE", "cloudflare").lower()
NOTEBOOK_IMAGE = os.environ.get("NOTEBOOK_IMAGE", "vle-notebook:latest")
DOCKER_NETWORK = os.environ.get("DOCKER_NETWORK_NAME", "web")
MEM_LIMIT = os.environ.get("CONTAINER_MEM_LIMIT", "512m")
CPU_LIMIT = float(os.environ.get("CONTAINER_CPU_LIMIT", "0.5"))
IDLE_TIMEOUT_MIN = int(os.environ.get("IDLE_TIMEOUT_MINUTES", "15"))
ADMIN_EMAIL = os.environ.get("JUPYTERHUB_ADMIN_EMAIL", "").strip()
HUB_CONNECT_NAME = os.environ.get("HUB_CONNECT_NAME", "vle-jupyterhub")

CF_ACCESS_HEADER = "Cf-Access-Authenticated-User-Email"


# ---------------------------------------------------------------------------
# Spawner: per-user Docker container
# ---------------------------------------------------------------------------

c.JupyterHub.spawner_class = DockerSpawner
c.DockerSpawner.image = NOTEBOOK_IMAGE
c.DockerSpawner.network_name = DOCKER_NETWORK
c.DockerSpawner.mem_limit = MEM_LIMIT
c.DockerSpawner.cpu_limit = CPU_LIMIT
c.DockerSpawner.remove = True
c.DockerSpawner.notebook_dir = "/home/jovyan/work"
c.DockerSpawner.volumes = {"vle-user-{username}": "/home/jovyan/work"}
c.DockerSpawner.default_url = "/lab"
# DockerSpawner starts the container on the same network as the hub and
# reaches back to the hub by container name.
c.JupyterHub.hub_ip = "0.0.0.0"
c.JupyterHub.hub_connect_ip = HUB_CONNECT_NAME


# ---------------------------------------------------------------------------
# Authentication
# ---------------------------------------------------------------------------

class CloudflareAccessLoginHandler(BaseHandler):
    """Consume the upstream-proxy-set email header and log the user in."""

    async def get(self) -> None:
        email = self.request.headers.get(CF_ACCESS_HEADER)
        if not email:
            raise web.HTTPError(
                401,
                f"Missing {CF_ACCESS_HEADER} header. "
                "This hub must be reached through Cloudflare Access.",
            )
        user = await self.auth_to_user({"name": email.lower()})
        self.set_login_cookie(user)
        self.redirect(self.get_next_url(user))


class CloudflareAccessAuthenticator(Authenticator):
    """Header-trust authenticator for Cloudflare Access.

    Cloudflare Access authenticates the visitor, signs a JWT, and injects the
    verified email into ``Cf-Access-Authenticated-User-Email``. We trust that
    header *only because* the hub is reachable exclusively through the
    Cloudflare -> Traefik path; Traefik does not forward a client-supplied
    copy of the header.
    """

    auto_login = True

    async def authenticate(self, handler, data=None):  # noqa: ANN001 — JupyterHub API
        email = handler.request.headers.get(CF_ACCESS_HEADER)
        if email:
            return {"name": email.lower()}
        return None

    def get_handlers(self, app):  # noqa: ANN001 — JupyterHub API
        return [("/login", CloudflareAccessLoginHandler)]


if AUTH_MODE == "cloudflare":
    c.JupyterHub.authenticator_class = CloudflareAccessAuthenticator
elif AUTH_MODE == "dummy":
    # Local development only. Accepts any username + any password.
    c.JupyterHub.authenticator_class = DummyAuthenticator
else:
    raise ValueError(
        f"Unknown AUTH_MODE={AUTH_MODE!r}. Expected 'cloudflare' or 'dummy'."
    )

if ADMIN_EMAIL:
    c.Authenticator.admin_users = {ADMIN_EMAIL.lower()}
c.Authenticator.allow_all = True


# ---------------------------------------------------------------------------
# Idle culler
# ---------------------------------------------------------------------------

c.JupyterHub.services = [
    {
        "name": "idle-culler",
        "command": [
            sys.executable,
            "-m",
            "jupyterhub_idle_culler",
            f"--timeout={IDLE_TIMEOUT_MIN * 60}",
            "--cull-every=300",
        ],
    }
]
c.JupyterHub.load_roles = [
    {
        "name": "idle-culler",
        "services": ["idle-culler"],
        "scopes": [
            "list:users",
            "read:users:activity",
            "read:servers",
            "delete:servers",
            "admin:servers",
        ],
    }
]


# ---------------------------------------------------------------------------
# Misc
# ---------------------------------------------------------------------------

c.JupyterHub.cleanup_servers = False  # let the idle culler manage lifecycles
c.ConfigurableHTTPProxy.should_start = True
c.JupyterHub.db_url = "sqlite:////srv/jupyterhub/state/jupyterhub.sqlite"
c.JupyterHub.cookie_secret_file = "/srv/jupyterhub/state/jupyterhub_cookie_secret"
