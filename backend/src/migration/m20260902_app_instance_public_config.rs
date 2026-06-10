use sea_orm_migration::prelude::*;

/// App Instance Public Config — adds public_slug and custom_domain to
/// atlas_app_deployment_config (G-33).
///
/// These two columns enable zero-tenant domain resolution:
///
/// - `public_slug`     — short handle for shared-platform URLs,
///                       e.g. "oakwood" → oakwood.folio.app/listings
/// - `custom_domain`   — full FQDN for white-label deployments,
///                       e.g. "listings.oakwoodpm.com"
///
/// Both are globally unique across all tenants. The
/// `GET /api/pub/tenant-context` endpoint resolves either to a
/// tenant brand config for use by unauthenticated public pages.
///
/// In dedicated-instance mode (ATLAS_TENANT_ID env var set), these
/// columns are still stored but the resolver short-circuits to the
/// env var tenant ID.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_app_deployment_config
                    ADD COLUMN IF NOT EXISTS public_slug    TEXT UNIQUE,
                    ADD COLUMN IF NOT EXISTS custom_domain  TEXT UNIQUE,
                    -- 'active' | 'suspended' | 'archived' — lifecycle state
                    ADD COLUMN IF NOT EXISTS instance_status TEXT NOT NULL DEFAULT 'active';

                 CREATE INDEX IF NOT EXISTS idx_app_deployment_config_public_slug
                     ON atlas_app_deployment_config (public_slug)
                     WHERE public_slug IS NOT NULL;

                 CREATE INDEX IF NOT EXISTS idx_app_deployment_config_custom_domain
                     ON atlas_app_deployment_config (custom_domain)
                     WHERE custom_domain IS NOT NULL;

                 CREATE INDEX IF NOT EXISTS idx_app_deployment_config_status
                     ON atlas_app_deployment_config (instance_status);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_app_deployment_config_public_slug;
                 DROP INDEX IF EXISTS idx_app_deployment_config_custom_domain;
                 DROP INDEX IF EXISTS idx_app_deployment_config_status;
                 ALTER TABLE atlas_app_deployment_config
                     DROP COLUMN IF EXISTS public_slug,
                     DROP COLUMN IF EXISTS custom_domain,
                     DROP COLUMN IF EXISTS instance_status;",
            )
            .await?;
        Ok(())
    }
}
