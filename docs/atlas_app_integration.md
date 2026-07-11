# Atlas App Integration Protocol

> **Strongly Recommended First Read**: [`docs/CURRENT_STATE.md`](CURRENT_STATE.md) for the living registry of platform generics, the unified Account/Contact model, and ops ground truth.
>
> **Before any net-new table:** run **Rule 7 — Generic Fitness Test** in [`docs/architecture/generic_fitness_test.md`](architecture/generic_fitness_test.md) (diagram: [`architecture/diagrams/generic_fitness_test_flow.mmd`](architecture/diagrams/generic_fitness_test_flow.mmd)).

This document contains the technical integration rules. The strategic "why" and current reality are in `CURRENT_STATE.md`.

This protocol determines how applications attach to the Atlas Platform. The `AtlasApp` Rust trait
is the **only** sanctioned integration point — direct mutation of `api.rs` is forbidden for app routes.

> See also: [`docs/architecture/platform_layer_map.md`](architecture/platform_layer_map.md)
> for the full Tier 1 / Tier 2 / Tier 3 route ownership model.
>
> See also: [`docs/architecture/auth_and_permissions.md`](architecture/auth_and_permissions.md)
> for authentication flows, the two-layer permission model, and how to define app-specific permissions.

---

## Purpose

The legacy approach hardcoded each app's routes directly into `backend/src/api.rs`, making
cross-app isolation impossible and causing repeated "overlapping method route" panics whenever
two apps registered the same prefix.

The **`AtlasApp` trait** solves this by enforcing:

1. **State-free route constructors** — every handler returns `Router<DatabaseConnection>`,
   with `.with_state(db)` called exactly once at the `AtlasApp` boundary.
2. **Scoped multi-tenant SeaORM schemas** — every table requires a `tenant_id UUID` column.
3. **Isolated background polling** — background jobs declared as `BackgroundJob` structs,
   not side-effecting page loads.
4. **Idempotent provisioning** — the `provision()` hook seeds default data for new tenants.

---

## Implementing the `AtlasApp` Trait

Create your app in `backend/src/atlas_apps/your_app.rs` and implement the trait:

```rust
use crate::traits::atlas_app::{AtlasApp, BackgroundJob};
use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use async_trait::async_trait;

pub struct MyApp;

#[async_trait]
impl AtlasApp for MyApp {
    fn app_id(&self) -> &'static str {
        "my_app"  // must be globally unique
    }

    // ── Public routes (no auth required) ─────────────────────────────────────
    // Return a state-FREE router. The platform calls .with_state(db) exactly once.
    fn public_router(&self, db: DatabaseConnection) -> Router {
        crate::handlers::my_app::public_routes_raw()
            .with_state(db)
    }

    // ── Authenticated routes (JWT required) ───────────────────────────────────
    fn authenticated_router(&self, db: DatabaseConnection) -> Router {
        crate::handlers::my_app::authenticated_routes_raw()
            .with_state(db)
    }

    // ── Migrations ────────────────────────────────────────────────────────────
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(crate::migration::m20260101_000001_create_my_app_tables::Migration),
        ]
    }

    // ── Provisioning (optional) ───────────────────────────────────────────────
    // Called by POST /api/admin/platform/provision/{tenant_id}.
    // Use ON CONFLICT DO NOTHING — this must be idempotent.
    async fn provision(
        &self,
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Seed default data for a new tenant here.
        Ok(())
    }

    // ── Background Jobs (optional) ────────────────────────────────────────────
    fn background_jobs(&self) -> Vec<BackgroundJob> {
        vec![
            BackgroundJob {
                job_type: "my_app_sync".to_string(),
                default_interval_seconds: 600,
                is_active_by_default: true,
                default_config_payload: None,
                executor: Box::new(|db, _config| {
                    Box::pin(async move {
                        crate::atlas_apps::my_app::services::sync(db).await
                    })
                }),
            }
        ]
    }
}
```

Register the app in `backend/src/atlas_apps/mod.rs`:

```rust
pub fn get_active_apps() -> Vec<Box<dyn AtlasApp>> {
    vec![
        Box::new(core_platform::CorePlatformApp),  // ← always first
        Box::new(anchor::AnchorApp),
        Box::new(my_app::MyApp),                   // ← add here
    ]
}
```

---

## State Binding Contract ⚠️

> **This is the most common source of silent 404 regressions.**

Every handler file must expose a `_raw()` constructor that returns a **state-free** router:

```rust
// ✅ CORRECT — state is NOT bound inside the handler file
pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/public/my-app/{tenant_id}", get(get_items))
        .route("/api/public/my-app/{tenant_id}/{slug}", get(get_item))
}

// ✅ The AtlasApp impl binds state exactly once at its boundary:
fn public_router(&self, db: DatabaseConnection) -> Router {
    crate::handlers::my_app::public_routes_raw().with_state(db)
}
```

```rust
// ❌ WRONG — pre-binding state inside the handler causes Axum to silently
//            drop routes when the router is merged at the platform level.
pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/public/my-app/{tenant_id}", get(get_items))
        .with_state(db)  // ← NEVER do this inside a handler file
}
```

**Historical regressions caused by violating this contract:**
- Apr 8–15, 2026 — global 404 on all CMS pages (`app_pages` pre-binding)
- May 2, 2026 commit `1b84c375` — "Overlapping method route" panic

---

## Important Integration Rules

### 1. Route Prefix Ownership

