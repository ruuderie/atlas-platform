use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// G-27 Patch: Add `deleted_at` soft-delete column to `atlas_scorecard_templates`.
///
/// `m20260711_0_g27_add_deleted_at_scorecards` added `deleted_at` to `atlas_scorecards`
/// but missed `atlas_scorecard_templates`. The seed migration that follows
/// (`m20260712_g27_seed_portfolio_job`) queries:
///
///   SELECT DISTINCT tenant_id FROM atlas_scorecard_templates
///   WHERE is_published = true AND deleted_at IS NULL
///
/// On live databases (UAT/dev) that had `atlas_scorecard_templates` created before
/// soft-delete was added, this column does not exist, causing a panic at startup.
///
/// File name prefix `m20260711_1_` sorts:
///   - AFTER  `m20260711_0_g27_add_deleted_at_scorecards` (scorecards column)
///   - BEFORE `m20260711_g27_data_science_v3`            (views referencing deleted_at)
///   - BEFORE `m20260712_g27_seed_portfolio_job`         (seed job that reads deleted_at)
///
/// `IF NOT EXISTS` makes this idempotent — fresh databases that received the column
/// from the original `m20260701_g27_scorecards` ALTER TABLE block will skip silently.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Add soft-delete sentinel to templates. NULL = active, non-null = archived.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ DEFAULT NULL;"
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
             DROP COLUMN IF EXISTS deleted_at;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }
}
