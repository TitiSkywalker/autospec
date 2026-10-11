# Review pipeline for LeanMode `Spec.lean`

This directory holds an automated review pipeline for the LeanMode benchmark (`lean_mode/LeanMode/`). For each testcase it launches a coding agent (codex or claude code) that audits the *semantics* of the hand-written `Spec.lean` — is it provable? are preconditions missing? is there a counterexample? — and writes one report per testcase. The agent does **not** check compilation or re-translate the testcase; those are already covered by `scripts/check_all.py` and `scripts/translate_all.sh`.

```
review/
├── README.md          # this file (running the pipeline)
├── ENVIRONMENT.md     # how to run the translator + Lean environment
├── REVIEW_PROMPT.md   # the task handed to every agent (review.py fills in the tokens)
├── review.py          # orchestrator: one agent per testcase, collects reports
└── reports/           # generated: <name>.md, <name>.log, SUMMARY.md  (gitignored)
```

---

## 1. Running the review

```bash
python3 review/review.py                  # review every testcase, 4 concurrent agents
python3 review/review.py --only matrix_qr # one testcase
python3 review/review.py --agent claude   # use Claude Code instead of codex
python3 review/review.py -w 8             # 8 concurrent agents
python3 review/review.py --resume         # skip testcases already reviewed
python3 review/review.py --list           # list reviewable testcases and exit
```

| flag | default | meaning |
|---|---|---|
| `--only NAME` | — | review only this testcase (repeatable) |
| `--agent {codex,claude}` | `codex` | which agent backend to use |
| `-w, --workers N` | `4` | concurrent agents |
| `--timeout SECS` | `1200` | per-testcase time limit |
| `--model MODEL` | agent default | model override (`codex` → `-m`, `claude` → `--model`) |
| `--output DIR` | `review/reports` | where reports land |
| `--resume` | off | skip testcases whose report already exists |

Reports are written to `review/reports/<name>.md` (the agent's final message) plus `<name>.log` (full stdout/stderr for debugging), and an index `review/reports/SUMMARY.md`. Each report carries a `Verdict: PASS | FAIL | CONCERNS` line; `review.py` extracts it into the summary table.

### Agent prerequisites

- **codex** (default): configured via `~/.codex/config.toml` (`model_provider = "paratera"`, `model = "GPT-5.6-Sol"`, project marked `trusted`). `review.py` runs `codex exec --ephemeral -s danger-full-access -C <root>`, so the agent has full read/write to compile and run code.
- **claude**: run `review.py --agent claude` in a shell where the model endpoint is set, e.g. per `translation/local/command.sh`:

  ```bash
  export ANTHROPIC_AUTH_TOKEN=...
  export ANTHROPIC_BASE_URL=...
  python3 review/review.py --agent claude
  ```

The agent may only *read* `Spec.lean` / generated `.lean` / Rust sources and may write throwaway files under `/tmp` (see the "Hard constraints" in `REVIEW_PROMPT.md`). It never edits the testcase.

---

## 2. Translator & Lean environment

See [`ENVIRONMENT.md`](ENVIRONMENT.md) for how to run the translator (single and batch) and build the Lean project (`lake build`, `cargo test`, `scripts/check_all.py`), and [`lean_mode/TESTCASE_GUIDE.md`](../lean_mode/TESTCASE_GUIDE.md) for the full testcase-writing guide and pitfalls.
