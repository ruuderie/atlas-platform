//! Admin — Platform Support Inbox
//!
//! Operator-side view of all `platform_support` rooms across every tenant.
//! Each Folio user who opens a support thread creates an `atlas_ws_room`
//! with `room_type = 'platform_support'` scoped to their tenant.
//! This module queries them platform-wide (no tenant filter) and lets
//! platform operators reply into the thread.
//!
//! Routes:
//!   GET  /api/admin/support/threads            — list all support threads (paginated)
//!   GET  /api/admin/support/threads/{id}        — thread detail + messages
//!   POST /api/admin/support/threads/{id}/reply  — operator sends a message
//!   PUT  /api/admin/support/threads/{id}/close  — mark thread closed

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_ws_message, atlas_ws_room, user};

// ── Router ────────────────────────────────────────────────────────────────────

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/support/threads",             get(list_threads))
        .route("/api/admin/support/threads/:id",         get(get_thread))
        .route("/api/admin/support/threads/:id/reply",   post(reply_thread))
        .route("/api/admin/support/threads/:id/close",   put(close_thread))
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListQuery {
    /// "open" | "closed" | "all" (default: "open")
    status: Option<String>,
    limit:  Option<u64>,
    offset: Option<u64>,
}

// ── Response models ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ThreadSummary {
    id:              Uuid,
    tenant_id:       Uuid,
    /// The submitting user's ID (stored as entity_id on the room)
    entity_id:       Uuid,
    is_active:       bool,
    created_at:      DateTime<Utc>,
    last_message:    Option<String>,
    last_at:         Option<DateTime<Utc>>,
    message_count:   u64,
    submitter_name:  Option<String>,
    submitter_email: Option<String>,
}

#[derive(Debug, Serialize)]
struct ThreadDetail {
    #[serde(flatten)]
    summary:  ThreadSummary,
    messages: Vec<MessageRow>,
}

#[derive(Debug, Serialize, Clone)]
struct MessageRow {
    id:             Uuid,
    sender_user_id: Option<Uuid>,
    sender_name:    Option<String>,
    /// "text" | "system" | "operator_reply"
    message_type:   String,
    content:        String,
    created_at:     DateTime<Utc>,
    is_operator:    bool,
}

// ── Input models ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ReplyInput {
    content: String,
}

// ── GET /api/admin/support/threads ───────────────────────────────────────────

