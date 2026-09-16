"""
Proving-agent backends.

The pipeline invokes whichever CLI agent is configured, streams its stdout, and
archives its session trace. Each backend differs in three places — the command
line, the stdout JSON schema, and the on-disk trace location — so they are
hidden behind one small interface, mirroring algoveri's ``ProviderBase`` (one
subclass per backend, a factory to pick one).

``parse_stream_line`` maps one stdout line to a normalized :class:`StreamEvent`
that :class:`~evaluation.prover.Prover` turns into progress + final-answer
logging; ``sessions`` returns the trace candidates a run may have produced.
"""

import json
import re
from dataclasses import dataclass
from pathlib import Path

from evaluation.config import AgentConfig, McpConfig


@dataclass
class StreamEvent:
    """One normalized event parsed from an agent's live stdout stream."""

    step: bool = False          # counts as one unit of work (a tool call / turn)
    tool: str = ""              # label for the progress line, e.g. "Bash ✓"
    answer: str = ""            # latest assistant text (the final answer)
    stop: str = ""              # non-empty => agent signalled completion
    pass_through: str = ""      # raw line to log verbatim (e.g. trust warnings)


class Agent:
    """A CLI proving agent backend."""

    kind = "base"
    streaming_output = ""  # the ``cfg.output`` value that selects live streaming

    def __init__(self, cfg: AgentConfig):
        self.cfg = cfg

    def is_streaming(self) -> bool:
        """Whether the configured output format supports live line parsing."""
        return self.cfg.output == self.streaming_output

    def argv(self) -> list[str]:
        """Full argv used to spawn the agent inside the workspace."""
        raise NotImplementedError

    def env(self) -> dict[str, str]:
        """Extra environment variables for the agent subprocess."""
        return {}

    def prepare(self, ws: Path) -> None:
        """Per-run hook to patch workspace files (e.g. a provider ``api_base``)."""
        return None

    def parse_stream_line(self, line: str) -> StreamEvent | None:
        """Map one stdout line to a :class:`StreamEvent`, or ``None`` to skip it."""
        raise NotImplementedError

    def sessions(self, ws: Path) -> list[Path]:
        """Trace candidates (files or dirs) a run in ``ws`` may have produced."""
        raise NotImplementedError


class VibeAgent(Agent):
    """The vibe CLI (Leanstral)."""

    kind = "vibe"
    streaming_output = "streaming"
    _SESSION_PREFIX = "session"

    def argv(self) -> list[str]:
        argv = [self.cfg.command, "-p", self.cfg.prompt] + self.cfg.args
        if self.cfg.max_turns is not None:
            argv += ["--max-turns", str(self.cfg.max_turns)]
        return argv + ["--output", self.cfg.output]

    def env(self) -> dict[str, str]:
        extra = {}
        if self.cfg.model:
            extra["VIBE_ACTIVE_MODEL"] = self.cfg.model
        return extra

    def prepare(self, ws: Path) -> None:
        """Point the deployed vibe agent profile at ``cfg.base_url`` (if set)."""
        if not self.cfg.base_url:
            return
        agents_dir = ws / ".vibe" / "agents"
        if not agents_dir.is_dir():
            return
        for toml in agents_dir.glob("*.toml"):
            text = toml.read_text()
            new, n = re.subn(
                r'api_base\s*=\s*"[^"]*"',
                f'api_base = "{self.cfg.base_url}"',
                text,
            )
            if n:
                toml.write_text(new)

    def parse_stream_line(self, line: str) -> StreamEvent | None:
        if not line:
            return None
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            # Non-JSON lines are either the stop event or a pass-through warning.
            if "vibe_stop_event" in line:
                return StreamEvent(stop=line.strip())
            return StreamEvent(pass_through=line)

        kind = entry.get("type")
        if kind == "effect":
            # ``--output streaming`` only emits completed tool calls as "effect".
            detail = entry.get("detail") or {}
            tool = entry.get("title") or detail.get("toolName") or "tool"
            status = (entry.get("state") or {}).get("status")
            mark = (
                "✓"
                if status == "completed"
                else "✗" if status in ("failed", "cancelled") else "•"
            )
            return StreamEvent(step=True, tool=f"{tool} {mark}")

        if kind == "message" and entry.get("role") == "assistant":
            event = StreamEvent(step=True)
            for block in entry.get("content") or []:
                if not isinstance(block, dict) or block.get("type") != "text":
                    continue
                text = block.get("text", "").strip()
                if not text:
                    continue
                if "vibe_stop_event" in text:
                    event.stop = text
                else:
                    event.answer = text
            return event

        return None

    def sessions(self, ws: Path) -> list[Path]:
        sessions = ws / ".vibe" / "sessions"
        if not sessions.is_dir():
            return []
        return [
            p
            for p in sessions.iterdir()
            if p.is_dir() and p.name.startswith(self._SESSION_PREFIX)
        ]


