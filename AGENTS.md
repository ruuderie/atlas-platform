# AGENTS.md

## Cursor Cloud specific instructions

This repo is a Rust monorepo (multi-tenant business-network SaaS). The README documents a
Docker Compose dev flow; in the Cursor Cloud VM we instead run the services **natively**
(no Docker) because the toolchain, Postgres, and proxy are pre-installed into the VM
snapshot. The update script only pre-fetches Cargo dependencies — everything below is the
durable, non-obvious context for actually running things.

### Services (core product) and how to run them

| Service | Path | Dev command | Port |
|---|---|---|---|
| backend (Axum + SeaORM API) | `backend/` | `cargo run` (from `backend/`) | 8000 |
| platform-admin (Leptos CSR/WASM) | `apps/platform-admin/` | `env -u NO_COLOR trunk serve --port 8081 --address 0.0.0.0` | 8081 |
| network-instance (Leptos SSR) | `apps/network-instance/` | `env -u NO_COLOR cargo-leptos watch` | 8080 |
| proxy (Caddy, `.localhost` routing) | root `Caddyfile` | see "Caddy" below | 80 |

`apps/anchor/` is a separate, optional secondary app (edition 2021, **nightly** toolchain,
Leptos 0.6, its own Postgres) — not part of the core product and not needed to run the above.

### Non-obvious gotchas

- **`NO_COLOR=1` breaks `trunk` and `cargo-leptos`.** The VM sets `NO_COLOR=1`, but these
  tools parse `NO_COLOR` as a boolean and error with `invalid value '1' for '--no-color'`.
  Always prefix their invocations with `env -u NO_COLOR` (e.g. `env -u NO_COLOR trunk build`).
  Plain `cargo`/`rustc` are unaffected.
- **Postgres is not auto-started.** The snapshot has PostgreSQL 16 installed with the
  `oplydb` / `oplydbtest` databases already created (owner `postgres`, password `postgres`).
  Start it each session with `sudo pg_ctlcluster 16 main start` (listens on `localhost:5432`).
  If the databases are ever missing: `sudo -u postgres createdb -O postgres oplydb` (and `oplydbtest`).
- **`.env` is required and gitignored.** The root `docker-compose.yml`, `init-scripts/`, and
  the backend all read a root `.env`. If it is missing, recreate it with at least:
  `DATABASE_URL=postgresql://postgres:postgres@localhost:5432/oplydb`, `ADMIN_USER=admin@oply.co`,
  `ADMIN_PASSWORD=Admin123!`, `ADMIN_FIRST_NAME`, `ADMIN_LAST_NAME`, `ADMIN_PHONE`,
  `ENVIRONMENT=development`, `FRONTEND_URL=http://localhost:8080`, `ADMIN_URL=http://localhost:8081`,
  `PGUSER=postgres`, `PGPASSWORD=postgres`, `PGDB=oplydb`. The backend needs the three
  `ADMIN_*` name/phone vars because `CREATE_ADMIN_ON_STARTUP` defaults to `true` and
  `admin/setup.rs` `.unwrap()`s them. When running the backend, load the file first:
  `set -a && source ../.env && set +a && cargo run`.
- **The backend auto-migrates and bootstraps the admin on every startup**, so no manual
  migration step is needed for local dev.
- **No `Cargo.lock` is committed** (gitignored for both `backend/` and `apps/`), so builds
  resolve to the newest semver-compatible deps each time. First native build of each crate is
  slow (a few minutes) due to heavy deps (aws-sdk-s3, async-stripe, Leptos/WASM).

### Auth / how to actually log in (hello-world)

- The platform-admin WASM app talks to the backend at `http://api.localhost`, so it needs Caddy
  running (see below).
- Admin auth is **passwordless magic-link**: submitting the admin email issues a single-use token
  and triggers an email send. With no SMTP configured the send is mocked and logged in the backend
  as `SMTP Host not configured. Mocking email send to: <email>`; the token is retrievable from the
  backend logs / `magic_link` table. This UI request (WASM → Caddy → backend → Postgres outbox) is a
  good hello-world even without completing the handshake.
- The backend also exposes a direct JSON login for scripting; on `dev` it returns HTTP 200 and
  delivers the JWT via an **httpOnly `session` cookie** (the token is no longer in the body):
  `curl -i -X POST http://localhost:8000/login -H 'Content-Type: application/json' -d '{"email":"admin@oply.co","password":"Admin123!"}'`
- **Completing the magic-link handshake in a browser needs HTTPS.** The session cookie is issued
  `Secure`, so Chrome silently drops it over plain `http://admin.localhost` and the verify-token
  page reports "Handshake failed". The backend side works (curl above shows the `Set-Cookie`). Serve
  the `.localhost` hosts over TLS (Caddy can, via its internal CA) if you need full in-browser login.
- On `dev` the admin app gates on a first-run setup that expects a `user_account` with role
  `PlatformSuperAdmin` for the admin user; the plain admin bootstrap does not create it.

### Caddy (`.localhost` routing)

The committed `Caddyfile` targets Docker service names. For native dev, run Caddy against a
local config that points at `127.0.0.1` and start it with `sudo` (binds port 80):

```
# /tmp/Caddyfile.local
http://api.localhost   { reverse_proxy 127.0.0.1:8000 }
http://admin.localhost { reverse_proxy 127.0.0.1:8081 }
http://*.network.localhost, http://network.localhost { reverse_proxy 127.0.0.1:8080 }
```
`sudo caddy run --config /tmp/Caddyfile.local --adapter caddyfile`

### Lint / test / build

- Backend tests: `cargo test` (from `backend/`; uses in-process sqlx-sqlite, no external DB needed).
  On `dev` this is ~970 tests and they pass. Build: `cargo build`.
- Backend lint: `cargo clippy`. NOTE: on `dev` clippy currently **fails** (exit 101) on a
  pre-existing, deny-by-default lint — `clippy::deprecated_semver` at
  `src/services/lead_billing.rs` (`#[deprecated(since = "2026-06-01", ...)]` is not a semver
  version). `cargo build` is unaffected; this is a source-code issue, not an environment problem.
- Frontends build with `env -u NO_COLOR trunk build` (platform-admin) and
  `env -u NO_COLOR cargo-leptos build` (network-instance).

### Notes on `dev` app state (not env issues)

- `dev` is the active development branch and moves fast; some flows are mid-refactor:
  - `apps/network-instance` builds and serves fine on `dev` (SSR HTML renders), but a tenant page
    shows "Connection Dropped" because the SSR calls `GET {api}/networks/lookup?domain=<host>`
    while the `dev` backend exposes `GET /tenants/lookup` — a frontend/backend route mismatch.
  - The admin browser login handshake needs HTTPS (see Auth section).
  - `dev` seeds several tenants (e.g. `directory.localhost`, `ct-build-pros.oply.co`) in the
    `app_domains` table; the network-instance resolves the tenant from the incoming `Host` header.
