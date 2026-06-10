//! Folio — Violations handler
//!
//! # Routes
//!
//! ```ignore
//! --- Landlord routes ---
//!
//! POST /api/folio/violations
//!      File a violation against a tenant's unit.
//!      Body: FileViolationHttpInput
//!      -> 201 ViolationRecord
//!
//! GET  /api/folio/assets/:asset_id/violations
//!      All violations on a specific unit/property.
//!      -> 200 [ViolationRecord]
//!
//! PATCH /api/folio/violations/:id/status
//!       Transition cure status (open → cured/escalated/dismissed).
//!       Body: UpdateCureStatusInput
//!       -> 200 ViolationRecord
//!
//! --- Tenant routes (read-only, own violations only) ---
//!
//! GET  /api/folio/tenant/violations
//!      All violations on the tenant's own leases.
//!      -> 200 [ViolationRecord]
//! ```

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use chrono::NaiveDate;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::services::pm::violation::{
    CureStatus, FileViolationInput, ViolationCategory, ViolationService,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/violations", post(file_violation))
        .route("/api/folio/violations/:id/status", patch(update_cure_status))
        .route("/api/folio/assets/:asset_id/violations", get(list_for_asset))
        .route("/api/folio/tenant/violations", get(list_for_tenant))
}

// ── HTTP input types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FileViolationHttpInput {
    pub asset_id: Uuid,
    /// Set for LTR violations (lease FK).
    pub contract_id: Option<Uuid>,
    /// Set for STR violations (booking FK). Use with `UnauthorizedParty` / `OverOccupancy` categories.
    pub reservation_id: Option<Uuid>,
    pub category: ViolationCategory,
    pub subject: String,
    pub description: String,
    /// Number of days to cure. Common: 3 (noise), 10 (unauthorized pet), 30 (property damage).
    /// Leave `null` for STR violations (guest is departing — no cure period).
    pub cure_days: Option<u8>,
    pub evidence_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCureStatusHttpInput {
    pub status: CureStatus,
    pub resolution_notes: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn file_violation(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Json(body): Json<FileViolationHttpInput>,
) -> impl IntoResponse {
    let input = FileViolationInput {
        asset_id: body.asset_id,
        contract_id: body.contract_id,
        reservation_id: body.reservation_id,
        category: body.category,
        subject: body.subject,
        description: body.description,
        cure_days: body.cure_days,
        evidence_notes: body.evidence_notes,
    };

    match ViolationService::file_violation(&db, tenant_id, user_id, input).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(e) => {
            tracing::error!("file_violation: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn update_cure_status(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path(violation_id): Path<Uuid>,
    Json(body): Json<UpdateCureStatusHttpInput>,
) -> impl IntoResponse {
    match ViolationService::update_cure_status(
        &db, tenant_id, violation_id, body.status, body.resolution_notes,
    ).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => {
            tracing::error!("update_cure_status: {e:#}");
            // Status transition errors are 422, not 500
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

async fn list_for_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path(asset_id): Path<Uuid>,
) -> impl IntoResponse {
    match ViolationService::list_for_asset(&db, tenant_id, asset_id).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => {
            tracing::error!("list_violations_for_asset: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn list_for_tenant(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match ViolationService::list_for_tenant(&db, tenant_id, user_id).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => {
            tracing::error!("list_violations_for_tenant: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
