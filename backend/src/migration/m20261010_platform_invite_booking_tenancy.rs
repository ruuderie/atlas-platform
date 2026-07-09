use sea_orm_migration::prelude::*;

/// Extend `platform_invite` for str_guest and tenant applicant workflows.
///
/// New columns:
///   booking_id     — links a str_guest invite to a pre-existing atlas_bookings row.
///                    If NULL, the guest selects their dates during onboarding wizard step 1.
///
///   asset_id       — links a str_guest or tenant-applicant invite to a specific property.
///                    For str_guest: required (which property to book).
///                    For tenant applicant: optional (applying for a unit or the portfolio).
///
///   tenancy_status — lifecycle stage for tenant role invites.
///                    'applicant' — filling application, not yet approved (no lease required)
///                    'pending'   — approved, completing pre-move-in onboarding (lease required)
///                    'active'    — live tenant with signed lease (lease required)
///                    Default: 'applicant' (safe for new tenant invites with no lease yet).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE platform_invite
                    ADD COLUMN IF NOT EXISTS booking_id      UUID
                        REFERENCES atlas_bookings(id) ON DELETE SET NULL,
                    ADD COLUMN IF NOT EXISTS asset_id        UUID
                        REFERENCES atlas_assets(id)   ON DELETE SET NULL,
                    ADD COLUMN IF NOT EXISTS tenancy_status  VARCHAR(20)
                        DEFAULT 'applicant'
                        CHECK (tenancy_status IN ('applicant', 'pending', 'active'));

                CREATE INDEX IF NOT EXISTS idx_platform_invite_booking_id
                    ON platform_invite (booking_id)
                    WHERE booking_id IS NOT NULL;"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"DROP INDEX IF EXISTS idx_platform_invite_booking_id;
                ALTER TABLE platform_invite
                    DROP COLUMN IF EXISTS tenancy_status,
                    DROP COLUMN IF EXISTS asset_id,
                    DROP COLUMN IF EXISTS booking_id;"#,
            )
            .await?;
        Ok(())
    }
}
