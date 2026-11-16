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

Four agent backends are supported, all listed in one config file under
`[prover.agent.vibe]`, `[prover.agent.claude]`, `[prover.agent.codex]`, and
`[prover.agent.dsh]`; the `backend` key under `[prover]` selects which one to
run (see `evaluation/agents.py`). Each table takes `command`, `prompt`, `model`,
`base_url`, `api_key`, `max_turns`, `retries`, `retry_backoff`, `args`,
`timeout`, and `output`:

```toml
[prover]
backend = "vibe"            # or "claude" / "codex" / "dsh"

[prover.agent.vibe]
command = "vibe"
model = "leanstral"                      # -> VIBE_ACTIVE_MODEL
base_url = "http://localhost:9000/v1"    # -> provider api_base in .vibe/agents/
max_turns = 50                           # -> --max-turns 50
retries = 3                              # -> resume-retries after an abnormal exit
retry_backoff = 10.0                     # -> base backoff seconds (exponential + jitter)

[prover.agent.claude]
command = "claude"
model = "claude-opus-4-8"                # -> --model
base_url = ""                            # -> ANTHROPIC_BASE_URL (empty = default)
api_key = ""                             # -> ANTHROPIC_AUTH_TOKEN (empty = inherit)
max_turns = 50                           # -> pipeline-level tool-call cap (see note below)

[prover.agent.codex]
command = "codex"
model = "gpt-5-codex"                    # -> -m
base_url = ""                            # -> self-contained provider (empty = inherit ~/.codex/config.toml)
api_key = ""                             # -> CODEX_API_KEY (empty = inherit shell env)
max_turns = 50                           # -> pipeline-level tool-call cap (see note below)

[prover.agent.dsh]
command = "dsh"
model = "deepseek-flash"                 # -> agent-default-model (via --patch overlay)
base_url = ""                            # -> DEEPSEEK_BASE_URL (empty = https://api.deepseek.com/anthropic)
api_key = ""                             # -> DEEPSEEK_API_KEY (empty = inherit shell env / stored creds)
max_turns = 50                           # -> pipeline-level tool-call cap (see note below)
```

