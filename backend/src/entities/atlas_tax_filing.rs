#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GENERIC-17 (part 2): AtlasTaxFiling
/// Periodic tax filing and remittance records.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_tax_filings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tax_type: String,
    pub jurisdiction_code: String,
    pub period_year: i16,
    pub period_month: Option<i16>,
    pub period_quarter: Option<i16>,
    pub total_taxable_revenue_cents: i64,
    pub total_tax_owed_cents: i64,
    pub platform_remitted_cents: i64,
    pub host_owed_cents: i64,
    pub status: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub filed_at: Option<DateTime<Utc>>,
    pub confirmation_number: Option<String>,
    pub filing_document_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
