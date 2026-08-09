"""Post-process generated Lean files: F64→Rat/ℝ, inline helpers, write output."""

import os
import re
from pathlib import Path

# Import from sibling tools package
import sys

_tools_dir = str(Path(__file__).resolve().parent.parent / "tools")
if _tools_dir not in sys.path:
    sys.path.insert(0, _tools_dir)
from lean_inline import inline_f64_helpers  # noqa: E402


def lean_f64_to_rat(content: str) -> str:
    """Replace F64 opaque type with Rat (computable rational arithmetic)."""
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
    # When sqrt is present the code generator cannot compile the module
    # (Rat has no computable sqrt).  Insert ``noncomputable section`` so
    # that the code generator skips all definitions in the file.
    if "std.f64.Rat.sqrt" in result:
        result = result.replace("noncomputable section\n", "")
        result = result.replace(
            "set_option linter.style.nameCheck false\n",
            "set_option linter.style.nameCheck false\n"
            "noncomputable section\n",
            count=1,
        )
    else:
        result = result.replace("noncomputable section\n", "")

    if "def RatClone.clone" not in result:
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

    if "def RealClone.clone" not in result:
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


# Integer types that Aeneas translates (all → Int) and their literal suffixes.
_INT_TYPES: list[tuple[str, str]] = [
    ("Std.I8", "#i8"),
    ("Std.I16", "#i16"),
    ("Std.I32", "#i32"),
    ("Std.I64", "#i64"),
    ("Std.I128", "#i128"),
    ("Std.Isize", "#isize"),
    ("Std.U8", "#u8"),
    ("Std.U16", "#u16"),
    ("Std.U32", "#u32"),
    ("Std.U64", "#u64"),
    ("Std.U128", "#u128"),
    # Std.Usize is intentionally excluded — used for array indexing.
]


def lean_integers_to_int(content: str) -> str:
    """Replace Aeneas integer types (I8–I128, U8–U128, Isize) with Int.

    Like ``F64→Rat``, this eliminates bit-width / overflow concerns so
    verification focuses on algorithmic correctness.

    ``Std.Usize`` is **not** replaced — it is used for array indexing.
    """
    # Check for any integer-related tokens (types, literals, trait instances)
    _INT_MARKERS = (
        [t for t, _ in _INT_TYPES]
        + [s for _, s in _INT_TYPES]
        + [
            "PartialEqI8",
            "PartialEqI16",
            "PartialEqI32",
            "PartialEqI64",
            "PartialEqI128",
            "PartialEqIsize",
            "PartialEqU8",
            "PartialEqU16",
            "PartialEqU32",
            "PartialEqU64",
            "PartialEqU128",
        ]
    )
    if not any(m in content for m in _INT_MARKERS):
        return content

    for ty, suffix in _INT_TYPES:
        # Replace the type itself
        content = content.replace(ty, "Int")
        # Replace the clone instance: core.clone.CloneI32 → IntClone
        clone_name = ty.replace("Std.", "")
        content = content.replace(f"core.clone.Clone{clone_name}", "IntClone")
        # Replace PartialEq instance: core.cmp.PartialEqI32 → PartialEqInt
        content = content.replace(f"core.cmp.PartialEq{clone_name}", "PartialEqInt")
        # Strip literal suffixes: 5#i32 → 5, (-5)#i32 → (-5)
        content = re.sub(rf"{re.escape(suffix)}\b", "", content)

    # After type replacement, arithmetic ops that were monadic on
    # Std.U64/Std.I32 (returning Result) are now pure Int operations.
    # Convert ``let x ← a % b`` → ``let x := a % b``.
    # Only % — the only op that appears on Int values after type
    # replacement (gcd).  / + - involve Usize which stays monadic.
    for op in ["%"]:
        escaped = re.escape(op)
        content = re.sub(
            rf"(let\s+\w+\s+)←\s+([^\s#]+)\s*{escaped}\s*([^\s#]+)",
            rf"\1:= \2 {op} \3",
            content,
        )

    extras: list[str] = []
    # Define helpers when Int-related types were replaced in this file.
    # IntClone is always needed; PartialEqInt is needed by tests (assert_eq!).
    # Define both helpers when integer types were replaced.
    # IntClone is needed in impl files (to_vec); PartialEqInt is needed
    # in test files (assert_eq!).  Since tests import the impl module,
    # we always define both together so tests can find them.
    need_clone = "def IntClone.clone" not in content and "IntClone" in content
    need_peq = "def PartialEqInt.eq" not in content and "PartialEqInt" in content
    if need_clone or need_peq:
        if "def IntClone.clone" not in content:
            extras.append(
                "/- Integer types replaced with Int by autospec -/\n"
                "def IntClone.clone (x : Int) : Result Int := ok x\n"
                "\n"
                "@[reducible]\n"
                "def IntClone : core.clone.Clone Int := {\n"
                "  clone := IntClone.clone\n"
                "}\n"
            )
        if "def PartialEqInt.eq" not in content:
            extras.append(
                "def PartialEqInt.eq (x y : Int) : Result Bool := ok (x = y)\n"
                "\n"
                "@[reducible]\n"
                "def PartialEqInt : core.cmp.PartialEq Int Int := {\n"
                "  eq := PartialEqInt.eq\n"
                "}\n"
            )

    if extras:
        block = "\n".join(extras)
        content = re.sub(r"(namespace \w+\n)", r"\1\n" + block, content, count=1)

    return content


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


def write_lean_output(
    workdir: Path,
    output_path: Path,
    *,
    label: str = "",
    target: str = "Rat",
) -> tuple[int, list[str]]:
    """Post-process and copy Lean files from *workdir* to *output_path*.

    Parameters
    ----------
    target:
        ``"Rat"`` (default) — replace f64 with Rat; sqrt is noncomputable.
        ``"Real"`` — replace f64 with ℝ; sqrt is computable via Real.sqrt.
    """
    lean_files = sorted(f for f in os.listdir(str(workdir)) if f.endswith(".lean"))
    def_count = 0
    written: list[str] = []

    _use_real = target == "Real"

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
            text = lean_f64_to_rat(text)
        text = lean_integers_to_int(text)
        text = inline_f64_helpers(text)
        if _use_real:
            # f64_helpers constants and helpers use ``Rat`` — fix type
            text = re.sub(r"\bRat\b", "ℝ", text)
            text = re.sub(r"\bRatClone\b", "RealClone", text)
        text = _fix_partial_mutual(text)
        (output_path / f).write_text(text.rstrip("\n") + "\n")
        written.append(f)
        def_count += len(
            re.findall(r"^\s*(?:def|theorem|axiom)\s+\S+", text, re.MULTILINE)
        )

    return def_count, written
