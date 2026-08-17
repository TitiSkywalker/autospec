"""Post-process generated Lean files: F64→Rat/ℝ, inline helpers, write output."""

import os
import re
from pathlib import Path

# Import from sibling tools package
import sys

_tools_dir = str(Path(__file__).resolve().parent.parent / "tools")
if _tools_dir not in sys.path:
    sys.path.insert(0, _tools_dir)
from lean_inline import inline_f64_helpers, inline_i64_helpers  # noqa: E402


class NonComputableError(Exception):
    """Rat float-model cannot handle non-computable operations like sqrt."""


def lean_f64_to_rat(content: str) -> str:
    """Replace F64 opaque type with Rat (computable rational arithmetic).

    Raises NonComputableError if the code uses sqrt, which is not
    computable for Rat — the caller should retry with ``--float-model Real``.
    """
    # Clean up any stray noncomputable section marker (aeneas adds this
    # when there are external definitions).  Must happen before the early
    # return so that projects without f64 also get the cleanup.
    content = content.replace("noncomputable section\n", "")

    if "F64" not in content:
        return content

    content = content.replace("F64.Insts.CoreCloneClone.clone", "RatClone.clone")
    content = content.replace("F64.Insts.CoreCloneClone", "RatClone")

    lines = content.split("\n")
    skip: set[int] = set()
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("/-- [") or (
            line.startswith("/-- Trait") and "Clone for f64" in line
        ):
            if any(
                [
                    "f64_helpers::" in line,
                    "core::clone::impls" in line
                    and "core::clone::Clone for f64" in line,
                    "Trait implementation:" in line
                    and "core::clone::Clone for f64" in line,
                    "core::f64::" in line,
                ]
            ):
                start = i
                while i < len(lines) and not lines[i].rstrip().endswith("-/"):
                    i += 1
                i += 1
                while i < len(lines) and lines[i].startswith("@["):
                    i += 1
                while i < len(lines) and lines[i].strip() != "":
                    i += 1
                if i < len(lines) and lines[i].strip() == "":
                    i += 1
                skip.update(range(start, i))
                continue
        i += 1

    out = []
    for i, line in enumerate(lines):
        if i in skip:
            continue
        out.append(line.replace("F64", "Rat"))
    result = "\n".join(out)

    result = re.sub(r"core\.f64\.\S+\.abs\b", "f64_helpers.abs", result)
    # ``is_nan`` / ``is_infinite`` are always false for Rat — replace with ``ok false``.
    result = re.sub(
        r"\blet\s+(\w+)\s+←\s+core\.f64\.\S+\.is_nan\s+(\S+)",
        r"let \1 := false",
        result,
    )
    result = re.sub(
        r"\blet\s+(\w+)\s+←\s+core\.f64\.\S+\.is_infinite\s+(\S+)",
        r"let \1 := false",
        result,
    )
    result = re.sub(r"\baxiom \b(?!.*noncomputable)", "noncomputable axiom ", result)
    # Detect non-computable external functions (sqrt, exp, sin, etc.).
    # Computable f64 ops were handled by the adapter → f64_helpers and
    # will be inlined later.  Anything still referencing std.f64.Rat.*
    # is a non-computable axiom from aeneas.
    non_comp = set(re.findall(r"std\.f64\.Rat\.(\w+)", result))
    if non_comp:
        funcs = ", ".join(sorted(non_comp))
        raise NonComputableError(
            f"{funcs} not computable in Rat mode"
        )

    if "RatClone" in result and "def RatClone.clone" not in result:
        rat_clone = (
            "/- F64 replaced with Rat by autospec -/\n"
            "def RatClone.clone (x : Rat) : Result Rat := ok x\n"
            "\n"
            "@[reducible]\n"
            "def RatClone : core.clone.Clone Rat := {\n"
            "  clone := RatClone.clone\n"
            "}\n"
        )
        result = re.sub(r"(namespace \w+\n)", r"\1\n" + rat_clone, result, count=1)

    return result


