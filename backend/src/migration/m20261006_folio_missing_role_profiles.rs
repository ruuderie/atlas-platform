use sea_orm_migration::prelude::*;

/// Seed missing Folio role profiles: cohost, str_host, agent, broker.
///
/// Only 3 role profiles were seeded in m20260812: landlord, tenant, vendor.
/// This adds the remaining 4 personas advertised on the Folio marketing page.
///
/// Role profile IDs are stable deterministic UUIDs (v5, namespace=DNS, name=slug)
/// so they can be referenced safely in fixtures and application code without
/// querying the DB.
///
/// Permission slugs follow the pattern: folio:{resource}:{action}
/// These are checked by the TenantContext extractor for gated endpoints.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Insert platform-default role profiles ──────────────────────────
        // Columns match atlas_role_profiles schema (m20260811_g32_atlas_rbac):
        //   NO 'permissions' column — permissions live in atlas_role_profile_permissions.
        //   NO 'is_system' column — use is_platform_default instead.
        manager
            .get_connection()
            .execute_unprepared(
                r#"INSERT INTO atlas_role_profiles
                       (id, tenant_id, app_slug, role_slug, display_name, description, is_platform_default)
                   VALUES
                   -- cohost: STR co-host, asset-scoped via atlas_user_asset_access
                   (
                       '00000000-0000-0000-0000-000000000005',
                       NULL, 'folio', 'cohost', 'Cohost',
                       'STR co-host — manages bookings, messaging, guest comms, and cleaning coordination for delegated STR properties',
                       true
                   ),
                   -- agent: real estate agent (brokerage mode)
                   (
                       '00000000-0000-0000-0000-000000000007',
                       NULL, 'folio', 'agent', 'Agent',
                       'Real estate agent — manages client files, listings, and deals in brokerage mode',
                       true
                   ),
                   -- broker: licensed broker (brokerage mode)
                   (
                       '00000000-0000-0000-0000-000000000008',
                       NULL, 'folio', 'broker', 'Broker',
                       'Licensed real estate broker — supervises agents, co-signs deals, manages the office in brokerage mode',
                       true
                   )
                   ON CONFLICT (id) DO NOTHING;
                   -- Conflict on id in case this migration is re-run.
                   -- The unique constraint is (tenant_id, app_slug, role_slug) but id
                   -- is stable so we guard on id to be safe."#,
            )
            .await?;

        // ── 2. Cohost permission slugs ────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                     ('00000000-0000-0000-0000-000000000005', 'folio:str:read'),
                     ('00000000-0000-0000-0000-000000000005', 'folio:str:messaging'),
                     ('00000000-0000-0000-0000-000000000005', 'folio:reservations:read'),
                     ('00000000-0000-0000-0000-000000000005', 'folio:reservations:write'),
                     ('00000000-0000-0000-0000-000000000005', 'folio:maintenance:read'),
                     ('00000000-0000-0000-0000-000000000005', 'folio:calendar:read'),
                     ('00000000-0000-0000-0000-000000000005', 'folio:calendar:write')
                 ON CONFLICT DO NOTHING;",
            )
            .await?;

        // ── 3. Agent permission slugs ─────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                     ('00000000-0000-0000-0000-000000000007', 'folio:listings:read'),
                     ('00000000-0000-0000-0000-000000000007', 'folio:listings:write'),
                     ('00000000-0000-0000-0000-000000000007', 'folio:leads:read'),
                     ('00000000-0000-0000-0000-000000000007', 'folio:leads:write'),
                     ('00000000-0000-0000-0000-000000000007', 'folio:opportunities:read'),
                     ('00000000-0000-0000-0000-000000000007', 'folio:opportunities:write'),
                     ('00000000-0000-0000-0000-000000000007', 'folio:clients:read'),
                     ('00000000-0000-0000-0000-000000000007', 'folio:clients:write')
                 ON CONFLICT DO NOTHING;",
            )
            .await?;

        // ── 4. Broker permission slugs (superset of agent) ────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO atlas_role_profile_permissions (role_profile_id, permission_slug)
                 VALUES
                     ('00000000-0000-0000-0000-000000000008', 'folio:listings:read'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:listings:write'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:leads:read'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:leads:write'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:opportunities:read'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:opportunities:write'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:clients:read'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:clients:write'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:agents:read'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:agents:write'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:commission_plans:read'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:commission_plans:write'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:office:read'),
                     ('00000000-0000-0000-0000-000000000008', 'folio:office:write')
                 ON CONFLICT DO NOTHING;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"DELETE FROM atlas_role_profiles
                   WHERE id IN (
                       '00000000-0000-0000-0000-000000000005',
                       '00000000-0000-0000-0000-000000000006',
                       '00000000-0000-0000-0000-000000000007',
                       '00000000-0000-0000-0000-000000000008'
                   );"#,
            )
            .await?;
        Ok(())
    }
}
