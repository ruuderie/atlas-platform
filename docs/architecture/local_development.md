# Local Development (Compose + `atlas-local` CLI)

**Status:** Implemented (July 2026)  
**Primary tool:** [`tools/atlas-local`](../../tools/atlas-local) — Rust / Clap CLI  
**Runtime:** Docker Compose + Caddy (`*.localhost`)  
**Server counterpart:** NixForge (K3s) + Woodpecker — see [`deployment_environments.md`](../deployment_environments.md)

This document is the architecture home for the **local** Atlas loop. Prefer extending `atlas-local` over inventing one-off shell scripts.

---

## Purpose vs server

| Concern | Local **parity** (`atlas-local up`) | Local **hot** (`up --hot`) | Server (NixForge) |
|---------|-------------------------------------|----------------------------|-------------------|
| Backend process | Baked `./atlas_backend` binary | `cargo run` (compile then listen) | Baked image binary |
| Boot to `/health` | Seconds after image exists | Often many minutes on cold compile | Seconds |
| Schema / migrations | Run immediately on start | Delayed until compile finishes | Immediate |
| Confidence vs server | **High** (same runtime shape) | Lower (dev loop convenience) | Source of truth |
| Orchestration | Docker Compose | Docker Compose + `docker-compose.hot.yml` | K3s |
| Edge / TLS / WebAuthn | Caddy `*.localhost`, `RP_ID=localhost` | same | Nginx + cert-manager + real RP_ID |

**Reconciliation rule:** treat **`origin/dev` (and deployed envs)** as source of truth. Default local to **parity** so “works on server / fails locally” is not a `cargo run` compile wait mistaken for app instability. Use `--hot` only when you need volume-mounted Rust iteration.

### Stale frontend builds (never again)

Trunk/`cargo-leptos` write a gitignored `apps/*/dist/`. Compose mounts the app directory into the container, so an **old `dist/` on the host will be served forever** even when `src/` matches `origin/dev` (this is why `admin.localhost` once showed a June “Intelligence Layer” login while the server showed the current Admin Sign In).

Mitigations (all required):

1. **`atlas-local up` / `refresh`** wipe `apps/*/dist`, then run **host `trunk build`** for platform-admin when `trunk` is on `PATH` (in-container `rust-lld` often SIGSEGVs on this WASM).
2. **platform-admin development entrypoint** refuses the known-stale hash `c0b757e5046922dd` and serves a current `dist/` via an SPA static server (not a months-old Trunk cache).
3. Do not commit or restore `dist/` from backups into the app tree.

After a refresh, expect the first host Trunk compile to take several minutes — that is the correct cost of a current build. Hard-refresh the browser afterward.

The `tenant` / `app_domains` “relation does not exist” errors during a hung `up` were **status probes querying before migrations** while the backend was still compiling — not a server-vs-local schema drift.

---

## Quick start

```bash
cd atlas-platform
cargo run -p atlas-local -- up          # PARITY — preferred
# after code changes:
cargo run -p atlas-local -- refresh backend

# local secrets / SMTP (writes .env.local — gitignored):
cargo run -p atlas-local -- env smtp
cargo run -p atlas-local -- env set SMTP_SERVER=smtp.example.com
cargo run -p atlas-local -- env edit
# then: cargo run -p atlas-local -- refresh backend

# optional hot-reload loop (diverges from server):
cargo run -p atlas-local -- up --hot
cargo run -p atlas-local -- watch
```

On first `up`, the CLI copies [`.env.local.example`](../../.env.local.example) → `.env.local` (gitignored) and starts Compose with `--env-file .env --env-file .env.local`.

---

## Command map

| Command | Role |
|---------|------|
| `atlas-local up` | **Parity** stack (baked backend ≈ K8s) |
| `atlas-local up --hot` | Hot overlay: mounts + `cargo run` (slow first boot) |
| `atlas-local services` | List Compose services from `docker-compose.yml` (+ `apps/` dirs) |
| `atlas-local refresh [services…]` | One-shot recreate so containers **match your latest saves** |
| `atlas-local watch` | Compose Watch (implies hot mode) |
| `atlas-local down` | Stop stack |
| `atlas-local status` | Ratatui dashboard (Overview · Capacity · Telemetry · **Env**) with **Next steps** (copy-paste CLI). `--plain` for text |
| `atlas-local status --plain` | Same report without TUI (CI / pipes) |
| `atlas-local logs [-f] [service]` | Compose logs |
| `atlas-local reset-db` | Wipe Postgres volume + recreate (confirm) |
| `atlas-local db info` | Print Host/Port/User/Password/URL for DBeaver (and friends) |
| `atlas-local db pull --from <dev\|uat\|prod>` | Salesforce-style sandbox pull |
| `atlas-local env list\|get\|set\|unset\|edit\|path` | Manage gitignored `.env.local` |
| `atlas-local env smtp` | SMTP status (mock vs configured) + set template |

