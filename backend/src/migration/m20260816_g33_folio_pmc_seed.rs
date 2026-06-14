use sea_orm_migration::prelude::*;

/// G-33 seed: Folio PMC mode — platform-default `property_manager` role profile.
///
/// Adds the `property_manager` role to the Folio app's role catalog.
/// This role is only meaningful when the tenant's app deployment config
/// has `"pmc_enabled": true` inside the `config` payload — the `AppDeploymentConfig`
/// extractor parses this at the handler level.
///
/// UUID: 00000000-0000-0000-0001-000000000004 (follows the existing 001/002/003 sequence)
///
/// Permission model:
///   property_manager = all landlord permissions + cross-client access grants
///   (client:*, analytics:cross_client)
///
/// These are Folio-specific and registered via FolioApp::migrations(), not the base vec.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Platform-default property_manager role profile ──────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profiles
                    (id, tenant_id, app_slug, role_slug, display_name, description, is_platform_default)
                 VALUES
                    ('00000000-0000-0000-0001-000000000004'::uuid,
                     NULL,
                     'folio',
                     'property_manager',
                     'Property Manager (PMC)',
                     'Cross-client property management: manages multiple landlord client accounts \
                      within a single PMC tenant. Requires pmc_enabled=true inside config in \
                      atlas_app_deployment_config.',
                     true)
                 ON CONFLICT (tenant_id, app_slug, role_slug) DO NOTHING;",
            )
            .await?;

        // ── 2. property_manager permission slugs ───────────────────────────────
        // Superset of landlord permissions + PMC-specific slugs.
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                    -- PMC-specific
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'client:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'client:write'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'client:onboard'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'analytics:cross_client'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'app_config:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'app_config:write'),
                    -- Portfolio (same as landlord)
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'portfolio:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'portfolio:write'),
                    -- Assets
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'asset:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'asset:write'),
                    -- Leases
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'lease:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'lease:write'),
                    -- Leads
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'lead:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'lead:write'),
                    -- Billing
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'billing:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'billing:write'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'billing:admin'),
                    -- STR
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'str:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'str:write'),
                    -- Catalog
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'catalog:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'catalog:write'),
                    -- Vendor management
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'vendor:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'vendor:manage'),
                    -- Campaigns
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'campaign:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'campaign:write'),
                    -- Reservations
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'reservation:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'reservation:write'),
                    -- Maintenance
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'maintenance:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'maintenance:dispatch'),
                    -- Work orders
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'work_order:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'work_order:complete'),
                    -- Invoices
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'invoice:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'invoice:write'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'invoice:approve'),
                    -- RBAC (PM can assign roles within their tenant)
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'rbac:read'),
                    ('00000000-0000-0000-0001-000000000004'::uuid, 'rbac:assign')
                 ON CONFLICT (role_profile_id, permission_slug) DO NOTHING;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM atlas_role_profiles
                  WHERE id = '00000000-0000-0000-0001-000000000004'::uuid;",
            )
            .await?;
        Ok(())
    }
}
