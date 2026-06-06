use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// G-27 Data Science Upgrade — Phase 1: Cold-Start & Confidence Weighting
///
/// Addresses the three gaps in `g27_data_science_upgrade_plan.md`:
///
/// ## Gap 1 — Cold-Start Problem
/// Adds `cold_start_strategy` and `cold_start_saturation_threshold` to
/// `atlas_scorecard_templates`. When a scorecard has fewer entries than
/// `min_entries_to_publish`, the current behavior is to suppress the score
/// entirely. With `cold_start_strategy = 'prior'`, the engine shows the
/// dimension's `global_reference_value` as a Bayesian prior, clearly labelled
/// as an estimate, until real data arrives.
///
/// ## Gap 1 (dim-level) — Bayesian Prior Weight
/// Adds `bayesian_prior_weight` to `atlas_scorecard_dimensions`.
/// When non-null, the aggregation service applies shrinkage:
///   `shrunk_mean = (prior_weight × global_ref + Σscores) / (prior_weight + n)`
/// This prevents small-n averages from displaying with false confidence.
/// The weight is denominated in "equivalent prior observations" — a weight of 5
/// means the prior has the same influence as 5 real entries.
///
/// ## Improvement 2 — Confidence-Weighted Composite
/// Adds `cold_start_saturation_threshold` (template-level). During composite
/// computation, each dimension's contribution is scaled by
///   `confidence_weight = min(contributor_count / saturation_threshold, 1.0)`
/// This means sparse dimensions pull the composite down less than well-rated ones.
/// The composite naturally converges to the true weighted mean as data accumulates.
///
/// ## scoring_method enum extension (Bug Fix #3)
/// Adds a COMMENT documenting 'percentile_rank' as a valid value of
/// `scoring_method` — the service already has a stub for this path.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── Templates: cold_start_strategy ─────────────────────────────────────
        //
        // 'suppress'  → show nothing until min_entries_to_publish met (current behavior, default)
        // 'prior'     → show global_reference_value as Bayesian prior with 'Estimated' label
        // 'category'  → show category-average as display prior (future; treated as 'prior' for now)
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD COLUMN IF NOT EXISTS cold_start_strategy VARCHAR(20) NOT NULL DEFAULT 'suppress';"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD CONSTRAINT chk_cold_start_strategy \
             CHECK (cold_start_strategy IN ('suppress', 'prior', 'category')) \
             NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_templates.cold_start_strategy IS \
             'Controls what to show when entry count < min_entries_to_publish. \
              suppress: hide score entirely (current default). \
              prior: show global_reference_value as a Bayesian prior estimate. \
              category: show category-pool average as estimate (planned, maps to prior until impl).';"
                .to_owned(),
        ))
        .await?;

        // ── Templates: cold_start_saturation_threshold ──────────────────────────
        //
        // Number of distinct contributors at which the confidence_weight reaches 1.0
        // (fully confident, no shrinkage). Below this threshold, confidence_weight =
        // min(contributor_count / saturation_threshold, 1.0).
        // Default 50 matches the G-27 spec recommendation.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD COLUMN IF NOT EXISTS cold_start_saturation_threshold INT NOT NULL DEFAULT 50;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD CONSTRAINT chk_saturation_threshold_positive \
             CHECK (cold_start_saturation_threshold > 0) NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_templates.cold_start_saturation_threshold IS \
             'Contributor count at which confidence_weight = 1.0 (fully saturated). \
              confidence_weight = MIN(contributor_count / threshold, 1.0). \
              Applied in composite calculation: each dimension is multiplied by its \
              confidence_weight before summing. Default: 50.';"
                .to_owned(),
        ))
        .await?;

        // ── Dimensions: bayesian_prior_weight ───────────────────────────────────
        //
        // NULL means disabled — no shrinkage applied (current behavior).
        // Non-null: shrunk_mean = (w * ref + Σscores) / (w + n)
        // where ref = global_reference_value, n = entry count.
        // Denominated in equivalent prior observations.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             ADD COLUMN IF NOT EXISTS bayesian_prior_weight DECIMAL(6,2) DEFAULT NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             ADD CONSTRAINT chk_bayesian_prior_weight_positive \
             CHECK (bayesian_prior_weight IS NULL OR bayesian_prior_weight > 0) NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_dimensions.bayesian_prior_weight IS \
             'Equivalent prior observations for Bayesian shrinkage. NULL = disabled. \
              Non-null requires global_reference_value to be set on this dimension. \
              Formula: shrunk_mean = (weight * global_ref + sum(scores)) / (weight + count). \
              Recommended starting value: 5.0 (prior has influence of 5 real entries). \
              Converges to observed mean as n >> weight.';"
                .to_owned(),
        ))
        .await?;

        // ── scoring_method enum documentation ───────────────────────────────────
        // The 'percentile_rank' path already exists as a stub in scorecard_service.rs.
        // This comment ensures the DB and service are in sync on valid values.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_templates.scoring_method IS \
             'Composite scoring method. Valid values: \
              weighted_mean (default): weighted average of dimension means. \
              simple_mean: unweighted average. \
              percentile_rank: composite = percentile rank within tenant pool / 10.0 \
                (requires portfolio_analytics materialized view — Phase 3).';"
                .to_owned(),
        ))
        .await?;

        // ── Templates: default_bayesian_prior_weight (Decision 4) ───────────────
        //
        // Template-level fallback for the hierarchical Bayesian prior lookup:
        //   1. dim.bayesian_prior_weight       → use if set (dimension-specific)
        //   2. template.default_bayesian_prior_weight → use if set (template default)
        //   3. NULL                             → no shrinkage (current behavior)
        //
        // This lets admins enable Bayesian shrinkage for an entire template with
        // one field, while data scientists override individual dimensions as needed.
        // Ships as NULL (disabled) — zero behavior change for existing tenants.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD COLUMN IF NOT EXISTS default_bayesian_prior_weight DECIMAL(5,2) DEFAULT NULL;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD CONSTRAINT chk_default_prior_weight_positive \
             CHECK (default_bayesian_prior_weight IS NULL OR default_bayesian_prior_weight > 0) \
             NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_templates.default_bayesian_prior_weight IS \
             'Template-wide Bayesian prior weight fallback (Decision 4). \
              Applied when a dimension has bayesian_prior_weight = NULL. \
              Lookup: dim.bayesian_prior_weight → this field → disabled. \
              NULL = no template default (shrinkage disabled unless set per-dimension). \
              Recommended value when enabling: 5.0 (prior = 5 equivalent observations). \
              Overridden per-dimension by atlas_scorecard_dimensions.bayesian_prior_weight.';"
                .to_owned(),
        ))
        .await?;

        // ── Templates: calibration_minimum_entries (Decision 3) ─────────────────
        //
        // Per-template threshold controlling when contributor bias calibration is
        // applied in compute_numeric_aggregate(). Calibration is only applied
        // when a contributor's entry_count for this template meets this threshold.
        //
        // Org-wide threshold was rejected: templates have different data velocities
        // (high-volume marketplace templates may hit 100 in a week; enterprise deal
        // health templates may take 18 months). Per-template is correct.
        //
        // Default 100 preserves current behavior (calibration not yet activated).
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD COLUMN IF NOT EXISTS calibration_minimum_entries INT NOT NULL DEFAULT 100;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             ADD CONSTRAINT chk_calibration_minimum_entries_positive \
             CHECK (calibration_minimum_entries > 0) NOT VALID;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_templates.calibration_minimum_entries IS \
             'Per-template contributor calibration activation threshold (Decision 3). \
              Calibration is applied in compute_numeric_aggregate() only when a \
              contributor has >= this many entries for this template. \
              Below this threshold, bias_offset and scale_factor from \
              atlas_scorecard_contributor_calibration are NOT applied. \
              Different templates have different data velocities; per-template is correct. \
              Default: 100 (backward compatible — no calibration change for existing tenants).';"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Drop constraints first
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             DROP CONSTRAINT IF EXISTS chk_cold_start_strategy;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             DROP CONSTRAINT IF EXISTS chk_saturation_threshold_positive;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             DROP CONSTRAINT IF EXISTS chk_bayesian_prior_weight_positive;"
                .to_owned(),
        ))
        .await?;

        // Drop columns
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             DROP COLUMN IF EXISTS cold_start_strategy, \
             DROP COLUMN IF EXISTS cold_start_saturation_threshold;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             DROP COLUMN IF EXISTS bayesian_prior_weight;"
                .to_owned(),
        ))
        .await?;

        // Drop Decision 3 & 4 columns
        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             DROP CONSTRAINT IF EXISTS chk_default_prior_weight_positive, \
             DROP CONSTRAINT IF EXISTS chk_calibration_minimum_entries_positive;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_templates \
             DROP COLUMN IF EXISTS default_bayesian_prior_weight, \
             DROP COLUMN IF EXISTS calibration_minimum_entries;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }
}
