"""Two-tier difficulty split of the benchmark (by code complexity).

``python3 -m analysis tiers`` splits the benchmark into a
*beginner* tier and an *advanced* tier on the ``rust + lean`` line count of each
testcase (the program plus its Aeneas translation), then reports the PROVED
accuracy and the Rust / Lean / Proof line lengths of each tier. The line
lengths are the evidence that the two tiers differ in complexity: the advanced
tier should be longer in all three of Rust (the algorithm), Lean (its formal
translation) and Proof (the agent's ``Spec.lean`` output). Every run gets an accuracy column, over the same tier
assignment.

The split is **hand-tuned and hard-coded** in :data:`TIER1` (93 cases) and
:data:`TIER2` (66 cases): within each category, cases are cut at a gap in the
sorted ``rust + lean`` sizes (see :data:`CUT_RUST_LEAN`). :func:`assign` is reused by
:mod:`analysis.accuracy.tier_radar` and fails loudly if the benchmark gains or
loses a case, so the split has to be extended deliberately.
"""

import statistics
from collections import defaultdict
from pathlib import Path

from analysis.accuracy.runs import Context
from analysis.accuracy.verdicts import category, collect

_BENCHMARK_FILES_IGNORED = ("Spec", "Test_test")


def _lines(path: Path) -> int:
    try:
        return len(path.read_text().splitlines())
    except OSError:
        return 0


def sizes(benchmark: Path) -> dict[str, int]:
    """``{case: complexity line count}`` — the ``rust + lean`` size of each case."""
    out = {}
    for d in sorted(Path(benchmark).iterdir()):
        if not d.is_dir() or d.name == "failure" or not (d / "Cargo.toml").is_file():
            continue
        rust = sum(_lines(f) for f in (d / "src").glob("*.rs"))
        lean = sum(
            _lines(f) for f in d.glob("*.lean") if f.stem not in _BENCHMARK_FILES_IGNORED
        )
        out[d.name] = rust + lean
    return out


# Hand-tuned split, curated 2026-11-02 against the then-current benchmark.
# Within every category, all cases above one cut go to TIER2 and the rest to
# TIER1, so each category splits into a single clean block (no interleaving).
# The cuts were picked so the TIER2 count is round in every category;
# CUT_RUST_LEAN records each cut as (largest TIER1 case, smallest TIER2 case)
# in lines of ``rust + lean``. Cuts differ by category (95-1058 lines), so the
# tiers are not a global size cut: each category keeps its own difficulty
# scale. Categories not listed (integer, tensor) are entirely TIER1.
CUT_RUST_LEAN = {
    "array": (173, 185),
    "crypto": (186, 216),
    "dp": (242, 266),
    "ds": (470, 499),
    "graph": (405, 428),
    "matrix": (410, 606),
    "string": (490, 559),
}

TIER1 = frozenset({
    "array_bloom_filter", "array_bubble_sort", "array_floyd_cycle", "array_fnv",
    "array_gram_schmidt", "array_permutations", "array_two_sum", "crypto_affine", "crypto_dh",
    "crypto_lwe", "crypto_rsa", "crypto_tea", "dp_coin_change", "dp_domino_tiling",
    "dp_edit_distance", "dp_house_robber", "dp_interval_dp", "dp_knapsack", "dp_lis",
    "dp_longest_common_subsequence", "dp_matrix_path", "dp_min_triangulation",
    "dp_optimal_bst", "dp_palindrome_partition", "ds_bst", "ds_cartesian_tree", "ds_deque",
    "ds_kdtree", "ds_max_heap", "ds_queue", "ds_segment_tree", "ds_sparse_table", "ds_trie",
    "ds_union_find", "graph_bellman_ford", "graph_bipartite_check", "graph_chordal_mcs",
    "graph_dijkstra", "graph_erdos_gallai", "graph_floyd_warshall", "graph_kruskal",
    "graph_matrix_tree", "graph_max_flow", "graph_minimum_cycle", "graph_negative_cycle",
    "graph_prim", "graph_scc_tarjan", "graph_stable_match", "graph_stoer_wagner",
    "graph_topological_sort", "graph_tutte", "integer_crt", "integer_div_mod",
    "integer_euler_totient", "integer_extended_gcd", "integer_mod_pow", "integer_moebius",
    "integer_nth_root", "integer_prime_check", "integer_prime_factors", "integer_prime_sieve",
    "matrix_charpoly", "matrix_cholesky", "matrix_det", "matrix_ldlt", "matrix_lu",
    "matrix_mul", "matrix_permanent", "matrix_pow", "matrix_rank", "matrix_solve",
    "string_aho_corasick", "string_debruijn", "string_kmp", "string_lyndon_array",
    "string_lz77", "string_manacher", "string_min_rotation", "string_myers_bits",
    "string_order_preserving_match", "string_palindromic_tree", "string_parameterized_match",
    "string_rabin_karp", "string_runs", "string_suffix_automaton", "string_two_way",
    "string_ukkonen", "string_z_algorithm", "tensor_convolution_2d", "tensor_cross_entropy",
    "tensor_layer_norm", "tensor_max_pool", "tensor_sigmoid",
})

