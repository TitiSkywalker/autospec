# Evaluation Pipeline

## Workspace

This directory implements the agentic evaluation pipeline for the Rust vericoding benchmark. Each agent should work in an independent workspace with configured Aeneas toolchain. For example, a typical workspace layout should look like:

```
- .lake
- aeneas
- RustVerif
    - Basic.lean
    - Function.lean
    - Spec.lean
- RustVerif.lean
- lean-toolchain
- lakefile.toml
```

This workspace can be created by the `create_workspace` function inside `evaluation/workspace.py`.

## Agent

The workspace defined above should be sufficient for verifying functions. Initially, `Function.lean` and `Spec.lean` are empty Lean files. For each testcase, we copy the translated Lean function and specification into `Function.lean` and `Spec.lean`. 

To enable concurrent processing, we initialize multiple independent workspaces, and route testcases in the benchmark to each workspace. This should be modeled as a "Producer-Consumer" model, where producer is the benchmark, and consumer is each workspace (with agent). They are connected with a single queue with lock.

The Lean proving agent could be any system that runs inside a terminal. We simply invoke it inside the workspace using cli command with prompt. For example, Leanstral-1.5 could be invoked by

```
vibe -p "Prove the theorems inside `RustVerif/Spec.lean`" --agent lean --auto-approve --max-turns N
```

Three agent backends are supported, all listed in one config file under
`[prover.agent.vibe]`, `[prover.agent.claude]`, and `[prover.agent.codex]`; the
`backend` key under `[prover]` selects which one to run (see
`evaluation/agents.py`). Each table takes `command`, `prompt`, `model`,
`base_url`, `api_key`, `max_turns`, `args`, `timeout`, and `output`:

```toml
[prover]
backend = "vibe"            # or "claude" / "codex"

[prover.agent.vibe]
command = "vibe"
model = "leanstral"                      # -> VIBE_ACTIVE_MODEL
base_url = "http://localhost:9000/v1"    # -> provider api_base in .vibe/agents/
max_turns = 50                           # -> --max-turns 50

[prover.agent.claude]
command = "claude"
model = "claude-opus-4-8"                # -> --model
base_url = ""                            # -> ANTHROPIC_BASE_URL (empty = default)
api_key = ""                             # -> ANTHROPIC_AUTH_TOKEN (empty = inherit)
max_turns = 50                           # -> CLAUDE_CODE_MAX_TURNS

[prover.agent.codex]
command = "codex"
model = "gpt-5-codex"                    # -> -m
base_url = ""                            # -> self-contained provider (empty = inherit ~/.codex/config.toml)
api_key = ""                             # -> CODEX_API_KEY (empty = inherit shell env)
max_turns = 50                           # -> pipeline-level tool-call cap (see note below)
```

- `vibe` — the Leanstral CLI, streaming via `--output streaming`, trace in
  `.vibe/sessions/`. `model`/`base_url` override the values in
  `vibe_dir/agents/*.toml` (leave empty to use those defaults).
- `claude` — the Claude Code CLI, streaming via `--output-format stream-json`,
  trace in `~/.claude/projects/...`. It is pointed at the per-workspace
  lean-lsp-mcp HTTP server with an inline `--mcp-config`. `model` becomes
  `--model`, `base_url` becomes `ANTHROPIC_BASE_URL`, `api_key` becomes the
  `ANTHROPIC_AUTH_TOKEN` bearer token, and `max_turns` becomes
  `CLAUDE_CODE_MAX_TURNS` (empty fields fall back to the ambient shell env).
  Claude Code has no `--max-turns` flag; use `max_turns` (turns) or add
  `--max-budget-usd <amount>` to `args` (cost).
- `codex` — the OpenAI Codex CLI, streaming via `--json`, trace in
  `~/.codex/sessions/YYYY/MM/DD/`. It is pointed at the per-workspace
  lean-lsp-mcp HTTP server with `-c mcp_servers.lean-lsp.url=...`. `model`
  becomes `-m`; when `base_url` is set, Codex routes through a self-contained
  `pipeline` provider (with `env_key = "CODEX_API_KEY"`) so `base_url` and
  `api_key` fully specify the endpoint/token (empty `base_url` inherits
  `~/.codex/config.toml`). `api_key` becomes the `CODEX_API_KEY` env var (empty
  = inherit the shell env). **Codex has no native turn cap**, so the pipeline
  enforces `max_turns` itself: it kills the process once the number of completed
  tool calls exceeds `max_turns` (Codex's stream has no turn boundary, so
  "turns" ≈ tool calls here). `timeout` remains the wall-clock backstop.

> Optionally, we should setup SKILLs and MCP servers for agents.

## Evaluation

Each testcase is a Rust-Lean project. It contains Rust source and translated Lean source. For example, the testcase `array_binary_search` has the following structure (omitting irrelevant files):

```
.
├── ArrayBinarySearch.lean
├── Spec.lean
├── Test_test.lean
├── src
│   └── lib.rs
└── tests
    └── test.rs
```

During evaluation, `ArrayBinarySearch.lean` and `Spec.lean` should be extracted and copied into workspace directory. This also includes fixing the imports of Lean files.

After the agent stops, we need to check the `Spec.lean` independently to prevent agent from cheating. We collect and save the modified `Spec.lean` to another independent directory for verification.

## Verification

The proving pipeline labels a case `PROVED` only from a `sorry`/`admit` regex
heuristic and never compiles the agent's final `Spec.lean`. Run the independent
verifier after proving to actually typecheck each saved proof:

```
python -m evaluation.verifier --config evaluation/local/config.toml
```

Each verifier worker copies the base mirror into its own workspace
(`[verifier].worker_dir`), compiles `<case>/Function.lean` + `<case>/Spec.lean`
with `lake build`, and writes `<case>/verdict.txt` (plus `<case>/build.log` on
failure). Verdicts:

- `PROVED` — compiles and no `sorry`/`admit` remain.
- `UNPROVED` — compiles but the spec still has `sorry`/`admit`.
- `BUILD_FAIL` — does not compile (broken/inconsistent proof).
