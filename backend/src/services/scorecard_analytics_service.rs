#![allow(dead_code)]
//! # Scorecard Analytics Service (G-27 Phase 3)
//!
//! Provides portfolio-level analytics for the Atlas Scorecards engine.
//! Reads from `mv_scorecard_portfolio_analytics` (materialized view) and
//! `v_scorecard_recent_anomalies` (live view). Never reads raw entry rows.
//!
//! ## Endpoints served
//!
//! | Method | Path | Function |
//! |--------|------|----------|
//! | GET | `/api/templates/:id/analytics` | [`portfolio_stats`] |
//! | GET | `/api/templates/:id/leaderboard` | [`leaderboard`] |
//! | GET | `/api/templates/:id/anomalies` | [`recent_anomalies`] |
//! | POST | `/api/templates/:id/analytics/refresh` | [`refresh_portfolio_view`] |
//!
//! ## BYOC integration
//!
//! [`peer_pool_snapshot`] assembles the `PeerPool` struct included in BYOC
//! `ComputeRequest` payloads (Phase 5). It queries the materialized view for
//! aggregate distribution stats — raw records are never included.

use anyhow::Result;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Response types ────────────────────────────────────────────────────────────

/// Per-dimension distribution statistics for a template's portfolio.
/// Returned by `portfolio_stats()` and used as the peer pool in BYOC requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionPortfolioStats {
    pub dimension_id:   Uuid,
    pub dimension_slug: String,
    pub dimension_name: String,
    /// Distinct scorecards with at least one valid entry for this dimension.
    pub cohort_size:    i64,
    pub pool_mean:      Option<f64>,
    pub pool_std_dev:   Option<f64>,
    pub pool_min:       Option<f64>,
    pub pool_p25:       Option<f64>,
    pub pool_p50:       Option<f64>,
    pub pool_p75:       Option<f64>,
    pub pool_p90:       Option<f64>,
    pub pool_max:       Option<f64>,
    /// Scorecards trending improving in their most recent time series period.
    pub improving_count: i64,
    /// Scorecards trending declining in their most recent time series period.
    pub declining_count: i64,
}

/// Portfolio-wide stats for a single template, returned by `portfolio_stats()`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub template_id:   Uuid,
    pub tenant_id:     Uuid,
    /// Total distinct scorecards across all dimensions (max cohort_size).
    pub total_scorecards: i64,
    /// Timestamp of the last materialized view refresh.
    pub refreshed_at:  Option<chrono::DateTime<chrono::Utc>>,
    pub dimensions:    Vec<DimensionPortfolioStats>,
}

/// A single entry in the leaderboard — one scorecard ranked by composite score.
#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank:            i64,
    pub scorecard_id:    Uuid,
    pub subject_id:      String,
    pub subject_type:    String,
    pub composite_score: Option<f64>,
    pub confidence_level: String,
    /// Percentile rank computed from the materialized view pool.
    pub percentile_rank: Option<f64>,
    pub trend_direction: Option<String>,
}

/// A recent anomaly alert — one time series row with is_anomaly = true.
#[derive(Debug, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub scorecard_id:     Uuid,
    pub dimension_id:     Uuid,
    pub dimension_slug:   String,
    pub dimension_name:   String,
    pub period_start:     chrono::NaiveDate,
    pub mean_score:       Option<f64>,
    pub z_score:          Option<f64>,
    pub anomaly_direction: Option<String>,
}

/// Peer pool snapshot included in BYOC `ComputeRequest` payloads (Phase 5).
/// Contains aggregate statistics only — no raw records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerPoolSnapshot {
    pub cohort_size: i64,
    pub pool_mean:   Option<f64>,
    pub pool_std_dev: Option<f64>,
    pub pool_p25:    Option<f64>,
    pub pool_p50:    Option<f64>,
    pub pool_p75:    Option<f64>,
    pub pool_p90:    Option<f64>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct ScorecardAnalyticsService;

impl ScorecardAnalyticsService {
    // ── Portfolio Stats ──────────────────────────────────────────────────────

