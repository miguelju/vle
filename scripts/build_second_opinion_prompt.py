#!/usr/bin/env python
"""Assemble the external second-opinion prompt for the ``d_ln_phi_d_n`` trial.

The prompt is built from **verbatim line ranges** of the real sources rather
than hand-copied snippets, so it can never drift from the code it asks about.
Re-run after touching ``engine/src/mixture.rs`` and the prompt regenerates.

Usage (from the repo root)::

    ~/miniconda3/envs/vle/bin/python scripts/build_second_opinion_prompt.py

Writes ``docs/plans/engine/second-opinion/dualn-prompt.md``.

See ``docs/plans/engine/SECOND_OPINION_TRIAL.md`` for what this is for.
"""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "docs/plans/engine/second-opinion/dualn-prompt.md"

MIXTURE = "engine/src/mixture.rs"
BENCH = "engine/benches/engine_bench.rs"


def slice_lines(rel_path: str, start: int, end: int) -> str:
    """Return lines ``start..end`` (1-indexed, inclusive) of ``rel_path``."""
    lines = (ROOT / rel_path).read_text().splitlines()
    return "\n".join(lines[start - 1 : end])


def block(rel_path: str, start: int, end: int, caption: str) -> str:
    body = slice_lines(rel_path, start, end)
    return f"**`{rel_path}:{start}-{end}` — {caption}**\n\n```rust\n{body}\n```\n"


TASK = """\
# Second opinion: composition-derivative strategy for a cubic-EOS mixture core

You are a senior Rust performance engineer with a strong numerical-methods
background. You are being asked for an **independent design opinion**, not for
agreement. If you think the premise below is wrong, say so.

## The system

`vle-thermo` is a vapor-liquid-equilibrium engine (Rust core, PyO3/UniFFI/wasm
bindings). Canonical internal units: temperature **K**, pressure **kPa
absolute**, dimensionless EOS parameters (A, B, U, W).

The mixture core is written **once**, generic over a scalar type
`D: DualNum<f64> + Copy` (the `num-dual` crate, version **0.11.2**, is already
a dependency). The same source serves three paths:

- `D = f64` — the plain value path.
- `D = Dual64` — exact first derivatives (composition, and separately T or P).
- `D = Dual2_64` — second derivatives for a Gibbs-Helmholtz consistency check.

This "algebra written once, generic over the scalar" property is load-bearing.
An earlier proposal to build a separate flattened `PreparedModel` struct was
rejected specifically because it implied a second implementation of every
mixing rule, which would then have to be kept in sync forever.

## The problem

`d_ln_phi_d_n` returns the Jacobian `jac[i][j] = ∂ln φ̂ᵢ/∂nⱼ`.

For classical / IVDW mixing with a 2-parameter EOS there is a hand-derived
analytic closed form, and it is fast. For **every other case** — Wong-Sandler,
Huron-Vidal, MHV1, MHV2, and any 3-parameter EOS (Schmidt-Wenzel, Patel-Teja) —
there is no closed form, and the code falls back to dual numbers: **one full
`Dual64` evaluation per Jacobian column**, in a loop over `j`. Each of those
sweeps re-runs the entire mixture evaluation (pure-component parameters, the
n² mixing algebra, the activity model, the cubic Z-root solve) to extract a
single column.

Measured on this machine with `cargo bench`, n = 4 components:

| benchmark | time |
|---|---|
| `d_ln_phi_d_n_classical_n4` (analytic closed form) | **252 ns** |
| `d_ln_phi_d_n_wong_sandler_n4` (n dual sweeps) | **1285 ns** |

So the dual path costs **5.1×** the analytic path at n = 4, and the gap grows
with n because the number of sweeps grows with n while each sweep is already
at least O(n²).

The project's own performance plan calls this "the largest remaining
algorithmic win" and also "the highest accuracy risk", and has deferred it
twice. That is why an outside opinion is wanted.

## Your task

Propose — and write — the change that removes the per-column sweep, so the
whole Jacobian comes out of **one** pass over the mixture algebra.

### Hard constraints

1. **The mixing algebra stays written once.** Whatever you propose must flow
   through the existing generic functions (`mixture_params`,
   `mixture_params_with`, `quad_a`, `ln_gamma_all_generic`, `z_mix_generic`,
   `ln_phi_from_params_generic`). Forking the mixing rules into a second
   implementation is an automatic rejection.
2. **No `unsafe`.** The project has an explicit rule against trading memory
   safety for bounds-check removal in a thermodynamics library.
3. **The public signature of `d_ln_phi_d_n` must not change.** It is exposed
   through PyO3, UniFFI (Swift/Kotlin) and wasm-bindgen; the returned
   `Vec<Vec<f64>>` is part of three published surfaces.
4. **Accuracy is non-negotiable and is tested.** The Jacobian must stay
   symmetric (a Gibbs-Duhem consequence) to ~1e-9 relative, and must match a
   central-difference oracle to 1e-5 (classical) / 1e-4 (exotic rules and
   3-parameter EOS). Existing tests assert exactly this.
5. **No heap allocation inside iteration loops** where it can be avoided. The
   working buffer type is `SmallVec<[D; 8]>`, chosen so mixtures up to 8
   components never touch the heap.
6. Today n is typically 2-10. A planned future track needs the same code to
   run at **n ≈ 300** (petroleum pseudocomponents in a crude distillation
   column). Say explicitly how your design behaves as n grows, and whether it
   should change shape at some n.

### What to deliver

1. **Diagnosis** — where the 1285 ns actually goes, and what the theoretical
   floor is for a single-pass design. Be quantitative.
2. **Design** — the exact Rust types you would use. Name them concretely. If a
   crate already in the dependency tree provides what you need, use it and say
   so rather than hand-rolling; if you think hand-rolling is genuinely better
   here, argue why.
3. **The code** — a complete, compiling replacement for `d_ln_phi_d_n` plus any
   new helpers or trait bounds it needs. Not a sketch. If a generic bound
   elsewhere has to be relaxed or widened, show that edit too.
4. **Accuracy analysis** — where your version could lose precision relative to
   the current per-column sweep, and why the two tolerance requirements above
   still hold.
5. **Prediction** — what you expect the `d_ln_phi_d_n_wong_sandler_n4`
   benchmark to read after the change, and what it would read at n = 20. State
   these as numbers; they will be measured against your answer.
6. **Risks and what you would reject** — anything you considered and dismissed,
   with the reason.

Be concrete and terse. Assume the reader knows both Rust and thermodynamics.
"""

