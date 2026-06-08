//! # G07 WebSocket Relay — Realtime room infrastructure
//!
//! ## Architecture
//!
//! ```text
//!  Client A          Server                Client B
//!    │                  │                    │
//!    │── WS upgrade ───►│                    │
//!    │                  │  join_room(room_id) │
//!    │                  │◄── WS upgrade ─────│
//!    │                  │                    │
//!    │── send msg ─────►│──broadcast(msg)────►│
//!    │◄─────────────────│── broadcast(msg) ───│
//! ```
//!
//! Each `atlas_ws_room` has a `tokio::sync::broadcast` channel stored in
//! `ROOM_REGISTRY: DashMap<room_id, Sender<WsEvent>>`. The sender is created
//! lazily on first connect and dropped when the last subscriber disconnects.
//!
//! ## Rooms
//!
//! Rooms are entity-scoped: a maintenance ticket, an opportunity, a campaign, etc.
//! Any entity in the platform can have a live room. Room rows persist in
//! `atlas_ws_rooms`; the in-memory channel is ephemeral — reconnect recreates it.
//!
//! ## Routes
//!
//! | Method | Path                                    | Auth | Description              |
//! |--------|-----------------------------------------|------|--------------------------|
//! | GET    | /api/ws/rooms                           | ✅   | List rooms for tenant    |
//! | POST   | /api/ws/rooms                           | ✅   | Create room              |
//! | GET    | /api/ws/rooms/{room_id}/connect         | ✅   | WS upgrade (long-lived)  |
//! | GET    | /api/ws/rooms/{room_id}/messages        | ✅   | Fetch message history    |
//!
//! ## Event envelope
//!
//! Every message on the wire (client→server and server→client) is JSON:
//!
//! ```json
//! {
//!   "type":       "chat" | "system" | "presence" | "notification",
//!   "room_id":    "uuid",
//!   "sender_id":  "uuid | null",
//!   "content":    "string",
//!   "ts":         "2024-01-01T00:00:00Z"
//! }
//! ```

use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::entities::{atlas_ws_room, atlas_ws_message, user};

// ── In-process room registry ──────────────────────────────────────────────────

/// Channel capacity per room: 256 buffered events before oldest is dropped.
const ROOM_CHANNEL_CAPACITY: usize = 256;

/// Global ephemeral room registry.
///
/// Key: `room_id` (matches `atlas_ws_rooms.id`).
/// Value: broadcast sender for that room.
/// Entry is inserted on first subscriber, dropped when all senders are gone.
static ROOM_REGISTRY: Lazy<Arc<DashMap<Uuid, broadcast::Sender<Arc<WsEvent>>>>> =
    Lazy::new(|| Arc::new(DashMap::new()));

// ── Wire event type ───────────────────────────────────────────────────────────

/// The canonical event envelope sent on the WebSocket wire (both directions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    /// Semantic event type: `"chat"`, `"system"`, `"presence"`, `"notification"`.
    #[serde(rename = "type")]
    pub event_type: String,
    pub room_id: Uuid,
    /// `null` for system events (server-generated).
    pub sender_id: Option<Uuid>,
    pub content: String,
    /// ISO-8601 timestamp.
    pub ts: String,
}

impl WsEvent {
    fn new(event_type: &str, room_id: Uuid, sender_id: Option<Uuid>, content: String) -> Self {
        Self {
            event_type: event_type.into(),
            room_id,
            sender_id,
            content,
            ts: Utc::now().to_rfc3339(),
        }
    }

    fn system(room_id: Uuid, content: &str) -> Arc<Self> {
        Arc::new(Self::new("system", room_id, None, content.into()))
    }
}

// ── Room registry helpers ─────────────────────────────────────────────────────

fn get_or_create_sender(room_id: Uuid) -> broadcast::Sender<Arc<WsEvent>> {
    if let Some(sender) = ROOM_REGISTRY.get(&room_id) {
        return sender.clone();
    }
    let (tx, _rx) = broadcast::channel(ROOM_CHANNEL_CAPACITY);
    ROOM_REGISTRY.insert(room_id, tx.clone());
    tx
}

/// Broadcast a server-generated event to all subscribers of a room.
///
/// Called by other services to push platform events into live rooms:
/// - Ledger payment confirmed → notify all room subscribers
/// - Lead status changed → notify sales team room
/// - Maintenance case updated → notify property owner room
pub fn broadcast_to_room(room_id: Uuid, event: Arc<WsEvent>) {
    if let Some(sender) = ROOM_REGISTRY.get(&room_id) {
        // `send()` errors if there are no active receivers — safe to ignore.
        let _ = sender.send(event);
    }
}

// ── Route constructor ─────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/ws/rooms", post(create_room).get(list_rooms))
        .route("/api/ws/rooms/{room_id}/connect", get(ws_connect))
        .route("/api/ws/rooms/{room_id}/messages", get(room_messages))
}

