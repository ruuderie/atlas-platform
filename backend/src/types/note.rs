//! Canonical Rust types for G-28 `atlas_note` domain concepts.
//!
//! # Rule
//! `NoteType`, `NoteVisibility`, and `NoteEntityType` replace the raw `String`
//! discriminators on the `notes` table. Services call `TryFrom<String>` after
//! reading; `Display::fmt` before writing.
//!
//! `NoteMetadata` is the typed union for the `note_metadata` JSONB column.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Note visibility ───────────────────────────────────────────────────────────

/// Who can see a note.
///
/// Stored as VARCHAR in `notes.visibility`.
///
/// `effective_visibility()` on `atlas_note::Model` should return this enum
/// (or its `Display` string) for typed callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteVisibility {
    /// Visible to all parties including external (e.g. tenant portal).
    Public,
    /// Visible only to internal team members. **Default.**
    Internal,
    /// Visible only to the creating user.
    Private,
}

impl NoteVisibility {
    /// Returns `true` if external parties (e.g. tenant portal users) can see this note.
    pub fn is_external_visible(&self) -> bool {
        matches!(self, Self::Public)
    }
}

impl fmt::Display for NoteVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Public   => "public",
            Self::Internal => "internal",
            Self::Private  => "private",
        })
    }
}

impl TryFrom<String> for NoteVisibility {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "public"   => Ok(Self::Public),
            "internal" => Ok(Self::Internal),
            "private"  => Ok(Self::Private),
            other      => Err(format!("unknown NoteVisibility: '{other}'")),
        }
    }
}

impl TryFrom<&str> for NoteVisibility {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

// ── Note type ─────────────────────────────────────────────────────────────────

/// Application-defined discriminator for a note's purpose.
///
/// Stored as VARCHAR in `notes.note_type`.
/// Non-exhaustive by design — vertical apps may define custom types not listed
/// here. Use `NoteType::Other(String)` for those.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteType {
    /// Generic free-text note with no special handling. **Default.**
    General,
    /// Log of a phone call (linked to a call `activity`).
    CallLog,
    /// Note from an on-site visit (PM, insurance, inspections).
    SiteVisit,
    /// Physical property inspection report note.
    Inspection,
    /// Underwriter or risk-analyst comment.
    UnderwritingComment,
    /// Legal analysis or memo.
    LegalMemo,
    /// Compliance or regulatory note.
    ComplianceNote,
    /// Coach or manager feedback to a team member.
    CoachFeedback,
    /// App-specific / vertical-specific type not in this list.
    Other(String),
}

impl fmt::Display for NoteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::General              => f.write_str("general"),
            Self::CallLog              => f.write_str("call_log"),
            Self::SiteVisit            => f.write_str("site_visit"),
            Self::Inspection           => f.write_str("inspection"),
            Self::UnderwritingComment  => f.write_str("underwriting_comment"),
            Self::LegalMemo            => f.write_str("legal_memo"),
            Self::ComplianceNote       => f.write_str("compliance_note"),
            Self::CoachFeedback        => f.write_str("coach_feedback"),
            Self::Other(s)             => f.write_str(s),
        }
    }
}

impl From<String> for NoteType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "general"               => Self::General,
            "call_log"              => Self::CallLog,
            "site_visit"            => Self::SiteVisit,
            "inspection"            => Self::Inspection,
            "underwriting_comment"  => Self::UnderwritingComment,
            "legal_memo"            => Self::LegalMemo,
            "compliance_note"       => Self::ComplianceNote,
            "coach_feedback"        => Self::CoachFeedback,
            other                   => Self::Other(other.to_string()),
        }
    }
}

impl From<&str> for NoteType {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

// ── Typed JSONB structs for note_metadata ─────────────────────────────────────
//
// Contract (mirrors G-27 EntryContext pattern):
//   Entity layer:  `pub note_metadata: Option<serde_json::Value>`
//   Service layer: `serde_json::from_value::<NoteMetadata>(raw)?` to read.

/// Metadata for a rich-text note (Quill Delta or Tiptap JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextMetadata {
    /// The full document in the editor's native JSON format.
    pub delta: serde_json::Value,
}

/// Metadata for a call-transcript note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallTranscriptMetadata {
    /// URL to the recording file (Cloudflare R2 pre-signed URL or similar).
    pub url: String,
    /// Full transcript text for search indexing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i32>,
}

/// Typed union for the `note_metadata` JSONB column.
///
/// `#[serde(untagged)]` preserves backward compatibility — shape determines variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NoteMetadata {
    RichText(RichTextMetadata),
    CallTranscript(CallTranscriptMetadata),
    /// Catch-all for note types without a typed struct yet.
    Generic(serde_json::Value),
}
