#!/usr/bin/env python3
"""autospec — Generate formal specifications for Rust projects from docs and tests.

Usage:
    autospec PROJECT_PATH [--output DIR] [--model MODEL] [--max-iterations N] [--verbose]

Examples:
    autospec ~/my-rust-lib
    autospec ~/my-rust-lib --output specs/ --model claude-opus-4-7-20250514
    autospec ~/my-rust-lib --max-iterations 50 --verbose
"""

import argparse
import os
import sys
from pathlib import Path

from dotenv import load_dotenv

# Load .env from current directory and ~/.autospec/ for ANTHROPIC_API_KEY.
load_dotenv(Path.home() / ".autospec" / ".env")
load_dotenv()


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="autospec",
        description="Generate formal specifications for Rust projects using an LLM agent",
    )
    parser.add_argument(
        "project_path",
        help="Path to the Rust project root (must contain Cargo.toml)",
    )
    parser.add_argument(
        "--output", "-o",
        default="specs",
        help="Output directory for specifications (default: specs/)",
    )
    parser.add_argument(
        "--model", "-m",
        default="claude-sonnet-4-6-20250514",
        help="Anthropic model ID to use (default: claude-sonnet-4-6-20250514)",
    )
    parser.add_argument(
        "--max-iterations", "-n",
        type=int,
        default=30,
        help="Maximum tool-calling iterations (default: 30)",
    )
    parser.add_argument(
        "--max-tokens",
        type=int,
        default=4096,
        help="Maximum tokens per model response (default: 4096)",
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Print detailed progress to stderr",
    )

    args = parser.parse_args()

    # Validate project path
    project = Path(args.project_path).resolve()
    if not project.is_dir():
        print(f"Error: {project} is not a directory", file=sys.stderr)
        sys.exit(1)
    if not (project / "Cargo.toml").is_file():
        print(f"Error: Cargo.toml not found in {project}", file=sys.stderr)
        sys.exit(1)

    # Check API key (supports both Anthropic and DeepSeek API keys)
    api_key = os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_AUTH_TOKEN")
    if not api_key:
        print("Error: ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN not set. "
              "Set it in the environment or in ~/.autospec/.env",
              file=sys.stderr)
        sys.exit(1)
    base_url = os.environ.get("ANTHROPIC_BASE_URL", "")

    output = Path(args.output).resolve()
    print(f"Project:  {project}")
    print(f"Output:   {output}")
    print(f"Model:    {args.model}")
    print(f"Max iterations: {args.max_iterations}")
    print()

    from agent import SpecAgent

    agent = SpecAgent(
        project_path=str(project),
        output_dir=str(output),
        model=args.model,
        max_iterations=args.max_iterations,
        max_tokens=args.max_tokens,
        verbose=args.verbose,
        api_key=api_key,
        base_url=base_url,
    )

    agent.run()

    print(f"\nDone. Output written to {output}/")


if __name__ == "__main__":
    main()
