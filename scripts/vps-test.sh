#!/usr/bin/env bash
# Run corpus tests on the VPS (Hetzner CX53 at 46.225.223.175).
#
# Usage:
#   scripts/vps-test.sh                 # full run on stressful corpus
#   scripts/vps-test.sh --tests ocr     # run only OCR tests
#   scripts/vps-test.sh --limit 100     # test first 100 PDFs
#   scripts/vps-test.sh --resume        # resume interrupted run
#
# Prerequisites on VPS:
#   - ONNX Runtime at /opt/onnxruntime/lib/
#   - PaddleOCR models at ~/.cache/xfa/ocr-models/
#   - veraPDF at /usr/local/bin/verapdf
#   - xfa-test-runner built: cargo build --release -p xfa-test-runner --features paddle-ocr

set -euo pipefail

VPS="root@46.225.223.175"
CORPUS="${CORPUS:-/opt/xfa-corpus/stressful}"
DB="${DB:-/opt/xfa-results/$(date +%Y%m%d-%H%M%S).sqlite}"

# Forward all extra arguments to xfa-test-runner run
EXTRA_ARGS="$@"

echo "=== VPS Test Runner ==="
echo "Corpus: $CORPUS"
echo "Database: $DB"
echo "Extra args: $EXTRA_ARGS"
echo ""

# Step 1: Pull latest code and rebuild
echo "→ Pulling latest code..."
ssh "$VPS" "cd /opt/xfa && git pull origin master 2>&1" | tail -3

echo "→ Building test runner..."
ssh "$VPS" "bash -l -c '
export ORT_DYLIB_PATH=/opt/onnxruntime/lib/libonnxruntime.so
cd /opt/xfa
cargo build --release -p xfa-test-runner --features paddle-ocr 2>&1 | tail -5
'"

# Step 2: Ensure results directory exists
ssh "$VPS" "mkdir -p /opt/xfa-results"

# Step 3: Run tests
echo "→ Running tests..."
ssh "$VPS" "bash -l -c '
export ORT_DYLIB_PATH=/opt/onnxruntime/lib/libonnxruntime.so
export LD_LIBRARY_PATH=/opt/onnxruntime/lib
cd /opt/xfa
./target/release/xfa-test-runner run \
  -c $CORPUS \
  -d $DB \
  --timeout 60 \
  $EXTRA_ARGS \
  2>&1
'"

# Step 4: Show summary
echo ""
echo "→ Clusters:"
ssh "$VPS" "bash -l -c '
cd /opt/xfa
./target/release/xfa-test-runner clusters -d $DB 2>&1
'" | head -30

echo ""
echo "Database saved: $DB"
