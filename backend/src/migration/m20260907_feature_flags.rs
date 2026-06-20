use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS feature_flags (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    key TEXT UNIQUE NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    is_enabled BOOLEAN NOT NULL DEFAULT true,
                    has_global BOOLEAN NOT NULL DEFAULT true,
                    global_rollout_pct INTEGER NOT NULL DEFAULT 0 CHECK (global_rollout_pct BETWEEN 0 AND 100),
                    is_plan_gated BOOLEAN NOT NULL DEFAULT false,
                    plan_gate_tier TEXT,
                    jira TEXT,
                    owner TEXT NOT NULL DEFAULT 'system',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_feature_flags_key ON feature_flags (key);
                CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON feature_flags (is_enabled);

                CREATE TABLE IF NOT EXISTS flag_overrides (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    flag_id UUID NOT NULL REFERENCES feature_flags(id) ON DELETE CASCADE,
                    tenant_id UUID NOT NULL,
                    override_type TEXT NOT NULL DEFAULT 'grant' CHECK (override_type IN ('grant', 'deny')),
                    rollout_pct INTEGER NOT NULL DEFAULT 100 CHECK (rollout_pct BETWEEN 0 AND 100),
                    reason TEXT NOT NULL DEFAULT '',
                    jira TEXT,
                    changed_by TEXT NOT NULL DEFAULT 'system',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                    UNIQUE (flag_id, tenant_id)
                );

                CREATE INDEX IF NOT EXISTS idx_flag_overrides_flag_id ON flag_overrides (flag_id);
                CREATE INDEX IF NOT EXISTS idx_flag_overrides_tenant_id ON flag_overrides (tenant_id);

                CREATE TABLE IF NOT EXISTS flag_audit_log (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    flag_id UUID NOT NULL REFERENCES feature_flags(id) ON DELETE CASCADE,
                    user_id TEXT NOT NULL DEFAULT 'system',
                    action TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_flag_audit_log_flag_id ON flag_audit_log (flag_id);
                CREATE INDEX IF NOT EXISTS idx_flag_audit_log_created ON flag_audit_log (created_at DESC);"
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS flag_audit_log;
                 DROP TABLE IF EXISTS flag_overrides;
                 DROP TABLE IF EXISTS feature_flags;"
            )
            .await?;
        Ok(())
    }
}