Every app owns a **unique route prefix**. Never register a prefix in more than one app.

| App                    | Owned public prefix          | Owned auth prefix            |
|------------------------|------------------------------|------------------------------|
| `CorePlatformApp`      | `/api/public/pages/*`        | `/api/pages/*`               |
| `CorePlatformApp`      | `/api/public/menus/*`        | `/api/menus/*`               |
| `AnchorApp`            | `/api/public/anchor/*`       | `/api/anchor/*`              |
| `NetworkInstanceApp`   | `/api/public/listings/*`     | `/api/listings/*`            |

Overlapping prefixes cause Axum to panic at startup with `Overlapping method route`.

### 2. Tenant Isolation — SeaORM Migrations

Every table your app creates **must** include a `tenant_id UUID` column with a foreign key
to the `tenants` table. No global, unscoped tables.

```rust
// ✅ Correct — tenant-scoped query
MyEntity::find()
    .filter(my_entity::Column::TenantId.eq(tenant_id))
    .all(db)
    .await?;

// ❌ Wrong — leaks data across tenants
MyEntity::find().all(db).await?;
```

### 3. Migrations — Unified Registry Rule

App-specific migrations **must** be declared inside your `AtlasApp::migrations()` method.

**Never** place app-specific migrations in the core platform registry
(`backend/src/migration/mod.rs`). Splitting migrations between the two registries causes
non-deterministic ordering and triggers a fatal `Migration file is missing` panic during
K8s pod startup → `CrashLoopBackOff`.

### 4. Provisioning — Idempotency

The `provision()` method is called:
- Automatically when `create_network()` completes in platform-admin
- Manually via `POST /api/admin/platform/provision/{tenant_id}`
- **Potentially multiple times** (retry on partial failure)

Always use `ON CONFLICT DO NOTHING` / `INSERT OR IGNORE`. Never assume a clean slate.

### 5. Background Jobs

Never trigger expensive external calls from a frontend page load or server function.
Wrap all third-party integrations in a `BackgroundJob` struct so the core poller can
regulate rate limits.

```rust
// ❌ Wrong — triggers on every page render
#[server]
async fn load_page() -> Result<Data, ServerFnError> {
    sync_with_external_service().await?;  // blocks render, burns rate limits
    Ok(data)
}

// ✅ Correct — runs on the background poller cadence
BackgroundJob {
    job_type: "my_app_external_sync".to_string(),
    default_interval_seconds: 600,
    executor: Box::new(|db, _| Box::pin(async move {
        sync_with_external_service(db).await
    })),
}
```

### 6. SSR Header Forwarding

When making `reqwest` calls from server-side Leptos components to platform endpoints,
you **must** explicitly extract and forward the browser's `Host` and `Origin` headers via Context.

Failure to forward `Host` causes the backend router to fail tenant resolution → `404 Not Found`.

### 7. Generic Subsystems — Check Before Creating Net-New Tables

**Mandatory:** complete **Rule 7** in [`docs/architecture/generic_fitness_test.md`](architecture/generic_fitness_test.md) and record the Q1–Q4 writeup before adding migrations.

Eight structural patterns appear in 3+ roadmap apps and are implemented **once** in the
base platform. After Rule 7, use the living registry in [`CURRENT_STATE.md`](CURRENT_STATE.md) and this starter table:

| Need | Platform Generic | Table(s) |
|------|-----------------|----------|
| Store a file in R2, generate presigned URLs, share with guests | `atlas_vault` | `attachment` (extended), `attachment_share_tokens`, `attachment_multipart_uploads` |
| Record any payment (rent, premium, creator payout, booking) | `atlas_payments` | `atlas_ledger_entries`, `atlas_ledger_splits` |
| Spatial / geo query (polygon containment, radius search) | `atlas_geo` | `geo_service_areas` (PostGIS GIST) |
| Connect to PMS / AMS / OTA / GDS third-party API | `atlas_external_integrations` | `atlas_external_integrations`, `atlas_integration_events` |
| Human trust verification (selfie, GPS, license, permit) | `atlas_verification_queue` | `atlas_verification_requests` |
| B2C recurring subscription (creator tier, city plan, SaaS) | `atlas_subscriptions` | `atlas_subscriptions` |
| Real-time WebSocket room scoped to an entity | `atlas_realtime` | `atlas_ws_rooms`, `atlas_ws_messages` |
| Call an LLM or AI model asynchronously | `atlas_ai_tasks` | `atlas_ai_tasks` |

> **Rule 7 (living):** [`docs/architecture/generic_fitness_test.md`](architecture/generic_fitness_test.md)  
> **Registry:** [`docs/CURRENT_STATE.md`](CURRENT_STATE.md)  
> **Historical original-8 DDL notes:** [`docs/architecture/platform_generics.md`](architecture/platform_generics.md)

---

## Registered Apps (as of v0.9.x)

| App                  | File                                        | Position | provision()? |
|----------------------|---------------------------------------------|----------|--------------|
| `CorePlatformApp`    | `atlas_apps/core_platform.rs`               | 1st      | ✅ Yes        |
| `AnchorApp`          | `atlas_apps/anchor.rs`                      | 2nd      | ❌ No         |
| `NetworkInstanceApp` | `atlas_apps/network_instance.rs`            | 3rd      | ❌ No         |

> **`CorePlatformApp` must always be registered first** — it seeds the base `app_pages`
> and `app_menus` that other apps may depend on via provisioning.
