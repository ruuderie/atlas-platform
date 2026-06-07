use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Adds `display_config` JSONB to `atlas_scorecard_templates`.
///
/// This column drives the G-27 Context-Aware Display Rules engine:
/// it stores a `ScorecardTemplateDisplayConfig` blob controlling which platform
/// surfaces render the scorecard (portfolio table, anomaly panel, leaderboard, etc.)
/// and whether nudge WebSocket pushes fire on case-close or STR checkout.
///
/// All fields in the JSONB default to `false` / `null` — explicit opt-in required.
/// Existing rows get NULL (all surfaces off) without any backfill — backward compat.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD COLUMN IF NOT EXISTS display_config JSONB;".to_owned(),
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
             DROP COLUMN IF EXISTS display_config;".to_owned(),
        ))
        .await?;

        Ok(())
    }
}
