# CI/CD Overview

This project uses a **hybrid three-runner-type** CI architecture: some
jobs run on self-hosted hardware (a Proxmox LXC for Linux x86_64 and a
Mac mini M1 for macOS arm64), and the rest run on GitHub-hosted runners
(Linux arm64, Windows, all publish jobs). The split balances cost,
latency, and the need to keep secrets off lab hardware.

If you are a maintainer setting up the lab side, read
[`runners/linux-setup.md`](runners/linux-setup.md) and
[`runners/macos-setup.md`](runners/macos-setup.md) for one-shot
provisioning of each self-hosted runner.

If you are a contributor opening a PR, read on.

## Workflow files

```
.github/workflows/
├── _build.yml      # reusable wheel matrix; called by ci.yml and release.yml
├── ci.yml          # push/PR/dispatch: lint + test + wheel artifacts
└── release.yml     # tag v*: publish PyPI + crates.io + GitHub Release + sandbox redeploy
```

`_build.yml` is a "reusable workflow" (called via `workflow_call`). The
leading underscore is a convention indicating it isn't meant to be run
directly.

## Runners

| Job                   | Runner                                | Ephemerality                                  |
|-----------------------|---------------------------------------|------------------------------------------------|
| `lint-rust`           | `ubuntu-latest` (GitHub-hosted)       | Fresh VM per run                                |
| `test-rust`           | `[self-hosted, linux, x64]`           | **Ephemeral Docker container** (clean per job)  |
| `build` linux/x86_64  | `[self-hosted, linux, x64]`           | **Ephemeral Docker container**                  |
| `build` linux/aarch64 | `ubuntu-24.04-arm` (GitHub-hosted)    | Fresh VM per run                                |
| `build` macOS arm64   | `[self-hosted, macos, arm64]`         | **Persistent** Mac mini M1 (periodic cleanup)   |
| `build` windows       | `windows-latest` (GitHub-hosted)      | Fresh VM per run                                |
| `build-sdist`         | `ubuntu-latest`                       | Fresh VM per run                                |
| All publish/deploy    | `ubuntu-latest`                       | Fresh VM per run                                |

Self-hosted runners are tagged `self-hosted, <os>, <arch>`. Ephemeral
Linux runners are recreated per job by a containerized runner image
(see `runners/linux-setup.md`); persistent macOS state means we rely on
the fork-PR guard below for safety.

## Fork-PR safety

Self-hosted runners share lab hardware. A malicious PR opened from a
fork could otherwise execute arbitrary code on that hardware. The
workflows protect against this two ways:

1. **`if:` guard on every self-hosted job**:
   ```yaml
   if: github.event_name != 'pull_request'
       || github.event.pull_request.head.repo.full_name == github.repository
   ```
   Fork PRs short-circuit before any self-hosted job starts. The job
   shows as "skipped" rather than running on lab hardware.

2. **Manual approval for outside collaborators** (repo Settings →
   Actions → General → "Require approval for all outside collaborators"):
   GitHub holds the entire workflow run until a maintainer reviews and
   approves it.

Fork PRs still get useful CI feedback: the GitHub-hosted jobs
(`lint-rust`, the Linux-arm64 + Windows wheel builds, the sdist build)
run unconditionally and report status back to the PR.

## Wheel matrix

Wheels are built with [cibuildwheel](https://cibuildwheel.readthedocs.io/)
using the configuration in `python/pyproject.toml`'s
`[tool.cibuildwheel]` block.

We ship **abi3 wheels** (PyO3 `abi3-py310` feature in
`engine/Cargo.toml`). One wheel per `(OS, arch)` covers CPython 3.10+
including future Python releases — there is no per-CPython-version
fan-out. The tradeoff is documented in `MODERNIZATION_PLAN.md`'s
M5 phase notes: for VLE's workload, the abi3 boundary overhead is
invisible against the pure-Rust compute kernel, and we trade it for a
4× smaller matrix and zero rebuild on new Python releases.

Resulting wheel filenames look like
`vle_thermo-X.Y.Z-cp310-abi3-manylinux_2_28_x86_64.whl`.

Tests run inside each built wheel via cibuildwheel's
`test-command = "pytest {project}/tests"` setting, so a regression in
the Python ↔ Rust boundary fails CI even when pure-Rust `cargo test`
passes.

## Triggering a release

Releases fire on annotated tags matching `v*`:

```sh
# In a clean working tree on main, with workspace Cargo.toml and
# python/pyproject.toml versions bumped to X.Y.Z:
git tag -a vX.Y.Z -m "Release X.Y.Z"
git push origin vX.Y.Z
```

The `release.yml` workflow then:

1. Re-runs the wheel matrix via `_build.yml` (with `artifact-suffix:
   release`).
2. Publishes to PyPI via Trusted Publishing (OIDC; no token, environment
   `pypi` may require manual approval).
3. Publishes `vle-units` then `vle-thermo` to crates.io (token loaded
   from 1Password).
4. Creates a GitHub Release and attaches all wheels + sdist.
5. SSHes to rocky (plain SSH) and Oracle (Tailscale SSH) and runs
   `/usr/local/bin/vle-deploy vX.Y.Z` on each. The wrapper validates
   the tag name against a regex, checks out the tag, and rebuilds the
   sandbox compose stack.

End-to-end clock from `git push origin vX.Y.Z` to "PyPI install works"
is typically 5–15 min. See [PUBLISHING.md](../PUBLISHING.md) for the
full release procedure.

## Manual config a maintainer must do once

(One-time setup that lives outside the repo.)

- **Settings → Actions → General → Require approval for all outside
  collaborators** — defense in depth on top of the fork-PR `if:` guard.
- **Settings → Environments → Create `pypi`** — optionally add
  yourself as a required reviewer so a stray tag can't ship to PyPI
  without a click.
- **Settings → Secrets and variables → Actions** — add a single secret
  `OP_SERVICE_ACCOUNT_TOKEN` (the 1Password Service Account token).
  Everything else (SSH keys, Tailscale OAuth, crates.io token, host
  names) lives in the 1Password vault `vle-thermo-ci` and is loaded at
  workflow runtime.
- **Self-hosted runners** — register one or more per the
  `runners/*-setup.md` docs.

## Debugging a failed workflow

- **Run logs** live in the Actions tab; each job has a step-by-step
  expandable view. cibuildwheel prints the wheel build for every
  `(OS, arch, Python)` combination in turn.
- **Re-run individual jobs** via the "Re-run failed jobs" button — no
  push needed.
- **Local repro**:
  - Lint: `cargo fmt --all -- --check && cargo clippy --workspace
    --all-targets -- -D warnings`
  - Rust tests: `cargo test --workspace --all-targets`
  - Wheel build: `cd python && pip install cibuildwheel && cibuildwheel
    --output-dir wheelhouse`
- **Self-hosted runner offline**: see `runners/linux-setup.md` or
  `runners/macos-setup.md` for the relevant health-check commands.

## See also

- [`runners/linux-setup.md`](runners/linux-setup.md) — Proxmox LXC + Docker + ephemeral runner image
- [`runners/macos-setup.md`](runners/macos-setup.md) — Mac mini M1 persistent runner setup
- [`../PUBLISHING.md`](../PUBLISHING.md) — release procedure
- [`../MODERNIZATION_PLAN.md`](../MODERNIZATION_PLAN.md) — Milestone 5 phase notes
