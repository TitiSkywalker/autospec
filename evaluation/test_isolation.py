"""Run with `python -m unittest evaluation.test_isolation`."""

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, Mock

from evaluation.config import AgentConfig, McpConfig
from evaluation.isolation import SandboxConfig, sandbox_command
from evaluation.prover import Prover
from evaluation.testcase import TestCase


@unittest.skipUnless(shutil.which("bwrap"), "bubblewrap not installed")
class SandboxTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.ws = self.root / "workers" / "ws-0"
        self.ws.mkdir(parents=True)
        self.results = self.root / "results"
        self.results.mkdir()
        (self.results / "solved.txt").write_text("secret proof")
        (self.ws / "escape").symlink_to(self.results, target_is_directory=True)
        (self.ws / "Spec.lean").write_text("theorem current : True := by trivial")
        self.python = "/usr/bin/python3"

    def launch(self, script, *, config=None, home_name=".agent-home"):
        argv, env = sandbox_command(
            [self.python, "-c", script], self.ws,
            config or SandboxConfig(), [self.results], home_name=home_name,
        )
        return subprocess.run(
            argv, cwd=self.ws, env={**os.environ, **env},
            capture_output=True, text=True, timeout=15,
        )

    def test_can_read_and_write_current_workspace(self):
        result = self.launch(
            "from pathlib import Path; "
            "assert Path('Spec.lean').exists(); "
            "Path('proof.txt').write_text('ok')"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.ws / "proof.txt").read_text(), "ok")

    def test_cannot_read_results_or_follow_workspace_symlink(self):
        result = self.launch(
            "from pathlib import Path; "
            f"assert not Path({str(self.results / 'solved.txt')!r}).exists(); "
            "assert not Path('escape/solved.txt').exists()"
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_home_is_private_and_kept_inside_workspace(self):
        result = self.launch(
            "import os; from pathlib import Path; "
            "h = Path.home(); assert h == Path(os.environ['HOME']); "
            "assert not (h / '.codex/sessions/old').exists(); "
            "(h / 'session.txt').write_text('current')"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.ws / ".agent-home/session.txt").read_text(), "current")

    def test_individual_home_file_is_read_only(self):
        host_home = self.root / "runner-home"
        (host_home / ".codex").mkdir(parents=True)
        (host_home / ".codex" / "auth.json").write_text("credentials")
        with patch("evaluation.isolation.Path.home", return_value=host_home):
            result = self.launch(
                "from pathlib import Path; "
                "p = Path.home() / '.codex/auth.json'; "
                "assert p.read_text() == 'credentials'; "
                "import errno\n"
                "try:\n p.write_text('changed')\nexcept OSError as e:\n"
                " assert e.errno in (errno.EACCES, errno.EROFS)\n"
                "else:\n raise AssertionError('credential mount is writable')\n",
                config=SandboxConfig(home_files=[".codex/auth.json"]),
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((host_home / ".codex/auth.json").read_text(), "credentials")

    def test_mcp_home_cannot_read_agent_credentials(self):
        host_home = self.root / "runner-home"
        (host_home / ".codex").mkdir(parents=True)
        (host_home / ".codex/auth.json").write_text("credentials")
        with patch("evaluation.isolation.Path.home", return_value=host_home):
            argv, env = sandbox_command(
                [self.python, "-c",
                 "from pathlib import Path; "
                 "assert not (Path.home() / '.codex/auth.json').exists()"],
                self.ws, SandboxConfig(home_files=[".codex/auth.json"]),
                [self.results], home_name=".mcp-home",
            )
            result = subprocess.run(
                argv, cwd=self.ws, env={**os.environ, **env},
                capture_output=True, text=True, timeout=15,
            )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_mcp_home_gets_writable_cache_and_read_only_config(self):
        host_home = self.root / "runner-home"
        (host_home / ".cache" / "uv").mkdir(parents=True)
        (host_home / ".config" / "uv").mkdir(parents=True)
        (host_home / ".config" / "uv" / "uv.toml").write_text("index-url = 'mirror'")
        with patch("evaluation.isolation.Path.home", return_value=host_home):
            result = self.launch(
                "import errno, os; from pathlib import Path; "
                "cache = Path(os.environ['HOME']) / '.cache/uv'; "
                "cache.mkdir(parents=True, exist_ok=True); "
                "(cache / 'lock').write_text('writable'); "
                "cfg = Path(os.environ['HOME']) / '.config/uv/uv.toml'; "
                "assert cfg.read_text() == \"index-url = 'mirror'\"; "
                "assert (Path(os.environ['HOME']) / '.cache/uv/lock').exists()",
                config=SandboxConfig(),
                home_name=".mcp-home",
            )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_mcp_cache_is_not_visible_to_the_agent(self):
        host_home = self.root / "runner-home"
        (host_home / ".cache" / "uv").mkdir(parents=True)
        with patch("evaluation.isolation.Path.home", return_value=host_home):
            result = self.launch(
                "import os; from pathlib import Path; "
                "assert not (Path(os.environ['HOME']) / '.cache/uv').exists()"
            )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_unsafe_mounts_and_missing_executable(self):
        with self.assertRaisesRegex(ValueError, "unsafe"):
            self.launch("pass", config=SandboxConfig(ro_paths=[self.root]))
        tool = self.root / "custom-tool"
        tool.write_text("#!/bin/sh\nexit 0\n")
        tool.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "not visible"):
            sandbox_command(
                [str(tool)], self.ws, SandboxConfig(),
                [self.results],
            )


