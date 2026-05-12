#!/usr/bin/env python3
"""Tiny static server for the INCIDENT browser build.

Adds the COOP/COEP headers required for SharedArrayBuffer (the worker uses it to
block on stdin), plus the application/wasm mime type. Serves the directory this
file lives in.

    python3 serve.py [port]      # default 8080  ->  http://localhost:8080/
"""
import http.server
import socketserver
import sys
import os

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
os.chdir(os.path.dirname(os.path.abspath(__file__)))


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm"}

    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


with socketserver.ThreadingTCPServer(("", PORT), Handler) as httpd:
    print(f"serving {os.getcwd()} on http://localhost:{PORT}/  (Ctrl-C to stop)")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
