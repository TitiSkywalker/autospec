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
import os
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
    # True when the pipeline enforces ``max_turns`` itself as a tool-call cap
    # (killing the process once ``steps`` exceeds it) instead of passing a
    # native turn limit to the CLI.
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
    # ``CLAUDE_CODE_MAX_TURNS`` is not set; the pipeline enforces ``max_turns``
    # as a tool-call cap (each ``tool_use`` block counts as one call).
    external_max_turns = True

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
        # reports the stop reason; ``error_max_turns`` is a controlled turn-budget
        # stop (only fires when the ambient CLAUDE_CODE_MAX_TURNS is set), not a
        # backend failure.
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


def _yaml_str(value: str) -> str:
    """A YAML double-quoted scalar; JSON strings are valid YAML scalars."""
    return json.dumps(value)


def _dsh_home() -> Path:
    """dsh's data root: ``$DSH_HOME`` (if set) else ``~/.dsh``."""
    env = os.environ.get("DSH_HOME", "").strip()
    if env:
        return Path(os.path.expanduser(env))
    return Path.home() / ".dsh"


def _project_key(cwd: str) -> str:
    """dsh's ``projectKey``: the per-cwd directory under ``$DSH_HOME/sessions``.

    Separators (``/\\:``) collapse to ``-``; unsafe code units become ``~XXXX``
    (uppercase hex); the result is wrapped as ``--<key>--`` (lossy, bounded).
    """
    readable = ""
    separator_run = False
    for ch in cwd:
        if ch in "/\\:":
            if not separator_run:
                readable += "-"
            separator_run = True
        elif ch != "~" and re.fullmatch(r"[A-Za-z0-9._-]", ch):
            readable += ch
            separator_run = False
        else:
            readable += "~" + f"{ord(ch):04X}"
            separator_run = False
    return "--" + (readable.lstrip("-") or "root")[:251] + "--"


