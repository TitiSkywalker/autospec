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
        agent = prover["agent"]
        mcp = prover["mcp"]
        workspace = data["workspace"]
        vibe_dir = workspace.get("vibe_dir")
        return cls(
            input_dir=resolve(prover["input_dir"]),
            output_dir=resolve(prover["output_dir"]),
            base_dir=resolve(workspace["base_dir"]),
            worker_dir=resolve(workspace["worker_dir"]),
            num_workers=int(workspace["workers"]),
            agent=AgentConfig(
                command=agent["command"],
                prompt=agent["prompt"],
                args=agent["args"],
                timeout=float(agent["timeout"]),
                output=agent.get("output", "streaming"),
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
