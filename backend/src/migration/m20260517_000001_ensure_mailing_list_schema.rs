use sea_orm_migration::prelude::*;

/// Ensures the `mailing_list` table exists with the correct schema expected by
/// `get_mailing_list` in anchor/src/pages/admin.rs.
///
/// ## Background
/// The `mailing_list` table was created by an early legacy Anchor migration.
/// Migration `m20260404_000004_crm_mailing_list_migration` was intended to port
/// its data to the CRM `lead` table and drop it, but that migration was never
/// registered in `Migrator::migrations()` or any `AtlasApp::migrations()` block.
///
/// As a result, the table exists in production/dev DBs but may have a schema that
/// is missing the `preferences JSONB` column, which causes `get_mailing_list` to
/// return a `ServerFnError` (manifesting as `ERR_NO_DATA` in the admin UI).
///
/// This migration uses `IF NOT EXISTS` and `ADD COLUMN IF NOT EXISTS` so it is
/// safe to run against a DB where the table already has the correct schema.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create table if it doesn't exist yet (fresh DB or if legacy table was dropped).
        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS mailing_list (
                id          SERIAL PRIMARY KEY,
                email       TEXT NOT NULL,
                list_type   TEXT NOT NULL DEFAULT 'general',
                preferences JSONB,
                tenant_id   UUID,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#,
        )
        .await?;

        // Add missing columns defensively in case the table exists with old schema.
        db.execute_unprepared(
            "ALTER TABLE mailing_list ADD COLUMN IF NOT EXISTS preferences JSONB;",
        )
        .await?;

        db.execute_unprepared("ALTER TABLE mailing_list ADD COLUMN IF NOT EXISTS tenant_id UUID;")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the table entirely on rollback — consistent with the intended
        // final state of `m20260404_000004_crm_mailing_list_migration`.
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS mailing_list CASCADE;")
            .await?;
        Ok(())
    }
}
