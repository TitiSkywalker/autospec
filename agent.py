"""Core agent loop — Anthropic tool-use conversation with Rust specification tools."""

import sys
import time
from pathlib import Path

import anthropic

import tools  # noqa: F401 — triggers tool registration
from prompt import (
    FUNC_SYSTEM_PROMPT,
    MODULE_SYSTEM_PROMPT,
    NEXT_STEP_PROMPT,
    SYSTEM_PROMPT,
)
from tool_registry import dispatch, get_definitions, set_context


class SpecAgent:
    """LLM agent that explores a Rust project and generates formal specifications.

    The agent uses Anthropic's tool-use API. It navigates the codebase with
    file/exploration tools, reads documentation and tests, then writes
    structured specifications to the output directory.
    """

    def __init__(
        self,
        project_path: str,
        output_dir: str = "specs",
        model: str = "claude-sonnet-4-6-20250514",
        max_iterations: int = 30,
        max_tokens: int = 4096,
        verbose: bool = False,
        api_key: str = "",
        base_url: str = "",
        system_prompt: str = "",
        tool_filter: list[str] | None = None,
    ):
        abs_project = str(Path(project_path).resolve())
        abs_output = str(Path(output_dir).resolve())
        set_context(abs_project, abs_output)

        self.project_path = abs_project
        self.output_dir = abs_output
        self.model = model
        self.max_iterations = max_iterations
        self.max_tokens = max_tokens
        self.verbose = verbose
        self.client = anthropic.Anthropic(
            api_key=api_key,
            base_url=base_url or None,
            timeout=120.0,  # per-request timeout in seconds
        )
        all_tools = get_definitions()
        if tool_filter is not None:
            self.tool_defs = [t for t in all_tools if t["name"] in tool_filter]
        else:
            self.tool_defs = all_tools

        self._system = [
            {
                "type": "text",
                "text": system_prompt or SYSTEM_PROMPT,
                "cache_control": {"type": "ephemeral"},
            }
        ]

    def run(self, task: str = "", quiet: bool = False) -> None:
        """Run the specification-generation conversation loop.

        If *task* is given, use it as the initial user prompt (for per-module
        sub-agents). Otherwise use the default monolithic exploration prompt.

        If *quiet* is True, suppress text-block output and completion messages
        (used for single-harness function agents where only the tool result matters).
        """
        self._quiet = quiet
        if task:
            task_prompt = task
        else:
            task_prompt = (
                f"Please analyze the Rust project at `{self.project_path}`. "
                f"1. Call generate_doc_json(features='all'). "
                f"2. Call read_rustdoc() to see the module summary. "
                f"3. Drill into the 2-4 most important modules with read_rustdoc(module=...). "
                f"4. Call write_spec IMMEDIATELY after drilling — do NOT read source "
                f"   files unless a function has zero doc comments. "
                f"Doc comments are the ONLY source of truth for pre/post conditions. "
                f"Tests are only for call-pattern reference. "
                f"Write the spec to `{self.output_dir}/spec.rs`."
            )

        messages = [{"role": "user", "content": task_prompt}]

        for iteration in range(1, self.max_iterations + 1):
            if self.verbose and not quiet:
                print(
                    f"\n{'─'*40} Iteration {iteration}/{self.max_iterations} {'─'*40}",
                    file=sys.stderr,
                )

            # Warn the model when approaching the budget (skip for quiet/single-harness agents).
            if not quiet:
                remaining = self.max_iterations - iteration
                if remaining == 1:
                    messages.append(
                        {
                            "role": "user",
                            "content": "[SYSTEM] You have ONE iteration remaining. "
                            "Use write_spec NOW to save your specification, "
                            "then respond with a brief summary.",
                        }
                    )
                elif remaining <= 3 and remaining > 1:
                    messages.append(
                        {
                            "role": "user",
                            "content": f"[SYSTEM] {remaining} iterations remaining. "
                            "Begin writing the specification with write_spec soon.",
                        }
                    )

            response = self._call_api(messages)

            # Separate text and tool-use blocks from the response.
            assistant_blocks: list[dict] = []
            tool_blocks: list[anthropic.types.ToolUseBlock] = []

            for block in response.content:
                if block.type == "text":
                    assistant_blocks.append({"type": "text", "text": block.text})
                    if self.verbose and not quiet:
                        print(
                            f"\n[text] {block.text[:300]}{'...' if len(block.text) > 300 else ''}",
                            file=sys.stderr,
                        )
                elif block.type == "tool_use":
                    assistant_blocks.append(
                        {
                            "type": "tool_use",
                            "id": block.id,
                            "name": block.name,
                            "input": block.input,
                        }
                    )
                    tool_blocks.append(block)
                elif block.type in ("thinking", "redacted_thinking"):
                    # Pass thinking blocks back through — required by
                    # DeepSeek and Claude extended-thinking models.
                    assistant_blocks.append(block.model_dump())

            messages.append({"role": "assistant", "content": assistant_blocks})

            # No tool calls means the agent considers the task complete.
            if not tool_blocks:
                if not quiet:
                    final_text = " ".join(
                        b["text"] for b in assistant_blocks if b["type"] == "text"
                    )
                    print(f"\nAgent finished after {iteration} iterations.")
                    if final_text:
                        print(f"\n{final_text}")
                break

            # Execute tools and collect results.
            if self.verbose and not quiet:
                print(
                    f"\n[tools] Running {len(tool_blocks)} tool(s)...", file=sys.stderr
                )

            tool_results: list[dict] = []
            for tb in tool_blocks:
                if self.verbose and not quiet:
                    print(f"  [debug] {tb.name} input={tb.input}", file=sys.stderr)
                result = dispatch(tb.name, tb.input)
                tool_results.append(
                    {
                        "type": "tool_result",
                        "tool_use_id": tb.id,
                        "content": result,
                    }
                )
                if self.verbose and not quiet:
                    preview = result[:120].replace("\n", " ")
                    print(
                        f"  {tb.name}: {preview}{'...' if len(result) > 120 else ''}",
                        file=sys.stderr,
                    )

            messages.append({"role": "user", "content": tool_results})
        else:
            # Exhausted all iterations.
            if not quiet:
                print(
                    f"\nReached iteration limit ({self.max_iterations}). "
                    "Forcing final summary..."
                )
            messages.append(
                {
                    "role": "user",
                    "content": "You have exhausted your iteration budget. "
                    "If you have not yet written the specification, "
                    "use write_spec NOW to save what you have, "
                    "then respond with a brief summary.",
                }
            )
            response = self._call_api(messages)
            for block in response.content:
                if block.type == "text":
                    if not quiet:
                        print(block.text)
                elif block.type == "tool_use":
                    result = dispatch(block.name, block.input)
                    if not quiet:
                        print(f"\n[tool] {block.name}: {result[:300]}")

    # ————————————————————————————— internals —————————————————————————————

    def _call_api(self, messages: list[dict]) -> anthropic.types.Message:
        """Call the Anthropic Messages API with prompt-cached system prompt."""
        t0 = time.monotonic()
        response = self.client.messages.create(
            model=self.model,
            max_tokens=self.max_tokens,
            system=self._system,
            messages=messages,
            tools=self.tool_defs,
        )
        elapsed = time.monotonic() - t0
        if self.verbose and not getattr(self, "_quiet", False):
            usage = response.usage
            tools = [b.name for b in response.content if b.type == "tool_use"]
            if tools:
                detail = "→ " + ", ".join(tools)
            elif response.stop_reason == "end_turn":
                detail = "→ (done)"
            else:
                detail = f"stop={response.stop_reason}"
            print(
                f"[api] {elapsed:.1f}s  "
                f"input={usage.input_tokens} output={usage.output_tokens}  "
                f"{detail}",
                file=sys.stderr,
            )
        return response
