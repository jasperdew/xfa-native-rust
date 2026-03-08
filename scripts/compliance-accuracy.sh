#!/usr/bin/env bash
# Run our PDF/A compliance checker against the ground truth test suites.
#
# Reads ground-truth.tsv, runs xfa-cli validate on each PDF,
# compares our result against expected pass/fail, reports accuracy.
#
# Usage:
#   ./scripts/compliance-accuracy.sh [--jobs N] [--suite SUITE] [--clause CLAUSE]

set -euo pipefail

BASE="/opt/xfa-corpus/compliance-suites"
TSV="$BASE/ground-truth.tsv"
RESULTS="$BASE/accuracy-results.tsv"
JOBS=8
FILTER_SUITE=""
FILTER_CLAUSE=""
CLI_BIN=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --jobs) JOBS="$2"; shift 2 ;;
        --suite) FILTER_SUITE="$2"; shift 2 ;;
        --clause) FILTER_CLAUSE="$2"; shift 2 ;;
        --bin) CLI_BIN="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

if [[ -z "$CLI_BIN" ]]; then
    if [[ -f "/opt/xfa/target/release/xfa-cli" ]]; then
        CLI_BIN="/opt/xfa/target/release/xfa-cli"
    elif command -v xfa-cli &>/dev/null; then
        CLI_BIN="$(command -v xfa-cli)"
    else
        echo "ERROR: xfa-cli not found. Build: cargo build --release -p xfa-cli"
        exit 1
    fi
fi

echo "Binary:  $CLI_BIN"
echo "TSV:     $TSV"
echo "Jobs:    $JOBS"
echo "Filter:  suite=${FILTER_SUITE:-all} clause=${FILTER_CLAUSE:-all}"
echo ""

# Write results header
printf 'path\tsuite\tprofile\tclause\texpected\tactual\tmatch\n' > "$RESULTS"

# Build filtered work list
WORK=$(mktemp)
skipped=0
while IFS=$'\t' read -r pdf suite profile clause expected; do
    [[ "$expected" == "unknown" ]] && { skipped=$((skipped + 1)); continue; }
    # Skip unsupported profiles
    case "$profile" in 4|4e|4f|unknown) skipped=$((skipped + 1)); continue ;; esac
    [[ -n "$FILTER_SUITE" && "$suite" != "$FILTER_SUITE" ]] && continue
    [[ -n "$FILTER_CLAUSE" && "$clause" != "$FILTER_CLAUSE"* ]] && continue
    printf '%s\t%s\t%s\t%s\t%s\n' "$pdf" "$suite" "$profile" "$clause" "$expected" >> "$WORK"
done < <(tail -n+2 "$TSV")

work_count=$(wc -l < "$WORK" | tr -d ' ')
echo "Testing $work_count files ($skipped skipped)..."

# Worker function: check one PDF
check_one() {
    local line="$1"
    IFS=$'\t' read -r pdf suite profile clause expected <<< "$line"

    local cli_profile="pdf-a${profile}"
    local exit_code=0
    timeout 10 "$CLI_BIN" validate "$pdf" -P "$cli_profile" > /dev/null 2>&1 || exit_code=$?

    local actual
    case $exit_code in
        0)   actual="pass" ;;
        1)   actual="fail" ;;
        124) actual="timeout" ;;
        *)   actual="error" ;;
    esac

    local match="false"
    [[ "$actual" == "$expected" ]] && match="true"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$pdf" "$suite" "$profile" "$clause" "$expected" "$actual" "$match"
}
export -f check_one
export CLI_BIN

# Run in parallel
cat "$WORK" | xargs -P "$JOBS" -d '\n' -I {} bash -c 'check_one "$@"' _ {} >> "$RESULTS"
rm -f "$WORK"

# Stats
total=$(tail -n+2 "$RESULTS" | wc -l | tr -d ' ')
correct=$(tail -n+2 "$RESULTS" | cut -f7 | grep -c "true" || true)
fp=$(tail -n+2 "$RESULTS" | awk -F'\t' '$5=="fail" && $6=="pass"' | wc -l | tr -d ' ')
fn=$(tail -n+2 "$RESULTS" | awk -F'\t' '$5=="pass" && $6=="fail"' | wc -l | tr -d ' ')
errs=$(tail -n+2 "$RESULTS" | awk -F'\t' '$6=="error" || $6=="timeout"' | wc -l | tr -d ' ')

if [[ $total -gt 0 ]]; then
    accuracy=$(echo "scale=1; $correct * 100 / $total" | bc)
else
    accuracy="0"
fi

echo ""
echo "=== Compliance Accuracy Report ==="
echo "Total tested:   $total (skipped: $skipped)"
echo "Correct:        $correct / $total ($accuracy%)"
echo "False positives (we pass, should fail): $fp"
echo "False negatives (we fail, should pass): $fn"
echo "Errors/timeouts: $errs"
echo ""

echo "=== Top Mismatched Clauses ==="
tail -n+2 "$RESULTS" | awk -F'\t' '$7=="false"' | cut -f4 | sort | uniq -c | sort -rn | head -15

echo ""
echo "=== Per-Suite Accuracy ==="
for s in verapdf isartor bfo; do
    stotal=$(tail -n+2 "$RESULTS" | awk -F'\t' -v s="$s" '$2==s' | wc -l | tr -d ' ')
    scorr=$(tail -n+2 "$RESULTS" | awk -F'\t' -v s="$s" '$2==s && $7=="true"' | wc -l | tr -d ' ')
    if [[ $stotal -gt 0 ]]; then
        sacc=$(echo "scale=1; $scorr * 100 / $stotal" | bc)
        echo "  $s: $scorr / $stotal ($sacc%)"
    fi
done

echo ""
echo "Results: $RESULTS"
