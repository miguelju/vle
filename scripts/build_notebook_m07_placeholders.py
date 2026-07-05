"""Build the three placeholder notebooks for deferred M7 sub-milestones.

Generates three notebooks that sit on the hub alongside `02_pure_component`
to advertise what's coming in later v0.x releases:

  - notebooks/02b_alpha_zoo.ipynb            ← M7.2, planned for v0.4.0
  - notebooks/02c_three_param_eos.ipynb      ← M7.3, planned for v0.5.0
  - notebooks/02d_advanced_saturation.ipynb  ← M7.4, planned for v0.6.0

Each placeholder:

1. Carries a DRAFT banner at the top making it clear it's not yet
   functional, with a pointer back to the v0.3.0 `02_pure_component`
   notebook for what *does* work today.
2. Sketches the scope (the legacy file + line numbers it'll port from).
3. Contains an executable code cell that **actually calls into the
   engine** and demonstrates the expected `NotImplementedError` /
   panic — so the hub user can see today's behavior and know it's
   intentional, not broken.

The notebooks are executed end-to-end at build time so a `pytest`-style
regression in the deferred-stub panic messages would surface here.

Run:

    $ /Users/migueljackson/miniconda3/envs/vle/bin/python \\
        scripts/build_notebook_m07_placeholders.py
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_DIR = REPO_ROOT / "notebooks"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


# Shared boilerplate ---------------------------------------------------------




def optional_pip_install_cell() -> list[nbf.NotebookNode]:
    return [
        md(
            "## Setup (optional)\n"
            "\n"
            "The cell below is **commented out by default**. Uncomment it "
            "if you want to use the latest `vle-thermo` released on PyPI "
            "instead of whatever version is currently installed in your "
            "kernel — this matters most for *this* notebook because the "
            "feature being demonstrated is **planned for a future release**, "
            "and you may already be on it by the time you read this."
        ),
        code(
            "# Optional: pull the latest vle-thermo from PyPI.\n"
            "# Uncomment if you want the newest released version instead of\n"
            "# whatever is currently in your kernel.\n"
            "# %pip install --upgrade vle-thermo"
        ),
    ]


@dataclass
class Placeholder:
    filename: str          # e.g. "02b_alpha_zoo.ipynb"
    milestone: str         # e.g. "M7.2"
    planned_release: str   # e.g. "v0.4.0"
    title: str             # H1 title
    one_liner: str         # short description for the banner
    motivation: str        # 2-3 sentence motivation paragraph
    scope_table_md: str    # markdown table of planned scope
    demo_cells: list[nbf.NotebookNode]  # the cells that demonstrate today's behavior


def build_placeholder(p: Placeholder) -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + DRAFT banner ------------------------------------------
    cells.append(md(
        f"# {p.title}\n"
        f"\n"
        f"> ⚠️ **DRAFT — {p.milestone} planned for `{p.planned_release}`.** "
        f"The functionality this notebook will exercise has **not yet been "
        f"implemented** in the engine. Today (v0.3.0) the calls in the "
        f"\"behavior today\" section below raise `NotImplementedError` "
        f"or panic with an `M7.x deferred` marker — that's intentional, not "
        f"a bug. For what does work right now, see "
        f"[`02_pure_component.ipynb`](02_pure_component.ipynb).\n"
        f"\n"
        f"{p.one_liner}\n"
    ))

    # ---- Optional pip cell ---------------------------------------------------
    cells.extend(optional_pip_install_cell())

    # ---- Motivation -----------------------------------------------------
    cells.append(md(f"## Why this is coming\n\n{p.motivation}"))

    # ---- Planned scope --------------------------------------------------
    cells.append(md(
        f"## Planned scope ({p.milestone} → `{p.planned_release}`)\n"
        f"\n"
        f"{p.scope_table_md}"
    ))

    # ---- Behavior today (executes against the current engine) ----------
    cells.append(md(
        "## Behavior today (v0.3.0)\n"
        "\n"
        "The cell below calls the planned feature **through the existing "
        f"engine binding**. Today it raises the deferred-stub error so "
        f"you can see exactly what {p.milestone} will eventually replace. "
        f"Once that milestone ships in `{p.planned_release}`, the same cell "
        "will produce real numbers — and this banner will go away."
    ))
    cells.extend(p.demo_cells)

    # ---- References pointer --------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- **ROADMAP.md** — the live status of this sub-milestone. "
        "[`ROADMAP.md`](https://github.com/miguelju/vle/blob/main/ROADMAP.md)\n"
        "- **MODERNIZATION_PLAN.md** — phase-level technical scope. "
        "[`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md)\n"
        "- **v0.3.0 functional notebook** — what works today. "
        "[`02_pure_component.ipynb`](02_pure_component.ipynb)"
    ))

    nb.cells = cells
    nb.metadata = {
        "kernelspec": {
            "display_name": "Python 3 (ipykernel)",
            "language": "python",
            "name": "python3",
        },
        "language_info": {"name": "python"},
    }
    return nb


# Per-placeholder content ----------------------------------------------------


def m7_2_alpha_zoo() -> Placeholder:
    return Placeholder(
        filename="02b_alpha_zoo.ipynb",
        milestone="M7.2",
        planned_release="v0.4.0",
        title="The α-Function Zoo (M7.2, planned v0.4.0)",
        one_liner=(
            "Ports the remaining **15 two-parameter cubic EOS variants** "
            "from `legacy/vb6/clsQbicsPure.cls:1719` — the polar "
            "Mathias-Naumann extensions, the PRSV (Stryjek-Vera) form with "
            "its component-specific K₁ parameter, the OL family that uses "
            "the family-table `h_k` coefficients, Berthelot, "
            "van-der-Waals-Adachi/Valderrama, and friends."
        ),
        motivation=(
            "M7.1 ships the **four α functions** that the Chapter IV "
            "validation cases actually use (PR, RKS, RK, VdW). That's "
            "enough to ship a real flash calculator at v0.5.0, but the "
            "VB6 legacy carries fifteen more α variants — polar "
            "Mathias-Naumann (`m`, `n`, `g` per-component), PRSV with its "
            "K₁ extra parameter, Graboski-Daubert, Lim's modifications, "
            "the OL family that pulls coefficients from the family "
            "`h_k` table. Each has its specific niche (PRSV for water, "
            "RKSGD for hydrocarbon-rich associating systems, …), and "
            "each follows the same Abbott form so the port is mostly "
            "translation + analytical-derivative bookkeeping."
        ),
        scope_table_md=(
            "| Variant | VB6 line | α form sketch | Notes |\n"
            "|---|---|---|---|\n"
            "| Berth1899 | 1730 | 1/T_r | Trivial — temperature-modified VdW |\n"
            "| VdWAda1984 | 1732 | 10^(m·(1−T_r)) | m a polynomial in ω |\n"
            "| RKSGD1978 | 1738 | (1+m(1−√T_r))², new m(ω) | Graboski-Daubert |\n"
            "| RKSL1997 | 1740 | RKS form, cubic-in-ω m(ω) | Lim modification |\n"
            "| RP1978 | 1744 | PR form, cubic m(ω) | Redlich-Prausnitz |\n"
            "| PRL1997 | 1746 | PR form, Lim m(ω) | |\n"
            "| VdWVald1989 | 1748 | 1+(1−T_r)(m+n/T_r) | ω·Z_c-driven m, n |\n"
            "| RKSmn1980 | 1753 | 1+(1−T_r)(m+n/T_r) | Polar, component-specific m, n |\n"
            "| RKSATmn1995 | 1755 | exp(…) | Adachi-Tagawa-MN, three constants m/n/g |\n"
            "| PRATmng1997 | 1757 | exp(…) | PR family of the above |\n"
            "| PRMmn1989 | 1759 | exp(…) | PR Mathias-Massih-Naumann |\n"
            "| PRSV1986 | 1762 | (1+(n+K₁(1+√T_r)(0.7−T_r))(1−√T_r))² | Stryjek-Vera; K₁ component-specific |\n"
            "| VdWOL1998 | 1768 | T_r(1+Σ h_k·…) | OL family — pulls h_k from family-table |\n"
            "| RKOL1998 | 1768 | T_r(1+Σ h_k·…) | Same form, RKS h_k |\n"
            "| PROL1998 | 1768 | T_r(1+Σ h_k·…) | Same form, PR h_k |\n"
            "\n"
            "Every variant requires an analytical dα/dT_r derivation "
            "(per CLAUDE.md *Algorithm Choices*). The component-specific "
            "polar parameters (`m_polar`, `n_polar`, `g_polar`, `prsv_k1`) "
            "are already on the `Component` struct — M7.2 just plugs in "
            "the formula and the corresponding test against a central-"
            "difference oracle."
        ),
        demo_cells=[
            code(
                "from vle._engine import CubicEos, eos_alpha\n"
                "\n"
                "# RKSGD1978 is one of the M7.2-deferred variants.\n"
                "try:\n"
                "    eos_alpha(CubicEos.RKSGD1978, 0.85, 0.252)\n"
                "    print('ERROR: expected a panic but got a value')\n"
                "except BaseException as exc:\n"
                "    print(f'{type(exc).__name__}: {exc}')"
            ),
        ],
    )


def m7_3_three_param() -> Placeholder:
    return Placeholder(
        filename="02c_three_param_eos.ipynb",
        milestone="M7.3",
        planned_release="v0.5.0",
        title="Three-Parameter Cubic EOS + Chao-Seader (M7.3, planned v0.5.0)",
        one_liner=(
            "Ports the **Pascal-origin three-parameter EOS** — "
            "Schmidt-Wenzel, Patel-Teja, Patel-Teja USB — plus the "
            "Chao-Seader liquid fugacity correlation. All from `legacy/"
            "pascal/TERMOII.PAS`, Ref (4) Da Silva & Báez (1989)."
        ),
        motivation=(
            "Two-parameter cubic EOS struggle with polar molecules "
            "(water, alcohols) and very asymmetric mixtures (light gas + "
            "heavy hydrocarbon). The Pascal program shipped three "
            "additions to handle those: Schmidt-Wenzel adds an "
            "acentric-factor-dependent covolume `β`, Patel-Teja uses a "
            "fitted Z_c correlation as the third parameter, and "
            "Chao-Seader provides a semi-empirical liquid-fugacity "
            "correlation with separate coefficient sets for hydrogen, "
            "methane, and \"normal\" compounds. Each addition is small "
            "and well-bounded; together they let v0.5.0 cover the "
            "Chapter IV validation cases that 2-parameter EOS alone "
            "cannot."
        ),
        scope_table_md=(
            "| Component | Source | Notes |\n"
            "|---|---|---|\n"
            "| Schmidt-Wenzel | TERMOII.PAS | Beta(ω) ⇒ k₁(ω), k₂(ω) per-component; special C-parameter mixing |\n"
            "| Patel-Teja (`PatelT`) | TERMOII.PAS | Fitted Z_c, c = OmC·R·T_c/P_c; mole-fraction C-mixing |\n"
            "| Patel-Teja USB (`PatelTUSB`) | TERMOII.PAS | Same form, √B-weighted C-mixing |\n"
            "| Chao-Seader liquid fugacity | TERMOII.PAS | 10+ params; H₂ / methane special cases |\n"
            "\n"
            "All four entries already have enum slots in "
            "`engine/src/eos.rs::CubicEos` (`SchmidtWenzel`, `PatelTeja`, "
            "`PatelTejaUSB`) and `LiquidModel::ChaoSeader`. M7.3 fills in "
            "the actual EOS-specific code paths and a sub-section in the "
            "C-mixing rule module."
        ),
        demo_cells=[
            code(
                "from vle._engine import CubicEos, eos_z_factor\n"
                "\n"
                "# Schmidt-Wenzel is M7.3-deferred — should raise NotImplementedError.\n"
                "try:\n"
                "    eos_z_factor(CubicEos.SchmidtWenzel, 300.0, 1000.0,\n"
                "                 190.564, 4599.0, 0.0115, 'vapor')\n"
                "    print('ERROR: expected NotImplementedError')\n"
                "except NotImplementedError as exc:\n"
                "    print(f'NotImplementedError: {exc}')"
            ),
        ],
    )


def m7_4_advanced_sat() -> Placeholder:
    return Placeholder(
        filename="02d_advanced_saturation.ipynb",
        milestone="M7.4",
        planned_release="v0.6.0",
        title="Advanced Saturation-Pressure Models (M7.4, planned v0.6.0)",
        one_liner=(
            "Ports the four saturation-pressure models that M7.1 left as "
            "stubs — **Riedel**, **Müller**, **RPM** (Reduced Pressure "
            "Model), **polynomial** (DIPPR-style database fits) — plus "
            "the **Maxwell equal-area construction** that derives a "
            "thermodynamically exact P_sat directly from a cubic EOS."
        ),
        motivation=(
            "Antoine is fast and accurate over a limited temperature "
            "range but extrapolates poorly. The remaining four "
            "correlations (Riedel, Müller, RPM, database polynomial) all "
            "extend that range using extra parameters from the property "
            "database. Maxwell goes further: by enforcing equal areas on "
            "the (P, V) loop of a cubic EOS isotherm, it gives a "
            "consistency check that's invaluable when validating new EOS "
            "variants. Together they round out the saturation layer so "
            "the M9 flash calculations can pick the right initialization "
            "for any compound class."
        ),
        scope_table_md=(
            "| Model | Source | Notes |\n"
            "|---|---|---|\n"
            "| Riedel | clsSatPressureSolver.cls + TERMOI.PAS | Corresponding-states with T_c, P_c, ω |\n"
            "| Müller | clsSatPressureSolver.cls + TERMOI.PAS | Alternative reduced-property form |\n"
            "| RPM | clsSatPressureSolver.cls + TERMOI.PAS | Pure corresponding-states (T_r, P_r) |\n"
            "| Polynomial | clsSatPressureSolver.cls | P = exp(A + B/T + C·ln(T) + D·T^E) (DIPPR) |\n"
            "| Maxwell | New code | Equal-area construction on the cubic-EOS isotherm |\n"
            "\n"
            "Each new path needs an **analytical dP_sat/dT** (per "
            "CLAUDE.md *Algorithm Choices*) where one is tractable; for "
            "Maxwell, dP_sat/dT comes from Clausius-Clapeyron with the "
            "EOS-evaluated phase volumes. M7.4 also delivers a "
            "PseudoAntoine helper that converts a fit in any of these "
            "models to an equivalent Antoine triple, useful for legacy "
            "data exchange."
        ),
        demo_cells=[
            code(
                "from vle._engine import SatPressureModel\n"
                "\n"
                "# Only Antoine is wired into the binding layer today; the four\n"
                "# correlations and the Maxwell construction are M7.4-deferred.\n"
                "models = [\n"
                "    ('Antoine',    SatPressureModel.Antoine,    True),\n"
                "    ('Riedel',     SatPressureModel.Riedel,     False),\n"
                "    ('Muller',     SatPressureModel.Muller,     False),\n"
                "    ('RPM',        SatPressureModel.RPM,        False),\n"
                "    ('Polynomial', SatPressureModel.Polynomial, False),\n"
                "    ('Maxwell',    SatPressureModel.Maxwell,    False),\n"
                "]\n"
                "for name, _, ready in models:\n"
                "    status = '✓ ready (v0.3.0)' if ready else '… deferred to M7.4'\n"
                "    print(f'{name:<11}  {status}')"
            ),
        ],
    )


def main() -> None:
    NB_DIR.mkdir(parents=True, exist_ok=True)
    # NB: 02b_alpha_zoo is now LIVE (M7.2 shipped in v0.4.0) — it is built
    # by scripts/build_notebook_m72.py, not here. `m7_2_alpha_zoo()` is kept
    # below only as historical reference; it is no longer generated so it
    # cannot clobber the live notebook. Only 02c / 02d remain placeholders.
    for p in (m7_3_three_param(), m7_4_advanced_sat()):
        nb = build_placeholder(p)
        out = NB_DIR / p.filename
        print(f"Executing {p.filename} ({p.milestone} → {p.planned_release})...")
        client = NotebookClient(nb, timeout=60, kernel_name="python3")
        client.execute(cwd=str(NB_DIR))
        out.write_text(nbf.writes(nb), encoding="utf-8")
        print(f"  Wrote {out}  ({len(nb.cells)} cells)")


if __name__ == "__main__":
    main()
