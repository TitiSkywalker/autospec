"""Proof-size table (paper Table "Proof-tree size by benchmark category").

Per testcase: theorems, tactics, distinct premises. Per theorem: tactics, depth,
premises. Each is reported as an average and a median, per category and overall.
Pure like :mod:`.stats`: cache entries in, a row dict out.
"""

from __future__ import annotations

import statistics
from typing import Callable

from analysis.proof_tree.stats import _avg, _category

# (row label, scope, extractor over a list of per-theorem dicts / a theorem dict)
ROWS = (
    ("Avg theorems", "case", "avg", lambda ths: len(ths)),
    ("Avg tactics", "case", "avg", lambda ths: sum(t["tactics"] for t in ths)),
    ("Med tactics", "case", "med", lambda ths: sum(t["tactics"] for t in ths)),
    ("Avg premises", "case", "avg", lambda ths: len(_premises(ths))),
    ("Med premises", "case", "med", lambda ths: len(_premises(ths))),
    ("Avg tactics", "theorem", "avg", lambda t: t["tactics"]),
    ("Med tactics", "theorem", "med", lambda t: t["tactics"]),
    ("Avg depth", "theorem", "avg", lambda t: t["depth"]),
    ("Med depth", "theorem", "med", lambda t: t["depth"]),
    ("Avg premises", "theorem", "avg", lambda t: len(t.get("premise_names", t.get("premises", [])))),
    ("Med premises", "theorem", "med", lambda t: len(t.get("premise_names", t.get("premises", [])))),
)


def _premises(theorems: list[dict]) -> set[str]:
    return {p for t in theorems for p in t.get("premise_names", t.get("premises", []))}


def _reduce(kind: str, xs: list[float]) -> float:
    return _avg(xs) if kind == "avg" else (statistics.median(xs) if xs else 0.0)


def size_table(results: dict[str, dict]) -> dict[str, dict]:
    """``{category: {"n": int, "rows": [value per ROWS entry]}}`` plus ``"Total"``.

    ``results`` maps case name to a cache entry; cases without theorems are skipped.
    """
    by_cat: dict[str, list[list[dict]]] = {}
    for name, result in results.items():
        ths = result.get("theorems") or []
        if ths:
            by_cat.setdefault(_category(name), []).append(ths)

    def column(cases: list[list[dict]]) -> dict:
        flat = [t for ths in cases for t in ths]
        values = []
        for _, scope, kind, fn in ROWS:
            xs = [fn(ths) for ths in cases] if scope == "case" else [fn(t) for t in flat]
            values.append(_reduce(kind, xs))
        return {"n": len(cases), "rows": values}

    table = {cat: column(cases) for cat, cases in sorted(by_cat.items())}
    table["Total"] = column([ths for cases in by_cat.values() for ths in cases])
    return table


def format_table(table: dict[str, dict]) -> str:
    cats = list(table)
    lines = [f"{'':<22}" + "".join(f"{c:>9}" for c in cats), f"{'n (testcases)':<22}" + "".join(f"{table[c]['n']:>9}" for c in cats)]
    section = None
    for i, (label, scope, _, _) in enumerate(ROWS):
        if scope != section:
            section = scope
            lines.append(f"-- per {scope}")
        lines.append(f"{label:<22}" + "".join(f"{table[c]['rows'][i]:>9.1f}" for c in cats))
    return "\n".join(lines)
