#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-15: AtlasOpportunity
/// Deal and pipeline tracking object with flexible financial modeling via JSONB.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_opportunities")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub opportunity_type: String,
    pub name: String,
    pub asset_id: Option<Uuid>,
    pub crm_lead_id: Option<Uuid>,
    pub owner_user_id: Option<Uuid>,
    pub counterparty_user_id: Option<Uuid>,
    pub status: String,
    pub deal_amount_cents: Option<i64>,
    pub currency: String,
    pub close_date: Option<chrono::NaiveDate>,
    pub probability_pct: Option<i16>,
    pub financial_inputs: Option<Value>,
    pub computed_outputs: Option<Value>,
    pub notes: Option<String>,
    pub won_at: Option<DateTime<Utc>>,
    pub lost_at: Option<DateTime<Utc>>,
    pub lost_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
