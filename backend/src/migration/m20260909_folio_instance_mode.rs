use sea_orm_migration::prelude::*;

/// Folio Instance Mode — foundational operational identity for Folio app instances.
///
/// Replaces the ad-hoc `pmc_enabled: true` JSON boolean in `atlas_app_deployment_config.config`
/// with a proper typed `folio_mode` column that enforces mutual exclusivity at the
/// database level.
///
/// # Valid modes
///
/// | Mode       | Operator type                           | Unlocked portals          |
/// |---|---|---|
/// | standard   | Solo landlord / portfolio operator      | /l/** (always)            |
/// | pmc        | Property Management Company             | /pmc/**, /l/** available  |
/// | brokerage  | Real estate brokerage                   | /a/**, /b/**              |
///
/// # Portal toggles (separate from mode — stored in config JSON)
///
/// `tenant_portal_enabled` and `vendor_portal_enabled` remain as JSON config keys.
/// They control whether /t/** and /v/** are exposed for the instance, independent
/// of the operator mode.
///
/// # Back-fill
///
/// Any existing row with `config->>'pmc_enabled' = 'true'` is migrated to
/// `folio_mode = 'pmc'`. The `pmc_enabled` key is NOT removed from the JSON
/// to preserve backward compatibility with any cached config reads during rollout.
/// It will be cleaned up in a subsequent migration once all readers are updated.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_app_deployment_config
                    ADD COLUMN IF NOT EXISTS folio_mode TEXT NOT NULL DEFAULT 'standard'
                        CHECK (folio_mode IN ('standard', 'pmc', 'brokerage'));

                 -- Back-fill: rows that had pmc_enabled=true in config JSON get folio_mode='pmc'
                 UPDATE atlas_app_deployment_config
                    SET folio_mode = 'pmc'
                  WHERE app_slug = 'property_management'
                    AND (config->>'pmc_enabled')::boolean IS TRUE;

                 -- Index for fast mode-based queries on Folio instances
                 CREATE INDEX IF NOT EXISTS idx_app_deployment_config_folio_mode
                     ON atlas_app_deployment_config (folio_mode)
                     WHERE app_slug = 'property_management';",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_app_deployment_config_folio_mode;
                 ALTER TABLE atlas_app_deployment_config DROP COLUMN IF EXISTS folio_mode;",
            )
            .await?;

        Ok(())
    }
}
