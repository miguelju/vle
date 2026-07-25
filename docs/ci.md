# CI/CD Overview

**Everything runs on GitHub-hosted runners except one job.** This repo is
public, and standard GitHub-hosted runners are free and unlimited on public
repositories — macOS and arm64 included (the billing split is
standard-vs-larger runners, not Linux-vs-macOS). So the wheel matrix, lint and
tests have no reason to touch lab hardware.

The single exception is `bench-rust`, the informational criterion benchmark,
which stays on a self-hosted Linux runner because benchmark deltas need a
quiet dedicated machine — measured drift on a dedicated idle box is already
±3–5 % between sessions, and a shared hosted VM would swamp the signal.

**This changed in v0.12.0.** The linux-x86_64 and macOS-arm64 wheel builds and
`test-rust` used to run self-hosted. That bought nothing and cost two real
things:

1. **Serialization.** The self-hosted Linux runner is ephemeral (one job per
   container), so a tag push racing a `main` push put the release third in
   line behind CI's bench job — ~8 minutes of dead wait on the v0.12.0
   release.
2. **Blast radius.** cibuildwheel needs Docker for manylinux, so that runner
   mounted `/var/run/docker.sock` — root-equivalent on the LXC host, on a
   public repo, with a fork-PR `if:` guard as the only thing in the way.

Wheel compatibility is unchanged: the manylinux image and the macOS
deployment target are both pinned in `python/pyproject.toml`, so the tags
(`manylinux_2_28_*`, `macosx_11_0_arm64`) do not depend on the runner.

If you are a maintainer setting up the lab side, read
[`runners/linux-setup.md`](runners/linux-setup.md) for the bench runner.
[`runners/macos-setup.md`](runners/macos-setup.md) is retained for reference
only — no job targets macOS self-hosted any more, so `vle-mac-01` can be
decommissioned.

If you are a contributor opening a PR, read on.

## Workflow files

```
.github/workflows/
├── _build.yml      # reusable wheel matrix; called by ci.yml and release.yml
├── ci.yml          # push/PR/dispatch: lint + test + wheel artifacts
└── release.yml     # tag v*: publish PyPI + crates.io + GitHub Release
```

`_build.yml` is a "reusable workflow" (called via `workflow_call`). The
leading underscore is a convention indicating it isn't meant to be run
directly.

## Runners

| Job                   | Runner                                | Ephemerality                                  |
|-----------------------|---------------------------------------|------------------------------------------------|
| `lint-rust`           | `ubuntu-latest` (GitHub-hosted)       | Fresh VM per run                                |
| `test-rust`           | `ubuntu-latest` (GitHub-hosted)       | Fresh VM per run                                |
| `bench-rust`          | `[self-hosted, linux, vle-runner]`    | **Ephemeral Docker container** (clean per job)  |
| `build` linux/x86_64  | `ubuntu-latest` (GitHub-hosted)       | Fresh VM per run                                |
| `build` linux/aarch64 | `ubuntu-24.04-arm` (GitHub-hosted)    | Fresh VM per run                                |
| `build` macOS arm64   | `macos-14` (GitHub-hosted)            | Fresh VM per run                                |
| `build` windows       | `windows-latest` (GitHub-hosted)      | Fresh VM per run                                |
| `build-sdist`         | `ubuntu-latest`                       | Fresh VM per run                                |
| All publish jobs      | `ubuntu-latest`                       | Fresh VM per run                                |

The one self-hosted runner is tagged `self-hosted, Linux, X64, vle-runner`
— the trailing repo-specific label is what a workflow pins to, so a second
Linux runner added to this repo later cannot silently steal the job. It is
recreated per job by a containerized runner image (see
`runners/linux-setup.md`). Because no job on it needs Docker any more, its
`/var/run/docker.sock` mount should be removed — that mount existed only for
cibuildwheel's manylinux container, which now runs hosted.

## Fork-PR safety

Self-hosted runners share lab hardware. A malicious PR opened from a
fork could otherwise execute arbitrary code on that hardware. The
workflows protect against this two ways:

1. **`bench-rust` runs only on `push` / `workflow_dispatch`**:
   ```yaml
   if: github.event_name == 'push' || github.event_name == 'workflow_dispatch'
   ```
   That *is* the fork-PR guard, not a separate one: a fork's push never
   triggers this repo's workflows, and a `pull_request` event fails both
   arms. Since every other job is now hosted, this is the only guard the
   repo needs — and with the wheel builds hosted, fork contributors get full
   build + test feedback on their PRs instead of skipped jobs.

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

The pipeline **publishes only — it deploys nowhere.** The teaching hub is
refreshed separately from a private operator repository (a gated deploy
workflow); see [PUBLISHING.md](../PUBLISHING.md) → *Deploying the teaching hub*.

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
  The crates.io token lives in the 1Password vault `vle-thermo-ci` and is
  loaded at workflow runtime. (Deploy SSH keys / host names are no longer
  needed — deployment moved to a separate private operator repository.)
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
