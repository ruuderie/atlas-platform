use sea_orm_migration::prelude::*;

/// Add `employer_user_id`, `accepted_by_user_id`, and `accepted_at` to
/// `atlas_invite_codes`.
///
/// # employer_user_id
/// When a landlord creates a `property_manager` invite code, this field is
/// stamped with the landlord's user_id.  The accept handler uses this to:
///   - Scope the G-32 role (atlas_user_app_roles.client_account_id = employer_user_id)
///   - Create a G-11 contract (atlas_contracts) linking PM ↔ landlord
///
/// This implements the "Landlord-as-Admin / Live-in PM" scenario: the landlord
/// generates the invite, the PM accepts, and the platform automatically wires
/// up the access relationship and management agreement.
///
/// # accepted_by_user_id / accepted_at
/// Audit columns: who accepted the invite and when.  Written atomically in the
/// accept handler so there is a clear record even if the user later revokes.
///
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE atlas_invite_codes
                    ADD COLUMN IF NOT EXISTS employer_user_id      UUID
                        REFERENCES "user"(id) ON DELETE SET NULL,
                    ADD COLUMN IF NOT EXISTS accepted_by_user_id   UUID
                        REFERENCES "user"(id) ON DELETE SET NULL,
                    ADD COLUMN IF NOT EXISTS accepted_at            TIMESTAMPTZ;

                   -- Index for "show all PM codes sent by this landlord"
                   CREATE INDEX IF NOT EXISTS idx_invite_codes_employer
                       ON atlas_invite_codes(employer_user_id)
                       WHERE employer_user_id IS NOT NULL;

                   -- Index for "which invite did this user accept"
                   CREATE INDEX IF NOT EXISTS idx_invite_codes_accepted_by
                       ON atlas_invite_codes(accepted_by_user_id)
                       WHERE accepted_by_user_id IS NOT NULL;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"DROP INDEX IF EXISTS idx_invite_codes_employer;
                   DROP INDEX IF EXISTS idx_invite_codes_accepted_by;
                   ALTER TABLE atlas_invite_codes
                       DROP COLUMN IF EXISTS employer_user_id,
                       DROP COLUMN IF EXISTS accepted_by_user_id,
                       DROP COLUMN IF EXISTS accepted_at;
                "#,
            )
            .await?;

        Ok(())
    }
}
