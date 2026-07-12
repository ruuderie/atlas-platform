# Atlas Platform

**For anyone (human or AI) joining this project:**  
Please start by reading **[`docs/CURRENT_STATE.md`](docs/CURRENT_STATE.md)**. It is the living registry of what is actually built (backend + frontend status for G01–G37+, apps, workers, migrations).

---

This repository contains the Atlas Platform — a multi-tenant application platform built in Rust (Axum + SeaORM) with Leptos frontends. It is designed around strong reuse of **platform generics (G01–G37+)** instead of duplicating vertical-specific tables across applications. Before any net-new table, run **Rule 7** in [`docs/architecture/generic_fitness_test.md`](docs/architecture/generic_fitness_test.md).

## Important Documentation

- **[`docs/CURRENT_STATE.md`](docs/CURRENT_STATE.md)** — Start here. Ground-truth architecture and status (Rev 11+).
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

## Local Development (Docker & Caddy)

The easiest way to develop locally is using the included Docker Compose configuration, which automatically sets up the database, backend, frontends, and a local reverse proxy for simulating multi-tenant networks:

1. Ensure ports 80, 8080, 8081, and 8000 are free.
2. Run standard Docker Compose in the root network:
   ```bash
   docker compose up --build
   ```
3. **Accessing the applications:**
   - **Backend API:** `http://api.localhost`
   - **Platform Admin:** `http://admin.localhost`
   - **Network Instances:** You can simulate any multi-tenant network by navigating to a `.network.localhost` subdomain (e.g., `http://my-first-dir.network.localhost`). The Caddy proxy will automatically route it to the instance, and the app will dynamically fetch the correct configurations!

## Deployment & CI/CD

CI/CD uses Woodpecker (see `docs/cicd_security_hardening.md` and `k8s/`). Pushing builds optimized images, updates manifests, and deploys via the cluster pipeline. Domain provisioning flows through the ingress sidecar + cert-manager (see `docs/architecture/tls_and_custom_domains.md`).

## API & Features

- Dynamic multi-tenant domain routing (Host → AppDomain → tenant)
- Platform generics G01–G37+ (payments, syndication, scorecards, programs, ambassadors, …)
- Folio PM portals, Anchor CMS/CRM, Network Instance marketplace
- Feature flags + per-instance enablements, verification queue, AI task worker

---

&copy; Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.
