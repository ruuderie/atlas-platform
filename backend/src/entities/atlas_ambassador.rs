//! G-37 — Growth ambassadors / referral partners / influencers / affiliates.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_ambassadors")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub code: String,
    pub display_name: String,
    /// VARCHAR — `AmbassadorPartnerType` at service/API boundary.
    pub partner_type: String,
    /// VARCHAR — `AmbassadorStatus` at service/API boundary.
    pub status: String,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub notes: Option<String>,
    pub channels: Option<serde_json::Value>,
    pub fulfillment_requests: serde_json::Value,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