    /// Returns per-dimension distribution statistics for a template.
    ///
    /// Reads from `mv_scorecard_portfolio_analytics`. If the view is empty
    /// for this template/tenant (not yet refreshed), returns empty dimensions
    /// with a `refreshed_at = None` signal — callers should show a
    /// "Portfolio data being calculated" loading state.
    ///
    /// Powers: `GET /api/templates/:id/analytics`
    /// Powers: `g27scPortfolioPanel` LWC callout in AppExchange
    pub async fn portfolio_stats(
        db:          &DatabaseConnection,
        template_id: Uuid,
        tenant_id:   Uuid,
    ) -> Result<PortfolioStats> {
        let backend = db.get_database_backend();

        let rows = db.query_all(Statement::from_sql_and_values(
            backend,
            "SELECT
               dimension_id,
               dimension_slug,
               dimension_name,
               cohort_size,
               pool_mean,
               pool_std_dev,
               pool_min,
               pool_p25,
               pool_p50,
               pool_p75,
               pool_p90,
               pool_max,
               improving_count,
               declining_count,
               refreshed_at
             FROM mv_scorecard_portfolio_analytics
             WHERE template_id = $1
               AND tenant_id   = $2
             ORDER BY dimension_slug",
            vec![
                sea_orm::Value::Uuid(Some(Box::new(template_id))),
                sea_orm::Value::Uuid(Some(Box::new(tenant_id))),
            ],
        ))
        .await?;

        let mut dimensions = Vec::with_capacity(rows.len());
        let mut total_scorecards: i64 = 0;
        let mut refreshed_at: Option<chrono::DateTime<chrono::Utc>> = None;

        for row in &rows {
            let dim_id: Uuid = row.try_get("", "dimension_id")?;
            let cohort: i64 = row.try_get("", "cohort_size")?;
            if cohort > total_scorecards { total_scorecards = cohort; }

            if refreshed_at.is_none() {
                refreshed_at = row.try_get::<Option<chrono::DateTime<chrono::Utc>>>("", "refreshed_at")
                    .ok()
                    .flatten();
            }

            dimensions.push(DimensionPortfolioStats {
                dimension_id:    dim_id,
                dimension_slug:  row.try_get("", "dimension_slug")?,
                dimension_name:  row.try_get("", "dimension_name")?,
                cohort_size:     cohort,
                pool_mean:       row.try_get::<Option<f64>>("", "pool_mean").ok().flatten(),
                pool_std_dev:    row.try_get::<Option<f64>>("", "pool_std_dev").ok().flatten(),
                pool_min:        row.try_get::<Option<f64>>("", "pool_min").ok().flatten(),
                pool_p25:        row.try_get::<Option<f64>>("", "pool_p25").ok().flatten(),
                pool_p50:        row.try_get::<Option<f64>>("", "pool_p50").ok().flatten(),
                pool_p75:        row.try_get::<Option<f64>>("", "pool_p75").ok().flatten(),
                pool_p90:        row.try_get::<Option<f64>>("", "pool_p90").ok().flatten(),
                pool_max:        row.try_get::<Option<f64>>("", "pool_max").ok().flatten(),
                improving_count: row.try_get("", "improving_count").unwrap_or(0),
                declining_count: row.try_get("", "declining_count").unwrap_or(0),
            });
        }

        Ok(PortfolioStats {
            template_id,
            tenant_id,
            total_scorecards,
            refreshed_at,
            dimensions,
        })
    }

    // ── Leaderboard ───────────────────────────────────────────────────────────

