use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Seed: Register the G-27 Phase 4 contributor calibration background job.
///
/// Adds a fourth G-27 background job per qualifying tenant:
///
///   4. `calibrate_scorecard_contributors` (every 7 days / 604800 seconds)
///      Computes per-contributor bias_offset and scale_factor for each dimension
///      of each template. Writes results to atlas_scorecard_contributor_calibration.
///
///      The calibration is applied in `compute_numeric_aggregate` on the next
///      `recompute_scorecard_aggregates` run. Contributors below
///      `template.calibration_minimum_entries` (default 100) are skipped.
///
///      A 7-day interval balances freshness vs. compute cost. For high-volume
///      templates (daily entries), consider reducing to 86400 (daily).
///
/// Idempotent — DO $$ block guards against duplicate insertions.
/// Sorts after the portfolio job seed (m20260713_ > m20260712_).
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
            BEGIN
                FOR v_tenant_id IN
                    SELECT DISTINCT tenant_id
                    FROM atlas_scorecard_templates
                    WHERE is_published = true
                LOOP
                    -- 4. Contributor calibration (every 7 days)
                    IF NOT EXISTS (
                        SELECT 1 FROM tenant_background_jobs
                        WHERE tenant_id = v_tenant_id
                          AND job_type = 'calibrate_scorecard_contributors'
                    ) THEN
                        INSERT INTO tenant_background_jobs
                            (id, tenant_id, job_type, config, interval_seconds, last_run, is_active)
                        VALUES (
                            gen_random_uuid(),
                            v_tenant_id,
                            'calibrate_scorecard_contributors',
                            '{
                                "description": "Weekly contributor bias calibration. Computes bias_offset + scale_factor per (contributor, template, dimension). Applied in compute_numeric_aggregate when entry_count >= calibration_minimum_entries.",
                                "algorithm": "mean_shift_scale",
                                "min_entries_gate": "template.calibration_minimum_entries"
                            }'::jsonb,
                            604800,  -- 7 days (604,800 seconds)
                            NULL,
                            true
                        );
                    END IF;
                END LOOP;
            END $$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DELETE FROM tenant_background_jobs
            WHERE job_type = 'calibrate_scorecard_contributors';
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
