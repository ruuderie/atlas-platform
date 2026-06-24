use sea_orm_migration::prelude::*;

/// Platform-generic syndication offer catalog.
///
/// Platform admin controlled. Defines what syndication connections exist on the platform,
/// their terms, and tier-based mandatory/self-service rules.
///
/// This is Layer A of the two-layer syndication model:
///
/// Layer A (this table): Platform admin defines what NI connections are available,
///   who must use them (free tier mandatory), and whether operators can self-service.
///
/// Layer B (`atlas_app_instance_syndication`): The active links — created when an
///   operator activates an offer, or auto-provisioned for mandatory offers.
///
/// # Link types
///
/// - "branded_portal"         → Operator gets their own branded website (1:1 coupling)
/// - "marketplace_syndication"→ Operator syndicates into a shared platform directory (many:1)
///
/// # Monetization model
///
/// `is_mandatory_for_tiers` is a JSONB array of billing tier slugs (e.g. ["free", "starter"])
/// for which this offer is automatically activated and cannot be revoked.
/// Example: free-tier Folio instances MUST syndicate to the platform marketplace.
/// This is how the platform monetizes operators who don't pay — their listings drive
/// traffic to our shared directory.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS atlas_syndication_offer (
                    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),

                    -- The destination Network Instance deployment config
                    ni_config_id            UUID        NOT NULL
                                            REFERENCES atlas_app_deployment_config(id)
                                            ON DELETE CASCADE,

                    -- Human-readable name shown in platform-admin and operator UI
                    display_name            TEXT        NOT NULL,

                    -- Brief description shown to operators in the self-service UI
                    description             TEXT,

                    -- What listing types flow through this offer
                    -- Valid values: 'ltr', 'str', 'for_sale', 'vendor_profile', 'tenant_profile'
                    syndication_types       JSONB       NOT NULL DEFAULT '[]',

                    -- How the NI is presented to the operator
                    -- 'branded_portal'         = operator gets their own branded website
                    -- 'marketplace_syndication' = operator syndicates into a shared directory
                    link_type               TEXT        NOT NULL DEFAULT 'marketplace_syndication'
                        CHECK (link_type IN ('branded_portal', 'marketplace_syndication')),

                    -- Monetization: billing tier slugs for which this offer is mandatory
                    -- Operators on these tiers cannot opt out
                    -- Example: [\"free\", \"starter\"]
                    is_mandatory_for_tiers  JSONB       NOT NULL DEFAULT '[]',

                    -- Permission: if true, operators can self-service activate/deactivate
                    -- If false, only platform admin can create the active link
                    self_service_allowed    BOOLEAN     NOT NULL DEFAULT false,

                    -- Filter: which folio_mode this offer applies to
                    -- NULL = applies to all modes / all app slugs
                    applies_to_folio_mode   TEXT,
                    applies_to_app_slug     TEXT,

                    -- Offer lifecycle
                    status                  TEXT        NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active', 'retired')),

                    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_syndication_offer_ni_config
                    ON atlas_syndication_offer (ni_config_id);

                CREATE INDEX IF NOT EXISTS idx_syndication_offer_status
                    ON atlas_syndication_offer (status);

                CREATE INDEX IF NOT EXISTS idx_syndication_offer_folio_mode
                    ON atlas_syndication_offer (applies_to_folio_mode)
                    WHERE applies_to_folio_mode IS NOT NULL;

                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_syndication_offer
                        BEFORE UPDATE ON atlas_syndication_offer
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
                "DROP TRIGGER IF EXISTS set_updated_at_syndication_offer
                     ON atlas_syndication_offer;
                 DROP TABLE IF EXISTS atlas_syndication_offer;",
            )
            .await?;

        Ok(())
    }
}
