use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lead_charge")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub lead_id: Uuid,
    pub amount_cents: i32,
    pub status: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Account,
    #[sea_orm(
        belongs_to = "super::lead::Entity",
        from = "Column::LeadId",
        to = "super::lead::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Lead,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::lead::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lead.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
