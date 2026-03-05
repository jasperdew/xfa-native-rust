#!/usr/bin/env bash
# veraPDF conformance validation for XFA-generated PDFs.
#
# Usage:
#   scripts/run-verapdf.sh [--install] [--flavour 2b] [--format json|html] [DIR]
#
# Arguments:
#   --install     Download and install veraPDF if not present
#   --flavour     PDF/A flavour: 1a, 1b, 2a, 2b, 2u, 3a, 3b, 3u (default: 2b)
#   --format      Report format: json or html (default: json)
#   DIR           Directory containing PDFs to validate (default: corpus/)
#
# Outputs:
#   reports/verapdf/summary.json     Aggregate pass/fail summary
#   reports/verapdf/<name>.json      Per-file validation results
#   reports/verapdf/report.html      HTML report (if --format html)

set -euo pipefail

VERAPDF_VERSION="1.28.2"
VERAPDF_DIR="${VERAPDF_DIR:-$HOME/.local/verapdf}"
VERAPDF_BIN="$VERAPDF_DIR/verapdf"
FLAVOUR="2b"
FORMAT="json"
INPUT_DIR="corpus"
REPORT_DIR="reports/verapdf"
INSTALL=false

# --- Parse arguments ---

while [[ $# -gt 0 ]]; do
    case "$1" in
        --install)
            INSTALL=true
            shift
            ;;
        --flavour|--profile)
            FLAVOUR="$2"
            shift 2
            ;;
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --help|-h)
            head -20 "$0" | grep '^#' | sed 's/^# \?//'
            exit 0
            ;;
        *)
            INPUT_DIR="$1"
            shift
            ;;
    esac
done

# --- Install veraPDF ---