### Local env / SMTP

Compose loads `.env` then `.env.local` (local wins). Prefer **`atlas-local env`** over hand-editing when possible:

```bash
cargo run -p atlas-local -- env smtp                 # mock vs real
cargo run -p atlas-local -- env set SMTP_SERVER=smtp.example.com
cargo run -p atlas-local -- env set SMTP_PORT=587 SMTP_USERNAME=u SMTP_TOKEN=secret
cargo run -p atlas-local -- env set SMTP_FROM='Atlas <noreply@example.com>'
cargo run -p atlas-local -- refresh backend          # containers must reload env
```

Empty or `localhost` `SMTP_SERVER` → backend **mocks** email (logs only). That is why local magic links often “don’t send.”

From **`atlas-local status` → tab 4 Env**: `s` SMTP form (writes `.env.local`), **`a` apply** (recreates backend so the running process picks up env), `e` open editor. **Set without apply does not change the live app.**

### Status dashboard

| Tab | Contents |
|-----|----------|
| **1 Overview** | System, domains, HTTP latency, DB, **Next steps** (shows what **`x`** will refresh) |
| **2 Capacity** | Application KPIs (tenants / domains / DB / sessions — same vocabulary as Platform Admin System Status) + host stack load (CPU/RAM), container stats, images, volumes |
| **3 Telemetry** | Sparklines + polled `/metrics`, `request_log`, `telemetry_events` |
| **4 Env** | SMTP mock vs configured, local overlay keys; set/apply from the TUI |

| Key | Action |
|-----|--------|
| **`x`** | Run the first Next-steps `refresh <services…>` (affected apps only, e.g. `network-instance anchor`) |
| **`r`** | Reload the status panel only — does **not** recreate containers |
| `q` | Quit |
| `1`–`4` / tab | Switch tabs |
| Env: `s` / `a` / `e` | SMTP form / apply `.env.local` to backend / open editor |

Panel auto-refreshes every 3s. CLI equivalent of `x`: `cargo run -p atlas-local -- refresh <services…>`.

When something fails, **Next steps** picks commands from stack state (down → `up`; unhealthy → `logs`/`refresh`; schema missing → wait/`reset-db`; ready → `refresh` after edits). Recovery ladder: `refresh` → `down && up` → `reset-db` → `up`.

### Edit → see cycle

| Mode | After you edit |
|------|----------------|
| Parity (default) | `atlas-local refresh` / `refresh backend` (rebuild image) |
| Hot | `atlas-local watch` or `refresh` |

Every subcommand has `--help` (and many have longer `--help` / `long_about`). Errors are **problem → fix** with concrete commands.

---

## Host / tenant routing

Caddy ([`Caddyfile`](../../Caddyfile)) wildcards:

| Pattern | Service |
|---------|---------|
| `api.localhost` | backend |
| `admin.localhost` | platform-admin |
| `*.network.localhost` | network-instance |
| `*.folio.localhost`, `folio.localhost` | folio |
| `*.anchor.localhost`, `anchor.localhost` | anchor |
| Convenience | `buildwithruud.localhost`, `oplystusa.localhost`, `directory.network.localhost`, `ruuderie.localhost` |

Migration [`m20261101_seed_local_dev_domain_aliases`](../../backend/src/migration/m20261101_seed_local_dev_domain_aliases.rs) inserts matching `app_domains` rows for seeded tenants. After `db pull`, the CLI re-applies the same aliases.

**Provision a new tenant locally:** use a domain under a wildcard, e.g. `acme.anchor.localhost`. DNS/ingress verification is bypassed when `ENVIRONMENT=development`. The tenant exists only in the local DB until you provision (or pull) on the server.

---

## WebAuthn / cookies (isolation rules)

