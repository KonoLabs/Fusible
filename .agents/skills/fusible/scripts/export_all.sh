#!/usr/bin/env bash
# ==============================================================================
# export_all.sh — Fuse-Gated Fabrication, CAD & Documentation Exporter
# ==============================================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}📦 Fusible Deliverables Exporter${NC}"

# Check for fusible CLI
if ! command -v fusible &>/dev/null; then
    echo -e "${RED}✗ Error: 'fusible' binary not found in PATH.${NC}"
    exit 2
fi

OUT_DIR="dist"
if [[ -f "fusible.yml" ]]; then
    CUSTOM_OUT=$(grep -E '^\s*out_dir:' fusible.yml | awk '{print $2}' || true)
    if [[ -n "$CUSTOM_OUT" ]]; then
        OUT_DIR="$CUSTOM_OUT"
    fi
fi

mkdir -p "$OUT_DIR"

echo -e "Target output directory: ${YELLOW}$OUT_DIR${NC}"
echo -e "${BLUE}1/3 Exporting Fabrication deliverables (Gerbers + Circuit JSON)...${NC}"
fusible export --preset fab

echo -e "${BLUE}2/3 Exporting CAD deliverables (KiCad project + 3D STEP)...${NC}"
fusible export --preset cad

echo -e "${BLUE}3/3 Exporting Documentation deliverables (SVGs + PDF)...${NC}"
fusible export --preset docs

echo ""
echo -e "${GREEN}✅ All exports generated successfully!${NC}"
echo "Generated artifacts in $OUT_DIR:"
ls -lh "$OUT_DIR"
