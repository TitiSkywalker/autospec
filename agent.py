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
        max_turns: int = 50,
        timeout: int = 7200,
        verbose: bool = False,
        cwd: str | Path | None = None,
    ):
        self.repo_path = Path(repo_path).resolve()
        self.instruction = Path(instruction_file).read_text().strip()
        self.max_turns = max_turns
        self.timeout = timeout
        self.verbose = verbose
        self.cwd = Path(cwd) if cwd else self.repo_path

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
        if prompt:
            full_prompt += "\n\n" + prompt

        cmd = [
            "vibe",
            "-p",
            full_prompt,
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

        return {
            "ok": True,
            "elapsed": elapsed,
            "output": output,
            "turn_summary": turn_summary,
        }

    # ── private: subprocess runners ───────────────────────────────────────

    def _run_streaming(self, cmd: list[str], output_lines: list[str]) -> str:
        """Run with real-time output, parse JSON tool results for compact display."""
        all_output: list[str] = []

        proc = subprocess.Popen(
            cmd,
            cwd=self.cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            env={**os.environ, "PYTHONUNBUFFERED": "1"},
        )

        for line in proc.stdout:
            line = line.rstrip()
            if not line:
                continue
            output_lines.append(line)
            all_output.append(line)

            try:
                obj = json.loads(line) if line.startswith("{") else None
            except Exception:
                obj = None

            if obj and isinstance(obj, dict):
                role = obj.get("role", "")
                if role == "tool":
                    self._print_tool_line(obj)
                elif role == "assistant":
                    content = str(obj.get("content", ""))
                    if content.strip():
                        print(f"  🤖 {content[:200]}")
            elif line.strip():
                ll = line.lower()
                if any(
                    kw in ll
                    for kw in ["error", "fail", "pass", "test result", "turn ", "done"]
                ):
                    print(f"  {line[:200]}")

        proc.wait(timeout=self.timeout)
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
        m = re.search(r"<(tool_error|tool_result)>(\S+)", content)
        if m:
            tool_name = m.group(2)
            prefix = "✗" if m.group(1) == "tool_error" else "•"
            err_msg = content[:120].replace("\n", " ")
            print(f"  [{tool_name}] {prefix} {err_msg}")
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
    args = parser.parse_args()

    agent = VibeAgent(
        args.repo,
        max_turns=args.max_turns,
        timeout=args.timeout,
        verbose=args.verbose,
    )
    result = agent.run(prompt=args.prompt)

    print(f"\n{'='*50}")
    print(f"Elapsed: {result['elapsed']:.0f}s")
    print(f"Turns:   {result['turn_summary'].get('turns', '?')}")
    print(f"Status:  {result['turn_summary'].get('final_status', '?')}")
    print(f"Success: {result['ok']}")
