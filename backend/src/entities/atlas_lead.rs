#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use rust_decimal::Decimal;

/// G-31: atlas_lead — Canonical lead / prospect entity.
///
/// This is the entry point for all inbound interest before qualification.
/// A lead becomes an opportunity only through an explicit human gate event
/// (the "Convert" action), which atomically creates:
///   - atlas_accounts record (the organization)
///   - atlas_contacts record (the person)
///   - atlas_opportunities record (the pipeline item)
///
/// See: docs/architecture/g31_atlas_lead_spec.md
///
/// # Field Notes
///
/// `name` (NOT NULL): Computed by the service layer as:
///   COALESCE(first_name + " " + last_name, company, email, "Unknown")
///   Never null at the application layer.
///
/// `geo_point`: Stored as a WKT string (e.g. "POINT(-73.98 40.76)") because
///   SeaORM has no native PostGIS type. Read/write via raw SQL or a helper
///   that parses WKT. Column may not exist in environments without PostGIS.
///
/// `lead_metadata` (JSONB): Industry-specific overflow. See spec section 4.2
///   for the structured patterns used by each vertical (FMCSA, MWBE, financial
///   health, professional profile enrichment).
///
/// `data_source` + `data_source_id`: Dedup anchor for batch imports. Without
///   these, re-importing the same source file creates duplicates.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_lead")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,

    // ── Contact — Individual ───────────────────────────────────────────────
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    /// Computed display name. Service layer MUST set this.
    /// Rule: COALESCE(first_name + last_name, company, email, "Unknown")
    pub name: String,
    pub title: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub fax: Option<String>,
    // Social channels
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub linkedin_url: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub avatar_url: Option<String>,

    // ── Company / Organization ─────────────────────────────────────────────
    pub company: Option<String>,
    /// "Doing business as" name — separate from legal company name
    pub company_dba: Option<String>,
    pub company_website: Option<String>,
    /// Extracted domain (from email or website). Primary company-level dedup key.
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub num_employees: Option<i32>,
    #[sea_orm(column_type = "Decimal(Some((18, 2)))", nullable)]
    pub annual_revenue: Option<Decimal>,
    /// 'public' | 'private' | 'government' | 'nonprofit' | 'individual'
    pub company_type: Option<String>,
    /// 'headquarters' | 'branch' | 'single' | 'franchise'
    pub location_type: Option<String>,
    pub year_established: Option<i16>,

    // ── Industry Classification Codes ──────────────────────────────────────
    pub sic_code: Option<String>,
    pub naics_code: Option<String>,
    /// D-U-N-S number — gold-standard business identity dedup key
    pub duns_number: Option<String>,

    // ── Credit / Risk Signals ──────────────────────────────────────────────
    pub credit_score_code: Option<String>,

    // ── Address ───────────────────────────────────────────────────────────
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
    /// Separate mailing address as JSONB.
    /// Structure: {"street": "...", "city": "...", "state": "...", "zip": "...", "country": "..."}
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub mailing_address: Option<Value>,
    // NOTE: geo_point (geometry(Point, 4326)) is deliberately absent from this
    // entity because SeaORM has no native PostGIS type mapping. Geo reads/writes
    // must use raw SQL via the database connection. The column exists in the DB
    // when PostGIS is available.

    // ── Lead Metadata ──────────────────────────────────────────────────────
    pub message: Option<String>,
    /// Pipeline status. Valid values:
    /// 'new' | 'contacted' | 'qualifying' | 'qualified' | 'disqualified' | 'converted'
    pub lead_status: String,
    /// Original intake channel.
    /// 'manual' | 'web_form' | 'referral' | 'import' | 'zoominfo' | etc.
    pub source: Option<String>,
    /// Import source identifier. 'business_leads_usa' | 'zoominfo' | 'fmcsa' | etc.
    pub data_source: Option<String>,
    /// Original record ID in the source system — critical for import dedup.
    pub data_source_id: Option<String>,
    /// Industry/vertical-specific overflow. See spec section 4.2.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub lead_metadata: Option<Value>,

    // ── Deduplication ──────────────────────────────────────────────────────
    pub is_duplicate: bool,
    /// Self-reference to the canonical record this row duplicates.
    pub duplicate_of_lead_id: Option<Uuid>,

    // ── Linked Records ─────────────────────────────────────────────────────
    /// Which listing generated this lead (inbound form)
    pub listing_id: Option<Uuid>,
    /// Registered user account that submitted the form (legacy account table)
    pub account_id: Option<Uuid>,

    // ── Conversion ────────────────────────────────────────────────────────
    pub is_converted: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub converted_at: Option<DateTime<Utc>>,
    /// atlas_accounts record created during conversion (G-01 gap-fill)
    pub converted_account_id: Option<Uuid>,
    /// atlas_contacts record created during conversion
    pub converted_contact_id: Option<Uuid>,
    /// atlas_opportunities record created during conversion (G-15)
    pub converted_opportunity_id: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub disqualified_at: Option<DateTime<Utc>>,
    pub disqualification_reason: Option<String>,

    // ── Timestamps ────────────────────────────────────────────────────────
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    /// Auto-updated by Postgres trigger `trg_atlas_lead_updated_at`.
    /// Service layer must still set this on insert.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // Intentionally no FK constraints at the ORM level — consistent with
    // the platform pattern established in unify_accounts_contacts.rs.
}

