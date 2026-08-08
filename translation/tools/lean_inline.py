#!/usr/bin/env python3
"""
Inline f64_helpers calls in generated Lean code.

The charon→aeneas pipeline cannot handle f64 operations natively, so the
LLBC adapter translates them into f64_helpers.* function calls.  Those
helpers are trivial wrappers:

    def f64_helpers.add (a b : Rat) : Result Rat := ok (a + b)   -- always ok

In a ``do`` block, ``←`` unwraps ``Result α`` → ``α``, and the body
simply returns ``ok (pure-expr)``.  So we can replace the monadic call
with a pure ``:=`` binding:

    let x ← f64_helpers.mul a b    →   let x := a * b
    let y ← f64_helpers.add x c    →   let y := x + c

After inlining all call sites the ``def f64_helpers.*`` definitions are
dead code and are removed.

Usage::

    from lean_inline import inline_f64_helpers
    clean = inline_f64_helpers(lean_source_text)
"""

from __future__ import annotations

import re

# ── operator tables ──────────────────────────────────────────────────────────

# helper name → Lean infix operator
_BINARY: dict[str, str] = {
    "add": "+",
    "sub": "-",
    "mul": "*",
    "div": "/",
}

_COMPARISON: dict[str, str] = {
    "eq": "=",
    "gt": ">",
    "lt": "<",
    "ge": "≥",
    "le": "≤",
}

# helper name → (prefix, suffix)
_UNARY: dict[str, tuple[str, str]] = {
    "neg": ("-", ""),
    "abs": ("|", "|"),
}

# ── public API ───────────────────────────────────────────────────────────────


def inline_f64_helpers(content: str) -> str:
    """Inline all f64_helpers.* calls and remove their definitions.

    Returns the transformed Lean source.
    """
    # Step 1: binary arithmetic (add / sub / mul / div)
    for name, op in _BINARY.items():
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+f64_helpers\.{name}\s+(\S+)\s+(\S+)",
            rf"\1:= \2 {op} \3",
            content,
        )

    # Step 2: comparisons (eq / gt / lt / ge / le)
    for name, op in _COMPARISON.items():
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+f64_helpers\.{name}\s+(\S+)\s+(\S+)",
            rf"\1:= \2 {op} \3",
            content,
        )

    # Step 3: unary (neg / abs)
    for name, (prefix, suffix) in _UNARY.items():
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+f64_helpers\.{name}\s+(\S+)",
            rf"\1:= {prefix}\2{suffix}",
            content,
        )

    # Step 4: from_usize
    content = re.sub(
        r"(let\s+\w+\s+)←\s+f64_helpers\.from_usize\s+(\S+)",
        r"\1:= (\2.val : Rat)",
        content,
    )

    # Step 5: floating-point constants (const_3_14, const_neg_1_0, …)
    content = re.sub(
        r"(let\s+\w+\s+)←\s+f64_helpers\.(const_\w+)",
        lambda m: f"{m.group(1)}:= {_const_to_rat(m.group(2))}",
        content,
    )

    # Step 6: handle tail-position f64_helpers calls (return value of do block)
    for name, op in _BINARY.items():
        content = re.sub(
            rf"(^|\n)(\s+)f64_helpers\.{name}\s+(\S+)\s+(\S+)[ \t]*$",
            rf"\1\2ok (\3 {op} \4)",
            content,
            flags=re.MULTILINE,
        )
    for name, op in _COMPARISON.items():
        content = re.sub(
            rf"(^|\n)(\s+)f64_helpers\.{name}\s+(\S+)\s+(\S+)[ \t]*$",
            rf"\1\2ok (\3 {op} \4)",
            content,
            flags=re.MULTILINE,
        )
    for name, (prefix, suffix) in _UNARY.items():
        content = re.sub(
            rf"(^|\n)(\s+)f64_helpers\.{name}\s+(\S+)[ \t]*$",
            rf"\1\2ok ({prefix}\3{suffix})",
            content,
            flags=re.MULTILINE,
        )
    content = re.sub(
        r"(^|\n)(\s+)f64_helpers\.from_usize\s+(\S+)[ \t]*$",
        r"\1\2ok ((\3.val : Rat))",
        content,
        flags=re.MULTILINE,
    )

    # Step 7: handle ``f64_helpers.neg`` / ``f64_helpers.abs`` that appear
    # outside a let binding (e.g. inside if-then branches or after
    # core.f64.XXX.abs → f64_helpers.abs rewrite).
    content = re.sub(r"f64_helpers\.neg\s+(\S+)", r"ok (-\1)", content)
    content = re.sub(r"f64_helpers\.abs\s+(\S+)", r"|\1|", content)

    # Step 8: remove now-unused ``def f64_helpers.*`` definitions
    content = _remove_f64_helper_defs(content)

    return content


