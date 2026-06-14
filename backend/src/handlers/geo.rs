//! # G01 Geo HTTP handlers — Platform Generic
//!
//! Exposes PostGIS spatial queries as REST endpoints at the platform level.
//!
//! ## Routes (all authenticated)
//!
//! | Method | Path                                | Description                              |
//! |--------|-------------------------------------|------------------------------------------|
//! | GET    | /api/geo/leads/radius               | Leads within radius of a point           |
//! | GET    | /api/geo/leads/nearest              | N nearest leads to a point               |
//! | POST   | /api/geo/leads/{id}/geocode         | Set geo_point on a lead                  |
//! | GET    | /api/geo/accounts/radius            | Accounts within radius of a point        |
//! | POST   | /api/geo/accounts/{id}/geocode      | Set geo_point on an account              |
//! | GET    | /api/geo/service-areas              | List service areas (with optional owner) |
//! | GET    | /api/geo/service-areas/contains     | Service areas containing a point         |
//! | GET    | /api/geo/service-areas/radius       | Service areas near a point               |
//! | GET    | /api/geo/status                     | PostGIS availability health check        |

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::geo_service::GeoService;

// ── Route constructor ─────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/geo/status", get(geo_status))
        .route("/api/geo/leads/radius", get(leads_radius))
        .route("/api/geo/leads/nearest", get(leads_nearest))
        .route("/api/geo/leads/{id}/geocode", post(geocode_lead))
        .route("/api/geo/accounts/radius", get(accounts_radius))
        .route("/api/geo/accounts/{id}/geocode", post(geocode_account))
        .route("/api/geo/service-areas", get(get_service_areas))
        .route("/api/geo/service-areas/contains", get(service_areas_contains))
        .route("/api/geo/service-areas/radius", get(service_areas_radius))
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

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PointRadiusQuery {
    /// Longitude (WGS-84). GeoJSON convention: lng before lat.
    lng: f64,
    /// Latitude (WGS-84).
    lat: f64,
    /// Search radius in metres. Defaults to 10 000 m (10 km).
    #[serde(default = "default_radius")]
    radius_m: f64,
}

#[derive(Debug, Deserialize)]
struct PointQuery {
    lng: f64,
    lat: f64,
}

#[derive(Debug, Deserialize)]
struct NearestQuery {
    lng: f64,
    lat: f64,
    /// Maximum results. Defaults to 20, capped at 100.
    #[serde(default = "default_limit")]
    limit: u32,
}

#[derive(Debug, Deserialize)]
struct GeocodeInput {
    /// Longitude (WGS-84).
    lng: f64,
    /// Latitude (WGS-84).
    lat: f64,
}

#[derive(Debug, Deserialize)]
struct ServiceAreasQuery {
    owner_entity_type: Option<String>,
    owner_entity_id: Option<Uuid>,
}

fn default_radius() -> f64 { 10_000.0 }
fn default_limit() -> u32  { 20 }

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct GeoStatusResponse {
    postgis_available: bool,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/geo/status — PostGIS availability check.
async fn geo_status(
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let available = GeoService::check_postgis(&db).await;
    Json(GeoStatusResponse { postgis_available: available })
}

/// GET /api/geo/leads/radius?lng=&lat=&radius_m=
async fn leads_radius(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<PointRadiusQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let radius_m = q.radius_m.clamp(1.0, 500_000.0); // cap at 500 km
    let leads = GeoService::leads_within_radius(&db, tenant_id, q.lng, q.lat, radius_m)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "geo/leads/radius failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(leads))
}

/// GET /api/geo/leads/nearest?lng=&lat=&limit=
async fn leads_nearest(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<NearestQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let limit = q.limit.clamp(1, 100);
    let leads = GeoService::nearest_leads(&db, tenant_id, q.lng, q.lat, limit)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "geo/leads/nearest failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(leads))
}

/// POST /api/geo/leads/{id}/geocode — Set geo_point on a lead.
async fn geocode_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(lead_id): Path<Uuid>,
    Json(body): Json<GeocodeInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Verify the lead belongs to this tenant before updating.
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    crate::entities::atlas_lead::Entity::find_by_id(lead_id)
        .filter(crate::entities::atlas_lead::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    GeoService::set_lead_geo_point(&db, lead_id, body.lng, body.lat)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "geo/leads/geocode failed");
            if e.to_string().contains("PostGIS not available") {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/geo/accounts/radius?lng=&lat=&radius_m=
async fn accounts_radius(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<PointRadiusQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let radius_m = q.radius_m.clamp(1.0, 500_000.0);
    let accounts = GeoService::accounts_within_radius(&db, tenant_id, q.lng, q.lat, radius_m)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "geo/accounts/radius failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(accounts))
}

/// POST /api/geo/accounts/{id}/geocode — Set geo_point on an account.
async fn geocode_account(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(account_id): Path<Uuid>,
    Json(body): Json<GeocodeInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    // Tenant-scope check: verify the account belongs to this tenant.
    crate::entities::account::Entity::find_by_id(account_id)
        .filter(crate::entities::account::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    GeoService::set_account_geo_point(&db, account_id, body.lng, body.lat)
        .await
        .map_err(|e| {
            if e.to_string().contains("PostGIS not available") {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/geo/service-areas
async fn get_service_areas(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ServiceAreasQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let areas = GeoService::get_service_areas(&db, tenant_id, q.owner_entity_type, q.owner_entity_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "geo/service-areas list failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(areas))
}

/// GET /api/geo/service-areas/contains?lng=&lat=
async fn service_areas_contains(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<PointQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let areas = GeoService::service_areas_containing_point(&db, tenant_id, q.lng, q.lat)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "geo/service-areas/contains failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(areas))
}

/// GET /api/geo/service-areas/radius?lng=&lat=&radius_m=
async fn service_areas_radius(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<PointRadiusQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let radius_m = q.radius_m.clamp(1.0, 500_000.0);
    let areas = GeoService::service_areas_within_radius(&db, tenant_id, q.lng, q.lat, radius_m)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "geo/service-areas/radius failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(areas))
}
