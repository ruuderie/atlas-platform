use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// G-27 Data Science Upgrade — Phase 2: Vector Masks, Anomaly Detection, Percentile Ranks
///
/// ## Gap 3 — Masked Cosine Similarity (Dimension Vector v2)
/// The existing `dimension_vector` (JSONB float array) uses `0.0` as a sentinel
/// for "no data on this dimension". This collapses the vector space: two entities
/// that are both unrated on dimension 3 look different from two entities that are
/// both rated 1.0 on dimension 3, even though the similarity should be equal.
///
/// The fix: parallel `has_data_mask` (bool[]) alongside `dimension_vector_v2` (float4[]).
/// The Combinator now only computes cosine similarity across dimensions where BOTH
/// entities have `has_data_mask[i] = true`. If the overlap is < 30%, the result
/// is `None` (insufficient shared data — not comparable).
///
/// `dimension_vector_v2` stores normalized f32 values (0.0–weight range), same as before.
/// The old `dimension_vector` JSONB column is preserved for backward compatibility.
///
/// ## Improvement 1 — Percentile Ranks on Aggregates
/// Adds `percentile_rank`, `percentile_cohort_size`, and `percentile_band` to
/// `atlas_scorecard_dimension_aggregates`. These are computed post-aggregation
/// by `ScorecardService::compute_percentile_ranks()` using a window function
/// over all scorecards for the same (template_id, dimension_id, tenant_id).
///
/// ## Improvement 3 — Anomaly Detection on Time Series
/// Adds `z_score`, `is_anomaly`, and `anomaly_direction` to `atlas_scorecard_time_series`.
/// The z-score is computed against the trailing 6 periods (rolling window).
/// |z| > 2.0 → is_anomaly = true. Direction: 'spike' (z > 2) or 'drop' (z < -2).
///
/// ## Contributor Calibration Table (Gap 2)
/// Creates `atlas_scorecard_contributor_calibration` — stores per-contributor bias
/// offsets computed by the weekly calibration background job (Phase 4).
/// The table is created now so Phase 4 can activate it without a schema migration.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── atlas_scorecards: dimension_vector_v2 + has_data_mask ──────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecards \
             ADD COLUMN IF NOT EXISTS dimension_vector_v2  JSONB DEFAULT NULL, \
             ADD COLUMN IF NOT EXISTS has_data_mask        JSONB DEFAULT NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecards.dimension_vector_v2 IS \
             'Parallel float4 array to has_data_mask. Stores normalized dimension scores \
              (0.0 = scale_min contribution, weight = scale_max contribution). \
              Only populated for Rating/Absolute/Boolean dimensions. \
              Used by masked_cosine_similarity in find_similar (Gap 3 fix). \
              Replaces the legacy dimension_vector JSONB column for similarity computation.';"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecards.has_data_mask IS \
             'Parallel bool array to dimension_vector_v2. \
              true = this dimension has at least one verified entry (real data). \
              false = no entries yet; the vector value is the midpoint placeholder. \
              The Combinator requires MIN 30% overlap (both masks true) to compute \
              a meaningful similarity score.';"
                .to_owned(),
        ))
        .await?;

        // ── atlas_scorecard_dimension_aggregates: percentile fields ─────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimension_aggregates \
             ADD COLUMN IF NOT EXISTS percentile_rank         DECIMAL(5,2) DEFAULT NULL, \
             ADD COLUMN IF NOT EXISTS percentile_cohort_size  INT          DEFAULT NULL, \
             ADD COLUMN IF NOT EXISTS percentile_band         VARCHAR(20)  DEFAULT NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimension_aggregates \
             ADD CONSTRAINT chk_percentile_rank_range \
             CHECK (percentile_rank IS NULL OR (percentile_rank >= 0 AND percentile_rank <= 100)) \
             NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimension_aggregates \
             ADD CONSTRAINT chk_percentile_band_values \
             CHECK (percentile_band IS NULL OR \
                    percentile_band IN ('top_10', 'top_quartile', 'median', 'bottom_quartile')) \
             NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_dimension_aggregates.percentile_rank IS \
             'Percentile rank of this scorecard within the tenant pool for this dimension. \
              0-100. Computed by ScorecardService::compute_percentile_ranks() after aggregation. \
              NULL until computed (at least 2 scorecards needed for meaningful ranking).';"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_dimension_aggregates.percentile_band IS \
             'Categorical band from percentile_rank. \
              top_10: rank >= 90. top_quartile: rank >= 75. \
              median: rank >= 50. bottom_quartile: rank < 50.';"
                .to_owned(),
        ))
        .await?;

        // ── atlas_scorecard_time_series: anomaly detection fields ───────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_time_series \
             ADD COLUMN IF NOT EXISTS z_score            DECIMAL(6,3)  DEFAULT NULL, \
             ADD COLUMN IF NOT EXISTS is_anomaly         BOOLEAN       NOT NULL DEFAULT false, \
             ADD COLUMN IF NOT EXISTS anomaly_direction  VARCHAR(10)   DEFAULT NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_time_series \
             ADD CONSTRAINT chk_anomaly_direction_values \
             CHECK (anomaly_direction IS NULL OR anomaly_direction IN ('spike', 'drop')) \
             NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_time_series.z_score IS \
             'Standard deviations from the trailing 6-period rolling mean. \
              Computed by refresh_time_series_for_dimension. \
              NULL for the first 3 periods (insufficient trailing history). \
              |z| > 2.0 → is_anomaly = true.';"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_time_series.anomaly_direction IS \
             'Direction of the anomaly when is_anomaly = true. \
              spike: z_score > 2.0 (unusually high score for this period). \
              drop:  z_score < -2.0 (unusually low score for this period). \
              NULL when is_anomaly = false.';"
                .to_owned(),
        ))
        .await?;

        // ── atlas_scorecard_contributor_calibration (Gap 2, Phase 4 activation) ─
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE TABLE IF NOT EXISTS atlas_scorecard_contributor_calibration (
                id                   UUID         NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
                contributor_user_id  UUID         NOT NULL,
                template_id          UUID         NOT NULL
                    REFERENCES atlas_scorecard_templates(id) ON DELETE CASCADE,
                dimension_id         UUID         REFERENCES atlas_scorecard_dimensions(id) ON DELETE CASCADE,
                -- bias_offset: contributor's mean score deviation from ensemble mean for this dimension.
                -- Applied in compute_numeric_aggregate: calibrated_score = score - bias_offset.
                bias_offset          DECIMAL(6,3) NOT NULL DEFAULT 0.0,
                -- scale_factor: ratio of contributor std to ensemble std.
                -- Applied after bias offset: calibrated_score = (score - bias_offset) * scale_factor.
                -- 1.0 = no scaling.
                scale_factor         DECIMAL(6,3) NOT NULL DEFAULT 1.0,
                -- entry_count: number of entries used to compute this calibration.
                -- Calibration is only applied when entry_count >= minimum_threshold (100).
                entry_count          INT          NOT NULL DEFAULT 0,
                last_calibrated_at   TIMESTAMPTZ,
                created_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW()
            );"
                .to_owned(),
        ))
        .await?;

        // Unique index for dimension-level rows (dimension_id IS NOT NULL)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_contrib_calibration_dim_unique \
             ON atlas_scorecard_contributor_calibration \
             (contributor_user_id, template_id, dimension_id) \
             WHERE dimension_id IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Unique index for template-level rows (dimension_id IS NULL)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_contrib_calibration_tmpl_unique \
             ON atlas_scorecard_contributor_calibration \
             (contributor_user_id, template_id) \
             WHERE dimension_id IS NULL;"
                .to_owned(),
        ))
        .await?;


        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_contributor_calibration_template \
             ON atlas_scorecard_contributor_calibration (template_id, dimension_id);"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON TABLE atlas_scorecard_contributor_calibration IS \
             'Per-contributor bias correction coefficients. \
              Populated by ScorecardService::calibrate_contributor_bias() (weekly background job). \
              Applied in compute_numeric_aggregate ONLY when entry_count >= 100 (Phase 4). \
              bias_offset: additive correction (subtract from raw score). \
              scale_factor: multiplicative correction (applied after offset). \
              dimension_id NULL = template-level calibration (applies to all dims).';"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Drop constraints
        for (table, constraint) in [
            ("atlas_scorecard_dimension_aggregates", "chk_percentile_rank_range"),
            ("atlas_scorecard_dimension_aggregates", "chk_percentile_band_values"),
            ("atlas_scorecard_time_series", "chk_anomaly_direction_values"),
        ] {
            db.execute(sea_orm::Statement::from_string(
                backend,
                format!("ALTER TABLE {table} DROP CONSTRAINT IF EXISTS {constraint};"),
            ))
            .await?;
        }

        // Drop calibration table
        db.execute(sea_orm::Statement::from_string(
            backend,
            "DROP TABLE IF EXISTS atlas_scorecard_contributor_calibration;".to_owned(),
        ))
        .await?;

        // Drop new columns
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecards \
             DROP COLUMN IF EXISTS dimension_vector_v2, \
             DROP COLUMN IF EXISTS has_data_mask;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimension_aggregates \
             DROP COLUMN IF EXISTS percentile_rank, \
             DROP COLUMN IF EXISTS percentile_cohort_size, \
             DROP COLUMN IF EXISTS percentile_band;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_time_series \
             DROP COLUMN IF EXISTS z_score, \
             DROP COLUMN IF EXISTS is_anomaly, \
             DROP COLUMN IF EXISTS anomaly_direction;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }
}
