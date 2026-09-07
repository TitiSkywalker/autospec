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


@dataclass
class Config:
    input_dir: Path
    output_dir: Path
    workspaces: list[Path]
    agent: AgentConfig
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
        vibe_dir = data.get("workspace", {}).get("vibe_dir")
        return cls(
            input_dir=resolve(prover["input_dir"]),
            output_dir=resolve(prover["output_dir"]),
            workspaces=[resolve(p) for p in data["workspace"]["candidates"]],
            agent=AgentConfig(
                command=agent["command"],
                prompt=agent["prompt"],
                args=agent["args"],
                timeout=float(agent["timeout"]),
            ),
            vibe_dir=resolve(vibe_dir) if vibe_dir else None,
        )
