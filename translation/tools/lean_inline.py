#!/usr/bin/env python3
"""
Inline f64_helpers / i64_helpers calls in generated Lean code.

The charon→aeneas pipeline cannot handle f64 operations natively, so the
LLBC adapter translates them into f64_helpers.* function calls.  Similarly,
i64 operations are translated into i64_helpers.* calls so they become pure
Int arithmetic instead of monadic Result-returning operations.

f64 helpers are trivial wrappers:

    def f64_helpers.add (a b : Rat) : Result Rat := ok (a + b)   -- always ok

In a ``do`` block, ``←`` unwraps ``Result α`` → ``α``, and the body
simply returns ``ok (pure-expr)``.  So we can replace the monadic call
with a pure ``:=`` binding:

    let x ← f64_helpers.mul a b    →   let x := a * b
    let y ← f64_helpers.add x c    →   let y := x + c

i64_helpers follow the same pattern, mapping to pure Int operations:

    let x ← i64_helpers.add a b    →   let x := a + b
    let x ← i64_helpers.eq a b     →   let x := a = b
    let x ← i64_helpers.shl a n    →   let x := Int.shiftLeft a n.val

After inlining all call sites the ``def f64_helpers.*`` / ``def i64_helpers.*``
definitions are dead code and are removed.

Usage::

    from lean_inline import inline_f64_helpers, inline_i64_helpers
    clean = inline_i64_helpers(inline_f64_helpers(lean_source_text))
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

# ── i64 operator tables ──────────────────────────────────────────────────────

# i64 helper name → Lean infix operator
_I64_BINARY: dict[str, str] = {
    "add": "+",
    "sub": "-",
    "mul": "*",
    "div": "/",
    "rem": "%",
}

_I64_COMPARISON: dict[str, str] = {
    "eq": "=",
    "ne": "≠",
    "gt": ">",
    "lt": "<",
    "ge": "≥",
    "le": "≤",
}

# i64 helper name → (function, needs_val)
# If needs_val is True, the second argument has .val appended (for usize→Nat).
_I64_BITWISE: dict[str, tuple[str, bool]] = {
    "bitand": ("Int.land", False),
    "bitor": ("Int.lor", False),
    "bitxor": ("Int.xor", False),
    "shl": ("Int.shiftLeft", True),
    "shr": ("Int.shiftRight", True),
}

# i64 helper name → (prefix, suffix)
_I64_UNARY: dict[str, tuple[str, str]] = {
    "neg": ("-", ""),
    "not": ("(-(", " + 1))"),
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


def inline_i64_helpers(content: str) -> str:
    """Inline all i64_helpers.* calls and remove their definitions.

    Returns the transformed Lean source.
    """
    # Step 1: binary arithmetic (add / sub / mul / div / rem)
    for name, op in _I64_BINARY.items():
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+i64_helpers\.{name}\s+(\S+)\s+(\S+)",
            rf"\1:= \2 {op} \3",
            content,
        )

    # Step 2: comparisons (eq / ne / gt / lt / ge / le)
    for name, op in _I64_COMPARISON.items():
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+i64_helpers\.{name}\s+(\S+)\s+(\S+)",
            rf"\1:= \2 {op} \3",
            content,
        )

    # Step 3: bitwise (bitand / bitor / bitxor) — Int.land / Int.lor / Int.xor
    for name, (func, _) in _I64_BITWISE.items():
        if name in ("shl", "shr"):
            continue  # shifts handled separately
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+i64_helpers\.{name}\s+(\S+)\s+(\S+)",
            rf"\1:= {func} \2 \3",
            content,
        )

    # Step 4: shifts (shl / shr) — Int.shiftLeft / Int.shiftRight, usize→Nat
    for name, (func, _) in _I64_BITWISE.items():
        if name not in ("shl", "shr"):
            continue
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+i64_helpers\.{name}\s+(\S+)\s+(\S+)",
            rf"\1:= {func} \2 \3.val",
            content,
        )

    # Step 5: unary (neg / not)
    for name, (prefix, suffix) in _I64_UNARY.items():
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+i64_helpers\.{name}\s+(\S+)",
            rf"\1:= {prefix}\2{suffix}",
            content,
        )

    # Step 6: from_usize
    content = re.sub(
        r"(let\s+\w+\s+)←\s+i64_helpers\.from_usize\s+(\S+)",
        r"\1:= (\2.val : Int)",
        content,
    )

    # Step 7: integer constants (const_42, const_neg_5, …)
    content = re.sub(
        r"(let\s+\w+\s+)←\s+i64_helpers\.(const_\w+)",
        lambda m: f"{m.group(1)}:= {_i64_const_to_int(m.group(2))}",
        content,
    )

    # Step 8: tail-position calls (return value of do block)
    for name, op in _I64_BINARY.items():
        content = re.sub(
            rf"(^|\n)(\s+)i64_helpers\.{name}\s+(\S+)\s+(\S+)[ \t]*$",
            rf"\1\2ok (\3 {op} \4)",
            content,
            flags=re.MULTILINE,
        )
    for name, op in _I64_COMPARISON.items():
        content = re.sub(
            rf"(^|\n)(\s+)i64_helpers\.{name}\s+(\S+)\s+(\S+)[ \t]*$",
            rf"\1\2ok (\3 {op} \4)",
            content,
            flags=re.MULTILINE,
        )
    for name, (func, needs_val) in _I64_BITWISE.items():
        if needs_val:
            content = re.sub(
                rf"(^|\n)(\s+)i64_helpers\.{name}\s+(\S+)\s+(\S+)[ \t]*$",
                rf"\1\2ok ({func} \3 \4.val)",
                content,
                flags=re.MULTILINE,
            )
        else:
            content = re.sub(
                rf"(^|\n)(\s+)i64_helpers\.{name}\s+(\S+)\s+(\S+)[ \t]*$",
                rf"\1\2ok ({func} \3 \4)",
                content,
                flags=re.MULTILINE,
            )
    for name, (prefix, suffix) in _I64_UNARY.items():
        content = re.sub(
            rf"(^|\n)(\s+)i64_helpers\.{name}\s+(\S+)[ \t]*$",
            rf"\1\2ok ({prefix}\3{suffix})",
            content,
            flags=re.MULTILINE,
        )
    content = re.sub(
        r"(^|\n)(\s+)i64_helpers\.from_usize\s+(\S+)[ \t]*$",
        r"\1\2ok ((\3.val : Int))",
        content,
        flags=re.MULTILINE,
    )

    # Step 9: handle i64_helpers calls outside a let binding
    content = re.sub(r"i64_helpers\.neg\s+(\S+)", r"ok (-\1)", content)
    content = re.sub(r"i64_helpers\.not\s+(\S+)", r"ok (-(\1 + 1))", content)

    # Step 10: remove now-unused ``def i64_helpers.*`` definitions
    content = _remove_i64_helper_defs(content)

    return content


# ── i64 constant conversion ──────────────────────────────────────────────────


def _i64_const_to_int(name: str) -> str:
    """Convert a const_* name to an Int literal.

    >>> _i64_const_to_int("const_42")
    '(42 : Int)'
    >>> _i64_const_to_int("const_neg_5")
    '(-5 : Int)'
    """
    v = name[len("const_"):]
    neg = v.startswith("neg_")
    if neg:
        v = v[len("neg_"):]
    if neg:
        return f"(-{v} : Int)"
    return f"({v} : Int)"


# ── i64 definition removal ───────────────────────────────────────────────────


def _remove_i64_helper_defs(content: str) -> str:
    """Remove i64_helpers definitions."""
    return _remove_helper_defs(content, "i64_helpers")


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


def _remove_helper_defs(content: str, prefix: str) -> str:
    """Remove ``def <prefix>.*`` / ``axiom <prefix>.*`` and their
    associated doc-comment + attribute blocks."""
    escaped = re.escape(prefix)
    # Remove the entire block: doc comment → attribute → axiom/def
    content = re.sub(
        rf"\n*/--\s+\[{escaped}::\S+\]:.*?\n"
        r"(?:.*?\n)*?"
        rf"@\[rust_fun\s+\"{escaped}::\S+\"\][ \t]*(?:\n[ \t]*)?"
        rf"(?:noncomputable\s+)?(?:axiom|def)\s+{escaped}\.\S+[ \t]*[^\n]*\n?",
        "",
        content,
        flags=re.DOTALL,
    )

    # Fallback: remove any remaining bare helper lines
    lines = content.split("\n")
    out: list[str] = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith(f"def {prefix}."):
            continue
        if stripped.startswith(f"axiom {prefix}."):
            continue
        out.append(line)

    # Collapse 4+ consecutive blank lines → 3
    result = "\n".join(out)
    result = re.sub(r"\n{4,}", "\n\n\n", result)
    return result


def _remove_f64_helper_defs(content: str) -> str:
    """Remove f64_helpers definitions, plus the autospec F64 comment."""
    content = content.replace(
        "/- F64 replaced with Rat by autospec -/",
        "/- F64 replaced with Rat; f64_helpers inlined by autospec -/",
    )
    return _remove_helper_defs(content, "f64_helpers")


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