async fn list_threads(
    Extension(db): Extension<DatabaseConnection>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    let status = q.status.as_deref().unwrap_or("open");
    let limit  = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);

    let mut query = atlas_ws_room::Entity::find()
        .filter(atlas_ws_room::Column::RoomType.eq("platform_support"))
        .order_by(atlas_ws_room::Column::CreatedAt, Order::Desc)
        .limit(limit)
        .offset(offset);

    if status == "open" {
        query = query.filter(atlas_ws_room::Column::IsActive.eq(true));
    } else if status == "closed" {
        query = query.filter(atlas_ws_room::Column::IsActive.eq(false));
    }
    // "all" → no is_active filter

    let rooms = query.all(&db).await.map_err(|e| {
        tracing::error!("support/list_threads db error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let summaries = enrich_rooms(&db, rooms).await?;
    Ok::<_, StatusCode>(axum::response::Json(summaries))
}

// ── GET /api/admin/support/threads/:id ───────────────────────────────────────

async fn get_thread(
    Extension(db): Extension<DatabaseConnection>,
    Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
    let room = atlas_ws_room::Entity::find_by_id(room_id)
        .filter(atlas_ws_room::Column::RoomType.eq("platform_support"))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let messages_raw = atlas_ws_message::Entity::find()
        .filter(atlas_ws_message::Column::RoomId.eq(room_id))
        .order_by_asc(atlas_ws_message::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%room_id, "support/get_thread messages error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Batch-resolve sender display names
    let sender_ids: Vec<Uuid> = messages_raw
        .iter()
        .filter_map(|m| m.sender_user_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let user_names = fetch_user_names(&db, &sender_ids).await;

    let messages: Vec<MessageRow> = messages_raw
        .into_iter()
        .map(|m| {
            let is_op = m.message_type == "operator_reply";
            let name = m.sender_user_id.and_then(|uid| user_names.get(&uid).cloned());
            MessageRow {
                id:             m.id,
                sender_user_id: m.sender_user_id,
                sender_name:    name,
                is_operator:    is_op,
                message_type:   m.message_type,
                content:        m.content,
                created_at:     m.created_at,
            }
        })
        .collect();

    let last = messages.last().cloned();
    let count = messages.len() as u64;
    let (submitter_name, submitter_email) = resolve_submitter(&db, room.entity_id).await;

    let detail = ThreadDetail {
        summary: ThreadSummary {
            id:              room.id,
            tenant_id:       room.tenant_id,
            entity_id:       room.entity_id,
            is_active:       room.is_active,
            created_at:      room.created_at,
            last_message:    last.as_ref().map(|m| truncate(&m.content, 120)),
            last_at:         last.map(|m| m.created_at),
            message_count:   count,
            submitter_name,
            submitter_email,
        },
        messages,
    };

    Ok::<_, StatusCode>(axum::response::Json(detail))
}

// ── POST /api/admin/support/threads/:id/reply ────────────────────────────────

async fn reply_thread(
    Extension(db):   Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
    Path(room_id):   Path<Uuid>,
    Json(body):      Json<ReplyInput>,
) -> impl IntoResponse {
    if body.content.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Verify room exists and is a support room
    atlas_ws_room::Entity::find_by_id(room_id)
        .filter(atlas_ws_room::Column::RoomType.eq("platform_support"))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let msg = atlas_ws_message::ActiveModel {
        id:                 Set(Uuid::new_v4()),
        room_id:            Set(room_id),
        sender_user_id:     Set(Some(user.id)),
        message_type:       Set("operator_reply".to_string()),
        content:            Set(body.content.trim().to_string()),
        translated_content: Set(None),
        attachment_id:      Set(None),
        created_at:         Set(Utc::now()),
        ..Default::default()
    };
    let created = msg.insert(&db).await.map_err(|e| {
        tracing::error!(%room_id, "support/reply error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok::<_, StatusCode>((
        StatusCode::CREATED,
        axum::response::Json(serde_json::json!({ "id": created.id })),
    ))
}

// ── PUT /api/admin/support/threads/:id/close ─────────────────────────────────

async fn close_thread(
    Extension(db): Extension<DatabaseConnection>,
    Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
    let room = atlas_ws_room::Entity::find_by_id(room_id)
        .filter(atlas_ws_room::Column::RoomType.eq("platform_support"))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: atlas_ws_room::ActiveModel = room.into();
    active.is_active = Set(false);
    active.update(&db).await.map_err(|e| {
        tracing::error!(%room_id, "support/close_thread error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Append a system event message so the user sees it in their thread too
    let sys = atlas_ws_message::ActiveModel {
        id:                 Set(Uuid::new_v4()),
        room_id:            Set(room_id),
        sender_user_id:     Set(None),
        message_type:       Set("system".to_string()),
        content:            Set("This support thread has been closed by the platform team.".to_string()),
        translated_content: Set(None),
        attachment_id:      Set(None),
        created_at:         Set(Utc::now()),
        ..Default::default()
    };
    let _ = sys.insert(&db).await;

    Ok::<_, StatusCode>(StatusCode::NO_CONTENT)
}

// ── Private helpers ───────────────────────────────────────────────────────────

async fn enrich_rooms(
    db: &DatabaseConnection,
    rooms: Vec<atlas_ws_room::Model>,
) -> Result<Vec<ThreadSummary>, StatusCode> {
    let mut summaries = Vec::with_capacity(rooms.len());

    for room in rooms {
        let last = atlas_ws_message::Entity::find()
            .filter(atlas_ws_message::Column::RoomId.eq(room.id))
            .order_by_desc(atlas_ws_message::Column::CreatedAt)
            .limit(1)
            .one(db)
            .await
            .unwrap_or(None);

        let count = atlas_ws_message::Entity::find()
            .filter(atlas_ws_message::Column::RoomId.eq(room.id))
            .count(db)
            .await
            .unwrap_or(0);

        let (submitter_name, submitter_email) = resolve_submitter(db, room.entity_id).await;

        summaries.push(ThreadSummary {
            id:              room.id,
            tenant_id:       room.tenant_id,
            entity_id:       room.entity_id,
            is_active:       room.is_active,
            created_at:      room.created_at,
            last_message:    last.as_ref().map(|m| truncate(&m.content, 120)),
            last_at:         last.map(|m| m.created_at),
            message_count:   count,
            submitter_name,
            submitter_email,
        });
    }

    summaries.sort_by(|a, b| b.last_at.cmp(&a.last_at));
    Ok(summaries)
}

async fn resolve_submitter(db: &DatabaseConnection, user_id: Uuid) -> (Option<String>, Option<String>) {
    match user::Entity::find_by_id(user_id).one(db).await {
        Ok(Some(u)) => {
            let name = format!("{} {}", u.first_name, u.last_name).trim().to_string();
            let name = if name.is_empty() { None } else { Some(name) };
            (name, Some(u.email))
        }
        _ => (None, None),
    }
}

async fn fetch_user_names(db: &DatabaseConnection, ids: &[Uuid]) -> std::collections::HashMap<Uuid, String> {
    if ids.is_empty() {
        return Default::default();
    }
    match user::Entity::find()
        .filter(user::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
    {
        Ok(users) => users
            .into_iter()
            .map(|u| {
                let name = format!("{} {}", u.first_name, u.last_name).trim().to_string();
                let name = if name.is_empty() { u.username.clone() } else { name };
                (u.id, name)
            })
            .collect(),
        Err(_) => Default::default(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
