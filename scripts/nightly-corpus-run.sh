#!/usr/bin/env bash
# Nightly corpus run with extended test matrix.
#
# Runs the full corpus against all test tiers, compares with the previous
# nightly run, and generates a dashboard. Designed to run on the VPS via
# systemd timer or cron.
#
# Usage:
#   ./scripts/nightly-corpus-run.sh [--corpus DIR] [--workers N] [--timeout N]
#
# Cron example (02:00 UTC daily):
#   0 2 * * * xfa /opt/xfa/scripts/nightly-corpus-run.sh >> /opt/xfa-results/logs/nightly.log 2>&1

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────

CORPUS_DIR="${1:-/opt/xfa-corpus}"
RESULTS_DIR="/opt/xfa-results"
DB_DIR="${RESULTS_DIR}/db"
DASHBOARD_DIR="${RESULTS_DIR}/dashboard"
LOG_DIR="${RESULTS_DIR}/logs"
RUNNER="/opt/xfa/target/release/xfa-test-runner"

WORKERS="${WORKERS:-auto}"
TIMEOUT="${TIMEOUT:-30}"
TODAY=$(date +%Y%m%d)
YESTERDAY=$(date -d "yesterday" +%Y%m%d 2>/dev/null || date -v-1d +%Y%m%d)

DB_TODAY="${DB_DIR}/nightly-${TODAY}.sqlite"
DB_YESTERDAY="${DB_DIR}/nightly-${YESTERDAY}.sqlite"

# ── Parse arguments ───────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case "$1" in
    --corpus) CORPUS_DIR="$2"; shift 2 ;;
    --workers) WORKERS="$2"; shift 2 ;;
    --timeout) TIMEOUT="$2"; shift 2 ;;
    *) shift ;;
  esac
done

# ── Helpers ───────────────────────────────────────────────────────────

log() {
  echo "$(date -Iseconds) [nightly] $*"
}

# ── Main ──────────────────────────────────────────────────────────────

log "Starting nightly corpus run"
log "  Corpus:  ${CORPUS_DIR}"
log "  DB:      ${DB_TODAY}"
log "  Workers: ${WORKERS}"
log "  Timeout: ${TIMEOUT}s"

# Ensure directories exist.
mkdir -p "${DB_DIR}" "${DASHBOARD_DIR}" "${LOG_DIR}"

# Pull latest code and rebuild if running on VPS.
if [[ -d /opt/xfa/.git ]]; then
  log "Pulling latest code..."
  cd /opt/xfa
  export PATH="${HOME}/.cargo/bin:${PATH}"
  git pull origin master --ff-only 2>&1 | tail -3
  log "Building release binary..."
  cargo build --release -p xfa-test-runner 2>&1 | tail -3
fi

# ── Step 1: Full corpus run (all tiers) ───────────────────────────────

log "Running full corpus test suite..."
"${RUNNER}" run \
  --corpus "${CORPUS_DIR}" \
  --db "${DB_TODAY}" \
  --workers "${WORKERS}" \
  --timeout "${TIMEOUT}" \
  --tier full \
  --no-verapdf

# ── Step 2: Generate report ──────────────────────────────────────────

log "Generating report..."
"${RUNNER}" report --db "${DB_TODAY}" || true

# ── Step 3: Regression check against yesterday ───────────────────────