TIER2 = frozenset({
    "string_lz_end", "ds_treap", "dp_held_karp", "array_interval_scheduling",
    "array_merge_sort", "ds_avl", "dp_digit_dp", "ds_binomial_heap", "graph_dinic",
    "graph_bigraph_weight_match", "matrix_hessenberg", "matrix_bidiagonal",
    "graph_general_match", "ds_van_emde_boas", "matrix_sparse_cholesky",
    "dp_chromatic_number", "matrix_qr", "crypto_threefish256", "string_sais",
    "string_generalized_sam", "array_median_of_medians", "graph_lgv_lemma", "string_maw",
    "graph_hopcroft_karp", "graph_brooks", "array_closest_pair", "dp_monotone_queue",
    "dp_hamiltonian_plug", "string_fm_index", "matrix_inverse", "dp_subset_convolution",
    "matrix_lblt", "ds_lct", "array_quick_sort", "ds_seg_beats", "crypto_des128",
    "graph_gomory_hu", "graph_chu_liu_edmonds", "matrix_sparse_cholesky_symbolic",
    "graph_min_cost_flow", "array_open_addressing", "ds_fibonacci_heap",
    "graph_rooted_tree_isomorphism", "graph_tree_decomposition", "string_suffix_array",
    "matrix_sparse_lu", "matrix_jordan", "ds_wblt", "string_bwt_inverse", "array_convex_hull",
    "graph_dilworth", "dp_steiner_tree", "crypto_hill", "graph_planarity", "crypto_aes128",
    "graph_matroid_intersection", "matrix_smith_normal_form", "string_lzw",
    "array_separate_chaining", "crypto_mpc_gc", "ds_red_black_tree",
    "graph_weighted_matching", "graph_flow_bound", "ds_splay_tree", "graph_min_mean_cycle",
    "dp_count_hamiltonian",
})

assert not TIER1 & TIER2 and len(TIER1) == 93 and len(TIER2) == 66


def assign(benchmark: Path) -> dict[str, int]:
    """``{case: 1 | 2}`` from the hard-coded TIER1/TIER2 sets.

    A new benchmark case that is in neither set is an error rather than being
    silently bucketed: the split is a curated artefact and has to be extended
    by hand when the benchmark grows.
    """
    cases = sizes(benchmark)
    missing = set(cases) - TIER1 - TIER2
    stale = (TIER1 | TIER2) - set(cases)
    if missing or stale:
        raise SystemExit(
            f"tier split is out of date: missing from TIER1/TIER2: {sorted(missing)}; "
            f"listed but no longer in the benchmark: {sorted(stale)}"
        )
    return {name: 1 if name in TIER1 else 2 for name in cases}


def _reference(verdicts: dict[str, dict[str, str]]) -> str:
    """The run whose ``Spec.lean`` supplies the proof-length column: the one
    with the most PROVED cases, so proof lengths cover as many cases as possible."""
    return max(verdicts, key=lambda l: sum(v == "PROVED" for v in verdicts[l].values()))


