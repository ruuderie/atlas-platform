use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Seed: Register G-27 background jobs for all tenants that have active scorecard templates.
///
/// Two jobs are registered per qualifying tenant:
///
///   1. `recompute_scorecard_aggregates` (every 5 minutes)
///      Scans atlas_scorecard_entries for rows that changed since the last run,
///      recomputes dimension_aggregates + poll_aggregates + composite_score +
///      confidence_level + dimension_vector for the affected scorecards.
///
///   2. `refresh_scorecard_time_series` (every 60 minutes)
///      Rebuilds monthly + quarterly trend buckets in atlas_scorecard_time_series,
///      computing mean_score, delta_from_prior, and trend_direction per dimension.
///
/// Both jobs are idempotent — duplicate inserts are protected by the
/// (tenant_id, job_type) uniqueness pattern in the DO $$ block.
///
/// If no tenants have active scorecard templates at migration time, no rows are
/// inserted. The jobs can be added later by re-running this migration's SQL.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Register both scorecard background jobs for every tenant that has at
        // least one active scorecard template. Tenants without templates skip silently
        // — jobs are registered on-demand when they first create a template.
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
                    -- 1. Aggregate recompute (every 5 minutes)
                    IF NOT EXISTS (
                        SELECT 1 FROM tenant_background_jobs
                        WHERE tenant_id = v_tenant_id
                          AND job_type = 'recompute_scorecard_aggregates'
                    ) THEN
                        INSERT INTO tenant_background_jobs
                            (id, tenant_id, job_type, config, interval_seconds, last_run, is_active)
                        VALUES (
                            gen_random_uuid(),
                            v_tenant_id,
                            'recompute_scorecard_aggregates',
                            '{"batch_size": 50}'::jsonb,
                            300,   -- 5 minutes
                            NULL,
                            true
                        );
                    END IF;

                    -- 2. Time-series refresh (every 60 minutes)
                    IF NOT EXISTS (
                        SELECT 1 FROM tenant_background_jobs
                        WHERE tenant_id = v_tenant_id
                          AND job_type = 'refresh_scorecard_time_series'
                    ) THEN
                        INSERT INTO tenant_background_jobs
                            (id, tenant_id, job_type, config, interval_seconds, last_run, is_active)
                        VALUES (
                            gen_random_uuid(),
                            v_tenant_id,
                            'refresh_scorecard_time_series',
                            '{"period_types": ["monthly", "quarterly"]}'::jsonb,
                            3600,  -- 60 minutes
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
        // Remove the G-27 background jobs only — preserves other job types.
        let sql = r#"
            DELETE FROM tenant_background_jobs
            WHERE job_type IN (
                'recompute_scorecard_aggregates',
                'refresh_scorecard_time_series'
            );
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
