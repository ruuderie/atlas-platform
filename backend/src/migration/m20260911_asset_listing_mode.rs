use sea_orm_migration::prelude::*;

/// G-10 Atlas Asset: listing_mode column.
///
/// Adds a `listing_mode` field to `atlas_assets` that declares the syndication
/// intent for a real-estate asset. This drives which syndication links the asset
/// is eligible for when Folio publishes listings to linked Network Instances.
///
/// # Values
///
/// | Value      | Meaning                                                          |
/// |---|---|
/// | ltr        | Long-term rental — eligible for LTR syndication links            |
/// | str        | Short-term rental — eligible for STR syndication links           |
/// | both       | Both LTR and STR (e.g. seasonal flex property)                   |
/// | for_sale   | Brokerage listing — requires folio_mode='brokerage'              |
/// | NULL       | Not listed / no syndication intent                               |
///
/// # Rationale
///
/// Previously, STR vs. LTR distinction was implied by contract type (atlas_contracts).
/// This required a join to determine an asset's syndication eligibility. The `listing_mode`
/// column makes this a direct, indexed attribute of the asset itself, enabling:
///
/// - Efficient syndication fan-out queries (filter assets by listing_mode)
/// - Listing type tagging visible in the operator UI without joining contracts
/// - Correct routing to the appropriate NI syndication link(s)
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_assets
                    ADD COLUMN IF NOT EXISTS listing_mode TEXT
                        CHECK (listing_mode IN ('ltr', 'str', 'both', 'for_sale'));

                 -- NULL = not listed (default for all existing assets — backward compatible)
                 -- Partial index: only index assets that are actually listed
                 CREATE INDEX IF NOT EXISTS idx_atlas_assets_listing_mode
                     ON atlas_assets (listing_mode)
                     WHERE listing_mode IS NOT NULL;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_atlas_assets_listing_mode;
                 ALTER TABLE atlas_assets DROP COLUMN IF EXISTS listing_mode;",
            )
            .await?;

        Ok(())
    }
}
