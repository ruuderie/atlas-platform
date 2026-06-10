use sea_orm_migration::prelude::*;

/// Folio: Add `client_account_id` to `atlas_user_app_roles`.
///
/// # Purpose
///
/// When a PMC (Property Management Company) invites a landlord to manage one
/// specific client account, the resulting G-32 role assignment needs to carry
/// which client book that Landlord user is scoped to.
///
/// # NULL semantics (backward compatible)
///
/// - NULL → org-level role (all existing rows — no behavioral change)
/// - UUID → this Landlord user's access is limited to the specified client account.
///
/// # Query impact
///
/// `RequireFolioRole` extractor reads this column and populates
/// `TenantContext.client_account_id`. Service-layer queries that scope by
/// `managed_account_id` can use this field directly without extra lookups.
///
/// # Constraint note
///
/// Unique constraint on `atlas_user_app_roles` is `(user_id, tenant_id, app_slug)`.
/// A user may hold at most one role per (tenant, app). A PMC client landlord is
/// assigned to exactly one client book — if they need access to more, the PM
/// creates additional accounts or promotes them to a PMC role.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_user_app_roles
                     ADD COLUMN IF NOT EXISTS client_account_id UUID
                         REFERENCES account(id) ON DELETE SET NULL;

                 -- Partial index: only relevant for PMC-scoped assignments.
                 -- No overhead for standard (NULL) rows.
                 CREATE INDEX IF NOT EXISTS idx_user_app_roles_client_account
                     ON atlas_user_app_roles(client_account_id)
                     WHERE client_account_id IS NOT NULL;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_user_app_roles_client_account;
                 ALTER TABLE atlas_user_app_roles DROP COLUMN IF EXISTS client_account_id;",
            )
            .await?;
        Ok(())
    }
}
