//! G-32 RBAC Management API — platform-generic role assignment and inspection.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/rbac/roles?app_slug=folio
//!      List role profiles available to the current tenant for an app.
//!      -> 200 [RoleProfileSummary]
//!
//! GET  /api/rbac/users/:user_id/roles?app_slug=folio
//!      Get a specific user's role assignments.
//!      -> 200 UserRoleResponse
//!
//! POST /api/rbac/users/:user_id/roles
//!      Assign a role to a user for an app within the current tenant.
//!      -> 200 { "assignment_id": uuid }
//!
//! DELETE /api/rbac/users/:user_id/roles/:app_slug
//!        Revoke a user's role in an app (soft delete).
//!        -> 204
//! ```

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::rbac::RbacService;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListRolesQuery {
    pub app_slug: String,
}

#[derive(Debug, Serialize)]
pub struct RoleProfileSummary {
    pub id:                  Uuid,
    pub role_slug:           String,
    pub display_name:        String,
    pub description:         Option<String>,
    pub is_platform_default: bool,
}

#[derive(Debug, Serialize)]
pub struct UserRoleResponse {
    pub user_id:   Uuid,
    pub app_slug:  String,
    pub role_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleInput {
    pub app_slug:  String,
    pub role_slug: String,
}

#[derive(Debug, Serialize)]
pub struct AssignRoleResponse {
    pub assignment_id: Uuid,
}

// ── Route constructors ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/rbac/roles",                           get(list_role_profiles))
        .route("/api/rbac/users/:user_id/roles",            get(get_user_role).post(assign_role))
        .route("/api/rbac/users/:user_id/roles/:app_slug",  delete(revoke_role))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/rbac/roles?app_slug=folio
async fn list_role_profiles(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<ListRolesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let profiles = RbacService::list_role_profiles(&db, tenant_id, &params.app_slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<RoleProfileSummary> = profiles
        .into_iter()
        .map(|p| RoleProfileSummary {
            id:                  p.id,
            role_slug:           p.role_slug,
            display_name:        p.display_name,
            description:         p.description,
            is_platform_default: p.is_platform_default,
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/rbac/users/:user_id/roles?app_slug=folio
async fn get_user_role(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<ListRolesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let role_slug = RbacService::get_user_app_role(&db, user_id, tenant_id, &params.app_slug).await;

    Ok(Json(UserRoleResponse { user_id, app_slug: params.app_slug, role_slug }))
}

/// POST /api/rbac/users/:user_id/roles
/// Body: { "app_slug": "folio", "role_slug": "tenant" }
async fn assign_role(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(user_id): Path<Uuid>,
    Json(input): Json<AssignRoleInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

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

    Ok(Json(AssignRoleResponse { assignment_id }))
}

/// DELETE /api/rbac/users/:user_id/roles/:app_slug
async fn revoke_role(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((user_id, app_slug)): Path<(Uuid, String)>,
) -> Result<StatusCode, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let affected = RbacService::revoke_role(&db, user_id, tenant_id, &app_slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if affected == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}

// ── Shared helper ─────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}
