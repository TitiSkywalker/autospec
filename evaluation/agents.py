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

    step: bool = False  # counts as one unit of work (a tool call / turn)
    tool: str = ""  # label for the progress line, e.g. "Bash ✓"
    answer: str = ""  # latest assistant text (the final answer)
    stop: str = ""  # non-empty => agent signalled completion
    pass_through: str = ""  # raw line to log verbatim (e.g. trust warnings)


class Agent:
    """A CLI proving agent backend."""

    kind = "base"
    streaming_output = ""  # the ``cfg.output`` value that selects live streaming
    # True when the CLI has no native turn cap: the pipeline then enforces
    # ``max_turns`` itself by killing the process once ``steps`` exceeds it.
    external_max_turns = False

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

    def resume_argv(self, session_id: str) -> list[str]:
        """argv to resume ``session_id`` after an abnormal exit."""
        raise NotImplementedError

    def session_id(self, trace: Path) -> str | None:
        """Extract the resume-able session id from a trace path, or ``None``."""
        return None

    def is_limit_stop(self, stop: str) -> bool:
        """Whether a reported ``stop`` signal means the agent exhausted its own
        budget (a controlled limit stop, e.g. ``max_turns``), as opposed to a
        backend/crash failure. The pipeline skips resume-retries for limit stops."""
        return False


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

    def resume_argv(self, session_id: str) -> list[str]:
        # ``--resume`` re-enters programmatic mode but still requires a prompt
        # (appended as the follow-up message), so re-send the original goal.
        argv = [self.cfg.command, "-p", self.cfg.prompt, "--resume", session_id]
        argv += self.cfg.args
        if self.cfg.max_turns is not None:
            argv += ["--max-turns", str(self.cfg.max_turns)]
        return argv + ["--output", self.cfg.output]

    def session_id(self, trace: Path) -> str | None:
        # A vibe trace is a session dir; the resume id is ``meta.json``'s UUID.
        meta = trace / "meta.json"
        if not meta.is_file():
            return None
        try:
            return json.loads(meta.read_text()).get("session_id")
        except (json.JSONDecodeError, OSError):
            return None


class ClaudeAgent(Agent):
    """Claude Code CLI (``claude -p --output-format stream-json``)."""

    kind = "claude"
    streaming_output = "stream-json"

    def __init__(self, cfg: AgentConfig, mcp: McpConfig, port: int):
        super().__init__(cfg)
        self.mcp = mcp
        self.port = port

    def _mcp_json(self) -> str:
        # Point Claude Code at this workspace's lean-lsp-mcp HTTP server. The
        # JSON is passed inline so it never lands in the workspace's git
        # checkpoint (which ``git reset`` would otherwise wipe between cases).
        return json.dumps(
            {
                "mcpServers": {
                    "lean-lsp": {
                        "type": "http",
                        "url": f"http://{self.mcp.host}:{self.port}/mcp",
                    }
                }
            }
        )

    def argv(self) -> list[str]:
        argv = [self.cfg.command, "-p", self.cfg.prompt] + self.cfg.args
        if self.cfg.model:
            argv += ["--model", self.cfg.model]
        if self.is_streaming():
            # ``-p --output-format stream-json`` requires --verbose.
            argv += ["--verbose"]
        return argv + [
            "--output-format",
            self.cfg.output,
            "--mcp-config",
            self._mcp_json(),
        ]

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

    def resume_argv(self, session_id: str) -> list[str]:
        argv = [self.cfg.command, "--resume", session_id, "-p", self.cfg.prompt]
        argv += self.cfg.args
        if self.cfg.model:
            argv += ["--model", self.cfg.model]
        if self.is_streaming():
            argv += ["--verbose"]
        return argv + [
            "--output-format",
            self.cfg.output,
            "--mcp-config",
            self._mcp_json(),
        ]

    def session_id(self, trace: Path) -> str | None:
        # A claude trace is ``<session-uuid>.jsonl``; the id is the filename stem.
        return trace.stem or None

    def is_limit_stop(self, stop: str) -> bool:
        # Claude Code's stream-json ends with a ``result`` event whose ``subtype``
        # reports the stop reason; ``error_max_turns`` is its own turn budget
        # (CLAUDE_CODE_MAX_TURNS), not a backend failure.
        return stop == "error_max_turns"


