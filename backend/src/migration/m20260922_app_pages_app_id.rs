use sea_orm_migration::prelude::*;

/// Add `app_id` to `app_pages` so pages can be scoped to a platform product
/// (e.g. "folio", "ruuderie", "network") rather than only to a tenant.
///
/// This is required by the platform-admin Landing Page Builder, which manages
/// pages as app-level acquisition assets — not tenant-scoped content.
///
/// Existing rows are backfilled to `'folio'` (all historical pages belong to Folio).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE app_pages
                    ADD COLUMN IF NOT EXISTS app_id TEXT NOT NULL DEFAULT 'folio';

                 -- Backfill all existing rows to folio (correct — only Folio pages exist today)
                 UPDATE app_pages SET app_id = 'folio' WHERE app_id IS NULL OR app_id = '';

                 CREATE INDEX IF NOT EXISTS idx_app_pages_app_id
                     ON app_pages (app_id);

                 CREATE INDEX IF NOT EXISTS idx_app_pages_app_id_published
                     ON app_pages (app_id, is_published);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_app_pages_app_id_published;
                 DROP INDEX IF EXISTS idx_app_pages_app_id;
                 ALTER TABLE app_pages DROP COLUMN IF EXISTS app_id;",
            )
            .await?;

        Ok(())
    }
}
