use sea_orm_migration::prelude::*;

/// G-32 cleanup: drops the `folio_role` column from `user_account` now that
/// all role assignments live in `atlas_user_app_roles`.
///
/// Safe to run after m20260813 (backfill) has been verified on live DBs.
/// The CHECK constraint is dropped first to avoid a dependency error.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE user_account
                     DROP CONSTRAINT IF EXISTS chk_user_account_folio_role;
                 ALTER TABLE user_account
                     DROP COLUMN IF EXISTS folio_role;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-add column and re-populate from atlas_user_app_roles
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE user_account
                     ADD COLUMN IF NOT EXISTS folio_role VARCHAR(20) NOT NULL DEFAULT 'landlord';
                 ALTER TABLE user_account ADD CONSTRAINT chk_user_account_folio_role
                     CHECK (folio_role IN ('landlord','tenant','vendor'));
                 UPDATE user_account ua
                 SET folio_role = COALESCE((
                     SELECT rp.role_slug
                     FROM atlas_user_app_roles uar
                     JOIN atlas_role_profiles rp ON uar.role_profile_id = rp.id
                     JOIN account a ON ua.account_id = a.id
                     WHERE uar.user_id   = ua.user_id
                       AND uar.tenant_id = a.tenant_id
                       AND uar.app_slug  = 'folio'
                       AND uar.is_active = true
                     LIMIT 1
                 ), 'landlord');",
            )
            .await?;
        Ok(())
    }
}
