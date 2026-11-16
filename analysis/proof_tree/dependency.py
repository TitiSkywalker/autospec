"""Theorem-dependency graph of one testcase: how its proof is organized.

Individual theorems look alike across testcases (similar depth, similar tactic
mix), so what distinguishes a large proof from a small one is how its theorems
are *organized*. This module measures that from data already in the cache: a
theorem's cited premises (``premise_names``) include the other ``Spec.lean``
theorems it uses, so a premise whose name is another theorem of the same
testcase is a dependency edge ``theorem -> premise``.

Only theorems with a proof tree are nodes; definitions and structures from
``Function.lean`` are not, so edges to them are ignored. Lean forbids cyclic
dependencies, but the layering below tolerates a cycle rather than looping.

This module is pure (dictionaries in, dictionaries out), like :mod:`.stats`.

Per-testcase metrics (see :func:`dependency_stats`):

* ``theorems``    — nodes.
* ``edges``       — theorem-to-theorem citations.
* ``chain``       — longest dependency chain, in theorems.
* ``width``       — theorems in the most populated layer, where a theorem's
                    layer is the length of the longest chain below it (leaves
                    are layer 1). Theorems in one layer are mutually
                    independent, so this is how much of the proof can be
                    developed in parallel.
* ``roots``       — theorems no other theorem cites (the final statements).
* ``critical``    — share of all tactic applications that lie on the heaviest
                    dependency chain. Near 1 means the proof is essentially one
                    linear argument; small means the work is spread over many
                    parallel lemmas.
"""

from __future__ import annotations

from typing import Optional

from analysis.proof_tree.stats import _avg, _category

METRICS = ("theorems", "edges", "chain", "width", "roots", "critical")


def _dependencies(theorems: list[dict]) -> dict[str, list[str]]:
    """``{theorem: [theorems it cites]}``, restricted to theorems of this testcase."""
    names = {t["theorem"] for t in theorems}
    return {
        t["theorem"]: [
            p
            for p in t.get("premise_names", t.get("premises", []))
            if p in names and p != t["theorem"]
        ]
        for t in theorems
    }


def dependency_stats(theorems: list[dict]) -> Optional[dict]:
    """Organization metrics for one testcase's per-theorem stats; ``None`` if it has no theorems."""
    if not theorems:
        return None
    deps = _dependencies(theorems)
    tactics = {t["theorem"]: t["tactics"] for t in theorems}

    layer: dict[str, int] = {}
    weight: dict[str, int] = {}  # tactics on the heaviest chain ending at this theorem

    def visit(name: str, stack: frozenset) -> None:
        if name in layer:
            return
        below = [d for d in deps[name] if d not in stack]
        for d in below:
            visit(d, stack | {name})
        layer[name] = 1 + max((layer[d] for d in below if d in layer), default=0)
        weight[name] = tactics[name] + max((weight[d] for d in below if d in weight), default=0)

    for name in deps:
        visit(name, frozenset())

    cited = {d for ds in deps.values() for d in ds}
    chain = max(layer.values())
    total = sum(tactics.values())
    return {
        "theorems": len(deps),
        "edges": sum(len(ds) for ds in deps.values()),
        "chain": chain,
        "width": max(sum(1 for v in layer.values() if v == k) for k in range(1, chain + 1)),
        "roots": sum(1 for n in deps if n not in cited),
        "critical": max(weight.values()) / total if total else 0.0,
    }


def dependency_by_category(results: dict[str, dict]) -> dict[str, dict]:
    """Mean of each metric per category (plus ``"Total"``), over cases with theorems.

    ``results`` maps case name to a cache entry (``{"theorems": [...]}``). The
    ``critical`` entry is a mean of per-testcase shares.
    """
    per_case: dict[str, list[dict]] = {}
    for name, result in results.items():
        s = dependency_stats(result.get("theorems") or [])
        if s is not None:
            per_case.setdefault(_category(name), []).append(s)

    def mean_row(rows: list[dict]) -> dict:
        out = {m: _avg([r[m] for r in rows]) for m in METRICS}
        out["n"] = len(rows)
        return out

    table = {cat: mean_row(rows) for cat, rows in sorted(per_case.items())}
    table["Total"] = mean_row([r for rows in per_case.values() for r in rows])
    return table
