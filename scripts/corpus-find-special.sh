#!/usr/bin/env bash
# Zoek en download specifieke categorieën PDFs — DRAAIT OP VPS
#
# Downloads:
#   - ZUGFeRD/Factur-X test facturen (Mustang project)
#   - XFA formulieren (IRS forms)
#   - XRechnung test PDFs
#
# Usage:
#   ./scripts/corpus-find-special.sh [--target DIR] [--category CAT]
#
# Categories: zugferd, xfa, xrechnung, all (default)

set -euo pipefail

CORPUS_DIR="/opt/xfa-corpus"
CATEGORY="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)   CORPUS_DIR="$2"; shift 2 ;;
        --category) CATEGORY="$2"; shift 2 ;;
        -h|--help)
            head -14 "$0" | grep '^#' | sed 's/^# \?//'
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

download_zugferd() {
    echo "=== ZUGFeRD/Factur-X test invoices ==="
    local dir="$CORPUS_DIR/invoices/zugferd"
    mkdir -p "$dir"

    # Mustang project — reference ZUGFeRD implementation with test PDFs
    local clone_dir="/tmp/mustang-$$"
    echo "  Cloning Mustang project..."
    git clone --depth 1 "https://github.com/ZUGFeRD/mustangproject.git" \
        "$clone_dir" 2>/dev/null || {
        echo "  SKIP: Could not clone Mustang project"
        return 1
    }

    local count
    count=$(find "$clone_dir" -iname "*.pdf" | wc -l | tr -d ' ')
    find "$clone_dir" -iname "*.pdf" -exec cp {} "$dir/" \;
    rm -rf "$clone_dir"
    echo "  Collected $count ZUGFeRD PDFs -> $dir"
}

download_xfa() {
    echo "=== XFA formulieren (IRS) ==="
    local dir="$CORPUS_DIR/forms/xfa"
    mkdir -p "$dir"

    local forms=(f1040 f1040a f1040ez f1099 f1099misc f1099nec
                 w2 w4 w9 f941 f940 f8879 f4506t f1065 f1120)
    local downloaded=0

    for form in "${forms[@]}"; do
        if [[ ! -f "$dir/${form}.pdf" ]]; then
            echo "  Downloading ${form}.pdf..."
            wget -q --timeout=30 \
                "https://www.irs.gov/pub/irs-pdf/${form}.pdf" \
                -O "$dir/${form}.pdf" 2>/dev/null || {
                echo "    SKIP: ${form}.pdf"
                rm -f "$dir/${form}.pdf"
                continue
            }
            downloaded=$((downloaded + 1))
        fi
    done
    echo "  Downloaded $downloaded IRS XFA forms -> $dir"
}

download_xrechnung() {
    echo "=== XRechnung test PDFs ==="
    local dir="$CORPUS_DIR/invoices/xrechnung"
    mkdir -p "$dir"

    # KoSIT XRechnung test documents
    local clone_dir="/tmp/xrechnung-$$"
    echo "  Cloning XRechnung testsuite..."
    git clone --depth 1 "https://github.com/itplr-kosit/xrechnung-testsuite.git" \
        "$clone_dir" 2>/dev/null || {
        echo "  SKIP: Could not clone XRechnung testsuite"
        return 1
    }

    local count
    count=$(find "$clone_dir" -iname "*.pdf" | wc -l | tr -d ' ')
    if [[ "$count" -gt 0 ]]; then
        find "$clone_dir" -iname "*.pdf" -exec cp {} "$dir/" \;
    fi
    rm -rf "$clone_dir"
    echo "  Collected $count XRechnung PDFs -> $dir"
}

run_category() {
    case "$1" in
        zugferd)    download_zugferd ;;
        xfa)        download_xfa ;;
        xrechnung)  download_xrechnung ;;
        all)
            download_zugferd; echo ""
            download_xfa; echo ""
            download_xrechnung
            ;;
        *)
            echo "Unknown category: $1"
            echo "Available: zugferd, xfa, xrechnung, all"
            exit 1
            ;;
    esac
}

echo "[$(date)] Finding special category PDFs"
echo "  Target: $CORPUS_DIR"
echo ""
run_category "$CATEGORY"
echo ""
echo "[$(date)] Special categories complete."