    /// Returns the top N scored entities for a template, ranked by composite score.
    ///
    /// Also includes each entity's percentile rank derived from the materialized
    /// view (number of scorecards with composite < this / total * 100).
    ///
    /// Powers: `GET /api/templates/:id/leaderboard?limit=25`
    pub async fn leaderboard(
        db:          &DatabaseConnection,
        template_id: Uuid,
        tenant_id:   Uuid,
        limit:       i64,
    ) -> Result<Vec<LeaderboardEntry>> {
        let backend = db.get_database_backend();
        let limit = limit.clamp(1, 100);

        // Joins atlas_scorecards with the max(cohort_size) from the MV to
        // compute percentile rank inline without a second query.
        let rows = db.query_all(Statement::from_sql_and_values(
            backend,
            "SELECT
               sc.id                   AS scorecard_id,
               sc.subject_id,
               sc.subject_type,
               sc.composite_score,
               sc.confidence_level,
               sc.trend_direction,
               -- Percentile rank: fraction of scorecards with lower composite * 100
               ROUND(
                 100.0 * (
                   COUNT(*) FILTER (
                     WHERE sc2.composite_score < sc.composite_score
                   ) OVER ()::numeric
                 ) / NULLIF(COUNT(*) OVER (), 1),
                 1
               )                       AS percentile_rank,
               RANK() OVER (
                 ORDER BY sc.composite_score DESC NULLS LAST
               )                       AS rank
             FROM atlas_scorecards sc
             -- Self-join to get the full ordered set for window functions
             JOIN atlas_scorecards sc2
               ON sc2.template_id = sc.template_id
              AND sc2.tenant_id   = sc.tenant_id
              AND sc2.deleted_at  IS NULL
              AND sc2.composite_score IS NOT NULL
             WHERE sc.template_id = $1
               AND sc.tenant_id   = $2
               AND sc.deleted_at  IS NULL
               AND sc.composite_score IS NOT NULL
             GROUP BY
               sc.id, sc.subject_id, sc.subject_type,
               sc.composite_score, sc.confidence_level, sc.trend_direction
             ORDER BY sc.composite_score DESC NULLS LAST
             LIMIT $3",
            vec![
                sea_orm::Value::Uuid(Some(Box::new(template_id))),
                sea_orm::Value::Uuid(Some(Box::new(tenant_id))),
                sea_orm::Value::BigInt(Some(limit)),
            ],
        ))
        .await?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in &rows {
            entries.push(LeaderboardEntry {
                rank:             row.try_get("", "rank")?,
                scorecard_id:     row.try_get("", "scorecard_id")?,
                subject_id:       row.try_get("", "subject_id")?,
                subject_type:     row.try_get("", "subject_type")?,
                composite_score:  row.try_get::<Option<f64>>("", "composite_score").ok().flatten(),
                confidence_level: row.try_get("", "confidence_level").unwrap_or_default(),
                percentile_rank:  row.try_get::<Option<f64>>("", "percentile_rank").ok().flatten(),
                trend_direction:  row.try_get::<Option<String>>("", "trend_direction").ok().flatten(),
            });
        }

        Ok(entries)
    }

    // ── Recent Anomalies ──────────────────────────────────────────────────────

    /// Returns recent anomaly alerts (is_anomaly=true) across all scorecards
    /// for a template within the last 90 days.
    ///
    /// Reads from `v_scorecard_recent_anomalies` (live view — always current).
    ///
    /// Powers: `GET /api/templates/:id/anomalies`
    /// Also used by the Platform Event publisher to batch anomaly signals.
    pub async fn recent_anomalies(
        db:          &DatabaseConnection,
        template_id: Uuid,
        tenant_id:   Uuid,
        limit:       i64,
    ) -> Result<Vec<AnomalyAlert>> {
        let backend = db.get_database_backend();
        let limit = limit.clamp(1, 500);

        let rows = db.query_all(Statement::from_sql_and_values(
            backend,
            "SELECT
               scorecard_id,
               dimension_id,
               dimension_slug,
               dimension_name,
               period_start,
               mean_score,
               z_score,
               anomaly_direction
             FROM v_scorecard_recent_anomalies
             WHERE template_id = $1
               AND tenant_id   = $2
             ORDER BY period_start DESC, ABS(z_score) DESC NULLS LAST
             LIMIT $3",
            vec![
                sea_orm::Value::Uuid(Some(Box::new(template_id))),
                sea_orm::Value::Uuid(Some(Box::new(tenant_id))),
                sea_orm::Value::BigInt(Some(limit)),
            ],
        ))
        .await?;

        let mut alerts = Vec::with_capacity(rows.len());
        for row in &rows {
            alerts.push(AnomalyAlert {
                scorecard_id:      row.try_get("", "scorecard_id")?,
                dimension_id:      row.try_get("", "dimension_id")?,
                dimension_slug:    row.try_get("", "dimension_slug")?,
                dimension_name:    row.try_get("", "dimension_name")?,
                period_start:      row.try_get("", "period_start")?,
                mean_score:        row.try_get::<Option<f64>>("", "mean_score").ok().flatten(),
                z_score:           row.try_get::<Option<f64>>("", "z_score").ok().flatten(),
                anomaly_direction: row.try_get::<Option<String>>("", "anomaly_direction").ok().flatten(),
            });
        }

        Ok(alerts)
    }

    // ── Materialized View Refresh ─────────────────────────────────────────────

