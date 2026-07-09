use sea_orm_migration::prelude::*;

/// Folio G-11 extension: `atlas_leases` — Long-Term Rental Tenancy Ledger.
///
/// # Context
///
/// `atlas_contracts` (G-11) is the platform-generic legal agreement table.
/// `atlas_leases` is a Folio-specific table for residential LTR lease tracking.
/// It stores the operational state of an active tenancy:
///   - Which unit (asset) is leased
///   - Who the tenant is (tenant_user_id — assigned when they accept an invite)
///   - Lease term dates, rent amount, status
///
/// # Why a separate table from atlas_contracts?
///
/// `atlas_contracts` is a generic legal entity; `atlas_leases` is the *operational*
/// record that Folio's property manager dashboard reads for:
///   - Upcoming renewals / expirations
///   - Rent roll (monthly income projections)
///   - Maintenance requests linked to a specific tenancy
///   - Tenant invite flow (platform_invite.lease_id FK)
///
/// # Relationships
///
/// - `platform_invite.lease_id` → `atlas_leases.id` (set when PM sends tenant invite)
/// - `auth_frontend.rs`: UPDATE atlas_leases SET tenant_user_id = $user after invite accept
/// - `atlas_bookings`: distinct from leases — covers STR guest reservations
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE IF NOT EXISTS atlas_leases (
                    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id           UUID        NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,

                    -- The unit being leased
                    asset_id            UUID        REFERENCES atlas_assets(id) ON DELETE SET NULL,

                    -- Assigned when tenant accepts their platform_invite
                    tenant_user_id      UUID        REFERENCES "user"(id) ON DELETE SET NULL,

                    -- Lease terms
                    start_date          DATE        NOT NULL,
                    end_date            DATE,
                    auto_renew          BOOLEAN     NOT NULL DEFAULT false,
                    lease_type          TEXT        NOT NULL DEFAULT 'ltr'
                                        CHECK (lease_type IN ('ltr', 'str', 'month_to_month')),

                    -- Financial
                    rent_amount_cents   BIGINT,
                    currency            CHAR(3)     NOT NULL DEFAULT 'USD',
                    billing_interval    TEXT        NOT NULL DEFAULT 'monthly',

                    -- Status
                    status              TEXT        NOT NULL DEFAULT 'draft'
                                        CHECK (status IN ('draft','pending','active','expired','terminated','renewed')),

                    -- Metadata
                    lease_metadata      JSONB       NOT NULL DEFAULT '{}',

                    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );

                -- PM dashboard: all leases for a given tenant account
                CREATE INDEX IF NOT EXISTS idx_atlas_leases_tenant_id
                    ON atlas_leases (tenant_id, status);

                -- Asset timeline: all leases for a unit (occupancy history)
                CREATE INDEX IF NOT EXISTS idx_atlas_leases_asset_id
                    ON atlas_leases (asset_id, start_date DESC)
                    WHERE asset_id IS NOT NULL;

                -- Tenant portal: find my lease record after invite acceptance
                CREATE INDEX IF NOT EXISTS idx_atlas_leases_tenant_user_id
                    ON atlas_leases (tenant_user_id)
                    WHERE tenant_user_id IS NOT NULL;

                -- Auto-update updated_at: trigger added in a follow-up migration
                -- once set_updated_at_column() is defined (m20260601_g31_atlas_lead).
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS atlas_leases CASCADE;")
            .await?;
        Ok(())
    }
}
