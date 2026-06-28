use sea_orm_migration::prelude::*;

/// `app_utm_presets` — Reusable UTM parameter sets for platform-admin landing pages.
///
/// Presets are scoped to an `app_id` so each platform product manages its own
/// campaign tracking templates independently.
///
/// A preset combines the 5 standard UTM parameters with a human-readable `name`
/// and a denormalized `click_count` that is incremented by the link-click webhook
/// (when integrated). The URL builder in the platform-admin UI generates tagged
/// URLs by combining a preset with a specific page slug and domain.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS app_utm_presets (
                    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    app_id          TEXT        NOT NULL DEFAULT 'folio',
                    name            TEXT        NOT NULL,
                    -- Core UTM parameters (source + medium + campaign are required)
                    utm_source      TEXT        NOT NULL,
                    utm_medium      TEXT        NOT NULL,
                    utm_campaign    TEXT        NOT NULL,
                    -- Optional UTM parameters
                    utm_content     TEXT,
                    utm_term        TEXT,
                    -- Denormalized engagement counter
                    click_count     INTEGER     NOT NULL DEFAULT 0,
                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_app_utm_presets_app_id
                    ON app_utm_presets (app_id);

                -- updated_at auto-maintenance trigger
                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_app_utm_presets
                        BEFORE UPDATE ON app_utm_presets
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                EXCEPTION WHEN duplicate_object THEN NULL; END $$;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS set_updated_at_app_utm_presets ON app_utm_presets;
                 DROP TABLE IF EXISTS app_utm_presets;",
            )
            .await?;

        Ok(())
    }
}
