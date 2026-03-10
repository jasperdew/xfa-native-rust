#!/usr/bin/env python3
"""XFA Quality Dashboard — reads results from VPS SQLite database.
Run: python3 scripts/dashboard-server.py
Open: http://localhost:8787
"""
import http.server
import json
import subprocess
import socketserver
from datetime import datetime

PORT = 8787
VPS = "root@46.225.223.175"
DB_PATH = "/opt/xfa-results/db/results-curated.sqlite"

# Cache to avoid hammering the VPS on every request
_cache = {"data": None, "ts": 0}
CACHE_TTL = 15  # seconds


class DashboardHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/" or self.path == "/index.html":
            self.send_response(200)
            self.send_header("Content-type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(generate_dashboard().encode())
        elif self.path == "/api/status":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(get_iterations()).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass  # Suppress logging


def query_vps(sql):
    """Run a sqlite3 query on the VPS via SSH. Returns raw stdout."""
    try:
        result = subprocess.run(
            ["ssh", "-o", "ConnectTimeout=5", "-o", "StrictHostKeyChecking=no",
             VPS, f"sqlite3 -separator '|' '{DB_PATH}' \"{sql}\""],
            capture_output=True, text=True, timeout=15,
        )
        if result.returncode != 0:
            print(f"SSH query failed: {result.stderr.strip()}")
            return ""
        return result.stdout.strip()
    except subprocess.TimeoutExpired:
        print("SSH query timed out")
        return ""
    except Exception as e:
        print(f"SSH query error: {e}")
        return ""


def get_iterations():
    """Fetch iteration data from the VPS database, with caching."""
    now = datetime.now().timestamp()
    if _cache["data"] is not None and (now - _cache["ts"]) < CACHE_TTL:
        return _cache["data"]

    # Query 1: per-run summary for pdfa_convert
    sql_runs = (
        "SELECT run_id, MIN(started_at) as started, "
        "SUM(CASE WHEN status='pass' THEN 1 ELSE 0 END) as pass_count, "
        "SUM(CASE WHEN status='fail' THEN 1 ELSE 0 END) as fail_count, "
        "SUM(CASE WHEN status='skip' THEN 1 ELSE 0 END) as skip_count, "
        "COUNT(*) as total "
        "FROM test_results "
        "GROUP BY run_id ORDER BY started ASC"
    )
    raw = query_vps(sql_runs)

    iterations = []
    if raw:
        for line in raw.split("\n"):
            parts = line.split("|")
            if len(parts) >= 6:
                run_id, started, pass_c, fail_c, skip_c, total = parts[:6]
                iterations.append({
                    "run_id": run_id,
                    "started_at": started,
                    "pass": int(pass_c),
                    "fail": int(fail_c),
                    "skip": int(skip_c),
                    "total": int(total),
                })

    # Query 2: per-test summary for the latest run (top failures)
    top_failures = []
    if iterations:
        latest_run = iterations[-1]["run_id"]
        sql_fails = (
            f"SELECT test_name, pdf_file, error_message "
            f"FROM test_results "
            f"WHERE run_id='{latest_run}' AND status='fail' "
            f"ORDER BY test_name, pdf_file LIMIT 50"
        )
        raw_fails = query_vps(sql_fails)
        if raw_fails:
            for line in raw_fails.split("\n"):
                parts = line.split("|", 2)
                if len(parts) >= 2:
                    top_failures.append({
                        "test": parts[0],
                        "pdf": parts[1],
                        "error": parts[2] if len(parts) > 2 else "",
                    })

    # Query 3: per-test breakdown for the latest run
    test_breakdown = []
    if iterations:
        latest_run = iterations[-1]["run_id"]
        sql_tests = (
            f"SELECT test_name, "
            f"SUM(CASE WHEN status='pass' THEN 1 ELSE 0 END), "
            f"SUM(CASE WHEN status='fail' THEN 1 ELSE 0 END), "
            f"SUM(CASE WHEN status='skip' THEN 1 ELSE 0 END), "
            f"COUNT(*) "
            f"FROM test_results WHERE run_id='{latest_run}' "
            f"GROUP BY test_name ORDER BY test_name"
        )
        raw_tests = query_vps(sql_tests)
        if raw_tests:
            for line in raw_tests.split("\n"):
                parts = line.split("|")
                if len(parts) >= 5:
                    test_breakdown.append({
                        "test_name": parts[0],
                        "pass": int(parts[1]),
                        "fail": int(parts[2]),
                        "skip": int(parts[3]),
                        "total": int(parts[4]),
                    })

    data = {
        "iterations": iterations,
        "top_failures": top_failures,
        "test_breakdown": test_breakdown,
        "updated_at": datetime.now().isoformat(),
    }
    _cache["data"] = data
    _cache["ts"] = now
    return data


def generate_dashboard():
    status = get_iterations()
    iterations = status["iterations"]
    top_failures = status["top_failures"]
    test_breakdown = status["test_breakdown"]

    # Current stats from latest iteration
    latest = iterations[-1] if iterations else {}
    current_pass = latest.get("pass", 0)
    current_fail = latest.get("fail", 0)
    current_skip = latest.get("skip", 0)
    current_total = latest.get("total", 0)
    pass_pct = f"{100*current_pass/current_total:.1f}" if current_total > 0 else "0"

    # Trend: first vs latest
    first = iterations[0] if iterations else {}
    first_pass = first.get("pass", 0)
    first_total = first.get("total", 1)
    first_pct = 100 * first_pass / first_total if first_total > 0 else 0
    latest_pct = 100 * current_pass / current_total if current_total > 0 else 0
    delta = latest_pct - first_pct
    delta_str = f"+{delta:.1f}%" if delta >= 0 else f"{delta:.1f}%"
    delta_color = "green" if delta >= 0 else "red"

    # Build iteration rows (newest first)
    iter_rows = ""
    for idx, i in enumerate(reversed(iterations)):
        total = i.get("total", 1)
        p = i.get("pass", 0)
        pct = f"{100*p/total:.1f}%" if total > 0 else "---"
        run_label = i.get("run_id", f"run-{idx}")
        started = i.get("started_at", "")[:19]
        iter_rows += f"""
        <tr>
            <td><code>{run_label}</code></td>
            <td>{started}</td>
            <td style="color:#10b981;font-weight:bold">{p}</td>
            <td style="color:#ef4444">{i.get('fail',0)}</td>
            <td style="color:#94a3b8">{i.get('skip',0)}</td>
            <td>{total}</td>
            <td style="font-weight:bold">{pct}</td>
        </tr>"""

    # Build test breakdown rows
    test_rows = ""
    for t in test_breakdown:
        total = t["total"]
        p = t["pass"]
        pct = f"{100*p/total:.1f}%" if total > 0 else "---"
        bar_w = int(100 * p / total) if total > 0 else 0
        test_rows += f"""
        <tr>
            <td><code>{t['test_name']}</code></td>
            <td style="color:#10b981">{p}</td>
            <td style="color:#ef4444">{t['fail']}</td>
            <td style="color:#94a3b8">{t['skip']}</td>
            <td>{total}</td>
            <td>
                <div style="background:#334155;border-radius:4px;height:18px;width:120px;position:relative">
                    <div style="background:#10b981;border-radius:4px;height:18px;width:{bar_w}%;"></div>
                    <span style="position:absolute;top:0;left:4px;font-size:11px;line-height:18px">{pct}</span>
                </div>
            </td>
        </tr>"""

    # Build failure rows
    fail_rows = ""
    for f in top_failures[:30]:
        err = f["error"][:120] if f["error"] else ""
        fail_rows += f"""
        <tr>
            <td><code>{f['test']}</code></td>
            <td style="font-size:12px">{f['pdf']}</td>
            <td style="font-size:12px;color:#f87171">{err}</td>
        </tr>"""

    # Chart data for trend (SVG sparkline)
    chart_svg = build_trend_chart(iterations)

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="refresh" content="30">
<title>XFA Quality Dashboard</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
         background: #0f172a; color: #e2e8f0; padding: 20px; max-width: 1400px; margin: 0 auto; }}
  h1 {{ color: #38bdf8; margin-bottom: 4px; }}
  .subtitle {{ color: #64748b; font-size: 13px; margin-bottom: 20px; }}
  h2 {{ color: #94a3b8; margin: 28px 0 10px; font-size: 14px; text-transform: uppercase;
       letter-spacing: 0.05em; }}
  .grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }}
  .card {{ background: #1e293b; border-radius: 8px; padding: 20px; }}
  .card .label {{ color: #94a3b8; font-size: 12px; text-transform: uppercase; }}
  .card .value {{ font-size: 32px; font-weight: bold; margin-top: 4px; }}
  .card .value.green {{ color: #10b981; }}
  .card .value.red {{ color: #ef4444; }}
  .card .value.yellow {{ color: #f59e0b; }}
  .card .value.blue {{ color: #38bdf8; }}
  .chart-card {{ background: #1e293b; border-radius: 8px; padding: 20px; margin-bottom: 24px; }}
  table {{ width: 100%; border-collapse: collapse; background: #1e293b; border-radius: 8px;
           overflow: hidden; margin-bottom: 16px; }}
  th {{ background: #334155; padding: 10px 12px; text-align: left; font-size: 11px;
       text-transform: uppercase; color: #94a3b8; letter-spacing: 0.05em; }}
  td {{ padding: 8px 12px; border-top: 1px solid #334155; font-size: 14px; }}
  tr:hover td {{ background: #253347; }}
  code {{ background: #334155; padding: 2px 6px; border-radius: 4px; font-size: 12px; }}
  .updated {{ color: #64748b; font-size: 12px; margin-top: 20px; text-align: right; }}
  .two-col {{ display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }}
  @media (max-width: 900px) {{
    .grid {{ grid-template-columns: repeat(2, 1fr); }}
    .two-col {{ grid-template-columns: 1fr; }}
  }}
</style>
</head>
<body>
<h1>XFA Quality Dashboard</h1>
<div class="subtitle">Live data from VPS &middot; {VPS}:{DB_PATH}</div>

<div class="grid">
  <div class="card">
    <div class="label">Pass Rate (latest)</div>
    <div class="value green">{pass_pct}%</div>
  </div>
  <div class="card">
    <div class="label">Pass / Total</div>
    <div class="value blue">{current_pass} / {current_total}</div>
  </div>
  <div class="card">
    <div class="label">Failing</div>
    <div class="value red">{current_fail}</div>
  </div>
  <div class="card">
    <div class="label">Trend (first &rarr; latest)</div>
    <div class="value {delta_color}">{delta_str}</div>
  </div>
</div>

<h2>Pass Rate Trend</h2>
<div class="chart-card">
{chart_svg}
</div>

<div class="two-col">
<div>
<h2>Test Breakdown (latest run)</h2>
<table>
<thead><tr><th>Test</th><th>Pass</th><th>Fail</th><th>Skip</th><th>Total</th><th>Rate</th></tr></thead>
<tbody>{test_rows if test_rows else '<tr><td colspan="6" style="text-align:center;color:#64748b">No data</td></tr>'}</tbody>
</table>
</div>

<div>
<h2>Run History ({len(iterations)} runs)</h2>
<table>
<thead><tr><th>Run</th><th>Started</th><th>Pass</th><th>Fail</th><th>Skip</th><th>Total</th><th>Rate</th></tr></thead>
<tbody>{iter_rows if iter_rows else '<tr><td colspan="7" style="text-align:center;color:#64748b">No iterations yet</td></tr>'}</tbody>
</table>
</div>
</div>

<h2>Recent Failures (latest run, max 30)</h2>
<table>
<thead><tr><th>Test</th><th>PDF</th><th>Error</th></tr></thead>
<tbody>{fail_rows if fail_rows else '<tr><td colspan="3" style="text-align:center;color:#64748b">No failures</td></tr>'}</tbody>
</table>

<div class="updated">Updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} &middot; auto-refreshes every 30s &middot; cache TTL {CACHE_TTL}s</div>
</body>
</html>"""


def build_trend_chart(iterations):
    """Build an SVG trend chart of pass rate over iterations."""
    if len(iterations) < 1:
        return '<div style="color:#64748b;text-align:center;padding:30px">No data for trend chart</div>'

    W, H = 900, 200
    PAD_L, PAD_R, PAD_T, PAD_B = 50, 20, 20, 40

    chart_w = W - PAD_L - PAD_R
    chart_h = H - PAD_T - PAD_B

    # Compute pass rates
    rates = []
    labels = []
    for i in iterations:
        total = i.get("total", 1)
        p = i.get("pass", 0)
        rates.append(100 * p / total if total > 0 else 0)
        labels.append(i.get("started_at", "")[:10])

    if not rates:
        return '<div style="color:#64748b;text-align:center;padding:30px">No data</div>'

    min_r = max(0, min(rates) - 2)
    max_r = min(100, max(rates) + 2)
    if max_r - min_r < 5:
        min_r = max(0, max_r - 5)
    r_range = max_r - min_r if max_r > min_r else 1

    def x(idx):
        if len(rates) == 1:
            return PAD_L + chart_w / 2
        return PAD_L + (idx / (len(rates) - 1)) * chart_w

    def y(val):
        return PAD_T + chart_h - ((val - min_r) / r_range) * chart_h

    # Build polyline points
    points = " ".join(f"{x(i):.1f},{y(r):.1f}" for i, r in enumerate(rates))

    # Area fill points
    area_points = f"{x(0):.1f},{PAD_T + chart_h:.1f} " + points + f" {x(len(rates)-1):.1f},{PAD_T + chart_h:.1f}"

    # Y axis labels
    y_labels = ""
    n_ticks = 5
    for i in range(n_ticks + 1):
        val = min_r + (i / n_ticks) * r_range
        yp = y(val)
        y_labels += f'<text x="{PAD_L - 8}" y="{yp + 4}" text-anchor="end" fill="#64748b" font-size="11">{val:.0f}%</text>'
        y_labels += f'<line x1="{PAD_L}" y1="{yp}" x2="{W - PAD_R}" y2="{yp}" stroke="#1e293b" stroke-width="1"/>'

    # X axis labels (show max ~10 labels to avoid overlap)
    x_labels = ""
    step = max(1, len(labels) // 10)
    for i in range(0, len(labels), step):
        xp = x(i)
        x_labels += f'<text x="{xp}" y="{H - 5}" text-anchor="middle" fill="#64748b" font-size="10">{labels[i]}</text>'

    # Data point dots
    dots = ""
    for i, r in enumerate(rates):
        dots += f'<circle cx="{x(i):.1f}" cy="{y(r):.1f}" r="4" fill="#10b981" stroke="#0f172a" stroke-width="2"/>'
        # Tooltip-style value on hover area
        dots += f'<title>Run {i+1}: {r:.1f}%</title>'

    # Latest value annotation
    latest_annotation = ""
    if rates:
        lx = x(len(rates) - 1)
        ly = y(rates[-1])
        latest_annotation = f'<text x="{lx + 8}" y="{ly + 4}" fill="#10b981" font-size="12" font-weight="bold">{rates[-1]:.1f}%</text>'

    return f"""<svg width="100%" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
  {y_labels}
  {x_labels}
  <polygon points="{area_points}" fill="#10b981" opacity="0.1"/>
  <polyline points="{points}" fill="none" stroke="#10b981" stroke-width="2.5" stroke-linejoin="round"/>
  {dots}
  {latest_annotation}
</svg>"""


if __name__ == "__main__":
    print(f"Querying VPS {VPS} for data...")
    # Test the connection on startup
    data = get_iterations()
    n = len(data.get("iterations", []))
    print(f"Found {n} run(s) in database")
    print(f"Dashboard running at http://localhost:{PORT}")

    with socketserver.TCPServer(("", PORT), DashboardHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down")
