//! Per-app-instance feature flag enablements (grant/deny overrides).

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
CREATE TABLE IF NOT EXISTS atlas_flag_instance_enablements (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  flag_key VARCHAR NOT NULL,
  app_instance_id UUID NOT NULL REFERENCES app_instances(id) ON DELETE CASCADE,
  effect VARCHAR NOT NULL CHECK (effect IN ('grant', 'deny')),
  rollout_pct INT NOT NULL DEFAULT 100 CHECK (rollout_pct BETWEEN 0 AND 100),
  updated_by UUID,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (flag_key, app_instance_id)
);

CREATE INDEX IF NOT EXISTS idx_flag_instance_enablements_instance
  ON atlas_flag_instance_enablements(app_instance_id);

CREATE INDEX IF NOT EXISTS idx_flag_instance_enablements_flag_key
  ON atlas_flag_instance_enablements(flag_key);
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
DROP TABLE IF EXISTS atlas_flag_instance_enablements;
"#,
            )
            .await?;
        Ok(())
    }
}
