#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub role: UserRole,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum UserRole {
    #[sea_orm(string_value = "Owner")]
    Owner,
    #[sea_orm(string_value = "Admin")]
    Admin,
    #[sea_orm(string_value = "Member")]
    Member,
    #[sea_orm(string_value = "PlatformSuperAdmin")]
    PlatformSuperAdmin,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User,
    Account,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::UserId)
                .to(super::user::Column::Id)
                .into(),
            Self::Account => Entity::belongs_to(super::account::Entity)
                .from(Column::AccountId)
                .to(super::account::Column::Id)
                .into(),
        }
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        // Define the relation via the Account entity
        super::account::Relation::Profile.def()
    }

    fn via() -> Option<RelationDef> {
        Some(Relation::Account.def())
    }
}

impl ActiveModelBehavior for ActiveModel {}
