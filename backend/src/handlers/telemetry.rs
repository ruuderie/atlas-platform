use axum::{
    extract::{State, Json, Extension},
    routing::post,
    Router,
    http::StatusCode,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::auth::Claims;
use crate::services::telemetry::TelemetryService;
use uuid::Uuid;

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/v1/telemetry/events", post(ingest_telemetry_events))
}

#[derive(Debug, Deserialize)]
pub struct TelemetryIngestPayload {
    pub events: Vec<TelemetryEventInput>,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryEventInput {
    pub event_source: String,
    pub event_type: String,
    pub payload: Option<Value>,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub success: bool,
    pub events_queued: usize,
}

pub async fn ingest_telemetry_events(
    State(db): State<DatabaseConnection>,
    Extension(_claims): Extension<Claims>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TelemetryIngestPayload>,
) -> Result<axum::Json<IngestResponse>, (StatusCode, String)> {
    // Extract tenant_id from X-Tenant-Id header
    let tenant_id_str = headers.get("X-Tenant-Id").and_then(|v| v.to_str().ok()).ok_or_else(|| {
        (StatusCode::FORBIDDEN, "Missing X-Tenant-Id header context".to_string())
    })?;
    
    let tenant_id = Uuid::parse_str(&tenant_id_str).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid tenant UUID".to_string())
    })?;

    let event_count = payload.events.len();

    for event in payload.events {
        // Enforce application namespaces: If an app tries to claim it is 'platform' we reject or override.
        let safe_source = if event.event_source.starts_with("app:") {
            event.event_source
        } else {
            format!("app:{}", event.event_source)
        };

        TelemetryService::log_event(
            db.clone(),
            tenant_id,
            safe_source,
            event.event_type,
            event.payload,
        );
    }

    Ok(axum::Json(IngestResponse {
        success: true,
        events_queued: event_count,
    }))
}