class FreshWorkspaceTests(unittest.TestCase):
    def test_next_case_does_not_see_ignored_files_from_previous_case(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            base = root / "base"
            base.mkdir()
            (base / "Spec.lean").write_text("initial")
            prover = Prover(
                base, root / "workers", 0,
                AgentConfig("unused", "", [], 10),
                McpConfig([], "streamable-http", "127.0.0.1", 3180, 10),
                root / "results", sandbox=SandboxConfig(),
            )
            shutil.copytree(base, prover.ws)
            prover._build = lambda name: True
            prover._run_agent = lambda name: None
            prover._stop_mcp = Mock()
            prover._start_mcp = Mock()

            class Case:
                def __init__(self, name):
                    self.name = name

                def install(self, ws):
                    if self.name == "second":
                        assert not (ws / ".lake" / "previous-proof").exists()
                    (ws / "Spec.lean").write_text(self.name)

                def extract(self, ws, output, session):
                    (ws / ".lake").mkdir(exist_ok=True)
                    (ws / ".lake" / "previous-proof").write_text(self.name)
                    return (ws / "Spec.lean").read_text()

            def copy_base(workspace_path, **kwargs):
                shutil.copytree(base, workspace_path)

            self.assertEqual(prover.prove(Case("first"))[2], "ok")
            with patch("evaluation.prover.create_workspace", side_effect=copy_base):
                self.assertEqual(prover.prove(Case("second"))[2], "ok")


class ExtractionTests(unittest.TestCase):
    def test_parent_does_not_follow_agent_symlinks(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            ws = root / "ws"
            (ws / "RustVerif").mkdir(parents=True)
            secret = root / "solved.txt"
            secret.write_text("solved proof")
            spec = ws / "RustVerif" / "Spec.lean"
            spec.symlink_to(secret)
            case = TestCase("current", "", "")
            with self.assertRaisesRegex(ValueError, "regular file"):
                case.extract(ws, root / "results", None)
            spec.unlink()
            spec.write_text("current")
            trace = ws / ".agent-home" / "trace"
            trace.mkdir(parents=True)
            (trace / "stolen").symlink_to(secret)
            with self.assertRaisesRegex(ValueError, "unsafe symlink"):
                case.extract(ws, root / "results", trace)


if __name__ == "__main__":
    unittest.main()