class ClaudeAgent(Agent):
    """Claude Code CLI (``claude -p --output-format stream-json``)."""

    kind = "claude"
    streaming_output = "stream-json"

    def __init__(self, cfg: AgentConfig, mcp: McpConfig, port: int):
        super().__init__(cfg)
        self.mcp = mcp
        self.port = port

    def argv(self) -> list[str]:
        # Point Claude Code at this workspace's lean-lsp-mcp HTTP server. The
        # JSON is passed inline so it never lands in the workspace's git
        # checkpoint (which ``git reset`` would otherwise wipe between cases).
        mcp_json = json.dumps(
            {
                "mcpServers": {
                    "lean-lsp": {
                        "type": "http",
                        "url": f"http://{self.mcp.host}:{self.port}/mcp",
                    }
                }
            }
        )
        argv = [self.cfg.command, "-p", self.cfg.prompt] + self.cfg.args
        if self.cfg.model:
            argv += ["--model", self.cfg.model]
        if self.is_streaming():
            # ``-p --output-format stream-json`` requires --verbose.
            argv += ["--verbose"]
        return argv + ["--output-format", self.cfg.output, "--mcp-config", mcp_json]

    def env(self) -> dict[str, str]:
        extra = {}
        if self.cfg.base_url:
            extra["ANTHROPIC_BASE_URL"] = self.cfg.base_url
        if self.cfg.api_key:
            extra["ANTHROPIC_AUTH_TOKEN"] = self.cfg.api_key
        if self.cfg.max_turns is not None:
            extra["CLAUDE_CODE_MAX_TURNS"] = str(self.cfg.max_turns)
        return extra

    def parse_stream_line(self, line: str) -> StreamEvent | None:
        if not line:
            return None
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            return StreamEvent(pass_through=line)

        kind = entry.get("type")
        if kind == "assistant":
            event = StreamEvent()
            for block in entry.get("message", {}).get("content") or []:
                if not isinstance(block, dict):
                    continue
                btype = block.get("type")
                if btype == "tool_use":
                    event.step = True
                    event.tool = f"{block.get('name', 'tool')} •"
                elif btype == "text":
                    text = block.get("text", "").strip()
                    if text:
                        event.answer = text
            return event

        if kind == "result":
            event = StreamEvent(stop=entry.get("subtype", ""))
            text = (entry.get("result") or "").strip()
            if text:
                event.answer = text
            return event

        return None

    def sessions(self, ws: Path) -> list[Path]:
        # Claude Code stores one transcript JSONL per session under
        # ~/.claude/projects/<sanitized abs path>/ where every non-alphanumeric
        # character ("/" and "_" included) becomes "-".
        project = (
            Path.home()
            / ".claude"
            / "projects"
            / re.sub(r"[^A-Za-z0-9]", "-", str(ws.resolve()))
        )
        if not project.is_dir():
            return []
        return list(project.glob("*.jsonl"))


def make_agent(cfg: AgentConfig, mcp: McpConfig, port: int) -> Agent:
    """Factory returning the backend matching ``cfg.kind`` (mirrors algoveri)."""
    if cfg.kind == "claude":
        return ClaudeAgent(cfg, mcp, port)
    return VibeAgent(cfg)
