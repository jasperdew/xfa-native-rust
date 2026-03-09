#!/bin/bash
# orchestrator-analyze.sh — Analyze VPS test results and create error clusters
# Usage: ./scripts/orchestrator-analyze.sh [iteration_number]
set -euo pipefail

ORCH_DIR="/tmp/xfa-orchestrator"
CONFIG="$ORCH_DIR/config.json"
VPS_HOST="root@46.225.223.175"
VPS_DB="/opt/xfa-results/curated-baseline.sqlite"

ITER="${1:-$(ls -1 "$ORCH_DIR/iterations/" 2>/dev/null | sort -V | tail -1 | sed 's/iter-//' || echo 0)}"
ITER=$((ITER))
ITER_DIR="$ORCH_DIR/iterations/iter-$(printf '%03d' $ITER)"
mkdir -p "$ITER_DIR"

echo "=== Iteration $ITER — Analyzing VPS results ==="

# Get latest run results
LATEST_RUN=$(ssh "$VPS_HOST" "sqlite3 $VPS_DB \"SELECT MAX(run_id) FROM test_results WHERE test_name='pdfa_convert'\"")
echo "Latest run: $LATEST_RUN"

# Get summary
ssh "$VPS_HOST" "sqlite3 -json $VPS_DB \"
SELECT
  COUNT(*) as total,
  SUM(CASE WHEN status='pass' THEN 1 ELSE 0 END) as pass,
  SUM(CASE WHEN status='fail' THEN 1 ELSE 0 END) as fail,
  SUM(CASE WHEN status='skip' THEN 1 ELSE 0 END) as skip
FROM test_results
WHERE test_name='pdfa_convert' AND run_id='$LATEST_RUN'
\"" > "$ITER_DIR/summary.json"

cat "$ITER_DIR/summary.json" | python3 -c "
import json,sys
d=json.load(sys.stdin)[0]
print(f\"  Total: {d['total']}  Pass: {d['pass']}  Fail: {d['fail']}  Skip: {d['skip']}\")
print(f\"  Pass rate: {100*d['pass']/d['total']:.1f}%\")
"

# Get per-PDF error profiles from veraPDF oracle cache
ssh "$VPS_HOST" "sqlite3 -json $VPS_DB \"
SELECT
  profile,
  COUNT(*) as pdf_count,
  GROUP_CONCAT(pdf_hash, ',') as hashes
FROM (
  SELECT
    oc.pdf_hash,
    GROUP_CONCAT(
      json_extract(value, '\\\$.clause') || ':' || json_extract(value, '\\\$.test_number'),
      '|'
    ) as profile
  FROM oracle_cache oc, json_each(oc.result_json, '\\\$.rule_failures')
  WHERE oc.oracle_name='verapdf'
    AND json_extract(oc.result_json, '\\\$.is_compliant')=0
  GROUP BY oc.pdf_hash
)
GROUP BY profile
ORDER BY pdf_count DESC
\"" > "$ITER_DIR/clusters.json" 2>/dev/null || echo "[]" > "$ITER_DIR/clusters.json"

# Show top clusters
echo ""
echo "=== Error Clusters ==="
python3 -c "
import json
clusters = json.load(open('$ITER_DIR/clusters.json'))
total_failing = sum(c['pdf_count'] for c in clusters)
print(f'Total failing PDFs in oracle: {total_failing}')
print(f'Unique error profiles: {len(clusters)}')
print()
for i, c in enumerate(clusters[:15]):
    rules = c['profile'].split('|')
    print(f'  Cluster {i+1}: {c[\"pdf_count\"]} PDFs — {c[\"profile\"]}')
"

# Get veraPDF rule descriptions
ssh "$VPS_HOST" "sqlite3 -json $VPS_DB \"
SELECT
  json_extract(value, '\\\$.clause') as clause,
  json_extract(value, '\\\$.test_number') as test_num,
  json_extract(value, '\\\$.description') as description,
  COUNT(*) as cnt
FROM oracle_cache oc, json_each(oc.result_json, '\\\$.rule_failures')
WHERE oc.oracle_name='verapdf' AND json_extract(oc.result_json, '\\\$.is_compliant')=0
GROUP BY clause, test_num
ORDER BY cnt DESC
\"" > "$ITER_DIR/rules.json" 2>/dev/null || echo "[]" > "$ITER_DIR/rules.json"

echo ""
echo "=== Top Rules ==="
python3 -c "
import json
rules = json.load(open('$ITER_DIR/rules.json'))
for r in rules[:10]:
    print(f\"  {r['clause']}:{r['test_num']} ({r['cnt']}x) — {r['description'][:80]}\")
"

# Save iteration metadata
python3 -c "
import json, datetime
summary = json.load(open('$ITER_DIR/summary.json'))[0]
clusters = json.load(open('$ITER_DIR/clusters.json'))
meta = {
    'iteration': $ITER,
    'run_id': '$LATEST_RUN',
    'timestamp': datetime.datetime.now().isoformat(),
    'pass': summary['pass'],
    'fail': summary['fail'],
    'skip': summary['skip'],
    'total': summary['total'],
    'cluster_count': len(clusters),
    'top_cluster': clusters[0]['profile'] if clusters else 'none',
    'top_cluster_count': clusters[0]['pdf_count'] if clusters else 0,
}
json.dump(meta, open('$ITER_DIR/meta.json', 'w'), indent=2)
print()
print('Iteration metadata saved to $ITER_DIR/meta.json')
"

echo ""
echo "=== Done ==="
