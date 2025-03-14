use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "directory")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub directory_type_id: Uuid,
    pub name: String,
    pub domain: String,
    pub description: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
    // New fields for multi-site management
    pub enabled_modules: i32,
    #[sea_orm(nullable)]
    pub theme: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    #[sea_orm(nullable)]
    pub custom_settings: Option<Value>,
    pub site_status: String,
    #[sea_orm(nullable, unique)]
    pub subdomain: Option<String>,
    #[sea_orm(nullable, unique)]
    pub custom_domain: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::directory_type::Entity",
        from = "Column::DirectoryTypeId",
        to = "super::directory_type::Column::Id"
    )]
    DirectoryType,
    #[sea_orm(has_many = "super::profile::Entity")]
    Profile,
    #[sea_orm(has_many = "super::template::Entity")]
    Template,
    #[sea_orm(has_many = "super::listing::Entity")]
    Listing,
}

impl Related<super::directory_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DirectoryType.def()
    }
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl Related<super::listing::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Listing.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}