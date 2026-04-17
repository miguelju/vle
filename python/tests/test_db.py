"""Tests for the ``vle.db`` component property database package.

Each test uses an isolated temporary SQLite file so that these tests never
touch the real ``data/components.db``. The path is injected via the module
helper ``set_db_path``.
"""

from __future__ import annotations

import pytest

from vle.db import connection
from vle.db.connection import init_db, set_db_path
from vle.db.models import ComponentRecord
from vle.db.queries import (
    get_activity_params,
    get_component,
    get_component_by_cas,
    get_experimental_vle,
    get_kij,
    list_components,
    search_components,
    set_activity_params,
    set_kij,
    upsert_component,
)


@pytest.fixture
def tmp_db(tmp_path, monkeypatch):
    """Create a fresh schema in a temp SQLite file for a single test."""
    db_file = tmp_path / "test_components.db"
    set_db_path(db_file)
    # Snapshot the override so the next test starts clean.
    monkeypatch.setattr(connection, "_db_path_override", db_file)
    init_db()
    yield db_file
    # Reset the module-level override so later tests don't leak into this path.
    connection._db_path_override = None


def _methane() -> ComponentRecord:
    return ComponentRecord(
        name="methane",
        formula="CH4",
        cas_number="74-82-8",
        mw=16.04,
        tc=190.56,
        pc=4599.2,
        w=0.0114,
        zc=0.286,
        vc=98.6,
        tb=111.66,
        source="test",
    )


def _ethane() -> ComponentRecord:
    return ComponentRecord(
        name="ethane",
        formula="C2H6",
        cas_number="74-84-0",
        mw=30.07,
        tc=305.32,
        pc=4872.2,
        w=0.0995,
        source="test",
    )


class TestComponentCrud:
    def test_upsert_and_get_by_name(self, tmp_db):
        upsert_component(_methane())
        got = get_component("methane")
        assert got is not None
        assert got.formula == "CH4"
        assert got.tc == pytest.approx(190.56)
        assert got.pc == pytest.approx(4599.2)
        assert got.has_critical_props()

    def test_get_component_is_case_insensitive(self, tmp_db):
        upsert_component(_methane())
        assert get_component("METHANE") is not None
        assert get_component("Methane") is not None

    def test_get_component_missing_returns_none(self, tmp_db):
        assert get_component("unobtainium") is None

    def test_get_by_cas(self, tmp_db):
        upsert_component(_methane())
        got = get_component_by_cas("74-82-8")
        assert got is not None
        assert got.name == "methane"

    def test_upsert_updates_existing_row(self, tmp_db):
        upsert_component(_methane())
        m2 = _methane()
        m2.pc = 4600.0  # pretend a correction
        upsert_component(m2)
        got = get_component("methane")
        assert got.pc == pytest.approx(4600.0)
        # Uniqueness still holds — only one row.
        assert len(list_components()) == 1

    def test_list_components_is_alphabetical(self, tmp_db):
        upsert_component(_ethane())
        upsert_component(_methane())
        names = [c.name for c in list_components()]
        assert names == ["ethane", "methane"]

    def test_search_by_formula_fragment(self, tmp_db):
        upsert_component(_methane())
        upsert_component(_ethane())
        hits = search_components("C2H")
        assert [c.name for c in hits] == ["ethane"]


class TestKij:
    def test_roundtrip_with_temperature(self, tmp_db):
        upsert_component(_methane())
        upsert_component(_ethane())
        set_kij("methane", "ethane", "PR", kij=0.005, temperature=300.0, source="test")
        got = get_kij("methane", "ethane", "PR", temperature=300.0)
        assert got is not None
        assert got.kij == pytest.approx(0.005)
        assert got.temperature == pytest.approx(300.0)

    def test_pair_order_does_not_matter(self, tmp_db):
        # Schema requires comp1_id < comp2_id; the query layer must normalize.
        upsert_component(_methane())
        upsert_component(_ethane())
        set_kij("ethane", "methane", "PR", kij=0.01, temperature=300.0)
        got1 = get_kij("methane", "ethane", "PR", temperature=300.0)
        got2 = get_kij("ethane", "methane", "PR", temperature=300.0)
        assert got1 is not None and got2 is not None
        assert got1.kij == got2.kij == pytest.approx(0.01)

    def test_missing_returns_none(self, tmp_db):
        upsert_component(_methane())
        upsert_component(_ethane())
        assert get_kij("methane", "ethane", "PR", temperature=300.0) is None

    def test_unknown_component_raises(self, tmp_db):
        upsert_component(_methane())
        with pytest.raises(ValueError):
            set_kij("methane", "unobtainium", "PR", kij=0.0)


class TestActivityParams:
    def test_van_laar_roundtrip(self, tmp_db):
        # Exact value from Chapter IV Table 4.5 (methanol/water at 298 K).
        upsert_component(ComponentRecord(name="methanol", formula="CH4O", tc=513.38, pc=8215.85, w=0.5625))
        upsert_component(ComponentRecord(name="water", formula="H2O", tc=647.10, pc=22064.0, w=0.3443))
        set_activity_params(
            "methanol",
            "water",
            "van_laar",
            a12=0.5853,
            a21=0.3458,
            temperature=298.0,
            source="orbey_sandler",
        )
        got = get_activity_params("methanol", "water", "van_laar", temperature=298.0)
        assert got is not None
        assert got.a12 == pytest.approx(0.5853)
        assert got.a21 == pytest.approx(0.3458)


class TestExperimentalVle:
    def test_empty_system_returns_empty_list(self, tmp_db):
        assert get_experimental_vle("nonexistent") == []


class TestChapter4SeedArtifact:
    """Run the real ``seed_chapter4`` seed against a fresh throwaway DB.

    This is the Chapter IV validation test on the packaging side: it proves the
    seed file + schema round-trip cleanly, independent of whatever
    ``data/components.db`` currently contains on the developer machine.
    """

    def test_seed_produces_all_15_chapter4_compounds(self, tmp_db):
        from vle.db.seed import seed_chapter4

        count = seed_chapter4()
        assert count == 15

        expected = [
            "methane", "ethane", "propane", "n-butane", "n-pentane",
            "carbon dioxide", "hydrogen sulfide",
            "benzene", "cyclohexane", "methylcyclohexane",
            "n-hexane", "n-heptane",
            "methanol", "water", "2-propanol",
        ]
        for name in expected:
            comp = get_component(name)
            assert comp is not None, f"missing compound: {name}"
            assert comp.has_critical_props(), f"incomplete: {name}"

    def test_seed_includes_van_laar_and_kij(self, tmp_db):
        from vle.db.seed import seed_chapter4

        seed_chapter4()
        vl = get_activity_params("methanol", "water", "van_laar", temperature=298.0)
        assert vl is not None
        assert vl.a12 == pytest.approx(0.5853)
        assert vl.a21 == pytest.approx(0.3458)

        kij = get_kij("carbon dioxide", "n-butane", "PR", temperature=357.57)
        assert kij is not None
        assert kij.kij == pytest.approx(0.1357)

    def test_seed_includes_10_co2_butane_points(self, tmp_db):
        from vle.db.seed import seed_chapter4

        seed_chapter4()
        pts = get_experimental_vle("CO2/n-butane")
        assert len(pts) == 10
        # Points must be sorted by x1 ascending (per the query).
        xs = [p.x1 for p in pts]
        assert xs == sorted(xs)
