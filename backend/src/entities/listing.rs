use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::listing::ListingStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "listing")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub profile_id: Uuid,
    pub directory_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub listing_type: String,
    pub price: Option<f64>,
    pub price_type: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub additional_info: Option<Value>,
    #[sea_orm(column_type = "String(StringLen::N(32))")]
    pub status: ListingStatus,
    pub is_featured: bool,
    pub is_based_on_template: bool,
    pub based_on_template_id: Option<Uuid>,
    pub is_ad_placement: bool,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    #[sea_orm(column_type = "Text", nullable = true)]
    pub slug: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Profile,
    Directory,
    Category,
    BasedOnTemplate,
    ListingAttribute,
    AdPurchase,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Profile => Entity::belongs_to(super::profile::Entity)
                .from(Column::ProfileId)
                .to(super::profile::Column::Id)
                .into(),
            Self::Directory => Entity::belongs_to(super::directory::Entity)
                .from(Column::DirectoryId)
                .to(super::directory::Column::Id)
                .into(),
            Self::Category => Entity::belongs_to(super::category::Entity)
                .from(Column::CategoryId)
                .to(super::category::Column::Id)
                .into(),
            Self::BasedOnTemplate => Entity::belongs_to(super::template::Entity)
                .from(Column::BasedOnTemplateId)
                .to(super::template::Column::Id)
                .into(),
            Self::ListingAttribute => Entity::has_many(super::listing_attribute::Entity).into(),
            Self::AdPurchase => Entity::has_many(super::ad_purchase::Entity).into(),
        }
    }
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl Related<super::directory::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Directory.def()
    }
}

impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BasedOnTemplate.def()
    }
}

impl Related<super::listing_attribute::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ListingAttribute.def()
    }
}

impl Related<super::ad_purchase::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdPurchase.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}