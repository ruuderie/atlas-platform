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

    fn authenticated_router(&self, db: DatabaseConnection) -> Router<DatabaseConnection> {
        Router::new()
            .merge(crate::handlers::anchor::pages::authenticated_routes(db))
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        // ═══════════════════════════════════════════════════════════════════════
        // ANCHOR MIGRATION REGISTRY — Single source of truth for all Anchor
        // app migrations. Ordered chronologically by filename date+sequence.
        //
        // RULES:
        //   1. All Anchor-specific migrations live HERE. Never in mod.rs base vec.
        //   2. Append new migrations at the BOTTOM only — never insert mid-list.
        //   3. Data migrations must use RAISE EXCEPTION for missing tenants and
        //      zero-row UPDATEs. Silent failure is not acceptable.
        //   4. pub mod declarations must exist in migration/mod.rs for each entry.
        // ═══════════════════════════════════════════════════════════════════════

        // --- Anchor schema: tables & background job seeds ---
        // The legacy table migration is kept in history so the migration runner's
        // applied-migrations log remains consistent on existing environments.
        // The subsequent drop migration tears down those tables on any environment
        // where they were created, and is a no-op on fresh clean installs.
        //
        // NOTE: Migrations up to m20260417_000002_fix_buildwithruud_pages are
        // registered in migration/mod.rs (base platform list) and must NOT be
        // duplicated here. anchor.rs only contains migrations that run AFTER
        // the base platform has been fully initialized.
        vec![
            Box::new(crate::migration::m20260408_000002_create_anchor_legacy_tables::Migration),
            Box::new(crate::migration::m20260408_000003_seed_anchor_background_jobs::Migration),
            Box::new(crate::migration::m20260408_000004_fix_anchor_tables_and_seed::Migration),

            // --- Anchor-only: formbuilder page seeds (after base form_engine schema) ---
            Box::new(crate::migration::m20260417_000003_seed_formbuilder_pages::Migration),

            // --- buildwithruud home page: layout migration chain ---
            Box::new(crate::migration::m20260425_000001_update_buildwithruud_home::Migration),
            Box::new(crate::migration::m20260425_000002_create_footer_items_table::Migration),
            Box::new(crate::migration::m20260425_000003_fix_buildwithruud_padding::Migration),
            Box::new(crate::migration::m20260425_000004_stitch_ruuderie_payload::Migration),
            Box::new(crate::migration::m20260425_000005_fix_ruud_tenant_lookup::Migration),
            Box::new(crate::migration::m20260425_000006_force_ruud_payload::Migration),

            // --- Hardened canonical payload fix ---
            Box::new(crate::migration::m20260426_000001_hardened_ruud_payload::Migration),

            // --- UAT stabilization ---
            Box::new(crate::migration::m20260427_000001_restore_consulting_page::Migration),
            Box::new(crate::migration::m20260427_000002_real_estate_ventures_redesign::Migration),
            Box::new(crate::migration::m20260427_000003_widget_instance_config::Migration),
            Box::new(crate::migration::m20260427_000005_seed_p_vs_np_blog_post::Migration),
            Box::new(crate::migration::m20260427_000006_real_estate_newsletter_form::Migration),
            Box::new(crate::migration::m20260427_000007_seed_kami_resume_profile::Migration),
            Box::new(crate::migration::m20260427_000008_fix_p_vs_np_math_delimiters::Migration),
            Box::new(crate::migration::m20260427_000009_blog_download_leads::Migration),
            Box::new(crate::migration::m20260427_000010_enable_kami_mode_buildwithruud::Migration),
            Box::new(crate::migration::m20260427_000011_kami_projects_layout::Migration),

            // --- Onboarding system: drop legacy anchor tables (idempotent) ---
            Box::new(crate::migration::m20260430_000001_drop_anchor_legacy_tables::Migration),

            // --- Data integrity: canonicalize tenant_setting from app_instances.settings ---
            // Fixes the UAT content gap (2026-04-30): settings were stored in app_instances.settings
            // but get_site_settings() reads tenant_setting. Also fixes lc_* → lead_capture_* key mismatch.
            Box::new(crate::migration::m20260501_000001_canonicalize_tenant_settings::Migration),
        ]
    }

    fn background_jobs(&self) -> Vec<BackgroundJob> {
        vec![
            BackgroundJob {
                job_type: "BitcoinSync".to_string(),
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
