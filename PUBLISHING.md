# Publishing

How releases work for `vle-thermo`: PyPI + crates.io publishes happen
automatically on `v*` tag pushes via GitHub Actions. End users get the
library via `pip install vle-thermo` or `cargo add vle-thermo`.

> **Note for Milestones 6+**: each milestone that adds Rust functionality
> also adds PyO3 bindings (CLAUDE.md "PyO3 Bindings Rule (M5+)") and
> typically cuts a new release tag. Tag → push → registries is the standard
> milestone-completion workflow.

---

## How a release flows today

```
git tag v0.X.Y && git push origin v0.X.Y
   │
   └── .github/workflows/release.yml fires:
        ├── _build.yml  → cibuildwheel matrix:
        │     • Linux x86_64 (self-hosted ephemeral runner)
        │     • Linux arm64 (ubuntu-24.04-arm GitHub-hosted)
        │     • macOS arm64 (self-hosted Mac mini)
        │     • Windows AMD64 (GitHub-hosted)
        │     • sdist on ubuntu-latest
        │     All wheels are abi3-tagged (cp310-abi3-*), covering
        │     CPython 3.10+ with one wheel per (OS, arch).
        ├── publish-pypi   → PyPI Trusted Publishing (OIDC, no token)
        ├── publish-crates → cargo publish vle-units, vle-steam, then vle-thermo
        │     (token loaded from 1Password via Service Account)
        └── gh-release     → GitHub Release with all wheels + sdist
```

End-to-end clock from `git push origin vX.Y.Z` to "available on PyPI" is
typically 5–15 minutes (the Mac mini wheel build is the long pole).

---

## Cutting a release

1. **Bump the version in all four places — they must match.**
   - `Cargo.toml` (workspace root) → `[workspace.package] version = "X.Y.Z"`
   - `python/pyproject.toml` → `[project] version = "X.Y.Z"`
   - `engine/Cargo.toml` → the **path-dependency version pins**
     `vle-units = { path = "../units", version = "X.Y.Z" }` and
     `vle-steam = { path = "../steam", version = "X.Y.Z", optional = true }`

   The two pins are easy to miss and they are **load-bearing**: `release.yml`
   publishes `vle-units` and `vle-steam` at the bumped workspace version, and
   `cargo publish -p vle-thermo` resolves those siblings *from crates.io*, not
   from the path. A stale `^0.11.0` pin cannot match a freshly published
   `0.12.0` sibling, so the publish fails or resolves the wrong version. Verify
   with `grep -rn '<old version>' Cargo.toml engine/Cargo.toml python/pyproject.toml`
   — it must return nothing.

2. **Update the changelog-ish docs** per `CLAUDE.md` release rules:
   - `ROADMAP.md`, `TODO.md` — check off completed milestones
   - `README.md` — feature list, status notes
   - `docs/plans/MODERNIZATION_PLAN.md` — if architecture changed
   - `docs/plans/engine/PERFORMANCE_PROPOSAL.md` — if a performance track (A–E) decision changed
   - `docs/plans/README.md` — if a plan or audit changed status

3. **Run the pre-push private-data gate** (see `CLAUDE.md`).

4. **Commit and push the version bump.** Land it on `main` first.

5. **Tag and push the tag.**
   ```sh
   git tag -a vX.Y.Z -m "Release X.Y.Z"
   git push origin vX.Y.Z
   ```

6. **Watch the workflow.** Open the Actions tab; the release.yml run
   should be the most recent. If `publish-pypi` is gated behind your
   `pypi` environment with required reviewers, click **Review
   deployments → Approve and deploy** to release the publish step.

7. **Verify** (next section).

---

## Manual fallback (emergency)

The original manual scripts under `deploy/scripts/` are kept as an
emergency path if the automated release flow is broken:

```sh
deploy/scripts/publish-crate.sh  --go   # crates.io
deploy/scripts/publish-pypi.sh   --go   # PyPI (needs $MATURIN_PYPI_TOKEN or ~/.pypirc)
```

