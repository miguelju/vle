"""Smoke tests for the PyO3 boundary (vle._engine).

These tests prove that the Rust extension built into the wheel:

1. Imports without errors on every CPython we build for.
2. Exposes the milestone-5 surface (version() + the four model enums).
3. Round-trips integer comparisons through the `eq_int` PyO3 attribute.

CI runs this file via cibuildwheel's `test-command = "pytest {project}/tests"`
on every (OS, arch) combination, so a missing or broken binding fails the
release pipeline before it can publish.
"""

import re

import pytest

# Importing `vle._engine` exercises the Rust shared object. If the wheel
# was built without the `python` feature, or the abi3 target mismatched,
# this import fails with an ImportError and pytest aborts before
# the per-test asserts run.
import vle._engine as e  # noqa: E402


SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+(?:[-.+][\w.-]+)?$")


def test_version_is_semver_string() -> None:
    """`version()` returns the Cargo.toml version as a semver-shaped string."""
    v = e.version()
    assert isinstance(v, str), f"expected str, got {type(v).__name__}"
    assert SEMVER_RE.match(v), f"not semver: {v!r}"


def test_module_has_all_milestone5_classes() -> None:
    """All four model-selection enums are reachable from the module."""
    for name in ("CubicEos", "ActivityModel", "MixingRule", "SatPressureModel"):
        assert hasattr(e, name), f"vle._engine.{name} is missing"


@pytest.mark.parametrize(
    "enum_name,variant_name",
    [
        # CubicEos.PR1976 has integer discriminant 0 (Peng-Robinson, the most
        # widely used cubic EOS — picked here because it's stable across
        # any future enum reordering).
        ("CubicEos", "PR1976"),
        # ActivityModel.IdealSolution has integer discriminant 25
        # (see engine/src/activity.rs).
        ("ActivityModel", "IdealSolution"),
        # MixingRule.WongSandler has integer discriminant 26.
        ("MixingRule", "WongSandler"),
        # SatPressureModel.Antoine has integer discriminant 0.
        ("SatPressureModel", "Antoine"),
    ],
)
def test_enum_variant_accessible(enum_name: str, variant_name: str) -> None:
    """Each enum exposes its variants as class attributes."""
    enum_cls = getattr(e, enum_name)
    assert hasattr(enum_cls, variant_name), (
        f"{enum_name}.{variant_name} not exposed"
    )


def test_enum_equality_via_eq_int() -> None:
    """The `eq_int` PyO3 attribute lets enums compare to their integer codes."""
    # PR1976 = 0, RK1949 = 1 (see engine/src/eos.rs). Compare the variant
    # against its discriminant; both directions should work.
    assert e.CubicEos.PR1976 == 0
    assert 0 == e.CubicEos.PR1976
    assert e.CubicEos.PR1976 != 1
    # Cross-variant inequality (PR1976 != RK1949).
    assert e.CubicEos.PR1976 != e.CubicEos.RK1949


def test_enum_variants_distinct() -> None:
    """No two registered enums collide on the same Python object."""
    seen = set()
    for cls_name in ("CubicEos", "ActivityModel", "MixingRule", "SatPressureModel"):
        cls = getattr(e, cls_name)
        # All __module__/__name__ should be unique per class.
        key = (cls.__module__, cls.__name__)
        assert key not in seen, f"duplicate class registration: {key}"
        seen.add(key)
