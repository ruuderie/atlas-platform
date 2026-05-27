# Atlas Platform

**For anyone (human or AI) joining this project:**  
Please start by reading **`docs/CURRENT_STATE.md`**. It contains the most up-to-date high-level overview of the current architecture (including the completed Platform Generics v2 + Legacy CRM Unification effort).

---

This repository contains the Atlas Platform — a multi-tenant application platform built in Rust (Axum + SeaORM) with Leptos SSR frontends. It is designed around strong reuse of 18 platform generics instead of duplicating vertical-specific tables across applications.

## Important Documentation

- **[`docs/CURRENT_STATE.md`](docs/CURRENT_STATE.md)** — Start here. Current architecture, status, and what changed in 2026.
- **[`docs/TEST_ENVIRONMENT_REQUIREMENTS.md`](docs/TEST_ENVIRONMENT_REQUIREMENTS.md)** — How to run the full test suite (PostGIS requirements, etc.).
- `docs/architecture/platform_generics_v2.md` — The authoritative spec for the 18 reusable generics.
- `docs/atlas_app_integration.md` — How new applications integrate with the platform (including the "Generic Fitness Test").

The old README content below is retained for historical context but is no longer the best starting point.

## Project Structure

The project has been migrated to a monorepo workspace containing:

1. **`apps/` (Frontend)**
   - `platform-admin`: A Svelte/Leptos CSR app for platform administration.
   - `network-instance`: A Leptos SSR app that powers the user-facing networks.
   - `shared-ui`: Shared UI components for a consistent design language.
2. **`backend/` (Backend API)**
   - Built using Rust, Axum, and SeaORM.
3. **`k8s/`**
   - Kustomize manifests for seamless Kubernetes deployments.

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

The project includes an enterprise-grade CI/CD pipeline using GitHub Actions (`.github/workflows/deploy.yml`). Pushing to the `main` branch automatically:
1. Builds optimized, multi-stage Docker images.
2. Pushes them to GitHub Container Registry (GHCR).
3. Updates the `k8s/kustomization.yaml` manifests with the fresh commit hashes for continuous delivery to Kubernetes.

## API & Features

- Dynamic Multi-Tenant Domain Routing
- Business Network listings, searching, and Profiles
- CRM, CMS, and Site Settings management for Admins

---

&copy; Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.
