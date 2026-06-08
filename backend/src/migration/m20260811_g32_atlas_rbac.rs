use sea_orm_migration::prelude::*;

/// G-32: atlas_rbac — Platform-generic Role/Profile/Assignment system.
///
/// Three tables that implement the Salesforce App→Profile→Role→User hierarchy
/// across all Atlas apps (Folio, Anchor, and future apps):
///
///   atlas_role_profiles            — named role templates per app (platform or tenant-scoped)
///   atlas_user_app_roles           — user↔role assignment per app+tenant
///   atlas_role_profile_permissions — permission slugs owned by each app's role templates
///
/// Design principles:
///   - `atlas_role_profiles` with `is_platform_default = true` are seeded by
///     each AtlasApp::provision() and are available to all tenants.
///   - A tenant can create custom role profiles (is_platform_default = false,
///     tenant_id set) to override or extend platform defaults.
///   - `atlas_user_app_roles` has a unique constraint on (user_id, tenant_id, app_slug)
///     enforcing one active role per user per app per tenant (isolation per instance).
///   - The existing `user_app_permission.permissions` JSONB stays as the
///     "Permission Set" layer — additive user-specific overrides on top of a profile.
///
/// Does NOT implement role hierarchy (parent_role_profile_id) — deferred to G-32 v2.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. atlas_role_profiles ────────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS atlas_role_profiles (
                    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id           UUID REFERENCES tenant(id) ON DELETE CASCADE,
                    app_slug            VARCHAR(50)  NOT NULL,
                    role_slug           VARCHAR(50)  NOT NULL,
                    display_name        VARCHAR(100) NOT NULL,
                    description         TEXT,
                    is_platform_default BOOLEAN      NOT NULL DEFAULT false,
                    metadata            JSONB        NOT NULL DEFAULT '{}',
                    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
                    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
                    CONSTRAINT uq_role_profiles_tenant_app_slug
                        UNIQUE (tenant_id, app_slug, role_slug),
                    CONSTRAINT chk_platform_profile_tenant
                        CHECK (is_platform_default = false OR tenant_id IS NULL)
                );
                CREATE INDEX IF NOT EXISTS idx_role_profiles_app
                    ON atlas_role_profiles (app_slug);
                CREATE INDEX IF NOT EXISTS idx_role_profiles_platform
                    ON atlas_role_profiles (app_slug, role_slug)
                    WHERE is_platform_default = true;",
            )
            .await?;

        // ── 2. atlas_user_app_roles ───────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS atlas_user_app_roles (
                    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    user_id           UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
                    tenant_id         UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                    app_slug          VARCHAR(50) NOT NULL,
                    role_profile_id   UUID NOT NULL REFERENCES atlas_role_profiles(id),
                    granted_by        UUID REFERENCES "user"(id) ON DELETE SET NULL,
                    granted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    expires_at        TIMESTAMPTZ,
                    is_active         BOOLEAN NOT NULL DEFAULT true,
                    CONSTRAINT uq_user_app_roles_user_tenant_app
                        UNIQUE (user_id, tenant_id, app_slug)
                );
                CREATE INDEX IF NOT EXISTS idx_user_app_roles_user
                    ON atlas_user_app_roles (user_id);
                CREATE INDEX IF NOT EXISTS idx_user_app_roles_tenant_app
                    ON atlas_user_app_roles (tenant_id, app_slug);
                CREATE INDEX IF NOT EXISTS idx_user_app_roles_active
                    ON atlas_user_app_roles (user_id, tenant_id, app_slug)
                    WHERE is_active = true;"#,
            )
            .await?;

        // ── 3. atlas_role_profile_permissions ─────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS atlas_role_profile_permissions (
                    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    role_profile_id   UUID NOT NULL
                        REFERENCES atlas_role_profiles(id) ON DELETE CASCADE,
                    permission_slug   VARCHAR(100) NOT NULL,
                    is_allowed        BOOLEAN NOT NULL DEFAULT true,
                    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    CONSTRAINT uq_role_permission_slug
                        UNIQUE (role_profile_id, permission_slug)
                );
                CREATE INDEX IF NOT EXISTS idx_role_permissions_profile
                    ON atlas_role_profile_permissions (role_profile_id)
                    WHERE is_allowed = true;",
            )
            .await?;

        // ── 4. updated_at trigger on atlas_role_profiles ──────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_role_profiles
                        BEFORE UPDATE ON atlas_role_profiles
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
                "DROP TRIGGER IF EXISTS set_updated_at_role_profiles ON atlas_role_profiles;
                 DROP TABLE IF EXISTS atlas_role_profile_permissions;
                 DROP TABLE IF EXISTS atlas_user_app_roles;
                 DROP TABLE IF EXISTS atlas_role_profiles;",
            )
            .await?;
        Ok(())
    }
}
