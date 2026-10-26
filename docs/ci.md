# CI/CD Overview

**Everything runs on GitHub-hosted runners.** This repo is
public, and standard GitHub-hosted runners are free and unlimited on public
repositories — macOS and arm64 included (the billing split is
standard-vs-larger runners, not Linux-vs-macOS). So the wheel matrix, lint and
tests have no reason to touch lab hardware.

**The bench moved hosted too (v0.18.x).** `bench-rust`, the informational
criterion benchmark, was the last job on a self-hosted Linux runner (the lab's
`vle-runner-01`), on the reasoning that deltas need a quiet dedicated machine.
Two things changed that. First, it was costing real time: it lived in
`ci.yml`, whose single non-cancelling concurrency group made the *next* push's
lint + test wait ~6 minutes behind its ~17-minute run (the v0.18.0 commit).
Second, nothing depends on its numbers: it never gates, and every performance
claim in this repo has to come from a measurement made in the session that
makes it (CLAUDE.md), not from a CI log — the lab box was itself a shared
4-core LXC, not an idle one. So it now lives in its own `bench.yml` on
`ubuntu-latest`, with its own concurrency group that cancels a superseded run.
Read its deltas as a coarse alarm (a 2× regression shows; a 5 % one does not)
and measure locally before claiming either.

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

No job targets lab hardware any more. [`runners/linux-setup.md`](runners/linux-setup.md)
and [`runners/macos-setup.md`](runners/macos-setup.md) are retained for
reference only. `vle-runner-01` was decommissioned on 2026-10-25 (container
removed, registration gone, config archived on the lab host), and
`vle-mac-01` was fully decommissioned the same day (unregistered, its
`vle-runner` macOS account, home directory and sudoers grant removed from the
Mac mini). vle has no self-hosted runners.

If you are a contributor opening a PR, read on.

## Workflow files

```
.github/workflows/
├── _build.yml      # reusable wheel matrix; called by build-smoke.yml and release.yml
├── ci.yml          # push/PR/dispatch: lint + test
├── build-smoke.yml # code-affecting pushes/PRs: wheel matrix as artifacts
├── bench.yml       # push to main/dispatch: informational criterion bench
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
| `bench-rust` (bench.yml) | `ubuntu-latest` (GitHub-hosted)    | Fresh VM per run                                |
| `build` linux/x86_64  | `ubuntu-latest` (GitHub-hosted)       | Fresh VM per run                                |
| `build` linux/aarch64 | `ubuntu-24.04-arm` (GitHub-hosted)    | Fresh VM per run                                |
| `build` macOS arm64   | `macos-14` (GitHub-hosted)            | Fresh VM per run                                |
| `build` windows       | `windows-latest` (GitHub-hosted)      | Fresh VM per run                                |
| `build-sdist`         | `ubuntu-latest`                       | Fresh VM per run                                |
| All publish jobs      | `ubuntu-latest`                       | Fresh VM per run                                |

There are no self-hosted runners. (The last one, `vle-runner-01`, tagged
`self-hosted, Linux, X64, vle-runner`, ran only the bench until v0.18.x; see
the top of this page.)

## Fork-PR safety

Every job runs on a fresh GitHub-hosted VM, so a malicious PR from a fork has
no lab hardware to reach. Two protections remain:

1. **The bench runs only on `push` / `workflow_dispatch`** — not because of
   lab hardware any more, but because PR-time benchmarking is noise and a
   17-minute job per PR revision is waste. A fork's push never triggers this
   repo's workflows, so fork PRs never run it.

2. **Manual approval for outside collaborators** (repo Settings →
   Actions → General → "Require approval for all outside collaborators"):
   GitHub holds the entire workflow run until a maintainer reviews and
   approves it.

Fork PRs get full CI feedback: lint, tests and the whole wheel matrix
(every OS and arch, plus the sdist) run on their PRs and report status back.

## Wheel matrix

Wheels are built with [cibuildwheel](https://cibuildwheel.readthedocs.io/)
using the configuration in `python/pyproject.toml`'s
`[tool.cibuildwheel]` block.

We ship **abi3 wheels** (PyO3 `abi3-py311` feature in
`engine/Cargo.toml`). One wheel per `(OS, arch)` covers CPython 3.11+
including future Python releases — there is no per-CPython-version
fan-out. The tradeoff is documented in `MODERNIZATION_PLAN.md`'s
M5 phase notes: for VLE's workload, the abi3 boundary overhead is
invisible against the pure-Rust compute kernel, and we trade it for a
4× smaller matrix and zero rebuild on new Python releases.

Resulting wheel filenames look like
`vle_thermo-X.Y.Z-cp311-abi3-manylinux_2_28_x86_64.whl`. (The floor was
`cp310` through v0.16.0; it moved to 3.11 in v0.17.0 after 3.10 reached end
of life.)

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
- **Settings → Environments → `crates-io` → Environment secrets** — add a
  single secret `OP_SERVICE_ACCOUNT_TOKEN` (the 1Password Service Account
  token). It is deliberately an *environment* secret, not a repository one:
  only the `publish-crates` job declares `environment: crates-io`, so no other
  job (and no PR workflow) can read it, and it is released only after the
  environment's reviewer approves. The crates.io token lives in the
  1Password vault `vle-thermo-ci` and is loaded at workflow runtime.
  (Moved from repo level and rotated on 2026-10-25, after the token was found
  on a lab runner host that no longer does vle work.) (Deploy SSH keys / host names are no longer
  needed — deployment moved to a separate private operator repository.)

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
- **Bench numbers**: `cargo bench -p vle-thermo` locally — the CI bench is
  a coarse alarm, not a measurement.

## See also

- [`runners/linux-setup.md`](runners/linux-setup.md) — Proxmox LXC + Docker + ephemeral runner image (**retired** for vle)
- [`runners/macos-setup.md`](runners/macos-setup.md) — Mac mini M1 persistent runner setup (**retired**)
- [`../PUBLISHING.md`](../PUBLISHING.md) — release procedure
- [`../MODERNIZATION_PLAN.md`](plans/MODERNIZATION_PLAN.md) — Milestone 5 phase notes
