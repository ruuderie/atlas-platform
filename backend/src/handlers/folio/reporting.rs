//! Folio — Reporting & Analytics handler
//!
//! # Routes
//!
//! ```ignore
//! --- Tenant self-service reports ---
//!
//! GET  /api/folio/tenant/reports
//!      List past report requests by this tenant.
//!      -> 200 [{ id, report_type, status, generated_at }]
//!
//! POST /api/folio/tenant/reports
//!      Request a new report. Body: { report_type: "rental_history" | "payment_history" | ... }
//!      -> 201 { case_id, report: TenantReport }
//!
//! --- Landlord analytics ---
//!
//! GET  /api/folio/analytics/landlord
//!      KPI overview: revenue, occupancy, maintenance load, late payment rate.
//!      -> 200 LandlordOverview
//!
//! --- Vendor analytics ---
//!
//! GET  /api/folio/analytics/vendor/{sp_id}
//!      Work order stats for a specific service provider.
//!      -> 200 VendorOverview
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::services::pm::reporting::{ReportType, ReportingService};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/tenant/reports",
            get(list_reports).post(request_report),
        )
        .route("/api/folio/analytics/landlord", get(landlord_overview))
        .route("/api/folio/analytics/vendor/{sp_id}", get(vendor_overview))
}

// ── HTTP input types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RequestReportInput {
    pub report_type: ReportType,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn list_reports(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match ReportingService::list_report_requests(&db, tenant_id, user_id).await {
        Ok(cases) => {
            // Return a lightweight summary — not the full case model
            let summaries: Vec<serde_json::Value> = cases
                .into_iter()
                .map(|c| {
                    let meta = c.case_metadata.as_ref();
                    serde_json::json!({
                        "id":           c.id,
                        "report_type":  meta.and_then(|m| m["report_type"].as_str()),
                        "status":       c.status,
                        "generated_at": meta.and_then(|m| m["generated_at"].as_str()),
                        "created_at":   c.created_at,
                    })
                })
                .collect();
            (StatusCode::OK, Json(summaries)).into_response()
        }
        Err(e) => {
            tracing::error!("list_reports: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn request_report(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Json(body): Json<RequestReportInput>,
) -> impl IntoResponse {
    match ReportingService::request_report(&db, tenant_id, user_id, body.report_type).await {
        Ok((case_id, report)) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "case_id": case_id,
                "report":  report,
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("request_report: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn landlord_overview(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
) -> impl IntoResponse {
    match ReportingService::landlord_overview(&db, tenant_id).await {
        Ok(overview) => (StatusCode::OK, Json(overview)).into_response(),
        Err(e) => {
            tracing::error!("landlord_overview: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn vendor_overview(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path(sp_id): Path<Uuid>,
) -> impl IntoResponse {
    match ReportingService::vendor_overview(&db, tenant_id, sp_id).await {
        Ok(overview) => (StatusCode::OK, Json(overview)).into_response(),
        Err(e) => {
            tracing::error!("vendor_overview: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
