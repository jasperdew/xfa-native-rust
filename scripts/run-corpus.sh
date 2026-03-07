#!/usr/bin/env bash
# Resilient corpus runner — auto-resumes after crashes (e.g. stack overflow).
set -uo pipefail

RUN_ID="${1:-baseline-001}"
CORPUS="/opt/xfa-corpus"
DB="/opt/xfa-results/db/results.sqlite"
WORKERS=8
TIMEOUT=30
MAX_RETRIES=20
LOG="/opt/xfa-results/logs/${RUN_ID}.log"

export PDFIUM_DYNAMIC_LIB_PATH=/opt/pdfium/lib
export LD_LIBRARY_PATH=/opt/pdfium/lib
source "$HOME/.cargo/env"

for attempt in $(seq 1 $MAX_RETRIES); do
    echo "[$(date -Iseconds)] Attempt $attempt/$MAX_RETRIES" >> "$LOG"

    /opt/xfa/target/release/xfa-test-runner run \
        --corpus "$CORPUS" \
        --db "$DB" \
        --workers "$WORKERS" \
        --timeout "$TIMEOUT" \
        --run-id "$RUN_ID" \
        --resume \
        >> "$LOG" 2>&1

    EXIT_CODE=$?

    if [ $EXIT_CODE -eq 0 ]; then
        echo "[$(date -Iseconds)] Run completed successfully." >> "$LOG"
        break
    fi

    echo "[$(date -Iseconds)] Runner exited with code $EXIT_CODE, resuming in 2s..." >> "$LOG"
    sleep 2
done

echo "[$(date -Iseconds)] Done after $attempt attempt(s)." >> "$LOG"
