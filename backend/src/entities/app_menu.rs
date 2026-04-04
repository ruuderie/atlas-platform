use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_menus")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub menu_type: String,
    pub label: String,
    #[sea_orm(nullable)]
    pub href: Option<String>,
    #[sea_orm(nullable)]
    pub parent_id: Option<Uuid>,
    pub display_order: i32,
    pub is_visible: bool,
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
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SelfRef,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
// Self-referencing relationship for hierarchical menus
impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SelfRef.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
