use sea_orm_migration::prelude::*;

/// `app_page_variants` — A/B test variants for platform-admin landing pages.
///
/// One `app_page` (control) maps to N variants. Each variant overrides the
/// parent page's `blocks_payload` / `hero_payload` independently.
///
/// Traffic allocation:
///   - `traffic_pct` integers across all active variants for a page must sum to 100.
///   - The routing layer (future) uses a deterministic hash on visitor fingerprint
///     to assign variant. For now the platform-admin UI enforces the split.
///
/// Promotion flow:
///   When a variant is promoted, its `blocks_payload` and `hero_payload` are
///   copied back to the parent `app_pages` row and all variants are deleted.
///   This is handled at the handler layer (no DB trigger needed).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS app_page_variants (
                    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    page_id         UUID        NOT NULL
                                    REFERENCES app_pages(id) ON DELETE CASCADE,
                    name            TEXT        NOT NULL,
                    -- Traffic split in whole-number percent (0–100)
                    traffic_pct     INTEGER     NOT NULL DEFAULT 50
                                    CONSTRAINT chk_variant_traffic_pct
                                    CHECK (traffic_pct >= 0 AND traffic_pct <= 100),
                    is_control      BOOLEAN     NOT NULL DEFAULT false,
                    -- Block-level content overrides (full payload, not delta)
                    blocks_payload  JSONB       NOT NULL DEFAULT '[]',
                    hero_payload    JSONB,
                    -- Denormalized engagement counters (incremented by telemetry)
                    view_count      INTEGER     NOT NULL DEFAULT 0,
                    lead_count      INTEGER     NOT NULL DEFAULT 0,
                    is_active       BOOLEAN     NOT NULL DEFAULT true,
                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_app_page_variants_page_id
                    ON app_page_variants (page_id);

                CREATE INDEX IF NOT EXISTS idx_app_page_variants_active
                    ON app_page_variants (page_id, is_active);

                -- updated_at auto-maintenance trigger
                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_app_page_variants
                        BEFORE UPDATE ON app_page_variants
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
                "DROP TRIGGER IF EXISTS set_updated_at_app_page_variants ON app_page_variants;
                 DROP TABLE IF EXISTS app_page_variants;",
            )
            .await?;

        Ok(())
    }
}