def lean_f64_to_real(content: str) -> str:
    """Replace F64 opaque type with ℝ (Real, computable including sqrt).

    Unlike Rat, ``Real.sqrt`` is computable (via Cauchy sequences in
    mathlib), so modules using ``sqrt`` do NOT need ``noncomputable``.
    ``sqrt`` and ``exp`` are both rewritten to the concrete Mathlib
    functions (``Real.sqrt`` / ``Real.exp``) so impl and spec share the
    same definition; other transcendentals stay as opaque axioms.
    """
    if "F64" not in content:
        return content

    content = content.replace("F64.Insts.CoreCloneClone.clone", "RealClone.clone")
    content = content.replace("F64.Insts.CoreCloneClone", "RealClone")

    lines = content.split("\n")
    skip: set[int] = set()
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("/-- [") or (
            line.startswith("/-- Trait") and "Clone for f64" in line
        ):
            if any(
                [
                    "f64_helpers::" in line,
                    "core::clone::impls" in line
                    and "core::clone::Clone for f64" in line,
                    "Trait implementation:" in line
                    and "core::clone::Clone for f64" in line,
                    "core::f64::" in line,
                ]
            ):
                start = i
                while i < len(lines) and not lines[i].rstrip().endswith("-/"):
                    i += 1
                i += 1
                while i < len(lines) and lines[i].startswith("@["):
                    i += 1
                while i < len(lines) and lines[i].strip() != "":
                    i += 1
                if i < len(lines) and lines[i].strip() == "":
                    i += 1
                skip.update(range(start, i))
                continue
        i += 1

    out = []
    for i, line in enumerate(lines):
        if i in skip:
            continue
        out.append(line.replace("F64", "ℝ"))
    result = "\n".join(out)

    result = re.sub(r"core\.f64\.\S+\.abs\b", "f64_helpers.abs", result)
    # Real.sqrt requires Mathlib — add import after Aeneas
    result = re.sub(
        r"import Aeneas\n",
        "import Aeneas\nimport Mathlib\n",
        result,
        count=1,
    )
    # Real.sqrt is noncomputable for C codegen (Cauchy sequences), so the
    # code generator must skip definitions that transitively call it.
    # Insert noncomputable section.
    result = result.replace("noncomputable section\n", "")
    result = result.replace(
        "set_option linter.style.nameCheck false\n",
        "set_option linter.style.nameCheck false\n"
        "noncomputable section\n",
        count=1,
    )

    # sqrt: replace axiom + call sites with Real.sqrt (computable for #eval)
    if "std.f64.ℝ.sqrt" in result:
        # Remove the axiom definition block (doc comment + attribute + axiom)
        # and the preceding blank lines so no gap is left behind.
        result = re.sub(
            r"\n*/-- \[std::f64::\{f64\}::sqrt\]:.*?\n"
            r"(?:.*?\n)*?"
            r"@\[rust_fun \"std::f64::\{f64\}::sqrt\"\] "
            r"(?:noncomputable )?axiom std\.f64\.ℝ\.sqrt : ℝ → Result ℝ\n",
            "\n",
            result,
            flags=re.DOTALL,
        )
        # Replace call sites (let-binding)
        result = re.sub(
            r"\blet\s+(\w+)\s+←\s+std\.f64\.ℝ\.sqrt\s+(\S+)",
            r"let \1 := Real.sqrt \2",
            result,
        )
        # Replace tail-position calls  (use [ \t] not \s — \s eats \n)
        result = re.sub(
            r"(^|\n)([ \t]+)std\.f64\.ℝ\.sqrt[ \t]+(\S+)[ \t]*$",
            r"\1\2ok (Real.sqrt \3)",
            result,
            flags=re.MULTILINE,
        )

    # exp: replace axiom + call sites with Real.exp (same treatment as sqrt)
    if "std.f64.ℝ.exp" in result:
        # Remove the axiom definition block (doc comment + attribute + axiom)
        # and the preceding blank lines so no gap is left behind.
        result = re.sub(
            r"\n*/-- \[std::f64::\{f64\}::exp\]:.*?\n"
            r"(?:.*?\n)*?"
            r"@\[rust_fun \"std::f64::\{f64\}::exp\"\] "
            r"(?:noncomputable )?axiom std\.f64\.ℝ\.exp : ℝ → Result ℝ\n",
            "\n",
            result,
            flags=re.DOTALL,
        )
        # Replace call sites (let-binding)
        result = re.sub(
            r"\blet\s+(\w+)\s+←\s+std\.f64\.ℝ\.exp\s+(\S+)",
            r"let \1 := Real.exp \2",
            result,
        )
        # Replace tail-position calls  (use [ \t] not \s — \s eats \n)
        result = re.sub(
            r"(^|\n)([ \t]+)std\.f64\.ℝ\.exp[ \t]+(\S+)[ \t]*$",
            r"\1\2ok (Real.exp \3)",
            result,
            flags=re.MULTILINE,
        )

    # ``is_nan`` / ``is_infinite`` are always false for ℝ.
    result = re.sub(
        r"\blet\s+(\w+)\s+←\s+core\.f64\.\S+\.is_nan\s+(\S+)",
        r"let \1 := false",
        result,
    )
    result = re.sub(
        r"\blet\s+(\w+)\s+←\s+core\.f64\.\S+\.is_infinite\s+(\S+)",
        r"let \1 := false",
        result,
    )

    if "RealClone" in result and "def RealClone.clone" not in result:
        real_clone = (
            "/- F64 replaced with ℝ by autospec -/\n"
            "def RealClone.clone (x : ℝ) : Result ℝ := ok x\n"
            "\n"
            "@[reducible]\n"
            "def RealClone : core.clone.Clone ℝ := {\n"
            "  clone := RealClone.clone\n"
            "}\n"
        )
        result = re.sub(r"(namespace \w+\n)", r"\1\n" + real_clone, result, count=1)

    return result


