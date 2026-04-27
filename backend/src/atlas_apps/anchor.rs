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
        vec![
            Box::new(crate::migration::m20260408_000002_create_anchor_legacy_tables::Migration),
            Box::new(crate::migration::m20260408_000003_seed_anchor_background_jobs::Migration),
            Box::new(crate::migration::m20260408_000004_fix_anchor_tables_and_seed::Migration),
            Box::new(crate::migration::m20260408_000006_create_app_content::Migration),

            // --- Form engine & tenant configuration ---
            Box::new(crate::migration::m20260412_000001_form_engine::Migration),
            Box::new(crate::migration::m20260412_000002_add_tenant_slug::Migration),

            // --- Tenant seeds: OplystUSA ---
            Box::new(crate::migration::m20260412_000003_seed_oplystusa::Migration),
            Box::new(crate::migration::m20260413_000001_seed_oplystusa_domains::Migration),
            Box::new(crate::migration::m20260415_000001_seed_oplystusa_home_page::Migration),
            Box::new(crate::migration::m20260415_000003_seed_oplystusa_pages::Migration),

            // --- Tenant seeds: buildwithruud ---
            Box::new(crate::migration::m20260415_000002_upgrade_buildwithruud_home_page::Migration),
            Box::new(crate::migration::m20260416_000001_rename_resume_tables::Migration),
            Box::new(crate::migration::m20260416_000002_seed_buildwithruud_block_pages::Migration),
            Box::new(crate::migration::m20260417_000001_seed_design_system_config::Migration),
            Box::new(crate::migration::m20260417_000002_fix_buildwithruud_pages::Migration),
            Box::new(crate::migration::m20260417_000003_seed_formbuilder_pages::Migration),

            // --- buildwithruud home page: layout migration chain ---
            // These patch the home payload from the old hardcoded-padding design
            // to the new block-owns-its-own-layout architecture (pt-32 in payload).
            Box::new(crate::migration::m20260425_000001_update_buildwithruud_home::Migration),
            Box::new(crate::migration::m20260425_000002_create_footer_items_table::Migration),
            Box::new(crate::migration::m20260425_000003_fix_buildwithruud_padding::Migration),
            Box::new(crate::migration::m20260425_000004_stitch_ruuderie_payload::Migration),
            Box::new(crate::migration::m20260425_000005_fix_ruud_tenant_lookup::Migration),
            Box::new(crate::migration::m20260425_000006_force_ruud_payload::Migration),

            // --- Hardened canonical payload fix (RAISE EXCEPTION on failure) ---
            // This is the terminal authoritative migration for the home page layout.
            // It supersedes all above pt-8→pt-32 patches on fresh databases.
            Box::new(crate::migration::m20260426_000001_hardened_ruud_payload::Migration),

            // --- UAT stabilization: layout, content, and widget system ---
            // Consulting page restored (was deleted in m20260417_000003)
            Box::new(crate::migration::m20260427_000001_restore_consulting_page::Migration),
            // Real-estate-ventures redesigned as investor/landlord landing (5 strategy pillars)
            Box::new(crate::migration::m20260427_000002_real_estate_ventures_redesign::Migration),
            // Widget instance config: bitcoin clock for buildwithruud, empty for all others
            Box::new(crate::migration::m20260427_000003_widget_instance_config::Migration),
            // Blog content_format column: supports 'markdown' | 'latex' | 'mdlatex'
            Box::new(crate::migration::m20260427_000004_blog_content_format::Migration),
            // Seed the P vs NP exploratory argument blog post into buildwithruud tenant
            Box::new(crate::migration::m20260427_000005_seed_p_vs_np_blog_post::Migration),
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
}
