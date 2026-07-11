use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-19: atlas_campaigns — Multi-Channel Campaign Management
///
/// A campaign is a coordinated outreach effort across one or more channels
/// (email, SMS, PPC, events, referrals) aimed at driving a specific conversion —
/// enrollment, booking, application, sale, or registration.
///
/// Salesforce analog: Campaign + CampaignMember objects.
///
/// Apps benefiting immediately: PM (open-house campaigns), AgentLink (agent
/// recruiting), Clipping Marketplace (brand campaigns), Nomad List (community
/// campaigns), Direct Booking Engine (hotel marketing), ClaimSwift (adjuster
/// outreach), Famtasm (creator campaigns).
///
/// Depends on: G-03 (atlas_ledger_entries), G-05 (atlas_external_integrations)
/// Referenced by: G-20 (atlas_attribution), G-21 (atlas_events)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Campaign type ENUM ────────────────────────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            DO $$ BEGIN
                CREATE TYPE atlas_campaign_type AS ENUM (
                    'cold_email',
                    'ppc',
                    'social',
                    'event_based',
                    'sms',
                    'content',
                    'referral',
                    'retargeting'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
            .to_owned(),
        ))
        .await?;

        // ── Campaign status ENUM ──────────────────────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            DO $$ BEGIN
                CREATE TYPE atlas_campaign_status AS ENUM (
                    'draft',
                    'scheduled',
                    'active',
                    'paused',
                    'completed',
                    'archived'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
            .to_owned(),
        ))
        .await?;

        // ── Enrollment status ENUM ────────────────────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            DO $$ BEGIN
                CREATE TYPE atlas_enrollment_status AS ENUM (
                    'active',
                    'paused',
                    'completed',
                    'exited',
                    'bounced',
                    'unsubscribed'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
            .to_owned(),
        ))
        .await?;

        // ── atlas_campaigns ───────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasCampaigns::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCampaigns::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasCampaigns::TenantId).uuid().not_null())
                    // Identity
                    .col(
                        ColumnDef::new(AtlasCampaigns::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::CampaignType)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("draft")),
                    )
                    // Audience
                    .col(
                        ColumnDef::new(AtlasCampaigns::AudienceFilter)
                            .json_binary()
                            .null(),
                    )
                    // Goal
                    .col(
                        ColumnDef::new(AtlasCampaigns::GoalType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::GoalEntityType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::TargetConversionCount)
                            .integer()
                            .null(),
                    )
                    // Budget
                    .col(
                        ColumnDef::new(AtlasCampaigns::BudgetCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::Currency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::SpentCents)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::AttributionWindowDays)
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    // External system
                    .col(
                        ColumnDef::new(AtlasCampaigns::ExternalCampaignId)
                            .string_len(255)
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasCampaigns::IntegrationId).uuid().null()) // FK atlas_external_integrations
                    // Linked entity (what this campaign is FOR — polymorphic)
                    .col(
                        ColumnDef::new(AtlasCampaigns::SubjectEntityType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::SubjectEntityId)
                            .uuid()
                            .null(),
                    )
                    // Dates
                    .col(
                        ColumnDef::new(AtlasCampaigns::StartsAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::EndsAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // UTM
                    .col(
                        ColumnDef::new(AtlasCampaigns::UtmSource)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::UtmMedium)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::UtmCampaign)
                            .string_len(100)
                            .null(),
                    )
                    // Computed counters (updated by CampaignService)
                    .col(
                        ColumnDef::new(AtlasCampaigns::TotalContacts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::TotalOpens)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::TotalClicks)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::TotalReplies)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::TotalConversions)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::CreatedByUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaigns::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_campaigns_tenant_type_status")
                    .table(AtlasCampaigns::Table)
                    .col(AtlasCampaigns::TenantId)
                    .col(AtlasCampaigns::CampaignType)
                    .col(AtlasCampaigns::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_campaigns_subject")
                    .table(AtlasCampaigns::Table)
                    .col(AtlasCampaigns::TenantId)
                    .col(AtlasCampaigns::SubjectEntityType)
                    .col(AtlasCampaigns::SubjectEntityId)
                    .to_owned(),
            )
            .await?;

        // ── atlas_sequence_steps ──────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasSequenceSteps::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::CampaignId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::StepNumber)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::StepType)
                            .string_len(30)
                            .not_null(),
                    )
                    // 'email', 'sms', 'wait', 'condition', 'task', 'linkedin'
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::SubjectTemplate)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::BodyTemplate)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::WaitDays)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::WaitHours)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::SendTimePreference)
                            .string_len(20)
                            .not_null()
                            .default(Expr::val("business_hours")),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::ConditionType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::ConditionValue)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::OnTrueStep)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::OnFalseStep)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::AbVariants)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::ExitOnReply)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::ExitOnConversion)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AtlasSequenceSteps::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_sequence_steps_campaign")
                    .table(AtlasSequenceSteps::Table)
                    .col(AtlasSequenceSteps::CampaignId)
                    .col(AtlasSequenceSteps::StepNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // ── atlas_campaign_enrollments ────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasCampaignEnrollments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::CampaignId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    // Who is enrolled
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ContactUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ContactEmail)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ContactName)
                            .string_len(200)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ContactMetadata)
                            .json_binary()
                            .null(),
                    )
                    // Progress
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("active")),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::CurrentStep)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    // Exit tracking
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ExitReason)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ExitAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // Conversion
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ConvertedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ConversionEntityType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ConversionEntityId)
                            .uuid()
                            .null(),
                    )
                    // External reference (Instantly lead ID, etc.)
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::ExternalEnrollmentId)
                            .string_len(255)
                            .null(),
                    )
                    // Timing
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::EnrolledAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEnrollments::NextStepAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_enrollments_campaign_status")
                    .table(AtlasCampaignEnrollments::Table)
                    .col(AtlasCampaignEnrollments::CampaignId)
                    .col(AtlasCampaignEnrollments::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_enrollments_contact_email")
                    .table(AtlasCampaignEnrollments::Table)
                    .col(AtlasCampaignEnrollments::TenantId)
                    .col(AtlasCampaignEnrollments::ContactEmail)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_enrollments_next_step")
                    .table(AtlasCampaignEnrollments::Table)
                    .col(AtlasCampaignEnrollments::Status)
                    .col(AtlasCampaignEnrollments::NextStepAt)
                    .to_owned(),
            )
            .await?;

        // ── atlas_campaign_events ─────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasCampaignEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::EnrollmentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::CampaignId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::SequenceStepId)
                            .uuid()
                            .null(),
                    )
                    // 'sent', 'delivered', 'opened', 'clicked', 'replied', 'bounced',
                    // 'unsubscribed', 'spam_reported', 'converted', 'form_fill'
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::EventType)
                            .string_len(50)
                            .not_null(),
                    )
                    // 'email', 'sms', 'ppc_click', 'social', 'event'
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::Channel)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::LinkClicked)
                            .string_len(512)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::Metadata)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasCampaignEvents::OccurredAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_campaign_events_enrollment")
                    .table(AtlasCampaignEvents::Table)
                    .col(AtlasCampaignEvents::EnrollmentId)
                    .col(AtlasCampaignEvents::OccurredAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_campaign_events_type")
                    .table(AtlasCampaignEvents::Table)
                    .col(AtlasCampaignEvents::CampaignId)
                    .col(AtlasCampaignEvents::EventType)
                    .col(AtlasCampaignEvents::OccurredAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasCampaignEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AtlasCampaignEnrollments::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasSequenceSteps::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AtlasCampaigns::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "DROP TYPE IF EXISTS atlas_enrollment_status;".to_owned(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "DROP TYPE IF EXISTS atlas_campaign_status;".to_owned(),
            ))
            .await?;
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "DROP TYPE IF EXISTS atlas_campaign_type;".to_owned(),
            ))
            .await?;

        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Iden enums
