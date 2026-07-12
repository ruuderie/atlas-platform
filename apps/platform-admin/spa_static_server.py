#!/usr/bin/env python3
"""Minimal SPA static server for platform-admin development dist/."""
from __future__ import annotations

import mimetypes
import os
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def main() -> None:
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} PORT DIST_DIR", file=sys.stderr)
        sys.exit(2)
    port = int(sys.argv[1])
    dist = Path(sys.argv[2]).resolve()
    os.chdir(dist)

    class SpaHandler(SimpleHTTPRequestHandler):
        def send_head(self):  # type: ignore[override]
            path = self.translate_path(self.path.split("?", 1)[0].split("#", 1)[0])
            # Asset or existing file → serve as-is
            if os.path.isdir(path):
                path = os.path.join(path, "index.html")
            if not os.path.exists(path) or os.path.isdir(path):
                # SPA fallback
                self.path = "/index.html"
            return SimpleHTTPRequestHandler.send_head(self)

        def log_message(self, fmt: str, *args) -> None:
            sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

    mimetypes.add_type("application/wasm", ".wasm")
    httpd = ThreadingHTTPServer(("0.0.0.0", port), SpaHandler)
    print(f"→ SPA static server on 0.0.0.0:{port} root={dist}", flush=True)
    httpd.serve_forever()


if __name__ == "__main__":
    main()
