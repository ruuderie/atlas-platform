//! G-32 RBAC Service — platform-generic role and permission resolution.
//!
//! This service is the single authoritative source for answering:
//!   - "What role does this user have in this app?"
//!   - "Does this user have permission X in this app?"
//!   - "What permissions does a given role have?"
//!   - "Assign / revoke a role for a user in an app."
//!
//! All callers (API handlers, server fns, middleware) should use this service
//! rather than querying `atlas_user_app_roles` directly.
//!
//! # Permission Set layer
//!
//! `user_app_permission.permissions` (JSONB array) acts as the "Permission Set"
//! layer on top of profile-level permissions — additive, user-specific overrides.
//! `has_permission()` checks profile permissions first, then the override layer.

use sea_orm::{
    ColumnTrait, DatabaseConnection,
    DbBackend, EntityTrait, QueryFilter, Statement,
};
use uuid::Uuid;

use crate::entities::{atlas_role_profiles, atlas_user_app_roles};

pub struct RbacService;

impl RbacService {
    // ── Role resolution ───────────────────────────────────────────────────────

    /// Resolve a user's `role_slug` for a given app+tenant.
    /// Returns `None` if the user has no active role in the app.
    pub async fn get_user_app_role(
        db:       &DatabaseConnection,
        user_id:  Uuid,
        tenant_id: Uuid,
        app_slug: &str,
    ) -> Option<String> {
        use sea_orm::ConnectionTrait;

        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT rp.role_slug
                   FROM atlas_user_app_roles uar
                   JOIN atlas_role_profiles rp ON uar.role_profile_id = rp.id
                   WHERE uar.user_id   = $1
                     AND uar.tenant_id = $2
                     AND uar.app_slug  = $3
                     AND uar.is_active = true
                     AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
                   LIMIT 1"#,
                [user_id.into(), tenant_id.into(), app_slug.into()],
            ))
            .await
            .ok()??;

        row.try_get::<String>("", "role_slug").ok()
    }

    // ── Permission check ──────────────────────────────────────────────────────

    /// Returns true if the user has the given `permission_slug` in the app,
    /// either via their role profile OR via a `user_app_permission` override.
    ///
    /// Slug matching supports two forms:
    ///   - Exact:    `"billing:read"`  matches stored slug `"billing:read"`
    ///   - Wildcard: stored slug `"billing:*"` matches any `"billing:*"` query
    ///
    /// Wildcard evaluation is done in Rust after fetching, not in SQL, to keep
    /// the query simple and avoid regex injection surface.
    pub async fn has_permission(
        db:             &DatabaseConnection,
        user_id:        Uuid,
        tenant_id:      Uuid,
        app_slug:       &str,
        permission_slug: &str,
    ) -> bool {
        use sea_orm::ConnectionTrait;

        // Derive the namespace prefix for wildcard matching: "billing:read" → "billing:"
        let slug_prefix = permission_slug
            .split_once(':')
            .map(|(ns, _)| format!("{}:*", ns));

        // Layer 1: profile-level permission (exact match OR wildcard slug stored in DB)
        // We fetch all permission slugs for the user's profile and evaluate in Rust.
        let profile_slugs: Vec<String> = {
            let rows = db
                .query_all(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT rpp.permission_slug
                       FROM atlas_user_app_roles uar
                       JOIN atlas_role_profile_permissions rpp
                         ON rpp.role_profile_id = uar.role_profile_id
                       WHERE uar.user_id   = $1
                         AND uar.tenant_id = $2
                         AND uar.app_slug  = $3
                         AND uar.is_active = true
                         AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
                         AND rpp.is_allowed = true"#,
                    [
                        user_id.into(),
                        tenant_id.into(),
                        app_slug.into(),
                    ],
                ))
                .await
                .unwrap_or_default();
            rows.into_iter()
                .filter_map(|r| r.try_get::<String>("", "permission_slug").ok())
                .collect()
        };

        let profile_match = profile_slugs.iter().any(|stored| {
            stored == permission_slug
                || slug_prefix.as_deref().map_or(false, |pfx| stored == pfx)
        });

        if profile_match {
            return true;
        }

        // Layer 2: user_app_permission override (Permission Set layer)
        let override_check = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT 1
                   FROM user_app_permission
                   WHERE user_id   = $1
                     AND tenant_id = $2
                     AND app_slug  = $3
                     AND permissions @> $4::jsonb
                   LIMIT 1"#,
                [
                    user_id.into(),
                    tenant_id.into(),
                    app_slug.into(),
                    serde_json::json!([permission_slug]).to_string().into(),
                ],
            ))
            .await
            .ok()
            .flatten();

        override_check.is_some()
    }

    // ── Role assignment ───────────────────────────────────────────────────────

    /// Assign a role profile to a user for an app+tenant. Idempotent upsert:
    /// if the user already has a role in this app, it is replaced.
    ///
    /// `role_slug` must match an existing `atlas_role_profiles.role_slug` for
    /// the given `app_slug` (platform-default or tenant-scoped).
    pub async fn assign_role(
        db:         &DatabaseConnection,
        user_id:    Uuid,
        tenant_id:  Uuid,
        app_slug:   &str,
        role_slug:  &str,
        granted_by: Option<Uuid>,
    ) -> Result<Uuid, sea_orm::DbErr> {
        use sea_orm::ConnectionTrait;

        // Resolve profile — prefer tenant-scoped over platform-default.
        // Raw SQL avoids the order_by_desc ambiguity on nullable tenant_id.
        let profile_row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT id FROM atlas_role_profiles
                   WHERE app_slug  = $1
                     AND role_slug = $2
                     AND (tenant_id = $3 OR is_platform_default = true)
                   ORDER BY (tenant_id = $3) DESC NULLS LAST
                   LIMIT 1"#,
                [app_slug.into(), role_slug.into(), tenant_id.into()],
            ))
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom(
                format!("Role profile '{role_slug}' not found for app '{app_slug}'")
            ))?;

        let profile_id: uuid::Uuid = profile_row
            .try_get("", "id")
            .map_err(|e| sea_orm::DbErr::Custom(format!("profile id parse: {e}")))?;

        let assignment_id = Uuid::new_v4();

        // Fix: granted_by must be passed as Option<Uuid> sea_orm Value, not as a string.
        // Passing None as an empty string caused a type mismatch on the UUID column.
        let granted_by_value: sea_orm::Value = match granted_by {
            Some(uid) => uid.into(),
            None      => sea_orm::Value::Uuid(None),
        };

        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO atlas_user_app_roles
                   (id, user_id, tenant_id, app_slug, role_profile_id, granted_by, granted_at, is_active)
               VALUES ($1, $2, $3, $4, $5, $6, NOW(), true)
               ON CONFLICT (user_id, tenant_id, app_slug)
               DO UPDATE SET
                   role_profile_id = EXCLUDED.role_profile_id,
                   granted_by      = EXCLUDED.granted_by,
                   granted_at      = NOW(),
                   expires_at      = NULL,
                   is_active       = true"#,
            [
                assignment_id.into(),
                user_id.into(),
                tenant_id.into(),
                app_slug.into(),
                profile_id.into(),
                granted_by_value,
            ],
        ))
        .await?;

        Ok(assignment_id)
    }

    /// Revoke a user's role in an app (soft-delete: sets is_active = false).
    pub async fn revoke_role(
        db:        &DatabaseConnection,
        user_id:   Uuid,
        tenant_id: Uuid,
        app_slug:  &str,
    ) -> Result<u64, sea_orm::DbErr> {
        use sea_orm::ConnectionTrait;

        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE atlas_user_app_roles
                    SET is_active = false
                  WHERE user_id   = $1
                    AND tenant_id = $2
                    AND app_slug  = $3
                    AND is_active = true",
                [user_id.into(), tenant_id.into(), app_slug.into()],
            ))
            .await?;

        Ok(result.rows_affected())
    }

    // ── Profile listing ───────────────────────────────────────────────────────

    /// List all role profiles available to a tenant for an app (platform defaults
    /// + any tenant-scoped custom profiles).
    pub async fn list_role_profiles(
        db:        &DatabaseConnection,
        tenant_id: Uuid,
        app_slug:  &str,
    ) -> Result<Vec<atlas_role_profiles::Model>, sea_orm::DbErr> {
        atlas_role_profiles::Entity::find()
            .filter(atlas_role_profiles::Column::AppSlug.eq(app_slug))
            .filter(
                sea_orm::Condition::any()
                    .add(atlas_role_profiles::Column::IsPlatformDefault.eq(true))
                    .add(atlas_role_profiles::Column::TenantId.eq(tenant_id)),
            )
            .all(db)
            .await
    }
}
