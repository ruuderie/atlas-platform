//! # G25 Commission HTTP handlers — Folio
//!
//! Routes:
//!
//! | Method | Path                                           | Description                              |
//! |--------|------------------------------------------------|------------------------------------------|
//! | GET    | /api/folio/commission-plans                    | List commission plans (active_only opt.) |
//! | GET    | /api/folio/commission-plans/{id}               | Get single plan + splits                 |
//! | POST   | /api/folio/commission-plans/{id}/compute       | Compute splits for a transaction amount  |

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::commission::CommissionService,
};

// ── Response types ────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct PlanDetailResponse {
    plan: crate::entities::atlas_commission_plan::Model,
    splits: Vec<crate::entities::atlas_commission_plan_split::Model>,
}

// ── Route constructor ─────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/commission-plans", get(list_plans))
        .route("/api/folio/commission-plans/{id}", get(get_plan))
        .route(
            "/api/folio/commission-plans/{id}/compute",
            post(compute_commission),
        )
}

// ── Tenant resolution ─────────────────────────────────────────────────────────

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

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListPlansQuery {
    active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ComputeInput {
    /// The gross transaction amount in cents.
    transaction_amount_cents: i64,
    /// Costs to deduct before net-percentage basis computation. Defaults to 0.
    deduction_cents: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn list_plans(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListPlansQuery>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let plans = CommissionService::list_plans(&db, tenant_id, q.active_only.unwrap_or(false))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(plans))
}

async fn get_plan(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let plan = CommissionService::get_plan(&db, tenant_id, id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let splits = CommissionService::get_splits(&db, tenant_id, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PlanDetailResponse { plan, splits }))
}

async fn compute_commission(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<ComputeInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let result = CommissionService::compute(
        &db,
        tenant_id,
        id,
        body.transaction_amount_cents,
        body.deduction_cents.unwrap_or(0),
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(result))
}
