"""Ablation study: does showing an agent which Aeneas files to read improve solve rate?

This runner is a thin wrapper around the existing evaluation pipeline.  It:
  1. Loads per-case Aeneas file hints (from ``analysis/ablation/ablation_hints.py``).
  2. Replaces ``TestCase`` with ``HintedTestCase``, which writes a ``HINTS.md``
     into the workspace alongside Function.lean / Spec.lean and appends a
     focused reading instruction to the agent prompt.
  3. Restricts the run to *ablation candidates*: cases where the hint source
     (claude-opus-5.5) proved the theorem and the target agent did not in its
     baseline run.
  4. Writes results to a new output dir so baseline results are untouched.

The prompt addition tells the agent exactly which files are relevant and asks
it not to waste turns reading unrelated library files.  No solution content is
leaked — only file paths.

Usage::

    # dry-run: print candidate cases and hints, do not run anything
    python3 analysis/ablation/run_ablation.py --config evaluation/local/config.dsh.toml --dry-run

    # real run (all ablation candidates)
    python3 analysis/ablation/run_ablation.py --config evaluation/local/config.dsh.toml

    # run a single case
    python3 analysis/ablation/run_ablation.py --config evaluation/local/config.dsh.toml --case dp_edit_distance
"""

from __future__ import annotations

import argparse
import glob
import json
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from evaluation.config import AgentConfig, Config  # noqa: E402
from evaluation.manager import Manager             # noqa: E402
from evaluation.testcase import TestCase           # noqa: E402
from evaluation.workspace import LIB               # noqa: E402

# Aeneas rel-path → workspace-relative path prefix
_AENEAS_WS_PREFIX = "aeneas/backends/lean"

_HINTS_JSON = ROOT / "analysis" / "ablation" / "aeneas_hints.json"


# ── hint-aware TestCase ───────────────────────────────────────────────────────

@dataclass
class HintedTestCase(TestCase):
    """A TestCase that writes a HINTS.md into the workspace on install."""

    hint_files: list[str]  # Aeneas rel-paths, e.g. ["Aeneas/Std/Slice.lean", ...]

    def install(self, ws: Path) -> None:
        super().install(ws)
        lib = ws / LIB
        hint_lines = [
            f"- `{_AENEAS_WS_PREFIX}/{f}`"
            for f in self.hint_files
        ]
        hints_md = "\n".join([
            "# Relevant Aeneas library files",
            "",
            "The following Aeneas library files are known to be relevant for this proof.",
            "Read these files to understand the available types and lemmas.",
            "**Do not spend turns reading other Aeneas or Mathlib files** — they are",
            "unlikely to be needed and will waste your budget.",
            "",
        ] + hint_lines + [""])
        (lib / "HINTS.md").write_text(hints_md, encoding="utf-8")


def _hint_prompt(base_prompt: str, hint_files: list[str]) -> str:
    """Append a focused reading instruction to the base prompt."""
    file_list = "\n".join(f"  - aeneas/backends/lean/{f}" for f in hint_files)
    appendix = (
        "\n\n"
        "Before writing any proof, read the following Aeneas library files "
        "(and only these — do not open other Aeneas or Mathlib files unless "
        "absolutely necessary):\n"
        f"{file_list}\n"
        "A file `RustVerif/HINTS.md` in the workspace lists the same files for reference."
    )
    return base_prompt + appendix


# ── hint + baseline data ──────────────────────────────────────────────────────

def load_hints() -> dict[str, list[str]]:
    if not _HINTS_JSON.exists():
        raise FileNotFoundError(
            f"{_HINTS_JSON} not found — run: python3 analysis/ablation/ablation_hints.py"
        )
    return json.loads(_HINTS_JSON.read_text())


def _keys(v) -> list[str]:
    if isinstance(v, dict):
        return list(v.keys())
    return list(v) if v else []


def load_baseline_verdicts(agent_prefix: str) -> dict[str, str]:
    """Load per-case verdicts from indirect_reads_data for a given agent prefix."""
    verdicts: dict[str, str] = {}
    pattern = str(ROOT / "analysis" / "agent_trace" / "local" / "indirect_reads_data" / f"{agent_prefix}__*.json")
    for fpath in glob.glob(pattern):
        case = Path(fpath).stem.split("__", 1)[1]
        d = json.load(open(fpath))
        verdicts[case] = d.get("verdict", "?")
    return verdicts


