# Atlas Platform

**For anyone (human or AI) joining this project:**  
Please start by reading **[`docs/CURRENT_STATE.md`](docs/CURRENT_STATE.md)**. It is the living registry of what is actually built (backend + frontend status for G01–G37+, apps, workers, migrations).

---

This repository contains the Atlas Platform — a multi-tenant application platform built in Rust (Axum + SeaORM) with Leptos frontends. It is designed around strong reuse of **platform generics (G01–G37+)** instead of duplicating vertical-specific tables across applications. Before any net-new table, run **Rule 7** in [`docs/architecture/generic_fitness_test.md`](docs/architecture/generic_fitness_test.md).

## Important Documentation

- **[`docs/CURRENT_STATE.md`](docs/CURRENT_STATE.md)** — Start here. Ground-truth architecture and status (Rev 11+).
- **[`docs/architecture/local_development.md`](docs/architecture/local_development.md)** — `atlas-local` CLI, Compose/Caddy, WebAuthn isolation, sandbox `db pull`, CLI extension policy.
- **[`docs/architecture/generic_fitness_test.md`](docs/architecture/generic_fitness_test.md)** — Rule 7 before new tables / G-numbers.
- **[`docs/architecture/platform_generics_v3.md`](docs/architecture/platform_generics_v3.md)** — Canonical generics design doc (G01–G37+). Status columns must match CURRENT_STATE.
- [`docs/architecture/platform_generics_v2.md`](docs/architecture/platform_generics_v2.md) — Historical G01–G31 design record (superseded by v3 for G32+).
- **[`docs/TEST_ENVIRONMENT_REQUIREMENTS.md`](docs/TEST_ENVIRONMENT_REQUIREMENTS.md)** — How to run the full test suite (PostGIS requirements, etc.).
- [`docs/atlas_app_integration.md`](docs/atlas_app_integration.md) — How new applications integrate (`AtlasApp` trait + Fitness Test).
- [`docs/backlog/README.md`](docs/backlog/README.md) — Known gaps and future work (not yet in schema/code).

## Project Structure

Monorepo workspace:

1. **`apps/` (Frontend)**
   - `anchor` — Leptos SSR+WASM CMS / listings / CRM
   - `folio` — Leptos SSR+WASM property management (9 role portals)
   - `network-instance` — Leptos SSR+WASM multi-tenant marketplace / directory
   - `platform-admin` — Leptos CSR operator console
   - `shared-ui` — Shared primitives (85+) + G-27 Configurator / scorecard widgets
2. **`backend/` (Backend API)**
   - Rust, Axum, SeaORM — headless REST + Folio/PM handlers + workers
3. **`k8s/`**
   - Kustomize manifests for Kubernetes deployments

## Local Development (`atlas-local`)

Use the Rust CLI — do **not** invent one-off shell scripts for local ops (extend `tools/atlas-local` instead). Architecture: [`docs/architecture/local_development.md`](docs/architecture/local_development.md).

**Parity by default** (baked backend ≈ K8s). Use `--hot` only for volume-mounted cargo iteration.

```bash
cargo run -p atlas-local -- --help
cargo run -p atlas-local -- up                 # PARITY — preferred
cargo run -p atlas-local -- status             # dashboard + Next steps
cargo run -p atlas-local -- refresh backend    # after code changes (parity)
cargo run -p atlas-local -- env smtp           # why magic links don't send
cargo run -p atlas-local -- env set KEY=value  # writes .env.local
cargo run -p atlas-local -- db info            # DBeaver: 127.0.0.1:5433

# optional hot loop (diverges from server; slow cold boot):
# cargo run -p atlas-local -- up --hot && cargo run -p atlas-local -- watch

# optional: cargo install --path tools/atlas-local
atlas-local up
```

**Stuck?** `atlas-local status` prints copy-paste Next steps (`refresh` → `down && up` → `reset-db`).
**URLs (after `up`):**

| App | URL |
|-----|-----|
| Backend API | `http://api.localhost` |
| Platform Admin | `http://admin.localhost` |
| Network | `http://directory.network.localhost` (or any `*.network.localhost`) |
| Folio | `http://folio.localhost` |
| Anchor | `http://buildwithruud.localhost` / `http://oplystusa.localhost` |

Local WebAuthn uses `RP_ID=localhost` (via `.env.local`). Register a **new** local passkey; server passkeys will not work here. Never copy local WebAuthn env into K8s overlays.

## Deployment & CI/CD

CI/CD uses Woodpecker (see `docs/cicd_security_hardening.md` and `k8s/`). Pushing builds optimized images, updates manifests, and deploys via the cluster pipeline. Domain provisioning flows through the ingress sidecar + cert-manager (see `docs/architecture/tls_and_custom_domains.md`).

## API & Features

- Dynamic multi-tenant domain routing (Host → AppDomain → tenant)
- Platform generics G01–G37+ (payments, syndication, scorecards, programs, ambassadors, …)
- Folio PM portals, Anchor CMS/CRM, Network Instance marketplace
- Feature flags + per-instance enablements, verification queue, AI task worker

---

&copy; Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.