These remain dry-run by default; pass `--go` to actually upload. They're
documented for completeness, but the GitHub Actions path is the
canonical release route.

---

## Post-publish verification

```sh
# crates.io
cargo search vle-thermo
cargo search vle-units
cargo search vle-steam
# In a scratch directory:
cargo new --lib scratch && cd scratch && cargo add vle-thermo && cargo build

# PyPI — use a fresh venv
python -m venv /tmp/vle-check && source /tmp/vle-check/bin/activate
pip install vle-thermo==X.Y.Z
python -c "import vle._engine; print(vle._engine.version())"
vle-db --help
deactivate && rm -rf /tmp/vle-check
```

Both registries should show the new version within a couple minutes of
publish.

---

## Rolling back

**crates.io**: `cargo yank --version X.Y.Z vle-thermo` (yank hides the
version from new resolvers but existing lockfiles can still download).
You **cannot** delete a published version. Fix by yanking + publishing
a patch release.

**PyPI**: `twine yank vle-thermo==X.Y.Z --reason "..."` (or file a PyPI
removal request). Same "cannot delete" rule — yank + patch.

---

## One-time setup (already done; documented for reference)

### PyPI Trusted Publishing
At <https://pypi.org/manage/account/publishing/>, add a Trusted Publisher
for project `vle-thermo` pointing at:
- Owner: `miguelju`
- Repository: `vle`
- Workflow: `release.yml`
- Environment: `pypi`

No API token needed thereafter.

### crates.io token (still token-based as of this writing)
1. Create a token at <https://crates.io/settings/tokens> scoped to
   `publish-new` + `publish-update` for **`vle-thermo`, `vle-units`, and
   `vle-steam`** (every crate the workspace publishes — see the new-crate
   gotcha below). Simplest is to leave the crate allowlist **empty** so the
   token covers all crates the account owns; a stale per-crate allowlist is
   exactly what broke the v0.10.0 CI publish.
2. Store it in 1Password under `vle-thermo-ci/crates-io/token`.
3. The release workflow loads it via `1password/load-secrets-action@v2`
   using `OP_SERVICE_ACCOUNT_TOKEN` (the single GitHub secret).

If crates.io graduates Trusted Publishing out of beta, switch to OIDC
and delete the secret.

### Adding a NEW crate to the workspace (the v0.10.0 gotcha)

The **first-ever publish of a new crate name** cannot go through the CI token
if that token has a per-crate allowlist — crates.io returns
`403: this token does not have the required permissions` because the crate
doesn't exist yet to be on the allowlist. This bit the v0.10.0 release when
`vle-steam` shipped: the CI token was scoped to `vle-thermo` + `vle-units`, so
`vle-units` published fine but `vle-steam` (and then `vle-thermo`, which
depends on it) failed. PyPI + the GitHub Release were unaffected.

When adding a new workspace crate, do the **first** publish by hand from a
laptop logged in with a full-permission token, then re-widen the CI token:

```sh
cargo login <token-with-publish-new>          # crates.io/settings/tokens
cargo publish -p <new-crate>                  # creates the crate + you own it
# wait ~30–60s for the index to propagate, then any dependent crate:
cargo publish -p vle-thermo
```

Then **update the CI token** (regenerate unscoped, or add the new crate to its
allowlist) and refresh the 1Password item, so the *next* release publishes the
new crate from `release.yml` automatically. The workflow itself already probes
and publishes in dependency order (vle-units → vle-steam → vle-thermo),
idempotently — only the token scope needs the one-time widening.

### 1Password Service Account
- Vault: `vle-thermo-ci`
- Service Account: read-only on that vault
- The token lives in `GitHub Settings → Secrets → OP_SERVICE_ACCOUNT_TOKEN`

This is the **only** secret in GitHub. The crates.io token is in the
1Password vault and resolved at workflow runtime via `op://vault/item/field`
paths.

See `docs/ci.md` and `docs/runners/` for the full CI architecture.
