#!/usr/bin/env python3
"""VibeAgent — thin wrapper around `vibe` CLI."""

import json
import os
import re
import subprocess
import time
from pathlib import Path


class VibeAgent:
    """Launch `vibe` as a subprocess, wait for completion, return results."""

    def __init__(
        self,
        repo_path: str | Path,
        instruction_file: str | Path = "instruction.md",
        results_dir: str | Path = "results",
        max_turns: int = 50,
        timeout: int = 7200,
        verbose: bool = False,
        cwd: str | Path | None = None,
        log_file: str | Path | None = None,
    ):
        self.repo_path = Path(repo_path).resolve()
        self.instruction = Path(instruction_file).read_text().strip()
        self.repo_name = self.repo_path.name
        self.results_dir = Path(results_dir).resolve() / self.repo_name
        self.max_turns = max_turns
        self.timeout = timeout
        self.verbose = verbose
        self.cwd = Path(cwd) if cwd else self.repo_path
        self.log_file = Path(log_file) if log_file else None

    # ── public API ────────────────────────────────────────────────────────

    def run(
        self,
        prompt: str = "",
        extra_args: list[str] | None = None,
    ) -> dict:
        """Execute vibe, wait for completion, return structured result.

        Returns dict with keys: ok, elapsed, output, turn_summary.
        """
        full_prompt = self.instruction
        full_prompt += (
            f"\n\nRepository: {self.repo_name}\n"
            f"Output directory: {self.results_dir}\n"
        )
        if prompt:
            full_prompt += "\n\n" + prompt

        cmd = [
            "vibe",
            "-p",
            full_prompt,
            "--agent",
            "auto-approve",
            "--max-turns",
            str(self.max_turns),
        ]
        if extra_args:
            cmd += extra_args
        if self.verbose:
            cmd += ["--output", "streaming"]

        t0 = time.time()
        output_lines: list[str] = []

        try:
            if self.verbose:
                output = self._run_streaming(cmd, output_lines)
            else:
                output = self._run_quiet(cmd, output_lines)
        except subprocess.TimeoutExpired:
            elapsed = time.time() - t0
            return {
                "ok": False,
                "elapsed": elapsed,
                "error": "timeout",
                "output": "\n".join(output_lines),
            }

        elapsed = time.time() - t0
        turn_summary = self._parse_turn_summary(output_lines)
        results = self.collect_results()

        return {
            "ok": True,
            "elapsed": elapsed,
            "output": output,
            "turn_summary": turn_summary,
            "results": results,
        }

    # ── private: subprocess runners ───────────────────────────────────────

    def _run_streaming(self, cmd: list[str], output_lines: list[str]) -> str:
        """Run with real-time output.

        If self.log_file is set, write all output there. Otherwise print to stdout.
        """
        all_output: list[str] = []
        log_fh = open(self.log_file, "w") if self.log_file else None

        proc = subprocess.Popen(
            cmd,
            cwd=self.cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            env={**os.environ, "PYTHONUNBUFFERED": "1"},
        )

        for line in proc.stdout:
            line_r = line.rstrip()
            if not line_r:
                continue
            output_lines.append(line_r)
            all_output.append(line_r)

            if log_fh:
                log_fh.write(line)
                log_fh.flush()
            else:
                try:
                    obj = json.loads(line_r) if line_r.startswith("{") else None
                except Exception:
                    obj = None

                if obj and isinstance(obj, dict):
                    role = obj.get("role", "")
                    if role == "tool":
                        self._print_tool_line(obj)
                    elif role == "assistant":
                        content = str(obj.get("content", ""))
                        if content.strip():
                            print(f"  [agent] {content[:200]}")
                elif line_r.strip():
                    ll = line_r.lower()
                    if any(
                        kw in ll
                        for kw in ["error", "fail", "pass", "test result", "turn ", "done"]
                    ):
                        print(f"  {line_r[:200]}")

        proc.wait(timeout=self.timeout)
        if log_fh:
            log_fh.close()
        return "\n".join(all_output)

    def _run_quiet(self, cmd: list[str], output_lines: list[str]) -> str:
        """Run with captured output."""
        r = subprocess.run(
            cmd,
            cwd=self.cwd,
            capture_output=True,
            text=True,
            timeout=self.timeout,
            env={**os.environ, "PYTHONUNBUFFERED": "1"},
        )
        output = r.stdout + "\n" + r.stderr
        output_lines.extend(output.split("\n"))
        return output

    # ── private: output parsing ───────────────────────────────────────────

    @staticmethod
    def _print_tool_line(obj: dict) -> None:
        """Print a compact one-line tool result."""
        content = str(obj.get("content", ""))
        tool_name = obj.get("name", "")

        # Sub-agent spawn
        if tool_name == "task":
            args_str = str(obj.get("arguments", ""))
            try:
                args = json.loads(args_str) if args_str else {}
            except Exception:
                args = {}
            sub_type = args.get("subagent_type", args.get("agent_type", "?"))
            sub_prompt = str(args.get("prompt", ""))
            preview = sub_prompt[:80].replace("\n", " ")
            print(f"  [task -> {sub_type}] {preview}...")
            return

        m = re.search(r"<(tool_error|tool_result)>(\S+)", content)
        if m:
            name = m.group(2)
            prefix = "ERR" if m.group(1) == "tool_error" else " OK"
            err_msg = content[:120].replace("\n", " ")
            print(f"  [{prefix}] {name}: {err_msg}")
        elif content.strip():
            first_line = content.strip().split("\n")[0][:120]
            print(f"  [tool] {first_line}")

    @staticmethod
    def _parse_turn_summary(lines: list[str]) -> dict:
        """Extract turn count and final status from output lines."""
        info = {"turns": 0, "final_status": "unknown"}
        for line in lines:
            if "turn " in line.lower() and "/" in line:
                m = re.search(r"turn\s+(\d+)/(\d+)", line, re.IGNORECASE)
                if m:
                    info["turns"] = int(m.group(1))
            if "test result: ok" in line.lower():
                info["final_status"] = "tests_passing"
            elif "test result:" in line.lower():
                m = re.search(r"test result:\s*(\S+)", line, re.IGNORECASE)
                if m:
                    info["final_status"] = m.group(1).lower()
        return info


    # ── result collection ────────────────────────────────────────────────

    def collect_results(self) -> dict:
        """Read manifest from results_dir/<repo>/manifest.txt.

        Returns dict with keys: functions, count, output_dir.
        """
        manifest = self.results_dir / "manifest.txt"
        functions: list[str] = []
        if manifest.exists():
            functions = [
                line.strip()
                for line in manifest.read_text().splitlines()
                if line.strip()
            ]

        return {
            "functions": functions,
            "count": len(functions),
            "output_dir": str(self.results_dir),
        }


