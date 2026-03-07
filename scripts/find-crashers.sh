#!/usr/bin/env bash
# Find PDFs that crash the runner by repeatedly running with a single worker
# and recording which PDF was being processed when the crash happens.
set -uo pipefail

source "$HOME/.cargo/env"
export PDFIUM_DYNAMIC_LIB_PATH=/opt/pdfium/lib
export LD_LIBRARY_PATH=/opt/pdfium/lib

SKIP_FILE="/opt/xfa-corpus/skip.txt"
DB="/opt/xfa-results/db/results.sqlite"
MAX_TRIES=50

for i in $(seq 1 $MAX_TRIES); do
    echo "[$(date -Iseconds)] Crasher hunt attempt $i..."

    # Count results before
    BEFORE=$(sqlite3 "$DB" 'SELECT COUNT(DISTINCT pdf_path) FROM test_results WHERE run_id="baseline-001"')

    # Run with 1 worker so we know which PDF crashed
    timeout 120 /opt/xfa/target/release/xfa-test-runner run \
        --corpus /opt/xfa-corpus \
        --db "$DB" \
        --workers 1 \
        --timeout 15 \
        --run-id baseline-001 \
        --resume \
        --no-verapdf \
        2>&1 | tail -3

    EXIT=$?

    AFTER=$(sqlite3 "$DB" 'SELECT COUNT(DISTINCT pdf_path) FROM test_results WHERE run_id="baseline-001"')

    if [ "$EXIT" -eq 0 ]; then
        echo "Run completed successfully! $AFTER PDFs processed."
        break
    fi

    # If no progress was made, the next unprocessed PDF is a crasher
    if [ "$BEFORE" -eq "$AFTER" ]; then
        # Find the first unprocessed PDF
        CRASHER=$(comm -23 \
            <(find /opt/xfa-corpus -iname '*.pdf' | sort) \
            <(sqlite3 "$DB" 'SELECT DISTINCT pdf_path FROM test_results WHERE run_id="baseline-001"' | sort) \
            | grep -v -F -f <(grep -v '^#' "$SKIP_FILE" 2>/dev/null || true) \
            | head -1)

        if [ -n "$CRASHER" ]; then
            echo "  CRASHER FOUND: $CRASHER"
            echo "$CRASHER" >> "$SKIP_FILE"
        fi
    fi

    echo "  Progress: $BEFORE -> $AFTER PDFs"
done

echo ""
echo "Skip list now contains:"
grep -c -v '^#' "$SKIP_FILE" 2>/dev/null || echo "0"
echo "entries"
