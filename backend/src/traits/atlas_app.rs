use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

// ──────────────────────────────────────────────────────────────────────────────
// PERMISSION MODEL — READ BEFORE IMPLEMENTING A NEW APP
// ──────────────────────────────────────────────────────────────────────────────
// Each AtlasApp is responsible for defining its own permission enum.
// Do NOT add app-specific variants to the global `TenantRole` enum in user_account.rs.
//
// The platform uses a two-layer model:
//   Layer 1: TenantRole (PlatformSuperAdmin | Owner | Admin | Member) — stable, never changes
//   Layer 2: App-specific permissions (each app owns its own enum) — stored in user_app_permission
//
// Owner/Admin roles implicitly bypass all app-level permission checks.
// Member roles require explicit permission grants stored in user_app_permission.
//
// Full details: docs/auth_and_permissions.md
// ──────────────────────────────────────────────────────────────────────────────

/// Represents a dynamic asynchronous executor closure for background jobs.
pub type JobExecutor = Box<
    dyn Fn(DatabaseConnection, uuid::Uuid, Option<serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;

// ──────────────────────────────────────────────────────────────────────────────
// SEED PACK CONTRACT
// ──────────────────────────────────────────────────────────────────────────────

/// Async executor closure for applying a seed pack to a tenant.
/// Receives (db, tenant_id, app_instance_id) and inserts demo/test data
/// scoped to that tenant. Must use ON CONFLICT DO NOTHING on global tables.
pub type SeedApplyFn = Box<
    dyn Fn(DatabaseConnection, Uuid, Uuid) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;

/// A declarative description of a demo/test seed dataset an app can offer.
/// Seed packs are listed in the platform-admin when launching an app instance
/// and can be applied on-demand for demos and development.
pub struct AppSeedPack {
    /// Stable machine-readable ID, e.g. "transportation_logistics_starter".
    /// Used as the URL segment in the apply endpoint.
    pub id: &'static str,
    /// Human-readable title shown in the platform-admin picker.
    pub title: &'static str,
    /// Short description of what this pack seeds.
    pub description: &'static str,
    /// Hint shown to the admin, e.g. "~6 categories, 55 listings"
    pub content_summary: &'static str,
    /// The async executor that inserts seed rows scoped to the tenant.
    pub apply: SeedApplyFn,
}

// Manual Debug impl since SeedApplyFn is not Debug.
impl std::fmt::Debug for AppSeedPack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppSeedPack")
            .field("id", &self.id)
            .field("title", &self.title)
            .finish()
    }
}

/// Configuration and Execution strategy for a standard Multi-Tenant Background Job.
/// This enforces Option B: Perfect Encapsulation where the app provides the executable logic.
pub struct BackgroundJob {
    /// A unique identifier string for this job, e.g., "anchor_sync"
    pub job_type: String,
    
    /// Default execution interval in seconds
    pub default_interval_seconds: i32,
    
    /// Is the job turned on globally by default
    pub is_active_by_default: bool,
    
    /// Default config payload if the tenant doesn't override it
    pub default_config_payload: Option<serde_json::Value>,
    
    /// The actual execution closure providing perfect encapsulation.
    /// The Core Backend Poller will inject the database connection and the tenant's parsed config.
    pub executor: JobExecutor,
}

// ──────────────────────────────────────────────────────────────────────────────
// ONBOARDING CONTRACT
// ──────────────────────────────────────────────────────────────────────────────