    /// Refresh `mv_scorecard_portfolio_analytics` concurrently.
    ///
    /// Uses `REFRESH MATERIALIZED VIEW CONCURRENTLY` so readers are never
    /// blocked. Takes a write lock on the unique index only for the swap,
    /// not the full computation.
    ///
    /// Called by:
    ///   1. Background worker (outbox_jobs, every 4 hours)
    ///   2. POST /api/templates/:id/analytics/refresh (admin only, on-demand)
    ///
    /// Returns the duration of the refresh for observability logging.
    pub async fn refresh_portfolio_view(
        db: &DatabaseConnection,
    ) -> Result<std::time::Duration> {
        let backend = db.get_database_backend();
        let started = std::time::Instant::now();

        db.execute(Statement::from_string(
            backend,
            "REFRESH MATERIALIZED VIEW CONCURRENTLY mv_scorecard_portfolio_analytics;".to_owned(),
        ))
        .await?;

        Ok(started.elapsed())
    }

    // ── BYOC Peer Pool Snapshot (Phase 5) ─────────────────────────────────────

    /// Builds the peer pool snapshot for a specific dimension, used in BYOC
    /// `ComputeRequest` payloads.
    ///
    /// Contains aggregate distribution stats only — no raw scorecard records.
    /// Safe to include in requests sent to client infrastructure.
    ///
    /// Returns `None` if no portfolio data exists yet for this dimension
    /// (view not yet refreshed or dimension has no data).
    pub async fn peer_pool_snapshot(
        db:           &DatabaseConnection,
        template_id:  Uuid,
        tenant_id:    Uuid,
        dimension_id: Uuid,
    ) -> Result<Option<PeerPoolSnapshot>> {
        let backend = db.get_database_backend();

        let rows = db.query_all(Statement::from_sql_and_values(
            backend,
            "SELECT
               cohort_size, pool_mean, pool_std_dev,
               pool_p25, pool_p50, pool_p75, pool_p90
             FROM mv_scorecard_portfolio_analytics
             WHERE template_id  = $1
               AND tenant_id    = $2
               AND dimension_id = $3
             LIMIT 1",
            vec![
                sea_orm::Value::Uuid(Some(Box::new(template_id))),
                sea_orm::Value::Uuid(Some(Box::new(tenant_id))),
                sea_orm::Value::Uuid(Some(Box::new(dimension_id))),
            ],
        ))
        .await?;

        let Some(row) = rows.first() else { return Ok(None) };

        Ok(Some(PeerPoolSnapshot {
            cohort_size:  row.try_get("", "cohort_size").unwrap_or(0),
            pool_mean:    row.try_get::<Option<f64>>("", "pool_mean").ok().flatten(),
            pool_std_dev: row.try_get::<Option<f64>>("", "pool_std_dev").ok().flatten(),
            pool_p25:     row.try_get::<Option<f64>>("", "pool_p25").ok().flatten(),
            pool_p50:     row.try_get::<Option<f64>>("", "pool_p50").ok().flatten(),
            pool_p75:     row.try_get::<Option<f64>>("", "pool_p75").ok().flatten(),
            pool_p90:     row.try_get::<Option<f64>>("", "pool_p90").ok().flatten(),
        }))
    }

    // ── Composite Refresh (convenience) ──────────────────────────────────────