def ablation_candidates(
    all_cases: list[TestCase],
    hints: dict[str, list[str]],
    baseline_verdicts: dict[str, str],
) -> list[HintedTestCase]:
    """Return HintedTestCase list: cases that have hints AND the target agent did not prove."""
    candidates = []
    for tc in all_cases:
        if tc.name not in hints:
            continue
        v = baseline_verdicts.get(tc.name, "missing")
        if v == "PROVED":
            continue   # already solved — no point re-running with hint
        candidates.append(HintedTestCase(
            name=tc.name,
            func_src=tc.func_src,
            spec_src=tc.spec_src,
            hint_files=hints[tc.name],
        ))
    return candidates


# ── Manager subclass that injects hints ──────────────────────────────────────

class AblationManager(Manager):
    """Manager that restricts to ablation candidates and injects per-case hints."""

    def __init__(self, config: Config, candidates: list[HintedTestCase], hints: dict[str, list[str]]):
        super().__init__(config)
        self._candidates = candidates
        self._hints = hints

    def run(self, skip_existing: bool = False) -> None:
        """Override to use the candidate list instead of scanning input_dir."""
        self._prepare()
        cases = self._candidates
        if skip_existing:
            cases = [c for c in cases if not self._already_done(c.name)]
        if not cases:
            self.logger.info("No ablation candidates to run.")
            return

        # Patch agent config: per-case prompt injection is handled in HintedTestCase.install;
        # the base prompt gets a general reading discipline instruction here.
        # We monkeypatch the config's agent prompt for the run.
        base_prompt = self.config.agent.prompt

        import multiprocessing
        from evaluation.manager import _worker
        from evaluation.progress import StatusBoard
        from evaluation.isolation import SandboxConfig

        # Build workers.
        workers = [
            multiprocessing.Process(
                target=_worker,
                args=(
                    self.config.base_dir,
                    self.config.worker_dir,
                    i,
                    self.input_queue,
                    self.output_queue,
                    self.config.agent,
                    self.config.mcp,
                    self.config.output_dir,
                    self.config.input_dir,
                    self.config.sandbox or SandboxConfig(),
                ),
            )
            for i in range(self.config.num_workers)
        ]
        for w in workers:
            w.start()

        board = StatusBoard([f"ws-{i}" for i in range(self.config.num_workers)])
        funcs = {c.name: c.func_src for c in cases}

        for case in cases:
            # Inject per-case prompt into a fresh AgentConfig copy.
            patched = AgentConfig(
                command=self.config.agent.command,
                prompt=_hint_prompt(base_prompt, case.hint_files),
                args=self.config.agent.args,
                timeout=self.config.agent.timeout,
                output=self.config.agent.output,
                kind=self.config.agent.kind,
                model=self.config.agent.model,
                base_url=self.config.agent.base_url,
                api_key=self.config.agent.api_key,
                max_turns=self.config.agent.max_turns,
                retries=self.config.agent.retries,
                retry_backoff=self.config.agent.retry_backoff,
            )
            # We can't vary the prompt per-worker via the queue; the hint is in
            # HINTS.md (written by HintedTestCase.install) AND in the prompt
            # sent to the agent at launch.  Since per-case prompt patching
            # requires the agent to see it at spawn time, we patch the shared
            # config before queueing each case and restore after.  This works
            # because the queue is drained sequentially per worker process;
            # each worker re-reads self.cfg.prompt from the AgentConfig that was
            # pickled at process start.
            #
            # Simpler: we patch the shared config object before starting workers
            # for the common case where all ablation cases share the same hint
            # set (they do not).  The cleanest approach for per-case prompts is
            # to embed the hint in HINTS.md only and keep the base prompt
            # constant — the agent discovers HINTS.md when it reads the workspace.
            # We do both: base prompt gets a general instruction, HINTS.md has
            # the specific files.
            self.input_queue.put(case)

        for _ in workers:
            self.input_queue.put(None)

        results = []
        pending = len(cases)
        while pending:
            msg = self.output_queue.get()
            kind = msg[0]
            if kind == "log":
                board.log(msg[1])
            elif kind == "progress":
                board.update(msg[1], msg[2])
            elif kind == "fatal":
                for w in workers:
                    if w.is_alive():
                        w.terminate()
                    w.join()
                raise RuntimeError(f"worker {msg[1]} failed: {msg[2]}")
            else:
                _, index, name, spec, status = msg
                results.append((name, spec, status))
                pending -= 1
                self._save(name, funcs[name], spec, status)
                board.update(index, f"{name} · {status}")

        for w in workers:
            w.join()

        self._summarize(results)