/// Declares how the platform should verify that a single onboarding step is complete.
/// These checks are evaluated server-side against real data — they are not flags.
#[derive(Debug, Clone)]
pub enum StepCompletionCheck {
    /// Step is complete when a TenantSetting row with `key` exists for this tenant.
    TenantSettingExists { key: String },
    /// Step is complete when at least one AppDomain row exists for this app_instance.
    AppDomainExists,
    /// Step is complete when a given table has at least `min` rows scoped to this tenant.
    EntityCountGte { table: &'static str, min: usize },
    /// Step completion is evaluated entirely by `onboarding_readiness()` custom logic.
    Custom,
}

/// A single declarative onboarding step.
#[derive(Debug, Clone)]
pub struct OnboardingStep {
    /// Unique stable identifier, e.g. "identity", "domain", "categories"
    pub id: String,
    /// Human-readable title shown in the wizard UI
    pub title: String,
    /// Descriptive sentence shown under the title in the wizard
    pub description: String,
    /// Required steps block the "launch ready" state. Optional steps can be skipped.
    pub is_required: bool,
    /// Explicit display order exposed to the frontend wizard.
    /// Prevents ordering from being implicitly coupled to Vec insertion order.
    pub position: u8,
    /// How the backend evaluates whether this step is complete
    pub completion_check: StepCompletionCheck,
}

// ──────────────────────────────────────────────────────────────────────────────
// ATLAS APP TRAIT
// ──────────────────────────────────────────────────────────────────────────────

/// The formal API Contract for any application plugging into the Atlas Platform.
///
/// # State Binding Contract
///
/// Handler modules that are used inside an `AtlasApp` implementation MUST expose
/// state-free route constructors (i.e., functions that return `Router<DatabaseConnection>`
/// WITHOUT calling `.with_state(db)` internally). State is applied exactly once,
/// at the `AtlasApp` boundary, inside `public_router()` or `authenticated_router()`.
///
/// ```rust,ignore
/// // ✅ Correct — state-free constructor in the handler module
/// pub fn public_routes_raw() -> Router<DatabaseConnection> {
///     Router::new().route("/api/public/pages/{tenant_id}", get(list_pages))
///     // No .with_state() here!
/// }
///
/// // ✅ Correct — state applied once at the AtlasApp boundary
/// fn public_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
///     Router::new()
///         .merge(crate::handlers::app_pages::public_routes_raw())
///         .with_state(db)  // Exactly once, here.
/// }
/// ```
///
/// Violating this contract causes Axum to silently drop routes from
/// pre-finalized sub-routers merged via the `get_active_apps()` loop.
/// This was the root cause of the Apr 8→Apr 15 2026 404 regression and the
/// May 2 2026 "Overlapping method route" panic.
///
/// # Registration Order
///
/// `get_active_apps()` returns apps in the order they are merged into the
/// global router. `CorePlatformApp` MUST be registered first so its routes
/// are established before any domain sub-app routes are merged.
///
/// # Canonical Reference Implementation
///
/// See `backend/src/atlas_apps/core_platform.rs` for the canonical example of
/// a correct `AtlasApp` implementation.
#[async_trait]
pub trait AtlasApp: Send + Sync {
    /// A unique, namespace-friendly identifier (e.g., "anchor", "crm", "telemetry")
    fn app_id(&self) -> &'static str;

    /// Exposes the Axum router containing the app's native endpoints accessible without authentication.
    /// State is applied once inside this method via `.with_state(db)`.
    fn public_router(&self, state: DatabaseConnection) -> Router<DatabaseConnection>;

    /// Exposes the Axum router containing the app's native endpoints that explicitly require authentication via core platform middleware.
    /// State is applied once inside this method via `.with_state(db)`.
    fn authenticated_router(&self, state: DatabaseConnection) -> Router<DatabaseConnection>;

    /// Provides SeaORM standard migrations localized strictly to this app.
    /// Validates tenant_id architecture rather than relying on legacy local tables.
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>>;

    /// Defines standard templates and execution capsules for background sync services.
    /// This pattern prevents frontend apps from silently pinging APIs by moving the burden to platform pollers.
    fn background_jobs(&self) -> Vec<BackgroundJob>;

    // ── Provisioning Contract ─────────────────────────────────────────────────

    /// Called when a new tenant is onboarded to this app.
    ///
    /// Override to create default pages, menus, app_instance records, onboarding
    /// seeds, or any other initial state a new tenant needs to get started.
    ///
    /// The implementation should be idempotent — safe to call multiple times
    /// on the same tenant without corrupting existing data (use ON CONFLICT DO NOTHING
    /// or EXISTS guards).
    ///
    /// Default: no-op. Returns `Ok(())` without touching the database.
    async fn provision(&self, _db: &DatabaseConnection, _tenant_id: Uuid) -> Result<(), String> {
        Ok(())
    }

    // ── Seed Pack Contract ────────────────────────────────────────────────────

