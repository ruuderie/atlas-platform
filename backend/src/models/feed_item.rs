use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::entities::feed_item;
use crate::models::attachment::AttachmentModel;

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedItemModel {
    pub id: Uuid,
    pub feed_id: Uuid,
    pub url: String,
    pub external_url: Option<String>,
    pub title: String,
    pub content_html: String,
    pub content_text: String,
    pub summary: Option<String>,
    pub image: Option<String>,
    pub banner_image: Option<String>,
    pub date_published: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub author_name: String,
    pub author_url: Option<String>,
    pub author_avatar: Option<String>,
    pub tags: Vec<String>,
    pub attachments: Option<Vec<AttachmentModel>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedItem {
    pub feed_id: Uuid,
    pub url: Option<String>,
    pub external_url: Option<String>,
    pub title: String,
    pub content_html: String,
    pub content_text: String,
    pub summary: Option<String>,
    pub image: Option<String>,
    pub banner_image: Option<String>,
    pub author_name: String,
    pub author_url: Option<String>,
    pub author_avatar: Option<String>,
    pub tags: Vec<String>,
    pub attachments: Option<Vec<CreateAttachment>>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFeedItem {
    pub url: Option<String>,
    pub external_url: Option<String>,
    pub title: Option<String>,
    pub content_html: Option<String>,
    pub content_text: Option<String>,
    pub summary: Option<String>,
    pub image: Option<String>,
    pub banner_image: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub author_avatar: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAttachment {
    pub url: String,
    pub mime_type: String,
    pub title: Option<String>,
    pub size_in_bytes: Option<i64>,
    pub duration_in_seconds: Option<i32>,
}

impl From<feed_item::Model> for FeedItemModel {
    fn from(item: feed_item::Model) -> Self {
        Self {
            id: item.id,
            feed_id: item.feed_id,
            url: item.url,
            external_url: item.external_url,
            title: item.title,
            content_html: item.content_html,
            content_text: item.content_text,
            summary: item.summary,
            image: item.image,
            banner_image: item.banner_image,
            date_published: item.date_published,
            date_modified: item.date_modified,
            author_name: item.author_name,
            author_url: item.author_url,
            author_avatar: item.author_avatar,
            tags: item.tags,
            attachments: None, // This will be populated separately
            status: item.status,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
} 