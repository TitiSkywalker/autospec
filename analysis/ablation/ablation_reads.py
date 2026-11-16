"""Read/use statistics (DR, IR, C) for the ablation conditions.

Re-uses ``indirect_reads.process_case`` on each ablation result dir, so the sets
are defined exactly as in the paper's File Read Analysis:

    DR  direct reads            (agent opened the file)
    IR  indirect reads          (seen only via tool-call text / elaboration)
    C   explicit citations      (named in the final proof; automation excluded)

Per-case records are cached next to the baseline ones in
``analysis/agent_trace/local/indirect_reads_data/<condition>__<case>.json``;
the baseline is ``deepseek-v4.1-flash``.

Usage::

    python3 analysis/ablation/ablation_reads.py            # extract (cached) + report
    python3 analysis/ablation/ablation_reads.py --report-only
"""

from __future__ import annotations

import argparse
import fcntl
import json
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm import discover_cases  # noqa: E402
from analysis.agent_trace.indirect_reads import CACHE_DIR, process_case  # noqa: E402
from analysis.proof_tree.workspace import ensure_workspace  # noqa: E402

BASELINE = "deepseek-v4.1-flash"
CONDITIONS = {  # name -> results dir
    "dsh+skill-v3": "evaluation/local/runs/ablation/deepseek-v4.1-flash-skill-v3",
    "dsh+hint": "evaluation/local/runs/ablation/deepseek-v4.1-flash-hint",
}
LIBS = ("aeneas", "mathlib")


def _load(cache: Path, name: str) -> dict[str, dict]:
    out = {}
    for f in cache.glob(f"{name}__*.json"):
        r = json.loads(f.read_text())
        if "error" not in r:
            out[f.stem.split("__", 1)[1]] = r
    return out


def extract() -> None:
    cache = ROOT / CACHE_DIR
    lock = ROOT / "analysis" / "local" / "ablation_reads.lock"
    with open(lock, "w") as lf:
        fcntl.flock(lf, fcntl.LOCK_EX)  # the proof-tree workspace is shared
        ws = None
        for name, rel in CONDITIONS.items():
            for case, case_dir in sorted(discover_cases(ROOT / rel).items()):
                cf = cache / f"{name}__{case}.json"
                if cf.is_file():
                    continue
                ws = ws or ensure_workspace()
                print(f"[{name}/{case}] building + extracting...", file=sys.stderr, flush=True)
                cf.write_text(json.dumps(process_case(case, case_dir, "dsh", ws)))


def _sets(recs: list[dict], lib: str):
    dr, ir, c = set(), set(), set()
    for r in recs:
        rd = {k for k in r["read"] if k.startswith(lib + ":")}
        seen = {k for k in set(r["seen_paths"]) | set(r["name_seen"]) if k.startswith(lib + ":")}
        dr |= rd
        ir |= seen - rd
        c |= {k for k in r["used_explicit"] if k.startswith(lib + ":")}
    return dr, ir - dr, c


def report() -> None:
    cache = ROOT / CACHE_DIR
    base = _load(cache, BASELINE)
    conds = {n: _load(cache, n) for n in CONDITIONS}
    # compare on the cases every condition covers, so rows are like-for-like
    shared = set(base)
    for r in conds.values():
        shared &= set(r)
    print(f"cases covered by baseline and both conditions: {len(shared)}\n")
    hdr = f"{'condition':<22}{'lib':<9}{'|DR|':>6}{'|IR|':>6}{'|C|':>6}{'(DR∪IR)∩C':>11}{'η':>8}{'ρ':>8}{'DR/case':>9}"
    print(hdr)
    print("-" * len(hdr))
    for name, recs in [(BASELINE, base), *conds.items()]:
        rs = [recs[c] for c in sorted(shared)]
        for lib in LIBS:
            dr, ir, c = _sets(rs, lib)
            acc = dr | ir
            hit = len(acc & c)
            per = sum(len([k for k in r["read"] if k.startswith(lib + ":")]) for r in rs) / len(rs)
            print(f"{name:<22}{lib:<9}{len(dr):>6}{len(ir):>6}{len(c):>6}{hit:>11}"
                  f"{100*hit/max(len(acc),1):>7.1f}%{100*hit/max(len(c),1):>7.1f}%{per:>9.1f}")
    # Proved runs only, set sizes SUMMED over cases (the aggregation used in the
    # paper's tab:file_read), so eta/rho do not saturate as n grows. Proved =
    # Comparator-verified: the baseline's verdict.txt, and the guided run's
    # verified flips (staged in the *_verify dirs).
    L = ROOT / "evaluation" / "local" / "runs"

    def verified(*dirs):
        return {p.name for d in dirs for p in (L / d).iterdir()
                if (p / "verdict.txt").is_file() and (p / "verdict.txt").read_text().strip() == "PROVED"}

    ok = {BASELINE: verified("main/deepseek-v4.1-flash"),
          "dsh+skill-v3": verified("ablation/deepseek-v4.1-flash-skill-v3-verify", "ablation/deepseek-v4.1-flash-skill-v3-verify-six")}
    print("\nproved runs only (Comparator-verified; sizes summed over cases):")
    print(f"{'condition':<22}{'lib':<9}{'n':>4}{'|R|':>6}{'|C|':>6}{'hit':>6}{'η':>8}{'ρ':>8}{'R/case':>8}{'C/case':>8}")
    for name in (BASELINE, "dsh+skill-v3"):
        recs = (base if name == BASELINE else conds[name])
        rs = [r for c, r in recs.items() if c in ok[name]]
        for lib in LIBS:
            R = C = hit = 0
            for r in rs:
                dr, ir, c = _sets([r], lib)
                R += len(dr | ir); C += len(c); hit += len((dr | ir) & c)
            n = max(len(rs), 1)
            print(f"{name:<22}{lib:<9}{len(rs):>4}{R:>6}{C:>6}{hit:>6}"
                  f"{100*hit/max(R,1):>7.1f}%{100*hit/max(C,1):>7.1f}%{R/n:>8.1f}{C/n:>8.1f}")
    # restrict to flips: the interesting subset
    print("\nflipped cases only (PROVED under the condition, not at baseline):")
    for name, recs in conds.items():
        flips = [c for c in shared if recs[c].get("verdict") == "PROVED" and base[c].get("verdict") != "PROVED"]
        print(f"  {name}: {len(flips)} cases")
        for lib in LIBS:
            for label, src in (("baseline", base), (name, recs)):
                dr, ir, c = _sets([src[x] for x in flips], lib) if flips else (set(), set(), set())
                hit = len((dr | ir) & c)
                print(f"    {lib:<8}{label:<14} DR={len(dr):>3} IR={len(ir):>3} C={len(c):>3} hit={hit:>3}")


def main() -> None:
    ap = argparse.ArgumentParser(description="DR/IR/C statistics for the ablation conditions")
    ap.add_argument("--report-only", action="store_true")
    args = ap.parse_args()
    if not args.report_only:
        extract()
    report()


if __name__ == "__main__":
    main()
