#!/usr/bin/env bash
# Index compliance test suites and generate ground truth TSV.
#
# Scans VeraPDF, Isartor, and BFO test suites for PDF files,
# extracts expected pass/fail from file names, and infers the
# PDF/A profile from the directory structure.
#
# Output: /opt/xfa-corpus/compliance-suites/ground-truth.tsv
# Format: path\tsuite\tprofile\tclause\texpected
#
# Usage:
#   ./scripts/compliance-index.sh [--target DIR]

set -euo pipefail

BASE="/opt/xfa-corpus/compliance-suites"
OUT="$BASE/ground-truth.tsv"

while [[ $# -gt 0 ]]; do
    case $1 in
        --target) BASE="$2"; OUT="$BASE/ground-truth.tsv"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "path	suite	profile	clause	expected" > "$OUT"
count=0

# --- VeraPDF corpus ---
if [[ -d "$BASE/verapdf" ]]; then
    while IFS= read -r -d '' pdf; do
        rel="${pdf#$BASE/verapdf/}"
        fname="$(basename "$pdf" .pdf)"

        # Extract expected result from filename
        if [[ "$fname" == *"-pass-"* ]] || [[ "$fname" == *"-pass" ]]; then
            expected="pass"
        elif [[ "$fname" == *"-fail-"* ]] || [[ "$fname" == *"-fail" ]]; then
            expected="fail"
        else
            expected="unknown"
        fi

        # Extract profile from directory
        profile=""
        case "$rel" in
            PDF_A-1a/*) profile="1a" ;;
            PDF_A-1b/*) profile="1b" ;;
            PDF_A-2a/*) profile="2a" ;;
            PDF_A-2b/*) profile="2b" ;;
            PDF_A-2u/*) profile="2u" ;;
            PDF_A-3a/*) profile="3a" ;;
            PDF_A-3b/*) profile="3b" ;;
            PDF_A-3u/*) profile="3u" ;;
            PDF_A-4*)   profile="4" ;;
            "Isartor test files/PDFA-1b"*) profile="1b" ;;
            "Isartor test files"*) profile="1b" ;;
            *) profile="unknown" ;;
        esac

        # Extract clause from filename or directory
        clause=""
        if [[ "$fname" =~ ([0-9]+-[0-9]+-[0-9]+-[0-9]+) ]]; then
            clause="${BASH_REMATCH[1]//-/.}"
        elif [[ "$fname" =~ ([0-9]+-[0-9]+-[0-9]+) ]]; then
            clause="${BASH_REMATCH[1]//-/.}"
        elif [[ "$rel" =~ ([0-9]+\.[0-9]+\.[0-9]+) ]]; then
            clause="${BASH_REMATCH[1]}"
        elif [[ "$rel" =~ ([0-9]+\.[0-9]+) ]]; then
            clause="${BASH_REMATCH[1]}"
        fi

        # Determine suite
        if [[ "$rel" == "Isartor test files"* ]]; then
            suite="isartor"
        else
            suite="verapdf"
        fi

        printf '%s\t%s\t%s\t%s\t%s\n' "$pdf" "$suite" "$profile" "$clause" "$expected" >> "$OUT"
        count=$((count + 1))
    done < <(find "$BASE/verapdf" -name "*.pdf" -print0 | sort -z)
fi

# --- BFO test suite ---
if [[ -d "$BASE/bfo" ]]; then
    while IFS= read -r -d '' pdf; do
        fname="$(basename "$pdf" .pdf)"

        if [[ "$fname" == *"-pass"* ]]; then
            expected="pass"
        elif [[ "$fname" == *"-fail"* ]]; then
            expected="fail"
        else
            expected="unknown"
        fi

        profile=""
        case "$fname" in
            pdfa1-*) profile="1b" ;;
            pdfa2-*) profile="2b" ;;
            pdfa3-*) profile="3b" ;;
            *) profile="unknown" ;;
        esac

        clause=""
        if [[ "$fname" =~ pdfa[0-9]+-([0-9]+-[0-9]+-[0-9]+-[0-9]+) ]]; then
            clause="${BASH_REMATCH[1]//-/.}"
        elif [[ "$fname" =~ pdfa[0-9]+-([0-9]+-[0-9]+-[0-9]+) ]]; then
            clause="${BASH_REMATCH[1]//-/.}"
        elif [[ "$fname" =~ pdfa[0-9]+-([0-9]+-[0-9]+) ]]; then
            clause="${BASH_REMATCH[1]//-/.}"
        fi

        printf '%s\t%s\t%s\t%s\t%s\n' "$pdf" "bfo" "$profile" "$clause" "$expected" >> "$OUT"
        count=$((count + 1))
    done < <(find "$BASE/bfo" -name "*.pdf" -print0 | sort -z)
fi

echo "Indexed $count files -> $OUT"

echo ""
echo "=== Summary ==="
echo "Total files: $count"
echo "By suite:"
tail -n+2 "$OUT" | cut -f2 | sort | uniq -c | sort -rn | while read cnt suite; do
    echo "  $suite: $cnt"
done
echo "By expected:"
tail -n+2 "$OUT" | cut -f5 | sort | uniq -c | sort -rn | while read cnt exp; do
    echo "  $exp: $cnt"
done
