//! # Migration: G19 `atlas_campaigns` — Multi-Channel Campaign Management
//!
//! ## Tables created
//!
//! | Table | Purpose |
//! |-------|---------|
//! | `atlas_campaigns` | Campaign definition — goal, budget, UTM, channel type, subject entity |
//! | `atlas_sequence_steps` | Multi-step outreach steps (email, SMS, wait, condition) |
//! | `atlas_campaign_enrollments` | A contact enrolled in a campaign + progress tracking |
//! | `atlas_campaign_events` | Per-enrollment interaction events (opened, clicked, converted) |
//!
//! ## Enums created
//!
//! - `atlas_campaign_type`: `cold_email | ppc | social | event_based | sms | content | referral | retargeting`
//! - `atlas_campaign_status`: `draft | scheduled | active | paused | completed | archived`
//! - `atlas_enrollment_status`: `active | paused | completed | exited | bounced | unsubscribed`
//!
//! ## Design
//!
//! The `subject_entity_type` + `subject_entity_id` pair on `atlas_campaigns`
//! links a campaign to any platform entity (asset, event, opportunity, listing)
//! using the same polymorphic FK pattern as `atlas_notes` and `atlas_activities`.
//!
//! Conversion tracking (`conversion_entity_type` + `conversion_entity_id` on
//! enrollments) records what a conversion actually created — an application,
//! contract, event registration, or reservation.
//!
//! Counters (`total_contacts`, `total_opens`, `total_conversions`, etc.) on
//! `atlas_campaigns` are updated by `CampaignService::record_event()` to avoid
//! expensive aggregate queries at read time.
//!
//! ## Indexes
//!
//! - `atlas_campaigns(tenant_id, campaign_type, status)` — list/filter
//! - `atlas_campaigns(tenant_id, subject_entity_type, subject_entity_id)` — find by subject
//! - `atlas_campaign_enrollments(campaign_id, status)` — sequence runner
//! - `atlas_campaign_enrollments(tenant_id, contact_email)` — dedup check
//! - `atlas_campaign_enrollments(status, next_step_at) WHERE status = 'active'` — scheduler poll
//! - `atlas_campaign_events(enrollment_id, occurred_at DESC)` — per-contact timeline
//! - `atlas_campaign_events(campaign_id, event_type, occurred_at)` — analytics aggregation

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260804_g19_atlas_campaigns"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;

        // ── Enums ─────────────────────────────────────────────────────────────

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE atlas_campaign_type AS ENUM (
                    'cold_email',
                    'ppc',
                    'social',
                    'event_based',
                    'sms',
                    'content',
                    'referral',
                    'retargeting'
                );
                CREATE TYPE atlas_campaign_status AS ENUM (
                    'draft',
                    'scheduled',
                    'active',
                    'paused',
                    'completed',
                    'archived'
                );
                CREATE TYPE atlas_enrollment_status AS ENUM (
                    'active',
                    'paused',
                    'completed',
                    'exited',
                    'bounced',
                    'unsubscribed'
                );",
            )
            .await?;

        // ── atlas_campaigns ───────────────────────────────────────────────────

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_campaigns"))
                    .if_not_exists()
                    // ── Hierarchy ─────────────────────────────────────────────
                    // Self-referential FK — NULL means this is a root campaign.
                    // Enables: Program → Campaign → Tactic trees with roll-up reporting.
                    .col(
                        ColumnDef::new(Alias::new("parent_campaign_id"))
                            .uuid()
                            .null(),
                    )
                    // ── Identity ──────────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(
                        ColumnDef::new(Alias::new("tenant_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("name"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("campaign_type"))
                            .custom(Alias::new("atlas_campaign_type"))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .custom(Alias::new("atlas_campaign_status"))
                            .not_null()
                            .default(Expr::cust("'draft'")),
                    )
                    // ── Audience ──────────────────────────────────────────────
                    // Future FK to atlas_audience_segments (nullable for now)
                    .col(
                        ColumnDef::new(Alias::new("audience_segment_id"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("audience_filter"))
                            .json_binary()
                            .null(),
                    )
                    // ── Goal ──────────────────────────────────────────────────
                    // 'lead_capture', 'booking', 'application', 'sale', 'registration'
                    .col(
                        ColumnDef::new(Alias::new("goal_type"))
                            .string_len(50)
                            .null(),
                    )
                    // What entity type a conversion creates
                    .col(
                        ColumnDef::new(Alias::new("goal_entity_type"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("target_conversion_count"))
                            .integer()
                            .null(),
                    )
                    // ── Budget ────────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("budget_cents"))
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("currency"))
                            .char_len(3)
                            .null()
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(Alias::new("spent_cents"))
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    // ── Attribution window ────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("attribution_window_days"))
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    // ── External integration link ─────────────────────────────
                    // Instantly campaign ID, Google campaign ID, Meta campaign ID, etc.
                    .col(
                        ColumnDef::new(Alias::new("external_campaign_id"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("integration_id"))
                            .uuid()
                            .null(),
                    )
                    // ── Linked entity (polymorphic FK) ────────────────────────
                    // What this campaign is FOR: 'atlas_assets', 'atlas_events', etc.
                    .col(
                        ColumnDef::new(Alias::new("subject_entity_type"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("subject_entity_id"))
                            .uuid()
                            .null(),
                    )
                    // ── Scheduling ────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("starts_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("ends_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // ── UTM parameters ────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("utm_source"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("utm_medium"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("utm_campaign"))
                            .string_len(100)
                            .null(),
                    )
                    // ── Computed counters (updated by CampaignService) ────────
                    .col(
                        ColumnDef::new(Alias::new("total_contacts"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("total_opens"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("total_clicks"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("total_replies"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("total_conversions"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // ── Audit ─────────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("created_by_user_id"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_campaigns
                 ADD CONSTRAINT atlas_campaigns_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE;
                 -- Self-referential hierarchy FK: parent must exist, but not required.
                 ALTER TABLE atlas_campaigns
                 ADD CONSTRAINT atlas_campaigns_parent_fk
                 FOREIGN KEY (parent_campaign_id) REFERENCES atlas_campaigns(id) ON DELETE SET NULL;

                 CREATE INDEX atlas_campaigns_tenant_type_status
                     ON atlas_campaigns(tenant_id, campaign_type, status);
                 CREATE INDEX atlas_campaigns_subject
                     ON atlas_campaigns(tenant_id, subject_entity_type, subject_entity_id)
                     WHERE subject_entity_type IS NOT NULL;
                 -- Efficiently find all children of a parent campaign.
                 CREATE INDEX atlas_campaigns_parent
                     ON atlas_campaigns(parent_campaign_id)
                     WHERE parent_campaign_id IS NOT NULL;",
            )
            .await?;

        // ── atlas_sequence_steps ──────────────────────────────────────────────

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_sequence_steps"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(
                        ColumnDef::new(Alias::new("campaign_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("step_number"))
                            .integer()
                            .not_null(),
                    )
                    // 'email', 'sms', 'wait', 'condition', 'task', 'linkedin'
                    .col(
                        ColumnDef::new(Alias::new("step_type"))
                            .string_len(30)
                            .not_null(),
                    )
                    // ── Content (email/sms) ───────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("subject_template"))
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("body_template"))
                            .text()
                            .null(),
                    )
                    // ── Wait step ─────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("wait_days"))
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("wait_hours"))
                            .integer()
                            .null(),
                    )
                    // 'business_hours', 'any_time', 'morning', 'afternoon'
                    .col(
                        ColumnDef::new(Alias::new("send_time_preference"))
                            .string_len(30)
                            .null()
                            .default("business_hours"),
                    )
                    // ── Condition step ────────────────────────────────────────
                    // 'opened', 'clicked', 'replied', 'not_opened_after'
                    .col(
                        ColumnDef::new(Alias::new("condition_type"))
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("condition_value"))
                            .json_binary()
                            .null(),
                    )
                    // Branch step numbers for if/else routing
                    .col(
                        ColumnDef::new(Alias::new("on_true_step"))
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("on_false_step"))
                            .integer()
                            .null(),
                    )
                    // ── A/B variants ──────────────────────────────────────────
                    // [{subject: "...", body: "...", weight: 50}, ...]
                    .col(
                        ColumnDef::new(Alias::new("ab_variants"))
                            .json_binary()
                            .null(),
                    )
                    // ── Exit triggers ─────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("exit_on_reply"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Alias::new("exit_on_conversion"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_sequence_steps
                 ADD CONSTRAINT atlas_sequence_steps_campaign_fk
                 FOREIGN KEY (campaign_id) REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_sequence_steps_campaign_step_unique
                 UNIQUE (campaign_id, step_number);",
            )
            .await?;

        // ── atlas_campaign_enrollments ────────────────────────────────────────

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_campaign_enrollments"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(
                        ColumnDef::new(Alias::new("campaign_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("tenant_id"))
                            .uuid()
                            .not_null(),
                    )
                    // ── Contact identity (one of these) ───────────────────────
                    .col(
                        ColumnDef::new(Alias::new("contact_user_id"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("contact_email"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("contact_name"))
                            .string_len(200)
                            .null(),
                    )
                    // Enrichment: {company, title, linkedin_url, ...}
                    .col(
                        ColumnDef::new(Alias::new("contact_metadata"))
                            .json_binary()
                            .null(),
                    )
                    // ── Progress ──────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .custom(Alias::new("atlas_enrollment_status"))
                            .not_null()
                            .default(Expr::cust("'active'")),
                    )
                    .col(
                        ColumnDef::new(Alias::new("current_step"))
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    // ── Exit tracking ─────────────────────────────────────────
                    // 'replied', 'converted', 'unsubscribed', 'bounced', 'manually_removed'
                    .col(
                        ColumnDef::new(Alias::new("exit_reason"))
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("exit_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // ── Conversion tracking ───────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("converted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("conversion_entity_type"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("conversion_entity_id"))
                            .uuid()
                            .null(),
                    )
                    // ── External system ───────────────────────────────────────
                    // Instantly lead ID, Lemlist lead ID, etc.
                    .col(
                        ColumnDef::new(Alias::new("external_enrollment_id"))
                            .string_len(255)
                            .null(),
                    )
                    // ── Timing ────────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("enrolled_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    // Polled by the sequence scheduler background job
                    .col(
                        ColumnDef::new(Alias::new("next_step_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_campaign_enrollments
                 ADD CONSTRAINT atlas_enrollments_campaign_fk
                 FOREIGN KEY (campaign_id) REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_enrollments_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE;

                 CREATE INDEX atlas_enrollments_campaign_status
                     ON atlas_campaign_enrollments(campaign_id, status);
                 CREATE INDEX atlas_enrollments_contact_email
                     ON atlas_campaign_enrollments(tenant_id, contact_email)
                     WHERE contact_email IS NOT NULL;
                 -- Scheduler poll index: only looks at active enrollments with a pending step
                 CREATE INDEX atlas_enrollments_next_step
                     ON atlas_campaign_enrollments(status, next_step_at)
                     WHERE status = 'active' AND next_step_at IS NOT NULL;",
            )
            .await?;

        // ── atlas_campaign_events ─────────────────────────────────────────────

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_campaign_events"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(
                        ColumnDef::new(Alias::new("enrollment_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("campaign_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("tenant_id"))
                            .uuid()
                            .not_null(),
                    )
                    // NULL for non-sequence events (PPC click, direct form fill)
                    .col(
                        ColumnDef::new(Alias::new("sequence_step_id"))
                            .uuid()
                            .null(),
                    )
                    // 'sent', 'delivered', 'opened', 'clicked', 'replied', 'bounced',
                    // 'unsubscribed', 'spam_reported', 'converted', 'form_fill'
                    .col(
                        ColumnDef::new(Alias::new("event_type"))
                            .string_len(50)
                            .not_null(),
                    )
                    // 'email', 'sms', 'ppc_click', 'social', 'event', 'referral'
                    .col(
                        ColumnDef::new(Alias::new("channel"))
                            .string_len(30)
                            .not_null(),
                    )
                    // ── Context ───────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("link_clicked"))
                            .string_len(512)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("ip_address"))
                            .custom(Alias::new("INET"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("user_agent"))
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("metadata"))
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("occurred_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_campaign_events
                 ADD CONSTRAINT atlas_campaign_events_enrollment_fk
                 FOREIGN KEY (enrollment_id) REFERENCES atlas_campaign_enrollments(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_campaign_events_campaign_fk
                 FOREIGN KEY (campaign_id) REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_campaign_events_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE;

                 CREATE INDEX atlas_campaign_events_enrollment_time
                     ON atlas_campaign_events(enrollment_id, occurred_at DESC);
                 CREATE INDEX atlas_campaign_events_type_time
                     ON atlas_campaign_events(campaign_id, event_type, occurred_at);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS atlas_campaign_events CASCADE;
                 DROP TABLE IF EXISTS atlas_campaign_enrollments CASCADE;
                 DROP TABLE IF EXISTS atlas_sequence_steps CASCADE;
                 DROP TABLE IF EXISTS atlas_campaigns CASCADE;
                 DROP TYPE IF EXISTS atlas_enrollment_status;
                 DROP TYPE IF EXISTS atlas_campaign_status;
                 DROP TYPE IF EXISTS atlas_campaign_type;",
            )
            .await?;

        Ok(())
    }
}
