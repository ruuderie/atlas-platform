use sea_orm_migration::prelude::*;

/// Creates a platform-level sentinel `tenant` and `account` row whose IDs are
/// the nil UUID (00000000-0000-0000-0000-000000000000).
///
/// ## Purpose
///
/// `PlatformSuperAdmin` is a role that exists at the platform level, independent
/// of any real tenant or account. When `toggle_admin` grants this role to a user
/// who has no existing `user_account` record, it must still satisfy the
/// `user_account.account_id REFERENCES account(id)` FK constraint.
///
/// Using `Uuid::nil()` as a well-known sentinel is preferable to:
/// - Picking an arbitrary existing account (cross-tenant data corruption — N1)
/// - Dropping the FK (schema integrity regression)
/// - Creating a real account per-admin-grant (semantic confusion)
///
/// ## Schema note
///
/// The sentinel rows carry a `name` of `__platform__` and `site_status` of
/// `inactive` so they are trivially filterable from real data. The tenant row
/// also uses nil for its own ID, which means all references to it are obviously
/// synthetic.
///
/// ## Idempotency
///
/// All INSERTs use `ON CONFLICT DO NOTHING` so the migration is safe to
/// run against a database that already has the sentinel rows.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Insert the platform sentinel tenant (nil UUID).
        //    Uses raw SQL because SeaORM's ActiveModel requires the entity to be
        //    in scope, and this migration must remain self-contained.
        db.execute_unprepared(
            r#"
            INSERT INTO tenant (
                id,
                name,
                description,
                site_status,
                created_at,
                updated_at
            )
            VALUES (
                '00000000-0000-0000-0000-000000000000',
                '__platform__',
                'Platform-level sentinel tenant. Not a real tenant. Do not delete.',
                'inactive',
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP
            )
            ON CONFLICT (id) DO NOTHING;
            "#,
        )
        .await?;

        // 2. Insert the platform sentinel account (nil UUID) referencing the
        //    sentinel tenant.
        db.execute_unprepared(
            r#"
            INSERT INTO account (
                id,
                tenant_id,
                name,
                is_active,
                created_at,
                updated_at
            )
            VALUES (
                '00000000-0000-0000-0000-000000000000',
                '00000000-0000-0000-0000-000000000000',
                '__platform__',
                false,
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP
            )
            ON CONFLICT (id) DO NOTHING;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove the sentinel account first (FK order).
        db.execute_unprepared(
            "DELETE FROM account WHERE id = '00000000-0000-0000-0000-000000000000';",
        )
        .await?;

        db.execute_unprepared(
            "DELETE FROM tenant WHERE id = '00000000-0000-0000-0000-000000000000';",
        )
        .await?;

        Ok(())
    }
}
