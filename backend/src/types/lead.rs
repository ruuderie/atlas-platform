//! Canonical Rust types for G-31 `atlas_lead` domain concepts.
//!
//! # Rule
//! These types are the **source of truth** for all G-31 discriminator fields.
//! DB VARCHAR columns are derived from them via `Display::fmt`.
//! Services call `TryFrom<String>` immediately after reading from the DB.
//!
//! **Never** write raw string literals for these fields in service code.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Lead pipeline status ──────────────────────────────────────────────────────

/// Pipeline status for an `atlas_lead` record.
///
/// Stored as VARCHAR in `atlas_lead.lead_status`.
///
/// Terminal states: `Disqualified` and `Converted`.
/// `LeadModel::is_terminal()` uses this enum — add a new state here and
/// the compiler will require you to update every exhaustive match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadStatus {
    /// Freshly created — no outreach yet.
    New,
    /// First outreach sent or made.
    Contacted,
    /// Active qualification conversation in progress.
    Qualifying,
    /// Qualification complete — lead meets ICP criteria.
    Qualified,
    /// Rejected from pipeline. Terminal state.
    Disqualified,
    /// Successfully converted to Account + Contact + Opportunity. Terminal state.
    Converted,
}

impl LeadStatus {
    /// Returns `true` if this status is a terminal state from which no
    /// further lifecycle transitions are permitted.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Disqualified | Self::Converted)
    }
}

impl fmt::Display for LeadStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::New          => "new",
            Self::Contacted    => "contacted",
            Self::Qualifying   => "qualifying",
            Self::Qualified    => "qualified",
            Self::Disqualified => "disqualified",
            Self::Converted    => "converted",
        })
    }
}

impl TryFrom<String> for LeadStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "new"          => Ok(Self::New),
            "contacted"    => Ok(Self::Contacted),
            "qualifying"   => Ok(Self::Qualifying),
            "qualified"    => Ok(Self::Qualified),
            "disqualified" => Ok(Self::Disqualified),
            "converted"    => Ok(Self::Converted),
            other          => Err(format!("unknown LeadStatus: '{other}'")),
        }
    }
}

impl TryFrom<&str> for LeadStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

// ── Data source (import origin) ───────────────────────────────────────────────

/// Import source identifier for leads and accounts.
///
/// Stored as VARCHAR in `atlas_lead.data_source` and `atlas_accounts.data_source`.
/// Used by `create_from_import` to decide scorecard auto-provision (G-31 §8.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    /// FMCSA DOT registry (commercial trucking carrier data).
    Fmcsa,
    /// Business Leads USA / B2BLeadsUSA bulk import.
    BusinessLeadsUsa,
    /// FMCSA DOT registry (alternate slug used in some import scripts).
    DotRegistry,
    /// Manual entry by a team member.
    Manual,
    /// Web form submission (lead capture widget).
    WebForm,
    /// ZoomInfo enrichment import.
    Zoominfo,
    /// LinkedIn Sales Navigator export.
    Linkedin,
    /// MWBE (Minority/Women-Owned Business Enterprise) registry.
    Mwbe,
    /// Referral from another lead or user.
    Referral,
    /// Unknown / catch-all for legacy rows.
    Other(String),
}

impl DataSource {
    /// Returns `true` if this source triggers scorecard auto-provision (G-31 §8.3).
    pub fn auto_provisions_scorecard(&self) -> bool {
        matches!(
            self,
            Self::Fmcsa | Self::BusinessLeadsUsa | Self::DotRegistry
        )
    }
}

impl fmt::Display for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fmcsa           => f.write_str("fmcsa"),
            Self::BusinessLeadsUsa => f.write_str("business_leads_usa"),
            Self::DotRegistry     => f.write_str("dot_registry"),
            Self::Manual          => f.write_str("manual"),
            Self::WebForm         => f.write_str("web_form"),
            Self::Zoominfo        => f.write_str("zoominfo"),
            Self::Linkedin        => f.write_str("linkedin"),
            Self::Mwbe            => f.write_str("mwbe"),
            Self::Referral        => f.write_str("referral"),
            Self::Other(s)        => f.write_str(s),
        }
    }
}

