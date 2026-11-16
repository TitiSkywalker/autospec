"""The evaluation runs every accuracy report compares ("the aligned agents").

Each agent has been run on the same benchmark and verified with the same
Comparator verifier, so their results dirs are directly comparable. Per-report
cosmetics (price table, tier-split input, colours) stay in the report module;
only the set of runs lives here.
"""

# Display label -> results directory (relative to the repository root).
RUNS = {
    "Opus-5.5":            "evaluation/local/runs/main/opus-5.5",
    "Opus-4.5":            "evaluation/local/runs/main/opus-4.5",
    "GPT-6-sol":           "evaluation/local/runs/main/gpt-6-sol",
    "GPT-5.6-sol":         "evaluation/local/runs/main/gpt-5.6-sol",
    "DeepSeek-V4.1-Flash": "evaluation/local/runs/main/deepseek-v4.1-flash",
    "GLM-5.3":             "evaluation/local/runs/main/glm-5.3",
    "Kimi-K3":             "evaluation/local/runs/main/kimi-k3",
    "Leanstral-1.5":       "evaluation/local/runs/main/leanstral-1.5",
}
