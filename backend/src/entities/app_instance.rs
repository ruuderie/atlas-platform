use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_instances")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub app_type: String,
    #[sea_orm(nullable)]
    pub database_url: Option<String>,
    #[sea_orm(nullable)]
    pub data_seed_name: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub settings: Option<Value>,
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
    #[sea_orm(has_many = "super::app_domain::Entity")]
    AppDomain,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl Related<super::app_domain::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppDomain.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
