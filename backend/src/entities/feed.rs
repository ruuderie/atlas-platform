use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "feed")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub directory_id: Uuid,
    pub title: String,
    pub description: String,
    pub feed_url: String,
    pub home_page_url: String,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub author: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::directory::Entity",
        from = "Column::DirectoryId",
        to = "super::directory::Column::Id"
    )]
    Directory,
    #[sea_orm(has_many = "super::feed_item::Entity")]
    FeedItem,
}

impl Related<super::directory::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Directory.def()
    }
}

impl Related<super::feed_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FeedItem.def()
    }
} 
impl ActiveModelBehavior for ActiveModel {}