use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// G-27 Data Science Upgrade — Phase 3: Portfolio Analytics
///
/// Creates `mv_scorecard_portfolio_analytics` — a materialized view that
/// aggregates per-template, per-tenant, per-dimension score distributions
/// across all scorecards. Powers:
///
/// ## Improvement 1 — Percentile Rank (full pool)
/// `compute_percentile_ranks()` in `scorecard_service.rs` queries this view
/// instead of iterating all scorecards inline. Index on (template_id, tenant_id,
/// dimension_id) makes the rank lookup sub-millisecond even at 100k scorecards.
///
/// ## New: Portfolio Analytics API endpoints
/// `scorecard_analytics_service.rs` reads this view for:
///   - GET /api/templates/:id/analytics   → portfolio_stats() — distribution shape
///   - GET /api/templates/:id/leaderboard → leaderboard()    — ranked entity list
///   - GET /api/templates/:id/anomalies   → recent is_anomaly=true across tenant
///
/// ## Refresh strategy
/// `REFRESH MATERIALIZED VIEW CONCURRENTLY mv_scorecard_portfolio_analytics`
/// is called by the background worker every 4 hours (registered in outbox_jobs).
/// CONCURRENTLY requires at least one UNIQUE index on the view — added below.
///
/// ## BYOC peer pool snapshot
/// This view is the source for the `peer_pool` field in BYOC ComputeRequest.
/// Atlas queries this view, extracts aggregate stats (p25, p50, p75, std_dev),
/// and includes them in the request payload. Raw records are never sent.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── Step 1: Create the materialized view ───────────────────────────
        //
        // Groups all verified, non-null entries by (template_id, tenant_id,
        // dimension_id) and computes distribution statistics across all
        // scorecards in that pool.
        //
        // Columns:
        //   template_id          UUID    — which template
        //   tenant_id            UUID    — tenant isolation (never cross-tenant)
        //   dimension_id         UUID    — which dimension
        //   dimension_slug       TEXT    — for human-readable API responses
        //   cohort_size          BIGINT  — distinct scorecards with ≥1 valid entry
        //   pool_mean            FLOAT8  — mean of all weighted_mean_score values
        //   pool_std_dev         FLOAT8  — population std dev (for z-score reference)
        //   pool_min             FLOAT8
        //   pool_p25             FLOAT8  — 25th percentile (PERCENTILE_CONT)
        //   pool_p50             FLOAT8  — median
        //   pool_p75             FLOAT8
        //   pool_p90             FLOAT8
        //   pool_max             FLOAT8
        //   improving_count      BIGINT  — scorecards with trend_direction='improving'
        //   declining_count      BIGINT  — scorecards with trend_direction='declining'
        //   refreshed_at         TIMESTAMPTZ — set to NOW() on each refresh
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE MATERIALIZED VIEW IF NOT EXISTS mv_scorecard_portfolio_analytics AS
             SELECT
               t.id                                       AS template_id,
               sc.tenant_id                               AS tenant_id,
               d.id                                       AS dimension_id,
               d.slug                                     AS dimension_slug,
               d.name                                     AS dimension_name,
               COUNT(DISTINCT agg.scorecard_id)           AS cohort_size,
               AVG(agg.weighted_mean_score)               AS pool_mean,
               STDDEV_POP(agg.weighted_mean_score)        AS pool_std_dev,
               MIN(agg.weighted_mean_score)               AS pool_min,
               PERCENTILE_CONT(0.25) WITHIN GROUP (
                   ORDER BY agg.weighted_mean_score
               )                                          AS pool_p25,
               PERCENTILE_CONT(0.50) WITHIN GROUP (
                   ORDER BY agg.weighted_mean_score
               )                                          AS pool_p50,
               PERCENTILE_CONT(0.75) WITHIN GROUP (
                   ORDER BY agg.weighted_mean_score
               )                                          AS pool_p75,
               PERCENTILE_CONT(0.90) WITHIN GROUP (
                   ORDER BY agg.weighted_mean_score
               )                                          AS pool_p90,
               MAX(agg.weighted_mean_score)               AS pool_max,
               COUNT(DISTINCT CASE
                   WHEN ts.trend_direction = 'improving' THEN agg.scorecard_id
               END)                                       AS improving_count,
               COUNT(DISTINCT CASE
                   WHEN ts.trend_direction = 'declining' THEN agg.scorecard_id
               END)                                       AS declining_count,
               NOW()                                      AS refreshed_at
             FROM atlas_scorecard_dimension_aggregates   agg
             JOIN atlas_scorecards                       sc  ON sc.id = agg.scorecard_id
             JOIN atlas_scorecard_dimensions             d   ON d.id = agg.dimension_id
             JOIN atlas_scorecard_templates              t   ON t.id = sc.template_id
             -- Left join to the most recent time series period for trend_direction.
             -- Must appear before WHERE — all JOINs precede the filter clause in SQL.
             LEFT JOIN LATERAL (
               SELECT ts2.trend_direction
               FROM   atlas_scorecard_time_series ts2
               WHERE  ts2.scorecard_id  = agg.scorecard_id
                 AND  ts2.dimension_id  = agg.dimension_id
               ORDER BY ts2.period_start DESC
               LIMIT 1
             ) ts ON TRUE
             -- Only include dimensions with valid computed data
             WHERE agg.weighted_mean_score IS NOT NULL
               AND d.is_active = TRUE
               AND sc.deleted_at IS NULL
             GROUP BY t.id, sc.tenant_id, d.id, d.slug, d.name
             WITH NO DATA;"
                .to_owned(),
        ))
        .await?;

        // ── Step 2: Unique index — required for REFRESH CONCURRENTLY ───────
        //
        // The unique constraint on (template_id, tenant_id, dimension_id)
        // mirrors the natural key of this aggregation. CONCURRENTLY refresh
        // holds no locks on readers, making it safe during business hours.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE UNIQUE INDEX IF NOT EXISTS uq_mv_portfolio_analytics_key
             ON mv_scorecard_portfolio_analytics (template_id, tenant_id, dimension_id);"
                .to_owned(),
        ))
        .await?;

        // ── Step 3: Tenant isolation index ─────────────────────────────────
        //
        // All API reads filter by (tenant_id, template_id). This index ensures
        // portfolio_stats() and leaderboard() queries are tenant-isolated and
        // sub-ms even at large scale.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_mv_portfolio_analytics_tenant
             ON mv_scorecard_portfolio_analytics (tenant_id, template_id);"
                .to_owned(),
        ))
        .await?;

        // ── Step 4: Initial population ──────────────────────────────────────
        //
        // Populate immediately so the view is non-empty after migration.
        // Subsequent refreshes are handled by the background worker.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "REFRESH MATERIALIZED VIEW mv_scorecard_portfolio_analytics;"
                .to_owned(),
        ))
        .await?;

        // ── Step 5: Anomaly leaderboard view ───────────────────────────────
        //
        // Lightweight view (not materialized) over atlas_scorecard_time_series
        // filtered to is_anomaly = true in the last 90 days. Used by
        // GET /api/templates/:id/anomalies — no refresh cycle needed, always live.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE OR REPLACE VIEW v_scorecard_recent_anomalies AS
             SELECT
               ts.id                 AS time_series_id,
               ts.scorecard_id,
               ts.dimension_id,
               sc.tenant_id,
               sc.template_id,
               d.slug                AS dimension_slug,
               d.name                AS dimension_name,
               ts.period_start,
               ts.period_end,
               ts.mean_score,
               ts.z_score,
               ts.is_anomaly,
               ts.anomaly_direction,
               ts.trend_direction,
               ts.created_at
             FROM   atlas_scorecard_time_series ts
             JOIN   atlas_scorecards            sc ON sc.id = ts.scorecard_id
             JOIN   atlas_scorecard_dimensions  d  ON d.id  = ts.dimension_id
             WHERE  ts.is_anomaly = TRUE
               AND  ts.period_start >= NOW() - INTERVAL '90 days'
               AND  sc.deleted_at IS NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON MATERIALIZED VIEW mv_scorecard_portfolio_analytics IS
             'G-27 Phase 3: Per-template per-tenant per-dimension score distribution. \
              Refreshed every 4 hours by background worker. \
              Powers percentile rank computation, portfolio panel, leaderboard API. \
              Source of peer_pool snapshot for BYOC ComputeRequest.';"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON VIEW v_scorecard_recent_anomalies IS
             'G-27 Phase 3: Live view of is_anomaly=true time series rows in the last 90 days. \
              No refresh cycle — always current. \
              Powers GET /api/templates/:id/anomalies endpoint.';"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP VIEW IF EXISTS v_scorecard_recent_anomalies;".to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP MATERIALIZED VIEW IF EXISTS mv_scorecard_portfolio_analytics;".to_owned(),
        ))
        .await?;

        Ok(())
    }
}
