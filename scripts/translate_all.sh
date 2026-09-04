#!/usr/bin/env bash
# Re-translate all LeanMode test cases concurrently using the Rust translator.
#
# Usage:
#   ./scripts/translate_all.sh            # default: 8 workers
#   ./scripts/translate_all.sh -w 16      # 16 workers
#   ./scripts/translate_all.sh --help     # all options
#
# Any extra arguments are forwarded to the Rust `translate_all` binary.

set -euo pipefail

# Locate the repository root (parent of this script's directory).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

CRATE_DIR="$ROOT/translation/translator"
BIN="$CRATE_DIR/target/release/translate_all"

# Build the binary if it doesn't exist yet.
if [[ ! -x "$BIN" ]]; then
    echo "Building translator + translate_all (release)..." >&2
    (cd "$CRATE_DIR" && cargo build --release --bin translate_all)
fi

exec "$BIN" "$@"
