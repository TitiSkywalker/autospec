"""``python3 -m analysis`` — accuracy reports over verified evaluation runs."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))  # repository root

from analysis.run_accuracy import main  # noqa: E402

if __name__ == "__main__":
    main()
