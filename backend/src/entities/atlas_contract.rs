#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-11: AtlasContract
/// Legal agreements registry (leases, insurance policies, corporate agreements, SLAs, etc.)
///
/// Uses `contract_type` + `terms_metadata` JSONB for jurisdiction-specific and type-specific data.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_contracts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub contract_type: String,
    pub counterparty_user_id: Option<Uuid>,
    pub asset_id: Option<Uuid>, // FK to atlas_assets
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub auto_renew: bool,
    pub recurring_amount_cents: Option<i64>,
    pub currency: String,
    pub billing_interval: String,
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub termination_reason: Option<String>,
    pub terms_metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    /// PMC: which client account this contract belongs to. NULL = PM's own. (m20260817)
    pub managed_account_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
