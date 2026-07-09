use sea_orm_migration::prelude::*;

/// G-32 seed: Property Owner Lite platform-default role profile for the Folio PM app.
///
/// Inserts one platform-default profile (tenant_id = NULL, is_platform_default = true)
/// for `app_slug = 'folio'`, role_slug = `'property_owner_lite'`.
///
/// Permission philosophy:
///   - CAN: track own property (asset read/write), view vendors, submit G-27 reviews,
///           log property value history, post marketplace service requests.
///   - CANNOT: manage leases, collect rent, dispatch maintenance, run campaigns,
///              access leads, or configure billing.
///
/// On upgrade to Landlord, `atlas_user_app_roles.role_slug` is updated in-place —
/// the user's existing `atlas_assets` and `atlas_asset_value_history` rows are not
/// touched. No re-entry required.
///
/// All inserts use ON CONFLICT DO NOTHING — re-runs are safe.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Insert profile row ─────────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profiles
                    (id, tenant_id, app_slug, role_slug, display_name, description, is_platform_default)
                 VALUES
                    ('00000000-0000-0000-0001-000000000007'::uuid,
                     NULL, 'folio', 'property_owner_lite', 'Property Owner',
                     'Free-tier owner: track property value and linked vendors, submit G-27 reviews, post marketplace service requests. No lease or billing access.',
                     true)
                 ON CONFLICT (tenant_id, app_slug, role_slug) DO NOTHING;",
            )
            .await?;

        // ── 2. Permission slugs ───────────────────────────────────────────────
        // Minimal surface: own property tracking + marketplace + reviews.
        // Landlord-only permissions (lease:*, billing:*, maintenance:*, campaign:*)
        // are intentionally absent.
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                    -- Property (own asset tracking only)
                    ('00000000-0000-0000-0001-000000000007'::uuid, 'asset:read'),
                    ('00000000-0000-0000-0001-000000000007'::uuid, 'asset:write'),
                    -- Vendor discovery (read-only; no vendor:manage)
                    ('00000000-0000-0000-0001-000000000007'::uuid, 'vendor:read'),
                    -- G-27 review submission
                    ('00000000-0000-0000-0001-000000000007'::uuid, 'review:submit'),
                    -- Property value history (log + view)
                    ('00000000-0000-0000-0001-000000000007'::uuid, 'property_value:read'),
                    ('00000000-0000-0000-0001-000000000007'::uuid, 'property_value:write'),
                    -- G-34 marketplace service request
                    ('00000000-0000-0000-0001-000000000007'::uuid, 'service_request:create')
                 ON CONFLICT (role_profile_id, permission_slug) DO NOTHING;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM atlas_role_profile_permissions
                 WHERE role_profile_id = '00000000-0000-0000-0001-000000000007'::uuid;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM atlas_role_profiles
                 WHERE id = '00000000-0000-0000-0001-000000000007'::uuid;",
            )
            .await?;

        Ok(())
    }
}
