#![allow(dead_code)]

//! G-27 Phase 1b: `atlas_scorecard_template_deployments`
//!
//! Controls which scorecard templates an app instance may list/use.
//! Contract: `docs/contracts/g27_scorecard_platform.md` §4 Deployment.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS atlas_scorecard_template_deployments (
                  id UUID PRIMARY KEY,
                  template_id UUID NOT NULL REFERENCES atlas_scorecard_templates(id) ON DELETE CASCADE,
                  app_instance_id UUID NOT NULL REFERENCES app_instances(id) ON DELETE CASCADE,
                  tenant_id UUID NOT NULL,
                  is_enabled BOOLEAN NOT NULL DEFAULT true,
                  trigger_event VARCHAR(40) NOT NULL DEFAULT 'manual',
                  trigger_context_entity_type VARCHAR(64) NULL,
                  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                  UNIQUE (template_id, app_instance_id)
                );
                CREATE INDEX IF NOT EXISTS idx_scorecard_deployments_instance
                  ON atlas_scorecard_template_deployments (app_instance_id, is_enabled);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_scorecard_deployments_instance;
                 DROP TABLE IF EXISTS atlas_scorecard_template_deployments;",
            )
            .await?;

        Ok(())
    }
}
