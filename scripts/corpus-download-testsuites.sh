#!/usr/bin/env bash
# Download PDF test suites van GitHub — DRAAIT OP VPS
#
# Sources:
#   - PDF Association test resources
#   - Isartor test suite (PDF/A violations)
#   - veraPDF test corpus
#   - Apache PDFBox test PDFs
#
# Usage:
#   ./scripts/corpus-download-testsuites.sh [--target DIR] [--suite SUITE]
#
# Suites: pdf-assoc, isartor, verapdf, pdfbox, all (default)

set -euo pipefail

CORPUS_DIR="/opt/xfa-corpus"
SUITE="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        --target) CORPUS_DIR="$2"; shift 2 ;;
        --suite)  SUITE="$2"; shift 2 ;;
        -h|--help)
            head -14 "$0" | grep '^#' | sed 's/^# \?//'
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

clone_and_collect() {
    local repo_url="$1"
    local target_dir="$2"
    local clone_dir="/tmp/corpus-clone-$$"

    mkdir -p "$target_dir"
    echo "  Cloning $repo_url..."
    git clone --depth 1 "$repo_url" "$clone_dir" 2>/dev/null || {
        echo "  SKIP: Could not clone $repo_url"
        return 1
    }

    local count
    count=$(find "$clone_dir" -iname "*.pdf" | wc -l | tr -d ' ')
    find "$clone_dir" -iname "*.pdf" -exec cp {} "$target_dir/" \;
    rm -rf "$clone_dir"
    echo "  Collected $count PDFs -> $target_dir"
}

download_pdf_assoc() {
    echo "=== PDF Association test resources ==="
    clone_and_collect \
        "https://github.com/pdf-association/pdf-resources.git" \
        "$CORPUS_DIR/tagged/pdf-assoc"
}

download_isartor() {
    echo "=== Isartor test suite (PDF/A violations) ==="
    local dir="$CORPUS_DIR/malformed/isartor"
    mkdir -p "$dir"

    echo "  Downloading Isartor test suite..."
    wget -q --timeout=60 \
        "https://www.pdfa.org/wp-content/uploads/2017/07/isartor-testsuite.zip" \
        -O /tmp/isartor.zip 2>/dev/null || {
        echo "  SKIP: Could not download Isartor suite"
        return 1
    }

    unzip -j -o -q /tmp/isartor.zip "*.pdf" -d "$dir" 2>/dev/null
    local count
    count=$(find "$dir" -iname "*.pdf" | wc -l | tr -d ' ')
    rm -f /tmp/isartor.zip
    echo "  Collected $count PDFs -> $dir"
}

download_verapdf() {
    echo "=== veraPDF test corpus ==="
    clone_and_collect \
        "https://github.com/veraPDF/veraPDF-corpus.git" \
        "$CORPUS_DIR/tagged/verapdf"
}

download_pdfbox() {
    echo "=== Apache PDFBox test PDFs ==="
    local dir="$CORPUS_DIR/general/pdfbox"
    local clone_dir="/tmp/pdfbox-tests-$$"
    mkdir -p "$dir"

    echo "  Cloning PDFBox (sparse checkout)..."
    git clone --depth 1 --filter=blob:none --sparse \
        "https://github.com/apache/pdfbox.git" "$clone_dir" 2>/dev/null || {
        echo "  SKIP: Could not clone PDFBox"
        return 1
    }

    (
        cd "$clone_dir"
        git sparse-checkout set pdfbox/src/test/resources/input 2>/dev/null || true
    )

    local count
    count=$(find "$clone_dir" -iname "*.pdf" | wc -l | tr -d ' ')
    find "$clone_dir" -iname "*.pdf" -exec cp {} "$dir/" \;
    rm -rf "$clone_dir"
    echo "  Collected $count PDFs -> $dir"
}

run_suite() {
    case "$1" in
        pdf-assoc) download_pdf_assoc ;;
        isartor)   download_isartor ;;
        verapdf)   download_verapdf ;;
        pdfbox)    download_pdfbox ;;
        all)
            download_pdf_assoc; echo ""
            download_isartor; echo ""
            download_verapdf; echo ""
            download_pdfbox
            ;;
        *)
            echo "Unknown suite: $1"
            echo "Available: pdf-assoc, isartor, verapdf, pdfbox, all"
            exit 1
            ;;
    esac
}

echo "[$(date)] Downloading test suites to $CORPUS_DIR"
echo ""
run_suite "$SUITE"
echo ""
echo "[$(date)] Test suites download complete."
total=$(find "$CORPUS_DIR" -iname "*.pdf" | wc -l | tr -d ' ')
echo "Total PDFs in corpus: $total"
