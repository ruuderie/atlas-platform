//! G-27 Atlas Scorecards — `ScorecardService`
//!
//! The primary service for all scorecard lifecycle operations: creation, session management,
//! entry submission, aggregate computation, similarity search, display rules, and nudge logic.
//!
//! ## G-27 Data Science Upgrade — Phase Summary
//!
//! | Phase | What shipped | Key functions |
//! |-------|--------------|---------------|
//! | 1 | Bayesian cold-start + confidence-weighted composite | `compute_numeric_aggregate`, `recompute_aggregates` |
//! | 2 | Masked cosine similarity, percentile ranks, anomaly detection | `find_similar`, `compute_percentile_ranks`, `refresh_time_series_for_dimension` |
//! | 3 | Portfolio analytics MV + 4 API endpoints + 4h background job | `refresh_and_rerank` |
//! | 4 | Contributor calibration (weekly bias/scale coefficients) | `calibrate_contributor_bias` |
//! | 5 | BYOC Compute SDK + `ComputeBackend` strategy enum | `ComputeBackend::resolve` |
//!
//! ## Background Job Schedule
//!
//! | Job key | Interval | Function |
//! |---------|----------|----------|
//! | `recompute_scorecard_aggregates` | 5 min | `recompute_aggregates` |
//! | `refresh_scorecard_time_series` | 24 h | `refresh_time_series_for_dimension` |
//! | `refresh_scorecard_portfolio_analytics` | 4 h | `refresh_and_rerank` |
//! | `calibrate_scorecard_contributors` | 7 days | `calibrate_contributor_bias` |
//!
//! ## Core Algorithms (see `crates/atlas-compute-sdk` for extracted pure-math versions)
//!
//! ### Bayesian Shrinkage (Phase 1)
//! ```text
//! shrunk = (prior_weight × global_ref + Σ(score_i × credibility_i))
//!          / (prior_weight + Σ(credibility_i))
//! ```
//! Configured per-dimension via `bayesian_prior_weight`. Converges to raw mean as n >> prior_weight.
//!
//! ### Confidence-Weighted Composite (Phase 1)
//! ```text
//! confidence_weight = MIN(contributor_count / saturation_threshold, 1.0)
//! ```
//! Sparse dimensions contribute proportionally less to the composite score.
//!
//! ### Contributor Calibration (Phase 4)
//! ```text
//! bias_offset  = contributor_mean − ensemble_mean
//! scale_factor = clamp(σ_contributor / σ_ensemble, 0.1, 3.0)
//! calibrated   = clamp((raw − bias) × scale, scale_min, scale_max)
//! ```
//! Gated: only applied when `entry_count >= template.calibration_minimum_entries` (default: 100).
//! Lookup hierarchy per entry: `(contributor, dimension)` → `(contributor, Uuid::nil())` → identity.
//!
//! ### Masked Cosine Similarity (Phase 2)
//! Requires ≥ 30% dimension overlap (both `has_data_mask[i] = true`). Returns `None` if insufficient.
//!
//! ### Anomaly Detection (Phase 2)
//! Rolling 6-period window z-score (`|z| > 2.0` → anomaly). Population std, min 3 periods.
//!
//! ## BYOC Compute Backend (Phase 5)
//!
//! [`ComputeBackend`] is resolved per-tenant from `tenant_settings.key = 'g27_compute_backend'`:
//! - `Local` (default): calls `atlas-compute-sdk` in-process
//! - `Byoc(url)`: POSTs a `ComputeRequest` JSON to the customer's Lambda — compute stays in their VPC

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait,
    ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use uuid::Uuid;
use chrono::{Datelike, Utc};
use serde_json::{json, Value};
use anyhow::{anyhow, bail, Result};

use crate::types::scorecard::{
    ScaleType, SourceType, ConfidenceLevel, BenchmarkTier, BenchmarkTiers,
    TriggerCategory,
    ScoringMethod, ColdStartStrategy, PercentileBand,
};

use crate::entities::{
    atlas_scorecard::{self as scorecards, ActiveModel as ScorecardActiveModel},
    atlas_scorecard_dimension::{self as dimensions, Model as DimensionModel},
    atlas_scorecard_dimension_option::{self as dim_options},
    atlas_scorecard_entry::{self as entries, ActiveModel as EntryActiveModel},
    atlas_scorecard_dimension_aggregate::ActiveModel as AggregateActiveModel,
    atlas_scorecard_poll_aggregate::ActiveModel as PollAggregateActiveModel,
    atlas_scorecard_time_series::ActiveModel as TimeSeriesActiveModel,
    atlas_rating_session::{self as sessions, ActiveModel as SessionActiveModel},
    atlas_scorecard_display_rule::{self as display_rules, Model as DisplayRuleModel},
    atlas_scorecard_template::{self as templates, Model as TemplateModel},
    atlas_scorecard_template_deployment as deployments,
};


pub struct ScorecardService;

/// G-27 Phase 5: Compute backend strategy.
///
/// Resolved per-tenant from `tenant_settings.key = 'g27_compute_backend'`.
///
/// | Variant        | Behaviour                                                          |
/// |----------------|--------------------------------------------------------------------|
/// | `Local`        | Default. Runs `atlas-compute-sdk` functions in-process (zero RTT) |
/// | `Byoc(String)` | Enterprise BYOC. POSTs a `ComputeRequest` JSON payload to the      |
/// |                | customer's Lambda URL; expects a `ComputeResponse` JSON reply.     |
///
/// When `Byoc` is active, Atlas Platform sends only **aggregated statistics**
/// (the `ComputeRequest`) — never raw PII or individual entries — so compute
/// stays inside the customer's network without any raw data leaving their VPC.
///
/// ## Activation
/// Set `tenant_settings` key `g27_compute_backend` to the Lambda URL:
/// ```sql
/// INSERT INTO tenant_settings (tenant_id, key, value)
/// VALUES ('<tenant>', 'g27_compute_backend', 'https://lambda.customer.com/g27');
/// ```
/// Removing or setting to `'local'` reverts to in-process compute.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ComputeBackend {
    /// In-process execution via `atlas-compute-sdk` — default for all tenants.
    Local,
    /// BYOC: dispatch `ComputeRequest` to a customer-hosted Lambda endpoint.
    /// The URL is read from `tenant_settings.value` for key `g27_compute_backend`.
    Byoc(String),
}

#[allow(dead_code)]
impl ComputeBackend {
    /// Resolve the compute backend for a tenant from `tenant_settings`.
    ///
    /// Returns `Local` if no setting exists or the value is `"local"`.
    /// Returns `Byoc(url)` if the value is a non-empty HTTPS URL.
    pub async fn resolve(db: &DatabaseConnection, tenant_id: uuid::Uuid) -> Self {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        use crate::entities::tenant_setting;

        let row = tenant_setting::Entity::find()
            .filter(tenant_setting::Column::TenantId.eq(tenant_id))
            .filter(tenant_setting::Column::Key.eq("g27_compute_backend"))
            .one(db)
            .await
            .ok()
            .flatten();

        match row {
            Some(r) if !r.value.is_empty() && r.value != "local" => {
                ComputeBackend::Byoc(r.value)
            }
            _ => ComputeBackend::Local,
        }
    }
}

// ── Result types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct SimilarityResult {
    pub scorecard_id: Uuid,
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
    /// Euclidean distance from the target vector — lower is more similar.
    pub distance: f64,
    /// Normalized similarity score: 1.0 / (1.0 + distance). Range: (0, 1].
    pub similarity: f64,
    pub composite_score: Option<f64>,
    pub confidence_level: String,
}

/// A dimension that should be surfaced as a post-activity nudge prompt.
///
/// Returned by `ScorecardService::get_nudge_dimensions_for_activity`.
/// The caller uses this to render the compact rating widget after an activity is logged.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NudgeDimension {
    pub dimension_id: Uuid,
    pub dimension_slug: String,
    pub dimension_name: String,
    /// The rule action that matched: 'surface_as_nudge' | 'require' | 'show'
    pub action: String,
    /// The dimension's scale_type for the UI to render the correct input.
    pub scale_type: String,
    pub scorecard_id: Uuid,
    /// Hint to the UI for what session_type to open. Derived from the activity_type.
    pub session_type_hint: String,
}

// ── Private aggregation intermediate types ───────────────────────────────────

/// Intermediate result for numeric (rating / absolute) aggregation.
struct NumericAgg {
    mean: Option<f64>,
    weighted_mean: Option<f64>,
    std_deviation: Option<f64>,
    min_score: Option<f64>,
    max_score: Option<f64>,
    contributor_count: i32,
    session_count: i32,
    consensus_level: Option<String>,
    benchmark_label: Option<String>,
    benchmark_color: Option<String>,
    display_value: Option<String>,
    vs_global_delta: Option<f64>,
    vs_global_label: Option<String>,
}

/// Intermediate result for boolean aggregation.
struct BooleanAgg {
    percent_true: Option<f64>,
    contributor_count: i32,
    session_count: i32,
    benchmark_label: Option<String>,
    benchmark_color: Option<String>,
    display_value: Option<String>,
}

// ── Core service ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
impl ScorecardService {
    // ── get_or_create ──────────────────────────────────────────────────────

