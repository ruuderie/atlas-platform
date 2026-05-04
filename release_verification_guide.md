# Release & Version Verification Guide

This guide explains how the Atlas Platform is versioned during CI/CD and how operators can independently verify exactly what code is currently running in any environment (Production, UAT, etc.).

## 1. How Versioning Works

The Atlas Platform uses a composite versioning strategy that combines semantic versioning with deterministic Git state. This ensures we always know exactly which commit is running in a container.

During the CI/CD pipeline (e.g., GitHub Actions / Woodpecker), the following environment variables are injected into the Rust compiler at build-time using `option_env!`:

1. **`CARGO_PKG_VERSION`**: The Semantic Version (e.g., `0.9.1`) defined in the `Cargo.toml` workspace.
2. **`ATLAS_BUILD_SHA`**: The exact Git commit SHA that triggered the build (e.g., `f0e7c167`).
3. **`ATLAS_BUILD_DATE`**: The UTC timestamp of the build (e.g., `2026-05-02`).

Because these are injected at compile time, the resulting Docker container does not need a `.git` folder or any external dependencies to know its identity.

---

## 2. How to Verify the Deployed Version

There are three ways to verify the running version, ranging from operator-friendly UI elements to automated drift-detection headers.

### Method A: Platform Admin UI (Easiest)
For platform administrators and operators:
1. Log in to the Platform Admin dashboard (`/admin`).
2. Look at the bottom of the left-hand navigation sidebar.
3. You will see a version chip (e.g., `v0.9.1  f0e7c16`).
4. **Hover your mouse** over the truncated 7-character SHA to see the full, exact Git commit hash in the tooltip.

### Method B: The Public API Endpoint
For external monitoring systems or quick curl checks:
You can hit the unauthenticated `GET /api/version` endpoint on any Atlas Platform deployment.

**Command:**
```bash
curl -s https://uat.atlas.ruuderie.com/api/version | jq
```

**Response:**
```json
{
  "version": "0.9.1",
  "build_sha": "f0e7c167a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3",
  "build_date": "2026-05-02"
}
```
*You can compare this `build_sha` directly against the latest commit in the `uat` or `main` branches on GitHub.*

### Method C: The `X-Atlas-Version` HTTP Header
For diagnosing caching issues, reverse-proxy misconfigurations, or cross-node drift (where one Kubernetes pod is on a newer version than another), the platform injects an HTTP header into **every single API response**.

**Command:**
```bash
curl -I https://uat.atlas.ruuderie.com/api/public/pages/some-tenant-id/home
```

**Response Headers:**
```http
HTTP/1.1 200 OK
content-type: application/json
x-atlas-version: 0.9.1+f0e7c167
...
```
*The format is always `<semver>+<sha>`. If you see inconsistent headers across multiple requests to the same endpoint, it means your Kubernetes pods are running mismatched images (environment drift).*

---

## 3. Resolving "Unknown" Versions

If you ever see `v0.9.1  dev` in the UI, or the `build_sha` returns as `dev`, it means:
1. You are running the application locally via `cargo run` outside of the CI pipeline.
2. The CI/CD pipeline failed to pass the `ATLAS_BUILD_SHA` build argument to the Dockerfile.

If this happens in a live environment (UAT or Prod), check the GitHub Actions deploy logs to ensure the `--build-arg ATLAS_BUILD_SHA=${{ github.sha }}` flag is being passed to the container build step.