# ── CLI ───────────────────────────────────────────────────────────────────────

def _infer_agent_prefix(config: Config) -> str:
    """Derive a short agent id from the output dir name, e.g. 'deepseek-v4.1-flash'."""
    name = config.output_dir.name   # e.g. "deepseek-v4.1-flash"
    _MAP = {"opus-5.5": "claude-opus-5.5", "opus-4.5": "claude-opus-4.5"}
    return _MAP.get(name, name)


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Ablation: run target agent with claude-opus-5.5 Aeneas file hints"
    )
    ap.add_argument(
        "--config", required=True,
        help="path to the target agent's config TOML (e.g. evaluation/local/config.dsh.toml)",
    )
    ap.add_argument(
        "--out-dir",
        help="output dir for ablation results (default: <original_output_dir>_ablation)",
    )
    ap.add_argument(
        "--agent-prefix",
        help="indirect_reads_data agent prefix to use as baseline, e.g. deepseek-v4.1-flash "
             "(default: inferred from config output_dir)",
    )
    ap.add_argument(
        "--case", action="append", dest="cases",
        help="restrict to one case by name (repeat for multiple)",
    )
    ap.add_argument(
        "--dry-run", action="store_true",
        help="print candidates and hints without running anything",
    )
    ap.add_argument(
        "--skip-existing", action="store_true",
        help="skip cases that already have a result in the ablation output dir",
    )
    args = ap.parse_args()

    config = Config.load(Path(args.config))

    # Infer agent prefix from the original output dir BEFORE overriding it.
    agent_prefix = args.agent_prefix or _infer_agent_prefix(config)
    baseline_dir = config.output_dir  # the unhinted run; verdicts come from here

    # Override output dir.
    if args.out_dir:
        config.output_dir = Path(args.out_dir)
    else:
        config.output_dir = config.output_dir.parent / (config.output_dir.name + "_ablation")

    hints = load_hints()
    from analysis.accuracy.verdicts import read_verdict
    baseline = {d.name: read_verdict(d) for d in baseline_dir.iterdir() if d.is_dir()}
    all_cases = TestCase.scan(config.input_dir)

    candidates = ablation_candidates(all_cases, hints, baseline)

    # Optionally restrict to specific case names.
    if args.cases:
        names = set(args.cases)
        candidates = [c for c in candidates if c.name in names]
        missing = names - {c.name for c in candidates}
        if missing:
            print(f"WARNING: no ablation candidate found for: {', '.join(sorted(missing))}")
            print("  (case may already be PROVED by target agent, or has no claude-opus-5.5 hint)")

    if not candidates:
        print("No ablation candidates found.")
        return

    print(f"Agent prefix (baseline): {agent_prefix}")
    print(f"Output dir:              {config.output_dir}")
    print(f"Candidates:              {len(candidates)}")
    print()
    print(f"{'case':<35} {'baseline':>10} {'hint_files':>10}")
    for c in sorted(candidates, key=lambda x: x.name):
        v = baseline.get(c.name, "missing")
        print(f"{c.name:<35} {v:>10} {len(c.hint_files):>10}")
        for f in c.hint_files:
            print(f"    aeneas/backends/lean/{f}")

    if args.dry_run:
        print("\n(dry-run — not running)")
        return

    # Patch agent prompt with a general reading-discipline instruction.
    base_prompt = config.agent.prompt
    general_hint = (
        "\n\nA file `RustVerif/HINTS.md` in the workspace lists the specific "
        "Aeneas library files that are relevant to this proof. Read those files "
        "first and avoid opening other Aeneas or Mathlib files unless you have "
        "exhausted the listed ones — unguided exploration wastes your turn budget."
    )
    config.agent = AgentConfig(
        command=config.agent.command,
        prompt=base_prompt + general_hint,
        args=config.agent.args,
        timeout=config.agent.timeout,
        output=config.agent.output,
        kind=config.agent.kind,
        model=config.agent.model,
        base_url=config.agent.base_url,
        api_key=config.agent.api_key,
        max_turns=config.agent.max_turns,
        retries=config.agent.retries,
        retry_backoff=config.agent.retry_backoff,
    )

    mgr = AblationManager(config, candidates, hints)
    mgr.run(skip_existing=args.skip_existing)


if __name__ == "__main__":
    main()
