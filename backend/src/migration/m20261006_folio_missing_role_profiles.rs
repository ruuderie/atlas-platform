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
        manager
            .get_connection()
            .execute_unprepared(
                r#"INSERT INTO atlas_role_profiles
                       (id, tenant_id, app_slug, role_slug, display_name, description, permissions, is_system, created_at)
                   VALUES
                   -- cohost: STR co-host, asset-scoped via atlas_user_asset_access
                   (
                       '00000000-0000-0000-0000-000000000005',
                       NULL,
                       'folio',
                       'cohost',
                       'Cohost',
                       'STR co-host — manages bookings, messaging, guest comms, and cleaning coordination for delegated STR properties',
                       '["folio:str:read","folio:str:messaging","folio:reservations:read","folio:reservations:write","folio:maintenance:read","folio:calendar:read","folio:calendar:write"]'::jsonb,
                       true,
                       NOW()
                   ),
                   -- NOTE: '00000000-0000-0000-0000-000000000006' (str_host) intentionally omitted.
                   -- STR capability is a property trait (atlas_assets.str_eligible), not a persona.
                   -- agent: real estate agent (brokerage mode)
                   (
                       '00000000-0000-0000-0000-000000000007',
                       NULL,
                       'folio',
                       'agent',
                       'Agent',
                       'Real estate agent — manages client files, listings, and deals in brokerage mode',
                       '["folio:listings:read","folio:listings:write","folio:leads:read","folio:leads:write","folio:opportunities:read","folio:opportunities:write","folio:clients:read","folio:clients:write"]'::jsonb,
                       true,
                       NOW()
                   ),
                   -- broker: licensed broker (brokerage mode)
                   (
                       '00000000-0000-0000-0000-000000000008',
                       NULL,
                       'folio',
                       'broker',
                       'Broker',
                       'Licensed real estate broker — supervises agents, co-signs deals, manages the office in brokerage mode',
                       '["folio:listings:read","folio:listings:write","folio:leads:read","folio:leads:write","folio:opportunities:read","folio:opportunities:write","folio:clients:read","folio:clients:write","folio:agents:read","folio:agents:write","folio:commission_plans:read","folio:commission_plans:write","folio:office:read","folio:office:write"]'::jsonb,
                       true,
                       NOW()
                   )
                   ON CONFLICT (id) DO NOTHING;"#,
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
