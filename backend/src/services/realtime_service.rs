use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_ws_room::{self, Entity as WsRoomEntity, ActiveModel as WsRoomActiveModel};
use crate::entities::atlas_ws_message::{self, Entity as WsMessageEntity, ActiveModel as WsMessageActiveModel};

/// Service layer for GENERIC-07: Realtime WebSocket infrastructure.
/// Manages rooms and messages for chat, live updates, collaboration, notifications.
pub struct RealtimeService;

impl RealtimeService {
    // Rooms
    pub async fn create_room(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        room_type: &str,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Uuid, String> {
        let room = WsRoomActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            room_type: Set(room_type.to_string()),
            entity_type: Set(entity_type.to_string()),
            entity_id: Set(entity_id),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = room.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_room_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        room_id: Uuid,
    ) -> Result<Option<atlas_ws_room::Model>, String> {
        WsRoomEntity::find()
            .filter(atlas_ws_room::Column::TenantId.eq(tenant_id))
            .filter(atlas_ws_room::Column::Id.eq(room_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    // Messages
    pub async fn post_message(
        db: &DatabaseConnection,
        room_id: Uuid,
        sender_user_id: Option<Uuid>,
        message_type: &str,
        content: &str,
    ) -> Result<Uuid, String> {
        let msg = WsMessageActiveModel {
            id: Set(Uuid::new_v4()),
            room_id: Set(room_id),
            sender_user_id: Set(sender_user_id),
            message_type: Set(message_type.to_string()),
            content: Set(content.to_string()),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = msg.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn list_messages_for_room(
        db: &DatabaseConnection,
        room_id: Uuid,
        limit: u64,
    ) -> Result<Vec<atlas_ws_message::Model>, String> {
        WsMessageEntity::find()
            .filter(atlas_ws_message::Column::RoomId.eq(room_id))
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }
}