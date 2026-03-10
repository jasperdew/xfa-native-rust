#!/usr/bin/env python3
"""XFA Quality Dashboard — reads results from VPS SQLite database.
Run: python3 scripts/dashboard-server.py
Open: http://localhost:8787
"""
import http.server
import json
import subprocess
import socketserver
import os
from datetime import datetime

PORT = 8787
VPS = "root@46.225.223.175"
DB_PATH = "/opt/xfa-results/db/results-curated.sqlite"

_cache = {"data": None, "ts": 0}
CACHE_TTL = 15


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
            self.wfile.write(json.dumps(get_all_data()).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass


def query_vps(sql):
    try:
        result = subprocess.run(
            ["ssh", "-o", "ConnectTimeout=5", "-o", "StrictHostKeyChecking=no",
             VPS, f"sqlite3 -separator '|' '{DB_PATH}' \"{sql}\""],
            capture_output=True, text=True, timeout=15,
        )
        return result.stdout.strip() if result.returncode == 0 else ""
    except (subprocess.TimeoutExpired, Exception):
        return ""


def get_github_issues():
    try:
        result = subprocess.run(
            ["gh", "issue", "list", "--state", "all",
             "--json", "number,title,state", "--limit", "30"],
            capture_output=True, text=True, timeout=10,
            cwd=os.path.expanduser("~/Documents/XFA")
        )
        if result.returncode == 0:
            return json.loads(result.stdout)
    except Exception:
        pass
    return []


def get_all_data():
    now = datetime.now().timestamp()
    if _cache["data"] is not None and (now - _cache["ts"]) < CACHE_TTL:
        return _cache["data"]

    # Per-run summary for pdfa_convert
    raw = query_vps(
        "SELECT run_id, MIN(started_at), "
        "SUM(CASE WHEN status='pass' THEN 1 ELSE 0 END), "
        "SUM(CASE WHEN status='fail' THEN 1 ELSE 0 END), "
        "SUM(CASE WHEN status='skip' THEN 1 ELSE 0 END), "
        "COUNT(*) FROM test_results WHERE test_name='pdfa_convert' "
        "GROUP BY run_id ORDER BY MIN(started_at) ASC"
    )
    iterations = []
    if raw:
        for line in raw.split("\n"):
            parts = line.split("|")
            if len(parts) >= 6:
                iterations.append({
                    "run_id": parts[0], "started_at": parts[1],
                    "pass": int(parts[2]), "fail": int(parts[3]),
                    "skip": int(parts[4]), "total": int(parts[5]),
                })

    # veraPDF rule breakdown from oracle cache
    rule_breakdown = []
    raw_rules = query_vps(
        "SELECT json_extract(rf.value, '$.clause') || ':' || "
        "json_extract(rf.value, '$.test_number'), "
        "json_extract(rf.value, '$.description'), COUNT(*) "
        "FROM oracle_cache oc, json_each(json_extract(oc.result_json, '$.rule_failures')) rf "
        "WHERE oc.oracle_name='verapdf' AND json_extract(oc.result_json, '$.failed_rules')>0 "
        "GROUP BY 1 ORDER BY 3 DESC"
    )
    if raw_rules:
        for line in raw_rules.split("\n"):
            parts = line.split("|", 2)
            if len(parts) >= 2:
                rule_breakdown.append({
                    "rule": parts[0],
                    "description": parts[1] if len(parts) > 1 else "",
                    "count": int(parts[-1]) if parts[-1].isdigit() else 0,
                })

    # Skip reasons for latest run
    skip_breakdown = []
    if iterations:
        latest_run = iterations[-1]["run_id"]
        raw_skips = query_vps(
            f"SELECT error_message, COUNT(*) FROM test_results "
            f"WHERE test_name='pdfa_convert' AND status='skip' AND run_id='{latest_run}' "
            f"GROUP BY error_message ORDER BY 2 DESC"
        )
        if raw_skips:
            for line in raw_skips.split("\n"):
                parts = line.rsplit("|", 1)
                if len(parts) == 2:
                    skip_breakdown.append({
                        "reason": parts[0],
                        "count": int(parts[1]) if parts[1].isdigit() else 0,
                    })

    running = iterations[-1]["total"] < 900 if iterations else False

    data = {
        "iterations": iterations,
        "rule_breakdown": rule_breakdown,
        "skip_breakdown": skip_breakdown,
        "github_issues": get_github_issues(),
        "running": running,
        "updated_at": datetime.now().isoformat(),
    }
    _cache["data"] = data
    _cache["ts"] = now
    return data


def generate_dashboard():
    data = get_all_data()
    iterations = data["iterations"]
    rule_breakdown = data["rule_breakdown"]
    skip_breakdown = data["skip_breakdown"]
    github_issues = data["github_issues"]
    running = data["running"]

    pdfa_iters = [i for i in iterations if i.get("total", 0) >= 900]
    latest = pdfa_iters[-1] if pdfa_iters else {}
    p, f_cnt = latest.get("pass", 0), latest.get("fail", 0)
    applicable = p + f_cnt
    pass_pct = f"{100*p/applicable:.1f}" if applicable > 0 else "0"

    first = pdfa_iters[0] if pdfa_iters else {}
    f_app = first.get("pass", 0) + first.get("fail", 0)
    f_pct = 100 * first.get("pass", 0) / f_app if f_app > 0 else 0
    l_pct = 100 * p / applicable if applicable > 0 else 0
    delta = l_pct - f_pct
    delta_str = f"+{delta:.1f}%" if delta >= 0 else f"{delta:.1f}%"
    delta_color = "#10b981" if delta >= 0 else "#ef4444"
    run_badge = '<span class="badge running">RUNNING</span>' if running else '<span class="badge idle">IDLE</span>'

    iter_rows = ""
    for i in reversed(pdfa_iters[-20:]):
        app = i["pass"] + i["fail"]
        pct = f"{100*i['pass']/app:.1f}%" if app > 0 else "---"
        iter_rows += f'<tr><td><code>{i["run_id"]}</code></td><td>{i["started_at"][:19]}</td><td style="color:#10b981;font-weight:bold">{i["pass"]}</td><td style="color:#ef4444">{i["fail"]}</td><td style="color:#94a3b8">{i["skip"]}</td><td style="font-weight:bold">{pct}</td></tr>'

    rule_rows = ""
    th = sum(r["count"] for r in rule_breakdown) or 1
    for r in rule_breakdown:
        bw = int(100 * r["count"] / th)
        rule_rows += f'<tr><td><code style="color:#f59e0b">{r["rule"]}</code></td><td style="font-weight:bold;color:#ef4444">{r["count"]}</td><td><div style="background:#334155;border-radius:3px;height:14px;width:200px"><div style="background:#ef4444;border-radius:3px;height:14px;width:{bw}%"></div></div></td><td style="font-size:12px;color:#94a3b8">{(r["description"] or "")[:80]}</td></tr>'

    skip_rows = "".join(f'<tr><td style="font-size:13px">{s["reason"]}</td><td style="font-weight:bold">{s["count"]}</td></tr>' for s in skip_breakdown)

    open_c = sum(1 for i in github_issues if i.get("state") == "OPEN")
    closed_c = sum(1 for i in github_issues if i.get("state") == "CLOSED")
    gh_rows = ""
    for issue in github_issues[:15]:
        st = issue.get("state", "OPEN")
        c = "#10b981" if st == "CLOSED" else "#f59e0b"
        ic = "&#x2713;" if st == "CLOSED" else "&#x25CB;"
        gh_rows += f'<tr><td style="color:{c}">{ic} #{issue.get("number","")}</td><td style="font-size:13px">{issue.get("title","")[:65]}</td><td style="color:{c};font-size:12px">{st}</td></tr>'

    chart = build_trend_chart(pdfa_iters)

    return f"""<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><meta http-equiv="refresh" content="30">
<title>XFA PDF/A Quality Dashboard</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,monospace;background:#0f172a;color:#e2e8f0;padding:20px;max-width:1600px;margin:0 auto}}
h1{{color:#38bdf8;margin-bottom:4px;font-size:24px}}
.subtitle{{color:#64748b;font-size:13px;margin-bottom:20px}}
h2{{color:#94a3b8;margin:20px 0 8px;font-size:13px;text-transform:uppercase;letter-spacing:.05em}}
.grid{{display:grid;grid-template-columns:repeat(5,1fr);gap:12px;margin-bottom:20px}}
.card{{background:#1e293b;border-radius:8px;padding:16px}}
.card .label{{color:#94a3b8;font-size:11px;text-transform:uppercase}}
.card .value{{font-size:28px;font-weight:bold;margin-top:2px}}
.chart-card{{background:#1e293b;border-radius:8px;padding:16px;margin-bottom:20px}}
table{{width:100%;border-collapse:collapse;background:#1e293b;border-radius:8px;overflow:hidden;margin-bottom:12px}}
th{{background:#334155;padding:8px 10px;text-align:left;font-size:11px;text-transform:uppercase;color:#94a3b8}}
td{{padding:6px 10px;border-top:1px solid #334155;font-size:13px}}
tr:hover td{{background:#253347}}
code{{background:#334155;padding:2px 6px;border-radius:4px;font-size:12px}}
.updated{{color:#64748b;font-size:11px;margin-top:16px;text-align:right}}
.three-col{{display:grid;grid-template-columns:1fr 1fr 1fr;gap:16px}}
.two-col{{display:grid;grid-template-columns:1fr 1fr;gap:16px}}
.badge{{display:inline-block;padding:2px 8px;border-radius:4px;font-size:10px;font-weight:bold;text-transform:uppercase;margin-right:6px}}
.badge.running{{background:#1e40af;color:#93c5fd}}
.badge.done{{background:#065f46;color:#6ee7b7}}
.badge.idle{{background:#334155;color:#94a3b8}}
@media(max-width:1200px){{.grid{{grid-template-columns:repeat(3,1fr)}}.three-col{{grid-template-columns:1fr}}}}
</style></head><body>
<h1>XFA PDF/A Quality Dashboard {run_badge}</h1>
<div class="subtitle">Curated corpus &middot; veraPDF oracle &middot; auto-refreshes 30s</div>
<div class="grid">
<div class="card"><div class="label">Pass Rate</div><div class="value" style="color:#10b981">{pass_pct}%</div></div>
<div class="card"><div class="label">Pass / Applicable</div><div class="value" style="color:#38bdf8">{p} / {applicable}</div></div>
<div class="card"><div class="label">Failing</div><div class="value" style="color:#ef4444">{f_cnt}</div></div>
<div class="card"><div class="label">Skipped</div><div class="value" style="color:#f59e0b">{latest.get("skip",0)}</div></div>
<div class="card"><div class="label">Trend</div><div class="value" style="color:{delta_color}">{delta_str}</div></div>
</div>
<h2>Pass Rate Trend</h2><div class="chart-card">{chart}</div>
<div class="three-col"><div>
<h2>veraPDF Rule Failures</h2>
<table><thead><tr><th>Rule</th><th>PDFs</th><th></th><th>Description</th></tr></thead>
<tbody>{rule_rows or '<tr><td colspan="4" style="text-align:center;color:#64748b">No failures</td></tr>'}</tbody></table>
</div><div>
<h2>Iteration History ({len(pdfa_iters)} runs)</h2>
<div style="max-height:400px;overflow-y:auto">
<table><thead><tr><th>Run</th><th>Started</th><th>Pass</th><th>Fail</th><th>Skip</th><th>Rate</th></tr></thead>
<tbody>{iter_rows or '<tr><td colspan="6" style="text-align:center;color:#64748b">No data</td></tr>'}</tbody></table></div>
</div><div>
<h2>Skip Reasons</h2>
<table><thead><tr><th>Reason</th><th>Count</th></tr></thead>
<tbody>{skip_rows or '<tr><td colspan="2" style="text-align:center;color:#64748b">No skips</td></tr>'}</tbody></table>
</div></div>
<div class="two-col" style="margin-top:16px"><div>
<h2>GitHub Issues ({open_c} open, {closed_c} closed)</h2>
<div style="max-height:300px;overflow-y:auto">
<table><thead><tr><th>Issue</th><th>Title</th><th>State</th></tr></thead>
<tbody>{gh_rows or '<tr><td colspan="3" style="text-align:center;color:#64748b">No issues</td></tr>'}</tbody></table></div>
</div><div></div></div>
<div class="updated">Updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</div>
</body></html>"""


def build_trend_chart(iterations):
    if not iterations:
        return '<div style="color:#64748b;text-align:center;padding:30px">No data</div>'
    W, H = 900, 180
    PL, PR, PT, PB = 50, 30, 20, 35
    cw, ch = W - PL - PR, H - PT - PB
    rates = [100 * i["pass"] / (i["pass"] + i["fail"]) if (i["pass"] + i["fail"]) > 0 else 0 for i in iterations]
    labels = [i.get("started_at", "")[:10] for i in iterations]
    mn = max(0, min(rates) - 2)
    mx = min(100, max(rates) + 2)
    if mx - mn < 5: mn = max(0, mx - 5)
    rr = mx - mn or 1
    def x(i): return PL + cw / 2 if len(rates) == 1 else PL + (i / (len(rates) - 1)) * cw
    def y(v): return PT + ch - ((v - mn) / rr) * ch
    pts = " ".join(f"{x(i):.1f},{y(r):.1f}" for i, r in enumerate(rates))
    area = f"{x(0):.1f},{PT+ch:.1f} " + pts + f" {x(len(rates)-1):.1f},{PT+ch:.1f}"
    yl = "".join(f'<text x="{PL-8}" y="{y(mn+(i/5)*rr)+4}" text-anchor="end" fill="#64748b" font-size="10">{mn+(i/5)*rr:.0f}%</text><line x1="{PL}" y1="{y(mn+(i/5)*rr)}" x2="{W-PR}" y2="{y(mn+(i/5)*rr)}" stroke="#1e293b"/>' for i in range(6))
    step = max(1, len(labels) // 8)
    xl = "".join(f'<text x="{x(i)}" y="{H-5}" text-anchor="middle" fill="#64748b" font-size="9">{labels[i]}</text>' for i in range(0, len(labels), step))
    dots = "".join(f'<circle cx="{x(i):.1f}" cy="{y(r):.1f}" r="3.5" fill="#10b981" stroke="#0f172a" stroke-width="1.5"><title>Run {i+1}: {r:.1f}%</title></circle>' for i, r in enumerate(rates))
    ann = f'<text x="{x(len(rates)-1)+8}" y="{y(rates[-1])+4}" fill="#10b981" font-size="11" font-weight="bold">{rates[-1]:.1f}%</text>' if rates else ""
    return f'<svg width="100%" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">{yl}{xl}<polygon points="{area}" fill="#10b981" opacity="0.1"/><polyline points="{pts}" fill="none" stroke="#10b981" stroke-width="2" stroke-linejoin="round"/>{dots}{ann}</svg>'


if __name__ == "__main__":
    print(f"Querying VPS {VPS}...")
    data = get_all_data()
    n = len([i for i in data.get("iterations", []) if i.get("total", 0) >= 900])
    print(f"Found {n} full run(s)")
    print(f"Dashboard at http://localhost:{PORT}")
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", PORT), DashboardHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down")
