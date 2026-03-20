use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "listing_ab_variant")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub test_id: Uuid,
    pub name: String,
    pub is_control: bool,
    pub views: i32,
    pub conversions: i32,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ListingAbTest,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ListingAbTest => Entity::belongs_to(super::listing_ab_test::Entity)
                .from(Column::TestId)
                .to(super::listing_ab_test::Column::Id)
                .into(),
        }
    }
}

impl Related<super::listing_ab_test::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ListingAbTest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