    /// Get or create a scorecard for an entity.
    ///
    /// Idempotent — returns the existing scorecard_id if one already exists for
    /// (template_id, subject_entity_type, subject_entity_id). Safe to call on
    /// every submission without racing — the UNIQUE constraint at the DB level
    /// guarantees exactly-once creation even under concurrent requests.
    ///
    /// # Example
    /// ```rust,ignore
    /// let scorecard_id = ScorecardService::get_or_create(
    ///     db, tenant_id, city_template_id, "atlas_asset", city_asset_id
    /// ).await?;
    /// ```
    pub async fn get_or_create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        template_id: Uuid,
        subject_entity_type: &str,
        subject_entity_id: Uuid,
    ) -> Result<Uuid> {
        // Check for existing scorecard first (hot path — the common case)
        if let Some(existing) = scorecards::Entity::find()
            .filter(scorecards::Column::TemplateId.eq(template_id))
            .filter(scorecards::Column::SubjectEntityType.eq(subject_entity_type))
            .filter(scorecards::Column::SubjectEntityId.eq(subject_entity_id))
            .one(db)
            .await?
        {
            return Ok(existing.id);
        }

        // Create new scorecard
        let now = Utc::now();
        let model = ScorecardActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            template_id: Set(template_id),
            subject_entity_type: Set(subject_entity_type.to_owned()),
            subject_entity_id: Set(subject_entity_id),
            composite_score: Set(None),
            confidence_level: Set("insufficient".to_owned()),
            total_contributors: Set(0),
            total_sessions: Set(0),
            total_entries: Set(0),
            dimension_vector: Set(None),
            dimension_vector_v2: Set(None),
            has_data_mask: Set(None),
            last_computed_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        let inserted = model.insert(db).await;

        match inserted {
            Ok(m) => Ok(m.id),
            Err(sea_orm::DbErr::RecordNotInserted | sea_orm::DbErr::Exec(_)) => {
                // Race condition: another request created it concurrently.
                // Re-fetch and return existing.
                scorecards::Entity::find()
                    .filter(scorecards::Column::TemplateId.eq(template_id))
                    .filter(scorecards::Column::SubjectEntityType.eq(subject_entity_type))
                    .filter(scorecards::Column::SubjectEntityId.eq(subject_entity_id))
                    .one(db)
                    .await?
                    .map(|m| m.id)
                    .ok_or_else(|| anyhow!("scorecard disappeared after race"))
            }
            Err(e) => Err(e.into()),
        }
    }

    // ── open_session ───────────────────────────────────────────────────────

    /// Open a rating session for a discrete occurrence.
    ///
    /// Sessions are the unit of longitudinal tracking. A new session is opened
    /// for each discrete event: a contractor job, a hotel stay, a qualification
    /// call, a pipeline review.
    ///
    /// `context_entity_type` + `context_entity_id` optionally link back to the
    /// originating platform record to avoid data duplication:
    ///   - job → atlas_case (G-13)
    ///   - stay → atlas_reservation (G-23)
    ///   - call → atlas_activity (G-29)
    pub async fn open_session(
        db: &DatabaseConnection,
        scorecard_id: Uuid,
        rater_user_id: Uuid,
        tenant_id: Uuid,
        occurred_at: chrono::DateTime<Utc>,
        session_type: &str,
        context_entity_type: Option<&str>,
        context_entity_id: Option<Uuid>,
        session_label: Option<&str>,
    ) -> Result<Uuid> {
        // Security: verify the scorecard belongs to this tenant before opening a session.
        // Prevents cross-tenant session injection by a caller who knows a foreign scorecard_id.
        let sc = scorecards::Entity::find_by_id(scorecard_id)
            .filter(scorecards::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("scorecard {} not found for tenant {}", scorecard_id, tenant_id))?;

        let _ = sc; // ownership confirmed

        let model = SessionActiveModel {
            id: Set(Uuid::new_v4()),
            scorecard_id: Set(scorecard_id),
            tenant_id: Set(tenant_id),
            rater_user_id: Set(rater_user_id),
            occurred_at: Set(occurred_at),
            session_type: Set(session_type.to_owned()),
            context_entity_type: Set(context_entity_type.map(|s| s.to_owned())),
            context_entity_id: Set(context_entity_id),
            session_label: Set(session_label.map(|s| s.to_owned())),
            status: Set("submitted".to_owned()),
            verification_request_id: Set(None),
            created_at: Set(Utc::now()),
        };

        let inserted = model.insert(db).await?;
        Ok(inserted.id)
    }

    // ── submit_entry ───────────────────────────────────────────────────────

    /// Submit a score for one dimension within a session.
    ///
    /// Enforces the UNIQUE(session, dimension, contributor) constraint — returns
    /// an error if the contributor has already rated this dimension in this session.
    ///
    /// Validates that exactly one of `score` or `option_id` is provided:
    ///   - rating / absolute / boolean → `score`
    ///   - poll_single / poll_multi   → `option_id`
    ///
    /// After successful insert, the entry is queued for aggregate recomputation.
    /// The background job `recompute_scorecard_aggregates` picks it up within 5 min.
    pub async fn submit_entry(
        db: &DatabaseConnection,
        session_id: Uuid,
        scorecard_id: Uuid,
        dimension_id: Uuid,
        tenant_id: Uuid,
        contributor_user_id: Uuid,
        score: Option<f64>,
        option_id: Option<Uuid>,
        source_type: &str,
        context: Option<Value>,
        note: Option<&str>,
    ) -> Result<Uuid> {
        // Security: verify the session belongs to this tenant before accepting an entry.
        // Prevents cross-tenant entry injection by an actor who guesses a session_id.
        let session = sessions::Entity::find_by_id(session_id)
            .filter(sessions::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("session {} not found for tenant {}", session_id, tenant_id))?;

        // Also verify the session is attached to the stated scorecard (prevents
        // an actor from coupling a real session to a foreign scorecard).
        if session.scorecard_id != scorecard_id {
            bail!(
                "session {} does not belong to scorecard {} — mismatched coupling",
                session_id, scorecard_id
            );
        }

        // Only the session's rater may submit entries (tenant-scoped sessions
        // must not accept cross-user writes from another authenticated user).
        if session.rater_user_id != contributor_user_id {
            bail!(
                "session {} rater mismatch: expected {}, got {}",
                session_id, session.rater_user_id, contributor_user_id
            );
        }

        // Parse source_type into the typed enum at the service boundary.
        // This is the only place in the service where source_type is a &str.
        // After this point, business logic uses the typed enum exclusively.
        let typed_source = SourceType::try_from(source_type.to_owned())
            .map_err(|e| anyhow!("invalid source_type: {e}"))?;

        // Validate: exactly one of score or option_id
        match (score.is_some(), option_id.is_some()) {
            (true, true) => bail!("exactly one of score or option_id must be set, not both"),
            (false, false) => bail!("one of score or option_id must be set"),
            _ => {}
        }

        let score_decimal = score.map(|s| {
            rust_decimal::Decimal::from_f64_retain(s)
                .ok_or_else(|| anyhow!("score {s} is not a valid decimal"))
        }).transpose()?;

        // transcript_inferred entries are NEVER auto-verified.
        // They appear in the session UI for the contributor to confirm or reject.
        // Only a call to verify_entry(confirmed: true) will set is_verified = true,
        // which gates their inclusion in composite recomputation.
        //
        // All other source types (manual, community_rating, official_data, …)
        // are verified at insert and counted by recompute_aggregates.
        let is_verified = typed_source.is_auto_verified();

        let model = EntryActiveModel {
            id: Set(Uuid::new_v4()),
            session_id: Set(session_id),
            scorecard_id: Set(scorecard_id),
            dimension_id: Set(dimension_id),
            tenant_id: Set(tenant_id),
            contributor_user_id: Set(contributor_user_id),
            score: Set(score_decimal),
            option_id: Set(option_id),
            source_type: Set(typed_source.to_string()),
            context: Set(context),
            note: Set(note.map(|s| s.to_owned())),
            is_verified: Set(is_verified),
            verification_request_id: Set(None),
            created_at: Set(Utc::now()),
        };

        let inserted = model.insert(db).await
            .map_err(|e| anyhow!("submit_entry failed (duplicate?): {e}"))?;

        // Queue recompute when the entry is already verified (direct ratings).
        // transcript_inferred waits for verify_entry to enqueue.
        if is_verified {
            use crate::entities::outbox_job;
            let job = outbox_job::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                job_type: Set("recompute_scorecard_aggregates".to_owned()),
                payload: Set(json!({ "scorecard_id": scorecard_id })),
                status: Set("pending".to_owned()),
                attempts: Set(0),
                error_message: Set(None),
                locked_by: Set(None),
                locked_at: Set(None),
                run_at: Set(Utc::now()),
                created_at: Set(Utc::now()),
            };
            job.insert(db).await
                .map_err(|e| anyhow!("failed to queue recompute job after submit_entry: {e}"))?;
        }

        Ok(inserted.id)
    }

    // ── verify_entry ─────────────────────────────────────────────────────────────

    /// Confirm or reject a transcript-inferred (or manually submitted unverified) entry.
    ///
    /// Called when a session contributor clicks "Confirm" or "Reject" on an
    /// AI-suggested scorecard entry in the session form.
    ///
    /// - `confirmed = true`:  sets `is_verified = true`, queues aggregate recompute.
    /// - `confirmed = false`: deletes the entry (rejected suggestions are not kept).
    ///
    /// Security: verifies tenant ownership before any mutation.
    pub async fn verify_entry(
        db: &DatabaseConnection,
        entry_id: Uuid,
        tenant_id: Uuid,
        confirmed: bool,
    ) -> Result<()> {
        use entries::Entity as EntryEntity;

        let entry = EntryEntity::find_by_id(entry_id)
            .filter(entries::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("entry {} not found for tenant {}", entry_id, tenant_id))?;

        if confirmed {
            let mut am: entries::ActiveModel = entry.clone().into();
            am.is_verified = Set(true);
            am.update(db).await?;

            // Queue immediate aggregate recompute for this scorecard.
            // The outbox worker will process it within 1.5 seconds.
            use crate::entities::outbox_job;
            let job = outbox_job::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                job_type: Set("recompute_scorecard_aggregates".to_owned()),
                payload: Set(json!({ "scorecard_id": entry.scorecard_id })),
                status: Set("pending".to_owned()),
                attempts: Set(0),
                error_message: Set(None),
                locked_by: Set(None),
                locked_at: Set(None),
                run_at: Set(Utc::now()),
                created_at: Set(Utc::now()),
            };
            job.insert(db).await
                .map_err(|e| anyhow!("failed to queue recompute job: {e}"))?;
        } else {
            // Rejected: delete the entry entirely. No audit trail needed—
            // transcript_inferred entries are AI suggestions, not submitted data.
            EntryEntity::delete_by_id(entry_id).exec(db).await?;
        }

        Ok(())
    }


    // ── recompute_aggregates ───────────────────────────────────────────────

    /// Recompute all aggregates for a scorecard after verified entries change.
    ///
    /// Called by:
    ///   1. The `recompute_scorecard_aggregates` background job (every 5 min).
    ///   2. The G-06 verification webhook when an entry transitions to verified.
    ///
    /// Aggregation branches by dimension `scale_type`:
    ///   - rating / absolute → weighted mean (credibility weight from context JSONB)
    ///   - boolean           → percent_true of verified entries
    ///   - poll_single /     → vote counts per option (written to poll_aggregates)
    ///     poll_multi
    ///
    /// After all dimension aggregates are written, rebuilds the `dimension_vector`
    /// (weighted normalized scores in sort_order sequence) and updates the
    /// scorecard's `composite_score` and `confidence_level`.
    pub async fn recompute_aggregates(
        db: &DatabaseConnection,
        scorecard_id: Uuid,
    ) -> Result<()> {
        let txn = db.begin().await?;

        // Load all verified entries for this scorecard
        let all_entries = entries::Entity::find()
            .filter(entries::Column::ScorecardId.eq(scorecard_id))
            .filter(entries::Column::IsVerified.eq(true))
            .all(&txn)
            .await?;

        if all_entries.is_empty() {
            txn.commit().await?;
            return Ok(());
        }

        // Load the scorecard to get template_id + tenant_id
        let scorecard = scorecards::Entity::find_by_id(scorecard_id)
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("scorecard {scorecard_id} not found"))?;

        // Load all active dimensions for this template (in sort_order)
        let all_dimensions = dimensions::Entity::find()
            .filter(dimensions::Column::TemplateId.eq(scorecard.template_id))
            .filter(dimensions::Column::IsActive.eq(true))
            .order_by_asc(dimensions::Column::SortOrder)
            .all(&txn)
            .await?;

        let mut dimension_vector: Vec<f64> = Vec::with_capacity(all_dimensions.len());
        // v2 masked arrays: f32 values + bool data-presence mask
        let mut dimension_vector_v2: Vec<f32> = Vec::with_capacity(all_dimensions.len());
        let mut has_data_mask: Vec<bool>       = Vec::with_capacity(all_dimensions.len());
        let mut composite_sum: f64 = 0.0;
        let mut composite_weight_sum: f64 = 0.0;
        let mut total_contributors_set: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut total_sessions_set: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        // Load template for cold_start_strategy and saturation_threshold.
        let template = crate::entities::atlas_scorecard_template::Entity::find_by_id(scorecard.template_id)
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("template {} not found", scorecard.template_id))?;
        let saturation_threshold = template.cold_start_saturation_threshold.max(1) as f64;
        let cold_start_strategy = ColdStartStrategy::try_from(template.cold_start_strategy.clone())
            .unwrap_or(ColdStartStrategy::Suppress);
        let scoring_method = ScoringMethod::try_from(template.scoring_method.clone())
            .unwrap_or(ScoringMethod::WeightedMean);

        // Phase 4 — Contributor Calibration: pre-load per-contributor offsets for this
        // template into a HashMap keyed by (contributor_user_id, dimension_id).
        // Applied in compute_numeric_aggregate ONLY when entry_count >= threshold.
        // Map value: (bias_offset: f64, scale_factor: f64).
        //
        // dimension_id IS NULL rows = template-level fallback (key uses Uuid::nil()).
        // Lookup order in compute_numeric_aggregate:
        //   1. (contributor_user_id, dimension_id) — dimension-specific row
        //   2. (contributor_user_id, Uuid::nil())  — template-level fallback
        //   3. (1.0 bias, 1.0 scale)               — no calibration (default)
        let calibrations: std::collections::HashMap<(Uuid, Uuid), (f64, f64)> = {
            let rows = sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT contributor_user_id, \
                        COALESCE(dimension_id, $2::uuid) AS dimension_id, \
                        bias_offset::float8, \
                        scale_factor::float8, \
                        entry_count \
                 FROM atlas_scorecard_contributor_calibration \
                 WHERE template_id = $1 \
                   AND entry_count >= $3",
                vec![
                    sea_orm::Value::Uuid(Some(Box::new(scorecard.template_id))),
                    sea_orm::Value::Uuid(Some(Box::new(Uuid::nil()))),
                    sea_orm::Value::Int(Some(template.calibration_minimum_entries)),
                ],
            );
            match txn.query_all(rows).await {
                Ok(cal_rows) => {
                    cal_rows.iter().filter_map(|r| {
                        let contributor: Uuid = r.try_get("", "contributor_user_id").ok()?;
                        let dim_id:      Uuid = r.try_get("", "dimension_id").ok()?;
                        let bias:        f64  = r.try_get("", "bias_offset").ok()?;
                        let scale:       f64  = r.try_get("", "scale_factor").ok()?;
                        Some(((contributor, dim_id), (bias, scale)))
                    }).collect::<std::collections::HashMap<(Uuid, Uuid), (f64, f64)>>()
                }
                Err(e) => {
                    tracing::warn!(
                        scorecard_id = %scorecard_id,
                        "Failed to load contributor calibrations (non-fatal): {e}"
                    );
                    std::collections::HashMap::new()
                }
            }
        };

        for dim in &all_dimensions {
            let dim_entries: Vec<_> = all_entries
                .iter()
                .filter(|e| e.dimension_id == dim.id)
                .collect();

            if dim_entries.is_empty() {
                dimension_vector.push(0.0);
                continue;
            }

            // Track contributors and sessions across all dimensions
            for e in &dim_entries {
                total_contributors_set.insert(e.contributor_user_id);
                total_sessions_set.insert(e.session_id);
            }

            let weight: f64 = dim.weight.try_into().unwrap_or(1.0);
            let scale_min: f64 = dim.scale_min.try_into().unwrap_or(1.0);
            let scale_max: f64 = dim.scale_max.try_into().unwrap_or(10.0);

            // Parse scale_type into the typed enum at the aggregation boundary.
            // This is the only place in recompute_aggregates where scale_type is a String.
            // All branching below uses ScaleType variants — no string comparisons.
            let scale_type = ScaleType::try_from(dim.scale_type.clone())
                .unwrap_or_else(|e| {
                    tracing::error!("scorecard {} dim {}: {e}", scorecard_id, dim.id);
                    // Treat unknown scale types as Rating to avoid silently dropping data.
                    // This case should only occur if the DB was written by a newer version
                    // of the service that added a scale type this binary doesn't know about.
                    ScaleType::Rating
                });

            match scale_type {
                ScaleType::Rating | ScaleType::Absolute => {
                    let agg = Self::compute_numeric_aggregate(dim, &template, &dim_entries, &calibrations)?;

                    if let Some(weighted_mean) = agg.weighted_mean {
                        let range = scale_max - scale_min;
                        let normalized = if range > 0.0 {
                            if dim.is_inverted {
                                ((scale_max - weighted_mean) / range * weight).clamp(0.0, weight)
                            } else {
                                ((weighted_mean - scale_min) / range * weight).clamp(0.0, weight)
                            }
                        } else {
                            0.0
                        };
                        dimension_vector.push(normalized);
                        // v2: real data — use normalized as-is, mark as present
                        dimension_vector_v2.push(normalized as f32);
                        has_data_mask.push(true);

                        // Confidence-weighted composite (Improvement 2):
                        // Scale each dimension's contribution by how saturated its data is.
                        let contributor_count = agg.contributor_count as f64;
                        let confidence_weight = (contributor_count / saturation_threshold).min(1.0);
                        composite_sum += weighted_mean * weight * confidence_weight;
                        composite_weight_sum += weight * confidence_weight;
                    } else {
                        dimension_vector.push(0.0);
                        // v2: no data — use midpoint placeholder (0.5 * weight), mask = false
                        dimension_vector_v2.push((0.5 * weight) as f32);
                        has_data_mask.push(false);
                    }

                    Self::upsert_numeric_aggregate(&txn, scorecard_id, dim, agg).await?;
                }

                ScaleType::Boolean => {
                    let agg = Self::compute_boolean_aggregate(dim, &dim_entries)?;

                    if let Some(pct) = agg.percent_true {
                        let normalized = (pct / 100.0 * weight).clamp(0.0, weight);
                        dimension_vector.push(normalized);
                        // v2: boolean treated as continuous for similarity (0–weight)
                        dimension_vector_v2.push(normalized as f32);
                        has_data_mask.push(true);

                        let contributor_count = agg.contributor_count as f64;
                        let confidence_weight = (contributor_count / saturation_threshold).min(1.0);
                        composite_sum += (pct / 10.0) * weight * confidence_weight;
                        composite_weight_sum += weight * confidence_weight;
                    } else {
                        dimension_vector.push(0.0);
                        dimension_vector_v2.push((0.5 * weight) as f32);
                        has_data_mask.push(false);
                    }

                    Self::upsert_boolean_aggregate(&txn, scorecard_id, dim, agg).await?;
                }

                ScaleType::PollSingle | ScaleType::PollMulti => {
                    // Poll dimensions do not contribute to composite score or v2 vector.
                    dimension_vector.push(0.0);
                    // v2: polls excluded from similarity vector (mask=false keeps them out)
                    dimension_vector_v2.push(0.0);
                    has_data_mask.push(false);
                    Self::recompute_poll_aggregate(&txn, scorecard_id, dim, &dim_entries).await?;
                }
            }

        }

        // Compute composite score with scoring_method routing (Bug Fix #3)
        //
        // weighted_mean / simple_mean: use confidence-weighted composite computed above.
        // percentile_rank: composite is set after a cross-entity query — stubbed to None
        //   here and filled by compute_percentile_ranks() after this transaction commits.
        let total_entries_count = all_entries.len() as i32;
        let total_contributors_count = total_contributors_set.len() as i32;

        let composite_score = match scoring_method {
            ScoringMethod::WeightedMean | ScoringMethod::SimpleMean => {
                if composite_weight_sum > 0.0 {
                    Some(composite_sum / composite_weight_sum)
                } else {
                    // Cold-start: no confident data. Check strategy.
                    match cold_start_strategy {
                        ColdStartStrategy::Prior | ColdStartStrategy::Category => {
                            // Return global_reference_value from any dimension as the template prior.
                            // We use the first dimension with a reference value as a proxy for the
                            // template-level prior. In practice, templates should have a global ref.
                            all_dimensions.iter()
                                .find_map(|d| d.global_reference_value.as_ref()
                                    .and_then(|r| <rust_decimal::Decimal as TryInto<f64>>::try_into(*r).ok()))
                        }
                        ColdStartStrategy::Suppress => None,
                    }
                }
            }
            ScoringMethod::PercentileRank => {
                // Percentile rank requires cross-entity query run outside this transaction.
                // Set to None here; compute_percentile_ranks() updates it after commit.
                // This is a stub — Phase 3 implements the full materialized view path.
                tracing::debug!("scoring_method=percentile_rank: composite deferred to percentile computation");
                None
            }
        };

        let total_entries = all_entries.len() as i32;
        // Use the typed ConfidenceLevel enum — .to_string() produces the DB string.
        let confidence_level = ConfidenceLevel::from_entry_count(total_entries);

        // Capture ids before scorecard.into() consumes the struct
        let scorecard_template_id = scorecard.template_id;
        let scorecard_tenant_id   = scorecard.tenant_id;

        // Update scorecard — write both legacy vector (JSONB) and v2 masked arrays
        let mut scorecard_am: scorecards::ActiveModel = scorecard.into();
        scorecard_am.composite_score = Set(
            composite_score.and_then(|s| rust_decimal::Decimal::from_f64_retain(s))
        );
        scorecard_am.confidence_level = Set(confidence_level.to_string());
        scorecard_am.total_contributors = Set(total_contributors_count);
        scorecard_am.total_sessions = Set(total_sessions_set.len() as i32);
        scorecard_am.total_entries = Set(total_entries_count);
        // Legacy JSONB vector (f64 euclidean — preserved for backward compat)
        scorecard_am.dimension_vector = Set(Some(json!(dimension_vector)));
        // v2: typed float32 + bool mask (for masked cosine similarity)
        scorecard_am.dimension_vector_v2 = Set(Some(json!(dimension_vector_v2)));
        scorecard_am.has_data_mask = Set(Some(json!(has_data_mask)));
        scorecard_am.last_computed_at = Set(Some(Utc::now()));
        scorecard_am.updated_at = Set(Utc::now());
        scorecard_am.update(&txn).await?;

        txn.commit().await?;

        // Post-commit: compute percentile ranks for all dimensions of this scorecard.
        // This is a separate read-only query across the tenant pool and does not need
        // to be inside the aggregate transaction.
        if let Err(e) = Self::compute_percentile_ranks(db, scorecard_id, scorecard_template_id, scorecard_tenant_id).await {
            // Non-fatal: percentile ranks are best-effort. Log and continue.
            tracing::warn!("compute_percentile_ranks failed for scorecard {scorecard_id}: {e}");
        }

        Ok(())
    }

    // ── find_similar (The Combinator v2 — masked cosine similarity) ───────────

    /// Find entities most similar to a target, using masked cosine similarity.
    ///
    /// ## Why masked cosine instead of Euclidean (Gap 3 fix)
    ///
    /// The old Euclidean approach used `0.0` as a sentinel for "no data on this
    /// dimension". This collapsed the vector space: two entities both missing
    /// dimension 3 looked different from two entities both rated 1.0, even though
    /// their similarity should be equal.
    ///
    /// The new approach:
    ///   1. Build `has_data_mask` in `recompute_aggregates` (true = real data).
    ///   2. Compute cosine similarity ONLY over dimensions where both vectors
    ///      have `has_data_mask[i] = true` (shared scored dimensions).
    ///   3. If overlap < 30% of dimensions, return `None` (insufficient shared data).
    ///   4. Fall back to legacy Euclidean if v2 fields not yet populated.
    ///
    /// Returns results ordered by similarity (most similar first, score range 0–1).
    pub async fn find_similar(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        template_id: Uuid,
        target_vector: Vec<f64>,
        target_mask: Vec<bool>,
        limit: usize,
        min_confidence: &str,
    ) -> Result<Vec<SimilarityResult>> {
        // Security: always filter by tenant_id.
        let candidates = scorecards::Entity::find()
            .filter(scorecards::Column::TenantId.eq(tenant_id))
            .filter(scorecards::Column::TemplateId.eq(template_id))
            .all(db)
            .await?;

        let min_confidence_rank = Self::confidence_rank(min_confidence);
        let n_dims = target_vector.len();
        // Minimum overlap fraction required for a valid masked similarity (30%)
        let min_overlap = (n_dims as f64 * 0.30).ceil() as usize;

        let mut results: Vec<SimilarityResult> = candidates
            .into_iter()
            .filter_map(|sc| {
                if Self::confidence_rank(&sc.confidence_level) < min_confidence_rank {
                    return None;
                }

                let composite_f64: Option<f64> = sc.composite_score
                    .and_then(|d| <rust_decimal::Decimal as TryInto<f64>>::try_into(d).ok());

                // Prefer v2 masked cosine similarity
                if let (Some(v2_val), Some(mask_val)) = (&sc.dimension_vector_v2, &sc.has_data_mask) {
                    let candidate_v2: Vec<f64> = v2_val.as_array()?
                        .iter().map(|x| x.as_f64().unwrap_or(0.0)).collect();
                    let candidate_mask: Vec<bool> = mask_val.as_array()?
                        .iter().map(|x| x.as_bool().unwrap_or(false)).collect();

                    if candidate_v2.len() != n_dims { return None; }

                    let (similarity, distance) = Self::masked_cosine_similarity(
                        &target_vector, &target_mask,
                        &candidate_v2, &candidate_mask,
                        min_overlap,
                    )?;

                    Some(SimilarityResult {
                        scorecard_id: sc.id,
                        subject_entity_type: sc.subject_entity_type,
                        subject_entity_id: sc.subject_entity_id,
                        distance,
                        similarity,
                        composite_score: composite_f64,
                        confidence_level: sc.confidence_level,
                    })
                } else {
                    // Fallback: legacy Euclidean on old JSONB vector
                    let vector = sc.dimension_vector.as_ref().and_then(|v| {
                        v.as_array().map(|arr| {
                            arr.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect::<Vec<f64>>()
                        })
                    })?;
                    if vector.len() != n_dims { return None; }
                    let distance = vector.iter().zip(target_vector.iter())
                        .map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt();
                    let similarity = 1.0 / (1.0 + distance);
                    Some(SimilarityResult {
                        scorecard_id: sc.id,
                        subject_entity_type: sc.subject_entity_type,
                        subject_entity_id: sc.subject_entity_id,
                        distance,
                        similarity,
                        composite_score: composite_f64,
                        confidence_level: sc.confidence_level,
                    })
                }
            })
            .collect();

        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    // ── Masked cosine similarity ────────────────────────────────────────────

    /// Compute cosine similarity restricted to dimensions where both masks are true.
    ///
    /// Returns `Some((similarity, angular_distance))` when overlap >= `min_overlap`.
    /// Returns `None` when overlap is insufficient (not enough shared dimensions).
    ///
    /// # Math
    /// Cosine similarity: dot(A,B) / (|A| * |B|) over masked dimensions.
    /// Angular distance: acos(cosine) / π  ∈ [0, 1] (0 = identical, 1 = orthogonal).
    /// Returned `similarity` = 1.0 - angular_distance ∈ [0, 1].
    ///
    /// # Why angular distance over raw cosine
    /// Raw cosine similarity is not a proper metric (violates triangle inequality).
    /// Angular distance converts it to a true metric, making the ranking stable.
    fn masked_cosine_similarity(
        a_vec: &[f64],
        a_mask: &[bool],
        b_vec: &[f64],
        b_mask: &[bool],
        min_overlap: usize,
    ) -> Option<(f64, f64)> {
        let overlap: Vec<(f64, f64)> = a_vec.iter()
            .zip(b_vec.iter())
            .zip(a_mask.iter().zip(b_mask.iter()))
            .filter_map(|((a, b), (ma, mb))| {
                if *ma && *mb { Some((*a, *b)) } else { None }
            })
            .collect();

        if overlap.len() < min_overlap {
            return None;
        }

        let dot: f64   = overlap.iter().map(|(a, b)| a * b).sum();
        let mag_a: f64 = overlap.iter().map(|(a, _)| a * a).sum::<f64>().sqrt();
        let mag_b: f64 = overlap.iter().map(|(_, b)| b * b).sum::<f64>().sqrt();

        if mag_a < 1e-12 || mag_b < 1e-12 {
            // One or both vectors are zero in the shared space — orthogonal
            return Some((0.0, 1.0));
        }

        let cosine = (dot / (mag_a * mag_b)).clamp(-1.0, 1.0);
        let angular_dist = cosine.acos() / std::f64::consts::PI;
        let similarity = 1.0 - angular_dist;
        Some((similarity, angular_dist))
    }

    // ── Percentile rank computation (Improvement 1) ─────────────────────────

    /// Compute and write percentile ranks for all dimensions of one scorecard.
    ///
    /// Called post-commit by `recompute_aggregates`. Queries all scorecards
    /// for the same (template_id, tenant_id) to build the ranking pool.
    ///
    /// Writes `percentile_rank`, `percentile_cohort_size`, and `percentile_band`
    /// to `atlas_scorecard_dimension_aggregates` for every dimension of this scorecard.
    ///
    /// Algorithm (Percentile of Score in Population):
    ///   rank = (count of scorecards with weighted_mean_score < this_score) / (total - 1) * 100
    ///
    /// Falls back gracefully when < 2 scorecards exist (sets NULL).
    pub async fn compute_percentile_ranks(
        db: &DatabaseConnection,
        scorecard_id: Uuid,
        template_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<()> {
        use sea_orm::{ConnectionTrait, Statement};
        let backend = db.get_database_backend();

        // Load all dimensions for this template (need IDs + sort order)
        let dims = crate::entities::atlas_scorecard_dimension::Entity::find()
            .filter(crate::entities::atlas_scorecard_dimension::Column::TemplateId.eq(template_id))
            .filter(crate::entities::atlas_scorecard_dimension::Column::IsActive.eq(true))
            .all(db)
            .await?;

        for dim in &dims {
            // Pull all weighted_mean_scores for this dimension across the tenant pool
            // (only scorecards that have been computed — NULL means no data yet).
            let rows = db.query_all(Statement::from_string(
                backend,
                format!(
                    "SELECT a.scorecard_id, a.weighted_mean_score \
                     FROM atlas_scorecard_dimension_aggregates a \
                     JOIN atlas_scorecards s ON s.id = a.scorecard_id \
                     WHERE s.tenant_id = '{}' \
                       AND s.template_id = '{}' \
                       AND a.dimension_id = '{}' \
                       AND a.weighted_mean_score IS NOT NULL;",
                    tenant_id, template_id, dim.id
                ),
            ))
            .await?;

            if rows.len() < 2 {
                continue;
            }

            let pool: Vec<(Uuid, f64)> = rows.iter().filter_map(|row| {
                let sc_id: Uuid = row.try_get("", "scorecard_id").ok()?;
                let score: rust_decimal::Decimal = row.try_get("", "weighted_mean_score").ok()?;
                let score_f64: f64 = <rust_decimal::Decimal as TryInto<f64>>::try_into(score).ok()?;
                Some((sc_id, score_f64))
            }).collect();

            let cohort_size = pool.len() as i32;

            let this_score = match pool.iter().find(|(id, _)| *id == scorecard_id) {
                Some((_, s)) => *s,
                None => continue,
            };

            let below_count = pool.iter().filter(|(_, s)| *s < this_score).count();
            let rank = below_count as f64 / (cohort_size - 1) as f64 * 100.0;
            let band = PercentileBand::from_rank(rank);
            let rank_decimal = rust_decimal::Decimal::from_f64_retain(rank);

            db.execute(Statement::from_string(
                backend,
                format!(
                    "UPDATE atlas_scorecard_dimension_aggregates \
                     SET percentile_rank = {rank_val}, \
                         percentile_cohort_size = {cohort}, \
                         percentile_band = '{band}' \
                     WHERE scorecard_id = '{sc}' AND dimension_id = '{dim}';",
                    rank_val = rank_decimal.map(|d| d.to_string()).unwrap_or_else(|| "NULL".to_owned()),
                    cohort = cohort_size,
                    band = band,
                    sc = scorecard_id,
                    dim = dim.id,
                ),
            ))
            .await?;
        }

        Ok(())
    }


    fn compute_numeric_aggregate(
        dim: &DimensionModel,
        template: &crate::entities::atlas_scorecard_template::Model,
        entries: &[&entries::Model],
        // Phase 4: keyed (contributor_user_id, dimension_id) → (bias_offset, scale_factor).
        // dimension_id = Uuid::nil() means template-level fallback.
        // Empty map = calibration disabled (not enough entries or table unavailable).
        calibrations: &std::collections::HashMap<(Uuid, Uuid), (f64, f64)>,
    ) -> Result<NumericAgg> {
        if entries.is_empty() {
            return Ok(NumericAgg {
                mean: None, weighted_mean: None, std_deviation: None,
                min_score: None, max_score: None, contributor_count: 0,
                session_count: 0, consensus_level: None, benchmark_label: None,
                benchmark_color: None, display_value: None,
                vs_global_delta: None, vs_global_label: None,
            });
        }

        let scores: Vec<f64> = entries.iter()
            .filter_map(|e| e.score.as_ref().and_then(|s| <rust_decimal::Decimal as TryInto<f64>>::try_into(*s).ok()))
            .collect();

        if scores.is_empty() {
            return Ok(NumericAgg {
                mean: None, weighted_mean: None, std_deviation: None,
                min_score: None, max_score: None,
                contributor_count: entries.len() as i32,
                session_count: entries.iter().map(|e| e.session_id).collect::<std::collections::HashSet<_>>().len() as i32,
                consensus_level: None, benchmark_label: None, benchmark_color: None,
                display_value: None, vs_global_delta: None, vs_global_label: None,
            });
        }

        // Credibility weight: prefer duration_days or worked_together_months from context
        let credibility_weight = |e: &&entries::Model| -> f64 {
            e.context.as_ref()
                .and_then(|c| {
                    c.get("duration_days").and_then(|v| v.as_f64())
                        .or_else(|| c.get("worked_together_months").and_then(|v| v.as_f64()))
                })
                .map(|v| (v / 30.0).clamp(0.5, 3.0))
                .unwrap_or(1.0)
        };

        let mut weighted_sum = 0.0f64;
        let mut weight_total = 0.0f64;
        for e in entries {
            let score_opt: Option<f64> = e.score.as_ref()
                .and_then(|s| <rust_decimal::Decimal as TryInto<f64>>::try_into(*s).ok());
            if let Some(raw_score) = score_opt {
                // Phase 4: apply contributor calibration if available.
                // Lookup order: dimension-specific → template-level → identity (no-op).
                let (bias, scale) = calibrations
                    .get(&(e.contributor_user_id, dim.id))
                    .or_else(|| calibrations.get(&(e.contributor_user_id, Uuid::nil())))
                    .copied()
                    .unwrap_or((0.0, 1.0)); // identity: no calibration
                let score = ((raw_score - bias) * scale)
                    .clamp(
                        dim.scale_min.try_into().unwrap_or(0.0),
                        dim.scale_max.try_into().unwrap_or(10.0),
                    );
                let w = credibility_weight(e);
                weighted_sum += score * w;
                weight_total += w;
            }
        }

        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let raw_weighted_mean = if weight_total > 0.0 { Some(weighted_sum / weight_total) } else { None };

        // Bayesian prior shrinkage (Gap 1 — dimension level, hierarchical lookup).
        //
        // Lookup order (Decision 4):
        //   1. dim.bayesian_prior_weight       → dimension-specific tuning
        //   2. template.default_bayesian_prior_weight → template-wide default
        //   3. None → no shrinkage (current behavior)
        //
        // When a prior_weight is resolved AND global_reference_value is set, apply
        // James-Stein shrinkage: shrunk = (w*ref + Σ(credibility*scores)) / (w + credibility_total)
        let effective_prior_weight: Option<f64> = dim.bayesian_prior_weight
            .as_ref()
            .and_then(|w| <rust_decimal::Decimal as TryInto<f64>>::try_into(*w).ok())
            .or_else(|| {
                // Fallback: template-level default
                template.default_bayesian_prior_weight
                    .as_ref()
                    .and_then(|w| <rust_decimal::Decimal as TryInto<f64>>::try_into(*w).ok())
            });

        let weighted_mean = match (
            raw_weighted_mean,
            effective_prior_weight,
            dim.global_reference_value.as_ref().and_then(|r| <rust_decimal::Decimal as TryInto<f64>>::try_into(*r).ok()),
        ) {
            (Some(_), Some(prior_weight), Some(global_ref)) if prior_weight > 0.0 => {
                // Apply shrinkage: blend prior with observed credibility-weighted mean
                let shrunk = (prior_weight * global_ref + weighted_sum) / (prior_weight + weight_total);
                Some(shrunk)
            }
            _ => raw_weighted_mean,
        };

        let min_score = scores.iter().cloned().reduce(f64::min);
        let max_score = scores.iter().cloned().reduce(f64::max);

        let variance = if scores.len() > 1 {
            scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (scores.len() - 1) as f64
        } else {
            0.0
        };
        let std_deviation = Some(variance.sqrt());

        let consensus_level = std_deviation.map(|std| {
            if std < 1.0 { "strong_consensus".to_owned() }
            else if std < 2.0 { "consensus".to_owned() }
            else if std < 3.0 { "mixed".to_owned() }
            else { "disputed".to_owned() }
        });

        // Resolve benchmark tier using the typed BenchmarkTiers struct.
        // Deserialize once; no raw JSONB key access in tier logic.
        let tiers: BenchmarkTiers = serde_json::from_value(dim.benchmark_tiers.clone())
            .unwrap_or_default();

        let (benchmark_label, benchmark_color) = weighted_mean.map(|wm| {
            if dim.is_inverted {
                Self::resolve_tier_inverted(wm, &tiers)
            } else {
                Self::resolve_tier(wm, &tiers)
            }
        }).unwrap_or_default();

        // Display value: "Fast: 16 Mbps" or "7.3/10"
        let display_value = weighted_mean.map(|wm| {
            if let Some(unit) = &dim.unit_label {
                format!("{:.0} {}", wm, unit)
            } else {
                let scale_max: f64 = dim.scale_max.try_into().unwrap_or(10.0);
                format!("{:.1}/{:.0}", wm, scale_max)
            }
        });

        let (vs_global_delta, vs_global_label) = if let (Some(wm), Some(ref_val)) = (
            weighted_mean,
            dim.global_reference_value.as_ref().and_then(|d| <rust_decimal::Decimal as TryInto<f64>>::try_into(*d).ok()),
        ) {
            let delta = wm - ref_val;
            let tolerance = 0.2;
            // For inverted dimensions (lower = better), the label direction is flipped:
            // a negative delta (score below reference) means the entity is BETTER than reference.
            let label = if dim.is_inverted {
                if delta < -tolerance { "above" }
                else if delta > tolerance { "below" }
                else { "at" }
            } else {
                if delta > tolerance { "above" }
                else if delta < -tolerance { "below" }
                else { "at" }
            };
            (Some(delta), Some(label.to_owned()))
        } else {
            (None, None)
        };

        let session_count = entries.iter()
            .map(|e| e.session_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as i32;

        Ok(NumericAgg {
            mean: Some(mean),
            weighted_mean,
            std_deviation,
            min_score,
            max_score,
            contributor_count: entries.len() as i32,
            session_count,
            consensus_level,
            benchmark_label,
            benchmark_color,
            display_value,
            vs_global_delta,
            vs_global_label,
        })
    }

    fn compute_boolean_aggregate(
        dim: &DimensionModel,
        entries: &[&entries::Model],
    ) -> Result<BooleanAgg> {
        if entries.is_empty() {
            return Ok(BooleanAgg {
                percent_true: None, contributor_count: 0, session_count: 0,
                benchmark_label: None, benchmark_color: None, display_value: None,
            });
        }

        let true_count = entries.iter()
            .filter(|e| e.score.as_ref()
                .and_then(|s| <rust_decimal::Decimal as TryInto<f64>>::try_into(*s).ok())
                .map(|v: f64| v >= 1.0)
                .unwrap_or(false))
            .count();

        let percent_true = (true_count as f64 / entries.len() as f64) * 100.0;

        // Resolve boolean tier
        let (benchmark_label, benchmark_color) =
            Self::resolve_boolean_tier_typed(percent_true, &dim.benchmark_tiers);

        let display_value = Some(format!("{}% yes", percent_true.round() as i32));

        let session_count = entries.iter()
            .map(|e| e.session_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as i32;

        Ok(BooleanAgg {
            percent_true: Some(percent_true),
            contributor_count: entries.len() as i32,
            session_count,
            benchmark_label,
            benchmark_color,
            display_value,
        })
    }

    async fn upsert_numeric_aggregate(
        txn: &impl sea_orm::ConnectionTrait,
        scorecard_id: Uuid,
        dim: &DimensionModel,
        agg: NumericAgg,
    ) -> Result<()> {
        let to_decimal = |opt: Option<f64>| {
            opt.and_then(|f| rust_decimal::Decimal::from_f64_retain(f))
        };

        // DELETE + INSERT is safe since we're inside a transaction and the
        // primary key is composite (scorecard_id, dimension_id).
        use sea_orm::Statement;
        let db_backend = txn.get_database_backend();
        txn.execute(Statement::from_string(
            db_backend,
            format!(
                "DELETE FROM atlas_scorecard_dimension_aggregates \
                 WHERE scorecard_id = '{}' AND dimension_id = '{}';",
                scorecard_id, dim.id
            ),
        ))
        .await?;

        let model = AggregateActiveModel {
            scorecard_id: Set(scorecard_id),
            dimension_id: Set(dim.id),
            mean_score: Set(to_decimal(agg.mean)),
            weighted_mean_score: Set(to_decimal(agg.weighted_mean)),
            percent_true: Set(None),
            benchmark_label: Set(agg.benchmark_label),
            benchmark_color: Set(agg.benchmark_color),
            display_value: Set(agg.display_value),
            std_deviation: Set(to_decimal(agg.std_deviation)),
            consensus_level: Set(agg.consensus_level),
            min_score: Set(to_decimal(agg.min_score)),
            max_score: Set(to_decimal(agg.max_score)),
            contributor_count: Set(agg.contributor_count),
            session_count: Set(agg.session_count),
            vs_global_delta: Set(to_decimal(agg.vs_global_delta)),
            vs_global_label: Set(agg.vs_global_label),
            // Percentile fields populated post-commit by compute_percentile_ranks()
            percentile_rank: Set(None),
            percentile_cohort_size: Set(None),
            percentile_band: Set(None),
            last_computed_at: Set(Some(Utc::now())),
        };
        model.insert(txn).await?;
        Ok(())
    }

    async fn upsert_boolean_aggregate(
        txn: &impl sea_orm::ConnectionTrait,
        scorecard_id: Uuid,
        dim: &DimensionModel,
        agg: BooleanAgg,
    ) -> Result<()> {
        let to_decimal = |opt: Option<f64>| {
            opt.and_then(|f| rust_decimal::Decimal::from_f64_retain(f))
        };

        use sea_orm::Statement;
        let db_backend = txn.get_database_backend();
        txn.execute(Statement::from_string(
            db_backend,
            format!(
                "DELETE FROM atlas_scorecard_dimension_aggregates \
                 WHERE scorecard_id = '{}' AND dimension_id = '{}';",
                scorecard_id, dim.id
            ),
        ))
        .await?;

        let model = AggregateActiveModel {
            scorecard_id: Set(scorecard_id),
            dimension_id: Set(dim.id),
            mean_score: Set(None),
            weighted_mean_score: Set(None),
            percent_true: Set(to_decimal(agg.percent_true)),
            benchmark_label: Set(agg.benchmark_label),
            benchmark_color: Set(agg.benchmark_color),
            display_value: Set(agg.display_value),
            std_deviation: Set(None),
            consensus_level: Set(None),
            min_score: Set(None),
            max_score: Set(None),
            contributor_count: Set(agg.contributor_count),
            session_count: Set(agg.session_count),
            vs_global_delta: Set(None),
            vs_global_label: Set(None),
            // Percentile fields populated post-commit by compute_percentile_ranks()
            percentile_rank: Set(None),
            percentile_cohort_size: Set(None),
            percentile_band: Set(None),
            last_computed_at: Set(Some(Utc::now())),
        };
        model.insert(txn).await?;
        Ok(())
    }

    async fn recompute_poll_aggregate(
        txn: &impl sea_orm::ConnectionTrait,
        scorecard_id: Uuid,
        dim: &DimensionModel,
        entries: &[&entries::Model],
    ) -> Result<()> {
        use sea_orm::Statement;
        let db_backend = txn.get_database_backend();

        // Load all options for this dimension
        let options = dim_options::Entity::find()
            .filter(dim_options::Column::DimensionId.eq(dim.id))
            .all(txn)
            .await?;

        let total_voters: i32 = entries
            .iter()
            .map(|e| e.contributor_user_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as i32;

        // Count votes per option
        let mut vote_counts: std::collections::HashMap<Uuid, i32> = std::collections::HashMap::new();
        for entry in entries {
            if let Some(opt_id) = entry.option_id {
                *vote_counts.entry(opt_id).or_insert(0) += 1;
            }
        }

        // Delete existing poll aggregates for this (scorecard, dimension)
        txn.execute(Statement::from_string(
            db_backend,
            format!(
                "DELETE FROM atlas_scorecard_poll_aggregates \
                 WHERE scorecard_id = '{}' AND dimension_id = '{}';",
                scorecard_id, dim.id
            ),
        ))
        .await?;

        // Rank options by vote count (descending)
        let mut ranked_options: Vec<(Uuid, i32)> = options.iter()
            .map(|o| (o.id, *vote_counts.get(&o.id).unwrap_or(&0)))
            .collect();
        ranked_options.sort_by(|a, b| b.1.cmp(&a.1));

        for (rank, (option_id, vote_count)) in ranked_options.iter().enumerate() {
            let vote_pct = if total_voters > 0 {
                Some(rust_decimal::Decimal::from_f64_retain(
                    (*vote_count as f64 / total_voters as f64) * 100.0,
                ))
                .flatten()
            } else {
                None
            };

            let model = PollAggregateActiveModel {
                scorecard_id: Set(scorecard_id),
                dimension_id: Set(dim.id),
                option_id: Set(*option_id),
                vote_count: Set(*vote_count),
                vote_pct: Set(vote_pct),
                rank: Set((rank + 1) as i32),
                total_voters: Set(total_voters),
                last_computed_at: Set(Some(Utc::now())),
            };
            model.insert(txn).await?;
        }

        Ok(())
    }

    // ── Benchmark tier resolution (typed) ────────────────────────────────────

    /// Resolve a weighted mean against rating/absolute benchmark tiers.
    ///
    /// For non-inverted dimensions: higher score matches tiers with `min_score`.
    /// Expected tier format (typed): `BenchmarkTier { min_score: Some(f64), label, color, .. }`
    fn resolve_tier(score: f64, tiers: &BenchmarkTiers) -> (Option<String>, Option<String>) {
        let mut best: Option<(&BenchmarkTier, f64)> = None;
        for tier in tiers {
            if let Some(min) = tier.min_score {
                if score >= min {
                    if best.map(|(_, b)| min > b).unwrap_or(true) {
                        best = Some((tier, min));
                    }
                }
            }
        }
        best.map(|(t, _)| (Some(t.label.clone()), Some(t.color.clone())))
            .unwrap_or_default()
    }

    /// Resolve a weighted mean against inverted benchmark tiers (lower score = better).
    ///
    /// For inverted dimensions: lower score matches tiers with `max_score`.
    /// Finds the lowest `max_score` that the actual score is still <= (tightest passing tier).
    /// Expected tier format (typed): `BenchmarkTier { max_score: Some(f64), label, color, .. }`
    fn resolve_tier_inverted(score: f64, tiers: &BenchmarkTiers) -> (Option<String>, Option<String>) {
        let mut best: Option<(&BenchmarkTier, f64)> = None;
        for tier in tiers {
            if let Some(max) = tier.max_score {
                if score <= max {
                    // Prefer the tightest (lowest) max_score that still passes
                    if best.map(|(_, b)| max < b).unwrap_or(true) {
                        best = Some((tier, max));
                    }
                }
            }
        }
        best.map(|(t, _)| (Some(t.label.clone()), Some(t.color.clone())))
            .unwrap_or_default()
    }

    /// Resolve a boolean percentage against boolean benchmark tiers.
    ///
    /// For boolean dimensions: `percent_true` matches tiers with `min_pct`.
    /// Expected tier format (typed): `BenchmarkTier { min_pct: Some(f64), label, color, .. }`
    fn resolve_boolean_tier_typed(pct: f64, tiers: &Value) -> (Option<String>, Option<String>) {
        let Some(arr) = tiers.as_array() else { return (None, None) };

        let mut best: Option<(&Value, f64)> = None;
        for tier in arr {
            if let Some(min_pct) = tier.get("min_pct").and_then(|v| v.as_f64()) {
                if pct >= min_pct {
                    if best.map(|(_, b)| min_pct > b).unwrap_or(true) {
                        best = Some((tier, min_pct));
                    }
                }
            }
        }

        best.map(|(t, _)| (
            t.get("label").and_then(|l| l.as_str()).map(|s| s.to_owned()),
            t.get("color").and_then(|c| c.as_str()).map(|s| s.to_owned()),
        ))
        .unwrap_or_default()
    }


    // ── Confidence level helpers ───────────────────────────────────────────

    // Confidence level helpers have been replaced by ConfidenceLevel::from_entry_count().
    // These stubs are kept for any external callers until migration is complete.
    #[deprecated(note = "Use ConfidenceLevel::from_entry_count(n).to_string() instead")]
    // Used by unit tests (scorecard_lead_unit_tests) and callers that need a
    // string confidence label without a full DB round-trip.
    #[allow(dead_code)]
    pub(crate) fn compute_confidence_level(total_entries: i32) -> String {
        ConfidenceLevel::from_entry_count(total_entries).to_string()
    }

    pub(crate) fn confidence_rank(level: &str) -> u8 {
        // Convert through the typed enum for correctness.
        ConfidenceLevel::try_from(level.to_owned())
            .map(|c| match c {
                ConfidenceLevel::Insufficient => 0,
                ConfidenceLevel::Low          => 1,
                ConfidenceLevel::Medium       => 2,
                ConfidenceLevel::High         => 3,
                ConfidenceLevel::VeryHigh     => 4,
            })
            .unwrap_or(0)
    }

    // ── Z-score anomaly detection helper (Improvement 3) ─────────────────────

    /// Compute z-score of `value` against a trailing `window` of prior period means.
    ///
    /// Returns `(z_score, is_anomaly, anomaly_direction)`.
    /// `is_anomaly = true` when `|z| > 2.0`.
    /// `anomaly_direction = Some("spike")` when `z > 2.0`, `Some("drop")` when `z < -2.0`.
    ///
    /// If the window has zero standard deviation (all values identical), returns z=0 (stable).
    fn compute_z_score(
        value: f64,
        window: &[f64],
    ) -> (f64, bool, Option<String>) {
        if window.is_empty() {
            return (0.0, false, None);
        }
        let n = window.len() as f64;
        let window_mean = window.iter().sum::<f64>() / n;
        let window_var = if window.len() > 1 {
            window.iter().map(|v| (v - window_mean).powi(2)).sum::<f64>() / (n - 1.0)
        } else {
            0.0
        };
        let window_std = window_var.sqrt();

        if window_std < 1e-10 {
            // No variance in window — z-score is undefined; treat as 0 (stable)
            return (0.0, false, None);
        }

        let z = (value - window_mean) / window_std;
        let is_anomaly = z.abs() > 2.0;
        let direction = if is_anomaly {
            Some(if z > 0.0 { "spike".to_owned() } else { "drop".to_owned() })
        } else {
            None
        };
        (z, is_anomaly, direction)
    }


    // ── Time series refresh ────────────────────────────────────────────────

    /// Refresh the time series for one dimension of a scorecard.
    ///
    /// Called by `refresh_scorecard_time_series` background job (hourly).
    /// Buckets verified entries into monthly periods and computes trend direction
    /// by comparing the current period's mean to the prior period's mean.
    ///
    /// Trend direction:
    ///   - 'improving': current mean > prior mean + threshold
    ///   - 'declining': current mean < prior mean - threshold
    ///   - 'stable': within threshold
    ///   - 'insufficient_data': < 2 entries in the period
    pub async fn refresh_time_series_for_dimension(
        db: &DatabaseConnection,
        scorecard_id: Uuid,
        dimension_id: Uuid,
    ) -> Result<()> {
        let txn = db.begin().await?;

        let dim_entries = entries::Entity::find()
            .filter(entries::Column::ScorecardId.eq(scorecard_id))
            .filter(entries::Column::DimensionId.eq(dimension_id))
            .filter(entries::Column::IsVerified.eq(true))
            .filter(entries::Column::Score.is_not_null())
            .all(&txn)
            .await?;

        if dim_entries.is_empty() {
            txn.commit().await?;
            return Ok(());
        }

        // Group by month (YYYY-MM-01)
        let mut monthly: std::collections::BTreeMap<chrono::NaiveDate, Vec<f64>> =
            std::collections::BTreeMap::new();

        for entry in &dim_entries {
            let date = entry.created_at.date_naive();
            let period_start = chrono::NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
                .unwrap_or(date);
            if let Some(score_f64) = entry.score.as_ref()
                .and_then(|s| <rust_decimal::Decimal as TryInto<f64>>::try_into(*s).ok())
            {
                monthly.entry(period_start).or_default().push(score_f64);
            }
        }

        let periods: Vec<_> = monthly.iter().collect();
        // Collect all period means for z-score rolling window computation
        let all_means: Vec<f64> = periods.iter()
            .map(|(_, scores)| scores.iter().sum::<f64>() / scores.len() as f64)
            .collect();

        for (i, (period_start, scores)) in periods.iter().enumerate() {
            let session_count = scores.len() as i32;
            let mean_score = if !scores.is_empty() {
                Some(scores.iter().sum::<f64>() / scores.len() as f64)
            } else {
                None
            };

            // Delta from prior period
            let (delta_from_prior, trend_direction) = if i > 0 {
                let prior_scores = periods[i - 1].1;
                let prior_mean = prior_scores.iter().sum::<f64>() / prior_scores.len() as f64;
                let current_mean = mean_score.unwrap_or(0.0);
                let delta = current_mean - prior_mean;
                let threshold = 0.3;
                let trend = if scores.len() < 2 {
                    "insufficient_data"
                } else if delta > threshold {
                    "improving"
                } else if delta < -threshold {
                    "declining"
                } else {
                    "stable"
                };
                (Some(delta), Some(trend.to_owned()))
            } else {
                (None, None)
            };

            // Anomaly detection via z-score (Improvement 3).
            //
            // Compute z-score against the trailing 6-period rolling window.
            // Minimum 3 periods of trailing history required for a meaningful z-score.
            // |z| > 2.0 → anomaly. Direction: z > 2 = 'spike', z < -2 = 'drop'.
            let (z_score_opt, is_anomaly, anomaly_direction) = if i >= 3 {
                // Trailing window: up to 6 periods before the current period
                let window_start = i.saturating_sub(6);
                let window_end = i; // exclusive: periods[window_start..window_end] are prior
                let window_means: Vec<f64> = all_means[window_start..window_end].to_vec();

                if window_means.len() >= 3 {
                    let (z, anomaly, direction) = Self::compute_z_score(
                        mean_score.unwrap_or(0.0),
                        &window_means,
                    );
                    (Some(z), anomaly, direction)
                } else {
                    (None, false, None)
                }
            } else {
                (None, false, None)
            };

            // Upsert: delete then insert
            use sea_orm::{ConnectionTrait as _, Statement};
            let db_backend = txn.get_database_backend();
            txn.execute(Statement::from_string(
                db_backend,
                format!(
                    "DELETE FROM atlas_scorecard_time_series \
                     WHERE scorecard_id = '{}' AND dimension_id = '{}' \
                       AND period_type = 'monthly' AND period_start = '{}';",
                    scorecard_id, dimension_id, period_start
                ),
            ))
            .await?;

            let model = TimeSeriesActiveModel {
                scorecard_id: Set(scorecard_id),
                dimension_id: Set(dimension_id),
                period_start: Set(**period_start),
                period_type: Set("monthly".to_owned()),
                mean_score: Set(mean_score.and_then(|f| rust_decimal::Decimal::from_f64_retain(f))),
                session_count: Set(session_count),
                contributor_count: Set(session_count), // approx — entries ≈ contributors per period
                delta_from_prior: Set(delta_from_prior.and_then(|f| rust_decimal::Decimal::from_f64_retain(f))),
                trend_direction: Set(trend_direction),
                z_score: Set(z_score_opt.and_then(|f| rust_decimal::Decimal::from_f64_retain(f))),
                is_anomaly: Set(is_anomaly),
                anomaly_direction: Set(anomaly_direction),
            };
            model.insert(&txn).await?;
        }

        txn.commit().await?;
        Ok(())
    }

    // ── Display Rules ───────────────────────────────────────────────────────────

    /// Return all active display rules for a template, ordered by priority ascending.
    ///
    /// The frontend evaluates these client-side against the current entity field values
    /// to determine which dimensions to show, hide, require, or surface as nudges.
    ///
    /// # Tier gate
    /// Starter tenants (no `scorecard_display_rules_enabled` setting) receive an empty
    /// Vec. All dimensions render unconditionally for Starter tier.
    /// Professional+ tenants receive the full active rule list.
    ///
    /// # Performance
    /// Rules are indexed on (template_id, is_active, priority). This query is O(rules)
    /// not O(entries) and is called once per session form render.
    pub async fn get_display_rules(
        db: &DatabaseConnection,
        template_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Vec<DisplayRuleModel>> {
        use crate::entities::tenant_setting;

        // Tier gate: check tenant setting before querying rules.
        let enabled = tenant_setting::Entity::find()
            .filter(tenant_setting::Column::TenantId.eq(tenant_id))
            .filter(tenant_setting::Column::Key.eq("scorecard_display_rules_enabled"))
            .one(db)
            .await?
            .map(|s| s.value == "true" || s.value == "1")
            .unwrap_or(false);

        if !enabled {
            return Ok(vec![]);
        }

        let rules = display_rules::Entity::find()
            .filter(display_rules::Column::TemplateId.eq(template_id))
            .filter(display_rules::Column::TenantId.eq(tenant_id))
            .filter(display_rules::Column::IsActive.eq(true))
            .order_by_asc(display_rules::Column::Priority)
            .all(db)
            .await?;

        Ok(rules)
    }

    /// Return the dimensions that should be surfaced as a post-activity nudge prompt.
    ///
    /// Called when an `atlas_activity` is created for an entity that has a scorecard.
    /// Finds all active display rules with:
    ///   - `trigger_category = 'activity_trigger'`
    ///   - `value_list` contains the `activity_type`
    ///   - `action` IN ('surface_as_nudge', 'require', 'show')
    ///
    /// Returns an empty Vec if:
    ///   - No matching rules exist
    ///   - The tenant's `scorecard_display_rules_enabled` setting is not true
    ///   - No active scorecard exists for this entity + template combination
    pub async fn get_nudge_dimensions_for_activity(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        template_id: Uuid,
        subject_entity_type: &str,
        subject_entity_id: Uuid,
        activity_type: &str,
    ) -> Result<Vec<NudgeDimension>> {
        // Verify a scorecard exists for this entity (required for the scorecard_id return value).
        let scorecard = scorecards::Entity::find()
            .filter(scorecards::Column::TenantId.eq(tenant_id))
            .filter(scorecards::Column::TemplateId.eq(template_id))
            .filter(scorecards::Column::SubjectEntityType.eq(subject_entity_type))
            .filter(scorecards::Column::SubjectEntityId.eq(subject_entity_id))
            .one(db)
            .await?;

        let scorecard_id = match scorecard {
            Some(sc) => sc.id,
            None => return Ok(vec![]), // No scorecard for this entity yet — nothing to nudge.
        };

        // Load all active activity_trigger rules for this template.
        let candidate_rules = display_rules::Entity::find()
            .filter(display_rules::Column::TemplateId.eq(template_id))
            .filter(display_rules::Column::TenantId.eq(tenant_id))
            .filter(display_rules::Column::IsActive.eq(true))
            .filter(display_rules::Column::TriggerCategory.eq(
                TriggerCategory::ActivityTrigger.to_string()
            ))
            .order_by_asc(display_rules::Column::Priority)
            .all(db)
            .await?;

        // Filter client-side: rules whose value_list contains the activity_type.
        // (SQL JSONB array containment would work but this keeps the query simple
        // and the rule count per template is small — typically < 50.)
        let matching_rules: Vec<&DisplayRuleModel> = candidate_rules
            .iter()
            .filter(|rule| {
                let nudge_actions = ["surface_as_nudge", "require", "show"];
                nudge_actions.contains(&rule.action.as_str())
                    && rule.value_list_as_strings().iter().any(|v| v == activity_type)
            })
            .collect();

        if matching_rules.is_empty() {
            return Ok(vec![]);
        }

        // Load dimension details for each matching rule.
        let mut nudge_dims: Vec<NudgeDimension> = Vec::new();
        for rule in matching_rules {
            let dim_id = match rule.dimension_id {
                Some(id) => id,
                None => continue, // Category-level rules without a dimension ID are skipped for nudge.
            };

            let dim = dimensions::Entity::find_by_id(dim_id)
                .filter(dimensions::Column::IsActive.eq(true))
                .one(db)
                .await?;

            if let Some(dim) = dim {
                // Derive a session_type_hint from the activity_type for the UI.
                let session_type_hint = match activity_type {
                    "call" | "discovery_call" => "call",
                    "demo"                    => "demo",
                    "meeting"                 => "meeting",
                    "email" | "email_thread"  => "email_thread",
                    _                         => "meeting",
                }.to_owned();

                nudge_dims.push(NudgeDimension {
                    dimension_id: dim.id,
                    dimension_slug: dim.slug.clone(),
                    dimension_name: dim.name.clone(),
                    action: rule.action.clone(),
                    scale_type: dim.scale_type.clone(),
                    scorecard_id,
                    session_type_hint,
                });
            }
        }

        Ok(nudge_dims)
    }

    // ── Phase 4: Contributor Calibration ────────────────────────────────────────

    /// Compute and persist per-contributor bias offsets for a template.
    ///
    /// This is the weekly background job (`calibrate_scorecard_contributors`). It computes:
    ///   - `bias_offset`  = contributor's per-dimension mean − ensemble mean
    ///   - `scale_factor` = contributor's per-dimension std / ensemble std (1.0 if std = 0)
    ///
    /// Only contributors with `entry_count >= template.calibration_minimum_entries` have
    /// calibrations written. Others are skipped (not enough data to calibrate reliably).
    ///
    /// Written to `atlas_scorecard_contributor_calibration` using UPSERT (ON CONFLICT UPDATE),
    /// so re-running is always safe and idempotent.
    ///
    /// Applied by `compute_numeric_aggregate` during `recompute_aggregates` — the calibration
    /// map is loaded once per scorecard recompute and applied per-entry.
    ///
    /// ## Algorithm
    ///
    /// For each (contributor, dimension) pair with enough entries:
    ///
    ///   1. Compute contributor mean:  μ_c = mean(contributor entries for this dim)
    ///   2. Compute ensemble mean:     μ_e = mean(ALL verified entries for this dim across template)
    ///   3. bias_offset = μ_c − μ_e   (positive = this contributor rates high vs peers)
    ///   4. scale_factor = σ_c / σ_e  (clamped to [0.1, 3.0]; 1.0 if either std = 0)
    ///
    /// After applying: calibrated_score = (raw_score − bias_offset) × scale_factor
    /// This shifts and scales each contributor's scores to align with the ensemble.
    pub async fn calibrate_contributor_bias(
        db:          &DatabaseConnection,
        template_id: Uuid,
    ) -> Result<usize> {
        // Load all verified entries for this template via raw SQL join.
        // atlas_scorecard_entry has no declared sea-orm Relation to atlas_scorecard,
        // so we use a parameterised query rather than .inner_join().
        let all_entries: Vec<entries::Model> = {
            let rows = db.query_all(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT e.* \
                 FROM atlas_scorecard_entries e \
                 JOIN atlas_scorecards sc ON sc.id = e.scorecard_id \
                 WHERE sc.template_id = $1 \
                   AND e.is_verified = true",
                vec![sea_orm::Value::Uuid(Some(Box::new(template_id)))],
            )).await?;
            rows.iter().filter_map(|r| {
                Some(entries::Model {
                    id:                       r.try_get("", "id").ok()?,
                    session_id:               r.try_get("", "session_id").ok()?,
                    scorecard_id:             r.try_get("", "scorecard_id").ok()?,
                    dimension_id:             r.try_get("", "dimension_id").ok()?,
                    tenant_id:                r.try_get("", "tenant_id").ok()?,
                    contributor_user_id:      r.try_get("", "contributor_user_id").ok()?,
                    score:                    r.try_get("", "score").ok()?,
                    option_id:                r.try_get("", "option_id").ok()?,
                    source_type:              r.try_get("", "source_type").ok()?,
                    context:                  r.try_get("", "context").ok()?,
                    note:                     r.try_get("", "note").ok()?,
                    is_verified:              r.try_get("", "is_verified").ok()?,
                    verification_request_id:  r.try_get("", "verification_request_id").ok()?,
                    created_at:               r.try_get("", "created_at").ok()?,
                })
            }).collect()
        };

        if all_entries.is_empty() {
            return Ok(0);
        }

        // ── Step 2: Load template for calibration_minimum_entries threshold ────
        let template = crate::entities::atlas_scorecard_template::Entity::find_by_id(template_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("template {template_id} not found"))?;
        let min_entries = template.calibration_minimum_entries as usize;

        // ── Step 3: Compute ensemble stats per dimension ───────────────────────
        // Group all scores by dimension_id → compute ensemble mean + std
        let mut ensemble_scores: std::collections::HashMap<Uuid, Vec<f64>> = std::collections::HashMap::new();
        for e in &all_entries {
            if let Some(score) = e.score.as_ref().and_then(|s| <rust_decimal::Decimal as TryInto<f64>>::try_into(*s).ok()) {
                ensemble_scores.entry(e.dimension_id).or_default().push(score);
            }
        }

        let ensemble_stats: std::collections::HashMap<Uuid, (f64, f64)> = ensemble_scores
            .iter()
            .map(|(dim_id, scores)| {
                let n = scores.len() as f64;
                let mean = scores.iter().sum::<f64>() / n;
                let variance = if n > 1.0 {
                    scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (n - 1.0)
                } else {
                    0.0
                };
                (*dim_id, (mean, variance.sqrt()))
            })
            .collect();

        // ── Step 4: Compute per-contributor stats per dimension ────────────────
        // Group by (contributor_user_id, dimension_id)
        let mut contributor_scores: std::collections::HashMap<(Uuid, Uuid), Vec<f64>> = std::collections::HashMap::new();
        for e in &all_entries {
            if let Some(score) = e.score.as_ref().and_then(|s| <rust_decimal::Decimal as TryInto<f64>>::try_into(*s).ok()) {
                contributor_scores
                    .entry((e.contributor_user_id, e.dimension_id))
                    .or_default()
                    .push(score);
            }
        }

        // ── Step 5: Upsert calibration rows ────────────────────────────────────
        let mut upserted = 0usize;

        for ((contributor_id, dim_id), scores) in &contributor_scores {
            if scores.len() < min_entries {
                // Not enough data for this contributor on this dimension — skip.
                continue;
            }

            let (ensemble_mean, ensemble_std) = ensemble_stats
                .get(dim_id)
                .copied()
                .unwrap_or((0.0, 0.0));

            let n = scores.len() as f64;
            let contributor_mean = scores.iter().sum::<f64>() / n;
            let contributor_variance = if n > 1.0 {
                scores.iter().map(|s| (s - contributor_mean).powi(2)).sum::<f64>() / (n - 1.0)
            } else {
                0.0
            };
            let contributor_std = contributor_variance.sqrt();

            let bias_offset  = contributor_mean - ensemble_mean;
            let scale_factor = if ensemble_std > 0.01 && contributor_std > 0.01 {
                (contributor_std / ensemble_std).clamp(0.1, 3.0)
            } else {
                1.0 // no scaling when either std is near-zero
            };

            db.execute_unprepared(&format!(
                "INSERT INTO atlas_scorecard_contributor_calibration \
                   (id, contributor_user_id, template_id, dimension_id, \
                    bias_offset, scale_factor, entry_count, last_calibrated_at, created_at) \
                 VALUES \
                   (gen_random_uuid(), '{contributor_id}', '{template_id}', '{dim_id}', \
                    {bias_offset}, {scale_factor}, {entry_count}, NOW(), NOW()) \
                 ON CONFLICT ON CONSTRAINT idx_contrib_calibration_dim_unique \
                 DO UPDATE SET \
                   bias_offset        = EXCLUDED.bias_offset, \
                   scale_factor       = EXCLUDED.scale_factor, \
                   entry_count        = EXCLUDED.entry_count, \
                   last_calibrated_at = EXCLUDED.last_calibrated_at",
                entry_count = scores.len()
            ))
            .await
            .map_err(|e| anyhow!("calibration upsert failed for contributor {contributor_id} dim {dim_id}: {e}"))?;

            upserted += 1;
        }

        tracing::info!(
            %template_id,
            upserted,
            contributors = contributor_scores.len(),
            "calibrate_contributor_bias: completed"
        );

        Ok(upserted)
    }

    // ── App-instance deployment helpers (G-27 runtime) ────────────────────────

    /// Templates deployed and enabled for an app instance, scoped to `tenant_id`.
    ///
    /// Returns only rows where:
    /// - deployment `(app_instance_id, is_enabled=true)` exists
    /// - deployment `tenant_id` matches
    /// - template `tenant_id` matches (defense in depth)
    ///
    /// Wrong-tenant / wrong-instance combinations yield an empty list (not an error).
    /// Optional `published_only`: when `Some(true)`, also requires `is_published`.
    pub async fn templates_enabled_for_instance(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        app_instance_id: Uuid,
        published_only: Option<bool>,
    ) -> Result<Vec<TemplateModel>> {
        let enabled = deployments::Entity::find()
            .filter(deployments::Column::AppInstanceId.eq(app_instance_id))
            .filter(deployments::Column::TenantId.eq(tenant_id))
            .filter(deployments::Column::IsEnabled.eq(true))
            .all(db)
            .await?;

        if enabled.is_empty() {
            return Ok(vec![]);
        }

        let template_ids: Vec<Uuid> = enabled.iter().map(|d| d.template_id).collect();
        let mut query = templates::Entity::find()
            .filter(templates::Column::Id.is_in(template_ids))
            .filter(templates::Column::TenantId.eq(tenant_id));

        if published_only == Some(true) {
            query = query.filter(templates::Column::IsPublished.eq(true));
        }

        let rows = query
            .order_by_asc(templates::Column::Name)
            .all(db)
            .await?;

        Ok(rows)
    }
}
