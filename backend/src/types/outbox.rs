//! Canonical Rust types for the `outbox_job` infrastructure.
//!
//! # Rule
//! `OutboxJobType` replaces the raw `String` in `outbox_job.job_type`.
//! `OutboxJobStatus` replaces the raw `String` in `outbox_job.status`.
//!
//! `outbox_worker.rs` dispatches on `OutboxJobType` — adding a new job type
//! here forces the compiler to demand a new match arm in the worker.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Job types ─────────────────────────────────────────────────────────────────

/// All registered outbox job types.
///
/// Stored as VARCHAR in `outbox_job.job_type`.
/// The `OutboxWorker::process_next_job` dispatch matches on this enum —
/// the `_ => Err(...)` fallback is eliminated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxJobType {
    // ── Email / comms ────────────────────────────────────────────────────────
    /// Send a magic-link authentication email via the email handler.
    SendMagicLinkEmail,

    /// Send a waitlist confirmation email to a new lead who just signed up.
    /// Payload: `{ to_email, name, product_slug, position, variant_slug? }`
    SendWaitlistConfirmation,

    // ── G-27 Scorecard compute ───────────────────────────────────────────────
    /// Recompute dimension aggregates + composite score for one scorecard or
    /// perform a tenant-wide sweep. Runs every 5 minutes.
    RecomputeScorecardAggregates,

    /// Rebuild monthly + quarterly time-series trend buckets for all
    /// scorecard dimensions for this tenant. Runs hourly.
    RefreshScorecardTimeSeries,

    /// Refresh portfolio analytics MV + rerank percentiles for all templates.
    /// Runs every 4 hours.
    RefreshScorecardPortfolio,

    /// Compute per-contributor calibration (bias_offset, scale_factor) for all
    /// published templates. Runs weekly.
    CalibrateScorecardContributors,

    /// Evaluate G-27 display rules after an activity is logged and push nudge
    /// dimensions to the rater via WebSocket (G-07).
    EvaluateScorecardNudge,

    // ── G-19 Reservations ────────────────────────────────────────────────────
    /// Release reservation holds that passed their hold expiry timestamp.
    ReleaseExpiredReservationHolds,

    // ── G-07 ext: Notifications ──────────────────────────────────────────────
    /// Deliver a notification via an external channel (telegram/whatsapp/sms/email).
    /// Payload: notification_service::NotifyChannelPayload
    NotifyChannel,
}

impl fmt::Display for OutboxJobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SendMagicLinkEmail               => "send_magic_link_email",
            Self::SendWaitlistConfirmation          => "send_waitlist_confirmation",
            Self::RecomputeScorecardAggregates     => "recompute_scorecard_aggregates",
            Self::RefreshScorecardTimeSeries       => "refresh_scorecard_time_series",
            Self::RefreshScorecardPortfolio        => "refresh_scorecard_portfolio",
            Self::CalibrateScorecardContributors   => "calibrate_scorecard_contributors",
            Self::EvaluateScorecardNudge           => "evaluate_scorecard_nudge",
            Self::ReleaseExpiredReservationHolds   => "release_expired_reservation_holds",
            Self::NotifyChannel                    => "notify_channel",
        })
    }
}

impl TryFrom<String> for OutboxJobType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "send_magic_link_email"              => Ok(Self::SendMagicLinkEmail),
            "send_waitlist_confirmation"         => Ok(Self::SendWaitlistConfirmation),
            "recompute_scorecard_aggregates"     => Ok(Self::RecomputeScorecardAggregates),
            "refresh_scorecard_time_series"      => Ok(Self::RefreshScorecardTimeSeries),
            "refresh_scorecard_portfolio"        => Ok(Self::RefreshScorecardPortfolio),
            "calibrate_scorecard_contributors"   => Ok(Self::CalibrateScorecardContributors),
            "evaluate_scorecard_nudge"           => Ok(Self::EvaluateScorecardNudge),
            "release_expired_reservation_holds"  => Ok(Self::ReleaseExpiredReservationHolds),
            "notify_channel"                     => Ok(Self::NotifyChannel),
            other                                => Err(format!("unknown OutboxJobType: '{other}'")),
        }
    }
}

impl TryFrom<&str> for OutboxJobType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

// ── Job status ────────────────────────────────────────────────────────────────

/// Lifecycle status for a row in `outbox_job`.
///
/// Stored as VARCHAR in `outbox_job.status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxJobStatus {
    /// Waiting for a worker to pick it up.
    Pending,
    /// Checked out by a worker — in-flight.
    Processing,
    /// Completed without error.
    Completed,
    /// Failed; will be retried if `attempts < 5`.
    Failed,
}

impl fmt::Display for OutboxJobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pending    => "pending",
            Self::Processing => "processing",
            Self::Completed  => "completed",
            Self::Failed     => "failed",
        })
    }
}

impl TryFrom<String> for OutboxJobStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "pending"    => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed"  => Ok(Self::Completed),
            "failed"     => Ok(Self::Failed),
            other        => Err(format!("unknown OutboxJobStatus: '{other}'")),
        }
    }
}
