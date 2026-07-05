"""Build notebooks/m06_numerics.ipynb.

The Milestone 6 notebook tours every numerical primitive shipped under
``engine/src/numerics/`` — the cubic solver, the bracketed and
quasi-Newton scalar solvers, Halley's cubic-convergence iteration, and
Broyden's multi-variable quasi-Newton. Each section runs a realistic
worked example, the convergence sections produce plots, and two
exercises at the end give the reader a chance to call into the wheel
themselves.

Follows the CLAUDE.md *Notebook Conventions* (title + motivation → hub
sandbox notice → optional pip install → research-paper context → what
was built → worked examples → exercises → references).

Run:

    $ python scripts/build_notebook_m06.py

The script also executes the generated notebook end-to-end via
NotebookClient before saving so a regression in the engine surfaces
here, not on the hub.
"""

from __future__ import annotations

from pathlib import Path

import nbformat as nbf
from nbclient import NotebookClient

REPO_ROOT = Path(__file__).resolve().parents[1]
NB_PATH = REPO_ROOT / "notebooks" / "m06_numerics.ipynb"


def md(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_markdown_cell(text)


def code(text: str) -> nbf.NotebookNode:
    return nbf.v4.new_code_cell(text)


def build() -> nbf.NotebookNode:
    nb = nbf.v4.new_notebook()
    cells: list[nbf.NotebookNode] = []

    # ---- Title + motivation ---------------------------------------------
    cells.append(md(
        "# Numerical Primitives — Milestone 6\n"
        "\n"
        "Every thermodynamic calculation in later milestones bottoms out in "
        "one of a handful of numerical operations: rooting a cubic in Z, "
        "finding the temperature that makes a residual zero, iterating a "
        "multi-variable system to convergence in a flash. Milestone 6 ships "
        "all of those primitives in one place — `engine/src/numerics/` — "
        "with tests, PyO3 bindings, and the (12) Poling & Prausnitz / Brent "
        "/ Halley / Broyden algorithm choices spelled out in "
        "[`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md). "
        "This notebook walks each primitive with a runnable example."
    ))

    # ---- Optional upgrade cell (CLAUDE.md §2b) --------------------------
    cells.append(md(
        "## Setup (optional)\n"
        "\n"
        "The cell below is **commented out by default**. Uncomment it if you "
        "want to use the latest `vle-thermo` released on PyPI instead of "
        "whatever version is currently installed in your kernel — useful when "
        "running locally to make sure you're testing against the published "
        "wheel, exactly the way a downstream user would."
    ))
    cells.append(code(
        "# Optional: pull the latest vle-thermo from PyPI.\n"
        "# Uncomment if you want the newest released version instead of\n"
        "# whatever is currently in your kernel.\n"
        "# %pip install --upgrade vle-thermo"
    ))

    # ---- Research-paper / MODERNIZATION_PLAN context --------------------
    cells.append(md(
        "## Context — what the legacy code did vs. what M6 ships\n"
        "\n"
        "The VB6 and Pascal sources hand-rolled their numerical methods "
        "inline with each thermodynamic routine. The "
        "[`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md) "
        "*Algorithm Choices* section spells out which legacy choices we "
        "kept and which we replaced. The M6 surface implements the\n"
        "replacements:\n"
        "\n"
        "| Legacy | M6 replacement | Why |\n"
        "|---|---|---|\n"
        "| Hand-rolled Cardano (`McommonFunctions.bas:324`) | `solve_cubic` + (12) Poling/Prausnitz robustness | Stable across the critical-point discriminant boundary. |\n"
        "| Regula Falsi (`clsSatPressureSolver.cls`) | `brent` (default) + `illinois` | Super-linear convergence with bisection fallback. |\n"
        "| Newton-Raphson inline | `halley` + `broyden` | Halley: cubic convergence for scalar Rachford-Rice. Broyden: rank-1 update + periodic refresh for the multi-variable flash, K=5 steps between refreshes per the §F default. |\n"
        "| Hand-rolled Gauss elimination (`McommonFunctions.bas:24`) | `nalgebra::DMatrix.lu()` (used inside `broyden`) | LU with partial pivoting + parallel BLAS backends. |\n"
        "\n"
        "Each algorithm is small enough to read in one sitting — see "
        "[`engine/src/numerics/`](https://github.com/miguelju/vle/tree/main/engine/src/numerics)."
    ))

    # ---- What was built --------------------------------------------------
    cells.append(md(
        "## What was built in Milestone 6\n"
        "\n"
        "Five primitives, each exposed as both a Rust function (engine-side) "
        "and a `vle._engine` PyO3 binding (Python-side). The Python names "
        "are flat — `from vle._engine import solve_cubic, brent, …` — so "
        "the higher-level `vle` wrapper can re-export them into themed "
        "submodules later without changing the binding shape.\n"
        "\n"
        "1. **`solve_cubic(a, b, c, d)`** — Cardano + (12) Poling robustness; "
        "   returns every real root of `a·x³ + b·x² + c·x + d = 0` in "
        "   ascending order.\n"
        "2. **`brent(f, a, b, …)`** — Brent's bracketed scalar root finder. "
        "   Default for `f(x) = 0` when you can bracket the answer.\n"
        "3. **`illinois(f, a, b, …)`** — Lighter modified-Regula-Falsi "
        "   alternative with weight halving on stalls.\n"
        "4. **`halley(f_and_derivs, x0, …)`** — Cubic-convergence Newton++ "
        "   when you have analytical `f`, `f'`, `f''`. Used in M9's "
        "   Rachford-Rice.\n"
        "5. **`broyden(f, x0, …)`** — Quasi-Newton for multi-variable "
        "   systems. Rank-1 secant updates + periodic finite-difference "
        "   refresh every K=5 steps. The workhorse for M9 flash.\n"
        "\n"
        "Plus utility functions exposed for parity with the Rust side: "
        "`sum_frac_residual`, `norm_l1`, `norm_l2`, `norm_linf`. Everything "
        "in the rest of this notebook is built on these eight names."
    ))

    # ---- Setup cell ------------------------------------------------------
    cells.append(md(
        "## Setup\n"
        "\n"
        "Pull every binding we'll use, plus matplotlib for the convergence "
        "plots."
    ))
    cells.append(code(
        "import math\n"
        "\n"
        "import matplotlib.pyplot as plt\n"
        "\n"
        "from vle._engine import (\n"
        "    brent,\n"
        "    broyden,\n"
        "    halley,\n"
        "    illinois,\n"
        "    norm_l2,\n"
        "    solve_cubic,\n"
        "    sum_frac_residual,\n"
        "    version,\n"
        ")\n"
        "\n"
        "print(f'vle._engine version: {version()}')"
    ))

    # ── Worked example 1: Cardano on a Z-factor-style cubic ──────────────
    cells.append(md(
        "## 1. Cardano on a Z-factor-style cubic\n"
        "\n"
        "A cubic equation of state (Peng-Robinson, RKS, …) reduces to a "
        "cubic in the compressibility factor `Z`. When the EOS yields **three** "
        "real roots, the smallest is the liquid `Z`, the largest is the "
        "vapor `Z`, and the middle root is thermodynamically unstable. "
        "`solve_cubic` returns every real root sorted ascending; we pick the "
        "right extreme with `min` / `max`.\n"
        "\n"
        "The cubic below is constructed from known roots so we can compare "
        "against an exact answer:\n"
        "$$ (Z - 0.10)(Z - 0.15)(Z - 0.80) = Z^3 - 1.05\\,Z^2 + 0.215\\,Z - 0.012 $$"
    ))
    cells.append(code(
        "coeffs = (1.0, -1.05, 0.215, -0.012)\n"
        "roots = solve_cubic(*coeffs)\n"
        "print(f'all roots (ascending): {roots}')\n"
        "print(f'liquid Z (smallest) : {min(roots):.4f}')\n"
        "print(f'vapor  Z (largest)  : {max(roots):.4f}')\n"
        "\n"
        "# Residual sanity check — plug each root back into the polynomial\n"
        "for r in roots:\n"
        "    a, b, c, d = coeffs\n"
        "    residual = ((a*r + b)*r + c)*r + d\n"
        "    print(f'  residual at Z={r:.4f}: {residual:+.2e}')\n"
        "\n"
        "assert math.isclose(min(roots), 0.10, abs_tol=1e-10)\n"
        "assert math.isclose(max(roots), 0.80, abs_tol=1e-10)"
    ))

    # ── Worked example 2: Brent vs Illinois convergence ──────────────────
    cells.append(md(
        "## 2. Brent vs. Illinois — convergence on the Dottie number\n"
        "\n"
        "Both `brent` and `illinois` solve `f(x) = 0` on a bracketed "
        "interval. Brent combines bisection, secant, and inverse-quadratic "
        "interpolation; Illinois is a lighter modified-Regula-Falsi. On "
        "smooth problems Brent typically wins on iteration count.\n"
        "\n"
        "Counting iterations is the cleanest way to see the difference. We "
        "wrap each function in a counting closure, find the Dottie number "
        "(`cos(x) − x = 0`, root ≈ 0.7390851332), and read off how many "
        "evaluations each algorithm took at a range of tolerances."
    ))
    cells.append(code(
        "def count_calls(f):\n"
        "    \"\"\"Wrap f so the .calls attribute reports invocation count.\"\"\"\n"
        "    def wrapped(x):\n"
        "        wrapped.calls += 1\n"
        "        return f(x)\n"
        "    wrapped.calls = 0\n"
        "    return wrapped\n"
        "\n"
        "tolerances = [1e-3, 1e-6, 1e-9, 1e-12, 1e-15]\n"
        "brent_counts = []\n"
        "illinois_counts = []\n"
        "for tol in tolerances:\n"
        "    f_b = count_calls(lambda x: math.cos(x) - x)\n"
        "    brent(f_b, 0.0, 1.0, tol, 100)\n"
        "    brent_counts.append(f_b.calls)\n"
        "\n"
        "    f_i = count_calls(lambda x: math.cos(x) - x)\n"
        "    illinois(f_i, 0.0, 1.0, tol, 100)\n"
        "    illinois_counts.append(f_i.calls)\n"
        "\n"
        "print(f'{\"xtol\":>10} {\"Brent\":>8} {\"Illinois\":>10}')\n"
        "for tol, b, i in zip(tolerances, brent_counts, illinois_counts):\n"
        "    print(f'{tol:>10.0e} {b:>8} {i:>10}')\n"
        "\n"
        "fig, ax = plt.subplots(figsize=(6, 4))\n"
        "ax.semilogx(tolerances, brent_counts, 'o-', label='Brent')\n"
        "ax.semilogx(tolerances, illinois_counts, 's-', label='Illinois')\n"
        "ax.set_xlabel('Convergence tolerance (xtol)')\n"
        "ax.set_ylabel('Function evaluations')\n"
        "ax.set_title('Iteration count vs. tolerance — cos(x) − x = 0')\n"
        "ax.legend(); ax.grid(True, alpha=0.3)\n"
        "ax.invert_xaxis()  # tighter tolerance on the right\n"
        "fig.tight_layout()\n"
        "plt.show()"
    ))

    # ── Worked example 3: Halley vs Newton ────────────────────────────────
    cells.append(md(
        "## 3. Halley vs. Newton — convergence-rate comparison\n"
        "\n"
        "Halley's method buys you cubic convergence (≈ tripling the number "
        "of correct digits per iteration on smooth `f`) for the cost of one "
        "extra derivative evaluation per step. Newton-Raphson is only "
        "quadratic. On well-behaved problems the gap shows up after just a "
        "few iterations.\n"
        "\n"
        "Solving `x² − 612 = 0` starting from `x = 10` is the textbook "
        "demonstration. We implement bare Newton in Python (we don't ship "
        "it — Halley is the default in the engine), count both, and confirm "
        "Halley needs fewer evaluations."
    ))
    cells.append(code(
        "def newton(f, df, x0, ftol, max_iter):\n"
        "    \"\"\"Plain Newton-Raphson for comparison. Halley is in the wheel.\n"
        "    Convergence on residual size (not step size) so we don't hit\n"
        "    the sub-ULP floor on this well-conditioned problem.\n"
        "    \"\"\"\n"
        "    x = x0\n"
        "    for i in range(max_iter):\n"
        "        fx = f(x)\n"
        "        if abs(fx) < ftol:\n"
        "            return x, i\n"
        "        x = x - fx / df(x)\n"
        "    raise RuntimeError('Newton: no convergence')\n"
        "\n"
        "# Halley with the engine's default tolerance (xtol=1e-12, max_iter=50)\n"
        "halley_iters = 0\n"
        "def halley_callback(x):\n"
        "    global halley_iters\n"
        "    halley_iters += 1\n"
        "    return (x*x - 612.0, 2.0*x, 2.0)\n"
        "\n"
        "halley_root = halley(halley_callback, 10.0)\n"
        "newton_root, newton_iters = newton(\n"
        "    lambda x: x*x - 612.0,\n"
        "    lambda x: 2.0*x,\n"
        "    10.0, 1e-12, 50,\n"
        ")\n"
        "\n"
        "true_sqrt = math.sqrt(612.0)\n"
        "print(f'true value     : √612 = {true_sqrt}')\n"
        "print(f'Halley result  : {halley_root}  (after {halley_iters} iters)')\n"
        "print(f'Newton result  : {newton_root}  (after {newton_iters} iters)')\n"
        "\n"
        "assert math.isclose(halley_root, true_sqrt, rel_tol=1e-12)\n"
        "assert math.isclose(newton_root, true_sqrt, rel_tol=1e-6)\n"
        "assert halley_iters < newton_iters, (\n"
        "    f'expected Halley to beat Newton; got {halley_iters} vs {newton_iters}'\n"
        ")"
    ))

    # ── Worked example 4: Broyden on a 2-equation system ─────────────────
    cells.append(md(
        "## 4. Broyden on a 2-equation nonlinear system\n"
        "\n"
        "`broyden` solves `F(x) = 0` where `F: ℝⁿ → ℝⁿ`. Each iteration: "
        "solve `J · Δx = −F` for the step, update `x`, then do a **rank-1 "
        "secant update** of `J` instead of recomputing it from scratch. "
        "Every K=5 iterations (configurable) we refresh `J` via finite "
        "differences to keep the approximation honest.\n"
        "\n"
        "The system below has a closed-form root we can check against:\n"
        "$$ x^2 + y^2 = 2, \\qquad x \\cdot y = 1 \\;\\;\\Longrightarrow\\;\\; (x, y) = (1, 1) $$"
    ))
    cells.append(code(
        "def F(v):\n"
        "    x, y = v\n"
        "    return [x*x + y*y - 2.0, x*y - 1.0]\n"
        "\n"
        "root = broyden(F, [0.5, 1.5], xtol=1e-14, ftol=1e-14)\n"
        "print(f'Broyden root: ({root[0]:.10f}, {root[1]:.10f})')\n"
        "print(f'F at root   : {F(root)}')\n"
        "\n"
        "assert math.isclose(root[0], 1.0, abs_tol=1e-6)\n"
        "assert math.isclose(root[1], 1.0, abs_tol=1e-6)\n"
        "# ‖F‖ should be near zero — every component a clean residual\n"
        "assert norm_l2(F(root)) < 1e-10"
    ))

    # ── Exercises ─────────────────────────────────────────────────────────
    cells.append(md(
        "## Your turn — exercises\n"
        "\n"
        "Two short exercises calling into the M6 surface. Template cells "
        "have `# TODO` markers; solutions are in the collapsed block at the "
        "bottom of the notebook."
    ))

    cells.append(md(
        "### Exercise 1 — Find the vapor Z of a custom cubic\n"
        "\n"
        "You're given an EOS cubic in `Z` with coefficients\n"
        "`(a, b, c, d) = (1.0, -1.42, 0.59, -0.045)`. Use `solve_cubic` to "
        "find every real root, then pick the **vapor** root (the largest "
        "one) and confirm it satisfies the cubic to within 1e-10.\n"
        "\n"
        "Bonus: also compute the liquid root and print the ratio "
        "`Z_vapor / Z_liquid`."
    ))
    cells.append(code(
        "# TODO: roots = solve_cubic(1.0, -1.42, 0.59, -0.045)\n"
        "# TODO: z_vapor = max(roots)\n"
        "# TODO: assert abs(((a*z + b)*z + c)*z + d) < 1e-10\n"
        "\n"
        "# your code here\n"
    ))

    cells.append(md(
        "### Exercise 2 — Solve a custom 2-equation system with Broyden\n"
        "\n"
        "Define the residual function:\n"
        "$$ F(x, y) = \\bigl(\\; e^x + y - 5, \\quad x + e^y - 5 \\;\\bigr) $$\n"
        "\n"
        "and use `broyden` (with default tolerances) to find a root "
        "starting from `[1.0, 1.0]`. Verify `‖F(root)‖₂ < 1e-7`.\n"
        "\n"
        "This is a small but genuinely non-quadratic system — the kind of "
        "shape M9's flash residuals will have."
    ))
    cells.append(code(
        "# TODO: import math\n"
        "# TODO: def F(v):\n"
        "#           x, y = v\n"
        "#           return [math.exp(x) + y - 5.0, x + math.exp(y) - 5.0]\n"
        "# TODO: root = broyden(F, [1.0, 1.0])\n"
        "# TODO: assert norm_l2(F(root)) < 1e-7\n"
        "\n"
        "# your code here\n"
    ))

    # ── Solutions (collapsed) ─────────────────────────────────────────────
    cells.append(md(
        "### Solutions (expand to see)\n"
        "\n"
        "<details>\n"
        "<summary>Click to show Exercise 1 solution</summary>\n"
        "\n"
        "```python\n"
        "a, b, c, d = 1.0, -1.42, 0.59, -0.045\n"
        "roots = solve_cubic(a, b, c, d)\n"
        "print(f'all real roots: {roots}')\n"
        "z_vapor = max(roots)\n"
        "z_liquid = min(roots)\n"
        "print(f'vapor Z : {z_vapor:.6f}')\n"
        "print(f'liquid Z: {z_liquid:.6f}')\n"
        "print(f'ratio   : {z_vapor / z_liquid:.2f}')\n"
        "residual = ((a*z_vapor + b)*z_vapor + c)*z_vapor + d\n"
        "assert abs(residual) < 1e-10, residual\n"
        "```\n"
        "\n"
        "</details>\n"
        "\n"
        "<details>\n"
        "<summary>Click to show Exercise 2 solution</summary>\n"
        "\n"
        "```python\n"
        "def F(v):\n"
        "    x, y = v\n"
        "    return [math.exp(x) + y - 5.0, x + math.exp(y) - 5.0]\n"
        "\n"
        "root = broyden(F, [1.0, 1.0])\n"
        "print(f'root: ({root[0]:.6f}, {root[1]:.6f})')\n"
        "print(f'F(root) L2 norm: {norm_l2(F(root)):.2e}')\n"
        "assert norm_l2(F(root)) < 1e-7\n"
        "```\n"
        "\n"
        "</details>"
    ))

    # ---- References ------------------------------------------------------
    cells.append(md(
        "## References\n"
        "\n"
        "- **Modernization plan**: "
        "[`MODERNIZATION_PLAN.md`](https://github.com/miguelju/vle/blob/main/MODERNIZATION_PLAN.md) "
        "*Algorithm Choices* (§A–§H) — rationale for every M6 algorithm pick.\n"
        "- **Source**: "
        "[`engine/src/numerics/`](https://github.com/miguelju/vle/tree/main/engine/src/numerics) "
        "— each algorithm lives in its own file with a module-level "
        "doc-comment.\n"
        "- **PyO3 bindings**: "
        "[`engine/src/py_bindings.rs`](https://github.com/miguelju/vle/blob/main/engine/src/py_bindings.rs) "
        "— the M6 surface lives in the same file as the rest of "
        "`vle._engine`.\n"
        "- (12) Poling, B. E.; Prausnitz, J. M.; O'Connell, J. P., *The "
        "Properties of Gases and Liquids* (5th ed.), §4-6 — discriminant-"
        "robustness recommendation for cubic-EOS Cardano.\n"
        "- Brent, R. P. (1971), *An algorithm with guaranteed convergence "
        "for finding a zero of a function*.\n"
        "- Halley, E. (1694), *A new, exact, and easy method of finding the "
        "roots of any equations generally*.\n"
        "- Broyden, C. G. (1965), *A class of methods for solving nonlinear "
        "simultaneous equations*.\n"
        "- Press, W. H. et al., *Numerical Recipes* (3rd ed.), §9.4 (Halley) "
        "and §9.7 (Broyden) — modern restatements with implementation "
        "notes."
    ))

    nb.cells = cells
    nb.metadata = {
        "kernelspec": {"display_name": "Python 3", "language": "python", "name": "python3"},
        "language_info": {"name": "python", "pygments_lexer": "ipython3"},
    }
    return nb


def main() -> None:
    nb = build()
    NB_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    print(f"Wrote {NB_PATH}")

    # Execute to verify cells run top-to-bottom in a fresh kernel.
    client = NotebookClient(
        nb,
        timeout=120,
        kernel_name="python3",
        resources={"metadata": {"path": str(NB_PATH.parent)}},
    )
    client.execute()
    NB_PATH.write_text(nbf.writes(nb), encoding="utf-8")
    print(f"Executed + saved {NB_PATH}")


if __name__ == "__main__":
    main()
