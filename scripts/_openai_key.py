"""Fetch the OpenAI API key from 1Password, at call time.

Shared by [`second_opinion.py`](second_opinion.py) (text) and
[`generate_image.py`](generate_image.py) (images) so the credential path exists
in exactly one place. Stdlib only — nothing is installed into the ``vle``
environment, and the key is never written to disk, echoed, or put in an
environment variable that outlives the subprocess that needs it.

Not a package: `scripts/` has no ``__init__.py``, so importers add this
directory to ``sys.path`` themselves::

    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from _openai_key import read_key

That works regardless of the caller's working directory, which matters because
these scripts are documented as being run from the repo root.
"""

from __future__ import annotations

import os
import subprocess
import sys

#: The 1Password item holding the key. One reference, one place to change it.
OP_REF = "op://PhotoGen/photo-platform-openai/credential"


def op_env() -> dict[str, str]:
    """Env for `op`, carrying the PhotoGen service-account token if the keychain has it.

    Injected per-subprocess rather than exported: a global OP_SERVICE_ACCOUNT_TOKEN
    would reroute every other `op` call on the machine through this PhotoGen-only
    identity. The ops_ check means a truncated or wrong keychain entry degrades to
    the desktop app rather than failing hard.
    """
    env = os.environ.copy()
    if "OP_SERVICE_ACCOUNT_TOKEN" not in env:
        token = subprocess.run(
            ["security", "find-generic-password",
             "-a", "photo-platform-mac", "-s", "op-service-account", "-w"],
            capture_output=True,
            text=True,
        ).stdout.strip()
        if token.startswith("ops_"):
            env["OP_SERVICE_ACCOUNT_TOKEN"] = token
    return env


def read_key() -> str:
    """Fetch the API key from 1Password. Never printed, never persisted."""
    try:
        key = subprocess.run(
            ["op", "read", OP_REF],
            env=op_env(),
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
    except FileNotFoundError:
        sys.exit("`op` (the 1Password CLI) is not on PATH — install it or unlock the app")
    except subprocess.CalledProcessError as exc:
        sys.exit(f"1Password read failed: {exc.stderr.strip()}")
    if not key.startswith("sk-"):
        sys.exit("1Password returned something that is not an API key")
    return key