// ══════════════════════════════════════════════════════════════════════════════

#[derive(DeriveIden)]
enum AtlasCampaigns {
    Table,
    Id,
    TenantId,
    Name,
    CampaignType,
    Status,
    AudienceFilter,
    GoalType,
    GoalEntityType,
    TargetConversionCount,
    BudgetCents,
    Currency,
    SpentCents,
    AttributionWindowDays,
    ExternalCampaignId,
    IntegrationId,
    SubjectEntityType,
    SubjectEntityId,
    StartsAt,
    EndsAt,
    UtmSource,
    UtmMedium,
    UtmCampaign,
    TotalContacts,
    TotalOpens,
    TotalClicks,
    TotalReplies,
    TotalConversions,
    CreatedByUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasSequenceSteps {
    Table,
    Id,
    CampaignId,
    StepNumber,
    StepType,
    SubjectTemplate,
    BodyTemplate,
    WaitDays,
    WaitHours,
    SendTimePreference,
    ConditionType,
    ConditionValue,
    OnTrueStep,
    OnFalseStep,
    AbVariants,
    ExitOnReply,
    ExitOnConversion,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasCampaignEnrollments {
    Table,
    Id,
    CampaignId,
    TenantId,
    ContactUserId,
    ContactEmail,
    ContactName,
    ContactMetadata,
    Status,
    CurrentStep,
    ExitReason,
    ExitAt,
    ConvertedAt,
    ConversionEntityType,
    ConversionEntityId,
    ExternalEnrollmentId,
    EnrolledAt,
    NextStepAt,
}

#[derive(DeriveIden)]
enum AtlasCampaignEvents {
    Table,
    Id,
    EnrollmentId,
    CampaignId,
    TenantId,
    SequenceStepId,
    EventType,
    Channel,
    LinkClicked,
    Metadata,
    OccurredAt,
}
