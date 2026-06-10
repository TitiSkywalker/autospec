#!/usr/bin/env python3
"""autospec — Generate formal specifications for Rust projects from doc strings.

Usage:
    autospec PROJECT_PATH [--output DIR] [--model MODEL] [--max-iterations N] [--verbose]
    autospec PROJECT_PATH --split   # one agent per documented function

Examples:
    autospec ~/my-rust-lib
    autospec ~/my-rust-lib --output specs/ --model deepseek-v4-pro --verbose
    autospec ~/my-rust-lib --split --max-iterations 5
"""

import argparse
import os
import sys
from pathlib import Path

from pipeline.agents import _run_split
from pipeline.auto_fix import _auto_fix_checks
from pipeline.checks import _check_specs, _structural_check, _verify_specs
from pipeline.files import _merge_specs
from pipeline.generation import _get_spec_targets, _load_specs_toml
from pipeline.report import _generate_report
from pipeline.report_html import _generate_html_report
from pipeline.server import serve_report


def _api_key() -> str:
    """Return the API key, or exit with an error message."""
    key = os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_AUTH_TOKEN")
    if not key:
        print(
            "Error: ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN not set.", file=sys.stderr
        )
        sys.exit(1)
    return key


def _run_checks(
    project_path: str,
    output_dir: str,
    verify: bool = False,
    kani_timeout: int = 120,
    auto_fix_rounds: int = 0,
    api_key: str = "",
    base_url: str = "",
    model: str = "",
    max_tokens: int = 4096,
    verbose: bool = False,
) -> tuple[int, int, int] | None:
    """Run all spec validation checks.  Returns (passed, failed, timed_out) or None.

    If *auto_fix_rounds* > 0, compilation failures trigger fix agents and
    retry up to that many rounds.
    """
    print("=" * 60)
    print("Specification validation")
    print("=" * 60)

    # 1. Structural checks
    print("\n[1] Structural checks ...")
    warnings = _structural_check(output_dir)
    if warnings:
        for w in warnings:
            print(f"  WARN: {w}")
    else:
        print("  OK: no structural issues detected.")

    # 2. Compilation check (with optional auto-fix)
    if auto_fix_rounds > 0:
        compilation_ok = _auto_fix_checks(
            project_path=project_path,
            output_dir=output_dir,
            max_rounds=auto_fix_rounds,
            api_key=api_key,
            base_url=base_url,
            model=model,
            max_tokens=max_tokens,
            verbose=verbose,
        )
    else:
        print("\n[2] Compilation check ...")
        compilation_ok, _, _ = _check_specs(
            project_path, output_dir, return_errors=True
        )

    if compilation_ok:
        if verify:
            print(f"\n[3] Kani verification (timeout={kani_timeout}s) ...")
            _, passed, failed, timed_out = _verify_specs(
                project_path, output_dir, timeout=kani_timeout
            )
            return (passed, failed, timed_out)
    return None


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="autospec",
        description="Generate formal specifications for Rust projects from doc strings",
    )
    parser.add_argument(
        "project_path",
        nargs="?",
        help="Path to the Rust project directory (not required for --merge)",
    )
    parser.add_argument(
        "--output",
        "-o",
        default="specs",
        help="Directory to write specifications to (default: specs/)",
    )
    parser.add_argument(
        "--model",
        default="claude-sonnet-4-6-20250514",
        help="Anthropic model to use (default: claude-sonnet-4-6-20250514)",
    )
    parser.add_argument(
        "--max-iterations",
        type=int,
        default=15,
        help="Maximum tool-use iterations per agent (default: 15)",
    )
    parser.add_argument(
        "--max-tokens",
        type=int,
        default=4096,
        help="Maximum tokens for LLM output (default: 4096)",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Show verbose tool-call output",
    )
    parser.add_argument(
        "--jobs",
        "-j",
        type=int,
        default=5,
        help="Number of parallel agents for --split mode (default: 5)",
    )
    parser.add_argument(
        "--split",
        action="store_true",
        help="Generate specs with one agent per documented function (faster).",
    )
    parser.add_argument(
        "--check",
        "-c",
        action="store_true",
        help="After generation, compile specs against the original crate "
        "and run structural validation.",
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="Skip generation; only run checks on existing specs in --output.",
    )
    parser.add_argument(
        "--auto-fix",
        type=int,
        const=2,
        nargs="?",
        default=None,
        metavar="N",
        help="Auto-fix compilation errors with LLM fix agents, up to N rounds "
        "(default: 2 if --auto-fix is given without a value). "
        "Requires ANTHROPIC_API_KEY.",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Run full Kani model-checking verification.",
    )
    parser.add_argument(
        "--kani-timeout",
        type=int,
        default=120,
        help="Timeout in seconds for each Kani verification run (default: 120).",
    )
    parser.add_argument(
        "--report",
        action="store_true",
        help="Generate a structured report: doc coverage → specs → Kani results.",
    )
    parser.add_argument(
        "--report-html",
        action="store_true",
        help="Generate a static HTML report with per-function spec content "
        "and verification status, styled like cargo doc.",
    )
    parser.add_argument(
        "--only",
        type=str,
        default="",
        help="Comma-separated list of functions to generate specs for, "
        "e.g. 'geometry::Point::new,linalg::svd::SVD::decompose'. "
        "Also reads specs.toml from the output directory if present.",
    )
    parser.add_argument(
        "--crate",
        type=str,
        default="",
        help="Crate name within a workspace (e.g. 'p256', 'k256'). "
        "Required when target/doc/ has multiple .json files.",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Run 'cargo clean' before generating rustdoc JSON "
        "(ensures a fresh build, removes stale artifacts).",
    )
    parser.add_argument(
        "--serve",
        action="store_true",
        help="Start a local HTTP server for the HTML report with live "
        "specs.toml save API. Implies --report-html.",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8765,
        help="Port for --serve (default: 8765).",
    )
    parser.add_argument(
        "--merge",
        action="store_true",
        help="Merge per-function files into per-type bundled files under merged/ and exit.",
    )

    args = parser.parse_args()

    output = Path(args.output).resolve()

    # --merge: just merge files into per-type bundled output under merged/
    if args.merge:
        _merge_specs(str(output))
        return

    if not args.project_path:
        print("Error: project_path is required (except with --merge)", file=sys.stderr)
        sys.exit(1)

    project = Path(args.project_path).resolve()
    if not project.is_dir():
        print(f"Error: {project} is not a directory", file=sys.stderr)
        sys.exit(1)
    if not (project / "Cargo.toml").is_file():
        print(f"Error: Cargo.toml not found in {project}", file=sys.stderr)
        sys.exit(1)

    auto_fix = args.auto_fix or 0

    api_endpoint = (
        os.environ.get("ANTHROPIC_BASE_URL") or "(default: api.anthropic.com)"
    )
    print(f"Project:  {project}")
    print(f"Output:   {output}")
    print(f"Model:    {args.model}")
    print(f"API:      {api_endpoint}")
    wants_generation = args.split or (
        not args.check_only and auto_fix == 0 and not args.report_html
    )
    mode = []
    if args.split:
        mode.append("generate (split)")
    elif wants_generation and not args.check_only:
        mode.append("generate (monolithic)")
    if args.check:
        mode.append("check")
    if args.check_only and not (auto_fix > 0):
        mode.append("check-only")
    if auto_fix > 0:
        mode.append(f"auto-fix (x{auto_fix})")
    if args.verify:
        mode.append("verify")
    if args.report_html:
        mode.append("report-html")
    if args.serve:
        mode.append("serve")
    if not mode:
        mode.append("check")
    print(f"Mode:     {', '.join(mode)}")
    print()

    # --clean: run cargo clean before analysis
    if args.clean:
        import os as _os
        import subprocess

        print(f"  Cleaning target directory (cargo clean) ...")
        proc = subprocess.run(
            ["cargo", "clean"],
            cwd=str(project),
            capture_output=True,
            text=True,
            timeout=60,
            env={**_os.environ, "CARGO_TERM_COLOR": "never"},
        )
        if proc.returncode != 0:
            print(f"  Warning: cargo clean failed: {proc.stderr.strip()[-500:]}")
        else:
            print(f"  Cleaned {project}/target/")

    def _report_and_serve(vcounts=None):
        if args.report:
            _generate_report(
                str(project),
                str(output),
                verify_counts=vcounts,
                kani_timeout=args.kani_timeout,
            )
        if args.report_html or args.serve:
            _generate_html_report(
                str(project), str(output), crate_name=args.crate or None
            )
        if args.serve:
            serve_report(str(output), port=args.port)

    # API-only modes (no generation, no checks, no verify)
    wants_work = (
        args.split or args.check or args.check_only or auto_fix > 0 or args.verify
    )
    if not wants_work:
        _report_and_serve()
        return

    # --check-only: validate existing specs, no auto-fix
    if args.check_only:
        vcounts = _run_checks(
            str(project),
            str(output),
            verify=args.verify,
            kani_timeout=args.kani_timeout,
            auto_fix_rounds=0,
            model=args.model,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
        )
        _report_and_serve(vcounts)
        return

    # --auto-fix without --split: fix existing specs (implies check-only)
    if auto_fix > 0 and not args.split:
        api_key = _api_key()
        _run_checks(
            str(project),
            str(output),
            verify=args.verify,
            kani_timeout=args.kani_timeout,
            auto_fix_rounds=auto_fix,
            api_key=api_key,
            base_url=os.environ.get("ANTHROPIC_BASE_URL", ""),
            model=args.model,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
        )
        _report_and_serve()
        return

    # ── Below this point: generation mode ──
    api_key = _api_key()
    base_url = os.environ.get("ANTHROPIC_BASE_URL", "")

    # Build the set of functions to target (--only flag + specs.toml)
    only_functions: list[str] | None = None
    if args.only:
        only_functions = [f.strip() for f in args.only.split(",") if f.strip()]
        print(f"  --only: targeting {len(only_functions)} function(s)")
    # specs.toml overrides/adds to --only
    toml_funcs = _load_specs_toml(str(output))
    if toml_funcs is not None:
        if only_functions:
            only_functions = list(set(only_functions) | set(toml_funcs))
        else:
            only_functions = toml_funcs
        print(f"  specs.toml: targeting {len(only_functions)} function(s) total")

    if args.split:
        _run_split(
            project=project,
            output=output,
            api_key=api_key,
            base_url=base_url,
            model=args.model,
            max_iterations=args.max_iterations,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
            jobs=args.jobs,
            only_functions=only_functions,
        )
    else:
        from agent import SpecAgent

        agent = SpecAgent(
            project_path=str(project),
            output_dir=str(output),
            model=args.model,
            max_iterations=args.max_iterations,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
            api_key=api_key,
            base_url=base_url,
        )
        agent.run()

    print(f"\nDone. Output written to {output}/")

    # Run checks after generation if requested
    if args.check or (auto_fix > 0):
        vcounts = _run_checks(
            str(project),
            str(output),
            verify=args.verify,
            kani_timeout=args.kani_timeout,
            auto_fix_rounds=auto_fix,
            api_key=api_key,
            base_url=base_url,
            model=args.model,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
        )
    else:
        vcounts = None

    _report_and_serve(vcounts)


if __name__ == "__main__":
    main()
