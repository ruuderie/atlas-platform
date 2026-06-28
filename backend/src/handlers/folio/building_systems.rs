//! Folio — Building System handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/systems
//!      List all building systems across the landlord's entire portfolio.
//!      -> 200 [BuildingSystemDetail]
//!
//! GET  /api/folio/assets/{property_id}/systems
//!      List all building systems for a property.
//!      -> 200 [BuildingSystemDetail]
//!
//! POST /api/folio/assets/{property_id}/systems
//!      Register a new building system (elevator, roof, HVAC, etc.).
//!      Body: CreateBuildingSystemInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/systems/{id}
//!      Fetch a single building system with full lifecycle data.
//!      -> 200 BuildingSystemDetail
//!
//! PATCH /api/folio/systems/{id}/lifecycle
//!      Update condition, next inspection date, cert expiry, or metadata.
//!      Body: UpdateBuildingSystemLifecycleInput
//!      -> 200 {}
//!
//! GET  /api/folio/lifecycle/alerts?days=30
//!      All assets (appliances + building systems) with service due or expiry
//!      within the alert horizon. Single query, all asset_types.
//!      Query param: days (default 30, max 365)
//!      -> 200 [LifecycleAlert]
//! ```

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::{atlas_asset, user};
use crate::services::pm::building_system::{
    BuildingSystemService, CreateBuildingSystemInput, UpdateBuildingSystemLifecycleInput,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // Portfolio-wide building system list
        .route("/api/folio/systems", get(list_all_systems))
        // Property-scoped building system routes
        .route(
            "/api/folio/assets/{property_id}/systems",
            get(list_systems).post(create_system),
        )
        // Building system instance routes
        .route("/api/folio/systems/{id}", get(get_system))
        .route("/api/folio/systems/{id}/lifecycle", patch(update_lifecycle))
        // Combined lifecycle alert query (all asset_types — appliances + systems)
        .route("/api/folio/lifecycle/alerts", get(get_lifecycle_alerts))
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AlertQuery {
    /// Alert horizon in days. Default: 30. Clamped to [1, 365].
    days: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/assets/{property_id}/systems
async fn list_systems(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(property_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let systems = BuildingSystemService::list_for_property(&db, tenant_id, property_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %property_id, "list_systems error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(systems))
}

/// POST /api/folio/assets/{property_id}/systems
async fn create_system(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(property_id): Path<Uuid>,
    Json(mut input): Json<CreateBuildingSystemInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    input.property_id = property_id;

    let id = BuildingSystemService::create(&db, tenant_id, input)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("required") || msg.contains("must be") {
                tracing::warn!(%tenant_id, "create_system validation: {msg}");
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                tracing::error!(%tenant_id, "create_system error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(serde_json::json!({ "id": id })),
    ))
}

/// GET /api/folio/systems/{id}
async fn get_system(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(system_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let system = BuildingSystemService::get(&db, tenant_id, system_id)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %system_id, "get_system error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(axum::response::Json(system))
}

/// PATCH /api/folio/systems/{id}/lifecycle
async fn update_lifecycle(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(system_id): Path<Uuid>,
    Json(input): Json<UpdateBuildingSystemLifecycleInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    BuildingSystemService::update_lifecycle(&db, tenant_id, system_id, input)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %system_id, "update_system_lifecycle error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::OK)
}

/// GET /api/folio/lifecycle/alerts?days=30
///
/// Combined alert query — returns ALL asset types (appliances + building systems)
/// with service due or cert/warranty expiry within the horizon.
/// Sorted: overdue first (negative days), then soonest.
async fn get_lifecycle_alerts(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<AlertQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let horizon = params.days.unwrap_or(30).clamp(1, 365);

    let alerts = BuildingSystemService::get_lifecycle_alerts(&db, tenant_id, horizon)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "get_lifecycle_alerts error: {e:#}");
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

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}

// ── GET /api/folio/systems — portfolio-wide list ──────────────────────────────

async fn list_all_systems(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, user.id).await {
        Ok(id) => id,
        Err(code) => return Err(code),
    };

    let rows = atlas_asset::Entity::find()
        .filter(atlas_asset::Column::TenantId.eq(tenant_id))
        .filter(atlas_asset::Column::AssetType.eq("building_system"))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_all_systems db error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let systems: Vec<crate::services::pm::building_system::BuildingSystemDetail> =
        rows.into_iter()
            .map(crate::services::pm::building_system::to_detail_pub)
            .collect();

    Ok(axum::response::Json(systems))
}
