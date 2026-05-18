# Publishing

How to release `vle-thermo` to **crates.io**, **PyPI**, and **GHCR**.

The workflow is deliberately manual — there is no CI/CD yet. Each registry has
a helper script under `deploy/scripts/` that is **dry-run by default**; pass
`--go` to actually publish.

---

## One-time setup

### crates.io
1. Create an API token at <https://crates.io/settings/tokens> (scope: `publish-new` + `publish-update`).
2. `cargo login <TOKEN>` — stores it in `~/.cargo/credentials.toml`.

### PyPI
1. Create a project-scoped API token at <https://pypi.org/manage/account/token/> (scope limited to `vle-thermo` after the first release).
2. Export it for each publish session:
   ```sh
   export MATURIN_PYPI_TOKEN=pypi-...
   ```
   (Or put it in `~/.pypirc` under `[pypi]` if you prefer.)

### GHCR (GitHub Container Registry)
1. Create a **classic** personal access token at <https://github.com/settings/tokens> with scopes `write:packages` + `read:packages` (and `repo` if the repo is private).
2. Log docker in:
   ```sh
   echo "$GITHUB_TOKEN" | docker login ghcr.io -u miguelju --password-stdin
   ```
3. Create a multi-arch buildx builder (once):
   ```sh
   docker buildx create --name vle-builder --use
   docker buildx inspect --bootstrap
   ```

---

## Release checklist

1. **Bump the version in both places — they must match.**
   - `Cargo.toml` (workspace root) → `[workspace.package] version = "X.Y.Z"`
   - `python/pyproject.toml` → `[project] version = "X.Y.Z"`
2. **Update the changelog-ish docs** per `CLAUDE.md` release rules:
   - `ROADMAP.md`, `TODO.md` — check off completed milestones
   - `README.md` — feature list, status notes
   - `MODERNIZATION_PLAN.md` — if architecture changed
3. **Run the pre-push private-data gate** (see `CLAUDE.md`).
4. **Commit the version bump.**
5. **Dry-run each publish path** (order does not matter for dry-runs):
   ```sh
   deploy/scripts/publish-crate.sh
   deploy/scripts/publish-pypi.sh
   deploy/scripts/publish-docker.sh
   ```
   Fix any errors (missing metadata, bad wheel build, Dockerfile issue) before going live.
6. **Go live — in this order:**
   ```sh
   deploy/scripts/publish-crate.sh  --go   # crates.io first (source of truth)
   deploy/scripts/publish-pypi.sh   --go   # then PyPI (depends on the crate, local)
   deploy/scripts/publish-docker.sh --go   # finally GHCR (rebuilds from source)
   ```
   Docker doesn't depend on the published crate — it builds from the local tree — so ordering is soft. The crate-first convention just keeps the "canonical source" registry published first in case anything goes wrong later.
7. **Tag + push the release:**
   ```sh
   git tag -a vX.Y.Z -m "Release X.Y.Z"
   git push origin vX.Y.Z
   ```
8. **Verify** (see next section).

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
python -c "import vle; print(vle.__version__)"
vle-db --help
deactivate && rm -rf /tmp/vle-check

# GHCR
docker pull ghcr.io/miguelju/vle-thermo:X.Y.Z
docker run --rm -p 8888:8888 ghcr.io/miguelju/vle-thermo:X.Y.Z
# → open http://localhost:8888, check that notebooks/ is populated
```

All three registries should show the new version within a minute or two of publish.

---

## Rolling back

**crates.io**: `cargo yank --version X.Y.Z vle-thermo` (yank hides the version
from new resolvers but existing lockfiles can still download). You **cannot**
delete a published version. Fix by yanking + publishing a patch release.

**PyPI**: `pip install twine && twine yank vle-thermo==X.Y.Z --reason "..."` if
supported, else file a PyPI removal request. Same "you cannot delete" rule —
yank + patch.

**GHCR**: delete the tag via
<https://github.com/miguelju/vle/pkgs/container/vle-thermo>. This is reversible
(unlike crates.io/PyPI), so you can safely re-push.

---

## Future: CI-driven releases

When you want to stop publishing manually:

- **crates.io + PyPI**: a `.github/workflows/release.yml` triggered on `v*` tags, using `PyO3/maturin-action` to build a wheel matrix (linux x86_64/aarch64, macOS x86_64/arm64, windows x64, CPython 3.10–3.13), then `cargo publish` the workspace.
- **GHCR**: `docker/build-push-action` with the same multi-arch targets, triggered on the same tags.
- Secrets needed in the GitHub repo settings: `CARGO_REGISTRY_TOKEN`, `PYPI_API_TOKEN`. GHCR uses the built-in `GITHUB_TOKEN`.

The manual scripts in `deploy/scripts/` map 1:1 to the CI steps, so porting is mostly copy-paste.
