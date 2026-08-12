"""Simplify Rust source so aeneas can handle it.

These are semantics-preserving text rewrites applied in-place on a
temporary copy — the original project is never touched.
"""

import re
from pathlib import Path


def simplify_for_aeneas(project_dir: Path) -> None:
    """Transform Rust source in-place for aeneas compatibility.

    Handles patterns that aeneas does not support:
    *copy_from_slice*, *min()*, explicit ``&mut *`` deref.
    """
    for rs in project_dir.rglob("*.rs"):
        if "target" in rs.parts:
            continue
        text = rs.read_text()
        changed = False

        # .copy_from_slice(other) → while loop
        m = re.search(r"(\w+)\.copy_from_slice\((\w+)\);", text)
        if m:
            text = text.replace(
                m.group(0),
                f"{{ let mut __i: usize = 0; while __i < {m.group(1)}.len()"
                f" {{ {m.group(1)}[__i] = {m.group(2)}[__i]; __i += 1; }}}}",
            )
            changed = True

        # a.min(b) for usize → if a < b { a } else { b }
        m = re.search(r"(\w+)\.min\((\w+)\)", text)
        if m:
            text = text.replace(
                m.group(0),
                f"(if {m.group(1)} < {m.group(2)}"
                f" {{ {m.group(1)} }} else {{ {m.group(2)} }})",
            )
            changed = True

        # &mut *var → var (explicit deref+ref)
        if "&mut *" in text:
            text = text.replace("&mut *lu_data", "lu_data")
            text = text.replace("&mut *ipiv", "ipiv")
            changed = True

        # <vec>.iter().all(|&<binder>| <var>) → manual while loop
        # aeneas cannot translate Iterator::all (takes a closure, and the
        # method implementation is an external stdlib definition).
        for m in re.finditer(
            r"(\w+)\.iter\(\)\.all\(\|&(\w+)\|\s*(\w+)\)",
            text,
        ):
            vec, binder, cond = m.group(1), m.group(2), m.group(3)
            if binder == cond:
                text = text.replace(
                    m.group(0),
                    "{ let mut __all = true; let mut __i: usize = 0;"
                    f" while __i < {vec}.len()"
                    f" {{ if !{vec}[__i]"
                    f" {{ __all = false; break; }} __i += 1; }} __all }}",
                )
                changed = True

        if changed:
            rs.write_text(text)
