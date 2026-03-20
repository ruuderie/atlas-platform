use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "category")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub directory_type_id: Uuid,
    pub parent_category_id: Option<Uuid>,
    pub name: String,
    pub description: String,
    #[sea_orm(nullable)]
    pub icon: Option<String>,
    #[sea_orm(nullable, unique)]
    pub slug: Option<String>,
    pub is_custom: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub directory_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    DirectoryType,
    ParentCategory,
    SubCategories,
    Templates,
    Listings,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::DirectoryType => Entity::belongs_to(super::directory_type::Entity)
                .from(Column::DirectoryTypeId)
                .to(super::directory_type::Column::Id)
                .into(),
            Self::ParentCategory => Entity::belongs_to(super::category::Entity)
                .from(Column::ParentCategoryId)
                .to(super::category::Column::Id)
                .into(),
            Self::SubCategories => Entity::has_many(super::category::Entity).into(),
            Self::Templates => Entity::has_many(super::template::Entity).into(),
            Self::Listings => Entity::has_many(super::listing::Entity).into(),
        }
    }
}

impl Related<super::directory_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DirectoryType.def()
    }
}

impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Templates.def()
    }
}

impl Related<super::listing::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Listings.def()
    }
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubCategories.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}