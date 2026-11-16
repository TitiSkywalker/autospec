"""Run an agent with the ``vericoding`` skill file injected.

The skill is distilled from trace analysis (see
``analysis/ablation/skills/vericoding/SKILL.md``).  This runner delivers it to
the agent through two channels, because the two backends differ in how they
pick up workspace instructions:

  1. **Prompt** — the skill text is appended to the base prompt, which both
     ``dsh`` and ``claude`` receive as a command-line argument.  This is
     guaranteed delivery.
  2. **Workspace file** — written to ``instructions.md`` in the workspace so
     the agent can re-read it later in the run.

The skill contains no case-specific content: only behavioural guidance
(verify before finishing, use the LSP tools, avoid thrash, Aeneas tactics,
reading discipline).

Usage::

    # preview what the agent will receive
    python3 analysis/ablation/run_skill.py --config evaluation/local/config.dsh.toml --dry-run

    # full run
    python3 analysis/ablation/run_skill.py --config evaluation/local/config.dsh.toml

    # a few cases first
    python3 analysis/ablation/run_skill.py --config evaluation/local/config.dsh.toml \
        --case array_two_sum --case dp_house_robber
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from evaluation.config import AgentConfig, Config  # noqa: E402
from evaluation.manager import Manager             # noqa: E402
from evaluation.testcase import TestCase           # noqa: E402

SKILL_PATH = ROOT / "analysis" / "ablation" / "skills" / "vericoding" / "SKILL.md"

_SORRY_RE = re.compile(r"\b(sorry|admit)\b")


def case_verdict(case_dir: Path) -> str:
    """Verdict of a finished case dir (same logic as Manager._summarize).

    PROVED | UNPROVED | BUILD_FAIL | ERROR | MISSING
    """
    status_file = case_dir / "status.txt"
    if not status_file.is_file():
        return "MISSING"
    status = status_file.read_text().strip()
    if status == "build_failed":
        return "BUILD_FAIL"
    if status != "ok":
        return "ERROR"
    spec = case_dir / "Spec.lean"
    text = spec.read_text() if spec.is_file() else ""
    return "UNPROVED" if text and _SORRY_RE.search(text) else "PROVED"


def failed_cases(results_dir: Path) -> set[str]:
    """Names of cases that did NOT prove in ``results_dir``."""
    out: set[str] = set()
    for d in sorted(results_dir.iterdir()):
        if not d.is_dir():
            continue
        if case_verdict(d) != "PROVED":
            out.add(d.name)
    return out


# Short pointer appended to the prompt; the full text lives in instructions.md.
_POINTER = (
    "\n\n"
    "A working guide distilled from successful runs is in `instructions.md` "
    "at the workspace root, and is reproduced below. Follow it. Three rules "
    "matter most: (1) never finish without a kernel check — `lean_verify`, or "
    "`#print axioms` — and no `sorry` left in place; (2) never edit "
    "`RustVerif/Function.lean` (the verifier restores it); (3) start from the "
    "guide's prelude of bridge lemmas. Its two environment facts are "
    "load-bearing: this benchmark's specs are equational (`f x = ok r` as a "
    "hypothesis), not `⦃ ⦄` WP triples, and its loops are "
    "`partial_fixpoint` definitions, so the `step*`-plus-WP recipes in "
    "`aeneas/documentation/` do not apply — work by `unfold … at h` on the "
    "`do`-block instead.\n\n"
)


def _strip_frontmatter(text: str) -> str:
    """Drop the YAML frontmatter block; the agent needs the body only."""
    if text.startswith("---"):
        parts = text.split("---", 2)
        if len(parts) == 3:
            return parts[2].lstrip("\n")
    return text


@dataclass
class SkillTestCase(TestCase):
    """A TestCase that writes the skill into the workspace on install."""

    skill_body: str = ""

    def install(self, ws: Path) -> None:
        super().install(ws)
        (ws / "instructions.md").write_text(self.skill_body, encoding="utf-8")


class SkillManager(Manager):
    """Manager that restricts to a case list and injects the skill file."""

    def __init__(self, config: Config, cases: list[TestCase]):
        super().__init__(config)
        self._cases = cases

    def run(self, skip_existing: bool = False) -> None:
        self._prepare()
        cases = self._cases
        if skip_existing:
            cases = [c for c in cases if not self._already_done(c.name)]
        if not cases:
            self.logger.info("No cases to run.")
            return

        funcs = {c.name: c.func_src for c in cases}

        import multiprocessing
        from evaluation.isolation import SandboxConfig
        from evaluation.manager import _worker
        from evaluation.progress import StatusBoard

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
        for case in cases:
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


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Run an agent guided by the vericoding skill file"
    )
    ap.add_argument("--config", required=True,
                    help="target agent config TOML")
    ap.add_argument("--skill", default=str(SKILL_PATH),
                    help="path to the skill file")
    ap.add_argument("--out-dir",
                    help="output dir (default: <original>_skill)")
    ap.add_argument("--case", action="append", dest="cases",
                    help="restrict to one case by name (repeatable)")
    ap.add_argument("--dry-run", action="store_true",
                    help="print the injected prompt and exit")
    ap.add_argument("--skip-existing", action="store_true")
    ap.add_argument("--failed-only", nargs="?", const="auto", default=None,
                    metavar="RESULTS_DIR",
                    help="run only cases that did not prove in RESULTS_DIR "
                         "(default: the config's own output_dir)")
    args = ap.parse_args()

    skill_text = Path(args.skill).read_text(encoding="utf-8")
    skill_body = _strip_frontmatter(skill_text)

    config = Config.load(Path(args.config))
    baseline_dir = config.output_dir
    if args.out_dir:
        config.output_dir = Path(args.out_dir)
    else:
        config.output_dir = config.output_dir.parent / (config.output_dir.name + "_skill")

    all_cases = TestCase.scan(config.input_dir)

    if args.failed_only:
        src = Path(args.failed_only) if args.failed_only != "auto" else baseline_dir
        if not src.is_dir():
            sys.exit(f"results dir not found: {src}")
        failed = failed_cases(src)
        all_cases = [c for c in all_cases if c.name in failed]
        print(f"Restricting to {len(failed)} non-proved case(s) from {src}")

    # --case narrows whatever the selection above produced (composes with
    # --failed-only rather than replacing it).
    if args.cases:
        names = set(args.cases)
        all_cases = [c for c in all_cases if c.name in names]
        missing = names - {c.name for c in all_cases}
        if missing:
            print(f"WARNING: case(s) not selected: {', '.join(sorted(missing))}")

    if not all_cases:
        print("No cases to run.")
        return

    new_prompt = config.agent.prompt + _POINTER + skill_body

    print(f"Skill:      {args.skill}")
    print(f"Output dir: {config.output_dir}")
    print(f"Cases:      {len(all_cases)}")
    print(f"Prompt len: {len(config.agent.prompt)} -> {len(new_prompt)} chars")

    if args.dry_run:
        print("\n--- injected prompt ---")
        print(new_prompt)
        return

    config.agent = AgentConfig(
        command=config.agent.command,
        prompt=new_prompt,
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

    cases = [
        SkillTestCase(
            name=c.name, func_src=c.func_src, spec_src=c.spec_src,
            skill_body=skill_body,
        )
        for c in all_cases
    ]

    SkillManager(config, cases).run(skip_existing=args.skip_existing)


if __name__ == "__main__":
    main()