class CodexAgent(Agent):
    """OpenAI Codex CLI (``codex exec --json``)."""

    kind = "codex"
    streaming_output = "stream-json"
    # Codex has no native turn cap, so the pipeline kills the process once the
    # number of completed tool calls exceeds ``max_turns``.
    external_max_turns = True
    _PROVIDER = (
        "pipeline"  # self-contained provider built from ``base_url``/``api_key``
    )
    # ``item.completed`` payloads that count as a unit of work (a tool call).
    _TOOL_TYPES = (
        "command_execution",
        "local_shell_call",
        "mcp_tool_call",
        "web_search",
        "computer_use",
        "apply_patch",
    )

    def __init__(self, cfg: AgentConfig, mcp: McpConfig, port: int):
        super().__init__(cfg)
        self.mcp = mcp
        self.port = port

    def argv(self) -> list[str]:
        argv = [self.cfg.command, "exec", self.cfg.prompt] + self.cfg.args
        if self.cfg.model:
            argv += ["-m", self.cfg.model]
        argv += self._provider_overrides()
        argv += self._mcp_override()
        if self.is_streaming():
            argv += ["--json"]
        return argv

    def _provider_overrides(self) -> list[str]:
        # When ``base_url`` is set, route through a self-contained provider so the
        # endpoint/auth are fully specified by config (mirrors claude's
        # ANTHROPIC_BASE_URL/AUTH_TOKEN). Otherwise inherit ``~/.codex/config.toml``.
        if not self.cfg.base_url:
            return []
        p = self._PROVIDER
        values = [
            f"model_provider={p}",
            f'model_providers.{p}.name="{p}"',
            f'model_providers.{p}.wire_api="responses"',
            f'model_providers.{p}.env_key="CODEX_API_KEY"',
            f'model_providers.{p}.base_url="{self.cfg.base_url}"',
        ]
        return [flag for value in values for flag in ("-c", value)]

    def _mcp_override(self) -> list[str]:
        # Attach this workspace's lean-lsp-mcp HTTP server via a ``-c`` override so
        # it never lands in the workspace git checkpoint (mirrors ClaudeAgent's
        # inline ``--mcp-config``).
        return [
            "-c",
            f'mcp_servers.lean-lsp.url="http://{self.mcp.host}:{self.port}/mcp"',
        ]

    def env(self) -> dict[str, str]:
        extra = {}
        if self.cfg.api_key:
            extra["CODEX_API_KEY"] = self.cfg.api_key
        return extra

    def parse_stream_line(self, line: str) -> StreamEvent | None:
        if not line:
            return None
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            return StreamEvent(pass_through=line)

        if entry.get("type") != "item.completed":
            return None
        item = entry.get("item") or {}
        itype = item.get("type")
        if itype == "agent_message":
            return StreamEvent(answer=(item.get("text") or "").strip())
        if itype in self._TOOL_TYPES:
            if itype == "mcp_tool_call":
                label = f"{item.get('server', 'mcp')}/{item.get('tool', 'tool')}"
            else:
                label = (item.get("command") or "").strip() or itype
            return StreamEvent(step=True, tool=f"{label} ✓")
        # ``reasoning`` and any other payload are not user-visible work units.
        return None

    def sessions(self, ws: Path) -> list[Path]:
        # Codex stores one rollout JSONL per session under ~/.codex/sessions/
        # YYYY/MM/DD/ (shared across all workspaces), keyed by the session's
        # ``cwd``. Filter to this workspace so concurrent workers don't cross-pick.
        root = Path.home() / ".codex" / "sessions"
        if not root.is_dir():
            return []
        targets = {str(ws), str(ws.resolve())}
        out = []
        for p in root.rglob("*.jsonl"):
            try:
                with p.open() as fh:
                    # Read the whole session_meta line: it embeds the (large)
                    # base_instructions, so a bounded readline would truncate it.
                    head = fh.readline()
            except OSError:
                continue
            try:
                cwd = json.loads(head).get("payload", {}).get("cwd", "")
            except json.JSONDecodeError:
                continue
            if cwd in targets:
                out.append(p)
        return out

    def resume_argv(self, session_id: str) -> list[str]:
        # ``codex exec resume <id> <prompt>`` re-attaches to the session and
        # sends ``prompt`` as the follow-up message.
        argv = [self.cfg.command, "exec", "resume", session_id, self.cfg.prompt]
        if self.cfg.model:
            argv += ["-m", self.cfg.model]
        argv += self._provider_overrides()
        argv += self._mcp_override()
        if self.is_streaming():
            argv += ["--json"]
        return argv

    def session_id(self, trace: Path) -> str | None:
        # A codex trace is a rollout JSONL; the resume id lives in the first
        # line's ``session_meta`` payload.
        try:
            with trace.open() as fh:
                head = fh.readline()
            return json.loads(head).get("payload", {}).get("session_id")
        except (json.JSONDecodeError, OSError, AttributeError):
            return None


def make_agent(cfg: AgentConfig, mcp: McpConfig, port: int) -> Agent:
    """Factory returning the backend matching ``cfg.kind`` (mirrors algoveri)."""
    if cfg.kind == "claude":
        return ClaudeAgent(cfg, mcp, port)
    if cfg.kind == "codex":
        return CodexAgent(cfg, mcp, port)
    return VibeAgent(cfg)
