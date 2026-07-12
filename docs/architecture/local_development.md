# Local Development (Compose + `atlas-local` CLI)

**Status:** Implemented (July 2026)  
**Primary tool:** [`tools/atlas-local`](../../tools/atlas-local) — Rust / Clap CLI  
**Runtime:** Docker Compose + Caddy (`*.localhost`)  
**Server counterpart:** NixForge (K3s) + Woodpecker — see [`deployment_environments.md`](../deployment_environments.md)

This document is the architecture home for the **local** Atlas loop. Prefer extending `atlas-local` over inventing one-off shell scripts.

---

## Purpose vs server

| Concern | Local (`atlas-local`) | Server (NixForge) |
|---------|----------------------|-------------------|
| Orchestration | Docker Compose | K3s namespaces `atlas-dev` / `atlas-uat` / `atlas-prod` |
| Edge proxy | Caddy on port 80 | Host Nginx → ingress-nginx |
| TLS | Plain HTTP `*.localhost` | cert-manager + Cloudflare |
| Database | Compose Postgres `:5433` | Bare-metal Postgres (`atlas_dev`, …) |
| WebAuthn | `RP_ID=localhost`, `WEBAUTHN_ORIGIN=http://admin.localhost` | `RP_ID=atlas.oply.co` (and tenant eTLD+1) |
| Domain provision | Not applicable (no ingress-sidecar) | ingress-sidecar + cert-manager |

Local is for a fast inner loop. Deployed envs remain the source of truth for TLS, ingress, and production-like WebAuthn.

---

## Quick start

```bash
cd atlas-platform
cargo run -p atlas-local -- --help
cargo run -p atlas-local -- up

# optional install onto PATH
cargo install --path tools/atlas-local
atlas-local up
```

On first `up`, the CLI copies [`.env.local.example`](../../.env.local.example) → `.env.local` (gitignored) and starts Compose with `--env-file .env --env-file .env.local`.

---

## Command map

| Command | Role |
|---------|------|
| `atlas-local up` | Preflight → ensure `.env.local` → `docker compose up` → health wait → URL map |
| `atlas-local down` | Stop stack |
| `atlas-local status` | `docker compose ps` |
| `atlas-local logs [-f] [service]` | Compose logs |
| `atlas-local reset-db` | Wipe Postgres volume + recreate (confirm) |
| `atlas-local db pull --from <dev\|uat\|prod>` | Salesforce-style sandbox: wipe local DB, restore remote dump, re-apply `*.localhost` aliases |

Every subcommand has `--help`. Errors are **problem → fix** (Docker not running, missing env, Postgres unhealthy, prod pull without `--i-understand-pii`, etc.).

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

Unit tests live in [`tools/atlas-local/src/lib.rs`](../../tools/atlas-local/src/lib.rs) (`#[cfg(test)]`) and cover Clap parsing, prod PII gate, repo root discovery, dotenv preference, and orb.local WebAuthn detection.

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
