#!/bin/bash
# orchestrator-cycle.sh — One full iteration cycle
# Usage: ./scripts/orchestrator-cycle.sh [iteration_number]
#
# Steps:
# 1. Sync code to VPS
# 2. Build on VPS
# 3. Clear oracle cache
# 4. Run pdfa_convert test
# 5. Wait for completion
# 6. Analyze results
# 7. Create tasks for workers
set -euo pipefail

ORCH_DIR="/tmp/xfa-orchestrator"
REPO="/Users/jasperdewinter/Documents/XFA"
VPS="root@46.225.223.175"
VPS_BUILD="/opt/xfa-build"
VPS_DB="/opt/xfa-results/curated-baseline.sqlite"
VPS_CORPUS="/opt/xfa-corpus/curated-1k"

# Determine iteration number
PREV_ITER=$(ls -1 "$ORCH_DIR/iterations/" 2>/dev/null | sort -V | tail -1 | sed 's/iter-//' || echo "-1")
PREV_ITER=$((PREV_ITER + 0))
ITER="${1:-$((PREV_ITER + 1))}"
ITER_DIR="$ORCH_DIR/iterations/iter-$(printf '%03d' $ITER)"
mkdir -p "$ITER_DIR"

echo "============================================"
echo "  XFA Orchestrator — Iteration $ITER"
echo "  $(date '+%Y-%m-%d %H:%M:%S')"
echo "============================================"
echo ""

# Step 1: Sync code
echo ">>> Step 1/7: Syncing code to VPS..."
rsync -az --exclude target --exclude .git "$REPO/" "$VPS:$VPS_BUILD/" 2>&1 | tail -3
echo "  Done"

# Step 2: Build
echo ">>> Step 2/7: Building on VPS..."
ssh "$VPS" "source ~/.cargo/env && cd $VPS_BUILD && cargo build --release -p xfa-test-runner 2>&1 | tail -5"
ssh "$VPS" "cp $VPS_BUILD/target/release/xfa-test-runner /usr/local/bin/"
echo "  Done"

# Step 3: Clear oracle cache
echo ">>> Step 3/7: Clearing veraPDF oracle cache..."
ssh "$VPS" "sqlite3 $VPS_DB \"DELETE FROM oracle_cache WHERE oracle_name='verapdf';\""
echo "  Done"

# Step 4: Start test run
echo ">>> Step 4/7: Starting pdfa_convert test (1000 PDFs, 10 workers)..."
ssh "$VPS" "cd /opt/xfa-results && nohup su -s /bin/bash xfa -c 'xfa-test-runner run --corpus $VPS_CORPUS --db curated-baseline.sqlite --tests pdfa_convert --workers 10 2>&1' > /tmp/curated-iter${ITER}.log 2>&1 &"
echo "  Started in background"

# Step 5: Wait for completion
echo ">>> Step 5/7: Waiting for test completion..."
while true; do
    TOTAL=$(ssh "$VPS" "sqlite3 $VPS_DB \"SELECT COUNT(*) FROM test_results WHERE test_name='pdfa_convert' AND run_id=(SELECT MAX(run_id) FROM test_results WHERE test_name='pdfa_convert')\"" 2>/dev/null || echo "0")
    if [ "$TOTAL" -ge 1000 ] 2>/dev/null; then
        echo "  Complete: $TOTAL PDFs processed"
        break
    fi
    echo "  Progress: $TOTAL / 1000..."
    sleep 30
done

# Step 6: Analyze results
echo ">>> Step 6/7: Analyzing results..."
bash "$REPO/scripts/orchestrator-analyze.sh" "$ITER"

# Step 7: Create tasks
echo ">>> Step 7/7: Creating tasks for workers..."
python3 "$REPO/scripts/orchestrator-create-tasks.py" "$ITER"

# Check for improvement
if [ "$PREV_ITER" -ge 0 ]; then
    PREV_META="$ORCH_DIR/iterations/iter-$(printf '%03d' $PREV_ITER)/meta.json"
    if [ -f "$PREV_META" ]; then
        PREV_PASS=$(python3 -c "import json; print(json.load(open('$PREV_META'))['pass'])")
        CURR_PASS=$(python3 -c "import json; print(json.load(open('$ITER_DIR/meta.json'))['pass'])")
        DELTA=$((CURR_PASS - PREV_PASS))
        if [ "$DELTA" -gt 0 ]; then
            echo ""
            echo "  Improvement: +$DELTA passes ($PREV_PASS → $CURR_PASS)"
        elif [ "$DELTA" -eq 0 ]; then
            echo ""
            echo "  WARNING: No improvement this iteration"
        else
            echo ""
            echo "  REGRESSION: $DELTA passes ($PREV_PASS → $CURR_PASS)"
        fi
    fi
fi

echo ""
echo "============================================"
echo "  Iteration $ITER complete"
echo "  Tasks are ready in $ORCH_DIR/tasks/"
echo "  Workers can now pick them up"
echo "============================================"
echo ""
echo "  Dashboard: http://localhost:8787"
echo "  Next cycle: ./scripts/orchestrator-cycle.sh $((ITER + 1))"
