use axum::Router;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

/// Represents a dynamic asynchronous executor closure for background jobs.
pub type JobExecutor = Box<
    dyn Fn(DatabaseConnection, uuid::Uuid, Option<serde_json::Value>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;

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
    /// How the backend evaluates whether this step is complete
    pub completion_check: StepCompletionCheck,
}

// ──────────────────────────────────────────────────────────────────────────────
// ATLAS APP TRAIT
// ──────────────────────────────────────────────────────────────────────────────

/// The formal API Contract for any application plugging into the Atlas Platform.
#[async_trait]
pub trait AtlasApp: Send + Sync {
    /// A unique, namespace-friendly identifier (e.g., "anchor", "crm", "telemetry")
    fn app_id(&self) -> &'static str;

    /// Exposes the Axum router containing the app's native endpoints accessible without authentication.
    fn public_router(&self, state: DatabaseConnection) -> Router<DatabaseConnection>;

    /// Exposes the Axum router containing the app's native endpoints that explicitly require authentication via core platform middleware.
    fn authenticated_router(&self, state: DatabaseConnection) -> Router<DatabaseConnection>;

    /// Provides SeaORM standard migrations localized strictly to this app.
    /// Validates tenant_id architecture rather than relying on legacy local tables.
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>>;

    /// Defines standard templates and execution capsules for background sync services.
    /// This pattern prevents frontend apps from silently pinging APIs by moving the burden to platform pollers.
    fn background_jobs(&self) -> Vec<BackgroundJob>;

    // ── Onboarding Contract ───────────────────────────────────────────────────

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
    async fn onboarding_readiness(
        &self,
        db: &DatabaseConnection,
        tenant_id: Uuid,
        app_instance_id: Uuid,
    ) -> Result<Vec<String>, String> {
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};

        let steps = self.onboarding_steps();
        let mut incomplete: Vec<String> = Vec::new();

        for step in &steps {
            if !step.is_required {
                continue;
            }
            let done = match &step.completion_check {
                StepCompletionCheck::TenantSettingExists { key } => {
                    crate::entities::tenant_setting::Entity::find()
                        .filter(crate::entities::tenant_setting::Column::TenantId.eq(tenant_id))
                        .filter(crate::entities::tenant_setting::Column::Key.eq(key.as_str()))
                        .count(db)
                        .await
                        .map(|c| c > 0)
                        .unwrap_or(false)
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
                    // For generic tables we issue a raw count. This is intentionally
                    // kept simple — apps needing complex checks should use Custom.
                    use sea_orm::{ConnectionTrait, Statement};
                    let sql = format!(
                        "SELECT COUNT(*) AS cnt FROM {} WHERE tenant_id = $1",
                        table
                    );
                    let result = db.query_one(Statement::from_sql_and_values(
                        sea_orm::DatabaseBackend::Postgres,
                        &sql,
                        vec![tenant_id.into()],
                    )).await;
                    match result {
                        Ok(Some(row)) => {
                            let count: i64 = row.try_get("", "cnt").unwrap_or(0);
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