# Integer types that Aeneas translates, with literal suffix, bit width, signedness.
_INT_TYPES: list[tuple[str, str, int | None, bool]] = [
    ("Std.I8", "#i8", 8, True),
    ("Std.I16", "#i16", 16, True),
    ("Std.I32", "#i32", 32, True),
    ("Std.I64", "#i64", 64, True),
    ("Std.I128", "#i128", 128, True),
    ("Std.Isize", "#isize", None, True),
    ("Std.U8", "#u8", 8, False),
    ("Std.U16", "#u16", 16, False),
    ("Std.U32", "#u32", 32, False),
    ("Std.U64", "#u64", 64, False),
    ("Std.U128", "#u128", 128, False),
    # Std.Usize is intentionally excluded — used for array indexing.
]

_INT_MODELS = ("int", "nat", "bitvec")


def _int_target_type(is_signed: bool, width: int | None, model: str) -> str:
    """Lean type a Rust integer maps to under *model*."""
    if model == "int":
        return "Int"
    if model == "nat":
        return "Int" if is_signed else "Nat"
    if model == "bitvec":
        return "Int" if is_signed else f"BitVec {width}"
    raise ValueError(f"unknown int model: {model}")


def _int_helper_stem(target_type: str) -> str:
    """Identifier-safe stem for a target type (``Int``→``Int``, ``BitVec 64``→``BitVec64``)."""
    return target_type.replace(" ", "")


def _type_arg(target_type: str) -> str:
    """Parenthesize a type used as an argument (``BitVec 64`` → ``(BitVec 64)``)."""
    return f"({target_type})" if " " in target_type else target_type


def _is_int_literal(tok: str) -> bool:
    """Whether *tok* is an integer literal after type replacement.

    Accepts ``6`` (Int/Nat) and ``(1 : BitVec 64)`` (bitvec).
    """
    t = tok.strip()
    if t.isdigit():
        return True
    return bool(re.fullmatch(r"\(\d+\s*:\s*BitVec\s+\d+\)", t))


def lean_integers_to_int(content: str) -> str:
    """Backward-compatible wrapper: every integer → Int."""
    return lean_integers_to_model(content, "int")