impl From<String> for DataSource {
    fn from(s: String) -> Self {
        match s.as_str() {
            "fmcsa"              => Self::Fmcsa,
            "business_leads_usa" => Self::BusinessLeadsUsa,
            "dot_registry"       => Self::DotRegistry,
            "manual"             => Self::Manual,
            "web_form"           => Self::WebForm,
            "zoominfo"           => Self::Zoominfo,
            "linkedin"           => Self::Linkedin,
            "mwbe"               => Self::Mwbe,
            "referral"           => Self::Referral,
            other                => Self::Other(other.to_string()),
        }
    }
}

impl From<&str> for DataSource {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

// ── Company type ──────────────────────────────────────────────────────────────

/// Legal classification of a company or organization.
///
/// Stored as VARCHAR in `atlas_lead.company_type` and `atlas_accounts.company_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyType {
    Public,
    Private,
    Government,
    Nonprofit,
    Individual,
}

impl fmt::Display for CompanyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Public     => "public",
            Self::Private    => "private",
            Self::Government => "government",
            Self::Nonprofit  => "nonprofit",
            Self::Individual => "individual",
        })
    }
}

impl TryFrom<String> for CompanyType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "public"     => Ok(Self::Public),
            "private"    => Ok(Self::Private),
            "government" => Ok(Self::Government),
            "nonprofit"  => Ok(Self::Nonprofit),
            "individual" => Ok(Self::Individual),
            other        => Err(format!("unknown CompanyType: '{other}'")),
        }
    }
}

// ── Location type ─────────────────────────────────────────────────────────────

/// Classification of a company's physical presence.
///
/// Stored as VARCHAR in `atlas_lead.location_type` and `atlas_accounts.location_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationType {
    Headquarters,
    Branch,
    Single,
    Franchise,
}

impl fmt::Display for LocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Headquarters => "headquarters",
            Self::Branch       => "branch",
            Self::Single       => "single",
            Self::Franchise    => "franchise",
        })
    }
}

impl TryFrom<String> for LocationType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "headquarters" => Ok(Self::Headquarters),
            "branch"       => Ok(Self::Branch),
            "single"       => Ok(Self::Single),
            "franchise"    => Ok(Self::Franchise),
            other          => Err(format!("unknown LocationType: '{other}'")),
        }
    }
}

// ── Opportunity type / status ─────────────────────────────────────────────────

/// High-level category for an atlas_opportunity record.
///
/// Stored as VARCHAR in `atlas_opportunities.opportunity_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityType {
    /// Auto-created by `LeadService::convert_lead`.
    CrmLeadConversion,
    /// Manually created by a sales rep.
    Manual,
    /// Renewal of an existing contract.
    Renewal,
    /// Upsell on an existing account.
    Upsell,
    /// Channel/partner deal.
    Partner,
}

impl fmt::Display for OpportunityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CrmLeadConversion => "crm_lead_conversion",
            Self::Manual            => "manual",
            Self::Renewal           => "renewal",
            Self::Upsell            => "upsell",
            Self::Partner           => "partner",
        })
    }
}

impl TryFrom<String> for OpportunityType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "crm_lead_conversion" => Ok(Self::CrmLeadConversion),
            "manual"              => Ok(Self::Manual),
            "renewal"             => Ok(Self::Renewal),
            "upsell"              => Ok(Self::Upsell),
            "partner"             => Ok(Self::Partner),
            other                 => Err(format!("unknown OpportunityType: '{other}'")),
        }
    }
}

/// Pipeline stage for an atlas_opportunity record.
///
/// Stored as VARCHAR in `atlas_opportunities.status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStatus {
    Prospecting,
    Qualification,
    Proposal,
    Negotiation,
    ClosedWon,
    ClosedLost,
}

impl OpportunityStatus {
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::ClosedWon | Self::ClosedLost)
    }
}

impl fmt::Display for OpportunityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Prospecting  => "prospecting",
            Self::Qualification => "qualification",
            Self::Proposal     => "proposal",
            Self::Negotiation  => "negotiation",
            Self::ClosedWon    => "closed_won",
            Self::ClosedLost   => "closed_lost",
        })
    }
}

impl TryFrom<String> for OpportunityStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "prospecting"   => Ok(Self::Prospecting),
            "qualification" => Ok(Self::Qualification),
            "proposal"      => Ok(Self::Proposal),
            "negotiation"   => Ok(Self::Negotiation),
            "closed_won"    => Ok(Self::ClosedWon),
            "closed_lost"   => Ok(Self::ClosedLost),
            other           => Err(format!("unknown OpportunityStatus: '{other}'")),
        }
    }
}
