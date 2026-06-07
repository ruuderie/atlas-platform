//! Canonical Rust types for the `atlas_accounts` (G-01 party model) domain.
//!
//! # Rule
//! `AccountType`, `AccountStatus`, and `TaxIdType` replace raw `String` discriminators.
//! Services call `TryFrom<String>` after reading; `Display::fmt` before writing.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Account type ──────────────────────────────────────────────────────────────

/// Whether the account represents a natural person or a legal entity.
///
/// Stored as VARCHAR in `atlas_accounts.account_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// A natural person (B2C, individual contractor, sole trader).
    Individual,
    /// A company, nonprofit, government body, etc.
    Organization,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Individual   => "individual",
            Self::Organization => "organization",
        })
    }
}

impl TryFrom<String> for AccountType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "individual"   => Ok(Self::Individual),
            "organization" => Ok(Self::Organization),
            other          => Err(format!("unknown AccountType: '{other}'")),
        }
    }
}

impl TryFrom<&str> for AccountType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

// ── Account status ────────────────────────────────────────────────────────────

/// Lifecycle status for an atlas_accounts record.
///
/// Stored as VARCHAR in `atlas_accounts.status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// Account created from lead conversion — not yet confirmed as a customer.
    Prospect,
    /// Active customer with a live relationship.
    Active,
    /// Temporarily suspended.
    Suspended,
    /// Closed / churned.
    Archived,
}

impl AccountStatus {
    pub fn is_active_relationship(&self) -> bool {
        matches!(self, Self::Active | Self::Prospect)
    }
}

impl fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Prospect => "prospect",
            Self::Active   => "active",
            Self::Suspended => "suspended",
            Self::Archived => "archived",
        })
    }
}

impl TryFrom<String> for AccountStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "prospect"  => Ok(Self::Prospect),
            "active"    => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "archived"  => Ok(Self::Archived),
            other       => Err(format!("unknown AccountStatus: '{other}'")),
        }
    }
}

// ── Tax ID type ───────────────────────────────────────────────────────────────

/// Discriminator for the type of tax identifier stored in `tax_id_primary`.
///
/// Stored as VARCHAR in `atlas_accounts.tax_id_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxIdType {
    /// US Employer Identification Number.
    Ein,
    /// Brazil national company registry.
    Cnpj,
    /// Brazil individual taxpayer registry.
    Cpf,
    /// US Social Security Number.
    Ssn,
    /// Taxpayer Identification Number (catch-all).
    Tin,
    /// EU/UK Value Added Tax number.
    Vat,
    /// US Department of Transportation number (trucking carriers).
    Usdot,
}

impl fmt::Display for TaxIdType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ein   => "ein",
            Self::Cnpj  => "cnpj",
            Self::Cpf   => "cpf",
            Self::Ssn   => "ssn",
            Self::Tin   => "tin",
            Self::Vat   => "vat",
            Self::Usdot => "usdot",
        })
    }
}

impl TryFrom<String> for TaxIdType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "ein"   => Ok(Self::Ein),
            "cnpj"  => Ok(Self::Cnpj),
            "cpf"   => Ok(Self::Cpf),
            "ssn"   => Ok(Self::Ssn),
            "tin"   => Ok(Self::Tin),
            "vat"   => Ok(Self::Vat),
            "usdot" => Ok(Self::Usdot),
            other   => Err(format!("unknown TaxIdType: '{other}'")),
        }
    }
}
