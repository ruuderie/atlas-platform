#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "file_associations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub file_id: String,
    pub associated_entity_type: String,
    pub associated_entity_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    File,
    Customer,
    Activity,
    Note,
    Contact,
    Case,
    Deal,
    Lead,
    User,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::File => Entity::belongs_to(super::file::Entity)
                .from(Column::FileId)
                .to(super::file::Column::Id)
                .into(),
            Self::Customer => Entity::belongs_to(super::customer::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::customer::Column::Id)
                .into(),
            Self::Activity => Entity::belongs_to(super::activity::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::activity::Column::Id)
                .into(),
            Self::Note => Entity::belongs_to(super::note::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::note::Column::Id)
                .into(),
            Self::Contact => Entity::belongs_to(super::contact::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::contact::Column::Id)
                .into(),
            Self::Case => Entity::belongs_to(super::case::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::case::Column::Id)
                .into(),
            Self::Deal => Entity::belongs_to(super::deal::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::deal::Column::Id)
                .into(),
            Self::Lead => Entity::belongs_to(super::lead::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::lead::Column::Id)
                .into(),
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::AssociatedEntityId)
                .to(super::user::Column::Id)
                .into(),
        }
    }
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::activity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Activity.def()
    }
}

impl Related<super::note::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Note.def()
    }
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl Related<super::case::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Case.def()
    }
}

impl Related<super::deal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deal.def()
    }
}

impl Related<super::lead::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lead.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
