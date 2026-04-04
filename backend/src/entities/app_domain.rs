use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_domains")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub app_instance_id: Uuid,
    pub domain_name: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
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
