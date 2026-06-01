use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Unified Contact entity — represents individual people.
///
/// Every contact belongs to an Account (atlas_accounts). The account is the
/// organization; the contact is the person at that organization.
///
/// Schema extended by: docs/architecture/account_contact_data_gap_analysis.md
/// Migration: m20260702_gap_fill_accounts_contacts.rs
///
/// # Channel Priority for B2B Latin America
/// WhatsApp is a first-class channel (not a JSONB afterthought) because it is
/// the dominant B2B communication channel across Brazil, Haiti, and LATAM markets.
///
/// # Sensitive Fields
/// `gender` and `birth_year` (from Haiti/LinkedIn dataset) live in
/// `contact_metadata` JSONB — never as indexed columns.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_contacts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// FK to atlas_accounts — always required
    pub account_id: Uuid,

    // ── Name ──────────────────────────────────────────────────────────────
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    /// Computed full name (stored for search performance) or None if unavailable
    pub full_name: Option<String>,
    /// Nickname / preferred form of address
    pub preferred_name: Option<String>,

    // ── Professional Context ───────────────────────────────────────────────
    /// e.g. "VP Operations", "Owner", "CEO"
    pub title: Option<String>,
    /// e.g. "Finance", "Operations", "Sales"
    pub department: Option<String>,
    pub is_primary: bool,

    // ── Contact Channels ──────────────────────────────────────────────────
    pub email: Option<String>,
    /// Set to true for contacts verified via MillionVerifier output files
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub fax: Option<String>,
    /// Critical for Latin America B2B — first-class column, not JSONB
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub linkedin_url: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub avatar_url: Option<String>,

    // ── Tax Identity ──────────────────────────────────────────────────────
    /// For individual contacts used as B2C customers (CPF, SSN, NIF, TIN).
    /// Migrated from legacy `customer.cpf` column.
    pub tax_id: Option<String>,
    /// 'cpf' | 'ssn' | 'tin' | 'nif'
    pub tax_id_type: Option<String>,

    // ── Import Attribution ─────────────────────────────────────────────────
    pub data_source: Option<String>,
    pub data_source_id: Option<String>,
    pub is_duplicate: bool,
    /// Self-reference to canonical record this row duplicates
    pub duplicate_of_contact_id: Option<Uuid>,

    // ── Overflow ──────────────────────────────────────────────────────────
    /// Gender, birth_year, skills, social profile URLs, geo details.
    /// Sensitive fields (gender, birth_year) always stay in JSONB — never indexed.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub contact_metadata: Option<Value>,

    // ── Timestamps ────────────────────────────────────────────────────────
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    /// Auto-updated by Postgres trigger `trg_atlas_contacts_updated_at`.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