def lean_integers_to_model(content: str, model: str = "int") -> str:
    """Replace Aeneas integer types with ``Int`` / ``Nat`` / ``BitVec``.

    *model* selects the integer semantics:

    * ``"int"`` — every integer → ``Int`` (unbounded signed).
    * ``"nat"`` — unsigned → ``Nat``, signed → ``Int``.
    * ``"bitvec"`` — unsigned → ``BitVec w`` (fixed-width wrapping),
      signed → ``Int``.

    ``Std.Usize`` is **not** replaced — it is used for array indexing.
    """
    if model not in _INT_MODELS:
        raise ValueError(f"unknown int model: {model}")

    _INT_MARKERS = (
        [t for t, _, _, _ in _INT_TYPES]
        + [s for _, s, _, _ in _INT_TYPES]
        + [f"PartialEq{t.replace('Std.', '')}" for t, _, _, _ in _INT_TYPES]
    )
    if not any(m in content for m in _INT_MARKERS):
        return content

    # Set of pure integer target types for this model (used for type tracking
    # in the monadic→pure conversion).
    produced: set[str] = {
        _int_target_type(is_signed, width, model)
        for _, _, width, is_signed in _INT_TYPES
    }

    for ty, suffix, width, is_signed in _INT_TYPES:
        tgt = _int_target_type(is_signed, width, model)
        clone_helper = f"{_int_helper_stem(tgt)}Clone"
        peq_helper = f"PartialEq{_int_helper_stem(tgt)}"
        clone_name = ty.replace("Std.", "")
        content = content.replace(ty, _type_arg(tgt))
        content = content.replace(f"core.clone.Clone{clone_name}", clone_helper)
        content = content.replace(f"core.cmp.PartialEq{clone_name}", peq_helper)
        # Literal handling: bitvec keeps the width for unsigned literals
        # (5#u64 → (5 : BitVec 64)); other models just strip the suffix.
        if model == "bitvec" and not is_signed:
            content = re.sub(
                rf"(\d+){re.escape(suffix)}\b",
                rf"(\1 : BitVec {width})",
                content,
            )
        else:
            content = re.sub(rf"{re.escape(suffix)}\b", "", content)

    # Convert monadic `let x ← <int-expr>` → `let x := <int-expr>` for ops that
    # became pure after replacement.  usize arithmetic stays monadic
    # (Std.Usize is untouched), so track integer- vs usize-typed identifiers.
    content = _monadic_ops_to_pure(content, sorted(produced))

    # Helper definitions for distinct target types actually referenced.
    # Clone and PartialEq are defined *together* for a type: the impl file is
    # imported by the tests, which may need the PartialEq helper even when the
    # impl itself only uses Clone.
    extras: list[str] = []
    for tgt in sorted(produced):
        stem = _int_helper_stem(tgt)
        clone_helper = f"{stem}Clone"
        peq_helper = f"PartialEq{stem}"
        need_clone = clone_helper in content and f"def {clone_helper}.clone" not in content
        need_peq = peq_helper in content and f"def {peq_helper}.eq" not in content
        if need_clone or need_peq:
            if f"def {clone_helper}.clone" not in content:
                extras.append(
                    f"/- Integer types replaced with {tgt} by autospec -/\n"
                    f"def {clone_helper}.clone (x : {_type_arg(tgt)}) : "
                    f"Result {_type_arg(tgt)} := ok x\n"
                    "\n"
                    f"@[reducible]\n"
                    f"def {clone_helper} : core.clone.Clone {_type_arg(tgt)} := {{\n"
                    f"  clone := {clone_helper}.clone\n"
                    "}\n"
                )
            if f"def {peq_helper}.eq" not in content:
                extras.append(
                    f"def {peq_helper}.eq (x y : {_type_arg(tgt)}) : "
                    "Result Bool := ok (x = y)\n"
                    "\n"
                    f"@[reducible]\n"
                    f"def {peq_helper} : core.cmp.PartialEq {_type_arg(tgt)} {_type_arg(tgt)} := {{\n"
                    f"  eq := {peq_helper}.eq\n"
                    "}\n"
                )

    if extras:
        block = "\n".join(extras)
        content = re.sub(r"(namespace \w+\n)", r"\1\n" + block, content, count=1)

    return content


