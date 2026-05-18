use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// SeaORM entity for the `app_instance_module` table.
///
/// Each row represents one admin module that is (or has been) configured
/// for a specific app instance. The `module_type` column maps to
/// `AdminModuleType` in `backend/src/models/admin_module.rs`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_instance_module")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub app_instance_id: Uuid,
    /// Serialized `AdminModuleType` — stored as SCREAMING_SNAKE_CASE string.
    pub module_type: String,
    pub display_name: String,
    #[sea_orm(nullable)]
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_enabled: bool,
    /// True for platform-fixed modules (Dashboard, Settings, Security).
    pub is_fixed: bool,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub config: Option<Value>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::app_instance::Entity",
        from = "Column::AppInstanceId",
        to = "super::app_instance::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AppInstance,
}

impl Related<super::app_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
