#![allow(dead_code)]

//! G-27 Phase C: nullable `app_instance_id` on `atlas_rating_sessions`.
//!
//! Stamps which app instance opened a rating session for multi-app tenants.
//! Contract: `docs/contracts/g27_scorecard_platform.md` §4 Session.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_rating_sessions
                   ADD COLUMN IF NOT EXISTS app_instance_id UUID
                     REFERENCES app_instances(id) ON DELETE SET NULL;
                 CREATE INDEX IF NOT EXISTS idx_rating_sessions_tenant_instance_occurred
                   ON atlas_rating_sessions (tenant_id, app_instance_id, occurred_at DESC);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_rating_sessions_tenant_instance_occurred;
                 ALTER TABLE atlas_rating_sessions DROP COLUMN IF EXISTS app_instance_id;",
            )
            .await?;

        Ok(())
    }
}
