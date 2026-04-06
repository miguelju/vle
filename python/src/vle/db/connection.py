"""SQLite connection management for the VLE component database.

The database file is located at ``data/components.db`` relative to the project
root. It is NOT checked into git — run ``vle-db init`` to create it from the
version-controlled schema (``data/schema.sql``).
"""

import os
import sqlite3
from pathlib import Path
from typing import Optional

# Default database location: <project_root>/data/components.db
_PROJECT_ROOT = Path(__file__).resolve().parents[4]  # python/src/vle/db -> project root
_DEFAULT_DB_PATH = _PROJECT_ROOT / "data" / "components.db"
_SCHEMA_PATH = _PROJECT_ROOT / "data" / "schema.sql"

# Allow override via environment variable
_db_path_override: Optional[Path] = None


def get_db_path() -> Path:
    """Return the path to the SQLite database file.

    Returns:
        Path to ``data/components.db`` (or override if set).
    """
    if _db_path_override is not None:
        return _db_path_override
    env_path = os.environ.get("VLE_DB_PATH")
    if env_path:
        return Path(env_path)
    return _DEFAULT_DB_PATH


def set_db_path(path: Path) -> None:
    """Override the default database path.

    Args:
        path: Path to the SQLite database file.
    """
    global _db_path_override
    _db_path_override = Path(path)


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
    """Create the database from the schema file.

    Reads ``data/schema.sql`` and executes it to create all tables.
    If the database already exists, tables are created only if they
    don't already exist (uses IF NOT EXISTS).

    Returns:
        Path to the created database file.

    Raises:
        FileNotFoundError: If ``data/schema.sql`` is not found.
    """
    if not _SCHEMA_PATH.exists():
        raise FileNotFoundError(f"Schema file not found at {_SCHEMA_PATH}")

    db_path = get_db_path()
    db_path.parent.mkdir(parents=True, exist_ok=True)

    schema_sql = _SCHEMA_PATH.read_text(encoding="utf-8")
    conn = sqlite3.connect(str(db_path))
    conn.executescript(schema_sql)
    conn.close()
    return db_path


def seed_from_sql(sql_path: Path) -> int:
    """Execute a SQL seed file against the database.

    Args:
        sql_path: Path to a ``.sql`` file containing INSERT statements.

    Returns:
        Number of rows affected (approximate — SQLite doesn't track
        INSERT OR IGNORE rows precisely).

    Raises:
        FileNotFoundError: If the SQL file or database does not exist.
    """
    if not sql_path.exists():
        raise FileNotFoundError(f"Seed file not found at {sql_path}")

    conn = get_connection()
    try:
        seed_sql = sql_path.read_text(encoding="utf-8")
        cursor = conn.executescript(seed_sql)
        conn.commit()
        # Count components as a rough measure
        count = conn.execute("SELECT COUNT(*) FROM components").fetchone()[0]
        return count
    finally:
        conn.close()
