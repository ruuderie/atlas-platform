//! G-36 platform-admin Programs API.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::services::program_service::{
    ProgramCreateInput, ProgramService, ProgramUpdatePatch, RewardRuleInput,
};
use crate::types::pm::ProgramKind;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/admin/programs",
            get(list_programs).post(create_program),
        )
        .route(
            "/api/admin/programs/{id}",
            get(get_program).patch(update_program),
        )
        .route(
            "/api/admin/programs/{id}/reward-rules",
            get(list_reward_rules).put(replace_reward_rules),
        )
        .route(
            "/api/admin/programs/{id}/actions",
            get(list_program_actions),
        )
        .route("/api/admin/programs/{id}/grants", get(list_program_grants))
        .route("/api/admin/programs/{id}/analytics", get(program_analytics))
        .route(
            "/api/admin/programs/{id}/instance-enablements",
            get(list_instance_enablements).put(set_instance_enablements),
        )
        .route(
            "/api/admin/app-instances/{instance_id}/programs",
            get(list_instance_programs),
        )
}

#[derive(Debug, Deserialize)]
struct ListProgramsQuery {
    kind: Option<ProgramKind>,
    include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ListActionsQuery {
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ListInstanceProgramsQuery {
    kind: Option<ProgramKind>,
    actor_role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SetEnablementItem {
    app_instance_id: Uuid,
    is_enabled: bool,
}

async fn list_programs(
    State(db): State<DatabaseConnection>,
    Query(q): Query<ListProgramsQuery>,
) -> impl IntoResponse {
    match ProgramService::list_programs_admin(&db, q.kind, q.include_inactive.unwrap_or(false))
        .await
    {
        Ok(programs) => ok(serde_json::json!({ "programs": programs })),
        Err(e) => server_error("list_programs_admin", e),
    }
}

async fn create_program(
    State(db): State<DatabaseConnection>,
    Json(input): Json<ProgramCreateInput>,
) -> impl IntoResponse {
    match ProgramService::create_program(&db, input).await {
        Ok(program) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "program": program })),
        )
            .into_response(),
        Err(e) => bad_request(e),
    }
}

async fn get_program(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    match ProgramService::get_program(&db, id).await {
        Ok(Some(program)) => ok(serde_json::json!({ "program": program })),
        Ok(None) => not_found("Program not found"),
        Err(e) => server_error("get_program", e),
    }
}

async fn update_program(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(patch): Json<ProgramUpdatePatch>,
) -> impl IntoResponse {
    match ProgramService::update_program(&db, id, patch).await {
        Ok(Some(program)) => ok(serde_json::json!({ "program": program })),
        Ok(None) => not_found("Program not found"),
        Err(e) => bad_request(e),
    }
}

async fn list_reward_rules(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    match ProgramService::list_reward_rules(&db, id).await {
        Ok(reward_rules) => ok(serde_json::json!({ "reward_rules": reward_rules })),
        Err(e) => server_error("list_reward_rules", e),
    }
}

async fn replace_reward_rules(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(rules): Json<Vec<RewardRuleInput>>,
) -> impl IntoResponse {
    match ProgramService::upsert_reward_rules(&db, id, rules).await {
        Ok(reward_rules) => ok(serde_json::json!({ "reward_rules": reward_rules })),
        Err(e) => bad_request(e),
    }
}

async fn list_program_actions(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Query(q): Query<ListActionsQuery>,
) -> impl IntoResponse {
    match ProgramService::list_actions_for_program(&db, id, q.limit.unwrap_or(100)).await {
        Ok(actions) => ok(serde_json::json!({ "actions": actions })),
        Err(e) => server_error("list_program_actions", e),
    }
}

async fn list_program_grants(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    match ProgramService::list_grants_for_program(&db, id).await {
        Ok(grants) => ok(serde_json::json!({ "grants": grants })),
        Err(e) => server_error("list_program_grants", e),
    }
}

async fn program_analytics(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    match ProgramService::program_analytics(&db, id).await {
        Ok(analytics) => ok(serde_json::json!({ "analytics": analytics })),
        Err(e) => server_error("program_analytics", e),
    }
}

async fn list_instance_enablements(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    match ProgramService::list_instance_enablements(&db, id).await {
        Ok(enablements) => ok(serde_json::json!({ "instance_enablements": enablements })),
        Err(e) => server_error("list_instance_enablements", e),
    }
}

async fn set_instance_enablements(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(items): Json<Vec<SetEnablementItem>>,
) -> impl IntoResponse {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match ProgramService::set_instance_enablement(
            &db,
            id,
            item.app_instance_id,
            item.is_enabled,
        )
        .await
        {
            Ok(row) => out.push(row),
            Err(e) => return server_error("set_instance_enablement", e),
        }
    }
    ok(serde_json::json!({ "instance_enablements": out }))
}

async fn list_instance_programs(
    Path(instance_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Query(q): Query<ListInstanceProgramsQuery>,
) -> impl IntoResponse {
    match ProgramService::list_programs_for_instance(
        &db,
        instance_id,
        q.kind,
        q.actor_role.as_deref(),
    )
    .await
    {
        Ok(programs) => ok(serde_json::json!({ "programs": programs })),
        Err(e) => server_error("list_instance_programs", e),
    }
}

fn ok(value: serde_json::Value) -> axum::response::Response {
    (StatusCode::OK, Json(value)).into_response()
}

fn bad_request(message: impl ToString) -> axum::response::Response {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": message.to_string() })),
    )
        .into_response()
}

fn not_found(message: &'static str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}

fn server_error(context: &str, error: impl std::fmt::Display) -> axum::response::Response {
    tracing::error!("{context}: {error}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Database error" })),
    )
        .into_response()
}
