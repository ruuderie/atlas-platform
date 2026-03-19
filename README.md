# Business Directory

This project is a high-performance business directory website using Svelte/Leptos for the frontend and Rust for the backend API. It allows users to view and search for business listings, register, and login. The architecture natively supports multi-tenant directories driven by a single codebase.

## Project Structure

The project has been migrated to a monorepo workspace containing:

1. **`apps/` (Frontend)**
   - `platform-admin`: A Svelte/Leptos CSR app for platform administration.
   - `directory-instance`: A Leptos SSR app that powers the user-facing directories.
   - `shared-ui`: Shared UI components for a consistent design language.
2. **`backend/` (Backend API)**
   - Built using Rust, Axum, and SeaORM.
3. **`k8s/`**
   - Kustomize manifests for seamless Kubernetes deployments.

## Local Development (Docker & Caddy)

The easiest way to develop locally is using the included Docker Compose configuration, which automatically sets up the database, backend, frontends, and a local reverse proxy for simulating multi-tenant directories:

1. Ensure ports 80, 8080, 8081, and 8000 are free.
2. Run standard Docker Compose in the root directory:
   ```bash
   docker compose up --build
   ```
3. **Accessing the applications:**
   - **Backend API:** `http://api.localhost`
   - **Platform Admin:** `http://admin.localhost`
   - **Directory Instances:** You can simulate any multi-tenant directory by navigating to a `.directory.localhost` subdomain (e.g., `http://my-first-dir.directory.localhost`). The Caddy proxy will automatically route it to the instance, and the app will dynamically fetch the correct configurations!

## Deployment & CI/CD

The project includes an enterprise-grade CI/CD pipeline using GitHub Actions (`.github/workflows/deploy.yml`). Pushing to the `main` branch automatically:
1. Builds optimized, multi-stage Docker images.
2. Pushes them to GitHub Container Registry (GHCR).
3. Updates the `k8s/kustomization.yaml` manifests with the fresh commit hashes for continuous delivery to Kubernetes.

## API & Features

- Dynamic Multi-Tenant Domain Routing
- Business Directory listings, searching, and Profiles
- CRM, CMS, and Site Settings management for Admins

---

&copy; Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.
