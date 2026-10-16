"""Count subdirectories named like {category}_{testcase} under a parent directory."""

import argparse
import os
import sys
from collections import Counter, defaultdict


def split_name(name, sep, mode):
    """Split a directory name into (category, testcase); return (None, name) if no separator."""
    if sep not in name:
        return None, name
    if mode == "first":  # category contains no separator (the common case)
        cat, _, case = name.partition(sep)
    else:  # testcase contains no separator, category may
        cat, _, case = name.rpartition(sep)
    return cat, case


def main():
    ap = argparse.ArgumentParser(description="Count {category}_{testcase} directories")
    ap.add_argument(
        "root",
        nargs="?",
        default=".",
        help="parent directory (default: current directory)",
    )
    ap.add_argument("--sep", default="_", help="separator, default '_'")
    ap.add_argument(
        "--split",
        choices=("first", "last"),
        default="first",
        help="first=split at the first separator (default); last=split at the last separator",
    )
    ap.add_argument(
        "--include-files",
        action="store_true",
        help="count files as testcases too (default: directories only)",
    )
    ap.add_argument(
        "--sort",
        choices=("count", "name"),
        default="count",
        help="sort order, default: by count descending",
    )
    args = ap.parse_args()

    root = os.path.abspath(args.root)
    if not os.path.isdir(root):
        sys.exit(f"not a directory: {root}")

    counter = Counter()
    cases_by_cat = defaultdict(list)
    unparsed = []

    for entry in os.scandir(root):
        keep = (
            (entry.is_dir() or entry.is_file())
            if args.include_files
            else entry.is_dir(follow_symlinks=False)
        )
        if not keep:
            continue

        cat, case = split_name(entry.name, args.sep, args.split)
        if cat is None:
            unparsed.append(entry.name)
            continue
        counter[cat] += 1
        cases_by_cat[cat].append(case)

    total = sum(counter.values())
    width = max([len(c) for c in counter] + [len("category")])

    if args.sort == "name":
        items = sorted(counter.items())
    else:
        items = sorted(counter.items(), key=lambda kv: (-kv[1], kv[0]))

    print(f"root: {root}")
    print(f"{total} testcases, {len(counter)} categories\n")
    print(f"{'category':<{width}}  {'testcases':>9}")
    print("-" * (width + 12))
    for cat, n in items:
        print(f"{cat:<{width}}  {n:>9}")
    print("-" * (width + 12))
    print(f"{'TOTAL':<{width}}  {total:>9}")

    # A testcase name appearing under multiple categories may be a duplicate upload or intentional
    seen = Counter(c for v in cases_by_cat.values() for c in v)
    dup = sorted((c, n) for c, n in seen.items() if n > 1)
    if dup:
        print(f"\nnote: {len(dup)} testcase names appear in multiple categories")
        for case, n in dup[:10]:
            where = [cat for cat, v in cases_by_cat.items() if case in v]
            print(f"  {case} ({n} times): {', '.join(where)}")
        if len(dup) > 10:
            print(f"  ... {len(dup) - 10} more omitted")

    if unparsed:
        print(f"\nskipped {len(unparsed)} names without '{args.sep}':")
        for name in unparsed[:10]:
            print(f"  {name}")
        if len(unparsed) > 10:
            print(f"  ... {len(unparsed) - 10} more omitted")


if __name__ == "__main__":
    main()
