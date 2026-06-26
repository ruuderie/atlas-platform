use sea_orm_migration::prelude::*;

/// Platform Account Link — adds platform_account_id to atlas_app_deployment_config.
///
/// This nullable FK connects a client deployment to its CRM Account record in the
/// platform admin's own CRM (atlas_accounts). It enables the Clients page to
/// surface a "View Account" action without a full join at query time.
///
/// ## Why here, not on atlas_accounts?
///
/// The relationship reads "a deployment config *optionally* belongs to an account".
/// Putting the FK on the deployment config (the owned entity) follows standard
/// relational design — the entity that *has* the account link holds the FK.
///
/// ## Lifecycle
///
/// - NULL: deployment not yet linked to a CRM account (e.g. provisioned before
///         the CRM record was created, or an internal instance with no account).
/// - UUID: admin has linked the deployment to a specific atlas_accounts row.
///
/// The FK is intentionally SET NULL on delete so removing a CRM account does not
/// cascade-delete or block the operational deployment config.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_app_deployment_config
                     ADD COLUMN IF NOT EXISTS platform_account_id UUID
                         REFERENCES atlas_accounts(id)
                         ON DELETE SET NULL;

                 CREATE INDEX IF NOT EXISTS idx_app_deployment_config_platform_account
                     ON atlas_app_deployment_config (platform_account_id)
                     WHERE platform_account_id IS NOT NULL;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_app_deployment_config_platform_account;
                 ALTER TABLE atlas_app_deployment_config
                     DROP COLUMN IF EXISTS platform_account_id;",
            )
            .await?;
        Ok(())
    }
}
