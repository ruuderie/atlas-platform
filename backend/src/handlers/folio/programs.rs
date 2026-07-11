//! G-36 Folio program handlers — /api/folio/programs
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | /api/folio/programs | List programs (?kind=&actor_role=) |
//! | POST | /api/folio/programs/{id}/actions | Create NetworkInvite action |
//! | GET | /api/folio/programs/actions/mine | Actor's actions |

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde::Deserialize;
use uuid::Uuid;

use crate::services::program_service::ProgramService;
use crate::types::pm::ProgramKind;

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

async fn resolve_caller_id(db: &DatabaseConnection, token: &str) -> Option<Uuid> {
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT user_id FROM sessions WHERE bearer_token_hash = encode(sha256($1::bytea), 'hex') \
         AND expires_at > now() LIMIT 1",
        [token.into()],
    );
    db.query_one(stmt).await.ok()??.try_get("", "user_id").ok()
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/programs", get(list_programs))
        .route("/api/folio/programs/actions/mine", get(list_my_actions))
        .route("/api/folio/programs/{id}/actions", post(create_action))
}

#[derive(Debug, Deserialize)]
pub struct ListProgramsQuery {
    pub kind: Option<ProgramKind>,
    pub actor_role: Option<String>,
    pub app_instance_id: Option<Uuid>,
}

async fn list_programs(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
    Query(q): Query<ListProgramsQuery>,
) -> impl IntoResponse {
    if extract_bearer(&headers).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response();
    }
    let result = if let Some(app_instance_id) = q.app_instance_id {
        ProgramService::list_programs_for_instance(
            &db,
            app_instance_id,
            q.kind,
            q.actor_role.as_deref(),
        )
        .await
        .map(|rows| serde_json::json!({ "programs": rows }))
    } else {
        let kind = q.kind.as_ref().map(|k| k.to_string());
        ProgramService::list_programs(&db, kind.as_deref(), q.actor_role.as_deref())
            .await
            .map(|rows| serde_json::json!({ "programs": rows }))
    };

    match result {
        Ok(body) => (StatusCode::OK, Json(body)).into_response(),
        Err(e) => {
            tracing::error!("list_programs: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateActionBody {
    pub target_email: String,
    pub target_role: String,
    pub personal_note: Option<String>,
    pub tenant_id: Option<Uuid>,
}

async fn create_action(
    Path(program_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
    Json(body): Json<CreateActionBody>,
) -> impl IntoResponse {
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Authentication required"})),
            )
                .into_response();
        }
    };
    let caller_id = match resolve_caller_id(&db, &token).await {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid session"})),
            )
                .into_response();
        }
    };

    if body.target_email.trim().is_empty() || !body.target_email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Valid target_email required"})),
        )
            .into_response();
    }

    match ProgramService::create_network_invite_action(
        &db,
        program_id,
        caller_id,
        body.tenant_id,
        body.target_email.trim().to_string(),
        body.target_role.trim().to_string(),
        body.personal_note,
    )
    .await
    {
        Ok(action) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "action": action,
                "join_url": action.invite_code.as_ref().map(|c| format!("/join/{c}")),
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("create_action: {e}");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
                .into_response()
        }
    }
}

async fn list_my_actions(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Authentication required"})),
            )
                .into_response();
        }
    };
    let caller_id = match resolve_caller_id(&db, &token).await {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid session"})),
            )
                .into_response();
        }
    };

    match ProgramService::list_actions_for_actor(&db, caller_id).await {
        Ok(rows) => (StatusCode::OK, Json(serde_json::json!({ "actions": rows }))).into_response(),
        Err(e) => {
            tracing::error!("list_my_actions: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
                .into_response()
        }
    }
}
