"""Auto-fix pipeline — feed compilation errors back to fix agents."""

import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from pipeline.agents import _run_fix_agent
from pipeline.checks import _check_specs, _parse_compilation_errors


def _auto_fix_checks(
    project_path: str,
    output_dir: str,
    max_rounds: int,
    api_key: str,
    base_url: str,
    model: str,
    max_tokens: int,
    verbose: bool,
) -> bool:
    """Run compilation check with auto-fix retry loop.

    Each individual harness file is compiled as a separate module, so
    ``harness_N`` maps 1:1 to the Nth individual file.  Fix agents write
    directly to ``individual/`` — no merge needed in the hot path.

    Returns True if compilation passes (on first try or after fixing).
    """
    import threading
    from concurrent.futures import ThreadPoolExecutor, as_completed

    for round_num in range(1, max_rounds + 1):
        if round_num == 1:
            print(f"\n[2] Compilation check (round {round_num}/{max_rounds}) ...")
        else:
            print(f"\n[2] Compilation check (retry round {round_num}/{max_rounds}) ...")

        ok, stderr, file_stems = _check_specs(
            project_path, output_dir, return_errors=True
        )

        if ok:
            print(f"  OK: compilation passed.")
            return True

        # Parse errors — harness_N maps 1:1 to individual file stem
        errors_by_file = _parse_compilation_errors(stderr, file_stems)
        if not errors_by_file:
            print("  Could not parse compilation errors from stderr.")
            return False

        total_errors = sum(len(v) for v in errors_by_file.values())
        print(f"  Found {total_errors} error(s) in {len(errors_by_file)} file(s):")
        for stem, errs in errors_by_file.items():
            codes = sorted(set(e["code"] for e in errs))
            print(f"    {stem}.rs: {len(errs)} error(s) {codes}")

        if round_num == max_rounds:
            print(f"  FAIL: compilation still broken after {max_rounds} round(s).")
            return False

        # Build fix tasks directly — no reverse-mapping needed
        fix_tasks: list[tuple[str, list[dict]]] = [
            (f"{stem}.rs", errs) for stem, errs in errors_by_file.items()
        ]

        print(f"\n  Running fix agents for {len(fix_tasks)} file(s)...")

        print_lock = threading.Lock()
        fixed = 0
        fix_failed = 0

        def run_fix(indiv_fn, errs):
            with print_lock:
                print(f"    Fixing {indiv_fn} ...")
            indiv_path = Path(output_dir) / indiv_fn
            if not indiv_path.is_file():
                return indiv_fn, False
            content = indiv_path.read_text()
            ok_fix = _run_fix_agent(
                project_path=project_path,
                output_dir=output_dir,
                indiv_filename=indiv_fn,
                file_content=content,
                errors=errs,
                api_key=api_key,
                base_url=base_url,
                model=model,
                max_tokens=max_tokens,
                verbose=verbose,
            )
            return indiv_fn, ok_fix

        workers = min(len(fix_tasks), 5)
        with ThreadPoolExecutor(max_workers=workers) as pool:
            futures = {pool.submit(run_fix, ifn, errs): ifn for ifn, errs in fix_tasks}
            for future in as_completed(futures):
                ifn, ok_fix = future.result()
                with print_lock:
                    if ok_fix:
                        fixed += 1
                        print(
                            f"    [{fixed + fix_failed}/{len(fix_tasks)}] {ifn}  FIXED"
                        )
                    else:
                        fix_failed += 1
                        print(
                            f"    [{fixed + fix_failed}/{len(fix_tasks)}] {ifn}  FAILED"
                        )

    return False
