#!/usr/bin/env python3
"""Simple dashboard server for XFA orchestrator.
Run: python3 scripts/dashboard-server.py
Open: http://localhost:8787
"""
import http.server
import json
import os
import socketserver
from datetime import datetime

ORCH_DIR = "/tmp/xfa-orchestrator"
PORT = 8787


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
            self.wfile.write(json.dumps(get_status()).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass  # Suppress logging


def get_iterations():
    """Load all iteration metadata."""
    iters = []
    iter_dir = os.path.join(ORCH_DIR, "iterations")
    if not os.path.exists(iter_dir):
        return iters
    for d in sorted(os.listdir(iter_dir)):
        meta_file = os.path.join(iter_dir, d, "meta.json")
        if os.path.exists(meta_file):
            iters.append(json.load(open(meta_file)))
    return iters


def get_tasks():
    """Load all current tasks."""
    tasks = []
    tasks_dir = os.path.join(ORCH_DIR, "tasks")
    if not os.path.exists(tasks_dir):
        return tasks
    for f in sorted(os.listdir(tasks_dir)):
        if f.endswith(".json"):
            tasks.append(json.load(open(os.path.join(tasks_dir, f))))
    return tasks


def get_workers():
    """Load worker statuses."""
    workers = []
    workers_dir = os.path.join(ORCH_DIR, "workers")
    if not os.path.exists(workers_dir):
        return workers
    for f in sorted(os.listdir(workers_dir)):
        if f.endswith(".json"):
            workers.append(json.load(open(os.path.join(workers_dir, f))))
    return workers


def get_status():
    return {
        "iterations": get_iterations(),
        "tasks": get_tasks(),
        "workers": get_workers(),
        "updated_at": datetime.now().isoformat(),
    }


def generate_dashboard():
    iterations = get_iterations()
    tasks = get_tasks()
    workers = get_workers()

    # Build iteration chart data
    iter_labels = [f"Iter {i.get('iteration', '?')}" for i in iterations]
    iter_pass = [i.get("pass", 0) for i in iterations]
    iter_fail = [i.get("fail", 0) for i in iterations]

    # Task stats
    task_open = sum(1 for t in tasks if t.get("status") == "open")
    task_progress = sum(1 for t in tasks if t.get("status") == "in_progress")
    task_done = sum(1 for t in tasks if t.get("status") == "done")
    task_failed = sum(1 for t in tasks if t.get("status") == "failed")

    # Build task rows
    task_rows = ""
    status_colors = {
        "open": "#6b7280",
        "in_progress": "#f59e0b",
        "done": "#10b981",
        "failed": "#ef4444",
    }
    for t in tasks:
        status = t.get("status", "open")
        color = status_colors.get(status, "#6b7280")
        task_rows += f"""
        <tr>
            <td><code>{t.get('id','?')}</code></td>
            <td>{t.get('domain','?')}</td>
            <td>{t.get('pdf_count',0)}</td>
            <td>{len(t.get('verapdf_rules',[]))}</td>
            <td><span style="color:{color};font-weight:bold">{status.upper()}</span></td>
            <td>{t.get('assigned_to','—')}</td>
        </tr>"""

    # Build iteration rows
    iter_rows = ""
    for i in reversed(iterations):
        total = i.get("total", 1000)
        p = i.get("pass", 0)
        pct = f"{100*p/total:.1f}%" if total > 0 else "—"
        iter_rows += f"""
        <tr>
            <td>Iter {i.get('iteration','?')}</td>
            <td>{i.get('timestamp','')[:16]}</td>
            <td style="color:#10b981;font-weight:bold">{p}</td>
            <td style="color:#ef4444">{i.get('fail',0)}</td>
            <td>{i.get('skip',0)}</td>
            <td>{pct}</td>
            <td>{i.get('cluster_count',0)}</td>
        </tr>"""

    # Worker rows
    worker_rows = ""
    for w in workers:
        status = w.get("status", "idle")
        color = "#10b981" if status == "working" else "#6b7280"
        worker_rows += f"""
        <tr>
            <td>{w.get('id','?')}</td>
            <td><span style="color:{color}">{status}</span></td>
            <td>{w.get('current_task','—')}</td>
            <td>{w.get('tasks_completed',0)}</td>
            <td>{w.get('last_active','—')[:16] if w.get('last_active') else '—'}</td>
        </tr>"""

    latest = iterations[-1] if iterations else {}
    current_pass = latest.get("pass", 0)
    current_fail = latest.get("fail", 0)
    current_total = latest.get("total", 1000)
    pass_pct = f"{100*current_pass/current_total:.1f}" if current_total > 0 else "0"

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="refresh" content="30">
<title>XFA PDF/A Quality Dashboard</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
         background: #0f172a; color: #e2e8f0; padding: 20px; }}
  h1 {{ color: #38bdf8; margin-bottom: 20px; }}
  h2 {{ color: #94a3b8; margin: 20px 0 10px; font-size: 16px; text-transform: uppercase; }}
  .grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }}
  .card {{ background: #1e293b; border-radius: 8px; padding: 20px; }}
  .card .label {{ color: #94a3b8; font-size: 12px; text-transform: uppercase; }}
  .card .value {{ font-size: 32px; font-weight: bold; margin-top: 4px; }}
  .card .value.green {{ color: #10b981; }}
  .card .value.red {{ color: #ef4444; }}
  .card .value.yellow {{ color: #f59e0b; }}
  .card .value.blue {{ color: #38bdf8; }}
  table {{ width: 100%; border-collapse: collapse; background: #1e293b; border-radius: 8px; overflow: hidden; }}
  th {{ background: #334155; padding: 10px 12px; text-align: left; font-size: 12px;
       text-transform: uppercase; color: #94a3b8; }}
  td {{ padding: 8px 12px; border-top: 1px solid #334155; font-size: 14px; }}
  tr:hover td {{ background: #253347; }}
  code {{ background: #334155; padding: 2px 6px; border-radius: 4px; font-size: 12px; }}
  .updated {{ color: #64748b; font-size: 12px; margin-top: 20px; text-align: right; }}
</style>
</head>
<body>
<h1>XFA PDF/A Quality Dashboard</h1>

<div class="grid">
  <div class="card">
    <div class="label">Pass Rate</div>
    <div class="value green">{pass_pct}%</div>
  </div>
  <div class="card">
    <div class="label">Passing / Total</div>
    <div class="value blue">{current_pass} / {current_total}</div>
  </div>
  <div class="card">
    <div class="label">Tasks</div>
    <div class="value yellow">{task_open} open / {task_progress} active / {task_done} done</div>
  </div>
  <div class="card">
    <div class="label">Iterations</div>
    <div class="value">{len(iterations)}</div>
  </div>
</div>

<h2>Iteration History</h2>
<table>
<thead><tr><th>Iteration</th><th>Timestamp</th><th>Pass</th><th>Fail</th><th>Skip</th><th>Rate</th><th>Clusters</th></tr></thead>
<tbody>{iter_rows if iter_rows else '<tr><td colspan="7" style="text-align:center;color:#64748b">No iterations yet</td></tr>'}</tbody>
</table>

<h2>Current Tasks</h2>
<table>
<thead><tr><th>ID</th><th>Domain</th><th>PDFs</th><th>Rules</th><th>Status</th><th>Assigned</th></tr></thead>
<tbody>{task_rows if task_rows else '<tr><td colspan="6" style="text-align:center;color:#64748b">No tasks yet</td></tr>'}</tbody>
</table>

<h2>Workers</h2>
<table>
<thead><tr><th>Worker</th><th>Status</th><th>Current Task</th><th>Completed</th><th>Last Active</th></tr></thead>
<tbody>{worker_rows if worker_rows else '<tr><td colspan="5" style="text-align:center;color:#64748b">No workers registered</td></tr>'}</tbody>
</table>

<div class="updated">Last updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} (auto-refreshes every 30s)</div>
</body>
</html>"""


if __name__ == "__main__":
    with socketserver.TCPServer(("", PORT), DashboardHandler) as httpd:
        print(f"Dashboard running at http://localhost:{PORT}")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down")
