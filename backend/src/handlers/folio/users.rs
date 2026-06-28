//! Folio — User profile lookup handler.
//!
//! Allows a landlord to look up a user who appears as a
//! `counterparty_user_id` on one of their contracts. Returns only
//! non-sensitive identity fields (no password hash, no session data).
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/users/:id
//!      Fetch basic identity for a counterparty user.
//!      Authorization: caller must have at least one atlas_contract where
//!        tenant_id = caller.tenant_id AND counterparty_user_id = :id
//!      -> 200 CounterpartyUser
//!      -> 403 if user is not a counterparty on any of caller's contracts
//!      -> 404 if user does not exist
//! ```

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use crate::entities::user;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/users/{id}", get(get_counterparty_user))
}

// ── Response types ────────────────────────────────────────────────────────────

/// Non-sensitive identity fields safe to expose to the landlord.
#[derive(Debug, Serialize)]
pub struct CounterpartyUser {
    pub id:         Uuid,
    pub first_name: String,
    pub last_name:  String,
    pub email:      String,
    pub phone:      String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Resolve the caller's tenant_id from user_account → account.
async fn resolve_tenant_id(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts
        .into_iter()
        .map(|ua| ua.account_id)
        .collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// GET /api/folio/users/:id
///
/// Returns basic identity for a counterparty user. The caller must be a
/// landlord (or PMC) who has at least one contract where counterparty_user_id
/// equals the requested user_id. This prevents arbitrary user enumeration.
async fn get_counterparty_user(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(target_user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // ── Authorization: ensure target is a counterparty on one of our contracts ──
    let contract = crate::entities::atlas_contract::Entity::find()
        .filter(crate::entities::atlas_contract::Column::TenantId.eq(tenant_id))
        .filter(
            crate::entities::atlas_contract::Column::CounterpartyUserId
                .eq(target_user_id),
        )
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %target_user_id, "get_counterparty_user: contract lookup error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if contract.is_none() {
        // User exists but is not a counterparty on any of our contracts — 403,
        // not 404, to avoid leaking whether the user_id exists in the system.
        return Err(StatusCode::FORBIDDEN);
    }

    // ── Fetch user ────────────────────────────────────────────────────────────
    let target = user::Entity::find_by_id(target_user_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%target_user_id, "get_counterparty_user: user lookup error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(axum::response::Json(CounterpartyUser {
        id:         target.id,
        first_name: target.first_name,
        last_name:  target.last_name,
        email:      target.email,
        phone:      target.phone,
        created_at: target.created_at,
    }))
}
