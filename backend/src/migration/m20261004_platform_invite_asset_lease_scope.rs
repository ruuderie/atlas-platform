use sea_orm_migration::prelude::*;

/// Add asset-level and lease-level scoping to `platform_invite`.
///
/// `asset_ids UUID[]` — for cohost/vendor/delegate invites: scope access to specific
///   atlas_assets rows. NULL = no asset restriction (full account access).
///
/// `lease_id UUID` — for tenant invites: automatically links the accepted user
///   to a specific lease record (`atlas_leases.tenant_user_id`).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE platform_invite
                    ADD COLUMN IF NOT EXISTS asset_ids  UUID[]  DEFAULT NULL,
                    ADD COLUMN IF NOT EXISTS lease_id   UUID    REFERENCES atlas_leases(id)  ON DELETE SET NULL;
                 CREATE INDEX IF NOT EXISTS idx_platform_invite_lease
                     ON platform_invite (lease_id)
                     WHERE lease_id IS NOT NULL;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_platform_invite_lease;
                 ALTER TABLE platform_invite
                     DROP COLUMN IF EXISTS lease_id,
                     DROP COLUMN IF EXISTS asset_ids;",
            )
            .await?;
        Ok(())
    }
}
