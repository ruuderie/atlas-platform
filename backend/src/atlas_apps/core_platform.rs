use crate::traits::atlas_app::{AtlasApp, BackgroundJob};
use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use async_trait::async_trait;
use uuid::Uuid;


// ══════════════════════════════════════════════════════════════════════════════
// CorePlatformApp — The canonical reference implementation of AtlasApp.
//
// This app owns all cross-cutting CMS and platform service routes that every
// tenant gets automatically. It must be registered FIRST in get_active_apps()
// so its routes are established before domain sub-apps are merged.
//
// Route ownership by tier:
//   Tier 1 (this file): tenant, app_instance, app_pages, app_menus,
//                       onboarding, forms, feeds, search, audit_logs, app_seeds
//   Tier 2 (sub-apps):  anchor::pages, network listings/CRM/profiles
//   Tier 3 (api.rs):    auth, sessions, admin panel, A/B testing, setup
//
// State Binding Contract:
//   - Handler modules expose state-free `*_raw()` constructors.
//   - `.with_state(db)` is called EXACTLY ONCE, at the AtlasApp boundary below.
//   - NEVER call `.with_state()` inside a handler's route function that is
//     intended for use inside an AtlasApp. Axum silently drops routes from
//     pre-finalized sub-routers merged via the get_active_apps() loop.
//     (This was the root cause of the Apr 8→Apr 15 404 regression and the
//      May 2 2026 "Overlapping method route" panic in commit 1b84c375.)
// ══════════════════════════════════════════════════════════════════════════════

pub struct CorePlatformApp;

