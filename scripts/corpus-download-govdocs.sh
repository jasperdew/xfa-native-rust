#!/usr/bin/env bash
# Download GovDocs1 PDF subset — DRAAIT OP VPS
# digitalcorpora.org/corpora/govdocs/
# 1000 zip files, elk ~300MB gemengde bestanden
# PDFs zijn ~23% van totaal -> ~231K PDFs
# Wij nemen een steekproef -> ~5000 PDFs
#
# Usage:
#   ./scripts/corpus-download-govdocs.sh [--target DIR] [--count N] [--dry-run]

set -euo pipefail

CORPUS_DIR="/opt/xfa-corpus/general/govdocs"
TARGET_COUNT=5000
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)  CORPUS_DIR="$2"; shift 2 ;;
        --count)   TARGET_COUNT="$2"; shift 2 ;;
        --dry-run) DRY_RUN=true; shift ;;
        -h|--help)
            head -9 "$0" | grep '^#' | sed 's/^# \?//'
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

TEMP_DIR="/tmp/govdocs-download"
CURRENT=0

echo "[$(date)] Starting GovDocs1 download"
echo "  Target:    $CORPUS_DIR"
echo "  Count:     $TARGET_COUNT PDFs"
echo "  Dry-run:   $DRY_RUN"
echo ""

mkdir -p "$CORPUS_DIR" "$TEMP_DIR"

for i in $(seq -w 000 999); do
    [[ "$CURRENT" -ge "$TARGET_COUNT" ]] && break

    ZIP_URL="https://digitalcorpora.org/downloads/govdocs1/${i}.zip"

    if $DRY_RUN; then
        echo "[DRY-RUN] Would download: $ZIP_URL"
        # Estimate ~50 PDFs per zip (23% of ~220 files)
        CURRENT=$((CURRENT + 50))
        continue
    fi

    echo "[$(date)] Downloading govdocs zip $i..."
    wget -q --timeout=60 --tries=3 "$ZIP_URL" \
        -O "$TEMP_DIR/${i}.zip" 2>/dev/null || {
        echo "  SKIP: zip $i (download failed)"
        continue
    }

    # Extract only PDFs
    mkdir -p "$TEMP_DIR/extracted"
    unzip -j -o -q "$TEMP_DIR/${i}.zip" "*.pdf" "*.PDF" \
        -d "$TEMP_DIR/extracted/" 2>/dev/null || true

    # Move extracted PDFs to corpus
    for pdf in "$TEMP_DIR/extracted/"*.pdf "$TEMP_DIR/extracted/"*.PDF; do
        [[ -f "$pdf" ]] || continue
        # Prefix with zip number to avoid name collisions
        local_name="${i}_$(basename "$pdf")"
        mv "$pdf" "$CORPUS_DIR/$local_name"
        CURRENT=$((CURRENT + 1))
        [[ "$CURRENT" -ge "$TARGET_COUNT" ]] && break
    done

    rm -rf "$TEMP_DIR/extracted/" "$TEMP_DIR/${i}.zip"
    echo "  Progress: $CURRENT / $TARGET_COUNT PDFs"
done

rm -rf "$TEMP_DIR"
echo ""
echo "[$(date)] Done: $CURRENT PDFs in $CORPUS_DIR"
