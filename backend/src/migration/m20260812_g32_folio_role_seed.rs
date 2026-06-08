use sea_orm_migration::prelude::*;

/// G-32 seed: platform-default role profiles for the Folio PM app.
///
/// Inserts three platform-default profiles (tenant_id = NULL,
/// is_platform_default = true) for `app_slug = 'folio'`:
///
///   landlord — Property Manager: full PM suite access
///   tenant   — Renter: own lease, payments, maintenance, reservations
///   vendor   — Contractor: assigned work orders + invoices
///
/// Also inserts the permission slugs for each profile.
///
/// All inserts use ON CONFLICT DO NOTHING so re-runs are safe.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Insert platform-default profiles ───────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profiles
                    (id, tenant_id, app_slug, role_slug, display_name, description, is_platform_default)
                 VALUES
                    -- landlord
                    ('00000000-0000-0000-0001-000000000001'::uuid,
                     NULL, 'folio', 'landlord', 'Property Manager',
                     'Full access to the Folio PM suite: portfolio, assets, leases, leads, billing, STR, catalog, vendors, campaigns.',
                     true),
                    -- tenant
                    ('00000000-0000-0000-0001-000000000002'::uuid,
                     NULL, 'folio', 'tenant', 'Tenant',
                     'Renter access: view own lease, submit payments, file maintenance requests, view reservations.',
                     true),
                    -- vendor
                    ('00000000-0000-0000-0001-000000000003'::uuid,
                     NULL, 'folio', 'vendor', 'Vendor',
                     'Contractor access: view and complete assigned work orders, submit and track invoices.',
                     true)
                 ON CONFLICT (tenant_id, app_slug, role_slug) DO NOTHING;",
            )
            .await?;

        // ── 2. Landlord permission slugs ──────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'portfolio:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'portfolio:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'asset:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'asset:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'lease:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'lease:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'lead:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'lead:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'billing:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'billing:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'billing:admin'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'str:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'str:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'catalog:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'catalog:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'vendor:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'vendor:manage'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'campaign:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'campaign:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'reservation:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'reservation:write'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'maintenance:read'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'maintenance:dispatch'),
                    ('00000000-0000-0000-0001-000000000001'::uuid, 'rbac:assign')
                 ON CONFLICT (role_profile_id, permission_slug) DO NOTHING;",
            )
            .await?;

        // ── 3. Tenant permission slugs ────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                    ('00000000-0000-0000-0001-000000000002'::uuid, 'lease:read'),
                    ('00000000-0000-0000-0001-000000000002'::uuid, 'payments:read'),
                    ('00000000-0000-0000-0001-000000000002'::uuid, 'payments:submit'),
                    ('00000000-0000-0000-0001-000000000002'::uuid, 'maintenance:submit'),
                    ('00000000-0000-0000-0001-000000000002'::uuid, 'reservation:read')
                 ON CONFLICT (role_profile_id, permission_slug) DO NOTHING;",
            )
            .await?;

        // ── 4. Vendor permission slugs ────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                    ('00000000-0000-0000-0001-000000000003'::uuid, 'work_order:read'),
                    ('00000000-0000-0000-0001-000000000003'::uuid, 'work_order:complete'),
                    ('00000000-0000-0000-0001-000000000003'::uuid, 'invoice:read'),
                    ('00000000-0000-0000-0001-000000000003'::uuid, 'invoice:write')
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
                  WHERE app_slug = 'folio' AND is_platform_default = true;",
            )
            .await?;
        Ok(())
    }
}
