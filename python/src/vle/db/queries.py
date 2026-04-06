"""Database query functions for component lookup, insertion, and search.

All returned values use canonical units:
- Temperature: **K**
- Pressure: **kPa** (absolute)
- Molar volume: **cm3/mol**
- Heat capacity coefficients: for Cp in **kJ/(kmol*K)** with T in **K**
"""

import sqlite3
from typing import Optional

from vle.db.connection import get_connection
from vle.db.models import (
    ComponentRecord,
    KijRecord,
    ActivityParamRecord,
    ExperimentalVlePoint,
)


def _row_to_component(row: sqlite3.Row) -> ComponentRecord:
    """Convert a database row to a ComponentRecord."""
    return ComponentRecord(**{k: row[k] for k in row.keys() if k in {
        f.name for f in ComponentRecord.__dataclass_fields__.values()
    }})


def get_component(name: str, conn: Optional[sqlite3.Connection] = None) -> Optional[ComponentRecord]:
    """Look up a component by name.

    Args:
        name: Component name (case-insensitive).
        conn: Optional database connection (opens new one if None).

    Returns:
        ComponentRecord or None if not found.
    """
    close = conn is None
    if conn is None:
        conn = get_connection(readonly=True)
    try:
        row = conn.execute(
            "SELECT * FROM components WHERE LOWER(name) = LOWER(?)", (name,)
        ).fetchone()
        return _row_to_component(row) if row else None
    finally:
        if close:
            conn.close()


def get_component_by_cas(cas: str, conn: Optional[sqlite3.Connection] = None) -> Optional[ComponentRecord]:
    """Look up a component by CAS number.

    Args:
        cas: CAS registry number (e.g., "74-82-8").
        conn: Optional database connection.

    Returns:
        ComponentRecord or None if not found.
    """
    close = conn is None
    if conn is None:
        conn = get_connection(readonly=True)
    try:
        row = conn.execute(
            "SELECT * FROM components WHERE cas_number = ?", (cas,)
        ).fetchone()
        return _row_to_component(row) if row else None
    finally:
        if close:
            conn.close()


def list_components(conn: Optional[sqlite3.Connection] = None) -> list[ComponentRecord]:
    """List all components in the database, ordered by name.

    Returns:
        List of ComponentRecord objects.
    """
    close = conn is None
    if conn is None:
        conn = get_connection(readonly=True)
    try:
        rows = conn.execute("SELECT * FROM components ORDER BY name").fetchall()
        return [_row_to_component(r) for r in rows]
    finally:
        if close:
            conn.close()


def search_components(query: str, conn: Optional[sqlite3.Connection] = None) -> list[ComponentRecord]:
    """Search components by name, formula, or CAS number.

    Args:
        query: Search string (partial match, case-insensitive).

    Returns:
        List of matching ComponentRecord objects.
    """
    close = conn is None
    if conn is None:
        conn = get_connection(readonly=True)
    try:
        pattern = f"%{query}%"
        rows = conn.execute(
            """SELECT * FROM components
               WHERE LOWER(name) LIKE LOWER(?)
                  OR LOWER(formula) LIKE LOWER(?)
                  OR cas_number LIKE ?
               ORDER BY name""",
            (pattern, pattern, pattern),
        ).fetchall()
        return [_row_to_component(r) for r in rows]
    finally:
        if close:
            conn.close()


def upsert_component(comp: ComponentRecord, conn: Optional[sqlite3.Connection] = None) -> int:
    """Insert or update a component record.

    Args:
        comp: ComponentRecord with at least ``name`` set.
        conn: Optional database connection.

    Returns:
        The component's database ID.
    """
    close = conn is None
    if conn is None:
        conn = get_connection()
    try:
        cursor = conn.execute(
            """INSERT INTO components (name, formula, cas_number, mw,
                   tc, pc, w, zc, vc, tb,
                   antoine_a1, antoine_a2, antoine_a3, antoine_t_min, antoine_t_max,
                   cp_a, cp_b, cp_c, cp_d, cp_e,
                   zra, w_srk, dipole_moment, delta, vl_at_tb, prsv_k1,
                   source, notes)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(name) DO UPDATE SET
                   formula=excluded.formula, cas_number=excluded.cas_number, mw=excluded.mw,
                   tc=excluded.tc, pc=excluded.pc, w=excluded.w, zc=excluded.zc, vc=excluded.vc, tb=excluded.tb,
                   antoine_a1=excluded.antoine_a1, antoine_a2=excluded.antoine_a2, antoine_a3=excluded.antoine_a3,
                   antoine_t_min=excluded.antoine_t_min, antoine_t_max=excluded.antoine_t_max,
                   cp_a=excluded.cp_a, cp_b=excluded.cp_b, cp_c=excluded.cp_c, cp_d=excluded.cp_d, cp_e=excluded.cp_e,
                   zra=excluded.zra, w_srk=excluded.w_srk,
                   dipole_moment=excluded.dipole_moment, delta=excluded.delta,
                   vl_at_tb=excluded.vl_at_tb, prsv_k1=excluded.prsv_k1,
                   source=excluded.source, notes=excluded.notes""",
            (comp.name, comp.formula, comp.cas_number, comp.mw,
             comp.tc, comp.pc, comp.w, comp.zc, comp.vc, comp.tb,
             comp.antoine_a1, comp.antoine_a2, comp.antoine_a3, comp.antoine_t_min, comp.antoine_t_max,
             comp.cp_a, comp.cp_b, comp.cp_c, comp.cp_d, comp.cp_e,
             comp.zra, comp.w_srk, comp.dipole_moment, comp.delta, comp.vl_at_tb, comp.prsv_k1,
             comp.source, comp.notes),
        )
        conn.commit()
        return cursor.lastrowid
    finally:
        if close:
            conn.close()


