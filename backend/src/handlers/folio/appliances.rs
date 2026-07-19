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
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::{atlas_asset, user};
use crate::services::pm::appliance::{
    ApplianceService, CreateApplianceInput, UpdateApplianceLifecycleInput,
};
use crate::services::pm::asset_archive::{AssetArchiveService, RetireReason};
use serde::Serialize;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // Portfolio-wide appliance list
        .route("/api/folio/appliances", get(list_all_appliances))
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
        .route("/api/folio/appliances/{id}/retire", post(retire_appliance))
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

#[derive(Debug, Deserialize)]
struct RetireHttpInput {
    pub reason: String,
    pub replaced_by_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct RetireResponse {
    pub id: Uuid,
    pub status: &'static str,
}

/// POST /api/folio/appliances/{id}/retire — status → inactive + replace chain.
async fn retire_appliance(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(appliance_id): Path<Uuid>,
    Json(input): Json<RetireHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let reason = RetireReason::parse(&input.reason).ok_or_else(|| {
        tracing::warn!(%tenant_id, reason = %input.reason, "retire_appliance: bad reason");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    AssetArchiveService::retire(
        &db,
        tenant_id,
        appliance_id,
        "appliance",
        reason,
        input.replaced_by_id,
        input.notes,
    )
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            tracing::error!(%tenant_id, %appliance_id, "retire_appliance: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(axum::response::Json(RetireResponse {
        id: appliance_id,
        status: "inactive",
    }))
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
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
// ── GET /api/folio/appliances — portfolio-wide list ───────────────────────────

async fn list_all_appliances(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, user.id).await {
        Ok(id) => id,
        Err(code) => return Err(code),
    };

    let rows = atlas_asset::Entity::find()
        .filter(atlas_asset::Column::TenantId.eq(tenant_id))
        .filter(atlas_asset::Column::AssetType.eq("appliance"))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_all_appliances db error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let appliances: Vec<crate::services::pm::appliance::ApplianceDetail> = rows
        .into_iter()
        .map(crate::services::pm::appliance::to_detail_pub)
        .collect();

    Ok(axum::response::Json(appliances))
}
