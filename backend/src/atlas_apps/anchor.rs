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
        // Extract isolated Anchor specific tables correctly away from Core Infrastructure.
        vec![
            Box::new(crate::migration::m20260408_000002_create_anchor_legacy_tables::Migration),
            Box::new(crate::migration::m20260408_000003_seed_anchor_background_jobs::Migration),
            Box::new(crate::migration::m20260408_000004_fix_anchor_tables_and_seed::Migration),
            Box::new(crate::migration::m20260425_000002_create_footer_items_table::Migration),
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
}
