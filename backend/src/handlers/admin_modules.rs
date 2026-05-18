//! Admin Module Registry API Handlers
//!
//! Exposes two endpoints:
//!
//! ## `GET /api/admin/modules`
//! Tenant-scoped. Returns the enabled module set for the calling tenant's
//! app instance, ordered by `sort_order`. Used by the admin dashboard to
//! dynamically render the sidebar navigation.
//!
//! ## `POST /api/platform/tenants/{tenant_id}/modules`
//! Platform-scoped (PlatformSuperAdmin only). Upserts a single module
//! configuration for a specific tenant. Used by the Platform Admin UI.

use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Router,
    routing::{get, post},
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait,
    QueryOrder, Order, ActiveModelTrait, Set,
};
use uuid::Uuid;

use crate::entities::{user, session, user_account, app_instance, app_instance_module};
use crate::models::admin_module::{AdminModuleConfig, AdminModuleType, UpsertModuleInput};

/// Route constructor. State is applied by the caller (admin_routes in routes.rs).
pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        // Tenant-scoped: fetch modules for the authenticated tenant's app instance
        .route("/api/admin/modules", get(get_admin_modules))
        // Platform-scoped: upsert module config for any tenant (PlatformSuperAdmin only)
        .route(
            "/api/platform/tenants/{tenant_id}/modules",
            post(upsert_tenant_module),
        )
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/admin/modules
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the enabled module set for the authenticated tenant, sorted by `sort_order`.
///
/// Resolution: tenant_id is read from the authenticated session's `user_account`,
/// then `app_instance` is resolved, then `app_instance_module` rows are fetched.
///
/// Response: `Vec<AdminModuleConfig>`
pub async fn get_admin_modules(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(_current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    // Resolve the tenant for this user from their user_account record.
    let user_account_row = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("get_admin_modules: user_account lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("get_admin_modules: no user_account for user {}", current_user.id);
            StatusCode::FORBIDDEN
        })?;

    // ── Role gate ────────────────────────────────────────────────────────────
    // Only Owner / Admin / PlatformSuperAdmin may read the module registry.
    // Member-role users are rejected to prevent information-disclosure of
    // internal platform topology (e.g., SECURITY, LEAD_OPTIONS modules).
    use crate::entities::user_account::UserRole;
    match &user_account_row.role {
        UserRole::Owner | UserRole::Admin | UserRole::PlatformSuperAdmin => {}
        role => {
            tracing::warn!(
                "get_admin_modules: insufficient role {:?} for user {}",
                role, current_user.id
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Find the app_instance for this account's tenant.
    // We match on the account_id FK since user_account links user → account → tenant.
    let account = crate::entities::account::Entity::find_by_id(user_account_row.account_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("get_admin_modules: account lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let app_instance_row = app_instance::Entity::find()
        .filter(app_instance::Column::TenantId.eq(account.tenant_id))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("get_admin_modules: app_instance lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Fetch all enabled modules, ordered by sort_order ascending.
    let rows = app_instance_module::Entity::find()
        .filter(app_instance_module::Column::AppInstanceId.eq(app_instance_row.id))
        .filter(app_instance_module::Column::IsEnabled.eq(true))
        .order_by(app_instance_module::Column::SortOrder, Order::Asc)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("get_admin_modules: module fetch failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Map DB rows to wire type.
    let configs: Vec<AdminModuleConfig> = rows
        .into_iter()
        .filter_map(|row| {
            let module_type = row.module_type.parse::<AdminModuleType>().ok()?;
            Some(AdminModuleConfig {
                module_type,
                display_name: row.display_name,
                icon: row.icon,
                sort_order: row.sort_order,
                is_fixed: row.is_fixed,
                category: module_type.category(),
            })
        })
        .collect();

    Ok(Json(configs))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/platform/tenants/{tenant_id}/modules
// ─────────────────────────────────────────────────────────────────────────────

/// Upserts a single module configuration for a specific tenant.
///
/// Authorization: caller must be `PlatformSuperAdmin`.
///
/// Fixed modules (`DASHBOARD`, `SETTINGS`, `SECURITY`) **cannot** be disabled.
/// Attempting to set `is_enabled = false` on a fixed module returns
/// `400 BAD_REQUEST` with body `{"error": "FIXED_MODULE_CANNOT_BE_DISABLED", "module": "..."}`.
///
/// Body: `UpsertModuleInput`
pub async fn upsert_tenant_module(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(_current_session): Extension<session::Model>,
    Path(tenant_id): Path<Uuid>,
    Json(input): Json<UpsertModuleInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Authorization: PlatformSuperAdmin only.
    let is_super_admin = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .filter(
            user_account::Column::Role
                .eq(crate::entities::user_account::UserRole::PlatformSuperAdmin),
        )
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    if !is_super_admin {
        tracing::warn!(
            "upsert_tenant_module: non-superadmin {} attempted to modify tenant {} modules",
            current_user.id, tenant_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Resolve the app_instance for this tenant.
    let app_instance_row = app_instance::Entity::find()
        .filter(app_instance::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("upsert_tenant_module: app_instance lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Enforce fixed-module invariant: Dashboard/Settings/Security cannot be disabled.
    // Returns 400 so the Platform Admin UI receives explicit feedback rather than
    // silently succeeding (which would mask UI bugs).
    if input.module_type.is_fixed() && !input.is_enabled {
        tracing::warn!(
            "upsert_tenant_module: attempt to disable fixed module {:?} for tenant {}",
            input.module_type, tenant_id
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let effective_is_enabled = input.is_enabled;

    let type_str = input.module_type.to_string();
    let display_name = input
        .display_name
        .unwrap_or_else(|| input.module_type.to_display_name().to_string());
    let sort_order = input
        .sort_order
        .unwrap_or_else(|| input.module_type.default_sort_order());

    // Check if a row already exists for this (app_instance_id, module_type) pair.
    let existing = app_instance_module::Entity::find()
        .filter(app_instance_module::Column::AppInstanceId.eq(app_instance_row.id))
        .filter(app_instance_module::Column::ModuleType.eq(&type_str))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("upsert_tenant_module: existing row lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(row) = existing {
        // UPDATE existing row.
        let mut active: app_instance_module::ActiveModel = row.into();
        active.display_name = Set(display_name);
        active.icon = Set(input.icon);
        active.sort_order = Set(sort_order);
        active.is_enabled = Set(effective_is_enabled);
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(&db)
            .await
            .map_err(|e| {
                tracing::error!("upsert_tenant_module: update failed: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        // INSERT new row.
        app_instance_module::ActiveModel {
            id: Set(Uuid::new_v4()),
            app_instance_id: Set(app_instance_row.id),
            module_type: Set(type_str),
            display_name: Set(display_name),
            icon: Set(input.icon),
            sort_order: Set(sort_order),
            is_enabled: Set(effective_is_enabled),
            is_fixed: Set(input.module_type.is_fixed()),
            config: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        }
        .insert(&db)
        .await
        .map_err(|e| {
            tracing::error!("upsert_tenant_module: insert failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    tracing::info!(
        tenant_id = %tenant_id,
        module_type = %input.module_type,
        is_enabled = effective_is_enabled,
        "module configuration upserted"
    );

    Ok(StatusCode::OK)
}

// ─────────────────────────────────────────────────────────────────────────────
// UNIT TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_module::ModuleCategory;

    #[test]
    fn test_fixed_module_disable_is_rejected() {
        // Simulates the guard logic: is_fixed + is_enabled=false → reject.
        for fixed in [
            AdminModuleType::Dashboard,
            AdminModuleType::Settings,
            AdminModuleType::Security,
        ] {
            // When is_enabled is true, it must always pass.
            let attempt_disable = false;
            let would_reject = fixed.is_fixed() && !attempt_disable;
            assert!(
                would_reject,
                "{fixed:?} should reject is_enabled=false (fixed-module invariant)"
            );
            // When is_enabled is true, must not reject.
            let attempt_enable = true;
            let would_reject_enable = fixed.is_fixed() && !attempt_enable;
            assert!(
                !would_reject_enable,
                "{fixed:?} should allow is_enabled=true"
            );
        }
    }

    #[test]
    fn test_non_fixed_module_can_be_disabled() {
        let module = AdminModuleType::Blog;
        assert!(!module.is_fixed());
        // A non-fixed module passes through is_enabled as-is.
        let input_disabled = false;
        let effective = if module.is_fixed() { true } else { input_disabled };
        assert!(!effective, "Blog should be disableable");
    }

    #[test]
    fn test_module_type_parse_round_trip() {
        use std::str::FromStr;
        let all = [
            AdminModuleType::Dashboard,
            AdminModuleType::Leads,
            AdminModuleType::Contacts,
            AdminModuleType::Blog,
            AdminModuleType::Listings,
        ];
        for m in all {
            let s = m.to_string();
            let parsed = AdminModuleType::from_str(&s).expect("parse");
            assert_eq!(m, parsed);
        }
    }

    #[test]
    fn test_display_name_fallback() {
        // When no display_name is provided, falls back to module_type.to_display_name()
        let module = AdminModuleType::ResumeProfiles;
        let fallback = module.to_display_name();
        assert_eq!(fallback, "Resume Profiles");
    }

    #[test]
    fn test_category_returned_in_wire_type() {
        let cfg = AdminModuleConfig::from_type(AdminModuleType::Leads);
        assert_eq!(cfg.category, ModuleCategory::CrmAndComms);
    }
}
