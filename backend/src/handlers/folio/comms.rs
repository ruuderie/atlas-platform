// backend/src/handlers/folio/comms.rs
//
//! Folio — Communications handler (GENERIC-07).
//!
//! # Routes (shared — any authenticated Folio user)
//!
//! ```ignore
//! GET  /api/folio/rooms
//!      List all rooms the current user participates in, most-recently-active
//!      first. Includes last message preview.
//!      -> 200 [RoomSummary]
//!
//! POST /api/folio/rooms
//!      Create a new room. `room_type` = "direct" | "group" | "platform_support".
//!      Body: CreateRoomInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/rooms/{id}/messages?limit=50
//!      Fetch messages for a room (descending, newest last).
//!      -> 200 [MessageRow]
//!
//! POST /api/folio/rooms/{id}/messages
//!      Send a message to a room.
//!      Body: SendMessageInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/rooms/support
//!      Get-or-create the caller's platform_support room.
//!      -> 200 { "room_id": uuid }
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_ws_message, atlas_ws_room, user};
use crate::types::realtime::WsMessageType;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/rooms", get(list_rooms).post(create_room))
        .route("/api/folio/rooms/support", get(get_or_create_support_room))
        .route(
            "/api/folio/rooms/{id}/messages",
            get(list_messages).post(send_message),
        )
}

// ── Input / output types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateRoomInput {
    /// "direct" | "group" | "platform_support"
    pub room_type: String,
    /// Human-readable thread name (optional for direct, required for group)
    pub name: Option<String>,
    /// The entity this room is anchored to (lease_id, property_id, etc.)
    /// Pass the caller's user_id for platform_support rooms.
    pub entity_id: Option<Uuid>,
    pub entity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SendMessageInput {
    pub content: String,
    /// Default: "text". Future: "image" | "file" | "system"
    #[serde(default = "default_msg_type")]
    pub message_type: String,
}
fn default_msg_type() -> String {
    "text".to_string()
}