class DshAgent(Agent):
    """DeepSeek Harness (``dsh --profile headless``)."""

    kind = "dsh"
    streaming_output = "json"  # ``--json`` emits newline-delimited run events
    # dsh has no native turn cap, so the pipeline kills the process once the
    # number of completed tool calls exceeds ``max_turns`` (mirrors codex).
    external_max_turns = True
    _PROFILE = "headless"
    _PROVIDER = "deepseek-official"  # the dsh-llm-deepseek provider route
    _MCP_SERVER = "lean-lsp"

    def __init__(self, cfg: AgentConfig, mcp: McpConfig, port: int):
        super().__init__(cfg)
        self.mcp = mcp
        self.port = port
        self._patch_path: Path | None = None

    def prepare(self, ws: Path) -> None:
        """Write the per-worker ``--patch`` overlay (model + lean-lsp MCP).

        dsh composes its config from a plugin tree; ``--patch`` is the per-run
        overlay. The file lives in the worker dir — outside the workspace git
        tree, which ``git reset`` would otherwise wipe between cases — mirroring
        ClaudeAgent's inline ``--mcp-config``.
        """
        self._patch_path = ws.parent / f"dsh-{self.port}.patch.yml"
        self._patch_path.parent.mkdir(parents=True, exist_ok=True)
        self._patch_path.write_text(self._patch_yaml())

    def _patch_yaml(self) -> str:
        entries = []
        if self.cfg.model:
            entries.append(
                "- id: agent-default-model\n"
                "  config:\n"
                f"    provider: {self._PROVIDER}\n"
                f"    model: {_yaml_str(self.cfg.model)}\n"
            )
        # Attach this workspace's lean-lsp-mcp HTTP server as an MCP client
        # entry (mirrors CodexAgent's ``-c mcp_servers.lean-lsp.url=...``).
        entries.append(
            "- insert:\n"
            "    - name: '@deepseek-ai/dsh-mcp-client'\n"
            "      config:\n"
            f"        serverName: {self._MCP_SERVER}\n"
            "        transport: streamable-http\n"
            f"        url: {_yaml_str(f'http://{self.mcp.host}:{self.port}/mcp')}\n"
        )
        return "".join(entries)

    def _patch_argv(self) -> list[str]:
        if self._patch_path is None:
            return []
        return ["--patch", str(self._patch_path)]

    def argv(self) -> list[str]:
        argv = [self.cfg.command, "--profile", self._PROFILE] + self._patch_argv()
        argv += self.cfg.args
        if self.is_streaming():
            argv += ["--json"]
        argv += [self.cfg.prompt]
        return argv

    def env(self) -> dict[str, str]:
        extra = {}
        # dsh's llm-deepseek adapter reads the Messages-protocol endpoint from
        # DEEPSEEK_BASE_URL (default https://api.deepseek.com/anthropic) and the
        # bearer token from DEEPSEEK_API_KEY (the launch env wins over stored
        # credentials). Empty fields fall back to the ambient shell env.
        if self.cfg.base_url:
            extra["DEEPSEEK_BASE_URL"] = self.cfg.base_url
        if self.cfg.api_key:
            extra["DEEPSEEK_API_KEY"] = self.cfg.api_key
        # Headless runs have no approval UI; the proving agent needs unfettered
        # workspace access (mirrors codex's --dangerously-bypass-approvals-and-sandbox).
        extra["DSH_PERMISSION_MODE"] = "danger-full-access"
        return extra

    def parse_stream_line(self, line: str) -> StreamEvent | None:
        if not line:
            return None
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            return StreamEvent(pass_through=line)

        kind = entry.get("type")
        if kind == "tool_call":
            return StreamEvent(step=True, tool=f"{entry.get('tool', 'tool')} ✓")
        if kind == "text":
            text = (entry.get("text") or "").strip()
            return StreamEvent(answer=text) if text else None
        if kind == "status":
            # ``turn_end`` carries the run's stop reason; ``step_*``/``turn_start``
            # bookkeeping is not user-visible work.
            if entry.get("phase") == "turn_end":
                reason = entry.get("reason") or {}
                rkind = reason.get("kind") if isinstance(reason, dict) else str(reason)
                return StreamEvent(stop=rkind)
            return None
        if kind == "final":
            # Terminal event carrying the lossless answer (empty on a failed turn;
            # the ``turn_end`` reason already signalled the failure).
            text = (entry.get("text") or "").strip()
            return StreamEvent(answer=text) if text else None
        # ``session``/``thinking``/``error`` are not work units; ``dsh:``
        # diagnostics reach stderr and are logged as pass-through already.
        return None

    def sessions(self, ws: Path) -> list[Path]:
        # Sessions persist under ``$DSH_HOME/sessions/<project-key>/<session-id>/``
        # (JSONL), keyed by the recorded cwd. Filter to this workspace's project
        # dir so concurrent workers don't cross-pick.
        root = _dsh_home() / "sessions" / _project_key(str(ws))
        if not root.is_dir():
            return []
        return [p for p in root.iterdir() if p.is_dir()]

    def resume_argv(self, session_id: str) -> list[str]:
        # ``--session-id <id>`` adopts the persisted Session and sends ``prompt``
        # as the follow-up message (the run must keep the same cwd).
        argv = [self.cfg.command, "--profile", self._PROFILE] + self._patch_argv()
        argv += self.cfg.args
        if self.is_streaming():
            argv += ["--json", "--session-id", session_id]
        argv += [self.cfg.prompt]
        return argv

    def session_id(self, trace: Path) -> str | None:
        # A dsh trace is the session dir; the resume id is its name
        # (``session-<uuid>``).
        return trace.name or None

    def is_limit_stop(self, stop: str) -> bool:
        # ``max-tokens`` is dsh's controlled budget stop (the model hit its
        # output-token cap), not a backend failure — skip resume-retries.
        return stop == "max-tokens"


def make_agent(cfg: AgentConfig, mcp: McpConfig, port: int) -> Agent:
    """Factory returning the backend matching ``cfg.kind`` (mirrors algoveri)."""
    if cfg.kind == "claude":
        return ClaudeAgent(cfg, mcp, port)
    if cfg.kind == "codex":
        return CodexAgent(cfg, mcp, port)
    if cfg.kind == "dsh":
        return DshAgent(cfg, mcp, port)
    return VibeAgent(cfg)
