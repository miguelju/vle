"""SQLite connection management for the VLE component database.

The database file defaults to ``<project_root>/data/components.db`` in a dev
checkout, but it is NOT checked into git — run ``vle-db init`` to create it
from the schema shipped with the package.

The schema itself (``schema.sql``) and the Chapter IV seed
(``seed_chapter4.sql``) live inside the installed wheel at
:mod:`vle.db.sql`, so the package is self-contained wherever it is installed.
Both locations can be overridden with environment variables for non-standard
deployments:

- ``VLE_DB_PATH`` — absolute path to the SQLite file to create/open.
- ``VLE_SCHEMA_PATH`` — absolute path to a custom ``schema.sql``.
- ``VLE_SEED_DIR`` — directory containing seed ``*.sql`` files
  (see :mod:`vle.db.seed`).
"""

import os
import sqlite3
from importlib import resources
from pathlib import Path
from typing import Optional

# Default DB location for developer checkouts: <project_root>/data/components.db.
# When the package is installed as a wheel this path will simply not exist
# until the user runs `vle-db init`, which honors VLE_DB_PATH.
_PROJECT_ROOT = Path(__file__).resolve().parents[4]
_DEFAULT_DB_PATH = _PROJECT_ROOT / "data" / "components.db"

# Allow override via set_db_path() for tests.
_db_path_override: Optional[Path] = None


def get_db_path() -> Path:
    """Return the path to the SQLite database file.

    Resolution order: ``set_db_path()`` override, ``VLE_DB_PATH`` env var,
    then the dev-checkout default at ``<repo>/data/components.db``.
    """
    if _db_path_override is not None:
        return _db_path_override
    env_path = os.environ.get("VLE_DB_PATH")
    if env_path:
        return Path(env_path)
    return _DEFAULT_DB_PATH


def set_db_path(path: Path) -> None:
    """Override the default database path (used by tests)."""
    global _db_path_override
    _db_path_override = Path(path)


def _read_schema_sql() -> str:
    """Return the contents of ``schema.sql``.

    Prefers the ``VLE_SCHEMA_PATH`` env var when set, otherwise falls back to
    the copy bundled inside the wheel at :mod:`vle.db.sql`.
    """
    env_path = os.environ.get("VLE_SCHEMA_PATH")
    if env_path:
        schema_path = Path(env_path)
        if not schema_path.exists():
            raise FileNotFoundError(
                f"VLE_SCHEMA_PATH points to {schema_path} but that file "
                "does not exist."
            )
        return schema_path.read_text(encoding="utf-8")
    return resources.files("vle.db.sql").joinpath("schema.sql").read_text(encoding="utf-8")


def get_connection(readonly: bool = False) -> sqlite3.Connection:
    """Open a connection to the component database.

    Args:
        readonly: If True, open in read-only mode (URI-based).

    Returns:
        A ``sqlite3.Connection`` with row_factory set to ``sqlite3.Row``.

    Raises:
        FileNotFoundError: If the database file does not exist.
            Run ``vle-db init`` to create it.
    """
    db_path = get_db_path()
    if not db_path.exists():
        raise FileNotFoundError(
            f"Database not found at {db_path}. "
            "Run 'vle-db init' to create it from schema."
        )
    if readonly:
        uri = f"file:{db_path}?mode=ro"
        conn = sqlite3.connect(uri, uri=True)
    else:
        conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys=ON")
    return conn


def init_db() -> Path:
    """Create the database from the bundled schema.

    The schema is loaded from the package resource ``vle.db.sql/schema.sql``
    (or from ``VLE_SCHEMA_PATH`` if that env var is set). Tables use
    ``IF NOT EXISTS``, so re-running against an existing DB is a no-op.

    Returns:
        Path to the created database file.
    """
    db_path = get_db_path()
    db_path.parent.mkdir(parents=True, exist_ok=True)

    schema_sql = _read_schema_sql()
    conn = sqlite3.connect(str(db_path))
    conn.executescript(schema_sql)
    conn.close()
    return db_path


def seed_from_sql(sql_path: Path) -> int:
    """Execute a SQL seed file against the database.

    Args:
        sql_path: Path to a ``.sql`` file containing INSERT statements.

    Returns:
        Number of components in the DB after the seed runs (approximate —
        SQLite does not distinguish IGNORE'd rows).

    Raises:
        FileNotFoundError: If the SQL file or database does not exist.
    """
    if not sql_path.exists():
        raise FileNotFoundError(f"Seed file not found at {sql_path}")

    conn = get_connection()
    try:
        seed_sql = sql_path.read_text(encoding="utf-8")
        conn.executescript(seed_sql)
        conn.commit()
        count = conn.execute("SELECT COUNT(*) FROM components").fetchone()[0]
        return count
    finally:
        conn.close()