install_verapdf() {
    echo "==> Installing veraPDF $VERAPDF_VERSION..."

    local url="https://software.verapdf.org/releases/verapdf-installer.zip"
    local tmpdir
    tmpdir=$(mktemp -d)

    echo "    Downloading from $url..."
    curl -fsSL -o "$tmpdir/verapdf.zip" "$url"
    unzip -q "$tmpdir/verapdf.zip" -d "$tmpdir"

    # Find the installer jar
    local installer_dir
    installer_dir=$(find "$tmpdir" -maxdepth 1 -type d -name 'verapdf-*' | head -1)
    if [[ -z "$installer_dir" ]]; then
        installer_dir="$tmpdir"
    fi

    mkdir -p "$VERAPDF_DIR"
    local installer_jar
    installer_jar=$(find "$installer_dir" -name 'verapdf-izpack-installer-*.jar' | head -1)

    if [[ -n "$installer_jar" ]]; then
        cat > "$tmpdir/auto-install.xml" <<XMLEOF
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<AutomatedInstallation langpack="eng">
    <com.izforge.izpack.panels.target.TargetPanel id="install_dir">
        <installpath>$VERAPDF_DIR</installpath>
    </com.izforge.izpack.panels.target.TargetPanel>
    <com.izforge.izpack.panels.packs.PacksPanel id="sdk_pack_select">
        <pack index="0" name="veraPDF GUI" selected="false"/>
        <pack index="1" name="veraPDF Mac and *nix Scripts" selected="true"/>
        <pack index="2" name="veraPDF Validation model" selected="true"/>
    </com.izforge.izpack.panels.packs.PacksPanel>
    <com.izforge.izpack.panels.install.InstallPanel id="install"/>
    <com.izforge.izpack.panels.finish.FinishPanel id="finish"/>
</AutomatedInstallation>
XMLEOF
        java -jar "$installer_jar" "$tmpdir/auto-install.xml" 2>/dev/null || {
            echo "    Java-based installer failed. Trying green-install fallback..."
        }
    fi

    # Fallback: extract green-install zip
    if [[ ! -x "$VERAPDF_BIN" ]]; then
        local green_zip
        green_zip=$(find "$tmpdir" -name 'verapdf-greeninstall-*.zip' | head -1)
        if [[ -n "$green_zip" ]]; then
            echo "    Using green-install package..."
            unzip -q -o "$green_zip" -d "$VERAPDF_DIR"
            local found_bin
            found_bin=$(find "$VERAPDF_DIR" -name 'verapdf' -type f | head -1)
            if [[ -n "$found_bin" ]] && [[ "$found_bin" != "$VERAPDF_BIN" ]]; then
                local nested_dir
                nested_dir=$(dirname "$found_bin")
                if [[ "$nested_dir" != "$VERAPDF_DIR" ]]; then
                    mv "$nested_dir"/* "$VERAPDF_DIR/" 2>/dev/null || true
                fi
            fi
        fi
    fi

    rm -rf "$tmpdir"

    if [[ -x "$VERAPDF_BIN" ]]; then
        echo "    veraPDF installed at $VERAPDF_BIN"
    else
        echo "    ERROR: veraPDF installation failed. Ensure Java 11+ is available."
        echo "    You can also install manually: https://verapdf.org/software/"
        exit 1
    fi
}

if $INSTALL; then
    install_verapdf
fi

# --- Check veraPDF availability ---

if ! [[ -x "$VERAPDF_BIN" ]]; then
    if command -v verapdf &>/dev/null; then
        VERAPDF_BIN="verapdf"
    else
        echo "ERROR: veraPDF not found."
        echo "Run: scripts/run-verapdf.sh --install"
        echo "  or: set VERAPDF_DIR to the installation directory"
        exit 1
    fi
fi

# --- Validate PDFs ---

if [[ ! -d "$INPUT_DIR" ]]; then
    echo "ERROR: Input directory '$INPUT_DIR' not found."
    exit 1
fi

pdf_files=("$INPUT_DIR"/*.pdf)
if [[ ${#pdf_files[@]} -eq 0 ]]; then
    echo "No PDF files found in '$INPUT_DIR'."
    exit 0
fi

mkdir -p "$REPORT_DIR"

echo "==> Validating ${#pdf_files[@]} PDFs against PDF/A-$FLAVOUR"
echo "    Input:  $INPUT_DIR"
echo "    Output: $REPORT_DIR"
echo ""

total=0
passed=0
failed=0
errors=0
results=()

for pdf in "${pdf_files[@]}"; do
    name=$(basename "$pdf" .pdf)
    total=$((total + 1))

    printf "  [%3d/%d] %-40s " "$total" "${#pdf_files[@]}" "$name"

    report_file="$REPORT_DIR/${name}.json"

    # veraPDF exits non-zero when validation fails; capture output regardless
    "$VERAPDF_BIN" --format json --flavour "$FLAVOUR" "$pdf" > "$report_file" 2>/dev/null || true

    # Parse the JSON result
    status=$(python3 -c "
import json, sys
try:
    data = json.load(open('$report_file'))
    jobs = data.get('report', data).get('jobs', [])
    if not jobs:
        print('ERROR (no jobs)')
        sys.exit()
    job = jobs[0]
    vr = job.get('validationResult', {})
    # validationResult can be an array or object
    if isinstance(vr, list):
        vr = vr[0] if vr else {}
    compliant = vr.get('compliant', False)
    details = vr.get('details', {})
    rules_failed = details.get('failedRules', vr.get('totalRuleFailures', '?'))
    rules_passed = details.get('passedRules', '?')
    if compliant:
        print(f'PASS ({rules_passed} rules)')
    else:
        print(f'FAIL ({rules_failed} failed)')
except Exception as e:
    print(f'ERROR ({e})')
" 2>/dev/null || echo "PARSE_ERROR")

    case "$status" in
        PASS*)
            echo "PASS  $status"
            passed=$((passed + 1))
            results+=("{\"file\":\"$name\",\"status\":\"pass\",\"detail\":\"$status\"}")
            ;;
        FAIL*)
            echo "FAIL  $status"
            failed=$((failed + 1))
            results+=("{\"file\":\"$name\",\"status\":\"fail\",\"detail\":\"$status\"}")
            ;;
        *)
            echo "ERR   $status"
            errors=$((errors + 1))
            results+=("{\"file\":\"$name\",\"status\":\"error\",\"detail\":\"$status\"}")
            ;;
    esac
done

# --- Generate summary ---

pass_rate=0
if [[ $total -gt 0 ]]; then
    pass_rate=$(python3 -c "print(round($passed / $total * 100, 1))" 2>/dev/null || echo "0")
fi

echo ""
echo "=== Conformance Summary (PDF/A-$FLAVOUR) ==="
echo "  Total:       $total"
echo "  Passed:      $passed"
echo "  Failed:      $failed"
echo "  Errors:      $errors"
echo "  Pass rate:   ${pass_rate}%"
echo ""

# Write JSON summary
results_json=$(printf '%s,' "${results[@]}" | sed 's/,$//')
cat > "$REPORT_DIR/summary.json" <<EOF
{
  "flavour": "PDF/A-$FLAVOUR",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "total": $total,
  "passed": $passed,
  "failed": $failed,
  "errors": $errors,
  "pass_rate": $pass_rate,
  "results": [$results_json]
}
EOF

echo "  Summary: $REPORT_DIR/summary.json"

# --- Generate HTML report (if requested) ---

if [[ "$FORMAT" == "html" ]]; then
    python3 - "$REPORT_DIR/summary.json" "$REPORT_DIR/report.html" <<'PYEOF'
import json, sys

with open(sys.argv[1]) as f:
    data = json.load(f)

html = f"""<!DOCTYPE html>
<html><head>
<meta charset="utf-8">
<title>veraPDF Conformance Report</title>
<style>
  body {{ font-family: system-ui, sans-serif; max-width: 900px; margin: 2rem auto; padding: 0 1rem; }}
  h1 {{ color: #1a1a2e; }}
  .summary {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin: 1.5rem 0; }}
  .card {{ background: #f8f9fa; border-radius: 8px; padding: 1rem; text-align: center; }}
  .card .value {{ font-size: 2rem; font-weight: bold; }}
  .card.pass .value {{ color: #28a745; }}
  .card.fail .value {{ color: #dc3545; }}
  table {{ width: 100%; border-collapse: collapse; margin-top: 1.5rem; }}
  th, td {{ padding: 0.5rem; text-align: left; border-bottom: 1px solid #dee2e6; }}
  th {{ background: #f8f9fa; }}
  .status-pass {{ color: #28a745; font-weight: bold; }}
  .status-fail {{ color: #dc3545; font-weight: bold; }}
  .status-error {{ color: #fd7e14; font-weight: bold; }}
  footer {{ margin-top: 2rem; color: #6c757d; font-size: 0.85rem; }}
</style>
</head><body>
<h1>veraPDF Conformance Report</h1>
<p>Flavour: <strong>{data['flavour']}</strong> | Generated: {data['timestamp']}</p>
<div class="summary">
  <div class="card"><div class="value">{data['total']}</div><div>Total</div></div>
  <div class="card pass"><div class="value">{data['passed']}</div><div>Passed</div></div>
  <div class="card fail"><div class="value">{data['failed']}</div><div>Failed</div></div>
  <div class="card"><div class="value">{data['pass_rate']}%</div><div>Pass Rate</div></div>
</div>
<table>
<tr><th>File</th><th>Status</th><th>Details</th></tr>
"""

for r in data['results']:
    status_class = f"status-{r['status']}"
    status_label = r['status'].upper()
    detail = r.get('detail', '')
    html += f'<tr><td>{r["file"]}</td><td class="{status_class}">{status_label}</td><td>{detail}</td></tr>\n'

html += f"""</table>
<footer>Generated by XFA-Native-Rust veraPDF integration</footer>
</body></html>"""

with open(sys.argv[2], 'w') as f:
    f.write(html)

print(f"  HTML report: {sys.argv[2]}")
PYEOF
fi

# Exit with non-zero if any validations failed (useful for CI)
if [[ $failed -gt 0 ]] || [[ $errors -gt 0 ]]; then
    exit 1
fi
