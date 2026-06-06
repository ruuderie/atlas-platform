use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Seed: Register the G-27 Phase 3 portfolio analytics background job.
///
/// Adds a third G-27 background job per qualifying tenant:
///
///   3. `refresh_scorecard_portfolio` (every 4 hours / 14400 seconds)
///      Refreshes `mv_scorecard_portfolio_analytics` CONCURRENTLY (no reader locks),
///      then batch-updates `percentile_rank`, `percentile_band`, and
///      `percentile_cohort_size` on `atlas_scorecard_dimension_aggregates` for every
///      scorecard in the pool.
///
///      After refresh, this data is also the source for BYOC `peer_pool` snapshots
///      included in `ComputeRequest` payloads (Phase 5).
///
/// Idempotent — DO $$ block guards against duplicate insertions.
/// Sorts after the existing G-27 job seed (m20260712_ > m20260706_).
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
                      AND is_deleted   = false
                LOOP
                    -- 3. Portfolio analytics refresh + percentile re-rank (every 4 hours)
                    IF NOT EXISTS (
                        SELECT 1 FROM tenant_background_jobs
                        WHERE tenant_id = v_tenant_id
                          AND job_type = 'refresh_scorecard_portfolio'
                    ) THEN
                        INSERT INTO tenant_background_jobs
                            (id, tenant_id, job_type, config, interval_seconds, last_run, is_active)
                        VALUES (
                            gen_random_uuid(),
                            v_tenant_id,
                            'refresh_scorecard_portfolio',
                            '{
                                "description": "Refresh mv_scorecard_portfolio_analytics CONCURRENTLY + batch-update percentile ranks. Source of BYOC peer_pool snapshots (Phase 5).",
                                "refresh_mode": "concurrent",
                                "rerank_after_refresh": true
                            }'::jsonb,
                            14400,   -- 4 hours (14,400 seconds)
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
            WHERE job_type = 'refresh_scorecard_portfolio';
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
