"""
Evaluation pipeline configuration, read entirely from ``config.toml``.
"""

import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CONFIG = ROOT / "evaluation" / "local" / "config.toml"


@dataclass
class AgentConfig:
    command: str
    prompt: str
    args: list[str]
    timeout: float
    output: str = "streaming"
    kind: str = "vibe"
    model: str = ""      # model name/alias (empty => backend default)
    base_url: str = ""   # API base URL (empty => backend default)
    api_key: str = ""    # API/auth token (empty => inherit from environment)
    max_turns: int | None = None  # turn cap (None => unlimited)


@dataclass
class McpConfig:
    command: list[str]  # e.g. ["uvx", "lean-lsp-mcp"]
    transport: str
    host: str
    start_port: int  # worker i -> start_port + i
    ready_timeout: float


@dataclass
class Config:
    input_dir: Path
    output_dir: Path
    base_dir: Path
    worker_dir: Path
    num_workers: int
    agent: AgentConfig
    mcp: McpConfig
    vibe_dir: Path | None

    @classmethod
    def load(cls, path: Path = DEFAULT_CONFIG) -> "Config":
        """Read every setting from ``path`` (relative paths resolve against the
        repository root)."""
        data = tomllib.loads(Path(path).read_text())

        def resolve(p: str) -> Path:
            p = Path(p)
            return p if p.is_absolute() else ROOT / p

        prover = data["prover"]
        mcp = prover["mcp"]
        workspace = data["workspace"]
        vibe_dir = workspace.get("vibe_dir")

        # ``backend`` selects one of the ``[prover.agent.<backend>]`` tables.
        # Fall back to a flat ``[prover.agent]`` table for backwards compat.
        backend = prover.get("backend", "vibe")
        agent = prover["agent"]
        selected = agent.get(backend) if isinstance(agent, dict) else None
        if isinstance(selected, dict):
            agent = selected
        kind = backend
        default_output = "stream-json" if kind in ("claude", "codex") else "streaming"
        max_turns = agent.get("max_turns")
        max_turns = int(max_turns) if max_turns is not None else None
        return cls(
            input_dir=resolve(prover["input_dir"]),
            output_dir=resolve(prover["output_dir"]),
            base_dir=resolve(workspace["base_dir"]),
            worker_dir=resolve(workspace["worker_dir"]),
            num_workers=int(workspace["workers"]),
            agent=AgentConfig(
                command=agent["command"],
                prompt=agent["prompt"],
                args=agent.get("args", []),
                timeout=float(agent["timeout"]),
                output=agent.get("output", default_output),
                kind=kind,
                model=agent.get("model", ""),
                base_url=agent.get("base_url", ""),
                api_key=agent.get("api_key", ""),
                max_turns=max_turns,
            ),
            mcp=McpConfig(
                command=mcp["command"],
                transport=mcp["transport"],
                host=mcp["host"],
                start_port=int(mcp["start_port"]),
                ready_timeout=float(mcp["ready_timeout"]),
            ),
            vibe_dir=resolve(vibe_dir) if vibe_dir else None,
        )
