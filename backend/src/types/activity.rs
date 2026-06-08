//! Canonical Rust types for G-29 `atlas_activity` domain concepts.
//!
//! # Rule
//! All discriminator `String` fields on the `activity` table have typed
//! Rust equivalents here. Services call `TryFrom<String>` after reading;
//! `Display::fmt` before writing.
//!
//! `ActivityMetadata` is the typed union for the `activity_metadata` JSONB
//! column. Use `serde_json::from_value::<ActivityMetadata>(raw)?` to
//! deserialize from the entity layer.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Legacy type discriminator ─────────────────────────────────────────────────

/// Legacy `activity_type` discriminator.
///
/// Stored as VARCHAR in `activity.activity_type`. Kept for backward compat with
/// handlers that have not yet migrated to `ActivityCategory`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    Log,
    Task,
    Event,
}

impl fmt::Display for ActivityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Log   => "Log",
            Self::Task  => "Task",
            Self::Event => "Event",
        })
    }
}

impl TryFrom<String> for ActivityType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "Log"   => Ok(Self::Log),
            "Task"  => Ok(Self::Task),
            "Event" => Ok(Self::Event),
            other   => Err(format!("unknown ActivityType: '{other}'")),
        }
    }
}

// ── Legacy status ─────────────────────────────────────────────────────────────

/// Legacy `status` field on the activity table.
///
/// Stored as VARCHAR in `activity.status`. Kept for backward compat.
/// New code should rely on `completed_at` / `due_date` instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityStatus {
    Open,
    Pending,
    Completed,
}

impl ActivityStatus {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

impl fmt::Display for ActivityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Open      => "Open",
            Self::Pending   => "Pending",
            Self::Completed => "Completed",
        })
    }
}

impl TryFrom<String> for ActivityStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "Open"      => Ok(Self::Open),
            "Pending"   => Ok(Self::Pending),
            "Completed" => Ok(Self::Completed),
            other       => Err(format!("unknown ActivityStatus: '{other}'")),
        }
    }
}

// ── G-29 platform category ────────────────────────────────────────────────────

/// Platform-level discriminator for the kind of activity.
///
/// Stored as VARCHAR in `activity.activity_category`.
/// Introduced in G-29; `activity_type` is the legacy equivalent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityCategory {
    /// Phone call, email, SMS, WhatsApp.
    Communication,
    /// In-person or virtual meeting.
    Meeting,
    /// Follow-up task or to-do.
    Task,
    /// Automated platform event (e.g. scorecard recompute, webhook delivery).
    SystemEvent,
    /// Stage transition or lifecycle gate (e.g. lead → opportunity).
    PipelineEvent,
}

impl ActivityCategory {
    /// Returns `true` for categories that represent direct human interaction.
    pub fn is_human_interaction(&self) -> bool {
        matches!(self, Self::Communication | Self::Meeting)
    }
}

impl fmt::Display for ActivityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Communication => "communication",
            Self::Meeting       => "meeting",
            Self::Task          => "task",
            Self::SystemEvent   => "system_event",
            Self::PipelineEvent => "pipeline_event",
        })
    }
}

impl TryFrom<String> for ActivityCategory {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "communication"  => Ok(Self::Communication),
            "meeting"        => Ok(Self::Meeting),
            "task"           => Ok(Self::Task),
            "system_event"   => Ok(Self::SystemEvent),
            "pipeline_event" => Ok(Self::PipelineEvent),
            other            => Err(format!("unknown ActivityCategory: '{other}'")),
        }
    }
}

// ── Direction ─────────────────────────────────────────────────────────────────

/// Communication direction.
///
/// Stored as VARCHAR in `activity.direction`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityDirection {
    Inbound,
    Outbound,
    /// Not applicable (meetings, tasks, etc.)
    Na,
}

impl fmt::Display for ActivityDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Inbound  => "inbound",
            Self::Outbound => "outbound",
            Self::Na       => "n_a",
        })
    }
}

impl TryFrom<String> for ActivityDirection {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "inbound"  => Ok(Self::Inbound),
            "outbound" => Ok(Self::Outbound),
            "n_a"      => Ok(Self::Na),
            other      => Err(format!("unknown ActivityDirection: '{other}'")),
        }
    }
}

// ── Outcome ───────────────────────────────────────────────────────────────────

/// Result of a communication or meeting activity.
///
/// Stored as VARCHAR in `activity.outcome`.
///
/// `is_completed_communication()` on `atlas_activity::Model` MUST be updated
/// whenever a new `Outcome` variant that represents a completed interaction is added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityOutcome {
    // ── Call / communication outcomes ────────────────────────────────────────
    /// Call connected and conversation took place.
    Connected,
    /// Call went to voicemail — no live conversation.
    Voicemail,
    /// No answer — call not picked up.
    NoAnswer,
    /// Email bounced.
    Bounced,
    // ── Meeting outcomes ─────────────────────────────────────────────────────
    /// Meeting happened as scheduled.
    MeetingHeld,
    /// Meeting scheduled but attendee(s) did not appear.
    NoShow,
    // ── Generic ──────────────────────────────────────────────────────────────
    /// Task or event was completed successfully.
    Completed,
    /// Activity was cancelled before it occurred.
    Cancelled,
}

impl ActivityOutcome {
    /// Returns `true` for outcomes that represent a completed, substantive
    /// interaction (live call, held meeting, completed task).
    ///
    /// This is the typed equivalent of `Model::is_completed_communication()`.
    pub fn is_completed_interaction(&self) -> bool {
        matches!(self, Self::Connected | Self::MeetingHeld | Self::Completed)
    }
}

impl fmt::Display for ActivityOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Connected  => "connected",
            Self::Voicemail  => "voicemail",
            Self::NoAnswer   => "no_answer",
            Self::Bounced    => "bounced",
            Self::MeetingHeld => "meeting_held",
            Self::NoShow     => "no_show",
            Self::Completed  => "completed",
            Self::Cancelled  => "cancelled",
        })
    }
}

impl TryFrom<String> for ActivityOutcome {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "connected"   => Ok(Self::Connected),
            "voicemail"   => Ok(Self::Voicemail),
            "no_answer"   => Ok(Self::NoAnswer),
            "bounced"     => Ok(Self::Bounced),
            "meeting_held" => Ok(Self::MeetingHeld),
            "no_show"     => Ok(Self::NoShow),
            "completed"   => Ok(Self::Completed),
            "cancelled"   => Ok(Self::Cancelled),
            other         => Err(format!("unknown ActivityOutcome: '{other}'")),
        }
    }
}

// ── Typed JSONB structs for activity_metadata ─────────────────────────────────
//
// Contract:
//   Entity layer:  `pub activity_metadata: Option<serde_json::Value>`
//   Service layer: `serde_json::from_value::<ActivityMetadata>(raw)?` to read,
//                  `serde_json::to_value(&typed)?` to write.

/// Metadata for a recorded call activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}

/// Metadata for an email activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmailMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

/// Metadata for a meeting activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeetingMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attendees: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meeting_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording_url: Option<String>,
}

/// Typed union for the `activity_metadata` JSONB column.
///
/// `#[serde(untagged)]` preserves backward compatibility with existing JSONB data —
/// the variant is determined by field shapes, not by a type discriminator key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityMetadata {
    Call(CallMetadata),
    Email(EmailMetadata),
    Meeting(MeetingMetadata),
    /// Catch-all for activity types without a typed struct yet.
    Generic(serde_json::Value),
}
