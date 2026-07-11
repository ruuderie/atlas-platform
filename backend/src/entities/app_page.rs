#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_pages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Platform product this page belongs to ("folio", "ruuderie", "network").
    /// Used by the platform-admin Landing Page Builder to scope pages by app.
    pub app_id: String,
    pub slug: String,
    pub locale: String,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub page_type: String,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub hero_payload: Option<Value>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub blocks_payload: Option<Value>,
    pub is_published: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Tenant,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
