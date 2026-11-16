"""
A single benchmark case and the operations around it.

``TestCase`` owns extracting a case from the benchmark, installing it into a
workspace, git-resetting that workspace afterward, and extracting the proved
result (``Spec.lean`` + vibe trace).
"""

import re
import shutil
from dataclasses import dataclass
from pathlib import Path

from evaluation.workspace import LIB
from utils.runner import CmdRunner

_EXCLUDED = {"failure", "lean_mode"}
_FUNC_STEMS = {"Spec", "Test_test", "Basic"}
_IMPORT_RE = re.compile(r"import LeanMode\.[^\n]+")


@dataclass
class TestCase:
    name: str
    func_src: str
    spec_src: str

    @classmethod
    def scan(cls, input_dir: Path) -> list["TestCase"]:
        """Extract every benchmark case under ``input_dir``."""
        cases = []
        for d in sorted(input_dir.iterdir()):
            if not d.is_dir() or d.name in _EXCLUDED:
                continue
            spec = d / "Spec.lean"
            funcs = [f for f in d.glob("*.lean") if f.stem not in _FUNC_STEMS]
            if spec.exists() and len(funcs) == 1:
                cases.append(cls(d.name, funcs[0].read_text(), spec.read_text()))
        return cases

    def install(self, ws: Path) -> None:
        """Write this case's function + spec into the workspace."""
        lib = ws / LIB
        (lib / "Function.lean").write_text(self.func_src)
        spec = _IMPORT_RE.sub(f"import {LIB}.Function", self.spec_src)
        (lib / "Spec.lean").write_text(spec)

    def reset(self, ws: Path) -> None:
        """Git-reset ``ws`` to its checkpoint, dropping this case's files."""
        runner = CmdRunner(cwd=ws)
        runner.run("git reset --hard HEAD", verbose=False, panic=False)
        runner.run("git clean -fd", verbose=False, panic=False)

    def extract(self, ws: Path, output_dir: Path, session: Path | None) -> str:
        """Archive the agent trace (if any) and return the proved ``Spec.lean`` text.

        ``session`` is a directory for vibe (``.vibe/sessions/...``) or a single
        transcript file for claude code (``*.jsonl``).
        """
        spec = ws / LIB / "Spec.lean"
        root = ws.resolve()
        if spec.is_symlink() or not spec.resolve().is_relative_to(root) or not spec.is_file():
            raise ValueError("agent spec must be a regular file inside its workspace")
        self.archive_trace(ws, output_dir, session)
        return spec.read_text()

    def archive_trace(self, ws: Path, output_dir: Path, session: Path | None) -> None:
        """Copy the agent ``session`` into ``output_dir/<case>/trace``.

        Also used for runs that end abnormally (turn cap, backend error), whose
        trace is the only record of what the agent did.
        """
        if session is None:
            return
        root = ws.resolve()
        if (
            session.is_symlink()
            or not session.resolve().is_relative_to(root)
            or (session.is_dir() and any(p.is_symlink() for p in session.rglob("*")))
        ):
            raise ValueError("agent trace contains an unsafe symlink")
        trace = output_dir / self.name / "trace"
        trace.parent.mkdir(parents=True, exist_ok=True)
        if trace.exists():
            shutil.rmtree(trace)
        if session.is_dir():
            shutil.copytree(session, trace)
        else:
            trace.mkdir(parents=True, exist_ok=True)
            shutil.copy2(session, trace / session.name)
