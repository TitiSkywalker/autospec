# Analysis

This tool summarises evaluation results as accuracy and cost reports. It reads runs that have already been verified. A run is the output directory of an evaluation, in which the verifier has written a verdict.txt for every case. See the [main README](../README.md) for how to run an evaluation.

The tool uses only the Python standard library, plus matplotlib for the figures.

## Selecting runs

You can pass each run as --run LABEL=DIR, or pass --runs-dir with a directory whose sub-directories are runs. Each sub-directory is then labelled by its name. The two options can be combined. The labels appear in the tables and figure legends.

```bash
python3 -m analysis --run my-agent=results/my-agent              # one run
python3 -m analysis --runs-dir results/                          # every run in results/
```

## Reports

All reports run by default. To run only some of them, list their names:

```bash
python3 -m analysis summary tiers --runs-dir results/
```

| Report | Description |
|---|---|
| summary | number of cases proved, unproved or rejected in each run, and its accuracy |
| category | accuracy for each benchmark category, with a bar chart |
| tiers | accuracy on the beginner (93 cases) and advanced (66 cases) tiers, and the program sizes in each tier |
| radar | radar charts of accuracy per category, one for each tier |
| tokens | token usage for each run, and dollar cost when prices are given |

Accuracy is computed over the cases present in a run. A case that failed before producing a verdict is excluded from the total.

## Options

| Option | Default | Description |
|---|---|---|
| --out DIR | analysis/out/ | where figures are saved |
| --benchmark DIR | lean_mode/LeanMode | the benchmark directory, used to measure tier sizes |
| --prices TOML | none | a price table for the tokens report |
| --list | | list the available reports and exit |

## Token usage and cost

The tokens report reads the trace directory of each case. It supports the session logs of Claude Code, Codex, Mistral Vibe and the DeepSeek Harness, and skips cases without a trace.

To report dollar costs, provide a price table in USD per million tokens, with one entry per run label. [prices.example.toml](prices.example.toml) shows the format:

```toml
["my-agent"]
input = 5.0
output = 25.0
cache_read = 0.5     # optional, defaults to the input price
cache_write = 6.25   # optional, defaults to the input price
```

```bash
python3 -m analysis tokens --runs-dir results/ --prices my-prices.toml
```

Runs without a price entry are reported in tokens only.

## Difficulty tiers

The two tiers are a fixed, hand-curated split of the benchmark. Within each category, cases are split at a clear gap in program size, measured as the combined line count of the Rust code and its Lean translation. The split is independent of any agent's results. The cut points are listed in [accuracy/tiers.py](accuracy/tiers.py).

If cases are added to or removed from the benchmark, the tiers and radar reports stop with an error that lists the cases to assign. This keeps the split from changing silently.
