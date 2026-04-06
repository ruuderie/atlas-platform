use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "billing_plans")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub price: i64,
    pub currency: String,
    pub interval: String,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tenant_subscription::Entity")]
    TenantSubscription,
}

impl Related<super::tenant_subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TenantSubscription.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
