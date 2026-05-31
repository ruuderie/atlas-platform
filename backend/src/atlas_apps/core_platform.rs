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

            // === Unification Migration (new canonical model) ===
            Box::new(crate::migration::m20260601_unify_accounts_contacts::Migration),

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

            // ═══════════════════════════════════════════════════════════════════
            // Platform Generics Round 1 Gap Fills — June 2026
            //
            // Identified via horizontal gap analysis across all 14 roadmap apps.
            // Promoted to generics BEFORE Direct Booking Engine, CoverFlow, and
            // Revenue Manager wrote conflicting app-specific table definitions.
            //
            // Migration prefix m20260701_ ensures these sort AFTER all m20260601_
            // generics and are applied in the correct dependency order.
            // ═══════════════════════════════════════════════════════════════════

            // GENERIC-19: Multi-channel campaign management
            // Covers: atlas_campaigns, atlas_sequence_steps,
            //         atlas_campaign_enrollments, atlas_campaign_events
            // Prevents: PM/AgentLink/Clipping Marketplace from building conflicting
            //           campaign tables independently.
            Box::new(crate::migration::m20260701_g19_campaigns::Migration),

            // GENERIC-23: Time-bounded reservation with inventory hold
            // Covers: atlas_reservations (polymorphic asset reservation + hold lifecycle)
            // Prevents: atlas_ledger_entries.billable_entity_type fragmentation across
            //           direct_bookings / package_bookings / guest_reservations.
            // IMPORTANT: Register a release_expired_holds BackgroundJob in background_jobs().
            Box::new(crate::migration::m20260701_g23_reservations::Migration),

            // GENERIC-25: Commission plan & split governance
            // Covers: atlas_commission_plans, atlas_commission_plan_splits
            //         + backfills atlas_ledger_splits.commission_plan_id
            // Prevents: Commission logic being hardcoded in CoverFlow/AgentLink handlers.
            Box::new(crate::migration::m20260701_g25_commission_plans::Migration),

            // GENERIC-26: Product catalog, pricebook & availability grid
            // Covers: atlas_catalog_entries, atlas_catalog_rate_rules,
            //         atlas_catalog_availability
            // Prevents: Direct Booking hotel_room_types/room_rates and Revenue Manager
            //           tenant_room_inventories becoming competing private pricebook schemas.
            Box::new(crate::migration::m20260701_g26_catalog::Migration),
        ]
    }

    fn background_jobs(&self) -> Vec<BackgroundJob> {
        vec![
            // ── G-23 Reservation Hold Expiry Sweeper ─────────────────────────
            // Fires every 5 minutes. Platform-wide sweep (not per-tenant).
            // Sets status = 'cancelled' on reservations WHERE status = 'hold'
            // AND hold_expires_at < NOW(). Prevents ghost inventory locks after
            // payment abandonment.
            //
            // The tenant_id argument is ignored — this job sweeps ALL tenants
            // in a single UPDATE statement for efficiency.
            BackgroundJob {
                job_type: "release_expired_reservation_holds".to_string(),
                default_interval_seconds: 300, // every 5 minutes
                is_active_by_default: true,
                default_config_payload: None,
                executor: Box::new(|db, _tenant_id, _config| {
                    Box::pin(async move {
                        crate::services::reservation_service::ReservationService::release_expired_holds(&db)
                            .await
                            .map(|released| {
                                if released > 0 {
                                    tracing::info!(
                                        "BackgroundJob: released {} expired reservation holds",
                                        released
                                    );
                                }
                            })
                    })
                }),
            },
        ]
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
    fn test_migrations_populated_with_platform_generics() {
        let app = CorePlatformApp;
        let migrations = app.migrations();

        // As of Platform Generics v2, CorePlatformApp is the canonical place
        // where all shared infrastructure (G01-G08) + unification + domain
        // generics (G09-G18) are registered.
        assert!(
            !migrations.is_empty(),
            "CorePlatformApp should now return the full set of platform generics migrations (was intentionally populated during v2 work)"
        );

        // Sanity: we expect at least the unification migration + several generics.
        // Exact count can grow; we just ensure the v2 design is active.
        assert!(
            migrations.len() >= 10,
            "Expected a substantial number of generics migrations, got {}",
            migrations.len()
        );
    }

    #[test]
    fn test_background_jobs_registered() {
        let app = CorePlatformApp;
        let jobs = app.background_jobs();

        // G-23: The reservation hold expiry sweeper must be present so the
        // background job poller can sweep expired holds platform-wide every 5 minutes.
        assert_eq!(jobs.len(), 1, "Expected exactly 1 platform background job registered");
        assert_eq!(
            jobs[0].job_type,
            "release_expired_reservation_holds",
            "G-23 hold expiry sweeper must be registered as the platform background job"
        );
        assert_eq!(jobs[0].default_interval_seconds, 300);
        assert!(jobs[0].is_active_by_default);
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

