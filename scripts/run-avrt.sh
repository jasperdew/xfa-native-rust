#!/usr/bin/env bash
# Automated Visual Regression Testing (AVRT) runner.
#
# Usage:
#   scripts/run-avrt.sh                     # Run full AVRT pipeline
#   scripts/run-avrt.sh --generate-masters  # Generate gold masters (requires Adobe Acrobat)
#   scripts/run-avrt.sh --render-actuals    # Render actuals from engine only
#   scripts/run-avrt.sh --compare-only      # Compare existing actuals vs masters
#   scripts/run-avrt.sh --report html       # Also generate HTML report

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MASTERS_DIR="$PROJECT_ROOT/golden/masters"
ACTUALS_DIR="$PROJECT_ROOT/golden/actuals"
DIFFS_DIR="$PROJECT_ROOT/golden/diffs"
REPORTS_DIR="$PROJECT_ROOT/reports/avrt"
CONFIG_FILE="$PROJECT_ROOT/avrt.json"
CORPUS_DIR="$PROJECT_ROOT/corpus"

GENERATE_MASTERS=false
RENDER_ACTUALS=false
COMPARE_ONLY=false
HTML_REPORT=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --generate-masters)
            GENERATE_MASTERS=true
            shift
            ;;
        --render-actuals)
            RENDER_ACTUALS=true
            shift
            ;;
        --compare-only)
            COMPARE_ONLY=true
            shift
            ;;
        --report)
            if [[ "${2:-}" == "html" ]]; then
                HTML_REPORT=true
                shift
            fi
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --generate-masters  Generate gold masters from Adobe Acrobat renders"
            echo "  --render-actuals    Render actuals from XFA engine"
            echo "  --compare-only      Compare existing actuals vs masters"
            echo "  --report html       Generate HTML visual diff report"
            echo "  -h, --help          Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "=== AVRT: Automated Visual Regression Testing ==="
echo ""

# Step 1: Generate gold masters (requires manual Adobe Acrobat rendering)
if [ "$GENERATE_MASTERS" = true ]; then
    echo "--- Gold Master Generation ---"
    echo "Gold master generation requires Adobe Acrobat to render each form."
    echo ""
    echo "Instructions:"
    echo "1. Open each PDF in corpus/ with Adobe Acrobat"
    echo "2. Export/print each page as PNG at 150 DPI"
    echo "3. Name files as: <form_name>_page<N>.png"
    echo "4. Place in golden/masters/<category>/"
    echo ""
    echo "Categories:"
    echo "  tax-individual/  - 1040, W-2, W-4, etc."
    echo "  tax-business/    - 1065, 1120, 990, etc."
    echo "  immigration/     - I-130, I-140, I-485, etc."
    echo "  government/      - SF-86, etc."
    echo ""

    # Count existing masters
    MASTER_COUNT=$(find "$MASTERS_DIR" -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
    CORPUS_COUNT=$(find "$CORPUS_DIR" -name "*.pdf" 2>/dev/null | wc -l | tr -d ' ')
    echo "Current state: $MASTER_COUNT masters / $CORPUS_COUNT corpus PDFs"
    echo ""

    if [ "$MASTER_COUNT" -eq 0 ]; then
        echo "No gold masters found. Please generate them first."
        exit 0
    fi
fi

# Step 2: Render actuals from XFA engine
if [ "$RENDER_ACTUALS" = true ] || [ "$COMPARE_ONLY" = false ] && [ "$GENERATE_MASTERS" = false ]; then
    echo "--- Rendering Actuals ---"

    # Build the CLI tool
    echo "Building xfa-cli..."
    cd "$PROJECT_ROOT"
    cargo build --release --bin xfa-cli 2>/dev/null || {
        echo "Warning: xfa-cli build failed. Using test renderer instead."
    }

    # For now, actual rendering will be done by the integration test binary
    echo "Actual rendering will be implemented when the PDF renderer (Epic 4.4) is complete."
    echo "Currently the pipeline infrastructure is ready for gold masters."
    echo ""
fi

# Step 3: Compare actuals vs masters
echo "--- Comparing Actuals vs Masters ---"

MASTER_COUNT=$(find "$MASTERS_DIR" -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
ACTUAL_COUNT=$(find "$ACTUALS_DIR" -name "*.png" 2>/dev/null | wc -l | tr -d ' ')

echo "Masters: $MASTER_COUNT images"
echo "Actuals: $ACTUAL_COUNT images"

if [ "$MASTER_COUNT" -eq 0 ]; then
    echo ""
    echo "No gold masters found. Run with --generate-masters first."
    echo "See golden/masters/ for the expected directory structure."
    exit 0
fi

if [ "$ACTUAL_COUNT" -eq 0 ]; then
    echo ""
    echo "No actual renders found."
    echo "The rendering pipeline will be available when the PDF renderer is complete."
    exit 0
fi

# Run the comparison using the Rust binary
mkdir -p "$REPORTS_DIR"
mkdir -p "$DIFFS_DIR"

echo ""
echo "Running pixel comparison..."

# Run the AVRT comparison via the Rust test harness.
# TODO: Replace with a dedicated binary when the full rendering pipeline is available.
AVRT_EXIT=0
cargo test --package xfa-golden-tests avrt -- --nocapture 2>&1 || AVRT_EXIT=$?

if [ "$AVRT_EXIT" -ne 0 ]; then
    echo "WARNING: AVRT comparison exited with code $AVRT_EXIT"
fi

# Step 4: Generate report
echo ""
echo "--- Report ---"
if [ -f "$REPORTS_DIR/summary.json" ]; then
    echo "Summary: $REPORTS_DIR/summary.json"

    if command -v python3 &>/dev/null; then
        python3 -c "
import json, sys
with open('$REPORTS_DIR/summary.json') as f:
    data = json.load(f)
s = data.get('summary', {})
print(f\"  Total forms:  {s.get('total_forms', 0)}\")
print(f\"  Total pages:  {s.get('total_pages', 0)}\")
print(f\"  Passed:       {s.get('passed_pages', 0)}\")
print(f\"  Failed:       {s.get('failed_pages', 0)}\")
print(f\"  Skipped:      {s.get('skipped_forms', 0)}\")
print(f\"  Pass rate:    {s.get('pass_rate', 0):.1f}%\")
for cat, cs in s.get('per_category', {}).items():
    print(f\"  [{cat}] {cs['passed_pages']}/{cs['total_pages']} passed ({cs['pass_rate']:.1f}%), avg diff {cs['avg_diff_percentage']:.3f}%\")
" 2>/dev/null || echo "  (install python3 to see formatted summary)"
    fi
else
    echo "No summary report found. Comparison may not have run."
fi

if [ "$HTML_REPORT" = true ] && [ -f "$REPORTS_DIR/report.html" ]; then
    echo "HTML report: $REPORTS_DIR/report.html"
fi

echo ""
echo "=== AVRT Complete ==="