#[derive(Debug, Deserialize)]
struct MsgQuery {
    limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct RoomSummary {
    pub id: Uuid,
    pub room_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_message: Option<String>,
    pub last_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct MessageRow {
    pub id: Uuid,
    pub room_id: Uuid,
    pub sender_user_id: Option<Uuid>,
    pub message_type: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/rooms
async fn list_rooms(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    let tenant_id = resolve_tenant_id(&db, user.id).await?;

    let rooms = atlas_ws_room::Entity::find()
        .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
        .filter(atlas_ws_room::Column::IsActive.eq(true))
        .order_by_desc(atlas_ws_room::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_rooms db error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Fetch last message for each room in a single query per room.
    // For production scale this should be a lateral join; for now N+1 is fine
    // given typical room counts per tenant.
    let mut summaries: Vec<RoomSummary> = Vec::with_capacity(rooms.len());
    for room in rooms {
        let last = atlas_ws_message::Entity::find()
            .filter(atlas_ws_message::Column::RoomId.eq(room.id))
            .filter(
                atlas_ws_message::Column::MessageType
                    .ne(WsMessageType::InternalNote.as_str()),
            )
            .order_by_desc(atlas_ws_message::Column::CreatedAt)
            .limit(1)
            .one(&db)
            .await
            .unwrap_or(None);

        summaries.push(RoomSummary {
            id: room.id,
            room_type: room.room_type,
            entity_type: room.entity_type,
            entity_id: room.entity_id,
            is_active: room.is_active,
            created_at: room.created_at,
            last_message: last.as_ref().map(|m| truncate(&m.content, 80)),
            last_at: last.as_ref().map(|m| m.created_at),
        });
    }

    // Re-sort by last_at desc (rooms with no messages stay at back)
    summaries.sort_by(|a, b| b.last_at.cmp(&a.last_at));

    Ok::<_, StatusCode>(axum::response::Json(summaries))
}

/// POST /api/folio/rooms
async fn create_room(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
    Json(body): Json<CreateRoomInput>,
) -> impl IntoResponse {
    let tenant_id = resolve_tenant_id(&db, user.id).await?;

    let entity_id = body.entity_id.unwrap_or(user.id);
    let entity_type = body.entity_type.unwrap_or_else(|| "user".to_string());

    let room = atlas_ws_room::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        room_type: Set(body.room_type.clone()),
        entity_type: Set(entity_type),
        entity_id: Set(entity_id),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        ..Default::default()
    };
    let created = room.insert(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "create_room error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok::<_, StatusCode>((
        StatusCode::CREATED,
        axum::response::Json(serde_json::json!({ "id": created.id })),
    ))
}

/// GET /api/folio/rooms/support — get or create the caller's platform_support room
async fn get_or_create_support_room(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    let tenant_id = resolve_tenant_id(&db, user.id).await?;

    // Look for an existing support room for this user
    let existing = atlas_ws_room::Entity::find()
        .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
        .filter(atlas_ws_room::Column::RoomType.eq("platform_support"))
        .filter(atlas_ws_room::Column::EntityId.eq(user.id))
        .filter(atlas_ws_room::Column::IsActive.eq(true))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "get_support_room db error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let room_id = if let Some(room) = existing {
        room.id
    } else {
        let room = atlas_ws_room::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            room_type: Set("platform_support".to_string()),
            entity_type: Set("user".to_string()),
            entity_id: Set(user.id),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            ..Default::default()
        };
        room.insert(&db)
            .await
            .map_err(|e| {
                tracing::error!(%tenant_id, "create_support_room error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .id
    };

    Ok::<_, StatusCode>(axum::response::Json(
        serde_json::json!({ "room_id": room_id }),
    ))
}

/// GET /api/folio/rooms/{id}/messages
async fn list_messages(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
    Path(room_id): Path<Uuid>,
    Query(q): Query<MsgQuery>,
) -> impl IntoResponse {
    let tenant_id = resolve_tenant_id(&db, user.id).await?;

    // Verify the room belongs to this tenant
    atlas_ws_room::Entity::find()
        .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
        .filter(atlas_ws_room::Column::Id.eq(room_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let limit = q.limit.unwrap_or(100).min(500);

    let msgs = atlas_ws_message::Entity::find()
        .filter(atlas_ws_message::Column::RoomId.eq(room_id))
        .filter(
            atlas_ws_message::Column::MessageType.ne(WsMessageType::InternalNote.as_str()),
        )
        .order_by_asc(atlas_ws_message::Column::CreatedAt)
        .limit(limit)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%room_id, "list_messages error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let rows: Vec<MessageRow> = msgs
        .into_iter()
        .map(|m| MessageRow {
            id: m.id,
            room_id: m.room_id,
            sender_user_id: m.sender_user_id,
            message_type: m.message_type,
            content: m.content,
            created_at: m.created_at,
        })
        .collect();

    Ok::<_, StatusCode>(axum::response::Json(rows))
}

/// POST /api/folio/rooms/{id}/messages
async fn send_message(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
    Path(room_id): Path<Uuid>,
    Json(body): Json<SendMessageInput>,
) -> impl IntoResponse {
    let tenant_id = resolve_tenant_id(&db, user.id).await?;

    if body.content.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Folio callers may only post user-visible text; reject operator/system types.
    let message_type = match WsMessageType::try_from(body.message_type.as_str()) {
        Ok(WsMessageType::Text) => WsMessageType::Text.to_string(),
        Ok(_) => return Err(StatusCode::UNPROCESSABLE_ENTITY),
        Err(_) => return Err(StatusCode::UNPROCESSABLE_ENTITY),
    };

    // Tenant-scope guard
    atlas_ws_room::Entity::find()
        .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
        .filter(atlas_ws_room::Column::Id.eq(room_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let msg = atlas_ws_message::ActiveModel {
        id: Set(Uuid::new_v4()),
        room_id: Set(room_id),
        sender_user_id: Set(Some(user.id)),
        message_type: Set(message_type),
        content: Set(body.content.trim().to_string()),
        created_at: Set(Utc::now()),
        ..Default::default()
    };
    let created = msg.insert(&db).await.map_err(|e| {
        tracing::error!(%room_id, "send_message error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok::<_, StatusCode>((
        StatusCode::CREATED,
        axum::response::Json(serde_json::json!({ "id": created.id })),
    ))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();
    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    Ok(profile.tenant_id)
}
