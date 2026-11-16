"""Share of loop/recursion/invariant theorems among the largest theorems.

Two classifiers, reported side by side:

* ``name``    -- the theorem's own name contains a loop token (``loop``, ``loop0``,
                 ``inner``, ``outer``), a recursion token (``rec``, ``recursion``)
                 or an invariant token (``inv``, ``invariant``, camel-case ``Inv``).
* ``premise`` -- the theorem cites a translated loop or recursive function
                 (a premise whose last name component matches ``*_loop<d>*`` or ``*_rec``),
                 which is independent of how the agent named its lemma.

Usage: python3 -m analysis.proof_tree.loop_share [cache_namespace]
"""

from __future__ import annotations

import json
import re
import sys

from analysis.proof_tree.config import CACHE_DIR, ROOT
from analysis.proof_tree.stats import _category

NAME_TOKENS = re.compile(r"^(loop\d*|loops|inner|outer|rec|recursion|recursive|inv|invariant|invariants)$")
PREMISE_LOOP = re.compile(r"(_loop\d*|_rec)(_|$)|^loop\d*$")


def _tokens(name: str) -> list[str]:
    spaced = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    return [t.lower() for t in re.split(r"[._']", spaced) if t]


def by_name(theorem: dict) -> bool:
    return any(NAME_TOKENS.match(t) for t in _tokens(theorem["theorem"]))


def by_premise(theorem: dict) -> bool:
    prem = theorem.get("premise_names", theorem.get("premises", []))
    return any(PREMISE_LOOP.search(p.split(".")[-1]) for p in prem)


def load(namespace: str) -> list[dict]:
    out = []
    for f in sorted((ROOT / CACHE_DIR / namespace).glob("*.json")):
        r = json.loads(f.read_text())
        if r.get("verdict") == "PROVED":
            out += [dict(t, case=f.stem) for t in r.get("theorems", [])]
    return out


def shares(theorems: list[dict], classify, top: float = 0.10) -> dict:
    ranked = sorted(theorems, key=lambda t: -t["tactics"])
    k = round(len(ranked) * top)
    hi, rest = ranked[:k], ranked[k:]
    pct = lambda xs: 100 * sum(map(classify, xs)) / len(xs)
    tot = sum(t["tactics"] for t in theorems)
    return {
        "n": len(ranked), "top_k": k,
        "top": pct(hi), "rest": pct(rest), "all": pct(ranked),
        "tactics_in_loop": 100 * sum(t["tactics"] for t in ranked if classify(t)) / tot,
    }


def main() -> None:
    ths = load(sys.argv[1] if len(sys.argv) > 1 else "opus-5.5")
    for label, fn in (("name", by_name), ("premise", by_premise), ("name or premise", lambda t: by_name(t) or by_premise(t))):
        s = shares(ths, fn)
        print(f"{label:<16} top10%={s['top']:5.1f}%  rest={s['rest']:5.1f}%  all={s['all']:5.1f}%  "
              f"tactics-in-loop={s['tactics_in_loop']:5.1f}%  (n={s['n']}, top_k={s['top_k']})")
    print("\nper category (name classifier): % of theorems, % of tactics")
    for c in sorted({_category(t["case"]) for t in ths}):
        sub = [t for t in ths if _category(t["case"]) == c]
        tot = sum(t["tactics"] for t in sub)
        print(f"  {c:<8} {100*sum(map(by_name, sub))/len(sub):5.1f}%  {100*sum(t['tactics'] for t in sub if by_name(t))/tot:5.1f}%")


if __name__ == "__main__":
    main()