def _case_rows(ctx: Context) -> list[dict]:
    """One row per testcase: {name, rust, lean, spec, proof, code, verdicts}."""
    lean_mode = ctx.benchmark
    verdicts = {label: collect(path) for label, path in ctx.runs.items()}
    ref = ctx.runs[_reference(verdicts)]
    rows = []
    for d in sorted(lean_mode.iterdir()):
        if not d.is_dir() or d.name == "failure" or not (d / "Cargo.toml").exists():
            continue
        rust = sum(_lines(f) for f in (d / "src").glob("*.rs"))
        lean = sum(
            _lines(f) for f in d.glob("*.lean") if f.stem not in _BENCHMARK_FILES_IGNORED
        )
        rows.append(
            {
                "name": d.name,
                "rust": rust,
                "lean": lean,
                "spec": _lines(d / "Spec.lean"),
                "proof": _lines(ref / d.name / "Spec.lean"),
                "code": rust + lean,
                "verdicts": {label: v.get(d.name, "UNKNOWN") for label, v in verdicts.items()},
            }
        )
    return rows


def _accuracy(labels, beginner: list[dict], advanced: list[dict]) -> None:
    """PROVED / total per agent, once per tier and overall."""
    print(f"{'agent':<22} {'beginner':>16} {'advanced':>16} {'overall':>16}")
    print("-" * 70)
    for label in labels:
        out = f"{label:<22}"
        for rows in (beginner, advanced, beginner + advanced):
            proved = sum(1 for r in rows if r["verdicts"][label] == "PROVED")
            pct = proved / len(rows) if rows else 0.0
            out += f"{f'{proved}/{len(rows)} ({pct:5.1%})':>16}"
        print(out)


def _category_table(labels, beginner: list[dict], advanced: list[dict]) -> None:
    """Per category and tier, the solved/total of every agent."""
    cats = defaultdict(lambda: ([], []))  # category -> (beginner rows, advanced rows)
    for rows, tier in ((beginner, 0), (advanced, 1)):
        for r in rows:
            cats[category(r["name"])][tier].append(r)

    for c in sorted(cats):
        print(f"\n{c}")
        print(f"  {'agent':<22} {'beginner':>16} {'advanced':>16}")
        for label in labels:
            out = f"  {label:<22}"
            for rows in cats[c]:
                proved = sum(1 for r in rows if r["verdicts"][label] == "PROVED")
                pct = proved / len(rows) if rows else 0.0
                out += f"{f'{proved}/{len(rows)} ({pct:5.1%})':>16}"
            print(out)


def _tier_stats(rows: list[dict]) -> dict:
    def m(key):
        vals = [r[key] for r in rows]
        return statistics.mean(vals), statistics.median(vals)

    return {
        "n": len(rows),
        "rust": m("rust"),
        "lean": m("lean"),
        "proof": m("proof"),
        "spec": m("spec"),
        "code": m("code"),
    }


def main(ctx: Context) -> None:
    tiers = assign(ctx.benchmark)
    rows = _case_rows(ctx)
    beginner = [r for r in rows if tiers[r["name"]] == 1]
    advanced = [r for r in rows if tiers[r["name"]] == 2]

    b, a = _tier_stats(beginner), _tier_stats(advanced)

    print("tier cuts per category (largest TIER1 | smallest TIER2, rust+lean lines):")
    print("  " + " | ".join(f"{c} {a}/{b}" for c, (a, b) in sorted(CUT_RUST_LEAN.items())))
    print("  (integer, tensor: entirely TIER1)")
    print()
    print(f"{'':<18} {'beginner':>22} {'advanced':>22}")
    print(f"{'testcases':<18} {b['n']:>22} {a['n']:>22}")
    for label, key in (
        ("rust lines (mean/med)", "rust"),
        ("lean lines (mean/med)", "lean"),
        ("proof lines (mean/med)", "proof"),
        ("spec lines (mean/med)", "spec"),
        ("code lines (mean/med)", "code"),
    ):
        bm, bd = b[key]
        am, ad = a[key]
        print(f"{label:<18} {bm:>10.0f}/{bd:<6.0f}  {am:>10.0f}/{ad:<6.0f}")

    print(
        "tiers are an agent-independent split; proof lines use the Spec.lean of "
        f"{_reference({l: collect(p) for l, p in ctx.runs.items()})} (the run with most PROVED)"
    )

    print()
    _accuracy(list(ctx.runs), beginner, advanced)

    print()
    _category_table(list(ctx.runs), beginner, advanced)

    print()
    print("tiers:")
    print("  beginner:", ", ".join(r['name'] for r in sorted(beginner, key=lambda r: r['code'])))
    print("  advanced:", ", ".join(r['name'] for r in sorted(advanced, key=lambda r: r['code'])))

