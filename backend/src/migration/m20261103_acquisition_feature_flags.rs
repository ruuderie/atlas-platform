//! Seed acquisition feature flags for DM tracking / controlled signup.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                INSERT INTO feature_flags (
                    id, key, description, is_enabled, has_global, global_rollout_pct,
                    is_plan_gated, plan_gate_tier, jira, owner, created_at
                ) VALUES
                (
                    gen_random_uuid(),
                    'acquisition.dm_tracking',
                    'Enable G-20 attribution capture on waitlist / LP / DM offer-code paths',
                    true, true, 100, false, NULL, NULL, 'platform', now()
                ),
                (
                    gen_random_uuid(),
                    'acquisition.open_signup',
                    'When false, organic traffic stays waitlist-only (controlled invite/DM signup)',
                    false, true, 100, false, NULL, NULL, 'platform', now()
                )
                ON CONFLICT (key) DO NOTHING;
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DELETE FROM feature_flags
                WHERE key IN ('acquisition.dm_tracking', 'acquisition.open_signup');
                "#,
            )
            .await?;
        Ok(())
    }
}
