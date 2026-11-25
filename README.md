# AutoSpec

AutoSpec is a benchmark and toolchain for verifying Rust programs with AI agents. It translates Rust code into Lean 4 using [Charon](https://github.com/AeneasVerif/charon) and [Aeneas](https://github.com/AeneasVerif/aeneas), and pairs each translation with a hand-written specification. 

```
.
├── translation/      # Rust → Lean translator
├── lean_mode/        # the benchmark: 159 translated Rust crates
├── evaluation/       # runs an agent on the benchmark
├── analysis/         # accuracy and cost reports
├── comparator/       # proof judge
├── aeneas/           # pinned Aeneas submodule
└── scripts/, utils/  # benchmark maintenance and toolchain setup
```

## Leaderboard


| Model | Agent | Beginner | Advanced | Overall |
|---|---|---:|---:|---:|
| Claude Opus 5.5 | Claude Code | 89.2% | 50.0% | 73.0% |
| GPT-6-sol | Codex | 25.8% | 9.1% | 18.9% |
| GPT-5.6-sol | Codex | 17.2% | 6.1% | 12.6% |
| DeepSeek-V4.1-Flash | DeepSeek Harness | 16.1% | 3.0% | 10.7% |
| GLM-5.3 | Claude Code | 16.1% | 1.5% | 10.1% |
| Kimi-K3 | Claude Code | 10.8% | 0.0% | 6.3% |
| Claude Opus 4.5 | Claude Code | 6.5% | 0.0% | 3.8% |
| Leanstral-1.5 | Mistral Vibe | 4.3% | 0.0% | 2.5% |

Every agent had 100 tool calls per case (Leanstral-1.5 up to 400 turns).

## Setup
Prerequisites: 
- Linux, Python ≥ 3.11, Git
- Lean 4 ([elan](https://github.com/leanprover/elan))
- Rust ([rustup](https://rustup.rs/))
- OCaml ([opam](https://opam.ocaml.org/))
- To evaluate agents: [uv](https://docs.astral.sh/uv/) for Lean LSP MCP, and [bwrap](https://github.com/containers/bubblewrap) for sandbox

How to setup the environment:

```bash
git clone --recursive https://github.com/TitiSkywalker/autospec.git && cd autospec
pip install -r requirements.txt

# OCaml 5 with the packages Aeneas needs
opam switch create aeneas 5.3.0 && eval $(opam env --switch=aeneas)
opam install calendar core_unix domainslib easy_logging menhir ocamlformat.0.27.0 ocamlgraph odoc ppx_deriving ppx_deriving_yojson progress unionFind visitors yojson zarith

# Build the pinned Charon + Aeneas + Lean project
python3 utils/setup_toolchain.py --install
python3 utils/setup_toolchain.py            # should print "Toolchain OK"
```

## Translating a Rust project

Our translation toolchain can be applied to any Cargo crate:

```bash
(cd translation/translator && cargo build --release)
translation/translator/target/release/translator path/to/my_crate
```

This writes MyCrate.lean next to the crate's Cargo.toml, and translates tests/test.rs into Test_test.lean if it exists.

Feel free to translate your own Rust functions! See [translation/README.md](translation/README.md) for how numbers are represented and which Rust features are supported.

## The benchmark

Inside `lean_mode/LeanMode/` we provide 159 cases across nine categories: arrays, cryptography, dynamic programming, data structures, graphs, integers, matrices, strings, and tensors. Each case is a Rust crate plus its translation and a Spec.lean whose theorems are left as `sorry`:

```
array_merge_sort/
├── src/lib.rs, tests/test.rs   # the Rust program and its tests
├── ArrayMergeSort.lean         # generated translation
├── Test_test.lean              # generated #eval tests
└── Spec.lean                   # the specification to prove
```

To re-translate and re-check the whole benchmark after changing the toolchain:

```bash
scripts/translate_all.sh
python3 scripts/check_all.py
```

## Evaluating an agent

The pipeline supports Claude Code, Codex, Mistral Vibe and the DeepSeek Harness. Each agent works in its own sandboxed Lean workspace with a [lean-lsp-mcp](https://github.com/oOo0oOo/lean-lsp-mcp) server for feedback. 

Start from the example config:

```bash
mkdir -p evaluation/local
cp evaluation/config.example.toml evaluation/local/config.toml
```

Pick a backend, set the model and credentials, and list the paths your agent CLI and Lean toolchain need inside the sandbox:

```toml
[prover]
input_dir = "lean_mode/LeanMode"       # cases to prove
output_dir = "results/my-agent"
backend = "claude"                     # claude | codex | vibe | dsh

[prover.agent.claude]
model = "claude-opus-5-5"
max_turns = 100
timeout = 7200                         # seconds per case

[prover.sandbox]
ro_paths = ["/home/YOU/.elan", "/home/YOU/.local/bin"]
```

Then prove and verify:

```bash
python -m evaluation.manager  --config evaluation/local/config.toml
python -m evaluation.verifier --config evaluation/local/config.toml
```

The verifier uses the Comparator judge, which checks that the theorem statements are unchanged, that no `sorry` or extra axioms remain, and that the Lean kernel accepts the proof.  See [`evaluation/README.md`](evaluation/README.md) for backend options, retries, and sandbox details.

## Analyzing the results

Summarize one or more verified runs with:

```bash
python3 -m analysis --run my-agent=results/my-agent      # one run
python3 -m analysis --runs-dir results/                  # compare every run in results/
```

See [analysis/README.md](analysis/README.md) for the available reports and options.
