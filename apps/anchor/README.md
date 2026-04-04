# Services Template Application

Welcome to the Services template application! This application is a high-performance, full-stack WebAssembly application geared for personal branding, resumes, technology services, and blogging. It is powered by [Rust](https://www.rust-lang.org/) and [Leptos](https://github.com/leptos-rs/leptos). It features an [Axum](https://github.com/tokio-rs/axum) backend, a Tailwind frontend, Postgres integrations via `sqlx`, and hardware-backed Passkey (WebAuthn) security.

## Development Checklist (Cargo)

If you want to run the project natively using `cargo-leptos` instead of Docker, make sure you have the required prerequisites:

1. `rustup toolchain install nightly` (Leptos relies on Nightly Rust features)
2. `rustup target add wasm32-unknown-unknown`
3. `cargo binstall cargo-leptos` 
4. `npm install -g tailwindcss`

To start the development server that actively watches for file changes **and provides live hot-reloading**:
```bash
cargo leptos watch
```

> **Note:** Running `cargo leptos watch` natively is the fastest way to iterate on UI changes. However, if you are running the application entirely inside local Kubernetes (via Docker), changes to your local Rust source code will **not** automatically update the running containers. You must explicitly rebuild the Docker image and restart the deployment (see [DEPLOYMENT.md](DEPLOYMENT.md) for details).

## Kubernetes Deployment (Local & Production)

This project relies entirely on **Docker** and **Kubernetes** to mock its production environments locally, ensuring there are no surprises when you go live.

Due to the extreme security restrictions built into Passkeys and the WebAuthn API, local browser testing requires specific internal routing mechanics to bypass HTTPS constraints safely.

**👉 Please read [DEPLOYMENT.md](./DEPLOYMENT.md) for complete, step-by-step instructions on running the infrastructure.**

## Multi-Tenant Architecture

Anchor executes within a hybridized multi-tenant database context within the Atlas Platform. 
All requests strictly require the `X-Tenant-Id` HTTP header to resolve the user's `TenantContext` safely across all Axum middlewares and SQLx queries. The Atlas control plane automatically proxies and injects this secure context dynamically.

If you are developing locally, you can provide an override by specifying `DEFAULT_TENANT_ID` in your local `.env` file. 
If no tenant header is provided and `DEFAULT_TENANT_ID` is omitted, the application will reject inbound traffic with a `400 Bad Request`.
