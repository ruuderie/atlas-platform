use crate::traits::atlas_app::{AtlasApp, BackgroundJob};
use async_trait::async_trait;
use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;

// ══════════════════════════════════════════════════════════════════════════════
// PlatformAdminApp — AtlasApp implementation for the platform operator panel.
//
// The platform-admin is a first-party operator tool (not a tenanted sub-app).
// It is served as a Leptos/WASM SPA at a dedicated operator domain.
//
// Responsibilities at the AtlasApp boundary:
//   - authenticated_router(): wraps all /api/admin/* routes (users, CRM, billing,
//     network, AI tasks, feature flags, compliance, uploads, etc.)
//   - public_router(): empty — the admin panel has no public-facing endpoints
//   - migrations(): empty — admin panel tables are owned by CorePlatformApp
//     (shared schema). No tenant-scoped tables here.
//   - background_jobs(): empty — the admin panel is a read/write tool; it triggers
//     backend jobs but doesn't run any itself.
//   - provision(): no-op — platform-admin is not provisioned per-tenant.
//
// State Binding Contract:
//   admin_routes_raw() returns a state-free Router<DatabaseConnection>.
//   .with_state(db) is called EXACTLY ONCE here, inside authenticated_router().
//   This prevents the silent route-dropping that occurs when pre-finalized
//   sub-routers are merged via the get_active_apps() loop.
//   (See the State Binding Contract section in atlas_app.rs for full rationale.)
//
// Registration Order:
//   PlatformAdminApp is registered AFTER CorePlatformApp in get_active_apps()
//   so CorePlatformApp's routes (tenant, onboarding, CMS, seeds, etc.) are
//   established first.
// ══════════════════════════════════════════════════════════════════════════════

pub struct PlatformAdminApp;

#[async_trait]
impl AtlasApp for PlatformAdminApp {
    fn app_id(&self) -> &'static str {
        "platform_admin"
    }

    /// No public-facing routes — the platform-admin is an authenticated-only tool.
    fn public_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new().with_state(db)
    }

    /// All /api/admin/* routes, consumed by the Leptos WASM operator panel.
    ///
    /// Routes include: users, CRM (accounts, contacts, leads, deals, cases),
    /// billing plans, billing ledgers, tenant stats, platform apps, feature flags,
    /// AI tasks, compliance, verification queue, developer console, uploads,
    /// syndication admin, passkeys admin, A/B test management, and more.
    ///
    /// State is applied exactly once here via `.with_state(db)`.
    fn authenticated_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        crate::admin::routes::admin_routes_raw().with_state(db)
    }

    /// No app-specific migrations — admin panel uses the shared platform schema.
    ///
    /// All tables read and written by platform-admin are owned by CorePlatformApp's
    /// migrations (atlas_app_deployment_config, atlas_subscription, billing_plan, etc.).
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }

    /// No background jobs — platform-admin is a read/write UI tool.
    ///
    /// The admin panel triggers backend jobs (AI tasks, re-provisioning) via API
    /// calls but does not register any pollers of its own.
    fn background_jobs(&self) -> Vec<BackgroundJob> {
        vec![]
    }

    // provision() and onboarding_steps() use the default no-op implementations
    // from the trait. The platform-admin is not provisioned per-tenant.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_admin_app_id() {
        assert_eq!(PlatformAdminApp.app_id(), "platform_admin");
    }

    #[test]
    fn test_platform_admin_no_migrations() {
        assert!(
            PlatformAdminApp.migrations().is_empty(),
            "PlatformAdminApp owns no migrations — all shared schema is owned by CorePlatformApp"
        );
    }

    #[test]
    fn test_platform_admin_no_background_jobs() {
        assert!(
            PlatformAdminApp.background_jobs().is_empty(),
            "PlatformAdminApp has no background jobs — it is a UI tool, not a service"
        );
    }
}