impl ActiveModelBehavior for ActiveModel {}

// ── Helpers ──────────────────────────────────────────────────────────────────

impl Model {
    /// Compute the display name from available fields.
    /// Used by the service layer when inserting a new lead.
    pub fn compute_name(
        first_name: Option<&str>,
        last_name: Option<&str>,
        company: Option<&str>,
        email: Option<&str>,
    ) -> String {
        match (first_name, last_name) {
            (Some(f), Some(l)) if !f.is_empty() && !l.is_empty() => format!("{} {}", f, l),
            (Some(f), _) if !f.is_empty() => f.to_string(),
            (_, Some(l)) if !l.is_empty() => l.to_string(),
            _ => company
                .filter(|s| !s.is_empty())
                .or(email.filter(|s| !s.is_empty()))
                .unwrap_or("Unknown")
                .to_string(),
        }
    }

    /// Return true if this lead is in a terminal state (converted or disqualified).
    ///
    /// Panics in debug if `lead_status` contains an unregistered value — all writes
    /// must go through `LeadStatus::to_string()`, so this should never fire in practice.
    pub fn is_terminal(&self) -> bool {
        self.lead_status_typed()
            .unwrap_or_else(|e| panic!("corrupt lead_status '{}': {}", self.lead_status, e))
            .is_terminal()
    }

    /// Parse `lead_status` into the typed `LeadStatus` enum.
    ///
    /// Returns `Err` if the stored value is not a known variant — this should
    /// never happen in production but is caught defensively at the read boundary.
    pub fn lead_status_typed(
        &self,
    ) -> Result<crate::types::lead::LeadStatus, String> {
        crate::types::lead::LeadStatus::try_from(self.lead_status.as_str())
    }

    /// Deserialize `mailing_address` JSONB into the typed `MailingAddress` struct.
    ///
    /// Returns `None` if the column is NULL.
    /// Returns `Err` if the stored JSON does not match the expected shape.
    pub fn mailing_address_typed(
        &self,
    ) -> Result<Option<crate::types::shared::MailingAddress>, serde_json::Error> {
        match &self.mailing_address {
            Some(v) => serde_json::from_value(v.clone()).map(Some),
            None    => Ok(None),
        }
    }

    /// Parse `data_source` into the typed `DataSource` enum.
    ///
    /// Uses `From<String>` (not `TryFrom`) because `DataSource` has an `Other(String)` catch-all
    /// variant that ensures this never fails for unknown import sources.
    pub fn data_source_typed(&self) -> Option<crate::types::lead::DataSource> {
        self.data_source.as_ref().map(|s| crate::types::lead::DataSource::from(s.as_str()))
    }
}


