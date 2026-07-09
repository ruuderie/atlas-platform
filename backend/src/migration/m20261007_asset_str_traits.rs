use sea_orm_migration::prelude::*;

/// Add STR (short-term rental) trait columns to `atlas_assets`.
///
/// STR capability is NOT a user role — it is a property trait. A Landlord
/// who opts one of their assets into short-term rental is still a Landlord.
/// The frontend reads `has_str_assets` from SessionInfo and shows/hides
/// the STR navigation sections dynamically.
///
/// Column semantics:
///
/// `str_eligible`          — Landlord has accepted STR terms for this specific
///                           property. Gate for everything STR on this asset.
///
/// `str_listing_active`    — Listing is live on the Folio platform marketplace
///                           (visible to cohosts and guests searching listings).
///                           Cannot be true when str_eligible = false.
///
/// `str_syndicated`        — Listing is being pushed to the Cohost Network
///                           (internal syndication). External OTA channels will
///                           be tracked in a separate `str_channels TEXT[]`
///                           column added in a future migration.
///
/// `str_terms_accepted_at` — Audit timestamp of when str_eligible was first set
///                           true for this asset. Useful for compliance/billing.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE atlas_assets
                    ADD COLUMN IF NOT EXISTS str_eligible          BOOLEAN     NOT NULL DEFAULT false,
                    ADD COLUMN IF NOT EXISTS str_listing_active    BOOLEAN     NOT NULL DEFAULT false,
                    ADD COLUMN IF NOT EXISTS str_syndicated        BOOLEAN     NOT NULL DEFAULT false,
                    ADD COLUMN IF NOT EXISTS str_terms_accepted_at TIMESTAMPTZ;

                -- Partial index: fast lookup of a landlord's STR-eligible properties
                CREATE INDEX IF NOT EXISTS idx_assets_str_eligible
                    ON atlas_assets (tenant_id, owner_user_id)
                    WHERE str_eligible = true;

                -- Partial index: active marketplace listings
                CREATE INDEX IF NOT EXISTS idx_assets_str_listing_active
                    ON atlas_assets (tenant_id)
                    WHERE str_listing_active = true;

                -- Enforce: listing can't be active if asset isn't STR-eligible
                ALTER TABLE atlas_assets
                    ADD CONSTRAINT chk_str_listing_requires_eligible
                    CHECK (NOT str_listing_active OR str_eligible);

                -- Enforce: syndication requires active listing
                ALTER TABLE atlas_assets
                    ADD CONSTRAINT chk_str_syndicated_requires_listing
                    CHECK (NOT str_syndicated OR str_listing_active);"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE atlas_assets
                    DROP CONSTRAINT IF EXISTS chk_str_syndicated_requires_listing,
                    DROP CONSTRAINT IF EXISTS chk_str_listing_requires_eligible,
                    DROP COLUMN IF EXISTS str_terms_accepted_at,
                    DROP COLUMN IF EXISTS str_syndicated,
                    DROP COLUMN IF EXISTS str_listing_active,
                    DROP COLUMN IF EXISTS str_eligible;
                DROP INDEX IF EXISTS idx_assets_str_eligible;
                DROP INDEX IF EXISTS idx_assets_str_listing_active;"#,
            )
            .await?;
        Ok(())
    }
}
