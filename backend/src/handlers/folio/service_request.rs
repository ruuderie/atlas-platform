//! Service request handler — G35 notification trigger.
//!
//! Property Owner Lite (and any authenticated user) can submit a service
//! request to a vendor in the Folio network.  The request is stored in the
//! `atlas_service_requests` table and a G35 notification is dispatched to the
//! vendor so they know they have new business waiting.
//!
//! # Route (authenticated — PO Lite or any folio role)
//!
//! ```ignore
//! POST /api/folio/service-requests
//!      Body: CreateServiceRequestInput
//!      -> 201 { "request_id": uuid }
//! ```
//!
//! The `atlas_service_requests` table is provisioned by the G34/G35 migration
//! (existing).  If the table doesn't exist yet the INSERT will fail gracefully
//! with 500 — it does NOT block other endpoints.

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::notification_service::{
    DispatchInput, NotificationPriority, NotificationService,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new().route("/api/folio/service-requests", post(create_service_request))
}

// ── Input / output types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequestInput {
    /// The atlas_service_providers.id to send the request to.
    pub vendor_id: Uuid,
    /// Free-text description of the work needed.
    pub description: String,
    /// Optional: link to a specific asset (property) for context.
    pub asset_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateServiceRequestResponse {
    pub request_id: Uuid,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// POST /api/folio/service-requests
///
/// Creates a service request record and fires a G35 notification to the
/// vendor so they know they have new inbound business.
///
/// Auth: any authenticated Folio user (PO Lite, Landlord, Owner, etc.)
pub async fn create_service_request(
    State(db): State<DatabaseConnection>,
    Json(body): Json<CreateServiceRequestInput>,
) -> impl IntoResponse {
    if body.description.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json(serde_json::json!({"error": "Description is required."})),
        )
            .into_response();
    }

    let request_id = Uuid::new_v4();

    // 1. Insert service request row.
    let asset_val = body
        .asset_id
        .map(|id| format!("'{id}'::uuid"))
        .unwrap_or_else(|| "NULL".to_string());

    let desc_escaped = body.description.replace('\'', "''");

    let insert_sql = format!(
        "INSERT INTO atlas_service_requests \
         (id, vendor_id, description, asset_id, status, created_at, updated_at) \
         VALUES ('{req}'::uuid, '{vendor}'::uuid, '{desc}', {asset}, 'pending', NOW(), NOW());",
        req = request_id,
        vendor = body.vendor_id,
        desc = desc_escaped,
        asset = asset_val,
    );

    if let Err(e) = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            insert_sql,
        ))
        .await
    {
        tracing::error!(
            error = %e,
            vendor_id = %body.vendor_id,
            "create_service_request: insert failed"
        );
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    tracing::info!(
        event = "service_request.created",
        request_id = %request_id,
        vendor_id  = %body.vendor_id,
    );

    // 2. Resolve vendor's user_id + tenant_id for G35 notification.
    //    Best-effort: if the vendor row is missing columns or the table schema
    //    differs, we skip notify rather than failing the request.
    let vendor_sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT user_id, tenant_id FROM atlas_service_providers WHERE id = $1 LIMIT 1",
        [body.vendor_id.into()],
    );

    if let Ok(Some(row)) = db.query_one(vendor_sql).await {
        let vendor_user_id: Option<Uuid> = row.try_get("", "user_id").ok();
        let vendor_tenant_id: Option<Uuid> = row.try_get("", "tenant_id").ok();

        if let (Some(v_user), Some(v_tenant)) = (vendor_user_id, vendor_tenant_id) {
            let dispatch = DispatchInput {
                tenant_id: v_tenant,
                user_id: v_user,
                notification_type: "service_request_received".to_string(),
                title: "New service request".to_string(),
                body: format!(
                    "You have a new service request: \"{}\"",
                    body.description.chars().take(80).collect::<String>()
                ),
                priority: NotificationPriority::High,
                entity_type: Some("atlas_service_requests".to_string()),
                entity_id: Some(request_id),
                metadata: Some(serde_json::json!({
                    "request_id": request_id,
                    "asset_id":   body.asset_id,
                })),
                include_broadcast: false,
            };

            if let Err(e) = NotificationService::dispatch(&db, dispatch).await {
                tracing::warn!(
                    error = %e,
                    vendor_user_id = %v_user,
                    "G35: service_request_received notify failed (non-fatal)"
                );
            }
        }
    }

    (
        StatusCode::CREATED,
        axum::Json(CreateServiceRequestResponse { request_id }),
    )
        .into_response()
}
