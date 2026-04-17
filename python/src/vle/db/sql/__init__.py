"""Bundled SQL assets for the VLE component database.

The schema (``schema.sql``) and Chapter IV seed (``seed_chapter4.sql``) ship
inside the installed wheel so :mod:`vle.db` works regardless of the repo
layout on disk. Use :func:`importlib.resources.files` to read them.

Paths may also be overridden at runtime via ``VLE_SCHEMA_PATH`` and
``VLE_SEED_DIR`` environment variables — see :mod:`vle.db.connection` and
:mod:`vle.db.seed`.
"""
