"""Generate per-case Aeneas file hints from claude-opus-5.5 traces.

Reads ``analysis/agent_trace/local/indirect_reads_data/claude-opus-5.5__*.json`` and writes
``analysis/ablation/aeneas_hints.json``:

    {
      "array_closest_pair": ["Aeneas/Std/Slice.lean", "Aeneas/Std/Vec.lean", ...],
      ...
    }

Only cases where claude-opus-5.5 PROVED the theorem and directly read at least one
Aeneas file are included.  The rel-paths are stored as they appear in the
indirect_reads data (e.g. ``Aeneas/Std/Slice.lean``); the workspace path is
``aeneas/backends/lean/{rel}``.

Usage::

    python3 analysis/ablation/ablation_hints.py
    python3 analysis/ablation/ablation_hints.py --out path/to/custom.json
"""

from __future__ import annotations

import argparse
import glob
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DEFAULT_OUT = ROOT / "analysis" / "ablation" / "aeneas_hints.json"


def _keys(v) -> list[str]:
    if isinstance(v, dict):
        return list(v.keys())
    return list(v) if v else []


OPUS_RESULTS = ROOT / "evaluation" / "local" / "runs" / "archive" / "opus-5.5-merged"


def build_hints() -> dict[str, list[str]]:
    """Return {case_name: sorted Aeneas rel-paths} for every case Opus-5.5 proved.

    Read sets come straight from Opus-5.5's own traces (direct reads only), so
    coverage does not depend on the cross-agent ``indirect_reads`` cache.
    """
    sys.path.insert(0, str(ROOT))
    from analysis.accuracy.verdicts import read_verdict
    from analysis.agent_trace.hmm import discover_cases
    from analysis.agent_trace.read_used_overlap import read_files

    hints: dict[str, list[str]] = {}
    for case, case_dir in sorted(discover_cases(OPUS_RESULTS).items()):
        if read_verdict(case_dir) != "PROVED":
            continue
        aeneas = sorted(
            k.split(":", 1)[1] for k in read_files(case_dir, "claude")
            if k.startswith("aeneas:")
        )
        if aeneas:
            hints[case] = aeneas
    return hints


def main() -> None:
    ap = argparse.ArgumentParser(description="Build aeneas_hints.json from claude-opus-5.5 traces")
    ap.add_argument("--out", default=str(DEFAULT_OUT))
    args = ap.parse_args()

    hints = build_hints()
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(hints, indent=2, sort_keys=True), encoding="utf-8")

    print(f"Wrote {len(hints)} cases to {out}")
    for case, files in sorted(hints.items()):
        print(f"  {case}: {len(files)} file(s)")


if __name__ == "__main__":
    main()
