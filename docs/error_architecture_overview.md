# Atlas & Anchor Platform Architecture Overview

This document outlines the system architecture of the Atlas Platform and its CMS application (Anchor), providing context for the recent UI rendering anomalies on the `uat.buildwithruud.com` tenant instance.

## 1. System Architecture

The platform operates on a strict multi-tenant, modular architecture designed for high performance and horizontal scaling.

### Atlas (The Core Platform)
Atlas is the foundational backend infrastructure built in **Rust** using **Axum** (routing) and **SeaORM** (database ORM) backed by **PostgreSQL**. It is completely headless and acts as the central hub for:
*   **Multi-tenancy:** Routing requests to the correct tenant context based on `Host` headers or `x-tenant-id` tokens (resolved via `app_domains` → `app_instances` → `tenant`).
*   **Authentication & Billing:** Handling core identity, WebAuthn/Passkeys, and Stripe/Paddle payment integrations.
*   **Modular App Registry:** Atlas doesn't serve UI directly. Instead, it mounts "Apps" (like Anchor) via a plugin-style trait architecture (`AtlasApp`).

### Anchor (The CMS Frontend)
Anchor is an Atlas App that serves as a high-performance Content Management System. 
*   It is built using **Leptos** (a Rust-based frontend framework) compiled to WebAssembly (`.wasm`).
*   It utilizes **Server-Side Rendering (SSR)** coupled with client-side hydration to ensure SEO compliance and fast time-to-interactive.
*   Styling is handled entirely via **Tailwind CSS**.

---

## 2. The Dynamic Page Engine

The core philosophy of the Anchor frontend is that it is heavily **data-driven**. Rather than hardcoding Leptos views for every landing page, pages are constructed dynamically from the database.

### The JSONB Block Payload
If you look at the `app_pages` table in Postgres, you will see a `blocks_payload` column of type `JSONB`. This payload dictates exactly what renders on a page.

A payload looks like this (from `m20260425_000005_fix_ruud_tenant_lookup.rs`):
```json
[
  {
    "RawHtml": {
      "content": "<div class=\"w-full pt-32 pb-24 px-4 md:px-[8.5rem]\">...</div>"
    }
  },
  {
    "FormBuilder": {
      "form_id": "rev_intake"
    }
  }
]
```

### The Rendering Pipeline
In `apps/anchor/src/pages/dynamic_landing.rs`, Leptos fetches this JSON payload during SSR, deserializes it into a strongly-typed `DynamicBlock` enum, and maps each variant to a corresponding Rust UI component:

```rust
// Simplified extraction from dynamic_landing.rs
{parsed_blocks.into_iter().map(|block| match block {
    DynamicBlock::Hero(data) => view! { <HeroBlock data=data /> }.into_view(),
    DynamicBlock::Grid(data) => view! { <GridBlock data=data /> }.into_view(),
    DynamicBlock::RawHtml(data) => view! { <RawHtmlBlock data=data /> }.into_view(),
    DynamicBlock::FormBuilder(data) => view! { <FormBuilderBlock data=data /> }.into_view(),
}).collect_view()}
```

---

## 3. First Principles: The Layout Overlap Bug

The recent layout bug where the homepage content was shifting upwards and overlapping the navigation bar was a classic case of **distributed state desynchronization** between the `.wasm` client code and the database payload.

### The Design Requirement
The site features a `fixed` navigation bar (`top-0 py-6`). Because `fixed` elements are removed from normal HTML document flow, the main content area *must* have a top padding (e.g., `pt-32` / `8rem`) so it clears the navigation and doesn't render underneath it.

### The Initial State
Originally, the Rust `.wasm` component had a hardcoded layout wrapper:
```html
<!-- Old dynamic_landing.rs -->
<main class="pt-32 pb-24 px-4 md:px-[8.5rem]">
    {parsed_blocks}
</main>
```
Because the wrapper handled the padding, the database payloads did *not* need padding classes.

### The Architectural Shift
We realized that hardcoding `<main>` padding prevented users from creating edge-to-edge, full-bleed backgrounds (e.g., a dark hero section spanning the entire screen width). To fix this, we updated `dynamic_landing.rs` to remove the wrapper padding, shifting the responsibility of defining layout padding entirely to the JSON payload inside the database.

### The CI/CD Failure (The Root Cause)
We committed two things:
1. **The Rust update:** Stripping the padding from `dynamic_landing.rs`.
2. **The SQL Migration (`000005`):** Injecting the necessary `pt-32 px-4 md:px-[8.5rem]` directly into the `RawHtml` blocks payload in the UAT database.

The Woodpecker CI pipeline built the Docker images and began the rollout.
*   The **`anchor-app`** (frontend) rollout succeeded immediately.
*   The **`backend`** pod, however, runs `Migrator::up()` on startup. 
*   **The Panic:** SeaORM panicked on boot with: `Migration file ... is missing, this migration has been applied but its file is missing`.

This panic was caused by an architectural flaw in our migration registry. `AnchorApp` migrations were split—some were registered directly in the core `base` vector in `mod.rs`, and others were inside `AnchorApp::migrations()`. This split registry caused non-deterministic ordering. The database had recorded migrations `000001` and `000002` as applied, but a subsequent code restructuring caused the compiled binary to present a mismatched migration list to SeaORM, triggering a safety panic before *any* new migrations could run.

Because the backend panicked, Kubernetes triggered a `CrashLoopBackOff`.

### The Resulting Glitch
Because the backend rollout failed before migrations could execute, the `000005` SQL migration never applied. 
The live site ended up running the **new frontend code** (which expected the payload to provide padding) against the **old database payload** (which expected the frontend to provide padding). 

With zero padding applied, the Hero HTML slid straight to the top of the browser viewport, burying itself directly underneath the fixed navigation bar and breaking the grid aesthetics. 

### The Resolution & New Best Practices
To fix this and prevent it from ever happening again, we implemented three structural changes:

1. **Unified Migration Registry:** All app-specific content and seed migrations are strictly forbidden from residing in the core `base` vector (`mod.rs`). They must be declared exclusively within the app's trait implementation (e.g., `AnchorApp::migrations()`). The core `base` vector is reserved *only* for platform-wide infrastructure (users, tenants, billing).
2. **The "Hardened Migration" Pattern:** Previous patches used silent `DO $$ IF v_id IS NOT NULL` blocks. If a tenant lookup failed, the migration succeeded silently doing nothing. Moving forward, data migrations **must** enforce strict failure modes:
   * Use `RAISE EXCEPTION` if a target tenant cannot be located.
   * Check `GET DIAGNOSTICS v_rows_affected = ROW_COUNT;` and `RAISE EXCEPTION` if 0 rows are updated.
   * Emit a `RAISE NOTICE 'SUCCESS...'` so the outcome is visibly recorded in the backend startup logs.
3. **DynamicLanding Alignment:** `DynamicLanding` (the slug catch-all route) was updated to match `DynamicHomeLanding`. If a page contains dynamic JSON blocks, it receives a bare `<main>` wrapper (giving blocks total layout control). If it relies on legacy `hero_title` fields, it falls back to the hardcoded `pt-32` wrapper.
