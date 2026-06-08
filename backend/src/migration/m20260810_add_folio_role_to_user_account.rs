use sea_orm_migration::prelude::*;

/// Adds `folio_role` VARCHAR(20) NOT NULL DEFAULT 'landlord' to `user_account`.
///
/// This column stores the PM-context role for Folio app routing and endpoint
/// authorization. Existing accounts default to 'landlord' — the primary PM
/// operator role — which is safe because all pre-existing accounts were created
/// by or for property managers.
///
/// The column is set explicitly during:
///   - Tenant invite / application approval flows
///   - Vendor onboarding / service provider creation
///   - Any future role-assignment endpoint
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE user_account \
                 ADD COLUMN IF NOT EXISTS folio_role VARCHAR(20) NOT NULL DEFAULT 'landlord'",
            )
            .await?;

        // Add check constraint so only valid roles can be written.
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN
                    ALTER TABLE user_account ADD CONSTRAINT chk_user_account_folio_role
                        CHECK (folio_role IN ('landlord','tenant','vendor'));
                 EXCEPTION WHEN duplicate_object THEN NULL;
                 END $$",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE user_account DROP CONSTRAINT IF EXISTS chk_user_account_folio_role; \
                 ALTER TABLE user_account DROP COLUMN IF EXISTS folio_role",
            )
            .await?;
        Ok(())
    }
}