def _monadic_ops_to_pure(content: str, ctx_types: list[str]) -> str:
    """Rewrite ``let x ← <int-expr>`` → ``let x := <int-expr>`` for pure-int ops.

    Tracks integer- vs usize-typed identifiers so that usize arithmetic (still
    monadic on ``Std.Usize``) is left untouched.
    """
    int_vars: set[str] = set()
    usize_vars: set[str] = set()

    # Seed from signature parameters: (a : Int), (lo : Std.Usize), (a : (BitVec 64)).
    for m in re.finditer(r"\((\w+)\s*:\s*([^)]*)\)", content):
        name, ty = m.group(1), m.group(2).strip().lstrip("(")
        if ty == "Std.Usize":
            usize_vars.add(name)
        elif ty in ctx_types:
            int_vars.add(name)

    # Seed from `let X ← if … then ok LIT else ok LIT` (integer literals) —
    # X is integer-typed even though the `if` itself stays monadic.
    for m in re.finditer(
        r"let\s+(\w+)\s+←\s+if\b.*?\bthen\s+ok\s+([^\n]+)\n\s*else\s+ok\s+([^\n]+)",
        content,
        re.DOTALL,
    ):
        name, lit1, lit2 = m.group(1), m.group(2), m.group(3)
        if _is_int_literal(lit1) and _is_int_literal(lit2):
            int_vars.add(name)

    out: list[str] = []
    for line in content.split("\n"):
        out.append(_convert_let_line(line, int_vars, usize_vars))
    return "\n".join(out)


def _convert_let_line(line: str, int_vars: set[str], usize_vars: set[str]) -> str:
    """Convert one ``let x ← A op B`` line to ``:=`` if A, B are integer-typed.

    Handles ``let`` at line start or after ``then``/``else`` (aeneas indents
    ``then let x ← …`` on its own line).
    """

    def is_int(tok: str) -> bool:
        return tok in int_vars or tok.isdigit()

    def is_usize(tok: str) -> bool:
        return tok in usize_vars

    def track(x: str, a: str, b: str) -> None:
        """Record ``x``'s type based on its operands (for later lines)."""
        if is_usize(a) or is_usize(b):
            usize_vars.add(x)
        elif is_int(a) and is_int(b):
            int_vars.add(x)

    def apply(prefix: str, x: str, a: str, op: str, b: str) -> str:
        if is_int(a) and is_int(b) and not is_usize(a) and not is_usize(b):
            int_vars.add(x)
            return f"{prefix}:= {a} {op} {b}"
        if is_usize(a) or is_usize(b):
            usize_vars.add(x)
        return line

    _prefix = r"^(\s*(?:then\s+|else\s+)?let\s+(\w+)\s+)←\s+"

    # arithmetic: let x ← A op B
    m = re.match(_prefix + r"(\w+|\d+)\s*([+\-*/%])\s*(\w+|\d+)\s*$", line)
    if m:
        return apply(*m.groups())

    # bitwise: let x ← lift (A &&& B)
    m = re.match(
        _prefix + r"lift\s*\(\s*(\w+)\s*(&&&|\|\|\||\^\^\^)\s*(\w+)\s*\)\s*$", line
    )
    if m:
        return apply(*m.groups())

    # shift: let x ← A <<< B
    m = re.match(_prefix + r"(\w+)\s*(<<<|>>>)\s*(\w+|\d+)\s*$", line)
    if m:
        return apply(*m.groups())

    # Pure let-binding: let x := A op B — already pure, just record x's type
    # so that subsequent ``let y ← … x …`` / ``then x op …`` can be converted.
    m = re.match(
        r"^\s*(?:then\s+|else\s+)?let\s+(\w+)\s+:=\s+(\w+|\d+)\s*([+\-*/%])\s*(\w+|\d+)\s*$",
        line,
    )
    if m:
        x, a, _op, b = m.groups()
        track(x, a, b)
        return line

    # Tail-position pure integer expression (no `let`): wrap in `ok (...)`.
    def tail_ok(a: str, op: str, b: str) -> str | None:
        if is_int(a) and is_int(b) and not is_usize(a) and not is_usize(b):
            return f"ok ({a} {op} {b})"
        return None

    # arithmetic tail: A op B (optionally after then/else)
    m = re.match(r"^(\s*)((?:then\s+|else\s+)?)(\w+|\d+)\s*([+\-*/%])\s*(\w+|\d+)\s*$", line)
    if m:
        r = tail_ok(m.group(3), m.group(4), m.group(5))
        return m.group(1) + m.group(2) + r if r else line

    # bitwise tail: lift (A &&& B) (optionally after then/else)
    m = re.match(r"^(\s*)((?:then\s+|else\s+)?)lift\s*\(\s*(\w+)\s*(&&&|\|\|\||\^\^\^)\s*(\w+)\s*\)\s*$", line)
    if m:
        r = tail_ok(m.group(3), m.group(4), m.group(5))
        return m.group(1) + m.group(2) + r if r else line

    # shift tail: A <<< B (optionally after then/else)
    m = re.match(r"^(\s*)((?:then\s+|else\s+)?)(\w+)\s*(<<<|>>>)\s*(\w+|\d+)\s*$", line)
    if m:
        r = tail_ok(m.group(3), m.group(4), m.group(5))
        return m.group(1) + m.group(2) + r if r else line

    return line


