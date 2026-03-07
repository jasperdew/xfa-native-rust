#!/usr/bin/env bash
# Toon corpus statistieken vanuit metadata.sqlite — DRAAIT OP VPS
#
# Usage:
#   ./scripts/corpus-stats.sh [DATABASE_PATH]
#
# Default: /opt/xfa-corpus/metadata.sqlite

set -euo pipefail

DB="${1:-/opt/xfa-corpus/metadata.sqlite}"

if [[ ! -f "$DB" ]]; then
    echo "ERROR: Database not found: $DB"
    echo "Run corpus-categorize.py first to create it."
    exit 1
fi

if ! command -v sqlite3 &>/dev/null; then
    echo "ERROR: sqlite3 not found. Install with: apt install sqlite3"
    exit 1
fi

echo "=== Corpus Summary ==="
echo "Database: $DB"
echo ""
echo "Total PDFs: $(sqlite3 "$DB" "SELECT COUNT(*) FROM pdfs")"
echo "Total Size: $(sqlite3 "$DB" "SELECT printf('%.1f GB', SUM(size)/1073741824.0) FROM pdfs")"
echo ""

echo "=== By Category ==="
sqlite3 -column -header "$DB" \
    "SELECT category,
            COUNT(*) as count,
            printf('%.1f%%', 100.0*COUNT(*)/(SELECT COUNT(*) FROM pdfs)) as pct,
            printf('%.1f MB', SUM(size)/1048576.0) as total_size
     FROM pdfs GROUP BY category ORDER BY count DESC"
echo ""

echo "=== By Source ==="
sqlite3 -column -header "$DB" \
    "SELECT source,
            COUNT(*) as count,
            printf('%.1f%%', 100.0*COUNT(*)/(SELECT COUNT(*) FROM pdfs)) as pct
     FROM pdfs GROUP BY source ORDER BY count DESC"
echo ""

echo "=== By PDF Version ==="
sqlite3 -column -header "$DB" \
    "SELECT COALESCE(pdf_version, 'unknown') as version,
            COUNT(*) as count
     FROM pdfs GROUP BY pdf_version ORDER BY count DESC LIMIT 10"
echo ""

echo "=== Features ==="
sqlite3 -column -header "$DB" \
    "SELECT
        SUM(has_forms) as forms,
        SUM(has_xfa) as xfa,
        SUM(has_signatures) as signed,
        SUM(has_annotations) as annotated,
        SUM(has_encryption) as encrypted,
        SUM(claims_pdfa) as pdfa,
        SUM(claims_pdfua) as pdfua
     FROM pdfs"
echo ""

echo "=== File Size Distribution ==="
sqlite3 -column -header "$DB" \
    "SELECT file_size_category as size_cat, COUNT(*) as count
     FROM pdfs WHERE file_size_category IS NOT NULL
     GROUP BY file_size_category
     ORDER BY CASE file_size_category
        WHEN 'tiny' THEN 1
        WHEN 'small' THEN 2
        WHEN 'medium' THEN 3
        WHEN 'large' THEN 4
        WHEN 'huge' THEN 5
     END"
echo ""

echo "=== Top 10 Producers ==="
sqlite3 -column -header "$DB" \
    "SELECT COALESCE(producer, 'unknown') as producer,
            COUNT(*) as count
     FROM pdfs GROUP BY producer ORDER BY count DESC LIMIT 10"
