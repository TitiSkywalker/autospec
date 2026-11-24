"""Verdict counts per run: how many cases ended PROVED, UNPROVED, ...

Verdicts come from ``verdict.txt`` written by ``python -m evaluation.verifier``
(see :mod:`analysis.accuracy.verdicts`).
"""

from collections import Counter

from analysis.accuracy.runs import Context
from analysis.accuracy.verdicts import collect

VERDICTS = ("PROVED", "UNPROVED", "BUILD_FAIL", "CHEAT", "KERNEL_FAIL")


def main(ctx: Context) -> None:
    rows = {label: Counter(collect(path).values()) for label, path in ctx.runs.items()}
    extra = sorted({v for c in rows.values() for v in c} - set(VERDICTS))
    cols = list(VERDICTS) + extra
    width = max(len(label) for label in rows) + 2

    header = f"{'run':<{width}}" + "".join(f"{c:>12}" for c in cols) + f"{'total':>8}{'accuracy':>10}"
    print(header)
    print("-" * len(header))
    for label, counts in rows.items():
        total = sum(counts.values())
        acc = counts["PROVED"] / total if total else 0.0
        print(f"{label:<{width}}" + "".join(f"{counts[c]:>12}" for c in cols)
              + f"{total:>8}{acc:>10.1%}")