def _mark_noncomputable_defs(text: str) -> str:
    """Mark defs that depend on axioms (directly or transitively)
    as noncomputable.

    Without this, Lean's code generator rejects definitions whose
    evaluation chain reaches an axiom.
    """
    # Collect names declared as axiom (may already have noncomputable prefix)
    axiom_names: set[str] = set()
    for m in re.finditer(
        r"^\s*(?:noncomputable\s+)?axiom\s+([\w.]+)", text, re.MULTILINE
    ):
        axiom_names.add(m.group(1))
    if not axiom_names:
        return text

    # Find every top-level def/theorem and capture its body (up to the next
    # top-level keyword at indent 0).
    decls: list[tuple[str, str, int, int]] = []  # (name, body, start, end)
    pattern = re.compile(
        r"^(?:@\[[^\]]*\][ \t]*\n\s*)?"
        r"(?:(noncomputable[ \t]+)?(def|theorem)[ \t]+(\w+))",
        re.MULTILINE,
    )
    for m in pattern.finditer(text):
        name = m.group(3)
        is_noncomp = m.group(1) is not None
        body_start = m.end()
        # Search for the next top-level declaration or namespace closing
        next_decl = re.search(
            r"^\s*(?:end\b|namespace\b|(?:noncomputable\s+)?(?:def|axiom|theorem)\s+)",
            text[body_start:],
            re.MULTILINE,
        )
        body_end = body_start + next_decl.start() if next_decl else len(text)
        body = text[body_start:body_end]
        decls.append((name, body, m.start(), m.end()))
        if is_noncomp:
            axiom_names.add(name)

    # Fixed-point propagation: def A calls def B which calls axiom …
    changed = True
    while changed:
        changed = False
        for name, body, _, _ in decls:
            if name in axiom_names:
                continue
            # Check whether the body references any name already marked
            if any(
                re.search(rf"\b{re.escape(n)}\b", body) for n in axiom_names
            ):
                axiom_names.add(name)
                changed = True

    # Rewrite: insert noncomputable before affected defs (not axioms, which
    # are handled separately).
    if not axiom_names:
        return text

    # Collect names that need noncomputable (exclude original axioms)
    noncomp_names = sorted(
        set(n for n in axiom_names if n not in set(
            re.findall(r"^\s*(?:noncomputable\s+)?axiom\s+([\w.]+)", text, re.MULTILINE)
        )),
        key=len,
        reverse=True,  # longest first to avoid partial matches
    )
    if not noncomp_names:
        return text

    # Insert noncomputable before def lines that aren't already marked.
    # We match def <name> at start-of-line.
    for name in noncomp_names:
        text = re.sub(
            rf"^(@\[[^\]]*\]\s*\n\s*)?(def\s+{re.escape(name)}\b)",
            rf"\1noncomputable \2",
            text,
            flags=re.MULTILINE,
        )

    return text


