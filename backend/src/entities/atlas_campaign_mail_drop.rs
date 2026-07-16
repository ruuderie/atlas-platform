//! G-19 companion — physical mail drop under a campaign.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_campaign_mail_drops")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub campaign_id: Uuid,
    pub drop_name: String,
    pub creative_variant: Option<String>,
    pub utm_content: Option<String>,
    pub piece_count: i32,
    pub unit_cost_cents: Option<i64>,
    pub provider_job_id: Option<String>,
    /// VARCHAR — draft | ready | mailed | cancelled
    pub status: String,
    pub mailed_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
