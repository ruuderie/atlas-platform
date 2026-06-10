//! Folio — Owner Portal handler
//!
//! Read-only routes for beneficial property owners. All routes are protected
//! by `require_owner` middleware (FolioRole::Owner). Zero write operations
//! are exposed here — owners can only view.
//!
//! # Routes
//!
//! ```ignore
//! GET /api/folio/owner/summary
//!     Top-level KPI: properties, occupancy, revenue, outstanding, maintenance.
//!     -> 200 OwnerPortfolioSummary
//!
//! GET /api/folio/owner/properties
//!     Per-property breakdown (revenue, leases, open cases).
//!     -> 200 [OwnerPropertySummary]
//!
//! GET /api/folio/owner/leases
//!     All active and past leases across owned properties.
//!     -> 200 [OwnerLeaseEntry]
//!
//! GET /api/folio/owner/maintenance
//!     All open maintenance cases across owned properties.
//!     -> 200 [OwnerMaintenanceSummary]
//!
//! GET /api/folio/owner/inspections
//!     All scheduled + completed inspections.
//!     -> 200 [OwnerInspectionEntry]
//!
//! --- PMC-only write (called during client onboarding) ---
//!
//! POST /api/folio/pm/owner-links
//!      Link an owner user account to an asset they own.
//!      Body: { owner_user_id, asset_id }
//!      -> 201 { rel_id }
//! ```

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::services::pm::owner::OwnerService;

// ── Route registration ────────────────────────────────────────────────────────

/// Read-only owner routes — gated by `require_owner` middleware in folio.rs.
pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/owner/summary",     get(owner_summary))
        .route("/api/folio/owner/properties",  get(owner_properties))
        .route("/api/folio/owner/leases",      get(owner_leases))
        .route("/api/folio/owner/maintenance", get(owner_maintenance))
        .route("/api/folio/owner/inspections", get(owner_inspections))
}

/// PMC-only write route — gated by `require_landlord` (PM operators are Landlord-role
/// in the PMC client context). Registered in the landlord_router in folio.rs.
pub fn pmc_write_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/pm/owner-links", post(link_owner_to_asset))
}

// ── HTTP input types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LinkOwnerInput {
    pub owner_user_id: Uuid,
    pub asset_id: Uuid,
}

// ── Owner read handlers ───────────────────────────────────────────────────────

async fn owner_summary(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match OwnerService::portfolio_summary(&db, tenant_id, user_id).await {
        Ok(s) => (StatusCode::OK, Json(s)).into_response(),
        Err(e) => {
            tracing::error!("owner_summary: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn owner_properties(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match OwnerService::list_properties(&db, tenant_id, user_id).await {
        Ok(p) => (StatusCode::OK, Json(p)).into_response(),
        Err(e) => {
            tracing::error!("owner_properties: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn owner_leases(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match OwnerService::list_leases(&db, tenant_id, user_id).await {
        Ok(l) => (StatusCode::OK, Json(l)).into_response(),
        Err(e) => {
            tracing::error!("owner_leases: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn owner_maintenance(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match OwnerService::list_maintenance(&db, tenant_id, user_id).await {
        Ok(m) => (StatusCode::OK, Json(m)).into_response(),
        Err(e) => {
            tracing::error!("owner_maintenance: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn owner_inspections(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match OwnerService::list_inspections(&db, tenant_id, user_id).await {
        Ok(i) => (StatusCode::OK, Json(i)).into_response(),
        Err(e) => {
            tracing::error!("owner_inspections: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── PMC write handler ─────────────────────────────────────────────────────────

async fn link_owner_to_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Json(body): Json<LinkOwnerInput>,
) -> impl IntoResponse {
    match OwnerService::link_owner_to_asset(
        &db, tenant_id, user_id, body.owner_user_id, body.asset_id,
    ).await {
        Ok(rel_id) => (StatusCode::CREATED, Json(serde_json::json!({ "rel_id": rel_id }))).into_response(),
        Err(e) => {
            tracing::error!("link_owner_to_asset: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
