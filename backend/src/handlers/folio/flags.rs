//! Folio — Feature flags resolution
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | /api/folio/flags?app_instance_id= | Enabled flag keys for an app instance |

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::app_instance;
use crate::services::flag_service::FlagService;

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            let val = val.trim();
            if val.starts_with("Bearer ") {
                return Some(val["Bearer ".len()..].trim().to_string());
            }
        }
    }
    if let Some(cookie) = headers.get("cookie") {
        if let Ok(val) = cookie.to_str() {
            for part in val.split(';') {
                let part = part.trim();
                if part.starts_with("session=") {
                    return Some(part["session=".len()..].trim().to_string());
                }
                if part.starts_with("atlas_session=") {
                    return Some(part["atlas_session=".len()..].trim().to_string());
                }
            }
        }
    }
    None
}

async fn session_is_valid(db: &DatabaseConnection, token: &str) -> bool {
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT 1 FROM sessions WHERE bearer_token_hash = encode(sha256($1::bytea), 'hex') \
         AND expires_at > now() LIMIT 1",
        [token.into()],
    );
    matches!(db.query_one(stmt).await, Ok(Some(_)))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new().route("/api/folio/flags", get(list_enabled_flags))
}

#[derive(Debug, Deserialize)]
pub struct ListFlagsQuery {
    pub app_instance_id: Option<Uuid>,
}

async fn list_enabled_flags(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
    Query(q): Query<ListFlagsQuery>,
) -> impl IntoResponse {
    let Some(token) = extract_bearer(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response();
    };

    if !session_is_valid(&db, &token).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid session"})),
        )
            .into_response();
    }

    let Some(instance_id) = q.app_instance_id else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "app_instance_id is required"})),
        )
            .into_response();
    };

    let tenant_id = match app_instance::Entity::find_by_id(instance_id).one(&db).await {
        Ok(Some(inst)) => inst.tenant_id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "App instance not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("list_enabled_flags instance lookup: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
                .into_response();
        }
    };

    match FlagService::list_enabled_keys(&db, tenant_id, Some(instance_id)).await {
        Ok(keys) => (StatusCode::OK, Json(serde_json::json!({ "flags": keys }))).into_response(),
        Err(e) => {
            tracing::error!("list_enabled_flags: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
                .into_response()
        }
    }
}
