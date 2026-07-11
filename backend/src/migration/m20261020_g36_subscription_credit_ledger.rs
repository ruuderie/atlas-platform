//! G-36: apply subscription_credit_days grants to an internal credit ledger.
//!
//! Stripe / G-04 billing application remains a future integration; this ledger is
//! the durable balance Folio can read and Stripe can consume later.

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
CREATE TABLE IF NOT EXISTS atlas_subscription_credit_ledger (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    grant_id        UUID NOT NULL UNIQUE REFERENCES atlas_program_reward_grants(id) ON DELETE CASCADE,
    days            NUMERIC(12, 2) NOT NULL CHECK (days > 0),
    note            TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_atlas_subscription_credit_ledger_user
    ON atlas_subscription_credit_ledger (user_id, created_at DESC);

-- Allow grants to move from granted → applied after ledger write.
ALTER TABLE atlas_program_reward_grants
    DROP CONSTRAINT IF EXISTS atlas_program_reward_grants_status_check;
ALTER TABLE atlas_program_reward_grants
    ADD CONSTRAINT atlas_program_reward_grants_status_check
    CHECK (status IN ('pending', 'granted', 'applied', 'revoked'));
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
DROP TABLE IF EXISTS atlas_subscription_credit_ledger;
ALTER TABLE atlas_program_reward_grants
    DROP CONSTRAINT IF EXISTS atlas_program_reward_grants_status_check;
ALTER TABLE atlas_program_reward_grants
    ADD CONSTRAINT atlas_program_reward_grants_status_check
    CHECK (status IN ('pending', 'granted', 'revoked'));
"#,
            )
            .await?;
        Ok(())
    }
}
