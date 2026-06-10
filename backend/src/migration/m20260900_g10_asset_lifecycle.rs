use sea_orm_migration::prelude::*;

/// G-10 Enhancement: Universal Asset Lifecycle Extension
///
/// # What this adds
///
/// Four new columns on `atlas_assets` that every AtlasApp can use to track
/// physical asset lifecycle without creating app-specific tables:
///
/// | Column | Purpose |
/// |---|---|
/// | `scheduled_service_date` | Next maintenance / calibration / inspection due |
/// | `expiry_date`            | Warranty / cert / license / registration expiry |
/// | `condition`              | Operational state (excellent / good / fair / poor / retired) |
/// | `lifecycle_metadata`     | App-owned JSONB typed sidecar — make, model, serial, domain fields |
///
/// # Platform-level alert query (works for ALL asset_types, ALL verticals)
///
/// ```sql
/// SELECT * FROM atlas_assets
/// WHERE tenant_id = $1
///   AND (scheduled_service_date < NOW() + INTERVAL '30 days'
///        OR expiry_date          < NOW() + INTERVAL '30 days')
/// ORDER BY LEAST(scheduled_service_date, expiry_date) ASC;
/// ```
///
/// # App-layer contract
///
/// Each AtlasApp:
/// 1. Defines its own typed struct (e.g. `ApplianceMetadata`) for `lifecycle_metadata`
/// 2. Implements `TryFrom<&AssetModel>` to deserialize + domain-validate
/// 3. Writes the three indexed platform columns on every create/update
///
/// See: `docs/architecture/asset_metadata_shapes.md` for per-app shapes.
///
/// # Tradeoffs (deliberate — see platform_generics_v3.md §8 Risk #8)
///
/// JSONB is chosen over extension tables to avoid per-vertical migrations.
/// Optimization triggers are documented and will prompt migration to typed
/// columns if hit. `lifecycle_metadata` keys are treated as stable public API.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "-- G-10 Universal Asset Lifecycle Extension
                 -- Four columns that replace per-vertical extension tables.

                 -- Next scheduled maintenance / calibration / inspection due date.
                 -- Written by each app; queried platform-wide for alert generation.
                 ALTER TABLE atlas_assets
                     ADD COLUMN IF NOT EXISTS scheduled_service_date DATE;

                 -- Warranty / certificate / license / registration expiry date.
                 -- Written by each app; queried platform-wide for expiry alerts.
                 ALTER TABLE atlas_assets
                     ADD COLUMN IF NOT EXISTS expiry_date DATE;

                 -- Current operational state of the asset.
                 -- Valid values: excellent | good | fair | poor | retired
                 -- Additional values allowed per app (e.g. 'decommissioned', 'stolen').
                 ALTER TABLE atlas_assets
                     ADD COLUMN IF NOT EXISTS condition VARCHAR(30);

                 -- App-owned typed metadata sidecar.
                 -- Shape varies per asset_type. Each app defines its own Rust struct
                 -- that serializes here. Keys treated as stable public API.
                 -- See docs/architecture/asset_metadata_shapes.md for registered shapes.
                 ALTER TABLE atlas_assets
                     ADD COLUMN IF NOT EXISTS lifecycle_metadata JSONB;

                 -- ── Indexes ──────────────────────────────────────────────────────────

                 -- Service schedule index: powers 'due soon' alert queries.
                 -- Partial: only rows with a service date set (null rows excluded).
                 CREATE INDEX IF NOT EXISTS idx_assets_service_due
                     ON atlas_assets (tenant_id, scheduled_service_date)
                     WHERE scheduled_service_date IS NOT NULL;

                 -- Expiry index: powers warranty + cert expiry alert queries.
                 CREATE INDEX IF NOT EXISTS idx_assets_expiry
                     ON atlas_assets (tenant_id, expiry_date)
                     WHERE expiry_date IS NOT NULL;

                 -- Condition index: powers fleet-health dashboard queries.
                 CREATE INDEX IF NOT EXISTS idx_assets_condition
                     ON atlas_assets (tenant_id, condition)
                     WHERE condition IS NOT NULL;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_assets_condition;
                 DROP INDEX IF EXISTS idx_assets_expiry;
                 DROP INDEX IF EXISTS idx_assets_service_due;
                 ALTER TABLE atlas_assets
                     DROP COLUMN IF EXISTS lifecycle_metadata,
                     DROP COLUMN IF EXISTS condition,
                     DROP COLUMN IF EXISTS expiry_date,
                     DROP COLUMN IF EXISTS scheduled_service_date;",
            )
            .await?;
        Ok(())
    }
}
