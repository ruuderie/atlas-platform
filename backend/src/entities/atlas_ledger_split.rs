#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GENERIC-03: AtlasLedgerSplit
/// Allows a single ledger entry to be split across multiple recipients (platform fee + vendor + creator, etc.).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_ledger_splits")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub ledger_entry_id: Uuid,
    pub recipient_type: String,
    pub recipient_user_id: Option<Uuid>,
    pub recipient_label: Option<String>,
    pub amount_cents: i64,
    pub payout_rail: Option<String>,
    pub payout_status: String,
    pub payout_tx_id: Option<String>,
    pub settled_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
