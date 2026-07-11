use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Phase 0 — PM G-27 Template Scope Columns
///
/// Adds two columns required before PropertyManagementApp templates can be seeded:
///
/// 1. `atlas_scorecard_templates.template_scope` (VARCHAR(20), NOT NULL DEFAULT 'tenant')
///    'platform' = canonical cross-tenant template eligible for benchmark aggregation.
///    'tenant'   = private per-landlord template, excluded from cross-tenant pool.
///
/// 2. `atlas_scorecard_dimensions.is_tenant_extension` (BOOL, NOT NULL DEFAULT false)
///    true  = landlord-added custom dimension, excluded from cross-tenant benchmark pool.
///    false = canonical platform dimension, included in benchmark aggregation.
///
/// Note: display_config JSONB was already added by m20260714_g27_display_config.
/// This migration only adds the two remaining PM prerequisites.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── 1. template_scope on atlas_scorecard_templates ───────────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD COLUMN IF NOT EXISTS template_scope VARCHAR(20) NOT NULL DEFAULT 'tenant' \
             CHECK (template_scope IN ('platform', 'tenant'));"
                .to_owned(),
        ))
        .await?;

        // ── 2. is_tenant_extension on atlas_scorecard_dimensions ─────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             ADD COLUMN IF NOT EXISTS is_tenant_extension BOOLEAN NOT NULL DEFAULT false;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             DROP COLUMN IF EXISTS template_scope;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             DROP COLUMN IF EXISTS is_tenant_extension;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }
}
