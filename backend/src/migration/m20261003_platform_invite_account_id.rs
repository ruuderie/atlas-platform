use sea_orm_migration::prelude::*;

/// Add `account_id` column to `platform_invite`.
///
/// When set, the magic-link verify handler will link the invited user to this
/// existing `atlas_accounts` row instead of creating a new account. This enables:
///
/// 1. Platform-admin can add a user to an **existing** workspace (account).
/// 2. `POST /api/folio/team/invite` sets this to the caller's account so the
///    invitee is automatically linked to the right workspace on verify.
///
/// NULL = create a new account on verify (existing behaviour preserved).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE platform_invite
                    ADD COLUMN IF NOT EXISTS account_id UUID
                        REFERENCES atlas_accounts(id) ON DELETE SET NULL;
                 CREATE INDEX IF NOT EXISTS idx_platform_invite_account
                     ON platform_invite (account_id)
                     WHERE account_id IS NOT NULL;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_platform_invite_account;
                 ALTER TABLE platform_invite DROP COLUMN IF EXISTS account_id;",
            )
            .await?;
        Ok(())
    }
}
