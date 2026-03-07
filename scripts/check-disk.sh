#!/usr/bin/env bash
# Disk space monitor — alerts when corpus volume exceeds 85%.
# Add to cron: */30 * * * * /opt/xfa/scripts/check-disk.sh
set -euo pipefail

THRESHOLD=85
USAGE=$(df /opt/xfa-corpus --output=pcent | tail -1 | tr -d ' %')

if [ "$USAGE" -gt "$THRESHOLD" ]; then
    echo "$(date -Iseconds) DISK WARNING: ${USAGE}% used on /opt/xfa-corpus"
    # Uncomment and configure for Slack/webhook alerts:
    # curl -s -X POST -H 'Content-type: application/json' \
    #   --data "{\"text\":\"DISK WARNING: ${USAGE}% used on XFA corpus volume\"}" \
    #   "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
fi
