#!/usr/bin/env bash
# ==============================================================================
# fetch_ai_txt.sh — Fetch latest live tscircuit AI context documentation
# ==============================================================================
set -euo pipefail

DEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/references"
TARGET_FILE="$DEST_DIR/tscircuit_ai.txt"

echo "Fetching latest tscircuit ai.txt..."
if curl -sSL --fail "https://docs.tscircuit.com/ai.txt" -o "$TARGET_FILE" || curl -sSL --fail "https://tscircuit.com/ai.txt" -o "$TARGET_FILE"; then
    echo "✓ Saved $(wc -l < "$TARGET_FILE") lines of tscircuit context to $TARGET_FILE"
else
    echo "✗ Failed to fetch live ai.txt from docs.tscircuit.com (offline or network error)."
    exit 1
fi
