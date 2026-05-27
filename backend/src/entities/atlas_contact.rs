use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Unified Contact entity.
///
/// Represents individual people. Lighter and more focused than the legacy model.
/// Every contact should belong to an Account (atlas_accounts).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_contacts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub account_id: Uuid, // FK to atlas_accounts
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub is_primary: bool,
    pub contact_metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
