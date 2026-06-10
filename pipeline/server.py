"""Lightweight HTTP server for the HTML report with API endpoints.

Serves the report from ``output/report/`` and provides:

  GET  /api/specs-toml   — read existing specs.toml (restore selections)
  POST /api/save-specs   — write specs.toml directly (body = TOML text)
"""

import http.server
import json
import socketserver
from pathlib import Path


def serve_report(output_dir: str, port: int = 8765) -> None:
    """Serve the HTML report with API endpoints.

    Press Ctrl+C to stop.
    """
    report_dir = Path(output_dir) / "report"
    if not report_dir.is_dir():
        print(f"Error: report directory not found: {report_dir}")
        print("Run with --report-html first.")
        return

    output = Path(output_dir)

    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=str(report_dir), **kwargs)

        def do_POST(self) -> None:
            if self.path == "/api/save-specs":
                content_length = int(self.headers.get("Content-Length", 0))
                body = self.rfile.read(content_length).decode("utf-8")
                toml_path = output / "specs.toml"
                toml_path.write_text(body)
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(
                    json.dumps(
                        {
                            "ok": True,
                            "path": str(toml_path),
                            "size": len(body),
                        }
                    ).encode()
                )
                print(
                    f"  [api] saved specs.toml ({len(body)} bytes, "
                    f"{body.count(chr(10))} lines)"
                )
                return
            self.send_response(404)
            self.end_headers()

        def do_GET(self) -> None:
            if self.path == "/api/specs-toml":
                toml_path = output / "specs.toml"
                if toml_path.is_file():
                    self.send_response(200)
                    self.send_header("Content-Type", "text/plain; charset=utf-8")
                    self.end_headers()
                    self.wfile.write(toml_path.read_bytes())
                else:
                    self.send_response(404)
                    self.end_headers()
                return
            super().do_GET()

        def log_message(self, format, *args) -> None:
            # Suppress default access logs for API calls
            if "/api/" in str(args[0]):
                return
            super().log_message(format, *args)

    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", port), Handler) as httpd:
        print(f"\n  ═══════════════════════════════════════════")
        print(f"  Report: http://localhost:{port}/")
        print(f"  API:    http://localhost:{port}/api/save-specs  (POST)")
        print(f"  API:    http://localhost:{port}/api/specs-toml  (GET)")
        print(f"  ═══════════════════════════════════════════")
        print(f"  Press Ctrl+C to stop.")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n  Server stopped.")