def get_kij(
    comp1_name: str,
    comp2_name: str,
    eos_model: str = "PR",
    temperature: Optional[float] = None,
    conn: Optional[sqlite3.Connection] = None,
) -> Optional[KijRecord]:
    """Look up kij for a component pair and EOS model.

    The pair order doesn't matter (comp1/comp2 are normalized internally).

    Args:
        comp1_name: First component name.
        comp2_name: Second component name.
        eos_model: EOS variant (e.g., "PR", "RKS"). Default: "PR".
        temperature: Temperature in **K** for T-dependent kij; None for T-independent.
        conn: Optional database connection.

    Returns:
        KijRecord or None. Returns 0.0 as default if no entry exists? No — returns None.
        The caller should default to 0.0 if None is returned.
    """
    close = conn is None
    if conn is None:
        conn = get_connection(readonly=True)
    try:
        # Normalize pair order
        c1 = conn.execute("SELECT id, name FROM components WHERE LOWER(name) = LOWER(?)", (comp1_name,)).fetchone()
        c2 = conn.execute("SELECT id, name FROM components WHERE LOWER(name) = LOWER(?)", (comp2_name,)).fetchone()
        if not c1 or not c2:
            return None
        id1, id2 = min(c1["id"], c2["id"]), max(c1["id"], c2["id"])

        if temperature is not None:
            row = conn.execute(
                """SELECT * FROM kij_params
                   WHERE comp1_id=? AND comp2_id=? AND eos_model=? AND temperature=?""",
                (id1, id2, eos_model, temperature),
            ).fetchone()
        else:
            row = conn.execute(
                """SELECT * FROM kij_params
                   WHERE comp1_id=? AND comp2_id=? AND eos_model=? AND temperature IS NULL""",
                (id1, id2, eos_model),
            ).fetchone()

        if not row:
            return None
        rec = KijRecord(**{k: row[k] for k in row.keys() if k in {
            f.name for f in KijRecord.__dataclass_fields__.values()
        }})
        rec.comp1_name = c1["name"]
        rec.comp2_name = c2["name"]
        return rec
    finally:
        if close:
            conn.close()


def set_kij(
    comp1_name: str,
    comp2_name: str,
    eos_model: str,
    kij: float,
    temperature: Optional[float] = None,
    source: Optional[str] = None,
    conn: Optional[sqlite3.Connection] = None,
) -> None:
    """Set or update kij for a component pair.

    Args:
        comp1_name: First component name.
        comp2_name: Second component name.
        eos_model: EOS variant (e.g., "PR", "RKS").
        kij: Binary interaction parameter value (dimensionless).
        temperature: Temperature in **K** (None for T-independent).
        source: Data source description.
        conn: Optional database connection.
    """
    close = conn is None
    if conn is None:
        conn = get_connection()
    try:
        c1 = conn.execute("SELECT id FROM components WHERE LOWER(name) = LOWER(?)", (comp1_name,)).fetchone()
        c2 = conn.execute("SELECT id FROM components WHERE LOWER(name) = LOWER(?)", (comp2_name,)).fetchone()
        if not c1 or not c2:
            raise ValueError(f"Component not found: {comp1_name if not c1 else comp2_name}")
        id1, id2 = min(c1["id"], c2["id"]), max(c1["id"], c2["id"])

        conn.execute(
            """INSERT INTO kij_params (comp1_id, comp2_id, eos_model, kij, temperature, source)
               VALUES (?, ?, ?, ?, ?, ?)
               ON CONFLICT(comp1_id, comp2_id, eos_model, temperature) DO UPDATE SET
                   kij=excluded.kij, source=excluded.source""",
            (id1, id2, eos_model, kij, temperature, source),
        )
        conn.commit()
    finally:
        if close:
            conn.close()


