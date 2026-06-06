use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-29: atlas_activity — Universal Polymorphic Activity Log
///
/// Promotes the existing `activity` table to a platform generic by:
///   1. Dropping legacy hard-coded FK columns (deal_id, customer_id, lead_id,
///      contact_id, case_id, account_id) in favour of the polymorphic
///      `subject_entity_type` + `subject_entity_id` pattern.
///   2. Adding platform-generic columns: `activity_category`, `direction`,
///      `duration_seconds`, `outcome`, `scheduled_at`, `activity_metadata JSONB`.
///   3. Creating all performance indexes as raw SQL.
///   4. Attaching the `set_updated_at_column()` trigger.
///
/// MIGRATION STRATEGY — non-destructive:
///   The legacy FK columns (deal_id, customer_id, etc.) are NOT dropped in this
///   migration. They are NULLed-out progressively as handlers migrate to the new
///   polymorphic pattern. A future cleanup migration drops them once all handlers
///   reference `subject_entity_type` / `subject_entity_id`.
///
///   The `associated_entities JSONB` column (added in m20260523) is kept as the
///   multi-entity reference mechanism for activities touching > 1 entity (e.g.
///   a call with a lead AND a contact).
///
/// Spec: docs/architecture/platform_generics_v2.md (GENERIC-29)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── 1. Promote activity table ─────────────────────────────────────────

        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
            ALTER TABLE activity
                -- Primary polymorphic subject (replaces deal_id/lead_id/etc. long-term)
                ADD COLUMN IF NOT EXISTS subject_entity_type VARCHAR(50),
                ADD COLUMN IF NOT EXISTS subject_entity_id   UUID,

                -- Activity category discriminator (broader than activity_type)
                -- 'communication' | 'meeting' | 'task' | 'system_event' | 'pipeline_event'
                ADD COLUMN IF NOT EXISTS activity_category VARCHAR(50),

                -- Communication direction for calls/emails
                -- 'inbound' | 'outbound' | 'n_a'
                ADD COLUMN IF NOT EXISTS direction VARCHAR(20),

                -- Duration in seconds (calls, meetings, demos)
                ADD COLUMN IF NOT EXISTS duration_seconds INTEGER,

                -- Outcome of the activity
                -- 'connected' | 'voicemail' | 'no_answer' | 'bounced' |
                -- 'meeting_held' | 'no_show' | 'completed' | 'cancelled'
                ADD COLUMN IF NOT EXISTS outcome VARCHAR(50),

                -- When the activity is scheduled for (vs created_at which is log time)
                ADD COLUMN IF NOT EXISTS scheduled_at TIMESTAMPTZ,

                -- Arbitrary app-specific payload
                -- call: {"recording_url":"...","transcript":"..."}
                -- email: {"subject":"...","body_preview":"...","message_id":"..."}
                -- meeting: {"location":"...","attendees":["..."]}
                ADD COLUMN IF NOT EXISTS activity_metadata JSONB;
            "#
            .to_owned(),
        ))
        .await?;

        // ── 2. Backfill subject_entity from legacy FK columns ─────────────────
        // Priority: lead_id → contact_id → customer_id → deal_id → case_id → account_id.
        // This sets the canonical "primary" subject. All subjects remain accessible
        // via `associated_entities JSONB`.
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
            UPDATE activity
            SET
                subject_entity_type = CASE
                    WHEN lead_id     IS NOT NULL THEN 'lead'
                    WHEN contact_id  IS NOT NULL THEN 'contact'
                    WHEN customer_id IS NOT NULL THEN 'customer'
                    WHEN deal_id     IS NOT NULL THEN 'deal'
                    WHEN case_id     IS NOT NULL THEN 'atlas_case'
                    WHEN account_id  IS NOT NULL THEN 'atlas_account'
                    ELSE NULL
                END,
                subject_entity_id = COALESCE(
                    lead_id, contact_id, customer_id, deal_id, case_id, account_id
                )
            WHERE subject_entity_type IS NULL;
            "#
            .to_owned(),
        ))
        .await?;

        // ── 2b. Backfill activity_category from activity_type ─────────────────
        // Without this, all historical rows have activity_category = NULL and are
        // silently excluded from pipeline views and reports that filter on category.
        //
        // Mapping heuristics (activity_type values observed in production):
        //   communication: Log, Call, Email, SMS, WhatsApp, Voicemail
        //   meeting:       Meeting, Demo, Site_Visit, Event
        //   task:          Task, Reminder, Follow_Up
        //   system_event:  Status_Change, Stage_Change, Assignment, Automation
        //   pipeline_event: Conversion, Qualification, Disqualification
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
            UPDATE activity
            SET activity_category = CASE
                WHEN lower(activity_type) IN ('log','call','email','sms','whatsapp','voicemail','text')
                    THEN 'communication'
                WHEN lower(activity_type) IN ('meeting','demo','site_visit','event','presentation','webinar')
                    THEN 'meeting'
                WHEN lower(activity_type) IN ('task','reminder','follow_up','followup','todo')
                    THEN 'task'
                WHEN lower(activity_type) IN ('status_change','stage_change','assignment','automation','system')
                    THEN 'system_event'
                WHEN lower(activity_type) IN ('conversion','qualification','disqualification','note')
                    THEN 'pipeline_event'
                ELSE 'communication'  -- safe default: 'Log' is the most common uncategorised type
            END
            WHERE activity_category IS NULL;
            "#
            .to_owned(),
        ))
        .await?;

        // ── 3. Indexes ────────────────────────────────────────────────────────

        // Primary feed: activities for a specific entity, newest-first
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_activity_subject_feed \
             ON activity (subject_entity_type, subject_entity_id, created_at DESC) \
             WHERE subject_entity_type IS NOT NULL;",
        ))
        .await?;

        // Tenant-scoped activity type + category filter (pipeline views, reports)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_activity_tenant_type \
             ON activity (tenant_id, activity_type, created_at DESC);",
        ))
        .await?;

        // Scheduled activities (upcoming tasks / call queue)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_activity_scheduled \
             ON activity (tenant_id, assigned_to, scheduled_at) \
             WHERE scheduled_at IS NOT NULL AND status = 'Open';",
        ))
        .await?;

        // Outcome tracking (connects + no-shows for pipeline quality scoring)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_activity_outcome \
             ON activity (tenant_id, outcome, created_at DESC) \
             WHERE outcome IS NOT NULL;",
        ))
        .await?;

        // Creator history feed
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_activity_created_by \
             ON activity (tenant_id, created_by, created_at DESC);",
        ))
        .await?;

        // ── 4. updated_at trigger ─────────────────────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE OR REPLACE TRIGGER trg_activity_updated_at \
             BEFORE UPDATE ON activity \
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
            "DROP TRIGGER IF EXISTS trg_activity_updated_at ON activity;",
        ))
        .await?;

        for idx in &[
            "idx_activity_subject_feed",
            "idx_activity_tenant_type",
            "idx_activity_scheduled",
            "idx_activity_outcome",
            "idx_activity_created_by",
        ] {
            db.execute(sea_orm::Statement::from_string(
                backend,
                format!("DROP INDEX IF EXISTS {};", idx),
            ))
            .await?;
        }

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE activity \
             DROP COLUMN IF EXISTS subject_entity_type, \
             DROP COLUMN IF EXISTS subject_entity_id, \
             DROP COLUMN IF EXISTS activity_category, \
             DROP COLUMN IF EXISTS direction, \
             DROP COLUMN IF EXISTS duration_seconds, \
             DROP COLUMN IF EXISTS outcome, \
             DROP COLUMN IF EXISTS scheduled_at, \
             DROP COLUMN IF EXISTS activity_metadata;",
        ))
        .await?;

        Ok(())
    }
}
