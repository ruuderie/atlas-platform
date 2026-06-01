use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-28: atlas_note — Universal Polymorphic Note
///
/// Promotes the existing `notes` table to a platform generic by:
///   1. Adding the platform-standard columns: `note_type`, `subject`, `visibility`,
///      `is_pinned`, `note_metadata JSONB`, `parent_note_id` (threads).
///   2. Creating all performance indexes as raw SQL.
///   3. Attaching the `set_updated_at_column()` trigger (created by G-31).
///   4. Tightening the NULL constraint on `tenant_id` (backfill ran in m20260525).
///
/// The `notes` table is NOT renamed — `atlas_note` is the entity abstraction, but
/// the underlying Postgres table stays `notes` for backward compatibility with all
/// existing joins and handlers. A future migration can rename when safe.
///
/// Any entity on the platform can attach notes by setting:
///   entity_type = 'atlas_asset' | 'atlas_account' | 'atlas_contact' |
///                 'atlas_lead' | 'atlas_opportunity' | 'atlas_case' |
///                 'atlas_contract' | 'atlas_application' | 'atlas_service_provider'
///
/// Spec: docs/architecture/platform_generics_v2.md (GENERIC-28)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── 1. Promote notes table ────────────────────────────────────────────
        // Add platform-generic columns using IF NOT EXISTS (idempotent).
        // tenant_id was backfilled in m20260525; tighten to NOT NULL.
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
            ALTER TABLE notes
                -- Note classification discriminator
                ADD COLUMN IF NOT EXISTS note_type VARCHAR(50) NOT NULL DEFAULT 'general',
                -- Short heading displayed in feed views (content remains the body)
                ADD COLUMN IF NOT EXISTS subject VARCHAR(500),
                -- 'public' | 'internal' | 'private' (supercedes is_private, kept for compat)
                ADD COLUMN IF NOT EXISTS visibility VARCHAR(20) NOT NULL DEFAULT 'internal',
                -- Pinned notes surface at the top of the entity note feed
                ADD COLUMN IF NOT EXISTS is_pinned BOOLEAN NOT NULL DEFAULT false,
                -- Thread support: parent_note_id → self-referential FK
                ADD COLUMN IF NOT EXISTS parent_note_id UUID REFERENCES notes(id) ON DELETE CASCADE,
                -- Arbitrary app-specific payload (rich text delta, call transcript, etc.)
                ADD COLUMN IF NOT EXISTS note_metadata JSONB;
            "#
            .to_owned(),
        ))
        .await?;

        // Tighten tenant_id: previous migration backfilled all nulls, now enforce.
        // Wrapped in DO/EXCEPTION so the migration is re-runnable.
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
            DO $$
            BEGIN
                ALTER TABLE notes ALTER COLUMN tenant_id SET NOT NULL;
            EXCEPTION WHEN others THEN
                NULL; -- already NOT NULL
            END $$;
            "#
            .to_owned(),
        ))
        .await?;

        // ── 2. Indexes ────────────────────────────────────────────────────────
        // All partial indexes emitted as raw SQL per platform pattern.

        // Primary access pattern: entity feed, newest-first
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_notes_entity_feed \
             ON notes (tenant_id, entity_type, entity_id, created_at DESC);",
        ))
        .await?;

        // Pinned notes filter (sparse — most notes are not pinned)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_notes_pinned \
             ON notes (tenant_id, entity_type, entity_id) \
             WHERE is_pinned = true;",
        ))
        .await?;

        // Thread replies lookup
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_notes_thread \
             ON notes (parent_note_id) \
             WHERE parent_note_id IS NOT NULL;",
        ))
        .await?;

        // Note type filter (e.g. 'call_log', 'site_visit', 'underwriting_comment')
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_notes_type \
             ON notes (tenant_id, note_type, created_at DESC);",
        ))
        .await?;

        // Private notes lookup (is_private kept for compat; visibility is canonical)
        // Drop the old idx_notes_creator_private if it exists (replaced above)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP INDEX IF EXISTS idx_notes_creator_private;",
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_notes_visibility \
             ON notes (tenant_id, created_by, visibility, created_at DESC);",
        ))
        .await?;

        // ── 3. updated_at trigger ─────────────────────────────────────────────
        // set_updated_at_column() function was created in m20260601_g31_atlas_lead.
        // G-28 (m20260703_) sorts after G-31 (m20260601_), so function exists.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE OR REPLACE TRIGGER trg_notes_updated_at \
             BEFORE UPDATE ON notes \
             FOR EACH ROW EXECUTE FUNCTION set_updated_at_column();",
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP TRIGGER IF EXISTS trg_notes_updated_at ON notes;",
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP INDEX IF EXISTS idx_notes_entity_feed; \
             DROP INDEX IF EXISTS idx_notes_pinned; \
             DROP INDEX IF EXISTS idx_notes_thread; \
             DROP INDEX IF EXISTS idx_notes_type; \
             DROP INDEX IF EXISTS idx_notes_visibility;",
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE notes \
             DROP COLUMN IF EXISTS note_type, \
             DROP COLUMN IF EXISTS subject, \
             DROP COLUMN IF EXISTS visibility, \
             DROP COLUMN IF EXISTS is_pinned, \
             DROP COLUMN IF EXISTS parent_note_id, \
             DROP COLUMN IF EXISTS note_metadata;",
        ))
        .await?;

        Ok(())
    }
}
