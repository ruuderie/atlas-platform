use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// G-27 gap fill: Add `is_inverted` to `atlas_scorecard_dimensions`.
///
/// When `is_inverted = true`, **lower score = better outcome**.
///
/// This affects three places in `ScorecardService::recompute_aggregates`:
///   1. `build_dimension_vector`: normalization uses `(max - score)` instead of `(score - min)`
///   2. `vs_global_label`: delta direction is inverted — below reference = "above"
///   3. `resolve_rating_tier`: matches on `max_score` keys instead of `min_score`
///
/// Examples of inverted dimensions:
///   - `timeline_slippage` (days late — lower = on time = better)
///   - `competition_risk` (1–10 — lower = less competition = better)
///   - `ramp_to_close` (months — lower = faster = better)
///   - `air_pollution` (μg/m³ — lower = cleaner = better)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute(sea_orm::Statement::from_string(
            backend,
            "ALTER TABLE atlas_scorecard_dimensions \
             ADD COLUMN IF NOT EXISTS is_inverted BOOLEAN NOT NULL DEFAULT false;"
                .to_owned(),
        ))
        .await?;

        db.execute(sea_orm::Statement::from_string(
            backend,
            "COMMENT ON COLUMN atlas_scorecard_dimensions.is_inverted IS \
             'When true: lower score = better outcome. \
              Inverts dimension_vector normalization, vs_global_label direction, \
              and benchmark tier resolution (uses max_score instead of min_score). \
              Examples: timeline_slippage, competition_risk, ramp_to_close, air_pollution.';"
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
            "ALTER TABLE atlas_scorecard_dimensions \
             DROP COLUMN IF EXISTS is_inverted;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }
}
