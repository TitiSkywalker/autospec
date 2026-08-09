"""
Translate a Rust project into Lean using Charon + Aeneas.

Pipeline::

  Rust source → [simplify] → [flatten-loops] → charon → .llbc
  → [LLBC adapter] → aeneas → .lean → [F64→Rat + inline] → output

The original project is never modified — all transformations happen on a
temporary copy.
"""

from __future__ import annotations

import os
import shutil
import tempfile
from pathlib import Path

from pipeline.log import phase, fail, warn, set_verbose
from pipeline.simplify import simplify_for_aeneas
from pipeline.flatten import try_flatten_nested_loops
from pipeline.charon_aeneas import run_charon, run_adapter, run_aeneas
from pipeline.lean_post import write_lean_output, NonComputableError
from pipeline.extract_test import translate_test_file


class Translator:
    """Translate Rust projects to Lean via Charon + Aeneas."""

    def __init__(
        self,
        charon_bin: str | Path | None = None,
        aeneas_bin: str | Path | None = None,
        verbose: bool = False,
        target: str = "Rat",
        loops_to_rec: bool = False,
    ):
        _root = Path(__file__).resolve().parent
        self.charon = str(
            charon_bin or _root.parent / "aeneas" / "charon" / "bin" / "charon"
        )
        self.aeneas = str(aeneas_bin or _root.parent / "aeneas" / "bin" / "aeneas")
        self.verbose = verbose
        self.target = target
        self.loops_to_rec = loops_to_rec

    # ── public API ──────────────────────────────────────────────────────────

    def translate(
        self,
        project_dir: str | Path,
        output_dir: str | Path | None = None,
    ) -> bool:
        """Translate a Rust project to Lean.

        ``src/lib.rs`` (implementation) → ``<Crate>.lean``.
        ``tests/*.rs`` (concrete tests) → ``Test_<name>.lean``.
        """
        set_verbose(self.verbose)

        project_path = Path(project_dir).resolve()
        output_path = Path(output_dir).resolve() if output_dir else project_path
        output_path.mkdir(parents=True, exist_ok=True)

        if not (project_path / "Cargo.toml").exists():
            fail("validate", f"no Cargo.toml in {project_path}")
            return False

        print(f"autospec  \033[1m{project_path.name}\033[0m  →  {output_path}")

        results: list[tuple[str, str, int]] = []  # (tag, filename, def_count)

        # 1. Translate implementation
        try:
            outcome = self._translate_inner(
                project_path,
                output_path,
                flatten=False,
            )
            if outcome is None:
                warn("retrying with flatten-loops")
                outcome = self._translate_inner(
                    project_path,
                    output_path,
                    flatten=True,
                )
            if outcome is None:
                return False
        except NonComputableError as e:
            warn(f"{e}; auto-switching to --target Real")
            self.target = "Real"
            outcome = self._translate_inner(
                project_path,
                output_path,
                flatten=False,
                target="Real",
            )
            if outcome is None:
                warn("retrying with flatten-loops")
                outcome = self._translate_inner(
                    project_path,
                    output_path,
                    flatten=True,
                    target="Real",
                )
            if outcome is None:
                return False

        impl_defs, impl_files = outcome
        for name in sorted(impl_files):
            results.append(("impl", name, impl_defs))

        # 2. Translate each test file in tests/  (skip for Real: sqrt is
        #    noncomputable so #eval cannot execute anyway)
        if self.target != "Real":
            tests_dir = project_path / "tests"
            if tests_dir.is_dir():
                for test_file in sorted(tests_dir.glob("*.rs")):
                    test_defs = translate_test_file(
                        self._translate_inner,
                        project_path,
                        output_path,
                        test_file,
                        target=self.target,
                    )
                    if test_defs is not None:
                        results.append(
                            ("test", f"Test_{test_file.stem}.lean", test_defs)
                        )

        # Summary
        for tag, filename, count in results:
            phase(tag, f"{filename}  {count} defs")
        total_files = len(results)
        total_defs = sum(r[2] for r in results)
        phase("done", f"{total_files} file(s), {total_defs} defs")
        return True

    # ── core pipeline ───────────────────────────────────────────────────────

    def _translate_inner(
        self,
        project_path: Path,
        output_path: Path,
        *,
        flatten: bool,
        target: str | None = None,
    ) -> tuple[int, list[str]] | None:
        """Run the full charon→adapter→aeneas pipeline on a copy of the project.

        When *flatten* is True nested loops are extracted into separate
        functions *before* charon/aeneas (best-effort).

        *target* overrides ``self.target`` for this invocation only.

        Returns ``(def_count, written_filenames)``, or *None* on failure.
        """
        workdir = Path(tempfile.mkdtemp(prefix="autospec_"))
        try:
            shutil.copytree(
                project_path,
                workdir,
                dirs_exist_ok=True,
                ignore=shutil.ignore_patterns("target", "lean", "*.lean"),
            )

            if not (workdir / "Cargo.toml").exists():
                fail("copy", "target missing Cargo.toml")
                return None

            simplify_for_aeneas(workdir)

            if flatten:
                try_flatten_nested_loops(workdir, project_path)

            if not run_charon(self.charon, workdir):
                return None

            if run_adapter(workdir) is None:
                return None

            aeneas_warnings = run_aeneas(
                self.aeneas, workdir, loops_to_rec=self.loops_to_rec
            )
            if aeneas_warnings is None:
                return None

            def_count, written = write_lean_output(
                workdir,
                output_path,
                target=target if target is not None else self.target,
            )
            if def_count == 0:
                fail("output", "no definitions written")
                return None
            phase("output", f"{def_count} defs")

            for w in aeneas_warnings:
                warn(w)

            return def_count, written
        finally:
            shutil.rmtree(workdir, ignore_errors=True)


# ── CLI ───────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    p = argparse.ArgumentParser(description="Translate a Rust crate into Lean")
    p.add_argument("project_dir")
    p.add_argument("-o", "--output-dir", default=None)
    p.add_argument("-v", "--verbose", action="store_true", help="Print debug output")
    p.add_argument("--charon", default=None)
    p.add_argument("--aeneas", default=None)
    p.add_argument(
        "--target",
        default="Rat",
        choices=["Rat", "Real"],
        help="Replace f64 with Rat (default) or ℝ (computable sqrt)",
    )
    p.add_argument(
        "--loops-to-rec",
        action="store_true",
        help="Pass -loops-to-rec to aeneas: translate loops to recursive functions",
    )
    args = p.parse_args()

    t = Translator(
        charon_bin=args.charon,
        aeneas_bin=args.aeneas,
        verbose=args.verbose,
        target=args.target,
        loops_to_rec=args.loops_to_rec,
    )
    ok = t.translate(args.project_dir, args.output_dir)
    raise SystemExit(0 if ok else 1)
