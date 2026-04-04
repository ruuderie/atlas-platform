use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tenant_id: Uuid,  // New field to associate with Tenant
    pub name: String,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    pub stripe_customer_id: Option<String>,
    pub stripe_payment_method_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    UserAccount,
    Profile,
    Tenant,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::UserAccount => Entity::has_many(super::user_account::Entity).into(),
            Self::Profile => Entity::has_many(super::profile::Entity).into(),
            Self::Tenant => Entity::belongs_to(super::tenant::Entity)
                .from(Column::TenantId)
                .to(super::tenant::Column::Id)
                .into(),
            }
    }
}

impl Related<super::user_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAccount.def()
    }
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}