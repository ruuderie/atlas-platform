use sea_orm_migration::prelude::*;

/// Platform-generic instance syndication active links.
///
/// Layer B of the two-layer syndication model. One row per (source app instance,
/// destination NI) pair. Created when:
///   (a) An operator self-service activates an offer (requires offer.self_service_allowed = true)
///   (b) Platform admin manually creates a link
///   (c) Auto-provisioned at instance creation for mandatory offers
///       (offer.is_mandatory_for_tiers includes tenant's billing tier)
///
/// This table is NOT Folio-specific. Any Atlas app (current or future) can
/// have syndication links to NetworkInstance deployments.
///
/// # Bidirectional event contract
///
/// Outbound (source → NI): driven by G-05 outbox pattern. When a listing is
/// published/updated, a syndication event is fired. The outbox worker reads
/// active links and dispatches the event to each linked NI's integration handler.
///
/// Inbound (NI → source): the NI fires a webhook POST to `inbound_webhook_url`
/// for events like listing.inquiry, listing.application, vendor.signup.
/// The source app verifies HMAC-SHA256 using `inbound_webhook_secret` and
/// routes the event into its CRM (G-31 leads, lease applications, etc.).
///
/// All events logged in `atlas_integration_events` (G-05).
///
/// # Same-tenant vs cross-tenant
///
/// Both go through this table. The link has its own properties (types, mandatory flag,
/// status, webhook) that exist independently of tenant ownership. Best practice in
/// marketplace architecture (Shopify, Stripe Connect, Airbnb) is to always model
/// connections explicitly for consistency, auditability, and future flexibility.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS atlas_app_instance_syndication (
                    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),

                    -- Source: any app deployment config (Folio, future apps)
                    source_config_id        UUID        NOT NULL
                                            REFERENCES atlas_app_deployment_config(id)
                                            ON DELETE CASCADE,

                    -- Destination: NetworkInstance deployment config
                    ni_config_id            UUID        NOT NULL
                                            REFERENCES atlas_app_deployment_config(id)
                                            ON DELETE CASCADE,

                    -- Reference to the offer that governs this link's terms
                    -- NULL = manually created by platform admin without an offer template
                    offer_id                UUID
                                            REFERENCES atlas_syndication_offer(id)
                                            ON DELETE SET NULL,

                    -- Effective syndication types for this specific link
                    -- Defaults to offer.syndication_types; can be narrowed (not expanded) by operator
                    syndication_types       JSONB       NOT NULL DEFAULT '[]',

                    -- How this NI is presented / functions for this operator
                    -- 'branded_portal'         = operator's own branded website (1:1)
                    -- 'marketplace_syndication' = shared platform directory (many:1)
                    link_type               TEXT        NOT NULL DEFAULT 'marketplace_syndication'
                        CHECK (link_type IN ('branded_portal', 'marketplace_syndication')),

                    -- Whether this link was auto-created due to a mandatory offer rule
                    -- Mandatory links cannot be revoked by the operator
                    is_mandatory            BOOLEAN     NOT NULL DEFAULT false,

                    -- Link lifecycle: active | paused | revoked
                    status                  TEXT        NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active', 'paused', 'revoked')),

                    -- Inbound webhook: NI posts events to this URL on the source app side
                    -- NULL = unidirectional outbound only
                    inbound_webhook_url     TEXT,

                    -- HMAC-SHA256 secret for verifying inbound events from NI
                    inbound_webhook_secret  TEXT,

                    -- Audit
                    created_by_tenant_id    UUID        NOT NULL REFERENCES tenant(id),

                    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

                    -- Only one link per (source, NI) pair
                    CONSTRAINT uq_app_instance_syndication UNIQUE (source_config_id, ni_config_id)
                );

                CREATE INDEX IF NOT EXISTS idx_app_syndication_source
                    ON atlas_app_instance_syndication (source_config_id);

                CREATE INDEX IF NOT EXISTS idx_app_syndication_ni
                    ON atlas_app_instance_syndication (ni_config_id);

                CREATE INDEX IF NOT EXISTS idx_app_syndication_status
                    ON atlas_app_instance_syndication (status);

                CREATE INDEX IF NOT EXISTS idx_app_syndication_offer
                    ON atlas_app_instance_syndication (offer_id)
                    WHERE offer_id IS NOT NULL;

                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_app_instance_syndication
                        BEFORE UPDATE ON atlas_app_instance_syndication
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                 EXCEPTION WHEN duplicate_object THEN NULL;
                 END $$;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS set_updated_at_app_instance_syndication
                     ON atlas_app_instance_syndication;
                 DROP TABLE IF EXISTS atlas_app_instance_syndication;",
            )
            .await?;

        Ok(())
    }
}
