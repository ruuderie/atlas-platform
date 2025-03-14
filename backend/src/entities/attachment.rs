use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "attachment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub feed_item_id: Option<Uuid>,
    pub url: String,
    pub mime_type: String,
    pub title: Option<String>,
    pub size_in_bytes: Option<i64>,
    pub duration_in_seconds: Option<i32>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    FeedItem,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::FeedItem => Entity::belongs_to(super::feed_item::Entity)
                .from(Column::FeedItemId)
                .to(super::feed_item::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
