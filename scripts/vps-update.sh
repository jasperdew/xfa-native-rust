#!/usr/bin/env bash
# Auto-update script: pull latest code, rebuild, restart runner.
# Runs as the xfa user via cron:
#   0 5 * * * xfa /opt/xfa/scripts/vps-update.sh >> /opt/xfa-results/logs/update.log 2>&1
# Requires: sudoers entry for xfa to restart xfa-test-runner (see vps-setup.sh)
set -euo pipefail

echo "$(date -Iseconds) Starting update..."

cd /opt/xfa
git pull origin master

# Cargo is installed per-user for xfa; use explicit path for cron compatibility.
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release 2>&1 | tail -3

sudo /usr/bin/systemctl restart xfa-test-runner

echo "$(date -Iseconds) Update complete."
