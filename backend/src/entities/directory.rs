use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sea_orm::entity::EntityTrait;

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
    pub enabled_modules: u32,
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
    #[sea_orm(nullable)]
    pub logo: Option<String>,
    #[sea_orm(nullable)]
    pub favicon: Option<String>,
    #[sea_orm(nullable)]
    pub header_scripts: Option<String>,
    #[sea_orm(nullable)]
    pub footer_scripts: Option<String>,
    #[sea_orm(nullable)]
    pub google_analytics_id: Option<String>,
    #[sea_orm(nullable)]
    pub google_site_verification: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub meta_description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub meta_keywords: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub meta_title: Option<String>,
    #[sea_orm(nullable)]
    pub page_title: Option<String>,
    #[sea_orm(nullable)]
    pub page_description: Option<String>,
    #[sea_orm(nullable)]
    pub page_keywords: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub canonical_url: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    DirectoryType,
    Profile,
    Template,
    Listing,
    Account,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::DirectoryType => Entity::belongs_to(super::directory_type::Entity)
                .from(Column::DirectoryTypeId)
                .to(super::directory_type::Column::Id)
                .into(),
            Self::Profile => Entity::has_many(super::profile::Entity).into(),
            Self::Template => Entity::has_many(super::template::Entity).into(),
            Self::Listing => Entity::has_many(super::listing::Entity).into(),
            Self::Account => Entity::has_many(super::account::Entity).into(),
        }
    }
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

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}