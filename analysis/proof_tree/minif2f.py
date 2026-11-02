"""miniF2F control corpus: LLM-solved competition-math proofs.

A second pure-math baseline next to NavierStokes/Euler. The proving agent is an
LLM in both this corpus and the vericoding one, so only the *domain* differs.
Proofs are short and self-contained (``import Mathlib``), measured with the same
per-theorem statistics as every other corpus (:func:`~.stats._tree_stats`).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Optional

from analysis.proof_tree.config import CACHE_DIR, MINIF2F_CACHE_TAG, MINIF2F_SAMPLES, ROOT
from analysis.proof_tree.workspace import ensure_workspace, process_source


def sample_proofs(n_per_file: Optional[int] = None) -> list[tuple[str, str]]:
    """``[(sample_id, lean_code)]``: the first PASSED trial of the first N passed problems per file."""
    out = []
    for label, path, default_n in MINIF2F_SAMPLES:
        n = n_per_file if n_per_file is not None else default_n
        p = Path(path)
        if not p.is_file():
            print(f"(skipping miniF2F {label}: {path} not found)", file=sys.stderr)
            continue
        got = 0
        with p.open() as f:
            for line in f:
                if got >= n:
                    break
                rec = json.loads(line)
                if not rec.get("passed"):
                    continue
                for t in rec.get("trials", []):
                    if t.get("passed") and t.get("code"):
                        out.append((f"{label}_{rec['problem_id']}", t["code"]))
                        got += 1
                        break
    return out


def load_or_build(*, rebuild: bool = False, n_per_file: Optional[int] = None) -> dict[str, dict]:
    """Per-sample stats (``{sample_id: {"name", "theorems"|"error"}}``), cached on disk.

    The Lean workspace is only created when at least one sample is uncached.
    """
    cache_dir = ROOT / CACHE_DIR / MINIF2F_CACHE_TAG
    cache_dir.mkdir(parents=True, exist_ok=True)
    samples = sample_proofs(n_per_file)

    results: dict[str, dict] = {}
    ws: Optional[Path] = None
    for i, (sample_id, code) in enumerate(samples, 1):
        cache_file = cache_dir / f"{sample_id}.json"
        if cache_file.is_file() and not rebuild:
            results[sample_id] = json.loads(cache_file.read_text())
            continue
        if ws is None:
            ws = ensure_workspace()
        print(f"[minif2f {i}/{len(samples)}] {sample_id}: extracting...", file=sys.stderr)
        result = process_source(sample_id, code, ws)
        cache_file.write_text(json.dumps(result))
        results[sample_id] = result
    return results
