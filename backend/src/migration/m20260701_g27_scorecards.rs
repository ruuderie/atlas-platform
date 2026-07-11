use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-27: Atlas Scorecards — Universal Structured Evaluation Engine
///
/// A single engine that powers:
///   - NomadList city ratings (city, neighborhood, airline)
///   - Bridgewater-style employee Baseball Cards
///   - Restaurant / venue / hotel quality scoring
///   - Beauty + consumer product reviews
///   - Contractor performance per-job
///   - Event staff performance per-shift
///   - Insurance carrier / MGA ratings
///   - CRM: Lead Qualification, Deal Health, Rep Performance, Touch Quality
///
/// The entity changes; the engine does not.
///
/// 11 tables:
///   atlas_scorecard_templates           — what traits exist for an entity type
///   atlas_scorecard_dimensions          — individual traits with scale + benchmarks
///   atlas_scorecard_dimension_options   — poll options (poll_single / poll_multi)
///   atlas_scorecards                    — template applied to one specific entity
///   atlas_rating_sessions               — one per discrete occurrence (job/stay/visit)
///   atlas_scorecard_entries             — sparse scores per dimension per session
///   atlas_scorecard_dimension_aggregates — rolled-up community scores
///   atlas_scorecard_poll_aggregates     — vote counts for categorical dimensions
///   atlas_scorecard_time_series         — monthly/quarterly trend buckets
///   atlas_scorecard_targets             — target profiles for The Combinator
///   atlas_scorecard_target_criteria     — per-dimension criteria for a target
///
/// All indexes with WHERE clauses, DESC ordering, or USING GIST are emitted as
/// raw SQL — SeaORM's Index builder has no partial index or sort-direction support.
///
/// Spec: docs/architecture/g27_scorecards_spec.md
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── 1. atlas_scorecard_templates ─────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(ScorecardTemplates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScorecardTemplates::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    // Discriminator: 'city' | 'person' | 'restaurant' | 'product' |
                    // 'contractor' | 'airline' | 'property' | 'hotel' | 'agent' |
                    // 'carrier' | 'event' | 'atlas_lead' | 'atlas_opportunity'
                    .col(
                        ColumnDef::new(ScorecardTemplates::EntityType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::Description)
                            .text()
                            .null(),
                    )
                    // 'weighted_mean' | 'simple_mean' | 'percentile_rank'
                    .col(
                        ColumnDef::new(ScorecardTemplates::ScoringMethod)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("weighted_mean")),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::DefaultScaleMin)
                            .decimal_len(6, 2)
                            .not_null()
                            .default(Expr::val(1.0)),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::DefaultScaleMax)
                            .decimal_len(6, 2)
                            .not_null()
                            .default(Expr::val(10.0)),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::MinEntriesToPublish)
                            .integer()
                            .not_null()
                            .default(Expr::val(5)),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::IsPublished)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::CreatedByUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTemplates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_scorecard_templates_tenant_type \
             ON atlas_scorecard_templates (tenant_id, entity_type);"
                .to_owned(),
        ))
        .await?;

        // ── 2. atlas_scorecard_dimensions ────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(ScorecardDimensions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScorecardDimensions::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::TemplateId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::Slug)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::Category)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::Weight)
                            .decimal_len(5, 4)
                            .not_null()
                            .default(Expr::val(1.0)),
                    )
                    // 'rating' | 'absolute' | 'boolean' | 'poll_single' | 'poll_multi'
                    .col(
                        ColumnDef::new(ScorecardDimensions::ScaleType)
                            .string_len(20)
                            .not_null()
                            .default(Expr::val("rating")),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::ScaleMin)
                            .decimal_len(10, 2)
                            .not_null()
                            .default(Expr::val(1.0)),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::ScaleMax)
                            .decimal_len(10, 2)
                            .not_null()
                            .default(Expr::val(10.0)),
                    )
                    // Unit label: 'Mbps', 'USD/mo', '°C', 'hrs', '%'
                    .col(
                        ColumnDef::new(ScorecardDimensions::UnitLabel)
                            .string_len(30)
                            .null(),
                    )
                    // JSONB array of tier objects — see spec section 2.3
                    .col(
                        ColumnDef::new(ScorecardDimensions::BenchmarkTiers)
                            .json_binary()
                            .not_null()
                            .default(Expr::val("[]")),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::GlobalReferenceValue)
                            .decimal_len(10, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::GlobalReferenceLabel)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::MinEntriesToShow)
                            .integer()
                            .not_null()
                            .default(Expr::val(3)),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::IsCommunityRatable)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensions::SortOrder)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .to_owned(),
            )
            .await?;

        // UNIQUE(template_id, slug) — prevents duplicate dimension slugs within a template
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             ADD CONSTRAINT uq_scorecard_dimensions_template_slug \
             UNIQUE (template_id, slug);"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_scorecard_dimensions_template \
             ON atlas_scorecard_dimensions (template_id, sort_order);"
                .to_owned(),
        ))
        .await?;

        // ── 3. atlas_scorecard_dimension_options ─────────────────────────────
        // Only used for poll_single / poll_multi dimensions
        manager
            .create_table(
                Table::create()
                    .table(ScorecardDimensionOptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::DimensionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    // Display label: "Telkomsel", "BIMC Kuta Hospital"
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::Label)
                            .string_len(255)
                            .not_null(),
                    )
                    // Stable slug: 'telkomsel' — for API consumers
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::ValueKey)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::ImageUrl)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::SortOrder)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(ScorecardDimensionOptions::IsWriteIn)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_scorecard_dim_options_dimension \
             ON atlas_scorecard_dimension_options (dimension_id, sort_order);"
                .to_owned(),
        ))
        .await?;

        // ── 4. atlas_scorecards ───────────────────────────────────────────────
        // One per (template, entity instance). Polymorphic subject via
        // subject_entity_type + subject_entity_id.
        manager
            .create_table(
                Table::create()
                    .table(Scorecards::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Scorecards::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Scorecards::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Scorecards::TemplateId).uuid().not_null())
                    // Polymorphic subject — any platform entity
                    // Supported values: 'atlas_asset' | 'listing' | 'atlas_catalog_entry' |
                    // 'atlas_account' | 'atlas_service_provider' | 'profile' | 'customer' |
                    // 'atlas_opportunity' | 'atlas_portfolio' | 'atlas_lead' | 'atlas_contact'
                    .col(
                        ColumnDef::new(Scorecards::SubjectEntityType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Scorecards::SubjectEntityId)
                            .uuid()
                            .not_null(),
                    )
                    // Computed composite score — recomputed by background job
                    .col(
                        ColumnDef::new(Scorecards::CompositeScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    // 'insufficient'(<5) | 'low'(<10) | 'medium'(<50) | 'high'(<200) | 'very_high'
                    .col(
                        ColumnDef::new(Scorecards::ConfidenceLevel)
                            .string_len(20)
                            .not_null()
                            .default(Expr::val("insufficient")),
                    )
                    .col(
                        ColumnDef::new(Scorecards::TotalContributors)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(Scorecards::TotalSessions)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(Scorecards::TotalEntries)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    // dimension_vector DECIMAL(5,2)[] — stored as JSONB for SeaORM compat.
                    // Ordered array of weighted normalized scores (one per dimension, sort_order).
                    // Used by The Combinator for similarity search.
                    .col(
                        ColumnDef::new(Scorecards::DimensionVector)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Scorecards::LastComputedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Scorecards::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // UNIQUE(template_id, subject_entity_type, subject_entity_id) — one scorecard per entity per template
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecards \
             ADD CONSTRAINT uq_scorecards_template_subject \
             UNIQUE (template_id, subject_entity_type, subject_entity_id);"
                .to_owned(),
        ))
        .await?;

        // Entity lookup — the primary access pattern
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_scorecards_entity \
             ON atlas_scorecards (subject_entity_type, subject_entity_id);"
                .to_owned(),
        ))
        .await?;

        // Tenant + confidence filter for The Combinator
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_atlas_scorecards_tenant_confidence \
             ON atlas_scorecards (tenant_id, template_id, confidence_level) \
             WHERE composite_score IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // ── 5. atlas_rating_sessions ──────────────────────────────────────────
        // One per discrete occurrence: city visit, contractor job, event shift,
        // hotel stay, product purchase, pipeline review, qualification call.
        manager
            .create_table(
                Table::create()
                    .table(RatingSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RatingSessions::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(RatingSessions::ScorecardId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RatingSessions::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(RatingSessions::RaterUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RatingSessions::OccurredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    // 'job' | 'stay' | 'visit' | 'event_shift' | 'purchase' | 'flight' |
                    // 'meeting' | 'pipeline_review' | 'call' | 'email_thread' | 'demo' |
                    // 'monthly_review' | 'quarterly_review'
                    .col(
                        ColumnDef::new(RatingSessions::SessionType)
                            .string_len(30)
                            .not_null(),
                    )
                    // Links to existing platform records without data duplication
                    // e.g. 'atlas_case', 'atlas_reservation', 'atlas_activity'
                    .col(
                        ColumnDef::new(RatingSessions::ContextEntityType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(RatingSessions::ContextEntityId)
                            .uuid()
                            .null(),
                    )
                    .col(ColumnDef::new(RatingSessions::SessionLabel).text().null())
                    // 'draft' | 'submitted' | 'verified' | 'disputed'
                    .col(
                        ColumnDef::new(RatingSessions::Status)
                            .string_len(20)
                            .not_null()
                            .default(Expr::val("submitted")),
                    )
                    // G-06 verification gate when required by template config
                    .col(
                        ColumnDef::new(RatingSessions::VerificationRequestId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(RatingSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Primary read: sessions for a scorecard, newest first
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_rating_sessions_scorecard_time \
             ON atlas_rating_sessions (scorecard_id, occurred_at DESC);"
                .to_owned(),
        ))
        .await?;

        // Link back to the originating platform record (job, booking, activity)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_rating_sessions_context \
             ON atlas_rating_sessions (context_entity_type, context_entity_id) \
             WHERE context_entity_type IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Rater history feed
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_rating_sessions_rater \
             ON atlas_rating_sessions (tenant_id, rater_user_id, occurred_at DESC);"
                .to_owned(),
        ))
        .await?;

        // ── 6. atlas_scorecard_entries ────────────────────────────────────────
        // Sparse: contributor submits only dimensions they have experience with.
        // One row per (session, dimension, rater). Hard UNIQUE at DB level.
        manager
            .create_table(
                Table::create()
                    .table(ScorecardEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScorecardEntries::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(ScorecardEntries::SessionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardEntries::ScorecardId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardEntries::DimensionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ScorecardEntries::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(ScorecardEntries::ContributorUserId)
                            .uuid()
                            .not_null(),
                    )
                    // For rating / absolute / boolean: the numeric value
                    .col(
                        ColumnDef::new(ScorecardEntries::Score)
                            .decimal_len(8, 2)
                            .null(),
                    )
                    // For poll_single / poll_multi: the selected option
                    // Exactly one of score or option_id must be non-null (service enforced)
                    .col(ColumnDef::new(ScorecardEntries::OptionId).uuid().null())
                    // Evidence source type
                    // 'community_rating' | 'peer_review' | 'self_assessment' |
                    // 'manager_review' | 'test_result' | 'behavioral_signal' | 'official_data'
                    .col(
                        ColumnDef::new(ScorecardEntries::SourceType)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("community_rating")),
                    )
                    // Credibility context — source_type specific JSONB
                    // Community: {"visit_start":"2024-03","duration_days":90,"purpose":"work"}
                    // Peer review: {"relationship":"peer","worked_together_months":18}
                    // Test result: {"test_name":"CRT","date":"2024-01","administered_by":"HR"}
                    .col(
                        ColumnDef::new(ScorecardEntries::Context)
                            .json_binary()
                            .null(),
                    )
                    .col(ColumnDef::new(ScorecardEntries::Note).text().null())
                    // G-06 verification gate
                    .col(
                        ColumnDef::new(ScorecardEntries::IsVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ScorecardEntries::VerificationRequestId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardEntries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Hard unique constraint: one entry per (session, dimension, contributor)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_entries \
             ADD CONSTRAINT uq_scorecard_entries_session_dim_contributor \
             UNIQUE (session_id, dimension_id, contributor_user_id);"
                .to_owned(),
        ))
        .await?;

        // Aggregate recompute: fetch verified entries per dimension
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_scorecard_entries_scorecard_verified \
             ON atlas_scorecard_entries (scorecard_id, is_verified, dimension_id);"
                .to_owned(),
        ))
        .await?;

        // Contributor history lookup
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_scorecard_entries_contributor \
             ON atlas_scorecard_entries (tenant_id, contributor_user_id, created_at DESC);"
                .to_owned(),
        ))
        .await?;

        // ── 7. atlas_scorecard_dimension_aggregates ───────────────────────────
        // Recomputed by background job after verified entries change.
        // Primary key is (scorecard_id, dimension_id).
        manager
            .create_table(
                Table::create()
                    .table(DimensionAggregates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DimensionAggregates::ScorecardId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::DimensionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::MeanScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::WeightedMeanScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    // For boolean dimensions: percentage of true responses
                    .col(
                        ColumnDef::new(DimensionAggregates::PercentTrue)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    // Resolved benchmark tier for this aggregate
                    .col(
                        ColumnDef::new(DimensionAggregates::BenchmarkLabel)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::BenchmarkColor)
                            .string_len(7)
                            .null(),
                    )
                    // Human-readable: "Fast: 16 Mbps", "$1,183/mo", "83% say clean"
                    .col(
                        ColumnDef::new(DimensionAggregates::DisplayValue)
                            .string_len(150)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::StdDeviation)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    // 'strong_consensus' | 'consensus' | 'mixed' | 'disputed'
                    .col(
                        ColumnDef::new(DimensionAggregates::ConsensusLevel)
                            .string_len(20)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::MinScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::MaxScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::ContributorCount)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::SessionCount)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    // Delta from global_reference_value
                    .col(
                        ColumnDef::new(DimensionAggregates::VsGlobalDelta)
                            .decimal_len(8, 2)
                            .null(),
                    )
                    // 'above' | 'at' | 'below'
                    .col(
                        ColumnDef::new(DimensionAggregates::VsGlobalLabel)
                            .string_len(10)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DimensionAggregates::LastComputedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(DimensionAggregates::ScorecardId)
                            .col(DimensionAggregates::DimensionId),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 8. atlas_scorecard_poll_aggregates ────────────────────────────────
        // Vote counts for poll_single / poll_multi dimensions.
        // Primary key is (scorecard_id, dimension_id, option_id).
        manager
            .create_table(
                Table::create()
                    .table(PollAggregates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PollAggregates::ScorecardId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PollAggregates::DimensionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PollAggregates::OptionId).uuid().not_null())
                    .col(
                        ColumnDef::new(PollAggregates::VoteCount)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(PollAggregates::VotePct)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PollAggregates::Rank)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(PollAggregates::TotalVoters)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(PollAggregates::LastComputedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(PollAggregates::ScorecardId)
                            .col(PollAggregates::DimensionId)
                            .col(PollAggregates::OptionId),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 9. atlas_scorecard_time_series ────────────────────────────────────
        // Monthly / quarterly trend buckets per dimension.
        // Primary key is (scorecard_id, dimension_id, period_type, period_start).
        manager
            .create_table(
                Table::create()
                    .table(TimeSeries::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TimeSeries::ScorecardId).uuid().not_null())
                    .col(ColumnDef::new(TimeSeries::DimensionId).uuid().not_null())
                    .col(ColumnDef::new(TimeSeries::PeriodStart).date().not_null())
                    .col(
                        ColumnDef::new(TimeSeries::PeriodType)
                            .string_len(10)
                            .not_null()
                            .default(Expr::val("monthly")),
                    )
                    .col(
                        ColumnDef::new(TimeSeries::MeanScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TimeSeries::SessionCount)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(TimeSeries::ContributorCount)
                            .integer()
                            .not_null()
                            .default(Expr::val(0)),
                    )
                    .col(
                        ColumnDef::new(TimeSeries::DeltaFromPrior)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    // 'improving' | 'stable' | 'declining' | 'insufficient_data'
                    .col(
                        ColumnDef::new(TimeSeries::TrendDirection)
                            .string_len(20)
                            .null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(TimeSeries::ScorecardId)
                            .col(TimeSeries::DimensionId)
                            .col(TimeSeries::PeriodType)
                            .col(TimeSeries::PeriodStart),
                    )
                    .to_owned(),
            )
            .await?;

        // Recent trend lookup per scorecard
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_scorecard_time_series_recent \
             ON atlas_scorecard_time_series (scorecard_id, period_type, period_start DESC);"
                .to_owned(),
        ))
        .await?;

        // ── 10. atlas_scorecard_targets ───────────────────────────────────────
        // Target profiles for The Combinator similarity search.
        manager
            .create_table(
                Table::create()
                    .table(ScorecardTargets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScorecardTargets::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargets::TemplateId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ScorecardTargets::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(ScorecardTargets::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    // 'search_filter' | 'job_specification' | 'ideal_profile'
                    .col(
                        ColumnDef::new(ScorecardTargets::TargetType)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ScorecardTargets::Description).text().null())
                    // Source entities used to derive ideal_profile vector
                    .col(
                        ColumnDef::new(ScorecardTargets::SeedEntityIds)
                            .json_binary()
                            .null(),
                    )
                    // Precomputed target vector (JSONB array of f64)
                    .col(
                        ColumnDef::new(ScorecardTargets::TargetVector)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargets::CreatedByUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // ── 11. atlas_scorecard_target_criteria ───────────────────────────────
        // Per-dimension criteria for a target profile.
        manager
            .create_table(
                Table::create()
                    .table(ScorecardTargetCriteria::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::TargetId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::DimensionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::MinScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::MaxScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::IdealScore)
                            .decimal_len(5, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::IsDealbreaker)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ScorecardTargetCriteria::SearchWeight)
                            .decimal_len(5, 4)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_scorecard_target_criteria_target \
             ON atlas_scorecard_target_criteria (target_id, dimension_id);"
                .to_owned(),
        ))
        .await?;

        // ── 12. updated_at triggers ───────────────────────────────────────────
        // set_updated_at_column() was created by G-31 migration (m20260601_g31_atlas_lead).
        // G-27 (m20260701_) sorts after G-31 (m20260601_) so function exists.
        // Only templates and scorecards need it — the aggregate/time-series tables
        // are always fully replaced on recompute, not incrementally updated.
        for table in &["atlas_scorecard_templates", "atlas_scorecards"] {
            db.execute(sea_orm::Statement::from_string(
                backend,
                format!(
                    "CREATE OR REPLACE TRIGGER trg_{t}_updated_at \
                     BEFORE UPDATE ON {t} \
                     FOR EACH ROW EXECUTE FUNCTION set_updated_at_column();",
                    t = table
                ),
            ))
            .await?;
        }

        // Add updated_at + deleted_at to templates and scorecards (not in DDL above to keep
        // the builder blocks clean — easier to add via ALTER).
        // deleted_at: soft-delete sentinel. NULL = active, non-null = archived.
        // Required by mv_scorecard_portfolio_analytics and v_scorecard_recent_anomalies.
        for table in &["atlas_scorecard_templates", "atlas_scorecards"] {
            db.execute(sea_orm::Statement::from_string(
                backend,
                format!(
                    "ALTER TABLE {t} \
                     ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();",
                    t = table
                ),
            ))
            .await?;

            db.execute(sea_orm::Statement::from_string(
                backend,
                format!(
                    "ALTER TABLE {t} \
                     ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ DEFAULT NULL;",
                    t = table
                ),
            ))
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        for table in &["atlas_scorecard_templates", "atlas_scorecards"] {
            db.execute(sea_orm::Statement::from_string(
                backend,
                format!(
                    "DROP TRIGGER IF EXISTS trg_{t}_updated_at ON {t};",
                    t = table
                ),
            ))
            .await?;
        }

        // Drop in reverse FK dependency order
        manager
            .drop_table(
                Table::drop()
                    .table(ScorecardTargetCriteria::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ScorecardTargets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TimeSeries::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PollAggregates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DimensionAggregates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ScorecardEntries::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RatingSessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Scorecards::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ScorecardDimensionOptions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ScorecardDimensions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ScorecardTemplates::Table).to_owned())
            .await?;

        Ok(())
    }
}

// ── Iden enums ───────────────────────────────────────────────────────────────

#[derive(DeriveIden)]
enum ScorecardTemplates {
    #[sea_orm(iden = "atlas_scorecard_templates")]
    Table,
    Id,
    TenantId,
    Name,
    EntityType,
    Description,
    ScoringMethod,
    DefaultScaleMin,
    DefaultScaleMax,
    MinEntriesToPublish,
    IsPublished,
    CreatedByUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ScorecardDimensions {
    #[sea_orm(iden = "atlas_scorecard_dimensions")]
    Table,
    Id,
    TemplateId,
    TenantId,
    Slug,
    Name,
    Description,
    Category,
    Weight,
    ScaleType,
    ScaleMin,
    ScaleMax,
    UnitLabel,
    BenchmarkTiers,
    GlobalReferenceValue,
    GlobalReferenceLabel,
    MinEntriesToShow,
    IsCommunityRatable,
    IsActive,
    SortOrder,
}

#[derive(DeriveIden)]
enum ScorecardDimensionOptions {
    #[sea_orm(iden = "atlas_scorecard_dimension_options")]
    Table,
    Id,
    DimensionId,
    TenantId,
    Label,
    ValueKey,
    Description,
    ImageUrl,
    SortOrder,
    IsWriteIn,
}

#[derive(DeriveIden)]
enum Scorecards {
    #[sea_orm(iden = "atlas_scorecards")]
    Table,
    Id,
    TenantId,
    TemplateId,
    SubjectEntityType,
    SubjectEntityId,
    CompositeScore,
    ConfidenceLevel,
    TotalContributors,
    TotalSessions,
    TotalEntries,
    DimensionVector,
    LastComputedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum RatingSessions {
    #[sea_orm(iden = "atlas_rating_sessions")]
    Table,
    Id,
    ScorecardId,
    TenantId,
    RaterUserId,
    OccurredAt,
    SessionType,
    ContextEntityType,
    ContextEntityId,
    SessionLabel,
    Status,
    VerificationRequestId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ScorecardEntries {
    #[sea_orm(iden = "atlas_scorecard_entries")]
    Table,
    Id,
    SessionId,
    ScorecardId,
    DimensionId,
    TenantId,
    ContributorUserId,
    Score,
    OptionId,
    SourceType,
    Context,
    Note,
    IsVerified,
    VerificationRequestId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum DimensionAggregates {
    #[sea_orm(iden = "atlas_scorecard_dimension_aggregates")]
    Table,
    ScorecardId,
    DimensionId,
    MeanScore,
    WeightedMeanScore,
    PercentTrue,
    BenchmarkLabel,
    BenchmarkColor,
    DisplayValue,
    StdDeviation,
    ConsensusLevel,
    MinScore,
    MaxScore,
    ContributorCount,
    SessionCount,
    VsGlobalDelta,
    VsGlobalLabel,
    LastComputedAt,
}

#[derive(DeriveIden)]
enum PollAggregates {
    #[sea_orm(iden = "atlas_scorecard_poll_aggregates")]
    Table,
    ScorecardId,
    DimensionId,
    OptionId,
    VoteCount,
    VotePct,
    Rank,
    TotalVoters,
    LastComputedAt,
}

#[derive(DeriveIden)]
enum TimeSeries {
    #[sea_orm(iden = "atlas_scorecard_time_series")]
    Table,
    ScorecardId,
    DimensionId,
    PeriodStart,
    PeriodType,
    MeanScore,
    SessionCount,
    ContributorCount,
    DeltaFromPrior,
    TrendDirection,
}

#[derive(DeriveIden)]
enum ScorecardTargets {
    #[sea_orm(iden = "atlas_scorecard_targets")]
    Table,
    Id,
    TemplateId,
    TenantId,
    Name,
    TargetType,
    Description,
    SeedEntityIds,
    TargetVector,
    CreatedByUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ScorecardTargetCriteria {
    #[sea_orm(iden = "atlas_scorecard_target_criteria")]
    Table,
    Id,
    TargetId,
    DimensionId,
    MinScore,
    MaxScore,
    IdealScore,
    IsDealbreaker,
    SearchWeight,
}
