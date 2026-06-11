//! Folio — Appliance handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/assets/{unit_id}/appliances
//!      List all appliances for a unit.
//!      -> 200 [ApplianceDetail]
//!
//! POST /api/folio/assets/{unit_id}/appliances
//!      Register a new appliance on a unit.
//!      Body: CreateApplianceHttpInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/appliances/{id}
//!      Fetch a single appliance with full lifecycle data.
//!      -> 200 ApplianceDetail
//!
//! PATCH /api/folio/appliances/{id}/lifecycle
//!      Update condition, next service date, expiry, or metadata after a service event.
//!      Body: UpdateApplianceLifecycleInput
//!      -> 200 {}
//!
//! GET  /api/folio/appliances/alerts?days=30
//!      All assets (any type) with service due or expiry within horizon.
//!      Query param: days (default 30)
//!      -> 200 [AssetLifecycleAlert]
//! ```

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::appliance::{
    ApplianceService, CreateApplianceInput, UpdateApplianceLifecycleInput,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // Unit-scoped routes
        .route(
            "/api/folio/assets/{unit_id}/appliances",
            get(list_appliances).post(create_appliance),
        )
        // Appliance-scoped routes
        .route("/api/folio/appliances/{id}", get(get_appliance))
        .route(
            "/api/folio/appliances/{id}/lifecycle",
            patch(update_lifecycle),
        )
        // Platform-level alert query (any asset_type)
        .route("/api/folio/appliances/alerts", get(get_alerts))
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AlertQuery {
    /// Alert horizon in days. Default: 30.
    days: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/assets/{unit_id}/appliances
async fn list_appliances(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(unit_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let appliances = ApplianceService::list_for_unit(&db, tenant_id, unit_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %unit_id, "list_appliances error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(appliances))
}

/// POST /api/folio/assets/{unit_id}/appliances
async fn create_appliance(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(unit_id): Path<Uuid>,
    Json(mut input): Json<CreateApplianceInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    input.unit_id = unit_id;

    let id = ApplianceService::create(&db, tenant_id, input)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("required") || msg.contains("must be") {
                tracing::warn!(%tenant_id, "create_appliance validation: {msg}");
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                tracing::error!(%tenant_id, "create_appliance error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(serde_json::json!({ "id": id })),
    ))
}

/// GET /api/folio/appliances/{id}
async fn get_appliance(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(appliance_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let appliance = ApplianceService::get(&db, tenant_id, appliance_id)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %appliance_id, "get_appliance error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(axum::response::Json(appliance))
}

/// PATCH /api/folio/appliances/{id}/lifecycle
async fn update_lifecycle(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(appliance_id): Path<Uuid>,
    Json(input): Json<UpdateApplianceLifecycleInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    ApplianceService::update_lifecycle(&db, tenant_id, appliance_id, input)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %appliance_id, "update_lifecycle error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::OK)
}

/// GET /api/folio/appliances/alerts?days=30
async fn get_alerts(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<AlertQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let horizon = params.days.unwrap_or(30).clamp(1, 365);

    let alerts = ApplianceService::get_lifecycle_alerts(&db, tenant_id, horizon)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "get_alerts error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(alerts))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}