def _fix_partial_mutual(text: str) -> str:
    """Replace ``partial_fixpoint`` with ``partial`` for tail-recursive
    functions inside ``mutual`` blocks, so the Lean 4.31 monotonicity
    checker accepts them.  Also strips the ``@[rust_loop*]`` and
    ``@[reducible]`` attributes that are incompatible with ``partial``.
    """
    # _quick_sort_loop  (rust_loop)
    text = re.sub(
        r"@\[rust_loop\]\n(def _quick_sort_loop\b.*?)partial_fixpoint",
        r"partial \1",
        text,
        flags=re.DOTALL,
    )
    # _quick_sort_loop.body  (rust_loop_body)
    text = re.sub(
        r"@\[rust_loop_body\]\n(def _quick_sort_loop\.body\b.*?)partial_fixpoint",
        r"partial \1",
        text,
        flags=re.DOTALL,
    )
    # _quick_sort  (reducible)
    text = re.sub(
        r"@\[reducible\]\n(def _quick_sort\b.*?)partial_fixpoint",
        r"partial \1",
        text,
        flags=re.DOTALL,
    )
    return text


def _fix_vec_is_empty(text: str) -> str:
    """Replace ``alloc.vec.Vec.is_empty`` (not in aeneas runtime) with a
    length check."""
    if "alloc.vec.Vec.is_empty" not in text:
        return text
    text = re.sub(
        r"let\s+(\w+)\s+←\s+alloc\.vec\.Vec\.is_empty\s+Global\s+(\w+)",
        r"let \1 := (alloc.vec.Vec.deref \2).length = 0",
        text,
    )
    # Replace tail-position calls
    text = re.sub(
        r"alloc\.vec\.Vec\.is_empty\s+Global\s+(\w+)",
        r"ok ((alloc.vec.Vec.deref \1).length = 0)",
        text,
    )
    return text


def write_lean_output(
    workdir: Path,
    output_path: Path,
    *,
    label: str = "",
    float_model: str = "Rat",
    int_model: str = "int",
) -> tuple[int, list[str]]:
    """Post-process and copy Lean files from *workdir* to *output_path*.

    Parameters
    ----------
    float_model:
        ``"Rat"`` (default) — replace f64 with Rat; sqrt is noncomputable.
        ``"Real"`` — replace f64 with ℝ; sqrt is computable via Real.sqrt.
    int_model:
        ``"int"`` (default) — integers → Int.  ``"nat"`` — unsigned → Nat.
        ``"bitvec"`` — unsigned → BitVec w (fixed-width wrapping).
    """
    lean_files = sorted(f for f in os.listdir(str(workdir)) if f.endswith(".lean"))
    def_count = 0
    written: list[str] = []

    _use_real = float_model == "Real"

    for f in lean_files:
        text = (workdir / f).read_text()
        text = text.replace(
            "set_option linter.unusedVariables false",
            "set_option linter.unusedVariables false\n"
            "set_option linter.style.nameCheck false",
        )
        if _use_real:
            text = lean_f64_to_real(text)
        else:
            try:
                text = lean_f64_to_rat(text)
            except NonComputableError:
                raise
        text = lean_integers_to_model(text, int_model)
        text = inline_f64_helpers(text)
        text = inline_i64_helpers(text)
        if _use_real:
            # f64_helpers constants and helpers use ``Rat`` — fix type
            text = re.sub(r"\bRat\b", "ℝ", text)
            text = re.sub(r"\bRatClone\b", "RealClone", text)
        text = _fix_partial_mutual(text)
        text = _fix_vec_is_empty(text)
        text = _mark_noncomputable_defs(text)
        (output_path / f).write_text(text.rstrip("\n") + "\n")
        written.append(f)
        def_count += len(
            re.findall(
                r"^\s*(?:noncomputable\s+)?(?:def|theorem|axiom)\s+\S+",
                text,
                re.MULTILINE,
            )
        )

    return def_count, written
