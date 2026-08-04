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

from pipeline.log import phase, fail, warn
from pipeline.simplify import simplify_for_aeneas
from pipeline.flatten import try_flatten_nested_loops
from pipeline.charon_aeneas import run_charon, run_adapter, run_aeneas
from pipeline.lean_post import write_lean_output
from pipeline.extract_test import translate_test_file


class Translator:
    """Translate Rust projects to Lean via Charon + Aeneas."""

    def __init__(
        self,
        charon_bin: str | Path | None = None,
        aeneas_bin: str | Path | None = None,
    ):
        _root = Path(__file__).resolve().parent
        self.charon = str(
            charon_bin or _root.parent / "aeneas" / "charon" / "bin" / "charon"
        )
        self.aeneas = str(
            aeneas_bin or _root.parent / "aeneas" / "bin" / "aeneas"
        )

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
        project_path = Path(project_dir).resolve()
        output_path = Path(output_dir).resolve() if output_dir else project_path
        output_path.mkdir(parents=True, exist_ok=True)

        print(f"autospec  \033[1m{project_path.name}\033[0m  →  {output_path}")

        results: list[tuple[str, str, int]] = []  # (tag, filename, def_count)

        # 1. Translate implementation
        outcome = self._translate_inner(
            project_path, output_path, flatten=False,
        )
        if outcome is None:
            warn("retrying with flatten-loops")
            outcome = self._translate_inner(
                project_path, output_path, flatten=True,
            )
        if outcome is None:
            return False

        impl_defs, impl_files = outcome
        for name in sorted(impl_files):
            results.append(("impl", name, impl_defs))

        # 2. Translate each test file in tests/
        tests_dir = project_path / "tests"
        if tests_dir.is_dir():
            for test_file in sorted(tests_dir.glob("*.rs")):
                test_defs = translate_test_file(
                    self._translate_inner,
                    project_path,
                    output_path,
                    test_file,
                )
                if test_defs is not None:
                    results.append(
                        ("test", f"Test_{test_file.stem}.lean", test_defs)
                    )

        # Summary
        for tag, filename, count in results:
            phase(f"  {tag}", f"{filename}  {count} defs")
        total_files = len(results)
        total_defs = sum(r[2] for r in results)
        phase("✓ done", f"{total_files} file(s), {total_defs} defs")
        return True

    # ── core pipeline ───────────────────────────────────────────────────────

    def _translate_inner(
        self,
        project_path: Path,
        output_path: Path,
        *,
        flatten: bool,
    ) -> tuple[int, list[str]] | None:
        """Run the full charon→adapter→aeneas pipeline on a copy of the project.

        When *flatten* is True nested loops are extracted into separate
        functions *before* charon/aeneas (best-effort).

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

            simplify_for_aeneas(workdir)

            if flatten:
                try_flatten_nested_loops(workdir, project_path)

            if not run_charon(self.charon, workdir):
                return None

            if run_adapter(workdir) is None:
                return None

            aeneas_warnings = run_aeneas(self.aeneas, workdir)
            if aeneas_warnings is None:
                return None

            def_count, written = write_lean_output(workdir, output_path)
            phase("  output", f"{def_count} defs")
            if def_count == 0:
                fail("output", "no definitions written")
                return None

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
    p.add_argument("--charon", default=None)
    p.add_argument("--aeneas", default=None)
    args = p.parse_args()

    t = Translator(charon_bin=args.charon, aeneas_bin=args.aeneas)
    ok = t.translate(args.project_dir, args.output_dir)
    raise SystemExit(0 if ok else 1)
