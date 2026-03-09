#!/bin/bash
# orchestrator-loop.sh — Continuous orchestration loop
# Runs iterations until stop conditions are met.
# Usage: ./scripts/orchestrator-loop.sh
#
# Stop conditions:
# - Max 20 iterations
# - 3 consecutive iterations without improvement
# - All tasks marked done before next cycle
set -euo pipefail

ORCH_DIR="/tmp/xfa-orchestrator"
REPO="/Users/jasperdewinter/Documents/XFA"
MAX_ITER=20
MAX_NO_IMPROVE=3
no_improve_count=0
last_pass=0

echo "======================================================="
echo "  XFA Orchestrator — Continuous Loop"
echo "  Max iterations: $MAX_ITER"
echo "  Stop after $MAX_NO_IMPROVE rounds without improvement"
echo "  $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================================="
echo ""

# Start dashboard in background
python3 "$REPO/scripts/dashboard-server.py" &
DASHBOARD_PID=$!
echo "Dashboard started at http://localhost:8787 (PID: $DASHBOARD_PID)"
echo ""

cleanup() {
    kill $DASHBOARD_PID 2>/dev/null || true
    echo ""
    echo "Orchestrator stopped."
}
trap cleanup EXIT

for ITER in $(seq 1 $MAX_ITER); do
    echo ""
    echo ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>"
    echo "  Starting iteration $ITER of $MAX_ITER"
    echo ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>"

    # Wait for all tasks from previous iteration to be done
    if [ "$ITER" -gt 1 ]; then
        echo ""
        echo "  Waiting for workers to complete tasks..."
        while true; do
            open_tasks=$(find "$ORCH_DIR/tasks/" -name "*.json" -exec python3 -c "
import json,sys
t=json.load(open(sys.argv[1]))
if t['status'] in ('open','in_progress'): print('1')
" {} \; 2>/dev/null | wc -l | tr -d ' ')
            if [ "$open_tasks" -eq 0 ] 2>/dev/null; then
                echo "  All tasks complete!"
                break
            fi
            echo "  $open_tasks tasks still pending... (checking every 60s)"
            sleep 60
        done
    fi

    # Run the cycle
    bash "$REPO/scripts/orchestrator-cycle.sh" "$ITER"

    # Check improvement
    ITER_DIR="$ORCH_DIR/iterations/iter-$(printf '%03d' $ITER)"
    if [ -f "$ITER_DIR/meta.json" ]; then
        current_pass=$(python3 -c "import json; print(json.load(open('$ITER_DIR/meta.json'))['pass'])")
        if [ "$current_pass" -le "$last_pass" ] 2>/dev/null; then
            no_improve_count=$((no_improve_count + 1))
            echo ""
            echo "  No improvement ($no_improve_count/$MAX_NO_IMPROVE)"
            if [ "$no_improve_count" -ge "$MAX_NO_IMPROVE" ]; then
                echo "  STOP: $MAX_NO_IMPROVE consecutive iterations without improvement"
                break
            fi
        else
            no_improve_count=0
            echo ""
            echo "  Improvement detected! Reset no-improve counter."
        fi
        last_pass=$current_pass
    fi
done

echo ""
echo "======================================================="
echo "  Orchestrator loop finished"
echo "  Total iterations: $ITER"
echo "  Final pass count: $last_pass"
echo "======================================================="
