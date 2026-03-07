#!/usr/bin/env bash
# VPS provisioning script for XFA Test Runner on Hetzner CX53 (Ubuntu 24.04)
# Usage: ssh root@<ip> 'bash -s' < scripts/vps-setup.sh
set -euo pipefail

echo "=== XFA Test Runner VPS Setup ==="

# 1. System updates & base tooling
echo "[1/7] Installing system packages..."
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get upgrade -y -qq
apt-get install -y -qq \
    build-essential pkg-config libssl-dev \
    git curl wget htop tmux \
    poppler-utils \
    default-jre \
    imagemagick \
    sqlite3 \
    logrotate \
    unzip

# 2. Create xfa service user
echo "[2/7] Creating xfa user..."
if ! id -u xfa &>/dev/null; then
    useradd -m -s /bin/bash xfa
fi

# 3. Rust toolchain (as xfa user)
echo "[3/7] Installing Rust toolchain..."
su - xfa -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
su - xfa -c 'source $HOME/.cargo/env && rustup default stable'

# 4. PDFium prebuilt binary
echo "[4/7] Installing PDFium..."
mkdir -p /opt/pdfium
if [ ! -f /opt/pdfium/lib/libpdfium.so ]; then
    PDFIUM_TAG=$(curl -sL "https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest" | \
        python3 -c "import sys,json; print(json.load(sys.stdin)['tag_name'])")
    wget -q "https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_TAG}/pdfium-linux-x64.tgz" -O /tmp/pdfium.tgz
    tar xzf /tmp/pdfium.tgz -C /opt/pdfium
    rm /tmp/pdfium.tgz
fi
# Environment for all users
cat > /etc/profile.d/pdfium.sh << 'ENVEOF'
export PDFIUM_DYNAMIC_LIB_PATH=/opt/pdfium/lib
export LD_LIBRARY_PATH=/opt/pdfium/lib:${LD_LIBRARY_PATH:-}
ENVEOF
chmod +x /etc/profile.d/pdfium.sh

# 5. veraPDF (silent izpack install)
echo "[5/7] Installing veraPDF..."
if [ ! -f /usr/local/bin/verapdf ]; then
    cd /tmp
    wget -q "https://software.verapdf.org/rel/verapdf-installer.zip" -O verapdf.zip
    unzip -qo verapdf.zip
    INSTALLER_JAR=$(ls /tmp/verapdf-greenfield-*/verapdf-izpack-installer-*.jar)
    cat > /tmp/verapdf-auto-install.xml << 'AUTOXML'
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<AutomatedInstallation langpack="eng">
    <com.izforge.izpack.panels.htmlhello.HTMLHelloPanel id="welcome"/>
    <com.izforge.izpack.panels.target.TargetPanel id="install_dir">
        <installpath>/opt/verapdf</installpath>
    </com.izforge.izpack.panels.target.TargetPanel>
    <com.izforge.izpack.panels.packs.PacksPanel id="sdk_pack_select">
        <pack index="0" name="veraPDF GUI" selected="true"/>
        <pack index="1" name="veraPDF Mac and *nix Scripts" selected="true"/>
        <pack index="2" name="veraPDF Validation model" selected="true"/>
        <pack index="3" name="veraPDF Documentation" selected="false"/>
        <pack index="4" name="veraPDF Sample Plugins" selected="false"/>
    </com.izforge.izpack.panels.packs.PacksPanel>
    <com.izforge.izpack.panels.install.InstallPanel id="install"/>
    <com.izforge.izpack.panels.finish.FinishPanel id="finish"/>
</AutomatedInstallation>
AUTOXML
    rm -rf /opt/verapdf
    java -jar "$INSTALLER_JAR" /tmp/verapdf-auto-install.xml
    ln -sf /opt/verapdf/verapdf /usr/local/bin/verapdf
    chmod +x /opt/verapdf/verapdf
    rm -rf /tmp/verapdf*
fi

# 6. Repository clone & build
echo "[6/7] Cloning repository and building..."
if [ ! -d /opt/xfa ]; then
    git clone https://github.com/jasperdew/xfa-native-rust.git /opt/xfa
    chown -R xfa:xfa /opt/xfa
fi
su - xfa -c 'source $HOME/.cargo/env && cd /opt/xfa && cargo build --release 2>&1 | tail -3'

# 7. Directory structure
echo "[7/7] Creating directory structure..."
mkdir -p /opt/xfa-corpus/{general,forms,signed,invoices,scanned,tagged,encrypted,malformed,large}
mkdir -p /opt/xfa-results/{db,diffs,reports,logs}
chown -R xfa:xfa /opt/xfa-corpus /opt/xfa-results

# Install systemd service, timer, and logrotate
cp /opt/xfa/scripts/systemd/xfa-test-runner.service /etc/systemd/system/
cp /opt/xfa/scripts/systemd/xfa-test-runner.timer /etc/systemd/system/
cp /opt/xfa/scripts/logrotate/xfa-test-runner /etc/logrotate.d/

systemctl daemon-reload
systemctl enable xfa-test-runner.timer
systemctl start xfa-test-runner.timer

# Cron jobs for auto-update and disk monitoring
cat > /etc/cron.d/xfa << 'CRONEOF'
# Auto-update: pull master and rebuild daily at 05:00
0 5 * * * xfa /opt/xfa/scripts/vps-update.sh >> /opt/xfa-results/logs/update.log 2>&1
# Disk monitoring: every 30 minutes
*/30 * * * * xfa /opt/xfa/scripts/check-disk.sh >> /opt/xfa-results/logs/disk.log 2>&1
CRONEOF

# Allow xfa user to restart the service after updates
echo "xfa ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart xfa-test-runner" > /etc/sudoers.d/xfa-runner

echo ""
echo "=== Setup complete ==="
echo "Corpus directory: /opt/xfa-corpus"
echo "Results directory: /opt/xfa-results"
echo "Start runner:  systemctl start xfa-test-runner"
echo "Check timer:   systemctl list-timers xfa-test-runner.timer"
echo "View logs:     journalctl -u xfa-test-runner -f"