FOOTER = """
## Reference: the invariants your change must not break

**`engine/src/mixture.rs` (test module) — the accuracy oracle**

```rust
{tests}
```

## Reference: the benchmark that will grade you

{bench}

Baseline to beat: `d_ln_phi_d_n_wong_sandler_n4` = **1285 ns**.
The analytic classical path, for scale, is **252 ns**.
"""


def main() -> None:
    parts: list[str] = [TASK, "\n## The code\n"]

    parts.append(block(MIXTURE, 66, 90, "imports and the working-buffer type"))
    parts.append(block(MIXTURE, 108, 160, "the spec types the API takes"))
    parts.append(block(MIXTURE, 216, 310, "pure-component params and the mixture-params struct"))
    parts.append(block(MIXTURE, 358, 430, "the generic entry points and the n² quadratic form"))
    parts.append(block(MIXTURE, 505, 546, "the Wong-Sandler branch — the slow case"))
    parts.append(block(MIXTURE, 760, 861, "Z root, fugacity from params, and the generic evaluator"))
    parts.append(block(MIXTURE, 1170, 1226, "THE TARGET — the per-column dual sweep"))
    parts.append(block(MIXTURE, 1228, 1250, "the analytic fast path it falls back from"))

    parts.append(
        FOOTER.format(
            tests=slice_lines(MIXTURE, 1876, 1894),
            bench=block(BENCH, 610, 634, "criterion benchmark"),
        )
    )

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(parts))
    text = OUT.read_text()
    print(f"wrote {OUT.relative_to(ROOT)}")
    print(f"  {len(text):,} chars  ~{len(text) // 4:,} tokens (rough)")


if __name__ == "__main__":
    main()
