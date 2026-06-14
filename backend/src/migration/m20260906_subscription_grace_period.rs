use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_subscriptions
                     ADD COLUMN IF NOT EXISTS is_billing_exempt BOOLEAN NOT NULL DEFAULT false,
                     ADD COLUMN IF NOT EXISTS billing_exemption_reason TEXT,
                     ADD COLUMN IF NOT EXISTS grace_period_ends_at TIMESTAMPTZ;

                 CREATE INDEX IF NOT EXISTS idx_subscriptions_grace_period
                     ON atlas_subscriptions (tenant_id, grace_period_ends_at)
                     WHERE grace_period_ends_at IS NOT NULL;"
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_subscriptions_grace_period;
                 ALTER TABLE atlas_subscriptions
                     DROP COLUMN IF EXISTS is_billing_exempt,
                     DROP COLUMN IF EXISTS billing_exemption_reason,
                     DROP COLUMN IF EXISTS grace_period_ends_at;"
            )
            .await?;
        Ok(())
    }
}
