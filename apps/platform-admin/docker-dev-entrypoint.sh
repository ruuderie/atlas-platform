#!/bin/sh
# Development entrypoint for platform-admin.
#
# Problem: host-mounted gitignored dist/ can shadow current sources forever
# (admin.localhost once served June WASM c0b757e5046922dd while src matched origin/dev).
#
# Problem 2: `trunk serve` inside this container often SIGSEGVs in rust-lld on
# large WASM links. Prefer a host `trunk build` (atlas-local does this after wipe)
# and serve the resulting dist statically with SPA fallback.
set -eu

cd /app

is_stale_dist() {
  ls /app/dist/atlas_platform_admin-c0b757e5046922dd* >/dev/null 2>&1
}

has_usable_dist() {
  [ -f /app/dist/index.html ] && ls /app/dist/atlas_platform_admin-*_bg.wasm >/dev/null 2>&1 && ! is_stale_dist
}

if is_stale_dist || [ "${ATLAS_WIPE_FRONTEND_DIST:-0}" = "1" ]; then
  echo "→ platform-admin: wiping stale/forced dist/"
  rm -rf /app/dist
fi

if has_usable_dist; then
  echo "→ platform-admin: serving current dist/ (SPA) — matches host trunk build / origin/dev sources"
  exec python3 /spa_static_server.py 8081 /app/dist
fi

echo "→ platform-admin: no usable dist — running trunk serve (may be slow; prefer host: trunk build)"
exec trunk serve --port 8081 --address 0.0.0.0
