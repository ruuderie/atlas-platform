#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// GENERIC-03: AtlasLedgerEntry
/// Records payments *within* a tenant's applications (rent, bookings, creator payouts, etc.).
/// This is distinct from the platform's own SaaS billing (the `transaction` table).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_ledger_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub billable_entity_type: String,
    pub billable_entity_id: Uuid,
    pub payer_user_id: Option<Uuid>,
    pub payer_email: Option<String>,
    pub gross_amount_cents: i64,
    pub fee_amount_cents: i64,
    pub net_amount_cents: i64,
    pub currency: String,
    pub payment_rail: Option<String>,
    pub external_tx_id: Option<String>,
    pub receipt_attachment_id: Option<Uuid>,
    pub status: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub paid_at: Option<DateTime<Utc>>,
    pub verified_by_user_id: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub reconciliation_note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
