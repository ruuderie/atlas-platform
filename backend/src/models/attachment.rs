use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::entities::attachment;

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentModel {
    pub id: Uuid,
    pub feed_item_id: Option<Uuid>,
    pub url: String,
    pub mime_type: String,
    pub title: Option<String>,
    pub size_in_bytes: Option<i64>,
    pub duration_in_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<attachment::Model> for AttachmentModel {
    fn from(attachment: attachment::Model) -> Self {
        Self {
            id: attachment.id,
            feed_item_id: attachment.feed_item_id,
            url: attachment.url,
            mime_type: attachment.mime_type,
            title: attachment.title,
            size_in_bytes: attachment.size_in_bytes,
            duration_in_seconds: attachment.duration_in_seconds,
            created_at: attachment.created_at,
            updated_at: attachment.updated_at,
        }
    }
} 