# macOS self-hosted runner (Mac mini M1, persistent)

This is the recipe for the **macOS arm64 self-hosted GitHub Actions
runner** that builds the macOS wheel in our CI matrix. Unlike the
Linux runner (which is ephemeral inside Docker), macOS doesn't
containerize well, so this runner is **persistent**: a single
launchd-managed process that lives for months at a time with periodic
cleanup.

Designed for the lab's Mac mini M1; tagged `self-hosted, macos, arm64`
so the workflows in `.github/workflows/` target it correctly.

## Persistent ≠ unsafe

The persistent model has one downside: a malicious job could in
principle leave state behind (cached files, modified env). We mitigate
this two ways:

1. **Fork-PR guard** (`if:` in `_build.yml`) — fork PRs never run on
   this runner.
2. **GitHub's "Require approval for outside collaborators" setting** —
   any PR from a non-collaborator is held for manual approval before
   any workflow runs.

For a single-maintainer project these are sufficient. If you ever
onboard collaborators with push access, revisit.

## Prerequisites

- A Mac mini M1 (or any Apple silicon Mac) running a recent macOS
  (Ventura 13.0+ recommended; Sonoma / Sequoia tested in CI).
- An admin user account on the Mac for the runner. We use `vle-runner`;
  pick what suits you. Avoid running the runner as your personal user
  account.
- Outbound HTTPS to `github.com`, PyPI, crates.io, GHCR (if your
  workflows touch any). Inbound is not needed.

## Step 1 — Install Xcode Command Line Tools

The Rust toolchain and cibuildwheel need this:

```sh
xcode-select --install
```

Accept the GUI prompt. Verify with `xcrun --version`.

## Step 2 — Install Homebrew (recommended)

Homebrew is convenient for the rest of the toolchain:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Add Homebrew to your shell init per its post-install instructions.

## Step 3 — Install Rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source $HOME/.cargo/env
```

Pin the MSRV per the repo's `rust-toolchain.toml` if it exists; the
workspace currently sets `rust-version = "1.85"` in `Cargo.toml`.

## Step 4 — Install Python 3.10 through 3.13

cibuildwheel needs each CPython explicitly because abi3 wheels are
*tested* against multiple interpreters even though only one wheel is
*built*. The simplest path is the **python.org installers** — drop
each into `/Library/Frameworks/`:

```
https://www.python.org/downloads/macos/
# Grab the "macOS 64-bit universal2 installer" for each of
# 3.10.x, 3.11.x, 3.12.x, 3.13.x
```

Each installer adds `python3.X` to `/usr/local/bin/`. Confirm:

```sh
for v in 3.10 3.11 3.12 3.13; do python$v --version; done
```

Alternative: use [`uv`](https://github.com/astral-sh/uv) to manage
Pythons:

```sh
brew install uv
uv python install 3.10 3.11 3.12 3.13
```

Either approach works; cibuildwheel discovers all interpreters on PATH.

## Step 5 — Install maturin

```sh
pip3 install "maturin>=1.5,<2"
```

(Or `uv pip install maturin` if you're on uv.)

## Step 6 — Register the runner

In the repo, go to **Settings → Actions → Runners → New self-hosted
runner**. Select **macOS** → **arm64**. The page shows the download
URL and a one-time registration token. Then on the Mac mini:

```sh
mkdir -p ~/actions-runner && cd ~/actions-runner
# Download — exact filename varies; copy from the GitHub setup page
curl -o actions-runner-osx-arm64.tar.gz \
    -L https://github.com/actions/runner/releases/download/vX.Y.Z/actions-runner-osx-arm64-X.Y.Z.tar.gz
tar xzf actions-runner-osx-arm64.tar.gz

# Configure — the URL and TOKEN come from the GitHub setup page.
./config.sh \
    --url https://github.com/miguelju/vle \
    --token <REGISTRATION_TOKEN> \
    --name vle-mac-01 \
    --labels self-hosted,macos,arm64 \
    --unattended

# Install as a launchd service so it survives reboots.
./svc.sh install
./svc.sh start
```

The runner now lives in the menu bar (sort of — it's a background
launchd job). `./svc.sh status` shows whether the service is up.

## Step 7 — Verify

1. **GitHub side**: Settings → Actions → Runners should list
   `vle-mac-01` as Idle with the right labels.
2. **Trigger a test workflow**: push a branch with a small commit; the
   macOS wheel build job should land on this runner. Watch the "Set up
   job" step in the Actions UI for the runner name.
3. **First wheel build is slow** because the Rust toolchain needs to
   compile dependencies from source for the first time. Subsequent
   builds are fast thanks to the cargo cache.

## Step 8 — Periodic maintenance (monthly is fine)

Persistent runners accumulate state. A monthly cron pass keeps things
tidy:

```sh
# Cargo registry cleanup (keeps installs but trims downloads).
cargo install cargo-cache && cargo cache -a

# Homebrew cache.
brew cleanup -s

# Old pip download cache.
rm -rf ~/Library/Caches/pip ~/Library/Caches/pypa

# Runner work tree (gets large with wheelhouse artifacts).
rm -rf ~/actions-runner/_work/_temp/* ~/actions-runner/_work/_actions/*
```

Schedule with `crontab -e`:

```cron
0 3 1 * *  cd ~/maintenance && ./cleanup.sh >> ~/maintenance/cleanup.log 2>&1
```

(Save the snippet above as `~/maintenance/cleanup.sh`.)

## Troubleshooting

- **Runner offline in GitHub**: SSH to the Mac mini. `./svc.sh status`
  in `~/actions-runner` should show "active". If not,
  `./svc.sh start`; if that fails, check `~/actions-runner/_diag/
  Runner_*.log`.
- **Wheel build fails with "linker `cc` not found"**: Xcode CLT not
  installed or got nuked by a macOS upgrade. Re-run `xcode-select
  --install`.
- **Wheel build fails on a specific Python version**: that interpreter
  isn't on PATH. Verify with `python3.X --version`. Reinstall via
  python.org or `uv python install 3.X`.
- **Reboots**: launchd brings the runner back automatically. If you
  changed power settings to sleep the Mac, the runner will be offline
  while asleep. Recommended: System Settings → Energy → Prevent
  sleeping when the display is off.

## Removing the runner

If you ever decommission this Mac:

```sh
cd ~/actions-runner
./svc.sh stop && ./svc.sh uninstall
./config.sh remove --token <REMOVAL_TOKEN_FROM_GITHUB>
```

The removal token comes from the same GitHub Settings → Actions →
Runners page (click the runner → "Remove" → token shown).

## See also

- [`../ci.md`](../ci.md) — overall CI architecture
- [`linux-setup.md`](linux-setup.md) — the ephemeral Linux runner
- [`actions/runner` releases](https://github.com/actions/runner/releases)