# ── CLI ────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Run Vibe agent on a repo")
    parser.add_argument("repo", help="Path to the repo")
    parser.add_argument("--max-turns", type=int, default=50)
    parser.add_argument("--timeout", type=int, default=7200)
    parser.add_argument("-v", "--verbose", action="store_true")
    parser.add_argument(
        "--prompt", default="", help="Extra prompt appended to instructions"
    )
    parser.add_argument(
        "--results-dir", default="results", help="Root directory for per-repo results (default: results/)"
    )
    args = parser.parse_args()

    agent = VibeAgent(
        args.repo,
        results_dir=args.results_dir,
        max_turns=args.max_turns,
        timeout=args.timeout,
        verbose=args.verbose,
    )
    result = agent.run(prompt=args.prompt)

    r = result.get("results", {})
    print(f"\n{'='*50}")
    print(f"Repo:    {agent.repo_name}")
    print(f"Elapsed: {result['elapsed']:.0f}s")
    print(f"Turns:   {result['turn_summary'].get('turns', '?')}")
    print(f"Status:  {result['turn_summary'].get('final_status', '?')}")
    print(f"Success: {result['ok']}")
    if r:
        print(f"\nFunctions extracted: {r.get('count', 0)}")
        for sig in r.get("functions", []):
            print(f"  - {sig}")
        print(f"\nOutput: {r.get('output_dir')}")
