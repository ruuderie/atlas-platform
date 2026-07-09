use sea_orm_migration::prelude::*;

/// Add `lease_type` to `atlas_leases`.
///
/// A user's FolioRole is always `Tenant` regardless of whether they are a
/// long-term renter or a short-term guest. The TYPE of tenancy is on the
/// lease, not the person — a person can hold both an LTR lease in one city
/// and be a short-term guest in another property.
///
/// The Tenant portal adapts based on the active lease's lease_type:
///   ltr → Full tenant portal (rent payments, maintenance, lease docs)
///   str → Guest portal view (reservation details, check-in, house rules)
///
/// Constraint: a lease can only be `str` if the parent asset has str_eligible = true.
/// This is enforced at the service layer (not as a DB constraint to avoid
/// cross-table constraint complexity).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE atlas_leases
                    ADD COLUMN IF NOT EXISTS lease_type VARCHAR(3)
                        NOT NULL DEFAULT 'ltr'
                        CHECK (lease_type IN ('ltr', 'str'));

                CREATE INDEX IF NOT EXISTS idx_leases_type
                    ON atlas_leases (lease_type)
                    WHERE lease_type = 'str';"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"DROP INDEX IF EXISTS idx_leases_type;
                ALTER TABLE atlas_leases DROP COLUMN IF EXISTS lease_type;"#,
            )
            .await?;
        Ok(())
    }
}
