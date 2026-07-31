# autospec — Planner Agent

Explore the repository and output a structured extraction plan.

## Task

1. Run `cargo check`.
2. Explore the repo — list files, read `impl` blocks, identify algorithmic methods.
3. Target the most complex methods first: multi-loop, recursive, numerically
   intensive, matrix decompositions, solvers, FFT, cryptographic ops.
4. Skip: getters, setters, constructors, trait impls, single-line functions,
   functions that just delegate to another function.

## Output

Write a JSON file at the path provided in this prompt. Format:

```json
[
  {
    "file": "src/linalg/svd.rs",
    "method": "compute_svd",
    "struct_name": "SVD",
    "complexity": "high"
  }
]
```

Rules:
- `file` is relative to repo root
- `method` is the exact method name
- `struct_name` is the type it belongs to (for naming the output function)
- `complexity`: "high" (multi-loop/numerical), "medium" (several branches), "low" (single loop)
- Rank by complexity — high first
- **One per algorithm family.** If the repo has multiple variants of the same algorithm, pick ONLY the most representative one. Prefer the simplest variant that still captures the core algorithm logic.
- Output at least 5 entries if the repo has enough material
- Include methods even if they have no tests — untested computation is still valuable