    /// Returns the list of demo/test seed packs this app offers.
    /// Each pack can be applied from the platform-admin to a specific tenant instance.
    ///
    /// Seed packs are intended for demos, development, and UAT — not for production
    /// content seeding. They are idempotent on global tables (ON CONFLICT DO NOTHING)
    /// and can be applied multiple times; each application is timestamped in `tenant_setting`.
    ///
    /// Default implementation returns an empty list (no seed packs available).
    fn seed_packs(&self) -> Vec<AppSeedPack> {
        vec![]
    }

    /// Returns the default admin module set for this app.
    ///
    /// Each tuple is `(AdminModuleType, display_name, sort_order, is_fixed)`.
    /// This is used by `provision()` to seed the `app_instance_module` table when
    /// a new tenant is onboarded to this app.
    ///
    /// Fixed modules (Dashboard, Settings, Security) should always be included.
    /// The `is_fixed` flag prevents them from being disabled via Platform Admin.
    ///
    /// Default implementation returns an empty list. Apps MUST override this.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn default_modules(&self) -> Vec<(AdminModuleType, &'static str, i32, bool)> {
    ///     vec![
    ///         (AdminModuleType::Dashboard, "Dashboard", 0,  true),
    ///         (AdminModuleType::Blog,      "Blog",      10, false),
    ///         (AdminModuleType::Settings,  "Settings",  20, true),
    ///         (AdminModuleType::Security,  "Security",  30, true),
    ///     ]
    /// }
    /// ```
    fn default_modules(&self) -> Vec<(crate::models::admin_module::AdminModuleType, &'static str, i32, bool)> {
        vec![]
    }


    /// Returns the ordered list of onboarding steps this app requires before it is
    /// considered "live-ready". Each step is declarative and serializable so the
    /// frontend wizard can render it without knowing app internals.
    ///
    /// Default implementation returns an empty list (no onboarding required).
    fn onboarding_steps(&self) -> Vec<OnboardingStep> {
        vec![]
    }

    /// Evaluates which onboarding steps are still incomplete for a specific tenant.
    /// Returns a Vec of step IDs that are NOT yet done. Empty = app is ready to go live.
    ///
    /// The default implementation uses `onboarding_steps()` and evaluates each
    /// `StepCompletionCheck` against the database. Apps may override for custom logic.
    ///
    /// Performance: TenantSetting checks are batched into a single query evaluated in
    /// memory, rather than issuing one COUNT per step (N+1 anti-pattern).
    async fn onboarding_readiness(
        &self,
        db: &DatabaseConnection,
        tenant_id: Uuid,
        app_instance_id: Uuid,
    ) -> Result<Vec<String>, String> {
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
        use std::collections::HashSet;

        let steps = self.onboarding_steps();
        let required_steps: Vec<_> = steps.iter().filter(|s| s.is_required).collect();

        // ── Batch fetch: load all non-empty TenantSetting keys in a single query ──
        // This replaces the N+1 pattern (one COUNT query per TenantSettingExists step).
        let existing_setting_keys: HashSet<String> = crate::entities::tenant_setting::Entity::find()
            .filter(crate::entities::tenant_setting::Column::TenantId.eq(tenant_id))
            .filter(crate::entities::tenant_setting::Column::Value.ne(""))
            .all(db)
            .await
            .map_err(|e| format!("DB error fetching tenant_settings: {e}"))?
            .into_iter()
            .map(|r| r.key)
            .collect();

        let mut incomplete: Vec<String> = Vec::new();

        for step in &required_steps {
            let done = match &step.completion_check {
                StepCompletionCheck::TenantSettingExists { key } => {
                    // In-memory lookup — no additional DB roundtrip needed.
                    existing_setting_keys.contains(key.as_str())
                }
                StepCompletionCheck::AppDomainExists => {
                    crate::entities::app_domain::Entity::find()
                        .filter(crate::entities::app_domain::Column::AppInstanceId.eq(app_instance_id))
                        .count(db)
                        .await
                        .map(|c| c > 0)
                        .unwrap_or(false)
                }
                StepCompletionCheck::EntityCountGte { table, min } => {
                    // Use sea_query to build a safe, idiomatic COUNT query.
                    // `table` is &'static str so no injection is possible, but
                    // using the query builder is the correct architectural pattern.
                    use sea_orm::sea_query::{Alias, Expr, Query, SelectStatement};
                    use sea_orm::{ConnectionTrait, Statement};

                    let stmt: SelectStatement = Query::select()
                        .expr(Expr::col(sea_orm::sea_query::Asterisk).count())
                        .from(Alias::new(*table))
                        .and_where(
                            Expr::col(Alias::new("tenant_id")).eq(tenant_id.to_string())
                        )
                        .to_owned();

                    let (sql, values) = stmt.build(sea_orm::sea_query::PostgresQueryBuilder);
                    let result = db.query_one(Statement::from_sql_and_values(
                        sea_orm::DatabaseBackend::Postgres,
                        &sql,
                        values,
                    )).await;
                    match result {
                        Ok(Some(row)) => {
                            let count: i64 = row.try_get("", "count").unwrap_or(0);
                            count >= *min as i64
                        }
                        _ => false,
                    }
                }
                StepCompletionCheck::Custom => {
                    // Custom steps must be handled by the app's own override of
                    // `onboarding_readiness`. Here we conservatively mark as incomplete.
                    false
                }
            };

            if !done {
                incomplete.push(step.id.clone());
            }
        }

        Ok(incomplete)
    }
}

// ==========================================
// UNIT TESTS
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use sea_orm_migration::prelude::*;
    use serde_json::json;

