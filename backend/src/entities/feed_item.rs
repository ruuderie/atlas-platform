use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "feed_item")]
pub struct Model {
    #[sea_orm(primary_key)]
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
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub date_published: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub date_modified: DateTime<Utc>,
    pub author_name: String,
    pub author_url: Option<String>,
    pub author_avatar: Option<String>,
    pub tags: Vec<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub attachments: Option<Value>,
    pub status: String, // draft, published, archived
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::feed::Entity",
        from = "Column::FeedId",
        to = "super::feed::Column::Id"
    )]
    Feed,
}

impl Related<super::feed::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feed.def()
    }
} 
impl ActiveModelBehavior for ActiveModel {}