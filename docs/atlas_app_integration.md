# Atlas App Integration Protocol

This protocol determines how applications (both internal systems and third-party modules) attach to the Atlas Platform. We leverage the `AtlasApp` Rust Trait API to guarantee full compatibility.

## Purpose

The legacy approach involved hardcoding an app's router setups via `backend/src/api.rs`, pushing raw `sqlx::query!(...)` patterns, and relying on frontend page loads fetching APIs under the hood.

The **AtlasApp API** solves this by strictly enforcing:
1. True SSR/Proxy Header Forwarding
2. Scoped Multi-Tenant SeaORM DB Schemas 
3. Isolated Background Polling Handlers

## Implementing the `AtlasApp` Trait

Create your application in `backend/src/apps/your_app/` and implement the standard `AtlasApp` trait imported from `traits::atlas_app::AtlasApp`.

```rust
use crate::traits::atlas_app::{AtlasApp, BackgroundJob};
use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use async_trait::async_trait;

pub struct AnchorApp;

#[async_trait]
impl AtlasApp for AnchorApp {
    fn app_id(&self) -> &'static str {
        "anchor"
    }

    fn router(&self, state: DatabaseConnection) -> Router {
        // Build and return your Axum router.
        // Make sure you bind it specifically to your paths!
        crate::apps::anchor::api::create_router(state)
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        // Map your app-specific migration structs here.
        // All models MUST support a tenant_id foreign key constraint. No rigid local tables.
        vec![
            Box::new(crate::apps::anchor::migrations::m20260408_000002_create_anchor_legacy_tables::Migration),
        ]
    }

    fn background_jobs(&self) -> Vec<BackgroundJob> {
        // Perfectly Encapsulated Application Logic
        vec![
            BackgroundJob {
                job_type: "anchor_bitcoin_block_sync".to_string(),
                default_interval_seconds: 600,
                is_active_by_default: true,
                default_config_payload: None,
                executor: Box::new(|db, config| {
                    Box::pin(async move {
                        // Your internal service logic runs here.
                        // The backend poller loop will inject `db` dynamically.
                        crate::apps::anchor::services::sync_bitcoin_blocks(db).await
                    })
                })
            }
        ]
    }
}
```

## Important Integration Rules

### 1. SSR Reqwest Headers (Preventing 404s)
When building Leptos or SSR components that make `reqwest` calls internally to platform endpoints, you **must explicitly extract and forward** the browser's `Host` and `Origin` headers via Context.

Failure to forward the incoming `Host` header will cause the backend router proxy to fail rendering tenant configurations (producing `404 Not Found`).

### 2. SeaORM Migrations 
DO NOT execute raw `sqlx::query!(...)` strings assuming an unmanaged schema structure. Every table your application defines MUST contain a `tenant_id` UUID column enforcing strict isolation. Include these isolated migrations within the `migrations()` interface.

### 3. Background Syncs
Never trigger costly background integrations silently via a Frontend page load (`use_effect` / Server Functions that call external systems on block build). Wrap all 3rd Party systems into encapsulated `BackgroundJob` structs so the Core Poller can regulate rate limits efficiently inline.
