use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Drop Legacy `lead` table + Rename Compat View
///
/// Prerequisites (enforced by sort order):
///   - m20260601_g31_atlas_lead: `atlas_lead` table + `atlas_lead_compat_view` exist.
///   - All handlers that previously selected from `lead` now select from `atlas_lead`
///     or via the compat view.
///
/// What this migration does:
///   1. Renames `atlas_lead_compat_view` to `lead` so that any remaining old
///      SELECT calls against `lead` keep working transparently.
///   2. Drops the legacy `lead` table.
///
/// Order matters:
///   - Must RENAME VIEW before DROP TABLE, because the view currently SELECTs
///     from `atlas_lead` (not `lead`), so dropping `lead` first is harmless, but
///     renaming first is the safe, explicit sequence.
///
/// Down migration recreates the legacy `lead` table as an empty shell with the
/// original columns so the migration can be safely reversed during development.
/// In production, `down` should never be run — it is provided for test cleanup only.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Step 1: Rename compat view so old SELECT ... FROM lead still works.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER VIEW atlas_lead_compat_view RENAME TO lead_view;".to_owned(),
        ))
        .await?;

        // Step 2: Drop the legacy lead table.
        // Foreign keys from other legacy tables (activity.lead_id, notes.entity_id
        // where entity_type='Lead', etc.) are soft references — no FK constraint
        // enforced at the DB level in the legacy schema.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP TABLE IF EXISTS lead CASCADE;".to_owned(),
        ))
        .await?;

        // Step 3: Re-expose the view as `lead` (in addition to `lead_view`) so
        // any callers that referenced `lead` by name still resolve.
        // This makes the rename fully transparent.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE OR REPLACE VIEW lead AS SELECT * FROM lead_view;".to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Drop the thin wrapper view
        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP VIEW IF EXISTS lead;".to_owned(),
        ))
        .await?;

        // Restore the legacy lead table as a minimal shell (dev/test rollback only).
        // Data is NOT restored — this is for schema rollback, not data rollback.
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
            CREATE TABLE IF NOT EXISTS lead (
                id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id    UUID,
                first_name   VARCHAR(100),
                last_name    VARCHAR(100),
                email        VARCHAR(255),
                phone        VARCHAR(50),
                company      VARCHAR(255),
                status       VARCHAR(50) NOT NULL DEFAULT 'new',
                source       VARCHAR(100),
                created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#
            .to_owned(),
        ))
        .await?;

        // Restore the compat view name
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER VIEW lead_view RENAME TO atlas_lead_compat_view;".to_owned(),
        ))
        .await?;

        Ok(())
    }
}