// ── Tenant resolution ─────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
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

// ── Request/Response types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateRoomInput {
    /// Semantic room type: `"chat"`, `"notifications"`, `"presence"`, `"audit"`.
    room_type: String,
    /// The entity this room is attached to (e.g. `"atlas_maintenance_case"`).
    entity_type: String,
    entity_id: Uuid,
}

#[derive(Debug, Serialize)]
struct CreateRoomResponse {
    room_id: Uuid,
    /// The WS upgrade URL clients should connect to.
    connect_url: String,
}

#[derive(Debug, Deserialize)]
struct MessagesQuery {
    limit: Option<u64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/ws/rooms — Create an entity-scoped realtime room.
async fn create_room(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(body): Json<CreateRoomInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let now = chrono::Utc::now();

    let room = atlas_ws_room::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        room_type: Set(body.room_type),
        entity_type: Set(body.entity_type),
        entity_id: Set(body.entity_id),
        is_active: Set(true),
        created_at: Set(now),
    }
    .insert(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateRoomResponse {
            connect_url: format!("/api/ws/rooms/{}/connect", room.id),
            room_id: room.id,
        }),
    ))
}

/// GET /api/ws/rooms — List active rooms for this tenant.
async fn list_rooms(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let rooms = atlas_ws_room::Entity::find()
        .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
        .filter(atlas_ws_room::Column::IsActive.eq(true))
        .order_by_desc(atlas_ws_room::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rooms))
}

/// GET /api/ws/rooms/{room_id}/messages — Fetch paginated message history.
async fn room_messages(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(room_id): Path<Uuid>,
    Query(q): Query<MessagesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Verify room belongs to this tenant.
    atlas_ws_room::Entity::find_by_id(room_id)
        .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let limit: usize = q.limit.unwrap_or(50).min(200) as usize;
    let messages: Vec<atlas_ws_message::Model> = atlas_ws_message::Entity::find()
        .filter(atlas_ws_message::Column::RoomId.eq(room_id))
        .order_by_desc(atlas_ws_message::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .take(limit)
        .collect();

    Ok(Json(messages))
}

// ── WebSocket upgrade & relay ──────────────────────────────────────────────────

/// GET /api/ws/rooms/{room_id}/connect — WebSocket upgrade.
///
/// The connection lifecycle:
/// 1. Validate the room exists and belongs to this tenant.
/// 2. Subscribe to the room's broadcast channel (or create it).
/// 3. Send a `system` "joined" event to the new subscriber.
/// 4. Pump: forward incoming WS text frames as chat events (broadcast + persist).
/// 5. On close: send a `system` "left" event; sender ref is dropped.
async fn ws_connect(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(room_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Validate room.
    atlas_ws_room::Entity::find_by_id(room_id)
        .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
        .filter(atlas_ws_room::Column::IsActive.eq(true))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let user_id = current_user.id;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, db, room_id, user_id)))
}

async fn handle_socket(
    mut socket: WebSocket,
    db: DatabaseConnection,
    room_id: Uuid,
    user_id: Uuid,
) {
    let tx = get_or_create_sender(room_id);
    let mut rx = tx.subscribe();

    // Announce join.
    let joined = WsEvent::system(room_id, &format!("{user_id} joined"));
    let _ = tx.send(joined);

    // Pump loop — select between:
    //   a) incoming WS message from this client
    //   b) broadcast event from another subscriber
    loop {
        tokio::select! {
            // ── Inbound: client → server ─────────────────────────────────────
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Persist message.
                        let content = text.to_string();
                        let _ = atlas_ws_message::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            room_id: Set(room_id),
                            sender_user_id: Set(Some(user_id)),
                            message_type: Set("text".into()),
                            content: Set(content.clone()),
                            translated_content: Set(None),
                            attachment_id: Set(None),
                            created_at: Set(Utc::now()),
                        }
                        .insert(&db)
                        .await;

                        // Broadcast to all room subscribers.
                        let event = Arc::new(WsEvent::new(
                            "chat",
                            room_id,
                            Some(user_id),
                            content,
                        ));
                        let _ = tx.send(event);
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        // Client closed — announce departure.
                        let left = WsEvent::system(room_id, &format!("{user_id} left"));
                        let _ = tx.send(left);
                        break;
                    }
                    Some(Ok(_)) => {} // ping/pong/binary — ignore
                    Some(Err(_)) => break, // transport error
                }
            }

            // ── Outbound: broadcast → this client ─────────────────────────────
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        let json = match serde_json::to_string(&*ev) {
                            Ok(j) => j,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            // Client disconnected — exit loop.
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // Subscriber was too slow — missed `n` events. Continue.
                        tracing::warn!(room_id = %room_id, skipped = n, "WS subscriber lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    // Clean up: if no receivers remain, remove the registry entry.
    if tx.receiver_count() == 0 {
        ROOM_REGISTRY.remove(&room_id);
    }
}
