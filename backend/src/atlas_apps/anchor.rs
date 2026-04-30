use crate::traits::atlas_app::{AtlasApp, BackgroundJob, OnboardingStep, StepCompletionCheck};
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

    fn public_router(&self, _state: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new()
    }

    fn authenticated_router(&self, _state: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new()
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        // The legacy table migration is kept in history so the migration runner's
        // applied-migrations log remains consistent on existing environments.
        // The subsequent drop migration tears down those tables on any environment
        // where they were created, and is a no-op on fresh clean installs.
        vec![
            Box::new(crate::migration::m20260408_000002_create_anchor_legacy_tables::Migration),
            Box::new(crate::migration::m20260408_000003_seed_anchor_background_jobs::Migration),
            Box::new(crate::migration::m20260408_000004_fix_anchor_tables_and_seed::Migration),
            Box::new(crate::migration::m20260430_000001_drop_anchor_legacy_tables::Migration),
        ]
    }

    fn background_jobs(&self) -> Vec<BackgroundJob> {
        vec![
            BackgroundJob {
                job_type: "BitcoinSync".to_string(), // Matching the explicit trigger previously hardcoded
                default_interval_seconds: 600,
                is_active_by_default: true,
                default_config_payload: None,
                executor: Box::new(|db, tenant_id, _config| {
                    Box::pin(async move {
                        crate::services::data_sync::DataSyncService::sync_bitcoin_blocks(
                            &db, 
                            tenant_id, 
                            "https://mempool.space/api/blocks"
                        )
                        .await
                        .map_err(|e| e.to_string())
                    })
                })
            }
        ]
    }

    /// Anchor onboarding steps, in the order the wizard presents them.
    /// Steps are evaluated server-side against real data — not flags.
    /// `position` is explicit so the frontend wizard has a stable sort key
    /// that is independent of Vec insertion order.
    fn onboarding_steps(&self) -> Vec<OnboardingStep> {
        vec![
            OnboardingStep {
                id: "identity".to_string(),
                title: "Brand Identity".to_string(),
                description: "Set your site name and tagline so visitors know who you are.".to_string(),
                is_required: true,
                position: 1,
                completion_check: StepCompletionCheck::TenantSettingExists {
                    key: "site_title".to_string(),
                },
            },
            OnboardingStep {
                id: "domain".to_string(),
                title: "Custom Domain".to_string(),
                description: "Connect your domain so your site has its live web address.".to_string(),
                is_required: true,
                position: 2,
                completion_check: StepCompletionCheck::AppDomainExists,
            },
            OnboardingStep {
                id: "design".to_string(),
                title: "Design Theme".to_string(),
                description: "Choose your color palette and typography to match your brand.".to_string(),
                is_required: true,
                position: 3,
                completion_check: StepCompletionCheck::TenantSettingExists {
                    key: "design_config".to_string(),
                },
            },
            OnboardingStep {
                id: "first_page".to_string(),
                title: "Your First Page".to_string(),
                description: "Create your home page so your site has something to show visitors.".to_string(),
                is_required: true,
                position: 4,
                completion_check: StepCompletionCheck::EntityCountGte {
                    table: "app_page",
                    min: 1,
                },
            },
            OnboardingStep {
                id: "audience_mode".to_string(),
                title: "Audience Mode".to_string(),
                description: "Tell us whether your site targets businesses (B2B) or consumers (B2C).".to_string(),
                is_required: false,
                position: 5,
                completion_check: StepCompletionCheck::TenantSettingExists {
                    key: "b2b_mode".to_string(),
                },
            },
        ]
    }
}
