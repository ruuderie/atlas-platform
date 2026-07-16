//! # Admin Integration Webhooks
//!
//! Webhook endpoints for external service integrations (direct mail providers, etc.)

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use sea_orm::DatabaseConnection;
use serde_json::Value;

use crate::services::pm::direct_mail::resolve_direct_mail_provider;

/// POST /api/admin/integrations/dm/:provider/webhook
/// Generic webhook receiver for direct mail providers.
pub async fn direct_mail_webhook(
    State(_db): State<DatabaseConnection>,
    Path(provider): Path<String>,
    Json(payload): Json<Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Resolve the provider
    let dm_provider = resolve_direct_mail_provider(&provider)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Unknown direct mail provider: {}", provider)))?;

    // Try to parse the webhook payload
    match dm_provider.parse_webhook(&payload) {
        Ok(_events) => {
            // TODO: Process the events (update mail drop status, record costs, etc.)
            tracing::info!(
                provider = %provider,
                "direct_mail_webhook: received webhook (processing not implemented)"
            );
            Ok(StatusCode::OK)
        }
        Err(crate::services::pm::direct_mail::DirectMailError::NotImplemented(_)) => {
            // Provider explicitly returns NotImplemented - return 501
            Ok(StatusCode::NOT_IMPLEMENTED)
        }
        Err(e) => {
            tracing::warn!(
                provider = %provider,
                error = %e,
                "direct_mail_webhook: failed to parse payload"
            );
            Err((StatusCode::BAD_REQUEST, e.to_string()))
        }
    }
}

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new().route(
        "/api/admin/integrations/dm/{provider}/webhook",
        post(direct_mail_webhook),
    )
}