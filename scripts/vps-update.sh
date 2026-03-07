#!/usr/bin/env bash
# Auto-update script: pull latest code, rebuild, restart runner.
# Intended for cron: 0 5 * * * /opt/xfa/scripts/vps-update.sh >> /opt/xfa-results/logs/update.log 2>&1
set -euo pipefail

echo "$(date -Iseconds) Starting update..."

cd /opt/xfa
git pull origin master

source "$HOME/.cargo/env"
cargo build --release 2>&1 | tail -3

sudo systemctl restart xfa-test-runner

echo "$(date -Iseconds) Update complete."
