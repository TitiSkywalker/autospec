"""Post-process generated Lean files: F64→Rat, inline helpers, write output."""

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
    result = re.sub(
        r"\baxiom \b(?!.*noncomputable)", "noncomputable axiom ", result
    )
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
) -> tuple[int, list[str]]:
    """Post-process and copy Lean files from *workdir* to *output_path*.

    Returns ``(def_count, written_filenames)``.
    """
    lean_files = sorted(f for f in os.listdir(str(workdir)) if f.endswith(".lean"))
    def_count = 0
    written: list[str] = []

    for f in lean_files:
        text = (workdir / f).read_text()
        text = text.replace(
            "set_option linter.unusedVariables false",
            "set_option linter.unusedVariables false\n"
            "set_option linter.style.nameCheck false",
        )
        text = lean_f64_to_rat(text)
        text = inline_f64_helpers(text)
        text = _fix_partial_mutual(text)
        (output_path / f).write_text(text)
        written.append(f)
        def_count += len(
            re.findall(r"^\s*(?:def|theorem|axiom)\s+\S+", text, re.MULTILINE)
        )

    return def_count, written
