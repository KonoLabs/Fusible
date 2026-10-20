#!/usr/bin/env bash
# ==============================================================================
# smoke_test.sh — End-to-End Toolchain Health and Verification Sanity Check
# ==============================================================================
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🧪 Fusible & tscircuit Smoke Test${NC}"

# 1. Run doctor
echo -e "${YELLOW}1. Checking toolchain components...${NC}"
fusible doctor

# 2. Locate minimal template or generate inline in a local .tmp dir
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMPLATE="$SKILL_DIR/templates/minimal-board.tsx"

if [[ -f "$TEMPLATE" ]]; then
    echo -e "${YELLOW}2. Validating bundled minimal board template...${NC}"
    fusible check "$TEMPLATE"
else
    echo -e "${YELLOW}2. Creating local workspace test circuit...${NC}"
    LOCAL_TMP=".fusible_smoke_tmp"
    mkdir -p "$LOCAL_TMP"
    trap 'rm -rf "$LOCAL_TMP"' EXIT

    cat << 'EOF' > "$LOCAL_TMP/circuit.tsx"
import React from "react"

export default () => (
  <board width="30mm" height="20mm">
    <resistor name="R1" resistance="1k" footprint="0603" schX={-5} pcbX={-5} pcbY={0} />
    <led name="D1" color="green" footprint="0603" schX={5} pcbX={5} pcbY={0} />
    <trace from=".R1 > .pin2" to=".D1 > .anode" />
    <trace from=".R1 > .pin1" to=".D1 > .cathode" />
  </board>
)
EOF
    fusible check "$LOCAL_TMP/circuit.tsx"
fi

echo -e "${GREEN}✅ Smoke test passed! Fusible and tscircuit are ready to design.${NC}"
