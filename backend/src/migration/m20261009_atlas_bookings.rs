use sea_orm_migration::prelude::*;

/// Create `atlas_bookings` — the source of truth for STR reservations.
///
/// A booking is what links an `str_guest` user to a specific property and stay.
/// It is distinct from `atlas_leases` (which covers long-term tenancies).
///
/// Booking lifecycle (status field):
///   pending_guest  — invite sent, guest hasn't completed onboarding
///   confirmed      — guest onboarded, booking is locked in
///   checked_in     — guest is on-property (check-in date reached)
///   completed      — guest checked out, stay finished
///   cancelled      — cancelled by guest, host, or platform
///
/// Syndication source (source field):
///   direct         — created directly by a landlord or cohost in Folio
///   cohost_network — originated from a Cohost Network (internal Atlas network instance)
///   ota_import     — imported from an external OTA (Airbnb, VRBO, etc.)
///
/// str_syndicated on the parent asset controls whether this property's
/// listings are visible to internal Cohost Network instances.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS atlas_bookings (
                    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id       UUID        NOT NULL,
                    asset_id        UUID        NOT NULL REFERENCES atlas_assets(id) ON DELETE RESTRICT,
                    guest_user_id   UUID        REFERENCES users(id) ON DELETE SET NULL,
                    -- Date range for the stay
                    check_in_date   DATE        NOT NULL,
                    check_out_date  DATE        NOT NULL,
                    guest_count     INT         NOT NULL DEFAULT 1,
                    -- Lifecycle status
                    status          VARCHAR(20) NOT NULL DEFAULT 'pending_guest'
                                    CHECK (status IN (
                                        'pending_guest', 'confirmed', 'checked_in',
                                        'completed', 'cancelled'
                                    )),
                    -- Financials
                    total_amount    NUMERIC(10,2),
                    currency        VARCHAR(3)  NOT NULL DEFAULT 'USD',
                    -- Origin tracking
                    source          VARCHAR(30) NOT NULL DEFAULT 'direct'
                                    CHECK (source IN ('direct', 'cohost_network', 'ota_import')),
                    -- Network instance that referred this booking (if source = 'cohost_network')
                    -- References the atlas network instance UUID from the referring platform.
                    network_instance_id UUID,
                    -- External OTA booking ID (if source = 'ota_import')
                    ota_booking_ref VARCHAR(100),
                    -- Guest-facing notes (check-in instructions, host notes)
                    host_note       TEXT,
                    -- Timestamps
                    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    -- Constraints
                    CHECK (check_out_date > check_in_date),
                    CHECK (guest_count > 0)
                );

                -- Index: fast lookup of all bookings for a given property
                CREATE INDEX IF NOT EXISTS idx_bookings_asset_id
                    ON atlas_bookings (asset_id, check_in_date, check_out_date);

                -- Index: fast lookup of all bookings for a given guest
                CREATE INDEX IF NOT EXISTS idx_bookings_guest_user_id
                    ON atlas_bookings (guest_user_id)
                    WHERE guest_user_id IS NOT NULL;

                -- Index: active/upcoming bookings by status
                CREATE INDEX IF NOT EXISTS idx_bookings_status
                    ON atlas_bookings (tenant_id, status)
                    WHERE status IN ('pending_guest', 'confirmed', 'checked_in');

                -- Prevent double-bookings: no two confirmed/checked_in bookings
                -- can overlap on the same asset.
                -- (Soft constraint — enforced at service layer for flexibility;
                --  hard enforcement done in atlas_bookings_service::check_availability)
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"DROP INDEX IF EXISTS idx_bookings_status;
                DROP INDEX IF EXISTS idx_bookings_guest_user_id;
                DROP INDEX IF EXISTS idx_bookings_asset_id;
                DROP TABLE IF EXISTS atlas_bookings;"#,
            )
            .await?;
        Ok(())
    }
}
