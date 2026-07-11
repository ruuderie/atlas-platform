//! G-32 RBAC Management API — platform-generic role assignment and inspection.
//!
//! # Authorization
//!
//! All write routes (assign, revoke) require the caller to have `rbac:assign`
//! permission in the target app, OR be a tenant Owner/Admin (UserRole layer 1).
//! Read routes require `rbac:read` or Owner/Admin.
//!
//! The guard is applied via `require_rbac_manage()` called at the top of each
//! handler — the same pattern used by `admin_modules.rs`.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/rbac/roles?app_slug=folio               — list profiles (rbac:read or Owner/Admin)
//! GET  /api/rbac/users/{user_id}/roles?app_slug=folio — get user role (rbac:read or Owner/Admin)
//! POST /api/rbac/users/{user_id}/roles                — assign role   (rbac:assign or Owner/Admin)
//! DELETE /api/rbac/users/{user_id}/roles/{app_slug}    — revoke role   (rbac:assign or Owner/Admin)
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user_account::UserRole;
use crate::entities::{user, user_account};
use crate::services::rbac::RbacService;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AppSlugQuery {
    pub app_slug: String,
}

#[derive(Debug, Serialize)]
pub struct RoleProfileSummary {
    pub id: Uuid,
    pub role_slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_platform_default: bool,
}

#[derive(Debug, Serialize)]
pub struct UserRoleResponse {
    pub user_id: Uuid,
    pub app_slug: String,
    pub role_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleInput {
    pub app_slug: String,
    pub role_slug: String,
}

#[derive(Debug, Serialize)]
pub struct AssignRoleResponse {
    pub assignment_id: Uuid,
}

// ── Route constructors ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/rbac/roles", get(list_role_profiles))
        .route(
            "/api/rbac/users/{user_id}/roles",
            get(get_user_role).post(assign_role),
        )
        .route(
            "/api/rbac/users/{user_id}/roles/{app_slug}",
            delete(revoke_role),
        )
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/rbac/roles?app_slug=folio
async fn list_role_profiles(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<AppSlugQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, user_account_row) = resolve_caller(&db, current_user.id).await?;
    require_rbac_permission(
        &db,
        &user_account_row,
        current_user.id,
        tenant_id,
        &params.app_slug,
        "rbac:read",
    )
    .await?;

    let profiles = RbacService::list_role_profiles(&db, tenant_id, &params.app_slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<RoleProfileSummary> = profiles
        .into_iter()
        .map(|p| RoleProfileSummary {
            id: p.id,
            role_slug: p.role_slug,
            display_name: p.display_name,
            description: p.description,
            is_platform_default: p.is_platform_default,
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/rbac/users/{user_id}/roles?app_slug=folio
async fn get_user_role(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<AppSlugQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, user_account_row) = resolve_caller(&db, current_user.id).await?;
    require_rbac_permission(
        &db,
        &user_account_row,
        current_user.id,
        tenant_id,
        &params.app_slug,
        "rbac:read",
    )
    .await?;

    let role_slug = RbacService::get_user_app_role(&db, user_id, tenant_id, &params.app_slug).await;
    Ok(Json(UserRoleResponse {
        user_id,
        app_slug: params.app_slug,
        role_slug,
    }))
}

/// POST /api/rbac/users/{user_id}/roles
/// Body: { "app_slug": "folio", "role_slug": "tenant" }
async fn assign_role(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(user_id): Path<Uuid>,
    Json(input): Json<AssignRoleInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, user_account_row) = resolve_caller(&db, current_user.id).await?;
    require_rbac_permission(
        &db,
        &user_account_row,
        current_user.id,
        tenant_id,
        &input.app_slug,
        "rbac:assign",
    )
    .await?;

    let assignment_id = RbacService::assign_role(
        &db,
        user_id,
        tenant_id,
        &input.app_slug,
        &input.role_slug,
        Some(current_user.id),
    )
    .await
    .map_err(|e| {
        tracing::warn!("rbac::assign_role failed: {e}");
        StatusCode::BAD_REQUEST
    })?;

    tracing::info!(
        assigner = %current_user.id, target_user = %user_id,
        app_slug = %input.app_slug, role_slug = %input.role_slug,
        %assignment_id, "rbac: role assigned"
    );

    Ok(Json(AssignRoleResponse { assignment_id }))
}

/// DELETE /api/rbac/users/{user_id}/roles/{app_slug}
async fn revoke_role(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((user_id, app_slug)): Path<(Uuid, String)>,
) -> Result<StatusCode, StatusCode> {
    let (tenant_id, user_account_row) = resolve_caller(&db, current_user.id).await?;
    require_rbac_permission(
        &db,
        &user_account_row,
        current_user.id,
        tenant_id,
        &app_slug,
        "rbac:assign",
    )
    .await?;

    let affected = RbacService::revoke_role(&db, user_id, tenant_id, &app_slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!(
        revoker = %current_user.id, target_user = %user_id,
        %app_slug, rows_affected = affected, "rbac: role revoked"
    );

    if affected == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

// ── Authorization helpers ─────────────────────────────────────────────────────

/// Resolve the calling user's tenant_id and user_account row in one pass.
async fn resolve_caller(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(Uuid, user_account::Model), StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts.iter().map(|ua| ua.account_id).collect();

    // Use the first active user_account for the UserRole check (Layer 1)
    let ua = user_accounts
        .into_iter()
        .next()
        .ok_or(StatusCode::FORBIDDEN)?;

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok((profile.tenant_id, ua))
}

/// Dual-layer RBAC guard:
///   Layer 1 (platform UserRole): Owner, Admin, PlatformSuperAdmin pass immediately.
///   Layer 2 (app permission):    has_permission(user, tenant, app, permission_slug).
///
/// A tenant Member who has been explicitly granted `rbac:assign` via their app
/// profile or Permission Set override will also pass — this enables delegated
/// admin patterns without promoting them to Owner/Admin on the whole platform.
async fn require_rbac_permission(
    db: &DatabaseConnection,
    ua: &user_account::Model,
    user_id: Uuid,
    tenant_id: Uuid,
    app_slug: &str,
    permission_slug: &str,
) -> Result<(), StatusCode> {
    // Layer 1: platform role bypass
    match &ua.role {
        UserRole::Owner | UserRole::Admin | UserRole::PlatformSuperAdmin => return Ok(()),
        UserRole::Member => {} // fall through to Layer 2
    }

    // Layer 2: app-level permission check (profile + Permission Set overrides)
    if RbacService::has_permission(db, user_id, tenant_id, app_slug, permission_slug).await {
        return Ok(());
    }

    tracing::warn!(
        %user_id, %tenant_id, %app_slug, %permission_slug,
        "rbac: access denied — insufficient role/permission"
    );
    Err(StatusCode::FORBIDDEN)
}
