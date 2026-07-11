#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Unified Account entity (replaces legacy customer + parts of contact).
///
/// This is the top-level party concept. It can represent either an individual (B2C)
/// or an organization (B2B).
///
/// Schema extended by: docs/architecture/account_contact_data_gap_analysis.md
/// Migration: m20260702_gap_fill_accounts_contacts.rs
///
/// # Design Decisions
///
/// Fields promoted from `account_metadata` JSONB to first-class columns are
/// those that will be filtered, searched, or joined on at scale (5–10M rows
/// from BusinessLeadsUSA alone). JSONB is preserved for vertical-specific
/// overflow (FMCSA safety data, MWBE certifications, financial health signals).
///
/// `account_metadata` was renamed from `attributes` — zero breaking call sites
/// confirmed (account_service.rs uses ..Default::default()).
///
/// `geo_point` (geometry(Point, 4326)) is absent from this entity because
/// SeaORM has no native PostGIS type. Geo reads/writes use raw SQL.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// 'individual' | 'organization'
    pub account_type: String,
    pub status: String,

    // ── Organization Identity ──────────────────────────────────────────────
    /// Legal company name (or full name for individuals)
    pub name: String,
    /// "Doing business as" — separate from legal name
    pub dba_name: Option<String>,
    pub website: Option<String>,
    /// Extracted domain — primary dedup anchor; unique partial index enforces
    /// no two non-duplicate accounts share the same domain per tenant.
    pub domain: Option<String>,
    /// D-U-N-S number — gold-standard business dedup key
    pub duns_number: Option<String>,
    /// Primary tax ID (EIN for US orgs, CNPJ for Brazil orgs, CPF for Brazil individuals)
    pub tax_id_primary: Option<String>,
    pub tax_id_secondary: Option<String>,
    /// 'ein' | 'cnpj' | 'cpf' | 'ssn' | 'tin' | 'vat' | 'usdot'
    pub tax_id_type: Option<String>,

    // ── Individual Identity (only when account_type = 'individual') ─────────
    pub first_name: Option<String>,
    pub last_name: Option<String>,

    // ── General Contact (company-level, not person-specific) ────────────────
    pub company_phone: Option<String>,
    pub company_email: Option<String>,
    pub company_fax: Option<String>,

    // ── Firmographic Data ──────────────────────────────────────────────────
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub sic_code: Option<String>,
    pub naics_code: Option<String>,
    pub num_employees: Option<i32>,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))", nullable)]
    pub annual_revenue: Option<Decimal>,
    /// 'public' | 'private' | 'government' | 'nonprofit' | 'individual'
    pub company_type: Option<String>,
    /// 'headquarters' | 'branch' | 'single' | 'franchise'
    pub location_type: Option<String>,
    pub year_established: Option<i16>,
    pub credit_score_code: Option<String>,

    // ── Address ───────────────────────────────────────────────────────────
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    /// Separate mailing address as JSONB (rarely queried — kept as JSONB per spec).
    /// Structure: {"street": "...", "city": "...", "state": "...", "zip": "...", "country": "..."}
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub mailing_address: Option<Value>,
    // NOTE: geo_point (geometry(Point, 4326)) is absent — use raw SQL.

    // ── Relationships ──────────────────────────────────────────────────────
    /// FK to atlas_contacts — set after contacts are created for this account
    pub primary_contact_id: Option<Uuid>,

    // ── Import Attribution ─────────────────────────────────────────────────
    /// 'manual' | 'zoominfo' | 'business_leads_usa' | 'fmcsa' | 'mwbe' | 'linkedin'
    pub data_source: Option<String>,
    /// Original record ID in source system — import dedup anchor
    pub data_source_id: Option<String>,
    pub is_duplicate: bool,
    /// Self-reference to canonical record this row duplicates
    pub duplicate_of_account_id: Option<Uuid>,

    // ── Overflow ──────────────────────────────────────────────────────────
    /// Vertical-specific fields: carrier FMCSA data, MWBE certifications,
    /// financial health signals (bankruptcy, liens, judgments), etc.
    /// See g31_atlas_lead_spec.md section 4.2 for structured patterns.
    /// Renamed from `attributes` (migration: m20260702_gap_fill_accounts_contacts).
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub account_metadata: Option<Value>,

    // ── Timestamps ────────────────────────────────────────────────────────
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    /// Auto-updated by Postgres trigger `trg_atlas_accounts_updated_at`.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