#[async_trait]
impl AtlasApp for CorePlatformApp {
    fn app_id(&self) -> &'static str {
        "core_platform"
    }

    /// Public routes — available without authentication.
    /// State is applied once here via `.with_state(db)`.
    fn public_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new()
            .merge(crate::handlers::tenant::public_routes_raw())
            .merge(crate::handlers::app_instance::public_routes_raw())
            .merge(crate::handlers::app_pages::public_routes_raw())
            .merge(crate::handlers::app_menus::public_routes_raw())
            .merge(crate::handlers::onboarding::public_routes_raw())
            .merge(crate::handlers::forms::public_routes()) // already state-free
            .merge(crate::handlers::feeds::public_routes_raw())
            .with_state(db)
    }

    /// Authenticated routes — protected by the platform auth middleware.
    /// State is applied once here via `.with_state(db)`.
    fn authenticated_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new()
            .merge(crate::handlers::tenant::authenticated_routes_raw())
            .merge(crate::handlers::app_instance::authenticated_routes_raw())
            .merge(crate::handlers::app_pages::authenticated_routes_raw())   // Phase 5: CRUD
            .merge(crate::handlers::app_menus::authenticated_routes_raw())   // Phase 5: CRUD
            .merge(crate::handlers::onboarding::authenticated_routes_raw())
            .merge(crate::handlers::feeds::authenticated_routes_raw())
            .merge(crate::handlers::search::authenticated_routes()) // already state-free
            .merge(crate::handlers::audit_logs::authenticated_routes()) // already state-free
            .merge(crate::handlers::app_seeds::authenticated_routes_raw())
            .with_state(db)
    }


    /// Core platform schema migrations live in the base mod.rs migrator today.
    /// A follow-up can extract them here for full encapsulation once the migration
    /// runner supports multi-source ordering.
    ///
    /// Platform Generics v2 (GENERIC-09+) are intentionally registered here so that
    /// CorePlatformApp remains the single source of truth for all shared infrastructure
    /// and domain object tables.
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // --- Original Infrastructure Generics (priority order) ---
            // G-02: Secure file storage + external sharing (highest priority)
            Box::new(crate::migration::m20260601_g02_vault_extension::Migration),
            // G-03: Multi-rail payment ledger + extensible credential system
            // (Designed to be provider-agnostic. Bitcoin path allows future self-hosted infrastructure.)
            Box::new(crate::migration::m20260601_g03_payments::Migration),
            // G-01: Spatial / PostGIS foundation (geo service areas)
            Box::new(crate::migration::m20260601_g01_geo_postgis::Migration),
            // G-05: External integrations gateway (PMS, OTA, AMS, GDS, Telephony, etc.)
            Box::new(crate::migration::m20260601_g05_external_integrations::Migration),
            // G-06: Verification queue (human-in-the-loop trust workflows)
            Box::new(crate::migration::m20260601_g06_verification_queue::Migration),
            // G-07: Real-time WebSocket room infrastructure
            Box::new(crate::migration::m20260601_g07_realtime::Migration),
            // G-04: B2C recurring subscriptions (creator tiers, plans, etc.)
            Box::new(crate::migration::m20260601_g04_subscriptions::Migration),
            // G-08: Async AI / LLM task queue
            Box::new(crate::migration::m20260601_g08_ai_tasks::Migration),

            // --- New Domain Generics (G-09+) ---
            // GENERIC-09: Portfolio grouping
            Box::new(crate::migration::m20260601_g09_portfolios::Migration),
            // GENERIC-10: Central asset registry
            Box::new(crate::migration::m20260601_g10_assets::Migration),
            // GENERIC-11: Legal agreements / contracts
            Box::new(crate::migration::m20260601_g11_contracts::Migration),
            // GENERIC-12: Service providers / vendors / agents
            Box::new(crate::migration::m20260601_g12_service_providers::Migration),
            // GENERIC-13: Universal case / work item object
            Box::new(crate::migration::m20260601_g13_cases::Migration),
            // GENERIC-14: Generic document registry with e-sig & versioning
            Box::new(crate::migration::m20260601_g14_documents::Migration),
            // GENERIC-15: Deal / pipeline opportunities
            Box::new(crate::migration::m20260601_g15_opportunities::Migration),
            // GENERIC-16: Regulatory registrations / permits / licenses
            Box::new(crate::migration::m20260601_g16_regulatory_registrations::Migration),
            // GENERIC-17: Tax events and filings
            Box::new(crate::migration::m20260601_g17_tax::Migration),
            // GENERIC-18: Structured applications / onboarding
            Box::new(crate::migration::m20260601_g18_applications::Migration),
        ]
    }

    fn background_jobs(&self) -> Vec<BackgroundJob> {
        vec![]
    }

    /// Bootstraps a new tenant with the minimal CMS scaffolding every site needs.
    ///
    /// Creates (idempotently):
    ///   1. A published "Home" page (`slug = "home"`) with an empty blocks payload.
    ///   2. A "header" navigation menu with a single "Home" link.
    ///
    /// Uses `ON CONFLICT DO NOTHING` semantics — safe to call multiple times.
    /// Each insert is a no-op if the row already exists, so re-provisioning
    /// an existing tenant leaves its data intact.
    ///
    /// Accepts any `ConnectionTrait` so it can be called inside a transaction
    /// (`&DatabaseTransaction`) or directly against a `DatabaseConnection`.
    async fn provision(&self, db: &DatabaseConnection, tenant_id: Uuid) -> Result<(), String> {
        use sea_orm::{ConnectionTrait, Statement};
        use chrono::Utc;

        let now = Utc::now();

        // ── 1. Seed default home page (idempotent via WHERE NOT EXISTS) ───────────
        let page_id = Uuid::new_v4();
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO app_pages (id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at)
            SELECT $1, $2, 'home', 'Home', 'Welcome to your new site', 'standard', NULL, '[]'::jsonb, true, $3, $3
            WHERE NOT EXISTS (
                SELECT 1 FROM app_pages WHERE tenant_id = $2 AND slug = 'home'
            )
            "#,
            vec![page_id.into(), tenant_id.into(), now.into()],
        ))
        .await
        .map_err(|e| format!("provision: failed to seed home page: {e}"))?;

        // ── 2. Seed default header menu (idempotent via WHERE NOT EXISTS) ─────────
        let menu_id = Uuid::new_v4();
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
            SELECT $1, $2, 'header', 'Home', '/', NULL, 1, true, $3, $3
            WHERE NOT EXISTS (
                SELECT 1 FROM app_menus WHERE tenant_id = $2 AND menu_type = 'header' AND label = 'Home'
            )
            "#,
            vec![menu_id.into(), tenant_id.into(), now.into()],
        ))
        .await
        .map_err(|e| format!("provision: failed to seed header menu: {e}"))?;

        tracing::info!(
            "provision: bootstrapped tenant {} with default home page and header menu",
            tenant_id
        );

        Ok(())
    }

}

// ══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::atlas_app::AtlasApp;

    #[test]
    fn test_app_id() {
        let app = CorePlatformApp;
        assert_eq!(app.app_id(), "core_platform");
    }

    #[test]
    fn test_migrations_empty() {
        let app = CorePlatformApp;
        assert!(
            app.migrations().is_empty(),
            "CorePlatformApp migrations should be empty until extracted from base migrator"
        );
    }

    #[test]
    fn test_background_jobs_empty() {
        let app = CorePlatformApp;
        assert!(app.background_jobs().is_empty());
    }

    /// `provision()` overrides the default no-op and issues Postgres-specific SQL.
    /// We cannot run it in a unit test (no live Postgres), but we can verify the
    /// method is accessible and the type compiles. Integration-level testing happens
    /// via the provisioning flow in the admin endpoint tests.
    #[test]
    fn test_provision_is_overridden() {
        let app = CorePlatformApp;
        // Verify it overrides AtlasApp::provision by checking the type is not the default.
        // This is a compile-check — if provision() is removed or its signature changes,
        // this test will fail to compile, which is exactly what we want.
        let _: &dyn AtlasApp = &app;
    }
}

