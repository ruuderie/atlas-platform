use sea_orm_migration::prelude::*;

/// G-33: atlas_app_deployment_config — Platform-generic app deployment configuration.
///
/// Every tenant's deployment of an app can declare what "mode" it runs in and
/// carry arbitrary JSON config for that mode. The platform stores the string; the
/// app interprets it.
///
/// # Use cases across apps
///
/// | App slug             | Mode slug                    | Meaning |
/// |---|---|---|
/// | folio                | standard                     | Single-landlord (default) |
/// | folio                | property_management_co       | PMC: manages multiple client books |
/// | crm (future)         | standard                     | In-house sales team |
/// | crm (future)         | agency                       | CRM-as-a-service for multiple end clients |
/// | hr (future)          | single_company               | Internal HR |
/// | hr (future)          | peo_bureau                   | PEO managing multiple employer clients |
///
/// # Schema
///
/// One row per (tenant_id, app_slug). UNIQUE constraint enforces this.
/// Missing row = mode "standard" with empty config (backward compatible).
///
/// # Extractor
///
/// `crate::extractors::app_config::AppDeploymentConfig` reads this table and
/// caches the result in request extensions. Zero extra DB round trips for handlers
/// that only need `TenantContext` + `AppDeploymentConfig`.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS atlas_app_deployment_config (
                    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id   UUID        NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                    app_slug    VARCHAR(100) NOT NULL,
                    -- mode is app-defined. The platform stores the string; the app interprets it.
                    -- Examples: 'standard', 'property_management_co', 'agency', 'peo_bureau'
                    mode        VARCHAR(100) NOT NULL DEFAULT 'standard',
                    -- Arbitrary JSON for mode-specific settings, e.g.:
                    --   { \"max_clients\": 50, \"billing_model\": \"per_unit\" }
                    config      JSONB        NOT NULL DEFAULT '{}',
                    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
                    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
                    CONSTRAINT uq_app_deployment_config_tenant_app
                        UNIQUE (tenant_id, app_slug)
                );
                CREATE INDEX IF NOT EXISTS idx_app_deployment_config_tenant_app
                    ON atlas_app_deployment_config (tenant_id, app_slug);",
            )
            .await?;

        // Auto-update updated_at on mutation
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_app_deployment_config
                        BEFORE UPDATE ON atlas_app_deployment_config
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
                "DROP TRIGGER IF EXISTS set_updated_at_app_deployment_config
                    ON atlas_app_deployment_config;
                 DROP TABLE IF EXISTS atlas_app_deployment_config;",
            )
            .await?;
        Ok(())
    }
}