REGRESSION_EXIT=0
if [[ -f "${DB_YESTERDAY}" ]]; then
  log "Comparing with previous nightly (${YESTERDAY})..."

  # Get latest run IDs from both databases.
  RUN_A=$(sqlite3 "${DB_YESTERDAY}" \
    "SELECT run_id FROM test_results ORDER BY rowid DESC LIMIT 1" 2>/dev/null || echo "")
  RUN_B=$(sqlite3 "${DB_TODAY}" \
    "SELECT run_id FROM test_results ORDER BY rowid DESC LIMIT 1" 2>/dev/null || echo "")

  if [[ -n "${RUN_A}" && -n "${RUN_B}" ]]; then
    # Count crashes in both runs.
    CRASHES_OLD=$(sqlite3 "${DB_YESTERDAY}" \
      "SELECT COUNT(*) FROM test_results WHERE run_id='${RUN_A}' AND status='crash'" 2>/dev/null || echo "0")
    CRASHES_NEW=$(sqlite3 "${DB_TODAY}" \
      "SELECT COUNT(*) FROM test_results WHERE run_id='${RUN_B}' AND status='crash'" 2>/dev/null || echo "0")

    # Count failures in both runs.
    FAILS_OLD=$(sqlite3 "${DB_YESTERDAY}" \
      "SELECT COUNT(*) FROM test_results WHERE run_id='${RUN_A}' AND status='fail'" 2>/dev/null || echo "0")
    FAILS_NEW=$(sqlite3 "${DB_TODAY}" \
      "SELECT COUNT(*) FROM test_results WHERE run_id='${RUN_B}' AND status='fail'" 2>/dev/null || echo "0")

    # Count passes in both runs.
    PASS_OLD=$(sqlite3 "${DB_YESTERDAY}" \
      "SELECT COUNT(*) FROM test_results WHERE run_id='${RUN_A}' AND status='pass'" 2>/dev/null || echo "0")
    PASS_NEW=$(sqlite3 "${DB_TODAY}" \
      "SELECT COUNT(*) FROM test_results WHERE run_id='${RUN_B}' AND status='pass'" 2>/dev/null || echo "0")

    log "Comparison:"
    log "  Passes:  ${PASS_OLD} → ${PASS_NEW}"
    log "  Fails:   ${FAILS_OLD} → ${FAILS_NEW}"
    log "  Crashes: ${CRASHES_OLD} → ${CRASHES_NEW}"

    # Regression if new crashes appear.
    if [[ "${CRASHES_NEW}" -gt "${CRASHES_OLD}" ]]; then
      log "REGRESSION: crash count increased (${CRASHES_OLD} → ${CRASHES_NEW})"
      REGRESSION_EXIT=1
    fi

    # Regression if failure count increases by more than 1%.
    if [[ "${FAILS_OLD}" -gt 0 ]]; then
      FAIL_INCREASE=$(( (FAILS_NEW - FAILS_OLD) * 100 / FAILS_OLD ))
      if [[ "${FAIL_INCREASE}" -gt 1 ]]; then
        log "REGRESSION: failure count increased by ${FAIL_INCREASE}% (${FAILS_OLD} → ${FAILS_NEW})"
        REGRESSION_EXIT=1
      fi
    elif [[ "${FAILS_NEW}" -gt 0 && "${FAILS_OLD}" -eq 0 ]]; then
      log "REGRESSION: new failures detected (0 → ${FAILS_NEW})"
      REGRESSION_EXIT=1
    fi
  else
    log "Could not determine run IDs for comparison, skipping"
  fi
else
  log "No previous nightly found (${DB_YESTERDAY}), skipping comparison"
fi

# ── Step 4: Generate dashboard ────────────────────────────────────────

log "Generating dashboard..."
"${RUNNER}" dashboard --db "${DB_TODAY}" --output "${DASHBOARD_DIR}" || true

# ── Step 5: Collect metrics ──────────────────────────────────────────

log "Collecting metrics..."
TOTAL=$(sqlite3 "${DB_TODAY}" \
  "SELECT COUNT(*) FROM test_results WHERE run_id=(SELECT run_id FROM test_results ORDER BY rowid DESC LIMIT 1)" 2>/dev/null || echo "0")
PASS=$(sqlite3 "${DB_TODAY}" \
  "SELECT COUNT(*) FROM test_results WHERE status='pass' AND run_id=(SELECT run_id FROM test_results ORDER BY rowid DESC LIMIT 1)" 2>/dev/null || echo "0")
CRASH=$(sqlite3 "${DB_TODAY}" \
  "SELECT COUNT(*) FROM test_results WHERE status='crash' AND run_id=(SELECT run_id FROM test_results ORDER BY rowid DESC LIMIT 1)" 2>/dev/null || echo "0")

if [[ "${TOTAL}" -gt 0 ]]; then
  PASS_RATE=$(echo "scale=1; ${PASS} * 100 / ${TOTAL}" | bc 2>/dev/null || echo "?")
  log "Summary: ${PASS}/${TOTAL} pass (${PASS_RATE}%), ${CRASH} crashes"
else
  log "Summary: no results"
fi

# ── Step 6: Cleanup old nightly databases (keep 14 days) ──────────────

find "${DB_DIR}" -name 'nightly-*.sqlite' -mtime +14 -delete 2>/dev/null || true

log "Nightly run complete (exit ${REGRESSION_EXIT})"
exit "${REGRESSION_EXIT}"
