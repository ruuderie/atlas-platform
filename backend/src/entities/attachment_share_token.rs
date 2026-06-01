#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Part of GENERIC-02 (atlas_vault)
/// Tokens that allow external/guest access to attachments without platform login.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "attachment_share_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub attachment_id: Uuid,
    pub token: String,
    pub resource_type: String,
    pub permissions: serde_json::Value,
    pub recipient_email: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub one_time_use: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
