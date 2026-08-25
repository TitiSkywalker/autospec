"""Post-process generated Lean files: F64→Rat/ℝ, inline helpers, write output."""

import os
import re
import sys
from pathlib import Path

# Import from sibling tools package
_tools_dir = str(Path(__file__).resolve().parent.parent / "tools")
if _tools_dir not in sys.path:
    sys.path.insert(0, _tools_dir)
from lean_inline import inline_f64_helpers, inline_i64_helpers  # noqa: E402


# ── exceptions ─────────────────────────────────────────────────────────────────

class NonComputableError(Exception):
    """Rat float-model cannot handle non-computable operations like sqrt."""


# ── integer types table ────────────────────────────────────────────────────────

# (Aeneas type, literal suffix, bit width, signed)
_INT_TYPES: list[tuple[str, str, int | None, bool]] = [
    ("Std.I8",   "#i8",   8,   True),
    ("Std.I16",  "#i16",  16,  True),
    ("Std.I32",  "#i32",  32,  True),
    ("Std.I64",  "#i64",  64,  True),
    ("Std.I128", "#i128", 128, True),
    ("Std.Isize","#isize",None,True),
    ("Std.U8",   "#u8",   8,   False),
    ("Std.U16",  "#u16",  16,  False),
    ("Std.U32",  "#u32",  32,  False),
    ("Std.U64",  "#u64",  64,  False),
    ("Std.U128", "#u128", 128, False),
    # Std.Usize intentionally excluded — used for array indexing.
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
    """Identifier-safe stem (``Int``→``Int``, ``BitVec 64``→``BitVec64``)."""
    return target_type.replace(" ", "")


def _type_arg(target_type: str) -> str:
    """Parenthesize a type used as an argument."""
    return f"({target_type})" if " " in target_type else target_type


def _is_int_literal(tok: str) -> bool:
    """Whether *tok* is an integer literal after type replacement.

    Accepts ``6`` (Int/Nat) and ``(1 : BitVec 64)`` (bitvec).
    """
    t = tok.strip()
    if t.isdigit():
        return True
    return bool(re.fullmatch(r"\(\d+\s*:\s*BitVec\s+\d+\)", t))


# ── f64 → Rat / ℝ ──────────────────────────────────────────────────────────────

# Patterns that identify blocks to skip (f64_helpers / core::f64 / Clone impls).
_F64_SKIP_PATTERNS = [
    "f64_helpers::",
    "core::f64::",
    "core::clone::impls",
    "Trait implementation:",
]


def _skip_f64_blocks(lines: list[str]) -> set[int]:
    """Return indices of lines inside f64_helpers / core::f64 / Clone blocks."""
    skip: set[int] = set()
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("/-- [") or (
            line.startswith("/-- Trait") and "Clone for f64" in line
        ):
            if any(p in line for p in _F64_SKIP_PATTERNS):
                start = i
                # Skip doc comment lines
                while i < len(lines) and not lines[i].rstrip().endswith("-/"):
                    i += 1
                i += 1
                # Skip attributes
                while i < len(lines) and lines[i].startswith("@["):
                    i += 1
                # Skip the definition body
                while i < len(lines) and lines[i].strip() != "":
                    i += 1
                if i < len(lines) and lines[i].strip() == "":
                    i += 1
                skip.update(range(start, i))
                continue
        i += 1
    return skip


def _replace_f64_base(
    content: str,
    target: str,
    clone_name: str,
) -> str:
    """Common F64→target replacement shared by Rat and Real models."""
    if "F64" not in content:
        return content

    content = content.replace("F64.Insts.CoreCloneClone.clone", f"{clone_name}.clone")
    content = content.replace("F64.Insts.CoreCloneClone", clone_name)

    lines = content.split("\n")
    skip = _skip_f64_blocks(lines)

    out = []
    for j, line in enumerate(lines):
        if j in skip:
            continue
        out.append(line.replace("F64", target))
    result = "\n".join(out)

    # Replace abs call sites
    result = re.sub(r"core\.f64\.\S+\.abs\b", "f64_helpers.abs", result)

    # is_nan / is_infinite → always false
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

    # Add Clone helper if referenced but not defined
    if clone_name in result and f"def {clone_name}.clone" not in result:
        type_arg = f"({target})" if " " in target else target
        clone_def = (
            f"/- F64 replaced with {target} by autospec -/\n"
            f"def {clone_name}.clone (x : {type_arg}) : Result {type_arg} := ok x\n"
            "\n"
            f"@[reducible]\n"
            f"def {clone_name} : core.clone.Clone {type_arg} := {{\n"
            f"  clone := {clone_name}.clone\n"
            "}\n"
        )
        result = re.sub(r"(namespace \w+\n)", r"\1\n" + clone_def, result, count=1)

    return result


def lean_f64_to_rat(content: str) -> str:
    """Replace F64 opaque type with Rat (computable rational arithmetic).

    Raises NonComputableError if the code uses sqrt, which is not
    computable for Rat — the caller should retry with ``--float-model Real``.
    """
    content = content.replace("noncomputable section\n", "")
    result = _replace_f64_base(content, "Rat", "RatClone")

    # Mark remaining axioms as noncomputable
    result = re.sub(r"\baxiom \b(?!.*noncomputable)", "noncomputable axiom ", result)

    # Check for non-computable functions (sqrt, exp, sin, etc.)
    non_comp = set(re.findall(r"std\.f64\.Rat\.(\w+)", result))
    if non_comp:
        funcs = ", ".join(sorted(non_comp))
        raise NonComputableError(f"{funcs} not computable in Rat mode")

    return result


def _replace_f64_const(result: str, const_name: str, mathlib_name: str) -> str:
    """Replace an f64 const axiom with a Mathlib constant definition.

    ``core.f64.consts.PI`` → ``noncomputable def ... := ok Real.pi``
    """
    if const_name not in result:
        return result

    escaped = re.escape(const_name)

    # Replace let-binding call sites:  let x ← core.f64.consts.PI
    result = re.sub(
        rf"\blet\s+(\w+)\s+←\s+{escaped}\b",
        rf"let \1 := {mathlib_name}",
        result,
    )

    # Replace tail-position call sites
    result = re.sub(
        rf"(^|\n)([ \t]+){escaped}\s*$",
        rf"\1\2{mathlib_name}",
        result,
        flags=re.MULTILINE,
    )

    # Add a definition (the axiom is already stripped by _skip_f64_blocks)
    result = re.sub(
        r"(namespace \w+\n)",
        rf"\1\n/- {const_name} replaced with {mathlib_name} by autospec -/\n"
        rf"noncomputable def {const_name} : ℝ := {mathlib_name}\n",
        result,
        count=1,
    )

    return result


def _replace_transcendental(
    result: str, func_name: str, mathlib_name: str
) -> str:
    """Remove an f64 axiom and replace its call sites with a Mathlib function."""
    marker = f"std.f64.ℝ.{func_name}"
    if marker not in result:
        return result

    # Remove the axiom definition block
    result = re.sub(
        rf"\n*/-- \[std::f64::{{f64}}::{func_name}\]:.*?\n"
        r"(?:.*?\n)*?"
        rf"@\[rust_fun \"std::f64::{{f64}}::{func_name}\"\] "
        rf"(?:noncomputable )?axiom {re.escape(marker)} : ℝ → Result ℝ\n",
        "\n",
        result,
        flags=re.DOTALL,
    )

    # Replace let-binding call sites
    result = re.sub(
        rf"\blet\s+(\w+)\s+←\s+{re.escape(marker)}\s+(\S+)",
        rf"let \1 := {mathlib_name} \2",
        result,
    )

    # Replace tail-position call sites
    result = re.sub(
        rf"(^|\n)([ \t]+){re.escape(marker)}[ \t]+(\S+)[ \t]*$",
        rf"\1\2ok ({mathlib_name} \3)",
        result,
        flags=re.MULTILINE,
    )

    return result


def lean_f64_to_real(content: str) -> str:
    """Replace F64 opaque type with ℝ (Real, computable including sqrt).

    Unlike Rat, ``Real.sqrt`` is computable (via Cauchy sequences in
    mathlib), so modules using ``sqrt`` do NOT need ``noncomputable``.
    ``sqrt`` / ``exp`` / ``ln`` are rewritten to the concrete Mathlib
    functions (``Real.sqrt`` / ``Real.exp`` / ``Real.log``) so impl and
    spec share the same definition; other transcendentals stay as opaque axioms.
    """
    result = _replace_f64_base(content, "ℝ", "RealClone")

    # Add Mathlib import and noncomputable section
    result = re.sub(
        r"import Aeneas\n",
        "import Aeneas\nimport Mathlib\n",
        result,
        count=1,
    )
    result = result.replace("noncomputable section\n", "")
    result = result.replace(
        "set_option linter.style.nameCheck false\n",
        "set_option linter.style.nameCheck false\n"
        "noncomputable section\n",
    )

    # Replace transcendental functions with Mathlib equivalents
    result = _replace_transcendental(result, "sqrt", "Real.sqrt")
    result = _replace_transcendental(result, "exp", "Real.exp")
    result = _replace_transcendental(result, "ln", "Real.log")

    # Replace f64 constants with Mathlib equivalents
    result = _replace_f64_const(result, "core.f64.consts.PI", "Real.pi")

    return result


# ── integer model replacement ──────────────────────────────────────────────────

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
        if model == "bitvec" and not is_signed:
            content = re.sub(
                rf"(\d+){re.escape(suffix)}\b",
                rf"(\1 : BitVec {width})",
                content,
            )
        else:
            content = re.sub(rf"{re.escape(suffix)}\b", "", content)

    content = _monadic_ops_to_pure(content, sorted(produced))

    # Add Clone / PartialEq helpers for target types actually referenced
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


# ── monadic → pure conversion ──────────────────────────────────────────────────

# Regex to match a parameter declaration: ``(name : Type)``
_PARAM_RE = re.compile(r"\((\w+)\s*:\s*([^)]*)\)")
# Operand: bare identifier / numeral / parenthesized literal / usize literal
_OP_RE = r"(?:\w+|\([^()]*\)|\d+#usize)"
_CAP_RE = rf"({_OP_RE})"
# Prefix before the operator: ``let x ←`` (optionally after then/else)
_PREFIX_RE = r"^(\s*(?:then\s+|else\s+)?let\s+(\w+)\s+)←\s+"


def _seed_params_from_lines(
    lines: list[str],
    ctx_types: list[str],
    int_vars: set[str],
    usize_vars: set[str],
) -> None:
    """Extract parameter names from signature lines and seed tracking sets."""
    for line in lines:
        for m in _PARAM_RE.finditer(line):
            name, ty = m.group(1), m.group(2).strip().lstrip("(")
            if ty == "Std.Usize":
                usize_vars.add(name)
            elif ty in ctx_types:
                int_vars.add(name)


def _monadic_ops_to_pure(content: str, ctx_types: list[str]) -> str:
    """Rewrite ``let x ← <int-expr>`` → ``let x := <int-expr>`` for pure-int ops.

    Tracks integer- vs usize-typed identifiers so that usize arithmetic (still
    monadic on ``Std.Usize``) is left untouched.

    ``int_vars``/``usize_vars`` are reset at each ``def`` boundary because
    Aeneas reuses sequential names (``i1``, ``i2``, …) across functions, and
    a variable that is usize-typed in one function may be BitVec-typed in
    another.
    """
    int_vars: set[str] = set()
    usize_vars: set[str] = set()
    _DEF_RE = re.compile(r"^\s*def\s+\w+")
    _SIG_END_RE = re.compile(r":=\s*(do\b)?")

    out: list[str] = []
    in_signature = False
    sig_lines: list[str] = []

    for line in content.split("\n"):
        if _DEF_RE.match(line):
            int_vars.clear()
            usize_vars.clear()
            in_signature = True
            sig_lines = [line]
            if _SIG_END_RE.search(line):
                in_signature = False
                _seed_params_from_lines([line], ctx_types, int_vars, usize_vars)
            out.append(line)
        elif in_signature:
            sig_lines.append(line)
            if _SIG_END_RE.search(line):
                in_signature = False
                _seed_params_from_lines(sig_lines, ctx_types, int_vars, usize_vars)
            out.append(line)
        else:
            out.append(_convert_let_line(line, int_vars, usize_vars))
    return "\n".join(out)


# ── single-line let conversion ─────────────────────────────────────────────────

def _convert_let_line(line: str, int_vars: set[str], usize_vars: set[str]) -> str:
    """Convert one ``let x ← A op B`` line to ``:=`` if A, B are integer-typed.

    Handles ``let`` at line start or after ``then``/``else`` (aeneas indents
    ``then let x ← …`` on its own line).
    """

    def is_int(tok: str) -> bool:
        return tok in int_vars or _is_int_literal(tok)

    def is_usize(tok: str) -> bool:
        return tok in usize_vars or re.match(r"\d+#usize", tok) is not None

    def track(x: str, a: str, b: str) -> None:
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
        # If one operand is int-typed and the other is a bare identifier,
        # it was likely assigned from a function call that returns BitVec/Int.
        if is_int(a) and not is_int(b) and not is_usize(b) and re.match(r"^\w+$", b):
            int_vars.add(b)
            int_vars.add(x)
            return f"{prefix}:= {a} {op} {b}"
        if not is_int(a) and is_int(b) and not is_usize(a) and re.match(r"^\w+$", a):
            int_vars.add(a)
            int_vars.add(x)
            return f"{prefix}:= {a} {op} {b}"
        # Both operands are unknown bare identifiers (e.g. from a function
        # call return like ``let (result, num) ← loop ...``) and neither
        # is usize-typed — assume they are integer-typed.
        if (not is_usize(a) and not is_usize(b)
                and re.match(r"^\w+$", a) and re.match(r"^\w+$", b)):
            int_vars.add(a)
            int_vars.add(b)
            int_vars.add(x)
            return f"{prefix}:= {a} {op} {b}"
        return line

    # ── `←` patterns (monadic → pure) ──

    # arithmetic:  let x ← A + B
    m = re.match(_PREFIX_RE + rf"{_CAP_RE}\s*([+\-*/%])\s*{_CAP_RE}\s*$", line)
    if m:
        return apply(*m.groups())

    # bitwise:  let x ← lift (A &&& B)
    m = re.match(
        _PREFIX_RE + rf"lift\s*\(\s*{_CAP_RE}\s*(&&&|\|\|\||\^\^\^)\s*{_CAP_RE}\s*\)\s*$",
        line,
    )
    if m:
        return apply(*m.groups())

    # shift:  let x ← A <<< B
    m = re.match(_PREFIX_RE + rf"{_CAP_RE}\s*(<<<|>>>)\s*{_CAP_RE}\s*$", line)
    if m:
        return apply(*m.groups())

    # ── `:=` patterns (already pure — just track types) ──

    _pure_prefix = r"^\s*(?:then\s+|else\s+)?let\s+(\w+)\s+:=\s+"

    # arithmetic:  let x := A + B
    m = re.match(
        rf"{_pure_prefix}{_CAP_RE}\s*([+\-*/%])\s*{_CAP_RE}\s*$", line
    )
    if m:
        x, a, _opx, b = m.groups()
        if is_usize(a) or is_usize(b):
            usize_vars.add(x)
            return line.replace(":=", "←", 1)
        track(x, a, b)
        return line

    # bitwise:  let x := A &&& B
    m = re.match(
        rf"{_pure_prefix}{_CAP_RE}\s*(&&&|\|\|\||\^\^\^)\s*{_CAP_RE}\s*$", line
    )
    if m:
        x, a, _opx, b = m.groups()
        track(x, a, b)
        return line

    # shift:  let x := A <<< B
    m = re.match(
        rf"{_pure_prefix}{_CAP_RE}\s*(<<<|>>>)\s*{_CAP_RE}\s*$", line
    )
    if m:
        x, a, _opx, b = m.groups()
        track(x, a, b)
        return line

    # ── tail-position expressions (no `let`) — wrap in `ok (…)` ──

    def tail_ok(a: str, op: str, b: str) -> str | None:
        if is_int(a) and is_int(b) and not is_usize(a) and not is_usize(b):
            return f"ok ({a} {op} {b})"
        return None

    _tail_prefix = r"^(\s*)((?:then\s+|else\s+)?)"

    # arithmetic tail
    m = re.match(rf"{_tail_prefix}{_CAP_RE}\s*([+\-*/%])\s*{_CAP_RE}\s*$", line)
    if m:
        r = tail_ok(m.group(3), m.group(4), m.group(5))
        return m.group(1) + m.group(2) + r if r else line

    # bitwise tail: lift (A &&& B)
    m = re.match(
        rf"{_tail_prefix}lift\s*\(\s*{_CAP_RE}\s*(&&&|\|\|\||\^\^\^)\s*{_CAP_RE}\s*\)\s*$",
        line,
    )
    if m:
        r = tail_ok(m.group(3), m.group(4), m.group(5))
        return m.group(1) + m.group(2) + r if r else line

    # shift tail
    m = re.match(rf"{_tail_prefix}{_CAP_RE}\s*(<<<|>>>)\s*{_CAP_RE}\s*$", line)
    if m:
        r = tail_ok(m.group(3), m.group(4), m.group(5))
        return m.group(1) + m.group(2) + r if r else line

    return line


# ── bitvec scalar-op rewriting ─────────────────────────────────────────────────

def _bitvec_ops_to_bv(content: str) -> str:
    """Rewrite Aeneas scalar ops ``core.num.U<w>.*`` to ``BitVec`` ops.

    The ``bitvec`` int-model rewrites ``Std.U<w>`` *types* to ``BitVec w``, but
    Aeneas's named scalar methods (``rotate_left``, ``wrapping_add``, …) still
    target Aeneas's own ``U<w>`` scalar type and would be type-incompatible with
    ``BitVec w``.  Under ``BitVec w`` the equivalent is:

    * wrapping arithmetic → the plain operator (``BitVec`` ``+ - * / %`` wrap),
    * ``rotate_left/right``  → ``BitVec.rotateLeft/rotateRight <x> <n>.toNat``
      (the shift argument is ``BitVec w``, but the BitVec rotate takes ``Nat``).

    Only unsigned widths (``u8/u16/u32/u64/u128``) are handled; ``usize``
    operates on ``Std.Usize`` and is left untouched.
    """
    if "core.num.U" not in content and "UScalar.cast" not in content and "IScalar.cast" not in content:
        return content

    arg = r"(?:\([^()]*\)|[\w.]+)"

    for fn, op in (
        ("wrapping_add", "+"),
        ("wrapping_sub", "-"),
        ("wrapping_mul", "*"),
    ):
        content = re.sub(
            rf"core\.num\.U(?:8|16|32|64|128)\.{fn}\s+({arg})\s+({arg})",
            rf"\1 {op} \2",
            content,
        )

    for fn, bv in (("rotate_left", "rotateLeft"), ("rotate_right", "rotateRight")):
        content = re.sub(
            rf"core\.num\.U(?:8|16|32|64|128)\.{fn}\s+({arg})\s+({arg})",
            rf"BitVec.{bv} \1 \2.toNat",
            content,
        )

    if "UScalar.cast" in content or "IScalar.cast" in content:
        content = re.sub(
            rf"UScalar\.cast\s+\.Usize\s+({arg})",
            r"UScalar.mk (BitVec.ofNat (UScalarTy.Usize.numBits) \1.toNat)",
            content,
        )
        for w in (8, 16, 32, 64, 128):
            content = re.sub(
                rf"UScalar\.cast\s+\.U{w}\s+({arg})",
                rf"(BitVec.ofNat {w} \1.toNat)",
                content,
            )
        for w in (8, 16, 32, 64, 128):
            content = re.sub(
                rf"IScalar\.cast\s+\.I{w}\s+({arg})",
                rf"(BitVec.ofInt {w} \1.toInt)",
                content,
            )

    return content


# ── misc fixups ────────────────────────────────────────────────────────────────

def _mark_noncomputable_defs(text: str) -> str:
    """Mark defs that depend on axioms (directly or transitively)
    as noncomputable.

    Without this, Lean's code generator rejects definitions whose
    evaluation chain reaches an axiom.
    """
    axiom_names: set[str] = set()
    for m in re.finditer(
        r"^\s*(?:noncomputable\s+)?axiom\s+([\w.]+)", text, re.MULTILINE
    ):
        axiom_names.add(m.group(1))
    if not axiom_names:
        return text

    decls: list[tuple[str, str, int, int]] = []
    pattern = re.compile(
        r"^(?:@\[[^\]]*\][ \t]*\n\s*)?"
        r"(?:(noncomputable[ \t]+)?(def|theorem)[ \t]+(\w+))",
        re.MULTILINE,
    )
    for m in pattern.finditer(text):
        name = m.group(3)
        is_noncomp = m.group(1) is not None
        body_start = m.end()
        next_decl = re.search(
            r"^\s*(?:end\b|namespace\b|(?:noncomputable\s+)?(?:def|axiom|theorem)\s+)",
            text[body_start:],
            re.MULTILINE,
        )
        body_end = body_start + next_decl.start() if next_decl else len(text)
        body = text[body_start:body_end]
        body = re.sub(r"/-.*?-/", "", body, flags=re.DOTALL)
        decls.append((name, body, m.start(), m.end()))
        if is_noncomp:
            axiom_names.add(name)

    # Fixed-point propagation
    changed = True
    while changed:
        changed = False
        for name, body, _, _ in decls:
            if name in axiom_names:
                continue
            if any(re.search(rf"\b{re.escape(n)}\b", body) for n in axiom_names):
                axiom_names.add(name)
                changed = True

    if not axiom_names:
        return text

    noncomp_names = sorted(
        set(n for n in axiom_names if n not in set(
            re.findall(r"^\s*(?:noncomputable\s+)?axiom\s+([\w.]+)", text, re.MULTILINE)
        )),
        key=len,
        reverse=True,
    )
    if not noncomp_names:
        return text

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
    checker accepts them.  Also strips incompatible attributes.
    """
    for tag, label in [
        (r"@\[rust_loop\]\n", "_quick_sort_loop"),
        (r"@\[rust_loop_body\]\n", r"_quick_sort_loop\.body"),
        (r"@\[reducible\]\n", "_quick_sort"),
    ]:
        text = re.sub(
            rf"{tag}(def {label}\b.*?)partial_fixpoint",
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
    text = re.sub(
        r"alloc\.vec\.Vec\.is_empty\s+Global\s+(\w+)",
        r"ok ((alloc.vec.Vec.deref \1).length = 0)",
        text,
    )
    return text


# ── main entry point ───────────────────────────────────────────────────────────

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
        if int_model == "bitvec":
            text = _bitvec_ops_to_bv(text)
            # _monadic_ops_to_pure ran inside lean_integers_to_model *before*
            # _bitvec_ops_to_bv rewrote core.num.U*.* ops to BitVec operators.
            # Run it again so that the now-pure BitVec ops (+, -, <<<, …)
            # get their `←` converted to `:=`.
            produced = {
                _int_target_type(is_signed, width, int_model)
                for _, _, width, is_signed in _INT_TYPES
            }
            text = _monadic_ops_to_pure(text, sorted(produced))
        text = inline_f64_helpers(text)
        text = inline_i64_helpers(text)
        if _use_real:
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