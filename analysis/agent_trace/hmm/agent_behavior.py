"""Unsupervised behavioural comparison of proving agents (HMM "colouring").

``agent_trace.py`` measures *what* each tool call does (read/write/build/...)
with hand-written rules. This script goes a step further and asks: can we
*infer* a small set of latent "phases" purely from the data, then use those
phases to compare two agents (Claude-Opus vs dsh/DeepSeek) in a shared,
human-interpretable vocabulary?

Approach
--------
1. Every tool call of every trace is reduced to one of five discrete symbols —
   ``read`` / ``write`` / ``build`` / ``mcp`` / ``other`` — using the same
   classification as ``agent_trace.classify_bash`` (so the two agents are
   measured on an identical alphabet; dsh's native ``read``/``edit``/``write``
   tools are mapped to the same symbols as Claude's ``Bash``/``Read``/``Edit``).
2. A **categorical hidden Markov model** (discrete emissions, Baum–Welch EM,
   Viterbi decoding) is fit on the pooled sequences. Each hidden state is a
   distribution over the five symbols, so a state "means" a phase (e.g. a
   read-heavy state = *explore*, a write-heavy state = *draft*, an mcp-heavy
   state = *iterate/check*). Decoding assigns every tool call a state — the
   "colouring".
3. The two agents are compared **in that shared latent space**: per-state
   occupancy (who spends how much time in which phase) and a total-variation
   distance summarising the occupancy difference in one number.
4. A **likelihood-ratio test** then asks whether the agents' *dynamics* differ
   given the same phases: emissions are held at the shared ``B`` while each
   agent's transition matrix is refit (restricted Baum–Welch); the pooled
   (shared-``A``) model is the null, the per-agent ``A`` the alternative.

The HMM is implemented here with numpy (no ``hmmlearn``/``sklearn``): the
number of states ``K`` is chosen by BIC over ``2..max_k`` with several random
restarts; ``--k`` fixes it instead.

Usage::

    python3 analysis/agent_trace/hmm/agent_behavior.py               # fit + compare (BIC-selected K)
    python3 analysis/agent_trace/hmm/agent_behavior.py --k 4         # fixed number of states
    python3 analysis/agent_trace/hmm/agent_behavior.py --max-k 6     # search up to 6 states
    python3 analysis/agent_trace/hmm/agent_behavior.py --case array_binary_search   # show the colouring
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from itertools import combinations
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent.parent.parent
sys.path.insert(0, str(ROOT))  # make `analysis.*` importable regardless of cwd

from analysis.agent_trace.agent_trace import classify_bash  # noqa: E402

# ======================= Configuration (edit here) =======================

# name -> (results dir, trace kind). "claude" traces are Claude Code JSONL;
# "dsh" traces are dsh's zstd session log.
RUNS = {
    "claude-opus-5.5": ("evaluation/local/runs/main/opus-5.5", "claude"),
    "deepseek-v4.1-flash": ("evaluation/local/runs/main/deepseek-v4.1-flash", "dsh"),
    "kimi-k3": ("evaluation/local/runs/main/kimi-k3", "claude"),
    "glm-5.3": ("evaluation/local/runs/main/glm-5.3", "claude"),
    "gpt-5.6-sol": ("evaluation/local/runs/main/gpt-5.6-sol", "codex"),
    "gpt-6-sol": ("evaluation/local/runs/main/gpt-6-sol", "codex"),
    "claude-opus-4.5": ("evaluation/local/runs/main/opus-4.5", "claude"),
    "leanstral-1.5": ("evaluation/local/runs/main/leanstral-1.5", "vibe"),
}
# Agents with partial trace coverage ("vibe" = Mistral Vibe messages.jsonl). They are not part
# of the pooled fit or the shared-case intersection (that would shrink it from 159 cases and
# move every number); they are decoded under the fitted phases and get their own transitions.
# Agents decoded under the fitted phases but excluded from the fit itself. Leanstral's
# traces are dominated by language-server calls (55% of its tool calls are mcp, many of
# them lemma search), and pooling it moves the K=5 optimum to a different decomposition
# with two mcp phases, which would change the phase vocabulary for every other agent.
# We therefore keep the shared phases of the seven-agent fit and decode Leanstral in them.
EXTRA_RUNS: dict[str, tuple[str, str]] = {}  # all eight agents are in RUNS and the pooled fit
OUT_DIR = "analysis/figs"   # plot output dir
MAX_K = 5                   # BIC search tries K = 2..MAX_K
N_RESTARTS = 4              # random restarts per K (best log-lik kept)
LRT_RESTARTS = 2            # random restarts for fixed-emission (LRT) refits, plus one warm start
MAX_ITER = 300              # EM iterations per restart (safety cap)
TOL = 1e-5                  # absolute log-likelihood improvement for convergence

# The emission alphabet. Order matters: it is the symbol index.
CATEGORIES = ("read", "write", "build", "mcp", "other")
CAT_IDX = {c: i for i, c in enumerate(CATEGORIES)}

# ========================= (no need to edit below) =========================


# --- data extraction ------------------------------------------------------------


def _category_of(name: str, command: str) -> str:
    """Map one tool invocation to its coarse category (shared alphabet)."""
    if name.lower() == "bash":
        primary, _ = classify_bash(command or "")
        return primary  # read / write / build / other
    if name in ("Read", "read"):
        return "read"
    if name in ("Edit", "Write", "edit", "write"):
        return "write"
    if "mcp__" in name or name.startswith("lean_"):
        return "mcp"
    return "other"


def _vibe_category_of(name: str, command: str) -> str:
    """Category of a Mistral Vibe tool call: its native tool names, else the shared rules.

    Kept separate from ``_category_of`` so the other agents' classification (and hence
    the pooled fit) is untouched.
    """
    if name in ("read_file", "file_outline", "grep"):
        return "read"
    if name in ("write_file", "search_replace"):
        return "write"
    if name.startswith("lean-lsp_"):  # MCP tools are served as ``lean-lsp_<tool>``
        return "mcp"
    return _category_of(name, command)


def _iter_vibe_tool_calls(trace: Path):
    """Yield (tool_name, command, file_path) for each assistant tool call in a Vibe log.

    Only the top-level ``messages.jsonl`` is read: the ``task`` sub-agents under
    ``agents/`` are not traversed, matching how the other agents are parsed.
    """
    with trace.open(errors="replace") as fh:
        for line in fh:
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            if e.get("role") != "assistant":
                continue
            for tc in e.get("tool_calls") or []:
                fn = tc.get("function") or {}
                try:
                    args = json.loads(fn.get("arguments") or "{}")
                except json.JSONDecodeError:
                    args = {}
                if not isinstance(args, dict):
                    args = {}
                yield fn.get("name", ""), args.get("command", ""), args.get("file_path", "")


def _iter_claude_tool_calls(trace: Path):
    """Yield (tool_name, command, file_path) for every tool_use block in a transcript."""
    with trace.open(errors="replace") as fh:
        for line in fh:
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            if e.get("type") != "assistant":
                continue
            for block in (e.get("message") or {}).get("content") or []:
                if not (isinstance(block, dict) and block.get("type") == "tool_use"):
                    continue
                name = block.get("name", "")
                inp = block.get("input") or {}
                cmd = inp.get("command", "") if name == "Bash" else ""
                fp = inp.get("file_path", "") if name in ("Read", "Edit", "Write") else ""
                yield name, cmd, fp


def _read_zstd(path: Path) -> str:
    import zstandard

    with open(path, "rb") as fh:
        return zstandard.ZstdDecompressor().stream_reader(fh).read().decode()


def _iter_dsh_tool_calls(trace: Path):
    """Yield (tool_name, command, file_path) for every tool/call event in a dsh session log."""
    for line in _read_zstd(trace).splitlines():
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("type") != "tool/call":
            continue
        data = e.get("data") or {}
        name = data.get("name", "")
        cmd = ""
        fp = ""
        try:
            args = json.loads(data.get("arguments") or "{}")
        except json.JSONDecodeError:
            args = {}
        if name == "bash":
            cmd = args.get("command", "")
        elif name in ("read", "edit", "write"):
            fp = args.get("file_path", "")
        yield name, cmd, fp


# --- Codex (rollout JSONL) ---------------------------------------------------------

_CODEX_TOOL_RE = re.compile(r"tools\.([A-Za-z0-9_]+)\s*\(")


def _extract_balanced(s: str, open_pos: int, open_ch: str, close_ch: str) -> str | None:
    """Return the balanced ``open_ch..close_ch`` span starting at ``open_pos``.

    Tracks string literals and backslash escapes so nested braces/JSON inside a
    ``cmd:"..."`` string don't truncate the argument object.
    """
    depth = 0
    in_str = False
    esc = False
    i = open_pos
    while i < len(s):
        ch = s[i]
        if esc:
            esc = False
            i += 1
            continue
        if in_str:
            if ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
            i += 1
            continue
        if ch == '"':
            in_str = True
        elif ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return s[open_pos:i + 1]
        i += 1
    return None


def _codex_cmd_from_exec_args(args: str) -> str:
    """Extract the ``cmd:"..."`` field from an ``exec_command({...})`` argument.

    The command is stored as a JSON-escaped string (``\\n`` for newlines, ``\\"``
    for quotes); decode it back to the shell command the model actually ran so the
    downstream heredoc/segment classification sees real newlines.
    """
    m = re.search(r'cmd\s*:\s*"((?:[^"\\]|\\.)*)"', args)
    if not m:
        return ""
    try:
        return json.loads('"' + m.group(1) + '"')
    except json.JSONDecodeError:
        return m.group(1)


def _iter_codex_tool_calls(trace: Path):
    """Yield (tool_name, command, file_path) for one Codex rollout JSONL.

    Codex's surface tool is always ``exec`` (a JavaScript orchestrator); the real
    tool calls are nested inside its ``input`` as ``tools.<name>(<args>)``. We map
    ``exec_command``→Bash (with its ``cmd``), ``apply_patch``→Edit,
    ``write_stdin``→a write (feeding a running Lean process via stdin), and
    ``mcp__lean_lsp__*``→mcp, so the alphabet matches Claude/dsh exactly.
    """
    with trace.open(errors="replace") as fh:
        for line in fh:
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            if e.get("type") != "response_item":
                continue
            payload = e.get("payload") or {}
            if payload.get("type") != "custom_tool_call" or payload.get("name") != "exec":
                continue
            inp = payload.get("input", "")
            for m in _CODEX_TOOL_RE.finditer(inp):
                name = m.group(1)
                region = _extract_balanced(inp, m.end() - 1, "(", ")")
                args = region[1:-1] if region else ""
                if name == "exec_command":
                    yield "Bash", _codex_cmd_from_exec_args(args), ""
                elif name == "apply_patch":
                    yield "Edit", "", ""
                elif name == "write_stdin":
                    # ``chars:"..."`` writes real content into the running process
                    # (→ Edit/write). ``chars:""`` is a "press enter to yield" no-op
                    # the harness emits as a turn boundary, not a unit of work, and no
                    # Claude/dsh harness has an equivalent — so drop it rather than
                    # inflating codex's tool-call count with harness noise.
                    cm = re.search(r'chars\s*:\s*"((?:[^"\\]|\\.)*)"', args)
                    chars = (cm.group(1) or "") if cm else ""
                    if chars:
                        yield "Edit", "", ""
                elif name.startswith("mcp__") or name.startswith("lean_"):
                    yield name, "", ""


def discover_cases(results_dir: Path) -> dict[str, Path]:
    """{case_name: case_dir} for every case with an archived trace."""
    out = {}
    for d in sorted(results_dir.iterdir()):
        if not d.is_dir():
            continue
        tdir = d / "trace"
        if not tdir.is_dir():
            continue
        if list(tdir.glob("*.jsonl")):
            out[d.name] = d
        elif (tdir / "session.v4.jsonl.zstd").is_file():
            out[d.name] = d
    return out


def case_sequences(case_dir: Path, kind: str) -> list[list[str]]:
    """One category-symbol sequence per trace file in the case."""
    if kind == "claude":
        seqs = [[_category_of(n, c) for n, c, _fp in _iter_claude_tool_calls(f)]
                for f in (case_dir / "trace").glob("*.jsonl")]
    elif kind == "dsh":
        f = case_dir / "trace" / "session.v4.jsonl.zstd"
        seqs = [[_category_of(n, c) for n, c, _fp in _iter_dsh_tool_calls(f)]]
    elif kind == "vibe":
        seqs = [[_vibe_category_of(n, c) for n, c, _fp in _iter_vibe_tool_calls(case_dir / "trace" / "messages.jsonl")]]
    else:  # codex: one rollout JSONL per session, nested tools under ``exec``
        seqs = [[_category_of(n, c) for n, c, _fp in _iter_codex_tool_calls(f)]
                for f in (case_dir / "trace").glob("*.jsonl")]
    return [s for s in seqs if s]


def _iter_case_tool_calls(case_dir: Path, kind: str):
    """Yield (tool_name, command, file_path) for every tool call in a case."""
    if kind == "claude":
        for f in (case_dir / "trace").glob("*.jsonl"):
            yield from _iter_claude_tool_calls(f)
    elif kind == "dsh":
        yield from _iter_dsh_tool_calls(case_dir / "trace" / "session.v4.jsonl.zstd")
    elif kind == "vibe":
        yield from _iter_vibe_tool_calls(case_dir / "trace" / "messages.jsonl")
    else:  # codex
        for f in (case_dir / "trace").glob("*.jsonl"):
            yield from _iter_codex_tool_calls(f)


# --- categorical HMM (Baum-Welch + Viterbi, numpy only) -------------------------


def _kmeans_init_B(obs: list[np.ndarray], k: int, m: int, rng: np.random.Generator) -> np.ndarray:
    """Initialize emission rows from a k-means clustering of one-hot symbols.

    Gives EM a better-than-random start (one cluster per dominant symbol mix);
    falls back to random rows if k-means is unavailable/ill-conditioned.
    """
    xs = [np.eye(m)[s] for seq in obs for s in seq]
    if len(xs) < k:
        return rng.dirichlet(np.ones(m), size=k)
    try:
        from scipy.cluster.vq import kmeans2

        # minit="points" picks ``k`` *existing* data points as seeds; when a
        # symbol is rare (e.g. build/mcp) it can leave a cluster empty, which
        # scipy then silently (or via a warning) drives to a near-zero row and
        # erases that phase. minit="++" spreads the seeds instead, so every
        # cluster is non-empty and each emission row stays meaningful.
        centroids, _ = kmeans2(np.vstack(xs), k, iter=10, minit="++", seed=0)
    except Exception:
        return rng.dirichlet(np.ones(m), size=k)
    return np.clip(centroids, 1e-6, None) / np.clip(centroids, 1e-6, None).sum(1, keepdims=True)


class CategoricalHMM:
    """Discrete-emission HMM: pi (K,), A (K,K), B (K,M), all row-stochastic."""

    def __init__(self, k: int, m: int, rng: np.random.Generator):
        self.k, self.m = k, m
        self.rng = rng

    def _init(self, obs: list[np.ndarray]):
        k, m = self.k, self.m
        self.pi = np.full(k, 1.0 / k)
        # A gentle diagonal prior (not a 0.85 spike) biases toward persistent
        # phases *without* trapping EM in the diagonal basin: off-diagonal
        # transitions (e.g. read->write->build cycles) can still emerge if the
        # data supports them. A hard sticky start was absorbing every restart
        # into a pure-diagonal solution and erasing the phase-cycle structure.
        self.A = 0.5 * np.eye(k) + 0.5 * self.rng.dirichlet(np.ones(k), size=k)
        self.B = _kmeans_init_B(obs, k, self.m, self.rng)

    def _fb(self, seq: np.ndarray):
        """Scaled forward-backward; returns (gamma (T,K), xi_sum (K,K), loglik)."""
        pi, A, B = self.pi, self.A, self.B
        T = len(seq)
        K = self.k
        alpha = np.zeros((T, K))
        scale = np.zeros(T)
        alpha[0] = pi * B[:, seq[0]]
        scale[0] = alpha[0].sum() or 1e-300
        alpha[0] /= scale[0]
        for t in range(1, T):
            alpha[t] = (alpha[t - 1] @ A) * B[:, seq[t]]
            scale[t] = alpha[t].sum() or 1e-300
            alpha[t] /= scale[t]

        beta = np.zeros((T, K))
        beta[T - 1] = 1.0
        for t in range(T - 2, -1, -1):
            beta[t] = A @ (B[:, seq[t + 1]] * beta[t + 1])
            beta[t] /= scale[t + 1] or 1e-300

        gamma = alpha * beta
        gamma /= gamma.sum(1, keepdims=True)

        xi_sum = np.zeros((K, K))
        for t in range(T - 1):
            x = alpha[t][:, None] * A * (B[:, seq[t + 1]][None, :] * beta[t + 1][None, :])
            s = x.sum()
            xi_sum += x / (s if s else 1e-300)

        loglik = np.log(scale + 1e-300).sum()
        return gamma, xi_sum, loglik

    def _e_step(self, obs):
        """Accumulate the expected sufficient statistics (gamma, xi) over sequences."""
        A_num = np.zeros_like(self.A)
        B_num = np.zeros_like(self.B)
        pi_num = np.zeros_like(self.pi)
        total = 0.0
        for seq in obs:
            gamma, xi_sum, ll = self._fb(seq)
            total += ll
            pi_num += gamma[0]
            A_num += xi_sum
            for m in range(self.m):
                B_num[:, m] += gamma[seq == m].sum(0)
        return A_num, B_num, pi_num, total

    def fit(self, obs: list[np.ndarray], max_iter=MAX_ITER, tol=TOL):
        self._init(obs)
        prev = -np.inf
        for _ in range(max_iter):
            A_num, B_num, pi_num, total = self._e_step(obs)
            # M-step
            self.pi = pi_num / np.maximum(pi_num.sum(), 1e-300)
            self.A = A_num / np.maximum(A_num.sum(1, keepdims=True), 1e-300)
            self.B = B_num / np.maximum(B_num.sum(1, keepdims=True), 1e-300)
            # relative improvement: |Δll| / |ll| < TOL
            if (total - prev) < tol:
                break
            prev = total
        return total

    def fit_fixed_B(self, B_fixed: np.ndarray, obs: list[np.ndarray],
                    init: tuple[np.ndarray, np.ndarray] | None = None,
                    n_restarts=LRT_RESTARTS, max_iter=MAX_ITER, tol=TOL) -> float:
        """Refit ``pi`` and ``A`` with emissions held at ``B_fixed`` (shared phases).

        This is the group-specific-dynamics model: the phase *meanings* (B) are
        identical across agents, while each agent gets its own start/transition
        parameters. ``init=(A0, pi0)`` warm-starts the first run (e.g. from the
        pooled fit), followed by ``n_restarts`` random restarts. Returns the best
        log-likelihood.
        """
        self.B = np.asarray(B_fixed, dtype=float).copy()
        best = None
        starts = [init] + [None] * n_restarts if init is not None else [None] * n_restarts
        for st in starts:
            if st is not None:
                self.A = np.asarray(st[0], dtype=float).copy()
                self.pi = np.asarray(st[1], dtype=float).copy()
            else:
                self.pi = np.full(self.k, 1.0 / self.k)
                # Match the pooled model's *gentle* diagonal prior (see
                # CategoricalHMM._init) rather than a fully random A, so the
                # per-agent refit starts near the shared-dynamics solution but
                # can still move off the diagonal where the data warrants.
                self.A = 0.5 * np.eye(self.k) + 0.5 * self.rng.dirichlet(np.ones(self.k), size=self.k)
            ll = self._em_fixed_B(obs, max_iter, tol)
            if best is None or ll > best[2]:
                best = (self.A.copy(), self.pi.copy(), ll)
        self.A, self.pi, self.ll = best
        return self.ll

    def _em_fixed_B(self, obs, max_iter, tol) -> float:
        prev = -np.inf
        for _ in range(max_iter):
            A_num, _B_num, pi_num, total = self._e_step(obs)
            self.pi = pi_num / np.maximum(pi_num.sum(), 1e-300)
            self.A = A_num / np.maximum(A_num.sum(1, keepdims=True), 1e-300)
            # emissions stay fixed
            if (total - prev) < tol:
                break
            prev = total
        return total

    def log_likelihood(self, obs):
        return sum(self._fb(seq)[2] for seq in obs)

    def viterbi(self, seq: np.ndarray) -> np.ndarray:
        pi, A, B = self.pi, self.A, self.B
        T = len(seq)
        K = self.k
        logpi = np.log(pi + 1e-300)
        logA = np.log(A + 1e-300)
        logB = np.log(B[:, seq].T + 1e-300)
        delta = np.zeros((T, K))
        psi = np.zeros((T, K), dtype=int)
        delta[0] = logpi + logB[0]
        for t in range(1, T):
            s = delta[t - 1][:, None] + logA
            psi[t] = s.argmax(0)
            delta[t] = s.max(0) + logB[t]
        path = np.zeros(T, dtype=int)
        path[-1] = delta[-1].argmax()
        for t in range(T - 2, -1, -1):
            path[t] = psi[t + 1][path[t + 1]]
        return path


def _to_indices(seqs: list[list[str]], idx_map: dict = CAT_IDX) -> list[np.ndarray]:
    return [np.array([idx_map[c] for c in s]) for s in seqs if s]


def _bic(ll: float, k: int, m: int, n_obs: int) -> float:
    n_params = (k - 1) + k * (k - 1) + k * (m - 1)
    return -2 * ll + n_params * np.log(max(n_obs, 2))


def fit_best(obs: list[np.ndarray], m: int, max_k: int, seed: int = 0) -> tuple[CategoricalHMM, int, float]:
    """Fit HMMs for K=2..max_k with restarts; return (best_hmm, best_k, bic)."""
    best = None
    for k in range(2, max_k + 1):
        rng = np.random.default_rng(seed + k)
        top = None
        for _ in range(N_RESTARTS):
            h = CategoricalHMM(k, m, rng)
            ll = h.fit(obs)  # fit returns the final log-likelihood
            if top is None or ll > top[1]:
                top = (h, ll)
        n_obs = sum(len(s) for s in obs)
        bic = _bic(top[1], k, m, n_obs)
        print(f"  K={k}: loglik={top[1]:.1f}  bic={bic:.1f}", file=sys.stderr, flush=True)
        if best is None or bic < best[2]:
            best = (top[0], k, bic)
    return best


def group_transition_test(B_shared: np.ndarray, k: int, m: int,
                          obs_by_group: dict[str, list[np.ndarray]],
                          init: tuple[np.ndarray, np.ndarray] | None = None,
                          seed: int = 0) -> dict:
    """LRT: shared transition matrix (H0) vs per-agent transition matrices (H1).

    Emissions are fixed to ``B_shared`` in both hypotheses, so the phase
    vocabulary is identical across agents; only the dynamics (``pi``, ``A``)
    are allowed to differ. Returns the likelihood-ratio statistic, its degrees
    of freedom, and the chi-square p-value, plus the fitted per-agent models.
    """
    from scipy.stats import chi2

    pooled = [s for seqs in obs_by_group.values() for s in seqs]
    rng = np.random.default_rng(seed)

    h0 = CategoricalHMM(k, m, rng)
    ll0 = h0.fit_fixed_B(B_shared, pooled, init=init)

    models = {}
    ll1 = 0.0
    for name, obs in obs_by_group.items():
        h = CategoricalHMM(k, m, rng)
        ll1 += h.fit_fixed_B(B_shared, obs, init=(h0.A, h0.pi))
        models[name] = h

    g = len(obs_by_group)
    df = (g - 1) * ((k - 1) + k * (k - 1))  # extra free (pi, A) params per group
    lrt = max(0.0, 2.0 * (ll1 - ll0))
    p = float(chi2.sf(lrt, df))
    return {"lrt": lrt, "df": df, "p": p, "ll0": ll0, "ll1": ll1,
            "models": models, "h0": h0}


# --- comparison report -----------------------------------------------------------


def _decoded_occupancy(hmm: CategoricalHMM, seqs: list[np.ndarray]) -> tuple[np.ndarray, np.ndarray]:
    """Aggregate state occupancy and (from,to) transition counts over sequences."""
    occ = np.zeros(hmm.k)
    trans = np.zeros((hmm.k, hmm.k))
    for seq in seqs:
        if len(seq) == 0:
            continue
        p = hmm.viterbi(seq)
        occ += np.bincount(p, minlength=hmm.k)
        for a, b in zip(p[:-1], p[1:]):
            trans[a, b] += 1
    return occ, trans


def _state_label(b_row: np.ndarray, symbols=CATEGORIES) -> str:
    """A short human label for a state from its emission distribution."""
    top = int(b_row.argmax())
    parts = ", ".join(f"{symbols[i]} {100 * b_row[i]:.0f}%" for i in np.argsort(-b_row)[:2])
    return f"~{symbols[top]} ({parts})"


def _report_baseline(agents: dict[str, list[np.ndarray]]) -> None:
    """Print the model-free facts: raw action mix per agent."""
    names = list(agents)

    # raw category mix per agent
    print("\n--- baseline: raw tool-category mix (% of tool calls) ---")
    print(f"  {'category':<8}" + "".join(f"{a:>18}" for a in names))
    for c in CATEGORIES:
        row = f"  {c:<8}"
        for seqs in agents.values():
            flat = [s for seq in seqs for s in seq]
            row += f"{100.0 * sum(1 for s in flat if s == CAT_IDX[c]) / max(len(flat), 1):>17.1f}%"
        print(row)


def _report_layer(hmm: CategoricalHMM, k: int, bic: float, agents: dict[str, list[np.ndarray]],
                  symbols, title: str) -> None:
    """Print one HMM layer: latent phases, occupancy, and the dynamics LRT."""
    print(f"\n=== {title} (K={k}, BIC={bic:.0f}) ===")

    # emission profiles = the "meaning" of each phase
    print(f"--- latent phases ({title}) ---")
    for s in range(k):
        print(f"  state {s}: {_state_label(hmm.B[s], symbols)}")

    # occupancy per agent, decoded with the shared HMM
    print(f"\n--- state occupancy per agent (% of tool calls, {title}) ---")
    names = list(agents)
    print(f"  {'state':<10}" + "".join(f"{a:>18}" for a in names))
    occs = {name: _decoded_occupancy(hmm, seqs)[0] for name, seqs in agents.items()}
    for s in range(k):
        row = f"  state {s:<5}"
        for name in names:
            o = occs[name]
            row += f"{100.0 * o[s] / max(o.sum(), 1):>17.1f}%"
        print(row)
    # average pairwise total-variation distance between agents' occupancy vectors
    if len(names) >= 2:
        tvs = []
        for a, b in combinations(names, 2):
            oa = occs[a] / max(occs[a].sum(), 1)
            ob = occs[b] / max(occs[b].sum(), 1)
            tvs.append(0.5 * float(np.abs(oa - ob).sum()))
        print(f"  avg pairwise total-variation distance (occupancy): {np.mean(tvs):.3f}")

    # group-specific dynamics: refit per-agent (pi, A) with the shared emissions,
    # and test shared-vs-group transitions with a likelihood-ratio test.
    lrt = group_transition_test(hmm.B, k, len(symbols), agents, init=(hmm.A, hmm.pi))
    print("\n--- transition dynamics (shared emissions, per-agent transitions) ---")
    print(f"  H0 = one shared transition matrix; H1 = per-agent transition matrix")
    print(f"  log-lik: shared {lrt['ll0']:.1f}  per-agent {lrt['ll1']:.1f}")
    print(f"  LRT = {lrt['lrt']:.1f}  df = {lrt['df']}  p = {lrt['p']:.2e}")
    print(f"  -> {'agents differ in dynamics (reject H0)' if lrt['p'] < 0.05 else 'no significant dynamics difference'}")
    print("  transition matrices (rows=from, cols=to, MLE with shared phases):")
    for name in names:
        row_norm = lrt["models"][name].A
        print(f"  {name}:")
        header = "      " + "".join(f"{'to' + str(s):>7}" for s in range(k))
        print(header)
        for s in range(k):
            print(f"    fr{s} " + "".join(f"{row_norm[s, t] * 100:>6.1f}%" for t in range(k)))

    _plot(hmm, k, agents, occs, title)


def _show_colouring(hmm: CategoricalHMM, case_name: str, case_seqs: dict[str, list[np.ndarray]]) -> None:
    """Print the decoded state path for one case, one line per trace."""
    print(f"\n--- colouring for {case_name} (R=read W=write B=build M=mcp O=other) ---")
    for s in range(hmm.k):
        print(f"  state {s} = {_state_label(hmm.B[s])}")
    for name, seqs in case_seqs.items():
        for seq in seqs:
            if len(seq) == 0:
                continue
            path = hmm.viterbi(seq)
            glyphs = "".join(CATEGORIES[s][0].upper() for s in seq)
            states = "".join(str(p) for p in path)
            print(f"  {name} ({len(seq)} calls):")
            print(f"    act: {glyphs}")
            print(f"    st : {states}")


def _plot(hmm, k, agents, occs, label: str = "") -> None:
    try:
        import matplotlib

        matplotlib.use("Agg")
        import matplotlib.pyplot as plt

        names = list(agents)
        n = len(names)
        x = np.arange(k)
        width = 0.8 / n
        fig, ax = plt.subplots(figsize=(8, 4.5))
        for i, name in enumerate(names):
            o = occs[name]
            pct = 100.0 * o / max(o.sum(), 1)
            ax.bar(x + (i - (n - 1) / 2) * width, pct, width, label=name)
        ax.set_xticks(x)
        ax.set_xticklabels([f"state {s}" for s in range(k)])
        ax.set_ylabel("occupancy (%)")
        ax.set_title(f"HMM state occupancy by agent — {label}" if label else "HMM state occupancy by agent")
        ax.legend()
        fig.tight_layout()
        out = Path(OUT_DIR)
        out.mkdir(parents=True, exist_ok=True)
        slug = re.sub(r"[^A-Za-z0-9]+", "_", label).strip("_") if label else "coarse"
        path = out / f"agent_behavior_occupancy_{slug}.png"
        fig.savefig(path, dpi=150)
        plt.close(fig)
        print(f"\noccupancy plot -> {path}")
    except ImportError:
        print("\n(matplotlib unavailable; skipping plot)")


# --- main ------------------------------------------------------------------------


def _fit_layer(obs: list[np.ndarray], m: int, k_fixed: int | None) -> tuple[CategoricalHMM, int, float]:
    """Fit one HMM layer with ``m`` symbols; return (hmm, k, bic)."""
    if k_fixed is not None:
        rng = np.random.default_rng(0)
        hmm = CategoricalHMM(k_fixed, m, rng)
        ll = hmm.fit(obs)
        bic = _bic(ll, k_fixed, m, sum(len(s) for s in obs))
        return hmm, k_fixed, bic
    return fit_best(obs, m, MAX_K)


def main() -> None:
    ap = argparse.ArgumentParser(description="HMM-based behavioural comparison of proving agents")
    ap.add_argument("--k", type=int, default=None, help="fixed number of HMM states (default: BIC over 2..max-k)")
    ap.add_argument("--max-k", type=int, default=MAX_K, help="upper bound for BIC state search")
    ap.add_argument("--case", help="show the per-tool-call colouring for one case")
    args = ap.parse_args()

    cases_by_agent = {name: discover_cases(ROOT / path) for name, (path, _kind) in RUNS.items()}
    shared = set.intersection(*(set(c) for c in cases_by_agent.values()))
    if not shared:
        sys.exit("no cases shared across all runs")

    agents: dict[str, list[np.ndarray]] = {}      # coarse-action index sequences
    case_seqs: dict[str, dict[str, list[np.ndarray]]] = {name: {} for name in RUNS}
    for name, (path, kind) in RUNS.items():
        cases = cases_by_agent[name]
        seqs = []
        for case in sorted(shared):
            idx = _to_indices(case_sequences(cases[case], kind), CAT_IDX)
            case_seqs[name][case] = idx
            seqs.extend(idx)
        agents[name] = seqs

    print(f"\n=== comparing {len(shared)} shared cases ===")
    for name, seqs in agents.items():
        n = sum(len(s) for s in seqs)
        print(f"  {name:<14} {len(seqs):>4} traces, {n:>6} tool calls")

    _report_baseline(agents)

    obs = [s for seqs in agents.values() for s in seqs]

    hmm, k, bic = _fit_layer(obs, len(CATEGORIES), args.k)
    _report_layer(hmm, k, bic, agents, CATEGORIES, "coarse actions")

    if args.case:
        if args.case not in shared:
            sys.exit(f"no such shared case: {args.case}")
        _show_colouring(hmm, args.case, {name: case_seqs[name][args.case] for name in RUNS})


if __name__ == "__main__":
    main()