- `vibe` — the Leanstral CLI, streaming via `--output streaming`, trace in
  `.vibe/sessions/`. `model`/`base_url` override the values in
  `vibe_dir/agents/*.toml` (leave empty to use those defaults). `max_turns`
  becomes `--max-turns` (vibe's native turn cap).
- `claude` — the Claude Code CLI, streaming via `--output-format stream-json`,
  trace in `~/.claude/projects/...`. It is pointed at the per-workspace
  lean-lsp-mcp HTTP server with an inline `--mcp-config`. `model` becomes
  `--model`, `base_url` becomes `ANTHROPIC_BASE_URL`, and `api_key` becomes the
  `ANTHROPIC_AUTH_TOKEN` bearer token (empty fields fall back to the ambient
  shell env). `max_turns` is enforced by the pipeline as a tool-call cap (each
  `tool_use` block counts as one call; `CLAUDE_CODE_MAX_TURNS` is not set). Add
  `--max-budget-usd <amount>` to `args` for a cost backstop.
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
- `dsh` — the DeepSeek Harness CLI, one-shot `headless` profile, streaming via
  `--json` (newline-delimited run events), sessions under
  `$DSH_HOME/sessions/<project-key>/<session-id>/` (`~/.dsh` by default). Its
  model/provider and the per-workspace lean-lsp-mcp server are injected through
  a per-run `--patch` overlay written inside the disposable workspace: `model`
  overrides the `agent-default-model` entry (provider `deepseek-official`), and
  an `@deepseek-ai/dsh-mcp-client` entry points at
  `http://<host>:<port>/mcp`. `base_url` becomes the `DEEPSEEK_BASE_URL` env var
  (the DeepSeek adapter's Anthropic-Messages endpoint root; empty = the official
  `https://api.deepseek.com/anthropic`), `api_key` becomes `DEEPSEEK_API_KEY`
  (empty = inherit shell env / stored credentials), and `DSH_PERMISSION_MODE` is
  forced to `danger-full-access` so the headless run never blocks on approval.
  **dsh has no native turn cap**, so — like Codex — the pipeline enforces
  `max_turns` itself by killing the process once completed tool calls exceed the
  cap. `timeout` remains the wall-clock backstop.

### Agent filesystem isolation

The prover now requires Linux `bwrap` (Bubblewrap) by default. It starts both
the agent and its lean-lsp-mcp server in separate mount/PID/IPC namespaces.
Only the current workspace is writable; `/usr`, `/bin`, `/lib*`, a small set
of `/etc` runtime/TLS files, and explicitly listed runtime paths are readable.
The agent gets a private `HOME` in `ws-*/.agent-home`; MCP gets a separate
one. Neither process sees the input benchmark, saved results, other workers,
or the runner's home by default. Symlinks in the workspace cannot escape the
mount namespace. Sessions/retries stay in the case's private home.

Set the paths needed by **your installed CLI and Lean toolchain** (real
absolute paths; avoid mounting a parent containing benchmark/results):

```toml
[prover.sandbox]
enabled = true                 # default; startup fails if bwrap/mounts fail
fresh_workspace = true         # default; copy the base anew between cases
ro_paths = [
  "/home/YOU/.local/bin",       # vibe/codex/uvx binaries, if installed here
  "/home/YOU/.local/share/uv",  # uvx tools, if needed by MCP
  "/home/YOU/.elan",            # Lean toolchains, if installed here
  # "/path/to/python/env",      # interpreter targeted by a CLI script/venv
  # "/home/YOU/.codex/packages/standalone", # codex symlink target, if needed
  # "/home/YOU/.nvm/versions/node/vXX", # claude/dsh Node runtime, if needed
]
home_files = []                 # optional individual CLI auth/config files
# e.g. [".claude.json", ".codex/auth.json", ".codex/config.toml"]
mcp_cache_dirs = [".cache/uv"]  # default; runner package caches for the MCP
mcp_home_files = [".config/uv/uv.toml"]  # default; runner MCP config files
```

`mcp_cache_dirs` are mounted **read-write** into the MCP server's private home
(`ws-*/.mcp-home`), because `uvx` takes lock files and appends to its cache;
`mcp_home_files` are mounted read-only. Both names are relative to the runner's
home, missing entries are silently skipped, and neither is ever visible to the
agent. Without them a sandboxed `uvx lean-lsp-mcp` re-resolves its environment
on every launch and, if PyPI is only reachable through a mirror configured in
`~/.config/uv/uv.toml`, never binds the port — the worker then fails with
`did not become ready`. Preinstalling the tool (`uv tool install
lean-lsp-mcp`, then `command = ["lean-lsp-mcp"]`) avoids `uvx` resolution
entirely and needs no cache mount, but its interpreter must still be covered by
`ro_paths`.

Supply credentials via the configured `api_key`/environment or explicitly
mount *individual* auth files in `home_files`. The CLI executable and any
interpreters/modules it needs must be covered by the runtime mounts; missing
paths are errors, never an unsandboxed fallback. To diagnose an inaccessible
runtime, inspect the exit error and extend `ro_paths` narrowly. The MCP
server may need a preinstalled `uvx` package or an additional runtime mount.
`fresh_workspace` copies the (large) base mirror for each case because
`git clean` alone cannot purge ignored build directories or nested Git repos
an agent might use to leak a previous answer. Set it to `false` only for
trusted agents or if you accept that cross-case contamination risk. Setting
`enabled = false` restores the old, **unisolated** execution.

This is filesystem isolation, not a network firewall: the agent can still
reach its model API, the MCP port, and other services reachable from the
runner's network. Do not expose benchmark answers over those services; use
a separate network namespace/proxy for a stronger untrusted-host threat model.
If running the separate verifier on adversarial agent-written Lean, set
`[verifier] sandbox = true` with real `landrun`; the default verifier shim is
not an isolation boundary, and legacy `lake` verification is not sandboxed.
Check the boundary locally with
`python -m unittest evaluation.test_isolation`.

**Resilience.** A non-zero agent exit that the pipeline didn't cause itself
(timeout / `max_turns` cap) is treated as a backend failure. When `retries >
0`, the prover re-launches the agent with its CLI resume flag — `vibe
--resume`, `claude --resume`, `codex exec resume`, `dsh --session-id <id>` — after an exponential
backoff with jitter (`retry_backoff` seconds, doubling each attempt). The
resume session id is read from the newest session trace, so each backend keeps
its proof progress across a transient vLLM / external-API outage instead of
losing it. `retries = 0` (the default) disables this.

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
