"""HMM-based behavioural analysis of proving agents.

``agent_behavior`` fits a categorical HMM over the pooled tool-call sequences and
compares agents in the resulting latent phase space; ``hmm_heatmaps`` renders the
fitted emission/transition matrices as PNG heatmaps.
"""

from analysis.agent_trace.hmm.agent_behavior import (  # noqa: F401
    CATEGORIES,
    CAT_IDX,
    CategoricalHMM,
    RUNS,
    _category_of,
    _iter_case_tool_calls,
    _read_zstd,
    _state_label,
    _to_indices,
    case_sequences,
    discover_cases,
    group_transition_test,
)