1. Local WebAuthn lives **only** in `.env.local` (`WEBAUTHN_ORIGIN=http://admin.localhost`, `RP_ID=localhost`).
2. **Never** copy those values into `k8s/overlays/{dev,uat,prod}/config.yaml`.
3. Session / passkey cookies omit `Secure` when `ENVIRONMENT` is `development` / `dev` / `local` so HTTP localhost works. Server envs keep `Secure`.
4. Passkeys registered on the server will **not** work locally (and vice versa). That is intentional.

---

## Sandbox DB pull

```bash
export ATLAS_DEV_DATABASE_URL='postgresql://…'   # often via SSH tunnel to NixForge
atlas-local db pull --from dev
atlas-local db pull --from prod --i-understand-pii   # discouraged; PII
```

Requires `pg_dump` / `pg_restore` / `psql` on the host. This is a full replace snapshot, not live replication.

---

## CI advisory (non-blocking)

After a **successful** Woodpecker deploy to `dev` / `uat`, the pipeline runs `validate_atlas_local_cli`:

- `cargo test -p atlas-local`
- build + `atlas-local --help` / `db pull --help` smoke

This step uses **`failure: ignore`**: a broken CLI must **never** fail or roll back a deploy. It only makes the problem visible in the pipeline UI / logs. Fix the CLI on a follow-up commit; do not weaken this to a blocking gate without an explicit product decision.

Woodpecker **Telegram + email** notifications include a short `atlas-local` cheat sheet (**parity `up`**, `status` Next steps, `refresh`, `db info`). Keep that copy in [`.woodpecker.yml`](../../.woodpecker.yml) in sync when commands change.

Unit tests live in [`tools/atlas-local/src/lib.rs`](../../tools/atlas-local/src/lib.rs) (`#[cfg(test)]`) and cover Clap parsing, prod PII gate, repo root discovery, dotenv preference, and orb.local WebAuthn detection.

---

## Platform Admin System Status (deploy-safe)

Local Compose/Docker detail stays on the host (`atlas-local status`). On **dev / UAT / production**, operators use **Operations → System Status** in platform-admin (`/ops/status`):

| Concern | `atlas-local status` (host) | Admin System Status (SPA) |
| --- | --- | --- |
| Audience | Developer laptop | Super-admins on any env |
| Docker / compose / host CPU | Yes | Never |
| DB passwords / JDBC | Yes (local only) | Never |
| Env → Tenant → App → Domain tree | Flat domain sample | Hierarchical blast-radius tree |
| Health / version | HTTP probes + `/health` | Same shape via `GET /api/admin/system-status` → `fleet` + `environments[]` |
| Application capacity KPIs | Capacity tab (tenants / domains / DB / sessions) | Capacity tab: fleet totals + selected-env resources |
| Prometheus | Scrapes `/metrics` with `METRICS_TOKEN` | In-process aggregates; token never in browser |
| Next steps | CLI recovery ladder (kept) | Removed from deploy UI (not useful remotely) |

Backend: [`admin/system_status.rs`](../../backend/src/admin/system_status.rs) — `PlatformSuperAdmin` session required. Frontend: [`pages/ops/system_status.rs`](../../apps/platform-admin/src/pages/ops/system_status.rs).

---

## Extension policy (required reading for future work)

**Any new local-dev automation** (seeds, fixtures, smoke checks, tenant bootstrap helpers, log diagnostics, PostGIS checks, network seed packs) **lands as an `atlas-local` subcommand or module under [`tools/atlas-local`](../../tools/atlas-local)** — not a standalone `scripts/*.sh` — unless the PR documents a strong reason otherwise.

Suggested future subcommands (also listed in `CURRENT_STATE` follow-ups):

- `atlas-local seed network` — run `seed_db` / Network seed packs
- `atlas-local smoke` — hit `/api/health` + key frontends
- `atlas-local doctor` — PostGIS / port / WebAuthn env diagnostics

When you notice a repeated local ops pain during feature work, **surface it by extending this CLI** and update this doc + the Infrastructure row in [`CURRENT_STATE.md`](../CURRENT_STATE.md).

---

## Related docs

- [`CURRENT_STATE.md`](../CURRENT_STATE.md) — ground-truth registry (includes `atlas-local` row)
- [`deployment_environments.md`](../deployment_environments.md) — DEV/UAT/PROD matrix
- [`tls_and_custom_domains.md`](tls_and_custom_domains.md) — server TLS (not used locally)
- NixForge repo — bare-metal Postgres, K3s, Woodpecker host