def get_activity_params(
    comp1_name: str,
    comp2_name: str,
    model: str,
    temperature: Optional[float] = None,
    conn: Optional[sqlite3.Connection] = None,
) -> Optional[ActivityParamRecord]:
    """Look up activity model parameters for a component pair.

    Args:
        comp1_name: First component name.
        comp2_name: Second component name.
        model: Activity model name ("wilson", "van_laar", "margules").
        temperature: Temperature in **K** (None for T-independent).
        conn: Optional database connection.

    Returns:
        ActivityParamRecord or None if not found.
    """
    close = conn is None
    if conn is None:
        conn = get_connection(readonly=True)
    try:
        c1 = conn.execute("SELECT id, name FROM components WHERE LOWER(name) = LOWER(?)", (comp1_name,)).fetchone()
        c2 = conn.execute("SELECT id, name FROM components WHERE LOWER(name) = LOWER(?)", (comp2_name,)).fetchone()
        if not c1 or not c2:
            return None

        if temperature is not None:
            row = conn.execute(
                """SELECT * FROM activity_params
                   WHERE comp1_id=? AND comp2_id=? AND model=? AND temperature=?""",
                (c1["id"], c2["id"], model, temperature),
            ).fetchone()
        else:
            row = conn.execute(
                """SELECT * FROM activity_params
                   WHERE comp1_id=? AND comp2_id=? AND model=? AND temperature IS NULL""",
                (c1["id"], c2["id"], model),
            ).fetchone()

        if not row:
            return None
        rec = ActivityParamRecord(**{k: row[k] for k in row.keys() if k in {
            f.name for f in ActivityParamRecord.__dataclass_fields__.values()
        }})
        rec.comp1_name = c1["name"]
        rec.comp2_name = c2["name"]
        return rec
    finally:
        if close:
            conn.close()


def set_activity_params(
    comp1_name: str,
    comp2_name: str,
    model: str,
    a12: float,
    a21: float,
    temperature: Optional[float] = None,
    source: Optional[str] = None,
    conn: Optional[sqlite3.Connection] = None,
) -> None:
    """Set or update activity model parameters for a component pair.

    Args:
        comp1_name: First component name.
        comp2_name: Second component name.
        model: Activity model name ("wilson", "van_laar", "margules").
        a12: Forward interaction parameter.
        a21: Reverse interaction parameter.
        temperature: Temperature in **K** (None for T-independent).
        source: Data source description.
        conn: Optional database connection.
    """
    close = conn is None
    if conn is None:
        conn = get_connection()
    try:
        c1 = conn.execute("SELECT id FROM components WHERE LOWER(name) = LOWER(?)", (comp1_name,)).fetchone()
        c2 = conn.execute("SELECT id FROM components WHERE LOWER(name) = LOWER(?)", (comp2_name,)).fetchone()
        if not c1 or not c2:
            raise ValueError(f"Component not found: {comp1_name if not c1 else comp2_name}")

        conn.execute(
            """INSERT INTO activity_params (comp1_id, comp2_id, model, a12, a21, temperature, source)
               VALUES (?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(comp1_id, comp2_id, model, temperature) DO UPDATE SET
                   a12=excluded.a12, a21=excluded.a21, source=excluded.source""",
            (c1["id"], c2["id"], model, a12, a21, temperature, source),
        )
        conn.commit()
    finally:
        if close:
            conn.close()


def get_experimental_vle(
    system_name: str,
    conn: Optional[sqlite3.Connection] = None,
) -> list[ExperimentalVlePoint]:
    """Get all experimental VLE data points for a system.

    Args:
        system_name: System identifier (e.g., "CO2/n-butane").
        conn: Optional database connection.

    Returns:
        List of ExperimentalVlePoint records, ordered by x1.
    """
    close = conn is None
    if conn is None:
        conn = get_connection(readonly=True)
    try:
        rows = conn.execute(
            """SELECT * FROM experimental_vle
               WHERE system_name = ?
               ORDER BY x1""",
            (system_name,),
        ).fetchall()
        return [
            ExperimentalVlePoint(**{k: row[k] for k in row.keys() if k in {
                f.name for f in ExperimentalVlePoint.__dataclass_fields__.values()
            }})
            for row in rows
        ]
    finally:
        if close:
            conn.close()
