use sea_orm_migration::prelude::*;

/// G-34: Vendor Marketplace — opt-in cross-tenant vendor discovery.
///
/// # Concept
///
/// `atlas_service_providers` already supports `scope = "platform"` (available
/// across the Atlas network). This migration adds the UI surface for that scope:
/// opt-in fields that let a vendor publish themselves to the inter-landlord
/// marketplace, discoverable by proximity + trade type.
///
/// # Isolation model
///
/// - Vendor profiles remain per-tenant at the data layer.
/// - `is_marketplace_visible = true` is the opt-in gate. The vendor (or their
///   landlord) controls this flag via PATCH /marketplace/my-listing.
/// - The marketplace query reads across tenant rows via a platform-level query
///   (no tenant_id filter), but only exposes: business_name, bio, trade_types,
///   location, rating_avg, rating_count, endorsement_count.
/// - PII (notes, btc_wallet, stripe_id) is never surfaced cross-tenant.
///
/// # Endorsement storage
///
/// Endorsements use the existing G-22 `atlas_record_relationships` table:
///   source_entity_type = "atlas_account"  (the landlord's account)
///   target_entity_type = "atlas_service_providers"
///   relationship_type  = "marketplace_endorsement"
///
/// Endorsement count = COUNT(*) GROUP BY target_entity_id.
/// No new table needed.
///
/// # Cross-app applicability
///
/// Any future app that has contractors/agents/freelancers can use the same
/// `is_marketplace_visible` pattern on their service-provider equivalent.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── Non-geo columns (no PostGIS dependency) ───────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "-- Opt-in marketplace visibility flag.
                 -- false = private (existing behavior). true = discoverable cross-tenant.
                 ALTER TABLE atlas_service_providers
                     ADD COLUMN IF NOT EXISTS is_marketplace_visible BOOLEAN NOT NULL DEFAULT false;

                 -- Short public-facing bio shown on the marketplace card.
                 ALTER TABLE atlas_service_providers
                     ADD COLUMN IF NOT EXISTS marketplace_bio TEXT;

                 -- Trade type slugs this vendor advertises in the marketplace.
                 -- Subset of (or equal to) their internal service_categories JSON.
                 -- Stored as TEXT[] for fast @> containment queries.
                 ALTER TABLE atlas_service_providers
                     ADD COLUMN IF NOT EXISTS marketplace_trade_types TEXT[] NOT NULL DEFAULT '{}';

                 -- Partial indexes — only fire for marketplace-visible vendors.
                 CREATE INDEX IF NOT EXISTS idx_sp_marketplace_visible
                     ON atlas_service_providers(is_marketplace_visible)
                     WHERE is_marketplace_visible = true;

                 -- Trade-type containment index (GIN for TEXT[]).
                 CREATE INDEX IF NOT EXISTS idx_sp_marketplace_trade_types
                     ON atlas_service_providers USING GIN(marketplace_trade_types)
                     WHERE is_marketplace_visible = true;",
            )
            .await?;

        // ── PostGIS-dependent: marketplace_location + GIST index ─────────────
        // G-01 installs PostGIS only when available. In plain postgres environments
        // (e.g. CI without the postgis package), it returns Ok(()) with no-op.
        // We must guard here the same way G-10 does, falling back to TEXT.
        let has_postgis = match manager
            .get_connection()
            .query_one(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "SELECT 1 FROM pg_extension WHERE extname = 'postgis';".to_owned(),
            ))
            .await
        {
            Ok(Some(_)) => true,
            _ => false,
        };

        if has_postgis {
            manager
                .get_connection()
                .execute_unprepared(
                    "-- Public location for proximity matching.
                     -- Using GEOGRAPHY type for accurate distance calculations in meters.
                     ALTER TABLE atlas_service_providers
                         ADD COLUMN IF NOT EXISTS marketplace_location GEOGRAPHY(Point, 4326);

                     -- GiST spatial index for proximity queries.
                     CREATE INDEX IF NOT EXISTS idx_sp_marketplace_location
                         ON atlas_service_providers USING GIST(marketplace_location)
                         WHERE is_marketplace_visible = true;",
                )
                .await?;
        } else {
            // PostGIS unavailable (e.g. plain postgres in CI). Store as TEXT so the
            // column exists for ORM mapping; proximity search is skipped at runtime.
            tracing::warn!(
                "G-34: PostGIS not available — marketplace_location stored as TEXT (no GIST index). \
                 Proximity search will be disabled until PostGIS is installed."
            );
            manager
                .get_connection()
                .execute_unprepared(
                    "ALTER TABLE atlas_service_providers
                         ADD COLUMN IF NOT EXISTS marketplace_location TEXT;",
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_sp_marketplace_trade_types;
                 DROP INDEX IF EXISTS idx_sp_marketplace_location;
                 DROP INDEX IF EXISTS idx_sp_marketplace_visible;
                 ALTER TABLE atlas_service_providers
                     DROP COLUMN IF EXISTS marketplace_location,
                     DROP COLUMN IF EXISTS marketplace_trade_types,
                     DROP COLUMN IF EXISTS marketplace_bio,
                     DROP COLUMN IF EXISTS is_marketplace_visible;",
            )
            .await?;
        Ok(())
    }
}
