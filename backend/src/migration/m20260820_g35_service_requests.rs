use sea_orm_migration::prelude::*;

/// G-35: `atlas_service_requests` — Renter-to-vendor service request table.
///
/// # Concept
///
/// Renters (or property owners) can send a service request directly to a vendor
/// in the Folio network without needing a landlord to mediate. This enables:
///   - Cold-traffic renters arriving from `/help` (zero-auth, public endpoint)
///   - Property owners selecting a vendor from their dashboard
///   - Authenticated tenants via the tenant portal
///
/// # Authentication
///
/// Requests can come from:
///   - Zero-auth renters: `request_metadata->>'auth' = 'public'`
///   - Authenticated sessions: `requested_by_user_id` is set
///
/// # Notification hook (G-35)
///
/// After INSERT, the backend dispatches a `service_request_received`
/// notification (HIGH priority) to the vendor via `NotificationService::dispatch`.
///
/// # Lead-gen funnel
///
/// `utm_source` in `request_metadata` tracks referral source
/// (vendor link, QR code, Google, etc.) for growth analytics.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE IF NOT EXISTS atlas_service_requests (
                    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),

                    -- The vendor receiving this request (required)
                    vendor_id       UUID        NOT NULL
                                    REFERENCES atlas_service_providers(id) ON DELETE CASCADE,

                    -- Authenticated requester (NULL for public/renter zero-auth flow)
                    requested_by_user_id UUID   REFERENCES "user"(id) ON DELETE SET NULL,

                    -- Optional: which asset/property the request relates to
                    asset_id        UUID        REFERENCES atlas_assets(id) ON DELETE SET NULL,

                    -- Request content
                    description     TEXT        NOT NULL,
                    urgency         TEXT        NOT NULL DEFAULT 'not_urgent'
                                    CHECK (urgency IN ('not_urgent','this_week','emergency')),
                    address         TEXT,

                    -- Workflow state
                    status          TEXT        NOT NULL DEFAULT 'pending'
                                    CHECK (status IN ('pending','accepted','in_progress','completed','cancelled')),

                    -- Renter contact info + UTM source stored here for public requests
                    -- Keys: renter_name, renter_email, renter_phone, utm_source, auth
                    request_metadata JSONB      NOT NULL DEFAULT '{}',

                    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );

                -- Fast lookup: all requests for a given vendor (dashboard, notifications)
                CREATE INDEX IF NOT EXISTS idx_srv_req_vendor_id
                    ON atlas_service_requests (vendor_id, created_at DESC);

                -- Fast lookup: all requests by an authenticated user
                CREATE INDEX IF NOT EXISTS idx_srv_req_user_id
                    ON atlas_service_requests (requested_by_user_id)
                    WHERE requested_by_user_id IS NOT NULL;

                -- Status filter for vendor workflow views
                CREATE INDEX IF NOT EXISTS idx_srv_req_status
                    ON atlas_service_requests (vendor_id, status);

                -- Auto-update updated_at on row change
                DROP TRIGGER IF EXISTS set_updated_at_service_requests ON atlas_service_requests;
                CREATE TRIGGER set_updated_at_service_requests
                    BEFORE UPDATE ON atlas_service_requests
                    FOR EACH ROW EXECUTE FUNCTION set_updated_at_column();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS atlas_service_requests CASCADE;",
            )
            .await?;

        Ok(())
    }
}
