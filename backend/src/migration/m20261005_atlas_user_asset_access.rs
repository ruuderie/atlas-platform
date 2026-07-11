use sea_orm_migration::prelude::*;

/// Create `atlas_user_asset_access` — per-asset permission grants.
///
/// This table implements fine-grained asset-level scoping: "this user has access
/// to these specific properties". It complements `atlas_user_app_roles`:
///
///   - `atlas_user_app_roles` answers "what role does this user have?"
///   - `atlas_user_asset_access` answers "which specific assets can they see?"
///
/// Semantics (additive / open by default):
///   - 0 rows for a user → no asset restriction (full account access for their role)
///   - N rows for a user → access restricted to those N assets only
///
/// Use cases:
///   - Cohost: gets rows for each STR property they co-manage
///   - Delegate landlord: gets rows for a subset of the portfolio
///   - Vendor: optionally scoped to assets where work orders are assigned
///   - Tenant: NOT stored here — use `atlas_leases.tenant_user_id` instead
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS atlas_user_asset_access (
                    id               UUID        NOT NULL DEFAULT gen_random_uuid()  PRIMARY KEY,
                    user_id          UUID        NOT NULL REFERENCES "user"(id)                   ON DELETE CASCADE,
                    asset_id         UUID        NOT NULL REFERENCES atlas_assets(id)             ON DELETE CASCADE,
                    role_profile_id  UUID        NOT NULL REFERENCES atlas_role_profiles(id),
                    granted_by       UUID                 REFERENCES "user"(id)                   ON DELETE SET NULL,
                    granted_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    expires_at       TIMESTAMPTZ,
                    is_active        BOOLEAN     NOT NULL DEFAULT true,
                    UNIQUE (user_id, asset_id, role_profile_id)
                );

                CREATE INDEX IF NOT EXISTS idx_user_asset_access_user
                    ON atlas_user_asset_access (user_id, is_active);

                CREATE INDEX IF NOT EXISTS idx_user_asset_access_asset
                    ON atlas_user_asset_access (asset_id)
                    WHERE is_active = true;

                CREATE INDEX IF NOT EXISTS idx_user_asset_access_expiry
                    ON atlas_user_asset_access (expires_at)
                    WHERE expires_at IS NOT NULL AND is_active = true;"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS atlas_user_asset_access CASCADE;")
            .await?;
        Ok(())
    }
}
