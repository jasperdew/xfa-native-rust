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

# 5. veraPDF
echo "[5/7] Installing veraPDF..."
VERAPDF_VERSION="1.26.2"
if [ ! -f /usr/local/bin/verapdf ]; then
    wget -q "https://software.verapdf.org/releases/verapdf-greenfield-${VERAPDF_VERSION}-installer.zip" -O /tmp/verapdf.zip
    cd /tmp && unzip -qo verapdf.zip
    mkdir -p /opt/verapdf
    # Copy installer contents and create wrapper script.
    cp -r verapdf-greenfield-${VERAPDF_VERSION}/* /opt/verapdf/
    cat > /usr/local/bin/verapdf << 'WRAPPER'
#!/bin/sh
exec java -jar /opt/verapdf/verapdf-greenfield-*.jar "$@"
WRAPPER
    chmod +x /usr/local/bin/verapdf
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

echo ""
echo "=== Setup complete ==="
echo "Corpus directory: /opt/xfa-corpus"
echo "Results directory: /opt/xfa-results"
echo "Start runner:  systemctl start xfa-test-runner"
echo "Check timer:   systemctl list-timers xfa-test-runner.timer"
echo "View logs:     journalctl -u xfa-test-runner -f"
