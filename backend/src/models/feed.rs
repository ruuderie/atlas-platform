use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::entities::feed;
use crate::models::feed_item::FeedItemModel;

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub title: String,
    pub description: String,
    pub feed_url: String,
    pub home_page_url: String,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeed {
    pub tenant_id: Uuid,
    pub title: String,
    pub description: String,
    pub feed_url: Option<String>,
    pub home_page_url: Option<String>,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFeed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub feed_url: Option<String>,
    pub home_page_url: Option<String>,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedWithItems {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub title: String,
    pub description: String,
    pub feed_url: String,
    pub home_page_url: String,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<FeedItemModel>,
}

impl From<feed::Model> for FeedModel {
    fn from(feed: feed::Model) -> Self {
        Self {
            id: feed.id,
            tenant_id: feed.tenant_id,
            title: feed.title,
            description: feed.description,
            feed_url: feed.feed_url,
            home_page_url: feed.home_page_url,
            icon: feed.icon,
            favicon: feed.favicon,
            author: feed.author,
            created_at: feed.created_at,
            updated_at: feed.updated_at,
        }
    }
} 