# ── constant conversion ─────────────────────────────────────────────────────


def _const_to_rat(name: str) -> str:
    """Convert a const_* name to a Rat literal.

    >>> _const_to_rat("const_0_5")
    '(5/10 : Rat)'
    >>> _const_to_rat("const_neg_1_0")
    '(-(1/1 : Rat))'
    >>> _const_to_rat("const_1_0Eneg_10")
    '(1/10000000000 : Rat)'
    """
    v = name[len("const_") :]
    neg = v.startswith("neg_")
    if neg:
        v = v[len("neg_") :]

    # Handle scientific notation (e.g. "1_0Eneg_10" = 1.0 * 10^-10)
    em = re.match(r"^(\d+)_(\d+)E(\w+)_(\d+)$", v)
    if em:
        int_part, frac_part, exp_sign_str, exp_str = em.groups()
        mantissa_digits = int_part + frac_part
        mantissa_len = len(int_part) + len(frac_part)
        exp_val = int(exp_str)
        if exp_sign_str == "neg":
            # mantissa * 10^-exp = mantissa_digits / 10^(exp + len(frac_part))
            denom = 10 ** (exp_val + len(frac_part))
            num = int(mantissa_digits)
        else:
            # mantissa * 10^exp = mantissa_digits * 10^(exp - len(frac_part)) / 1
            shift = exp_val - len(frac_part)
            if shift >= 0:
                num = int(mantissa_digits) * (10**shift)
                denom = 1
            else:
                num = int(mantissa_digits)
                denom = 10 ** (-shift)
        if denom == 1:
            expr = f"({num} : Rat)"
        else:
            expr = f"({num}/{denom} : Rat)"
        return f"(-{expr})" if neg else expr

    parts = v.split("_")
    if len(parts) != 2:
        expr = f"({v} : Rat)"
        return f"(-{expr})" if neg else expr
    int_part, frac_part = parts
    denom = 10 ** len(frac_part)
    num = int(int_part) * denom + int(frac_part) if int_part else int(frac_part)
    expr = f"({num}/{denom} : Rat)"
    return f"(-{expr})" if neg else expr


# ── definition removal ──────────────────────────────────────────────────────


def _remove_f64_helper_defs(content: str) -> str:
    """Remove ``def f64_helpers.*`` lines and the associated comment."""
    lines = content.split("\n")
    out: list[str] = []

    for line in lines:
        stripped = line.strip()

        # Remove the old autospec comment
        if stripped == "/- F64 replaced with Rat by autospec -/":
            out.append("/- F64 replaced with Rat; f64_helpers inlined by autospec -/")
            continue

        # Remove f64_helpers definition lines
        if stripped.startswith("def f64_helpers."):
            continue

        out.append(line)

    # Collapse 4+ consecutive blank lines → 3
    result = "\n".join(out)
    result = re.sub(r"\n{4,}", "\n\n\n", result)

    return result


# ── CLI ──────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import sys
    from pathlib import Path

    if len(sys.argv) < 2:
        print("Usage: python lean_inline.py <file.lean> [--in-place]", file=sys.stderr)
        sys.exit(1)

    path = Path(sys.argv[1])
    content = path.read_text()
    result = inline_f64_helpers(content)

    if "--in-place" in sys.argv:
        path.write_text(result)
        print(f"  Inlined f64_helpers in {path.name}")
    else:
        print(result)
