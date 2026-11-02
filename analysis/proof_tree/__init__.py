"""Proof-tree statistics for the agent-written proofs in RESULTS_DIR.

For each testcase (a ``Function.lean`` + ``Spec.lean`` pair, plus a
``verdict.txt`` from the independent verifier), this package rebuilds the proof
inside a reusable Lean workspace, runs the ``treant`` proof-tree analyzer on
``Spec.lean``, and reports proof-tree size/depth/premise-usage statistics
broken down by category (``array``/``crypto``/.../ prefix) and by verdict.

Usage::

    python3 -m analysis.proof_tree                   # analyze every case (cached)
    python3 -m analysis.proof_tree --rebuild          # ignore the cache
    python3 -m analysis.proof_tree --only crypto_rsa  # just one case, verbose
    python3 -m analysis.proof_tree --limit 10         # quick sample

    python3 -m analysis.proof_tree.distribution      # box-plot figure from cache

Modules, in dependency order:

* :mod:`.config`       — paths and knobs (``ROOT``, ``RESULTS_DIR``, ``CACHE_DIR``, ...).
* :mod:`.taxonomy`     — tactic / premise bucket definitions.
* :mod:`.stats`        — pure aggregation over cached per-theorem statistics.
* :mod:`.workspace`    — the reusable Lean workspace, build and extraction.
* :mod:`.plots`        — the three summary charts drawn at the end of a run.
* :mod:`.report`       — the terminal tables, then the charts.
* :mod:`.cli`          — argument parsing and the run loop.
* :mod:`.distribution` — box-plot figure built from the cache.
* :mod:`.local.dot_export` — debug tool: render one case's proof tree to .dot/.png.
"""

from analysis.proof_tree.config import (  # noqa: F401
    CACHE_DIR,
    DEFAULT_VERDICTS,
    LIB,
    OUT_DIR,
    RESULTS_DIR,
    ROOT,
    cache_dir_for,
)
from analysis.proof_tree.stats import (  # noqa: F401
    _avg,
    _case_summary,
    _category,
    _tree_stats,
)
from analysis.proof_tree.taxonomy import (  # noqa: F401
    ALL_TACTIC_BUCKETS,
    BUCKET_COLUMNS,
    PREMISE_BUCKETS,
    TACTIC_BUCKETS,
    _premise_bucket,
    _tactic_bucket,
    _tactic_buckets_from_histogram,
)
from analysis.proof_tree.workspace import (  # noqa: F401
    CaseError,
    _extract_theorems,
    _lake_build,
    discover_cases,
    ensure_workspace,
    process_case,
)
