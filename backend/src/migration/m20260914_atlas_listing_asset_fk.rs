use sea_orm_migration::prelude::*;

/// atlas_listing: add asset_id FK to atlas_assets.
///
/// Formally links a listing to its source asset. Currently, listings link to a
/// `profile_id` which has an implicit relationship to properties/assets. This
/// migration adds a direct, nullable `asset_id` column so the relationship between
/// "thing being marketed" and "marketing record" is explicit and queryable.
///
/// # Why this matters for syndication
///
/// Syndication routing filters listings by `listing_type` (already on the listing).
/// The `asset_id` FK is not needed for syndication routing itself — it's needed so
/// the syndication payload can include accurate asset metadata (address, attributes,
/// photos) without joining through the profile.
///
/// # Nullable
///
/// NULL = listing is not backed by a tracked atlas_asset (e.g. legacy listings
/// created before G-10, or listings for non-asset products). These listings can
/// still be syndicated — the FK is informational, not a constraint on syndication
/// eligibility.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE listing
                    ADD COLUMN IF NOT EXISTS asset_id UUID
                        REFERENCES atlas_assets(id) ON DELETE SET NULL;

                 CREATE INDEX IF NOT EXISTS idx_listing_asset_id
                     ON listing (asset_id)
                     WHERE asset_id IS NOT NULL;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_listing_asset_id;
                 ALTER TABLE listing DROP COLUMN IF EXISTS asset_id;",
            )
            .await?;

        Ok(())
    }
}
