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
        ├── publish-crates → cargo publish vle-units, then vle-thermo
        │     (token loaded from 1Password via Service Account)
        └── gh-release     → GitHub Release with all wheels + sdist
```

End-to-end clock from `git push origin vX.Y.Z` to "available on PyPI" is
typically 5–15 minutes (the Mac mini wheel build is the long pole).

---

## Cutting a release

1. **Bump the version in both places — they must match.**
   - `Cargo.toml` (workspace root) → `[workspace.package] version = "X.Y.Z"`
   - `python/pyproject.toml` → `[project] version = "X.Y.Z"`

2. **Update the changelog-ish docs** per `CLAUDE.md` release rules:
   - `ROADMAP.md`, `TODO.md` — check off completed milestones
   - `README.md` — feature list, status notes
   - `MODERNIZATION_PLAN.md` — if architecture changed

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
   `publish-new` + `publish-update` for `vle-thermo` and `vle-units`.
2. Store it in 1Password under `vle-thermo-ci/crates-io/token`.
3. The release workflow loads it via `1password/load-secrets-action@v2`
   using `OP_SERVICE_ACCOUNT_TOKEN` (the single GitHub secret).

If crates.io graduates Trusted Publishing out of beta, switch to OIDC
and delete the secret.

### 1Password Service Account
- Vault: `vle-thermo-ci`
- Service Account: read-only on that vault
- The token lives in `GitHub Settings → Secrets → OP_SERVICE_ACCOUNT_TOKEN`

This is the **only** secret in GitHub. The crates.io token is in the
1Password vault and resolved at workflow runtime via `op://vault/item/field`
paths.

See `docs/ci.md` and `docs/runners/` for the full CI architecture.
