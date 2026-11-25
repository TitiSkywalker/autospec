# Analysis

This tool turns verified evaluation runs into accuracy and cost reports. A run is the output directory of an evaluation, after the verifier has written a verdict.txt into every case. See the [main README](../README.md) for how to run an evaluation.

## Usage

Pass the runs you want to compare as arguments. An argument can be a single run, or a directory of runs, in which case every run inside it is included. Each run is labelled by its directory name; write LABEL=DIR to choose a different label.

```bash
python3 -m analysis results/my-agent                    # one run
python3 -m analysis results/                            # every run in results/
```

Tables are printed to the terminal and figures are saved to analysis/out/. Run `python3 -m analysis --help` to see all options.

## Reports

All reports run by default. To run only some, pick them with -r:

```bash
python3 -m analysis results/ -r summary -r tiers
```

| Report | What it shows | Figures |
|---|---|---|
| summary | verdict counts and accuracy for each run | |
| category | accuracy for each benchmark category | accuracy_by_category.png |
| tiers | accuracy on the beginner and advanced tiers | |
| radar | accuracy per category within each tier | radar_tier1.png, radar_tier2.png |
| tokens | token usage, and dollar cost when prices are given | tokens_vs_solved.png, cost_vs_solved.png |

Accuracy counts only the cases present in a run, so a case that failed before producing a verdict is left out of the total.

## Token usage and cost

The tokens report reads each case's trace directory. It understands the session logs of Claude Code, Codex, Mistral Vibe and the DeepSeek Harness, and skips cases without a trace. To get dollar costs, pass a price table with --prices. Prices are in USD per million tokens, with one entry per run label, as in [prices.example.toml](prices.example.toml):

```toml
["my-agent"]
input = 5.0
output = 25.0
cache_read = 0.5     # optional, defaults to the input price
cache_write = 6.25   # optional, defaults to the input price
```

```bash
python3 -m analysis results/ -r tokens --prices my-prices.toml
```

Runs without a price entry are reported in tokens only.

## Difficulty tiers

The benchmark is split into a beginner tier (93 cases) and an advanced tier (66 cases). The split is fixed and chosen by hand: within each category, cases are divided at a clear gap in program size, measured as the line count of the Rust code plus its Lean translation. It does not depend on any agent's results. The cut points are listed in [accuracy/tiers.py](accuracy/tiers.py).

If cases are added to or removed from the benchmark, the tiers and radar reports stop with an error that lists the cases to assign, so the split never changes silently.
