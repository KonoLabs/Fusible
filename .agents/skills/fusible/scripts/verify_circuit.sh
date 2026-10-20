#!/usr/bin/env bash
# ==============================================================================
# verify_circuit.sh — Fusible Circuit Verification & Fuse Contract Runner
# ==============================================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Fusible Verification Runner${NC}"

# 1. Check for fusible CLI
if ! command -v fusible &>/dev/null; then
    echo -e "${RED}✗ Error: 'fusible' binary not found in PATH.${NC}"
    echo -e "  Install with: cargo install fusible (or build from ../Fusible)"
    exit 2
fi

# 2. Determine target
TARGET="${1:-}"
if [[ -z "$TARGET" ]]; then
    if [[ -f "fusible.yml" || -f "fusible.yaml" ]]; then
        echo -e "  Found manifest: ${YELLOW}fusible.yml${NC}"
    elif [[ -f "hardware/index.tsx" ]]; then
        TARGET="hardware/index.tsx"
        echo -e "  Using default entry: ${YELLOW}hardware/index.tsx${NC}"
    else
        echo -e "${RED}✗ No fusible.yml or hardware/index.tsx found in current directory.${NC}"
        echo "  Provide a file explicitly: ./scripts/verify_circuit.sh <path/to/circuit.tsx>"
        exit 2
    fi
fi

# 3. Run fusible check
echo -e "${BLUE}⚡ Running electrical rule checking & linter...${NC}"
set +e
if [[ -n "$TARGET" ]]; then
    fusible check "$TARGET"
    EXIT_CODE=$?
else
    fusible check
    EXIT_CODE=$?
fi
set -e

# 4. Interpret verdict
case $EXIT_CODE in
    0)
        echo -e "${GREEN}✅ SUCCESS: Fuse intact! All electrical and physical rules passed.${NC}"
        # Check if pins.rs was updated
        if [[ -f "fusible.yml" ]]; then
            PINS_RS=$(grep -E '^\s*pins_rs:' fusible.yml | awk '{print $2}' || true)
            if [[ -n "$PINS_RS" && -f "$PINS_RS" ]]; then
                echo -e "  ${GREEN}✓${NC} Firmware bindings updated: ${YELLOW}$PINS_RS${NC} ($(wc -l < "$PINS_RS") lines)"
            fi
        fi
        exit 0
        ;;
    1)
        echo -e "${RED}💥 THE FUSE BLEW! (Exit code 1)${NC}"
        echo -e "${RED}  Electrical violations detected in your circuit.${NC}"
        echo -e "  Common fixes:"
        echo -e "    1. Check for floating / single-connection nets in the report above."
        echo -e "    2. Verify all component pin labels match trace destinations."
        echo -e "    3. Ensure every net connects at least 2 distinct pads or ports."
        exit 1
        ;;
    2)
        echo -e "${YELLOW}⚠️ Toolchain / Syntax error (Exit code 2)${NC}"
        echo -e "  Ensure TSX syntax is valid and tscircuit is installed (run 'fusible doctor')."
        exit 2
        ;;
    *)
        echo -e "${RED}✗ Unexpected error code: $EXIT_CODE${NC}"
        exit "$EXIT_CODE"
        ;;
esac
