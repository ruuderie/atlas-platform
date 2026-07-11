use sea_orm_migration::prelude::*;

/// G-32 backfill: migrate existing `user_account.folio_role` values into
/// `atlas_user_app_roles` so all users have proper RBAC assignments before
/// the `folio_role` column is dropped.
///
/// Maps:
///   folio_role = 'landlord'  → role_profile_id = '00000000-0000-0000-0001-000000000001'
///   folio_role = 'tenant'    → role_profile_id = '00000000-0000-0000-0001-000000000002'
///   folio_role = 'vendor'    → role_profile_id = '00000000-0000-0000-0001-000000000003'
///
/// Fully idempotent via ON CONFLICT DO NOTHING.
/// Requires m20260811 (G-32 schema) and m20260812 (Folio seed) to have run first.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"INSERT INTO atlas_user_app_roles
                       (user_id, tenant_id, app_slug, role_profile_id, granted_at, is_active)
                   SELECT
                       ua.user_id,
                       a.tenant_id,
                       'folio',
                       CASE ua.folio_role
                           WHEN 'landlord' THEN '00000000-0000-0000-0001-000000000001'::uuid
                           WHEN 'tenant'   THEN '00000000-0000-0000-0001-000000000002'::uuid
                           WHEN 'vendor'   THEN '00000000-0000-0000-0001-000000000003'::uuid
                           ELSE                 '00000000-0000-0000-0001-000000000001'::uuid
                       END,
                       NOW(),
                       true
                   FROM user_account ua
                   JOIN account a ON ua.account_id = a.id
                   WHERE ua.folio_role IS NOT NULL
                     AND a.tenant_id   IS NOT NULL
                   ON CONFLICT (user_id, tenant_id, app_slug) DO NOTHING;"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove all folio role assignments (re-running up() will re-create them)
        manager
            .get_connection()
            .execute_unprepared("DELETE FROM atlas_user_app_roles WHERE app_slug = 'folio';")
            .await?;
        Ok(())
    }
}
