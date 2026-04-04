use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "template")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub is_active: bool,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub attributes_schema: Option<Value>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Tenant,
    Category,
    BasedListings,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Tenant => Entity::belongs_to(super::tenant::Entity)
                .from(Column::TenantId)
                .to(super::tenant::Column::Id)
                .into(),
            Self::Category => Entity::belongs_to(super::category::Entity)
                .from(Column::CategoryId)
                .to(super::category::Column::Id)
                .into(),
            Self::BasedListings => Entity::has_many(super::listing::Entity).into(),
        }
    }
}


impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::listing::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BasedListings.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}