    // A mock migration for testing
    struct MockMigration;

    impl MigrationName for MockMigration {
        fn name(&self) -> &str {
            "m20260101_000000_mock_migration"
        }
    }

    #[async_trait]
    impl MigrationTrait for MockMigration {
        async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }

        async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }
    }

    // A dummy compliant application wrapping all required behaviors
    struct DummyApp;

    #[async_trait]
    impl AtlasApp for DummyApp {
        fn app_id(&self) -> &'static str {
            "dummy_app"
        }

        fn public_router(&self, _state: DatabaseConnection) -> Router<DatabaseConnection> {
            Router::new().route("/dummy", get(|| async { "Hello from Dummy" }))
        }

        fn authenticated_router(&self, _state: DatabaseConnection) -> Router<DatabaseConnection> {
            Router::new()
        }

        fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
            vec![Box::new(MockMigration)]
        }

        fn background_jobs(&self) -> Vec<BackgroundJob> {
            vec![BackgroundJob {
                job_type: "dummy_sync".to_string(),
                default_interval_seconds: 60,
                is_active_by_default: true,
                default_config_payload: Some(json!({"target": "test"})),
                executor: Box::new(|_db, _tenant, payload| {
                    Box::pin(async move {
                        if payload.is_some() {
                            Ok(())
                        } else {
                            Err("Missing configuration".to_string())
                        }
                    })
                }),
            }]
        }

        fn onboarding_steps(&self) -> Vec<OnboardingStep> {
            vec![
                OnboardingStep {
                    id: "identity".to_string(),
                    title: "Brand Identity".to_string(),
                    description: "Set your site name.".to_string(),
                    is_required: true,
                    position: 1,
                    completion_check: StepCompletionCheck::TenantSettingExists {
                        key: "site_title".to_string(),
                    },
                },
            ]
        }
    }

    #[tokio::test]
    async fn test_atlas_app_encapsulation_compliance() {
        let app = DummyApp;
        
        // 1. Verify Application Identifier Isolation
        assert_eq!(app.app_id(), "dummy_app");

        // 2. Verify Migrations payload
        let migrations = app.migrations();
        assert_eq!(migrations.len(), 1);

        // 3. Verify Background Jobs Encapsulation
        let jobs = app.background_jobs();
        assert_eq!(jobs.len(), 1);
        
        let job = &jobs[0];
        assert_eq!(job.job_type, "dummy_sync");
        assert_eq!(job.default_interval_seconds, 60);

        // 4. Verify Onboarding Steps
        let steps = app.onboarding_steps();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "identity");
        assert!(steps[0].is_required);

        // 5. Verify Execution Closure executes within correct Isolated Context
        let db = sea_orm::Database::connect("sqlite::memory:").await.expect("Failed to create in-memory test db");

        let result = (job.executor)(db.clone(), uuid::Uuid::nil(), job.default_config_payload.clone()).await;
        assert!(result.is_ok(), "The job executor closure should resolve to Ok if payload exists");

        let missing_payload_result = (job.executor)(db, uuid::Uuid::nil(), None).await;
        assert!(missing_payload_result.is_err(), "The job executor closure should throw Err if payload is missing");
    }
}
