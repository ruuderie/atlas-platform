/// G-19 Gap-Fill: Add parent_campaign_id to atlas_campaigns
///
/// The original m20260804_g19_atlas_campaigns migration defined parent_campaign_id
/// in the entity model and query code, but the DB migration that created the table
/// did NOT include the column. This migration adds it as a nullable self-referential FK.
///
/// Impact: Resolves the "column atlas_campaigns.parent_campaign_id does not exist"
///         Postgres 500 error on every admin campaigns API call.
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260919_g19_campaigns_parent_id"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add the column only if it doesn't already exist (idempotent guard).
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'atlas_campaigns'
                      AND column_name = 'parent_campaign_id'
                ) THEN
                    ALTER TABLE atlas_campaigns
                        ADD COLUMN parent_campaign_id UUID
                        REFERENCES atlas_campaigns(id) ON DELETE SET NULL;

                    CREATE INDEX CONCURRENTLY IF NOT EXISTS
                        idx_atlas_campaigns_parent_id
                        ON atlas_campaigns(parent_campaign_id)
                        WHERE parent_campaign_id IS NOT NULL;
                END IF;
            END
            $$;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'atlas_campaigns'
                      AND column_name = 'parent_campaign_id'
                ) THEN
                    ALTER TABLE atlas_campaigns
                        DROP COLUMN parent_campaign_id;
                END IF;
            END
            $$;
            "#,
        )
        .await?;

        Ok(())
    }
}
