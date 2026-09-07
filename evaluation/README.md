# Evaluation Pipeline

## Workspace

This directory implements the agentic evaluation pipeline for the Rust vericoding benchmark. Each agent should work in an independent workspace with configured Aeneas toolchain. For example, a typical workspace layout should look like:

```
- .lake
- aeneas
- RustVerif
    - Basic.lean
    - Function.lean
    - Spec.lean
- RustVerif.lean
- lean-toolchain
- lakefile.toml
```

This workspace can be created by the `create_workspace` function inside `evaluation/workspace.py`.

## Agent

The workspace defined above should be sufficient for verifying functions. Initially, `Function.lean` and `Spec.lean` are empty Lean files. For each testcase, we copy the translated Lean function and specification into `Function.lean` and `Spec.lean`. 

To enable concurrent processing, we initialize multiple independent workspaces, and route testcases in the benchmark to each workspace. This should be modeled as a "Producer-Consumer" model, where producer is the benchmark, and consumer is each workspace (with agent). They are connected with a single queue with lock.

The Lean proving agent could be any system that runs inside a terminal. We simply invoke it inside the workspace using cli command with prompt. For example, Leanstral-1.5 could be invoked by

```
vibe -p "Prove the theorems inside `RustVerif/Spec.lean`" --agent lean --auto-approve --max-turns N
```

> Optionally, we should setup SKILLs and MCP servers for agents.

## Evaluation

Each testcase is a Rust-Lean project. It contains Rust source and translated Lean source. For example, the testcase `array_binary_search` has the following structure (omitting irrelevant files):

```
.
├── ArrayBinarySearch.lean
├── Spec.lean
├── Test_test.lean
├── src
│   └── lib.rs
└── tests
    └── test.rs
```

During evaluation, `ArrayBinarySearch.lean` and `Spec.lean` should be extracted and copied into workspace directory. This also includes fixing the imports of Lean files.

After the agent stops, we need to check the `Spec.lean` independently to prevent agent from cheating. We collect and save the modified `Spec.lean` to another independent directory for verification.
