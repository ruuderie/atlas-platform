#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// GENERIC-17 (part 1): AtlasTaxEvent
/// Individual taxable revenue events.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_tax_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tax_type: String,
    pub jurisdiction_code: String,
    pub source_integration_id: Option<Uuid>,
    pub source_ledger_entry_id: Option<Uuid>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
    pub gross_revenue_cents: i64,
    pub excluded_fees_cents: i64,
    pub taxable_revenue_cents: i64,
    pub tax_rate: f64,
    pub tax_amount_cents: i64,
    pub remitted_by: String,
    pub tax_filing_id: Option<Uuid>,
    pub event_date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
