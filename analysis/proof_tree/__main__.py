"""``python3 -m analysis.proof_tree`` entry point."""

import sys
from pathlib import Path

# ``analysis/proof_tree/__main__.py`` -> ``analysis/proof_tree`` -> ``analysis`` -> repository root
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))  # make `evaluation.*` importable regardless of cwd

from analysis.proof_tree.cli import main

if __name__ == "__main__":
    main()
