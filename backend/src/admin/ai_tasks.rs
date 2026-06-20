//! Admin — AI Tasks handler
//!
//! Manages background AI tasks registry (G-08) for asynchronous LLM/AI processing.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_ai_task, user};

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/ai-tasks", get(list_ai_tasks))
        .route("/api/admin/ai-tasks/{id}/abort", axum::routing::post(abort_ai_task))
        .route("/api/admin/ai-tasks/{id}/rerun", axum::routing::post(rerun_ai_task))
}

// ── Response models ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminAiTaskResponse {
    pub id: String,
    pub task_type: String,
    pub entity: String,
    pub status: String,
    pub status_class: String,
    pub runtime: String,
    pub tokens: String,
    pub completed: String,
    pub model: String,
    pub params: serde_json::Value,
    pub initial_logs: Vec<String>,
    pub streamable: bool,
}

// ── Helper to format logs dynamically from DB record state ───────────────────

fn generate_task_logs(task: &atlas_ai_task::Model) -> Vec<String> {
    let mut logs = vec![
        format!("[INFO] Task registered in queue at {}", task.queued_at.format("%H:%M:%S")),
        format!("[INFO] Task Type: {}, Target Engine: {}", task.task_type, task.model.as_deref().unwrap_or("gpt-4o-mini")),
    ];

    if let Some(started) = task.started_at {
        logs.push(format!("[INFO] Execution started at {}", started.format("%H:%M:%S")));
        logs.push("[INFO] Resolving context entity bindings...".to_string());
    }

    match task.status.as_str() {
        "queued" | "Pending" => {
            logs.push("[QUEUE] Waiting in pool atlas-llm-pool-04...".to_string());
        }
        "running" | "Running" => {
            logs.push("[INFO] Streaming context vectors to model...".to_string());
            logs.push("[INFO] Model token pipeline warming up...".to_string());
        }
        "success" | "Success" => {
            if let Some(completed) = task.completed_at {
                logs.push(format!("[INFO] Response vectors generated at {}", completed.format("%H:%M:%S")));
            }
            logs.push(format!(
                "[SUCCESS] LLM tokens consumed: input={}, output={}",
                task.input_tokens.unwrap_or(0),
                task.output_tokens.unwrap_or(0)
            ));
            logs.push("[INFO] Committing output payload to state DB context.".to_string());
            logs.push("[SUCCESS] Task execution completed cleanly.".to_string());
        }
        "failed" | "Failed" => {
            if let Some(err) = &task.error_message {
                logs.push(format!("[ERROR] Pipeline Execution Error: {}", err));
            } else {
                logs.push("[ERROR] Task failed with unknown pipeline exit code.".to_string());
            }
            logs.push("[WARNING] Job cancelled. Outbox rescheduled according to policy.".to_string());
        }
        _ => {}
    }

    logs
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_ai_tasks(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let list = atlas_ai_task::Entity::find()
        .order_by_desc(atlas_ai_task::Column::QueuedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch AI tasks: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<AdminAiTaskResponse> = list
        .into_iter()
        .map(|m| {
            let status = match m.status.as_str() {
                "queued" => "Queued".to_string(),
                "running" => "Running".to_string(),
                "success" => "Success".to_string(),
                "failed" => "Failed".to_string(),
                other => other.to_string(),
            };

            let status_class = match status.as_str() {
                "Running" => "bg-blue-500/10 border-blue-500/30 text-blue-400",
                "Queued" => "bg-slate-500/10 border-slate-500/30 text-slate-400",
                "Success" => "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
                "Failed" => "bg-red-500/10 border-red-500/30 text-red-400",
                _ => "bg-slate-500/10 border-slate-500/30 text-slate-400",
            };

            let runtime = match (m.started_at, m.completed_at) {
                (Some(s), Some(c)) => {
                    let diff = c.signed_duration_since(s);
                    format!("{:.2}s", diff.num_milliseconds() as f64 / 1000.0)
                }
                (Some(s), None) => {
                    let diff = Utc::now().signed_duration_since(s);
                    format!("{:.2}s", diff.num_milliseconds() as f64 / 1000.0)
                }
                _ => "—".to_string(),
            };

            let tokens = match (m.input_tokens, m.output_tokens) {
                (Some(i), Some(o)) => format!("{}", i + o),
                _ => "—".to_string(),
            };

            let completed = match m.completed_at {
                Some(c) => {
                    let diff = Utc::now().signed_duration_since(c);
                    if diff.num_minutes() < 1 {
                        "Just now".to_string()
                    } else if diff.num_minutes() < 60 {
                        format!("{} mins ago", diff.num_minutes())
                    } else {
                        format!("{} hours ago", diff.num_hours())
                    }
                }
                None => {
                    if status == "Queued" {
                        "Pending".to_string()
                    } else {
                        "In Progress".to_string()
                    }
                }
            };

            let entity = match (&m.source_entity_type, m.source_entity_id) {
                (Some(t), Some(id)) => {
                    let short_id = id.to_string().chars().take(8).collect::<String>();
                    format!("{} ({})", short_id, t)
                }
                _ => "Platform Outbox".to_string(),
            };

            let initial_logs = generate_task_logs(&m);

            AdminAiTaskResponse {
                id: format!("ait_{}", m.id.to_string().chars().take(8).collect::<String>()),
                task_type: m.task_type.clone(),
                entity,
                status,
                status_class: status_class.to_string(),
                runtime,
                tokens,
                completed,
                model: m.model.clone().unwrap_or_else(|| "gpt-4o-mini".to_string()),
                params: m.input_payload.clone(),
                initial_logs,
                streamable: m.status == "running",
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn abort_ai_task(
    State(db): State<DatabaseConnection>,
    Path(id_str): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Locate task by short Uuid prefix or actual ID
    let tasks = atlas_ai_task::Entity::find().all(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let clean_id = id_str.replace("ait_", "");
    let task = tasks
        .into_iter()
        .find(|t| t.id.to_string().starts_with(&clean_id))
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: atlas_ai_task::ActiveModel = task.into();
    active.status = Set("failed".to_string());
    active.error_message = Set(Some("Task manually aborted by Platform Super-Admin".to_string()));
    active.completed_at = Set(Some(Utc::now()));
    active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn rerun_ai_task(
    State(db): State<DatabaseConnection>,
    Path(id_str): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let tasks = atlas_ai_task::Entity::find().all(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let clean_id = id_str.replace("ait_", "");
    let task = tasks
        .into_iter()
        .find(|t| t.id.to_string().starts_with(&clean_id))
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: atlas_ai_task::ActiveModel = task.into();
    active.status = Set("queued".to_string());
    active.started_at = Set(None);
    active.completed_at = Set(None);
    active.error_message = Set(None);
    active.retry_count = Set(0);
    active.input_tokens = Set(None);
    active.output_tokens = Set(None);
    active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
