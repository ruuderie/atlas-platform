use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait,
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use uuid::Uuid;
use chrono::{Datelike, Utc};
use serde_json::{json, Value};
use anyhow::{anyhow, bail, Result};

use crate::entities::{
    atlas_scorecard::{self as scorecards, ActiveModel as ScorecardActiveModel},
    atlas_scorecard_dimension::{self as dimensions, Model as DimensionModel},
    atlas_scorecard_dimension_option::{self as dim_options},
    atlas_scorecard_entry::{self as entries, ActiveModel as EntryActiveModel},
    atlas_scorecard_dimension_aggregate::ActiveModel as AggregateActiveModel,
    atlas_scorecard_poll_aggregate::ActiveModel as PollAggregateActiveModel,
    atlas_scorecard_time_series::ActiveModel as TimeSeriesActiveModel,
    atlas_rating_session::{self as sessions, ActiveModel as SessionActiveModel},
};

pub struct ScorecardService;

// ── Result types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
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
            last_computed_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
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

        let model = EntryActiveModel {
            id: Set(Uuid::new_v4()),
            session_id: Set(session_id),
            scorecard_id: Set(scorecard_id),
            dimension_id: Set(dimension_id),
            tenant_id: Set(tenant_id),
            contributor_user_id: Set(contributor_user_id),
            score: Set(score_decimal),
            option_id: Set(option_id),
            source_type: Set(source_type.to_owned()),
            context: Set(context),
            note: Set(note.map(|s| s.to_owned())),
            is_verified: Set(false),
            verification_request_id: Set(None),
            created_at: Set(Utc::now()),
        };

        let inserted = model.insert(db).await
            .map_err(|e| anyhow!("submit_entry failed (duplicate?): {e}"))?;

        Ok(inserted.id)
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
        let mut composite_sum: f64 = 0.0;
        let mut composite_weight_sum: f64 = 0.0;
        let mut total_contributors_set: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut total_sessions_set: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

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

            match dim.scale_type.as_str() {
                "rating" | "absolute" => {
                    let agg = Self::compute_numeric_aggregate(dim, &dim_entries)?;

                    if let Some(weighted_mean) = agg.weighted_mean {
                        // Normalized for vector: (score - min) / (max - min) * weight
                        let range = scale_max - scale_min;
                        let normalized = if range > 0.0 {
                            ((weighted_mean - scale_min) / range * weight).clamp(0.0, weight)
                        } else {
                            0.0
                        };
                        dimension_vector.push(normalized);
                        composite_sum += weighted_mean * weight;
                        composite_weight_sum += weight;
                    } else {
                        dimension_vector.push(0.0);
                    }

                    // Upsert dimension aggregate
                    Self::upsert_numeric_aggregate(&txn, scorecard_id, dim, agg).await?;
                }

                "boolean" => {
                    let agg = Self::compute_boolean_aggregate(dim, &dim_entries)?;

                    if let Some(pct) = agg.percent_true {
                        // Normalize boolean: true_pct as 0.0-1.0
                        let normalized = (pct / 100.0 * weight).clamp(0.0, weight);
                        dimension_vector.push(normalized);
                        composite_sum += (pct / 10.0) * weight; // boolean on 0-10 scale for composite
                        composite_weight_sum += weight;
                    } else {
                        dimension_vector.push(0.0);
                    }

                    Self::upsert_boolean_aggregate(&txn, scorecard_id, dim, agg).await?;
                }

                "poll_single" | "poll_multi" => {
                    // Poll dimensions don't contribute to composite score
                    dimension_vector.push(0.0);
                    Self::recompute_poll_aggregate(&txn, scorecard_id, dim, &dim_entries).await?;
                }

                unknown => {
                    tracing::warn!("unknown scale_type '{unknown}' on dimension {}", dim.id);
                    dimension_vector.push(0.0);
                }
            }
        }

        // Compute composite score and confidence level
        let composite_score = if composite_weight_sum > 0.0 {
            Some(composite_sum / composite_weight_sum)
        } else {
            None
        };

        let total_entries = all_entries.len() as i32;
        let confidence_level = Self::compute_confidence_level(total_entries);

        // Update scorecard
        let mut scorecard_am: scorecards::ActiveModel = scorecard.into();
        scorecard_am.composite_score = Set(
            composite_score.and_then(|s| rust_decimal::Decimal::from_f64_retain(s))
        );
        scorecard_am.confidence_level = Set(confidence_level);
        scorecard_am.total_contributors = Set(total_contributors_set.len() as i32);
        scorecard_am.total_sessions = Set(total_sessions_set.len() as i32);
        scorecard_am.total_entries = Set(total_entries);
        scorecard_am.dimension_vector = Set(Some(json!(dimension_vector)));
        scorecard_am.last_computed_at = Set(Some(Utc::now()));
        scorecard_am.updated_at = Set(Utc::now());
        scorecard_am.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    // ── find_similar (The Combinator) ──────────────────────────────────────

    /// Find entities most similar to a target vector.
    ///
    /// Uses Euclidean distance across the dimension vector. Runs in Rust for
    /// catalogs < ~10K entities (sufficient for most tenants). For very large
    /// catalogs (100K+), consider pgvector or a dedicated similarity service.
    ///
    /// Returns results ordered by similarity (most similar first).
    ///
    /// # Example — predictive lead scoring
    /// ```rust,ignore
    /// let similar = ScorecardService::find_similar(
    ///     db, lead_qualification_template_id,
    ///     new_lead_vector, 20, "medium"
    /// ).await?;
    /// // Returns historically similar leads with their conversion outcomes
    /// ```
    pub async fn find_similar(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        template_id: Uuid,
        target_vector: Vec<f64>,
        limit: usize,
        min_confidence: &str,
    ) -> Result<Vec<SimilarityResult>> {
        // Security: always filter by tenant_id to prevent cross-tenant data leakage.
        // A caller who supplies another tenant's template_id must not see foreign subjects.
        let candidates = scorecards::Entity::find()
            .filter(scorecards::Column::TenantId.eq(tenant_id))
            .filter(scorecards::Column::TemplateId.eq(template_id))
            .all(db)
            .await?;

        let min_confidence_rank = Self::confidence_rank(min_confidence);

        let mut results: Vec<SimilarityResult> = candidates
            .into_iter()
            .filter_map(|sc| {
                // Filter by minimum confidence
                if Self::confidence_rank(&sc.confidence_level) < min_confidence_rank {
                    return None;
                }

                // Parse dimension vector from JSONB
                let vector = sc.dimension_vector.as_ref().and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .map(|x| x.as_f64().unwrap_or(0.0))
                            .collect::<Vec<f64>>()
                    })
                })?;

                if vector.len() != target_vector.len() {
                    return None;
                }

                // Euclidean distance
                let distance = vector
                    .iter()
                    .zip(target_vector.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();

                let similarity = 1.0 / (1.0 + distance);
                let composite_f64: Option<f64> = sc.composite_score
                    .and_then(|d| <rust_decimal::Decimal as TryInto<f64>>::try_into(d).ok());

                Some(SimilarityResult {
                    scorecard_id: sc.id,
                    subject_entity_type: sc.subject_entity_type,
                    subject_entity_id: sc.subject_entity_id,
                    distance,
                    similarity,
                    composite_score: composite_f64,
                    confidence_level: sc.confidence_level,
                })
            })
            .collect();

        // Sort by similarity descending (lowest distance = most similar)
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    // ── Internal aggregation helpers ───────────────────────────────────────

    fn compute_numeric_aggregate(
        dim: &DimensionModel,
        entries: &[&entries::Model],
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
            if let Some(score) = score_opt {
                let w = credibility_weight(e);
                weighted_sum += score * w;
                weight_total += w;
            }
        }

        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let weighted_mean = if weight_total > 0.0 { Some(weighted_sum / weight_total) } else { None };
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

        // Resolve benchmark tier
        let (benchmark_label, benchmark_color) = weighted_mean
            .map(|wm| Self::resolve_rating_tier(wm, &dim.benchmark_tiers))
            .unwrap_or_default();

        // Display value: "Fast: 16 Mbps" or "7.3/10"
        let display_value = weighted_mean.map(|wm| {
            if let Some(unit) = &dim.unit_label {
                format!("{:.0} {}", wm, unit)
            } else {
                let scale_max: f64 = dim.scale_max.try_into().unwrap_or(10.0);
                format!("{:.1}/{:.0}", wm, scale_max)
            }
        });

        // Global reference comparison
        let (vs_global_delta, vs_global_label) = if let (Some(wm), Some(ref_val)) = (
            weighted_mean,
            dim.global_reference_value.as_ref().and_then(|d| <rust_decimal::Decimal as TryInto<f64>>::try_into(*d).ok()),
        ) {
            let delta = wm - ref_val;
            let tolerance = 0.2; // ±0.2 = "at the bar"
            let label = if delta > tolerance { "above" }
                        else if delta < -tolerance { "below" }
                        else { "at" };
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
            Self::resolve_boolean_tier(percent_true, &dim.benchmark_tiers);

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

    // ── Benchmark tier resolution ──────────────────────────────────────────

    /// Resolve a numeric score against the dimension's benchmark_tiers JSONB.
    ///
    /// Expected tier format:
    /// `[{"label": "Outstanding", "min_score": 9.0, "color": "#00cc44"}, ...]`
    /// Tiers are matched by largest `min_score` that the score satisfies.
    fn resolve_rating_tier(score: f64, tiers: &Value) -> (Option<String>, Option<String>) {
        let Some(arr) = tiers.as_array() else { return (None, None) };

        // Find the matching tier: highest min_score that score >= min_score
        let mut best: Option<(&Value, f64)> = None;
        for tier in arr {
            if let Some(min) = tier.get("min_score").and_then(|v| v.as_f64()) {
                if score >= min {
                    if best.map(|(_, b)| min > b).unwrap_or(true) {
                        best = Some((tier, min));
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

    /// Resolve a boolean percentage against boolean benchmark_tiers.
    ///
    /// Expected tier format:
    /// `[{"label": "Most say clean", "min_pct": 70, "color": "#00cc44"}, ...]`
    fn resolve_boolean_tier(pct: f64, tiers: &Value) -> (Option<String>, Option<String>) {
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

    pub(crate) fn compute_confidence_level(total_entries: i32) -> String {
        match total_entries {
            0..=4   => "insufficient".to_owned(),
            5..=9   => "low".to_owned(),
            10..=49 => "medium".to_owned(),
            50..=199 => "high".to_owned(),
            _       => "very_high".to_owned(),
        }
    }

    pub(crate) fn confidence_rank(level: &str) -> u8 {
        match level {
            "insufficient" => 0,
            "low"          => 1,
            "medium"       => 2,
            "high"         => 3,
            "very_high"    => 4,
            _              => 0,
        }
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
            };
            model.insert(&txn).await?;
        }

        txn.commit().await?;
        Ok(())
    }
}