    /// Full portfolio refresh: refreshes the MV, then triggers percentile rank
    /// recomputation for all scorecards in the given template.
    ///
    /// Called by the 4-hour background worker after the MV refresh completes.
    /// The rank recomputation is lightweight — reads the freshly-refreshed MV
    /// and writes to atlas_scorecard_dimension_aggregates.
    pub async fn refresh_and_rerank(
        db:          &DatabaseConnection,
        template_id: Uuid,
        tenant_id:   Uuid,
    ) -> Result<()> {
        // 1. Refresh the materialized view
        let duration = Self::refresh_portfolio_view(db).await?;
        tracing::info!(
            template_id = %template_id,
            tenant_id   = %tenant_id,
            duration_ms = duration.as_millis(),
            "mv_scorecard_portfolio_analytics refreshed"
        );

        // 2. Update percentile ranks for all scorecards in this template/tenant
        //    using the freshly-refreshed pool data.
        let backend = db.get_database_backend();

        // Batch-update percentile_rank and percentile_band on all aggregates
        // by joining against the MV for each dimension's pool stats.
        db.execute(Statement::from_sql_and_values(
            backend,
            "UPDATE atlas_scorecard_dimension_aggregates agg
             SET
               percentile_rank        = ROUND(
                 100.0 * (
                   SELECT COUNT(*)
                   FROM   atlas_scorecard_dimension_aggregates agg2
                   WHERE  agg2.dimension_id  = agg.dimension_id
                     AND  agg2.weighted_mean_score < agg.weighted_mean_score
                     AND  agg2.scorecard_id IN (
                       SELECT id FROM atlas_scorecards
                       WHERE template_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                     )
                 ) / NULLIF(mv.cohort_size - 1, 0),
                 1
               ),
               percentile_cohort_size = mv.cohort_size,
               percentile_band        = CASE
                 WHEN ROUND(
                   100.0 * (
                     SELECT COUNT(*)
                     FROM   atlas_scorecard_dimension_aggregates agg3
                     WHERE  agg3.dimension_id  = agg.dimension_id
                       AND  agg3.weighted_mean_score < agg.weighted_mean_score
                       AND  agg3.scorecard_id IN (
                         SELECT id FROM atlas_scorecards
                         WHERE template_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                       )
                   ) / NULLIF(mv.cohort_size - 1, 0),
                   1
                 ) >= 90 THEN 'top_10'
                 WHEN ROUND(
                   100.0 * (
                     SELECT COUNT(*)
                     FROM   atlas_scorecard_dimension_aggregates agg4
                     WHERE  agg4.dimension_id  = agg.dimension_id
                       AND  agg4.weighted_mean_score < agg.weighted_mean_score
                       AND  agg4.scorecard_id IN (
                         SELECT id FROM atlas_scorecards
                         WHERE template_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                       )
                   ) / NULLIF(mv.cohort_size - 1, 0),
                   1
                 ) >= 75 THEN 'top_quartile'
                 WHEN ROUND(
                   100.0 * (
                     SELECT COUNT(*)
                     FROM   atlas_scorecard_dimension_aggregates agg5
                     WHERE  agg5.dimension_id  = agg.dimension_id
                       AND  agg5.weighted_mean_score < agg.weighted_mean_score
                       AND  agg5.scorecard_id IN (
                         SELECT id FROM atlas_scorecards
                         WHERE template_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                       )
                   ) / NULLIF(mv.cohort_size - 1, 0),
                   1
                 ) >= 50 THEN 'median'
                 ELSE 'bottom_quartile'
               END,
               updated_at             = NOW()
             FROM mv_scorecard_portfolio_analytics mv
             WHERE agg.dimension_id = mv.dimension_id
               AND mv.template_id   = $1
               AND mv.tenant_id     = $2
               AND agg.scorecard_id IN (
                 SELECT id FROM atlas_scorecards
                 WHERE template_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
               )
               AND agg.weighted_mean_score IS NOT NULL",
            vec![
                sea_orm::Value::Uuid(Some(Box::new(template_id))),
                sea_orm::Value::Uuid(Some(Box::new(tenant_id))),
            ],
        ))
        .await?;

        tracing::info!(
            template_id = %template_id,
            tenant_id   = %tenant_id,
            "Percentile ranks batch-updated from refreshed portfolio MV"
        );

        Ok(())
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    /// These tests require a live DB (pgvector enabled) and are tagged
    /// `#[ignore]` so they don't run in CI without the `--include-ignored` flag.
    ///
    /// Run with:
    ///   cargo test portfolio_analytics -- --include-ignored
    ///
    /// Smoke tests for the response shape (not values) when the view is empty.

    #[tokio::test]
    #[ignore = "requires live postgres with mv_scorecard_portfolio_analytics"]
    async fn portfolio_stats_returns_empty_when_no_data() {
        // Setup: connect to test DB, call portfolio_stats with a fresh template_id
        // Assert: returns Ok(PortfolioStats { dimensions: vec![], total_scorecards: 0 })
        // (not Err — an empty portfolio is a valid state)
        todo!("integration test — requires test DB fixture")
    }

    #[tokio::test]
    #[ignore = "requires live postgres with mv_scorecard_portfolio_analytics"]
    async fn leaderboard_clamps_limit_to_100() {
        // Assert: requesting limit=9999 returns at most 100 entries
        todo!("integration test — requires test DB fixture")
    }

    #[tokio::test]
    #[ignore = "requires live postgres with v_scorecard_recent_anomalies"]
    async fn recent_anomalies_returns_sorted_by_period_desc() {
        // Assert: most recent period first, then by |z_score| desc
        todo!("integration test — requires test DB fixture")
    }
}
