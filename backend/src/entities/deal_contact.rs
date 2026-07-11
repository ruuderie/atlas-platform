#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "deal_contact")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub deal_id: Uuid,
    #[sea_orm(primary_key)]
    pub contact_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Deal,
    Contact,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Deal => Entity::belongs_to(super::deal::Entity)
                .from(Column::DealId)
                .to(super::deal::Column::Id)
                .into(),
            Self::Contact => Entity::belongs_to(super::contact::Entity)
                .from(Column::ContactId)
                .to(super::contact::Column::Id)
                .into(),
        }
    }
}

impl Related<super::deal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deal.def()
    }
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
