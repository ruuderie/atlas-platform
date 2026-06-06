use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// G-27 Patch: Add `deleted_at` soft-delete column to `atlas_scorecards`.
///
/// This migration exists for databases that already applied `m20260701_g27_scorecards`
/// before `deleted_at` was added to that migration's ALTER TABLE block.
///
/// `deleted_at` is referenced in `m20260711_g27_data_science_v3` by:
///   - `mv_scorecard_portfolio_analytics` → `WHERE sc.deleted_at IS NULL`
///   - `v_scorecard_recent_anomalies`     → `WHERE sc.deleted_at IS NULL`
///
/// The `IF NOT EXISTS` guard makes this idempotent — fresh databases that received
/// the column from `m20260701` will skip the add silently.
///
/// File name prefix `m20260711_0_` sorts before `m20260711_g27_data_science_v3`
/// in SeaORM's alphabetical migration ordering, guaranteeing the column exists
/// when the view DDL runs.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Add soft-delete column — NULL means active, non-null means archived.
        // IF NOT EXISTS makes this safe to re-run on fresh databases.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecards \
             ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ DEFAULT NULL;"
                .to_owned(),
        ))
        .await?;

        // Partial index: exclude archived scorecards from The Combinator query paths.
        // Used by find_similar() and the portfolio analytics view.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_scorecards_active \
             ON atlas_scorecards (tenant_id, template_id) \
             WHERE deleted_at IS NULL;"
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
            "DROP INDEX IF EXISTS idx_atlas_scorecards_active;".to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecards DROP COLUMN IF EXISTS deleted_at;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }
}
