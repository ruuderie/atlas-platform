use sea_orm_migration::prelude::*;

/// Folio Instance Syndication — formal coupling between Folio instances and
/// Network Instances.
///
/// One row per (Folio deployment config, NI deployment config) pair.
/// A single Folio instance can syndicate to multiple NIs (e.g. LTR listings
/// to one directory, STR listings to a separate marketplace).
///
/// # Syndication types (stored as JSONB array)
///
/// | Value            | What flows                                              |
/// |---|---|
/// | "ltr"            | Long-term rental listings: Folio → NI property directory |
/// | "str"            | Short-term rental listings: Folio → NI STR marketplace  |
/// | "for_sale"       | Brokerage listings (requires folio_mode='brokerage')    |
/// | "vendor_profile" | Contractor profiles: Folio → NI vendor marketplace      |
/// | "tenant_profile" | Renter applications: NI → Folio (inbound)               |
///
/// # Bidirectional contract
///
/// Outbound (Folio → NI): handled by the SyndicationService when a listing
/// is published or updated.
///
/// Inbound (NI → Folio): the NI fires a webhook POST to `inbound_webhook_url`
/// on events like listing.inquiry, listing.application, vendor.signup.
/// Folio verifies the HMAC-SHA256 signature using `inbound_webhook_secret`
/// and routes the event into the Folio CRM (G31 leads / lease applications).
///
/// Both directions are logged in `atlas_integration_events` (G-05).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS folio_instance_syndication (
                    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),

                    -- Source: Folio app deployment config
                    folio_config_id         UUID        NOT NULL
                                            REFERENCES atlas_app_deployment_config(id)
                                            ON DELETE CASCADE,

                    -- Destination: NetworkInstance app deployment config
                    ni_config_id            UUID        NOT NULL
                                            REFERENCES atlas_app_deployment_config(id)
                                            ON DELETE CASCADE,

                    -- Which listing types flow through this link
                    -- Valid values: 'ltr', 'str', 'for_sale', 'vendor_profile', 'tenant_profile'
                    syndication_types       JSONB       NOT NULL DEFAULT '[]',

                    -- Link lifecycle: active | paused | revoked
                    status                  TEXT        NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active', 'paused', 'revoked')),

                    -- Inbound webhook: NI posts events to this URL on Folio's side
                    -- NULL = unidirectional push only (no inbound events)
                    inbound_webhook_url     TEXT,

                    -- HMAC-SHA256 secret for verifying inbound events from NI
                    inbound_webhook_secret  TEXT,

                    -- Operator tenant that owns this syndication agreement
                    created_by_tenant_id    UUID        NOT NULL REFERENCES tenant(id),

                    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

                    -- Only one active link per (Folio, NI) pair
                    CONSTRAINT uq_folio_ni_syndication UNIQUE (folio_config_id, ni_config_id)
                );

                CREATE INDEX IF NOT EXISTS idx_folio_syndication_folio
                    ON folio_instance_syndication (folio_config_id);

                CREATE INDEX IF NOT EXISTS idx_folio_syndication_ni
                    ON folio_instance_syndication (ni_config_id);

                CREATE INDEX IF NOT EXISTS idx_folio_syndication_status
                    ON folio_instance_syndication (status);

                -- Auto-update updated_at
                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_folio_syndication
                        BEFORE UPDATE ON folio_instance_syndication
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
                "DROP TRIGGER IF EXISTS set_updated_at_folio_syndication
                     ON folio_instance_syndication;
                 DROP TABLE IF EXISTS folio_instance_syndication;",
            )
            .await?;

        Ok(())
    }
}
