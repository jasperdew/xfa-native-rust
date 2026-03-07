#!/usr/bin/env bash
# Verwijder duplicaten op basis van SHA-256 — DRAAIT OP VPS
#
# Behoudt het eerste bestand per hash, verwijdert latere duplicaten.
# Werkt ook de metadata.sqlite bij als die bestaat.
#
# Usage:
#   ./scripts/corpus-dedup.sh [--target DIR] [--dry-run]

set -euo pipefail

CORPUS_DIR="/opt/xfa-corpus"
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)  CORPUS_DIR="$2"; shift 2 ;;
        --dry-run) DRY_RUN=true; shift ;;
        -h|--help)
            head -9 "$0" | grep '^#' | sed 's/^# \?//'
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "[$(date)] Deduplicating corpus in $CORPUS_DIR"
echo "  Dry-run: $DRY_RUN"
echo ""

# Detect hash command (sha256sum on Linux, shasum on macOS)
if command -v sha256sum &>/dev/null; then
    HASH_CMD="sha256sum"
elif command -v shasum &>/dev/null; then
    HASH_CMD="shasum -a 256"
else
    echo "ERROR: Neither sha256sum nor shasum found"
    exit 1
fi

DUPS=0
FREED=0

# Build hash list, sort by hash, detect duplicates
# Output format: "<hash>  <path>" — awk splits on first field
find "$CORPUS_DIR" -iname "*.pdf" -type f -print0 | \
    xargs -0 $HASH_CMD | \
    sort | \
    awk '{
        hash = $1
        # Reconstruct path (may contain spaces)
        path = ""
        for (i = 2; i <= NF; i++) {
            if (i > 2) path = path " "
            path = path $i
        }
        if (prev_hash == hash) {
            print path
        }
        prev_hash = hash
    }' | while IFS= read -r file; do
    file_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo 0)

    if $DRY_RUN; then
        echo "  [DRY-RUN] Would remove: $file ($((file_size / 1024))KB)"
    else
        rm -f "$file"
        echo "  Removed: $file ($((file_size / 1024))KB)"
    fi

    DUPS=$((DUPS + 1))
    FREED=$((FREED + file_size))
done

echo ""
echo "Duplicates found: $DUPS"
echo "Space freed: $((FREED / 1024 / 1024)) MB"

# Update metadata.sqlite if it exists
DB="$CORPUS_DIR/metadata.sqlite"
if [[ -f "$DB" ]] && ! $DRY_RUN; then
    echo ""
    echo "Cleaning stale entries from metadata.sqlite..."
    sqlite3 "$DB" "DELETE FROM pdfs WHERE NOT EXISTS (SELECT 1 FROM (SELECT path FROM pdfs) WHERE path = pdfs.path);" 2>/dev/null || true
    echo "  Database cleaned."
fi

echo ""
echo "[$(date)] Deduplication complete."
