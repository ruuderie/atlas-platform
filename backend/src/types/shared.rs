//! Shared value-object types used across multiple platform generics.
//!
//! # `MailingAddress`
//! Used identically in `atlas_lead.mailing_address` and
//! `atlas_accounts.mailing_address`. One struct eliminates two `serde_json::Value`
//! columns and deduplicates validation.
//!
//! # Conversion contract (same as all typed JSONB in this codebase)
//!   Entity layer:  `Option<serde_json::Value>`
//!   Service layer: `serde_json::from_value::<MailingAddress>(raw)?` to read,
//!                  `serde_json::to_value(&typed)?` to write.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ── Mailing address ───────────────────────────────────────────────────────────

/// A structured mailing / postal address.
///
/// Used in:
///  - `atlas_lead.mailing_address` JSONB
///  - `atlas_accounts.mailing_address` JSONB
///
/// All fields are optional to accommodate partial data from bulk imports.
/// The fields follow the USPS/Brazil common pattern; `state_province` covers
/// both US states and Brazilian estados.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MailingAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// US state abbreviation or full state/province name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// ZIP or postal code.
    #[serde(alias = "zip", skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// ISO 3166-1 alpha-2 country code (e.g. "US", "BR").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

impl MailingAddress {
    /// Returns `true` if none of the address fields are populated.
    pub fn is_empty(&self) -> bool {
        self.street.is_none()
            && self.street2.is_none()
            && self.city.is_none()
            && self.state.is_none()
            && self.postal_code.is_none()
            && self.country.is_none()
    }

    /// Format a single-line display string for UI / logging.
    /// Returns `None` if the address is entirely empty.
    pub fn one_line(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let mut parts = Vec::new();
        if let Some(s) = &self.street {
            parts.push(s.as_str());
        }
        if let Some(c) = &self.city {
            parts.push(c.as_str());
        }
        if let Some(st) = &self.state {
            parts.push(st.as_str());
        }
        if let Some(z) = &self.postal_code {
            parts.push(z.as_str());
        }
        if let Some(co) = &self.country {
            parts.push(co.as_str());
        }

        Some(parts.join(", "))
    }
}
