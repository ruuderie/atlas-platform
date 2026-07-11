//! G-06 verification domain types.
//!
//! Persist via `Display` / `.to_string()`; parse with `FromStr` at the API boundary.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Lifecycle status for `atlas_verification_requests.status`.
///
/// DB strings used by the admin queue UI and handlers today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Pending,
    Review,
    Approved,
    Rejected,
    NeedsInfo,
}

impl VerificationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Review => "review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::NeedsInfo => "needs_info",
        }
    }
}

impl fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VerificationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" | "pending_upload" | "queued" => Ok(Self::Pending),
            "review" | "requires_manual_review" | "auto_checking" => Ok(Self::Review),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "needs_info" => Ok(Self::NeedsInfo),
            other => Err(format!("unknown VerificationStatus: '{other}'")),
        }
    }
}

impl TryFrom<&str> for VerificationStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl TryFrom<String> for VerificationStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Kind of verification being requested (shown as `req_type` in the admin queue).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationRequestType {
    Business,
    Identity,
    Document,
}

impl VerificationRequestType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Business => "business",
            Self::Identity => "identity",
            Self::Document => "document",
        }
    }
}

impl fmt::Display for VerificationRequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VerificationRequestType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "business" => Ok(Self::Business),
            "identity" | "kyc" => Ok(Self::Identity),
            "document" => Ok(Self::Document),
            other => Err(format!("unknown VerificationRequestType: '{other}'")),
        }
    }
}

impl TryFrom<&str> for VerificationRequestType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl TryFrom<String> for VerificationRequestType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Polymorphic subject being verified (`atlas_verification_requests.subject_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationSubjectType {
    Tenant,
    User,
    Asset,
}

impl VerificationSubjectType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tenant => "tenant",
            Self::User => "user",
            Self::Asset => "asset",
        }
    }
}

impl fmt::Display for VerificationSubjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VerificationSubjectType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "tenant" | "business" => Ok(Self::Tenant),
            "user" | "identity" => Ok(Self::User),
            "asset" | "document" => Ok(Self::Asset),
            other => Err(format!("unknown VerificationSubjectType: '{other}'")),
        }
    }
}

impl TryFrom<&str> for VerificationSubjectType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl TryFrom<String> for VerificationSubjectType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}
