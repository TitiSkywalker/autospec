# AutoSpec

AutoSpec is a benchmark and toolchain for evaluating AI agents on **verifying
Rust programs**. Each task is a Rust crate that has been translated to Lean 4
with [Aeneas](https://github.com/AeneasVerif/aeneas), plus a hand-written
specification (`Spec.lean`) whose theorems are stated with `sorry`. The agent
has to replace every `sorry` with a proof. A separate verifier then checks the
result with the [Comparator](comparator/) judge. A case counts as proved only
if all three hold:

1. the theorem statements are unchanged,
2. only the standard axioms are used (no `sorry`, no new `axiom`), and
3. the Lean kernel accepts the proof.

```
.
├── lean_mode/        # the benchmark: 159 Rust → Lean cases (array, crypto, dp, ds, graph, ...)
├── evaluation/       # run an agent on the benchmark (prover) and check its proofs (verifier)
├── analysis/         # accuracy / tier / token-cost reports over verified runs
├── translation/      # the Charon + Aeneas based Rust → Lean translator
├── comparator/       # vendored Comparator proof judge
├── aeneas/           # Aeneas submodule (pinned in toolchain.toml)
├── scripts/          # benchmark maintenance (re-translate, build-check everything)
└── utils/            # toolchain setup helpers
```

## 1. Set up the environment

**Prerequisites:** Linux, Python ≥ 3.11, Git,
[elan](https://github.com/leanprover/elan) (Lean 4), a Rust toolchain
(`rustup`), OCaml/opam (to build Aeneas and Charon),
[uv](https://docs.astral.sh/uv/) (for `uvx lean-lsp-mcp`), and
[bubblewrap](https://github.com/containers/bubblewrap) (`bwrap`), which
sandboxes the agent.

```bash
git clone --recursive <this repo> autospec && cd autospec
pip install -r requirements.txt

# Check out and build the pinned Aeneas + Charon commits, then build the Lean
# project (Mathlib + Aeneas backend). The first build takes a while.
python3 utils/setup_toolchain.py --install
python3 utils/setup_toolchain.py            # verify: should report everything OK
```

You only need the translator (`translation/`) to add or re-translate cases.
The benchmark already ships with the generated Lean files. See
[`lean_mode/README.md`](lean_mode/README.md) and
[`lean_mode/TESTCASE_GUIDE.md`](lean_mode/TESTCASE_GUIDE.md) to write new cases.

Next, install the agent you want to evaluate. Four CLIs are supported:
[Claude Code](https://docs.anthropic.com/en/docs/claude-code) (`claude`),
[Codex](https://github.com/openai/codex) (`codex`), Mistral Vibe (`vibe`) and the
DeepSeek Harness (`dsh`). The agent gets Lean feedback from a per-worker
[lean-lsp-mcp](https://github.com/oOo0oOo/lean-lsp-mcp) server.

## 2. Evaluate an agent

Copy the example config and fill in the agent:

```bash
mkdir -p evaluation/local          # gitignored: safe for API keys
cp evaluation/config.example.toml evaluation/local/config.toml
```

The fields you need to set:

```toml
[prover]
output_dir = "results/my-agent"    # where the proofs go
backend = "claude"                 # claude | codex | vibe | dsh

[prover.agent.claude]
model = "claude-opus-4-5"
api_key = ""                       # or leave empty to inherit the environment
max_turns = 100
timeout = 7200                     # seconds per case

[prover.sandbox]
ro_paths = ["/home/YOU/.elan", "/home/YOU/.local/bin", ...]  # what the CLI needs to run
```

Then run the two stages:

```bash
# 1) Prove: builds the workspace mirror once (evaluation/local/base), then runs
#    `workers` agents in parallel, one sandboxed workspace per case.
python -m evaluation.manager --config evaluation/local/config.toml
#    (--skip-existing resumes an interrupted run)

# 2) Verify: re-checks every saved proof with Comparator and writes verdict.txt.
python -m evaluation.verifier --config evaluation/local/config.toml
```

Each case in `output_dir` ends up with:

```
results/my-agent/array_binary_search/
├── Function.lean    # the Aeneas translation the agent was given
├── Spec.lean        # the agent's final proof
├── status.txt       # ok, build_failed (case did not compile) or error: <reason>
├── trace/           # the agent's raw session trace (for token accounting)
└── verdict.txt      # PROVED | UNPROVED | BUILD_FAIL | CHEAT | KERNEL_FAIL
```

You can try a single case first by pointing `input_dir` at a directory that
holds only that case. For the backend options, retries/resume, sandbox mounts
and the verifier modes, see [`evaluation/README.md`](evaluation/README.md).
Run the pipeline's own tests with
`python -m unittest evaluation.test_verifier evaluation.test_isolation`.

## 3. Analyze the results

`python3 -m analysis` reads one or more verified runs. Pass each run with
`--run LABEL=DIR`, or pass a directory of runs with `--runs-dir`:

```bash
python3 -m analysis --run my-agent=results/my-agent            # all reports
python3 -m analysis --runs-dir results/                        # compare every run in results/
python3 -m analysis summary category --runs-dir results/       # selected reports
python3 -m analysis tokens --runs-dir results/ --prices analysis/prices.example.toml
python3 -m analysis --list
```

| report     | output |
|------------|--------|
| `summary`  | verdict counts and accuracy per run |
| `category` | PROVED rate per benchmark category + `accuracy_by_category.png` |
| `tiers`    | accuracy on the beginner (93) / advanced (66) difficulty tiers |
| `radar`    | per-tier, per-category radar charts `radar_tier{1,2}.png` |
| `tokens`   | token usage, dollar cost (with `--prices`), cost-vs-solved curves |

Figures go to `analysis/out/` (change it with `--out`). Example `summary`
output:

```
run              PROVED    UNPROVED  BUILD_FAIL       CHEAT KERNEL_FAIL   total  accuracy
-----------------------------------------------------------------------------------------
my-agent            116          38           5           0           0     159     73.0%
```
