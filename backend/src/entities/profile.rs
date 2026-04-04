use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "profile")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub tenant_id: Uuid,
    pub profile_type: ProfileType,
    pub display_name: String,
    pub contact_info: String,
    pub business_name: Option<String>,
    pub business_address: Option<String>,
    pub business_phone: Option<String>,
    pub business_website: Option<String>,
    pub additional_info: Option<Value>,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub properties: Option<Value>,
    #[sea_orm(column_type = "custom(\"text[]\")", nullable)]
    pub service_area_zips: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum ProfileType {
    #[sea_orm(string_value = "Individual")]
    Individual,
    #[sea_orm(string_value = "Business")]
    Business,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BusinessDetails {
    pub business_name: String,
    pub business_address: String,
    pub business_phone: String,
    pub website: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Account,
    Tenant,
    Listing,
    AdPurchase,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(super::account::Entity)
                .from(Column::AccountId)
                .to(super::account::Column::Id)
                .into(),
            Self::Tenant => Entity::belongs_to(super::tenant::Entity)
                .from(Column::TenantId)
                .to(super::tenant::Column::Id)
                .into(),
            Self::Listing => Entity::has_many(super::listing::Entity).into(),
            Self::AdPurchase => Entity::has_many(super::ad_purchase::Entity).into(),
        }
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl Related<super::listing::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Listing.def()
    }
}

impl Related<super::ad_purchase::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdPurchase.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn business_details(&self) -> Option<BusinessDetails> {
        match self.profile_type {
            ProfileType::Business => Some(BusinessDetails {
                business_name: self.business_name.clone()?,
                business_address: self.business_address.clone()?,
                business_phone: self.business_phone.clone()?,
                website: self.business_website.clone(),
            }),
            _ => None,
        }
    }
}