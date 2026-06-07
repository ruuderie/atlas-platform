//! Canonical Rust types for G-27 Atlas Scorecards.
//!
//! # Rule
//! These types are the **source of truth** for all G-27 domain concepts.
//! DB VARCHAR columns and JSON string values are derived from them via serde.
//!
//! **Never** match on `.as_str()` in service code — match on these enums.
//! The compiler enforces exhaustiveness; a missing `unknown =>` fallback arm
//! is not a safety net, it is a silent bug suppressor.
//!
//! # Boundary contract
//! - Entity models keep `String` at the SeaORM DB boundary.
//! - Services call `TryFrom<String>` immediately after reading from the DB.
//! - Services call `Display::fmt` (via `.to_string()`) immediately before writing.
//! - JSON serialization/deserialization is driven entirely by serde derives.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ── Scoring method ────────────────────────────────────────────────────────────

/// How a template computes its composite score.
///
/// Stored as VARCHAR in `atlas_scorecard_templates.scoring_method`.
/// The existing `weighted_mean` and `simple_mean` paths are extended;
/// `percentile_rank` is the new path enabled by Phase 3 analytics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoringMethod {
    /// Dimension-weight-scaled average (confidence-weighted in Phase 1).
    WeightedMean,
    /// Simple unweighted arithmetic mean.
    SimpleMean,
    /// Composite = percentile rank in tenant pool / 10.0. Requires Phase 3.
    PercentileRank,
}

impl fmt::Display for ScoringMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::WeightedMean   => "weighted_mean",
            Self::SimpleMean     => "simple_mean",
            Self::PercentileRank => "percentile_rank",
        })
    }
}

impl TryFrom<String> for ScoringMethod {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "weighted_mean"   => Ok(Self::WeightedMean),
            "simple_mean"     => Ok(Self::SimpleMean),
            "percentile_rank" => Ok(Self::PercentileRank),
            other => Err(format!("unknown ScoringMethod: '{other}'")),
        }
    }
}

// ── Cold-start strategy ───────────────────────────────────────────────────────

/// Strategy for displaying a scorecard when entry count < `min_entries_to_publish`.
///
/// Stored as VARCHAR in `atlas_scorecard_templates.cold_start_strategy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColdStartStrategy {
    /// Show nothing until min_entries_to_publish is met. (Default)
    Suppress,
    /// Show `global_reference_value` as a labelled Bayesian estimate.
    Prior,
    /// Show category-pool average as estimate. Currently maps to `Prior`.
    Category,
}

impl fmt::Display for ColdStartStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Suppress => "suppress",
            Self::Prior    => "prior",
            Self::Category => "category",
        })
    }
}

impl TryFrom<String> for ColdStartStrategy {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "suppress" => Ok(Self::Suppress),
            "prior"    => Ok(Self::Prior),
            "category" => Ok(Self::Category),
            other => Err(format!("unknown ColdStartStrategy: '{other}'")),
        }
    }
}

// ── Percentile band ───────────────────────────────────────────────────────────

/// Categorical band from `percentile_rank` for `<PercentileRankBadge>` UI component.
///
/// Stored as VARCHAR in `atlas_scorecard_dimension_aggregates.percentile_band`.
/// Computed by `ScorecardService::compute_percentile_ranks()` after aggregation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PercentileBand {
    Top10,
    TopQuartile,
    Median,
    BottomQuartile,
}

impl PercentileBand {
    /// Derive the band from a 0–100 percentile rank.
    pub fn from_rank(rank: f64) -> Self {
        if rank >= 90.0      { Self::Top10 }
        else if rank >= 75.0 { Self::TopQuartile }
        else if rank >= 50.0 { Self::Median }
        else                 { Self::BottomQuartile }
    }
}

impl fmt::Display for PercentileBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Top10          => "top_10",
            Self::TopQuartile    => "top_quartile",
            Self::Median         => "median",
            Self::BottomQuartile => "bottom_quartile",
        })
    }
}

impl TryFrom<String> for PercentileBand {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "top_10"          => Ok(Self::Top10),
            "top_quartile"    => Ok(Self::TopQuartile),
            "median"          => Ok(Self::Median),
            "bottom_quartile" => Ok(Self::BottomQuartile),
            other => Err(format!("unknown PercentileBand: '{other}'")),
        }
    }
}

// ── Scale types ───────────────────────────────────────────────────────────────

/// How a dimension collects and aggregates contributor input.
///
/// Stored as VARCHAR in `atlas_scorecard_dimensions.scale_type`.
/// Used by `ScorecardService::recompute_aggregates` to branch aggregation logic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleType {
    /// Subjective 1–10 (or custom range). Aggregated as credibility-weighted mean.
    Rating,
    /// Real-world unit (Mbps, USD/mo, °C). Aggregated as mean of actuals.
    Absolute,
    /// Yes = 1.0 / No = 0.0. Aggregated as percent_true.
    Boolean,
    /// Pick exactly one option. Aggregated as vote count → rank.
    PollSingle,
    /// Pick one or many options. Aggregated as vote count per option.
    PollMulti,
}

impl fmt::Display for ScaleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rating     => "rating",
            Self::Absolute   => "absolute",
            Self::Boolean    => "boolean",
            Self::PollSingle => "poll_single",
            Self::PollMulti  => "poll_multi",
        })
    }
}

impl FromStr for ScaleType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rating"      => Ok(Self::Rating),
            "absolute"    => Ok(Self::Absolute),
            "boolean"     => Ok(Self::Boolean),
            "poll_single" => Ok(Self::PollSingle),
            "poll_multi"  => Ok(Self::PollMulti),
            other => Err(format!("unknown ScaleType: '{other}'")),
        }
    }
}

impl TryFrom<String> for ScaleType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ── Source types ──────────────────────────────────────────────────────────────

/// How a scorecard entry was produced.
///
/// Controls:
/// 1. Whether the entry is auto-verified or requires human confirmation.
/// 2. The expected shape of the `context` JSONB field (see `EntryContext`).
/// 3. The composite inclusion rule in `recompute_aggregates`.
///
/// Stored as VARCHAR in `atlas_scorecard_entries.source_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Public user rates an entity (city, restaurant, product).
    /// Context shape: `CommunityRatingContext`
    CommunityRating,
    /// Colleague rates colleague (Bridgewater-style peer review).
    /// Context shape: `PeerReviewContext`
    PeerReview,
    /// Subject rates themselves.
    SelfAssessment,
    /// Manager rates a direct report or reviews a recorded session.
    /// Context shape: `ManagerReviewContext`
    ManagerReview,
    /// Objective scored test result (CRT, skill assessment).
    /// Context shape: `TestResultContext`
    TestResult,
    /// Inferred from recorded platform behaviour (activity count, close rate).
    BehavioralSignal,
    /// External API feed (Speedtest, Numbeo, weather).
    /// Always inserted with `is_verified = true` — no human gate required.
    OfficialData,
    /// AI pre-fill inferred from a call recording or transcript.
    ///
    /// **ALWAYS** inserted with `is_verified = false`.
    /// Never counted in composite score until a human confirms via `verify_entry()`.
    /// Context shape: `TranscriptInferredContext`
    TranscriptInferred,
    /// Direct manual entry by a rater without a specific session type.
    Manual,
    /// Entry created during an inspection / audit session.
    Inspection,
}

impl SourceType {
    /// Returns true if this source type must never be auto-verified at insert time.
    ///
    /// Use this instead of `== "transcript_inferred"` string comparisons.
    pub fn requires_human_verification(&self) -> bool {
        matches!(self, Self::TranscriptInferred)
    }

    /// Returns true if this source type is pre-verified by definition (no human gate).
    pub fn is_auto_verified(&self) -> bool {
        matches!(self, Self::OfficialData)
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CommunityRating    => "community_rating",
            Self::PeerReview         => "peer_review",
            Self::SelfAssessment     => "self_assessment",
            Self::ManagerReview      => "manager_review",
            Self::TestResult         => "test_result",
            Self::BehavioralSignal   => "behavioral_signal",
            Self::OfficialData       => "official_data",
            Self::TranscriptInferred => "transcript_inferred",
            Self::Manual             => "manual",
            Self::Inspection         => "inspection",
        })
    }
}

impl TryFrom<String> for SourceType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "community_rating"    => Ok(Self::CommunityRating),
            "peer_review"         => Ok(Self::PeerReview),
            "self_assessment"     => Ok(Self::SelfAssessment),
            "manager_review"      => Ok(Self::ManagerReview),
            "test_result"         => Ok(Self::TestResult),
            "behavioral_signal"   => Ok(Self::BehavioralSignal),
            "official_data"       => Ok(Self::OfficialData),
            "transcript_inferred" => Ok(Self::TranscriptInferred),
            "manual"              => Ok(Self::Manual),
            "inspection"          => Ok(Self::Inspection),
            other => Err(format!("unknown SourceType: '{other}'")),
        }
    }
}

// ── Session types ─────────────────────────────────────────────────────────────

/// The type of discrete occurrence that opened a rating session.
///
/// Stored as VARCHAR in `atlas_rating_sessions.session_type`.
/// Groups into two contexts: consumer (G-27 original) and CRM/sales (G27SC extension).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    // ── Consumer contexts ─────────────────────────────────────────────────────
    /// Contractor / service job completion.
    Job,
    /// Hotel / property stay.
    Stay,
    /// City or venue visit.
    Visit,
    /// Single event staff shift.
    EventShift,
    /// Product purchase.
    Purchase,
    /// Airline flight segment.
    Flight,
    // ── CRM / Sales contexts ──────────────────────────────────────────────────
    /// General meeting (in-person or virtual).
    Meeting,
    /// First discovery / qualification call.
    DiscoveryCall,
    /// Product demo session.
    Demo,
    /// Deal health pipeline review.
    PipelineReview,
    /// Ad-hoc sales or support call.
    Call,
    /// Email thread review.
    EmailThread,
    /// Async review of a call recording or transcript.
    TranscriptReview,
    /// Monthly performance / account review.
    MonthlyReview,
    /// Quarterly business review.
    QuarterlyReview,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Job              => "job",
            Self::Stay             => "stay",
            Self::Visit            => "visit",
            Self::EventShift       => "event_shift",
            Self::Purchase         => "purchase",
            Self::Flight           => "flight",
            Self::Meeting          => "meeting",
            Self::DiscoveryCall    => "discovery_call",
            Self::Demo             => "demo",
            Self::PipelineReview   => "pipeline_review",
            Self::Call             => "call",
            Self::EmailThread      => "email_thread",
            Self::TranscriptReview => "transcript_review",
            Self::MonthlyReview    => "monthly_review",
            Self::QuarterlyReview  => "quarterly_review",
        })
    }
}

impl TryFrom<String> for SessionType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "job"               => Ok(Self::Job),
            "stay"              => Ok(Self::Stay),
            "visit"             => Ok(Self::Visit),
            "event_shift"       => Ok(Self::EventShift),
            "purchase"          => Ok(Self::Purchase),
            "flight"            => Ok(Self::Flight),
            "meeting"           => Ok(Self::Meeting),
            "discovery_call"    => Ok(Self::DiscoveryCall),
            "demo"              => Ok(Self::Demo),
            "pipeline_review"   => Ok(Self::PipelineReview),
            "call"              => Ok(Self::Call),
            "email_thread"      => Ok(Self::EmailThread),
            "transcript_review" => Ok(Self::TranscriptReview),
            "monthly_review"    => Ok(Self::MonthlyReview),
            "quarterly_review"  => Ok(Self::QuarterlyReview),
            other => Err(format!("unknown SessionType: '{other}'")),
        }
    }
}

// ── Confidence levels ─────────────────────────────────────────────────────────

/// Statistical confidence tier for a scorecard's composite score.
///
/// Derives `PartialOrd` + `Ord` so confidence levels can be compared directly:
/// `ConfidenceLevel::Medium > ConfidenceLevel::Low` is valid and correct.
///
/// Stored as VARCHAR in `atlas_scorecards.confidence_level`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Insufficient, // < 5 verified entries
    Low,          // 5–9
    Medium,       // 10–49
    High,         // 50–199
    VeryHigh,     // 200+
}

impl ConfidenceLevel {
    /// Compute the confidence level from the total number of verified entries.
    ///
    /// This replaces the string-returning `compute_confidence_level(n: i32)` function
    /// in `scorecard_service.rs`. Call `.to_string()` at the DB write boundary.
    pub fn from_entry_count(n: i32) -> Self {
        match n {
            i32::MIN..=4 => Self::Insufficient,
            5..=9        => Self::Low,
            10..=49      => Self::Medium,
            50..=199     => Self::High,
            _            => Self::VeryHigh,
        }
    }
}

impl fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Insufficient => "insufficient",
            Self::Low          => "low",
            Self::Medium       => "medium",
            Self::High         => "high",
            Self::VeryHigh     => "very_high",
        })
    }
}

impl TryFrom<String> for ConfidenceLevel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "insufficient" => Ok(Self::Insufficient),
            "low"          => Ok(Self::Low),
            "medium"       => Ok(Self::Medium),
            "high"         => Ok(Self::High),
            "very_high"    => Ok(Self::VeryHigh),
            other => Err(format!("unknown ConfidenceLevel: '{other}'")),
        }
    }
}

// ── Display Rule enums ────────────────────────────────────────────────────────

/// What kind of condition fires a Display Rule.
///
/// Stored as VARCHAR in `atlas_scorecard_display_rules.trigger_category`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerCategory {
    /// A field on the subject entity has a specific value.
    /// e.g. `Opportunity.stage == "Negotiation"`
    RecordState,
    /// An event is within N days of now (upcoming) or N days past (overdue).
    /// e.g. `close_date` is within 7 days.
    TimeProximity,
    /// A matching `atlas_activity` was just logged for this entity.
    /// e.g. `activity_type IN ['call', 'demo']`
    ActivityTrigger,
    /// A specific scorecard dimension's aggregate is above or below a score threshold.
    /// e.g. `champion_strength < 5.0`
    ScoreGap,
}

impl fmt::Display for TriggerCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RecordState     => "record_state",
            Self::TimeProximity   => "time_proximity",
            Self::ActivityTrigger => "activity_trigger",
            Self::ScoreGap        => "score_gap",
        })
    }
}

impl TryFrom<String> for TriggerCategory {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "record_state"      => Ok(Self::RecordState),
            "time_proximity"    => Ok(Self::TimeProximity),
            "activity_trigger"  => Ok(Self::ActivityTrigger),
            "score_gap"         => Ok(Self::ScoreGap),
            other => Err(format!("unknown TriggerCategory: '{other}'")),
        }
    }
}

/// Comparison operator for a Display Rule condition.
///
/// Stored as VARCHAR in `atlas_scorecard_display_rules.operator`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleOperator {
    Equals,
    NotEquals,
    /// Field value is in a set. Compared against `value_list`.
    In,
    /// Field value is not in a set. Compared against `value_list`.
    NotIn,
    /// Date field is within N days of now. N stored in `value`.
    WithinDays,
    /// Date field is N days past now (overdue). N stored in `value`.
    OverdueDays,
    /// Dimension aggregate score is below threshold. Threshold in `value`.
    DimensionScoreBelow,
    /// Dimension aggregate score is above threshold. Threshold in `value`.
    DimensionScoreAbove,
    /// `atlas_activity.activity_category` matches one of the values in `value_list`.
    ActivityTypeIs,
    /// The targeted dimension has no verified entries yet for this scorecard.
    DimensionUnrated,
}

impl fmt::Display for RuleOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Equals              => "equals",
            Self::NotEquals           => "not_equals",
            Self::In                  => "in",
            Self::NotIn               => "not_in",
            Self::WithinDays          => "within_days",
            Self::OverdueDays         => "overdue_days",
            Self::DimensionScoreBelow => "dimension_score_below",
            Self::DimensionScoreAbove => "dimension_score_above",
            Self::ActivityTypeIs      => "activity_type_is",
            Self::DimensionUnrated    => "dimension_unrated",
        })
    }
}

impl TryFrom<String> for RuleOperator {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "equals"                => Ok(Self::Equals),
            "not_equals"            => Ok(Self::NotEquals),
            "in"                    => Ok(Self::In),
            "not_in"                => Ok(Self::NotIn),
            "within_days"           => Ok(Self::WithinDays),
            "overdue_days"          => Ok(Self::OverdueDays),
            "dimension_score_below" => Ok(Self::DimensionScoreBelow),
            "dimension_score_above" => Ok(Self::DimensionScoreAbove),
            "activity_type_is"      => Ok(Self::ActivityTypeIs),
            "dimension_unrated"     => Ok(Self::DimensionUnrated),
            other => Err(format!("unknown RuleOperator: '{other}'")),
        }
    }
}

/// What happens to a dimension when its Display Rule condition fires.
///
/// Stored as VARCHAR in `atlas_scorecard_display_rules.action`.
///
/// Conflict resolution precedence (highest to lowest):
/// `Require` > `Hide` > `Show` > `SurfaceAsNudge` = `ShowInPrepMode` = `ShowAlertBanner`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    /// Make the dimension visible (overrides default hidden state).
    Show,
    /// Suppress the dimension from the session form.
    Hide,
    /// Mark the dimension as required — contributor cannot skip it.
    /// Implies `Show`: a required dimension is always visible.
    Require,
    /// Surface as a compact post-activity nudge prompt.
    SurfaceAsNudge,
    /// Show only in pre-activity preparation mode.
    ShowInPrepMode,
    /// Show a banner alert above the scorecard widget.
    ShowAlertBanner,
}

impl RuleAction {
    /// Returns true if `self` takes precedence over `other` in a conflict.
    ///
    /// Used by the Display Rule evaluation engine to resolve conflicts when
    /// multiple rules target the same dimension.
    pub fn overrides(&self, other: &RuleAction) -> bool {
        matches!(
            (self, other),
            // Require beats everything
            (Self::Require, _) |
            // Hide beats Show (safety default)
            (Self::Hide, Self::Show) |
            (Self::Hide, Self::SurfaceAsNudge) |
            (Self::Hide, Self::ShowInPrepMode)
        )
    }
}

impl fmt::Display for RuleAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Show            => "show",
            Self::Hide            => "hide",
            Self::Require         => "require",
            Self::SurfaceAsNudge  => "surface_as_nudge",
            Self::ShowInPrepMode  => "show_in_prep_mode",
            Self::ShowAlertBanner => "show_alert_banner",
        })
    }
}

impl TryFrom<String> for RuleAction {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "show"              => Ok(Self::Show),
            "hide"              => Ok(Self::Hide),
            "require"           => Ok(Self::Require),
            "surface_as_nudge"  => Ok(Self::SurfaceAsNudge),
            "show_in_prep_mode" => Ok(Self::ShowInPrepMode),
            "show_alert_banner" => Ok(Self::ShowAlertBanner),
            other => Err(format!("unknown RuleAction: '{other}'")),
        }
    }
}

/// Which rendering context must be active for a Display Rule to apply.
///
/// Stored as VARCHAR in `atlas_scorecard_display_rules.mode_scope`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeScope {
    /// Rule is evaluated on every render of the scorecard widget.
    Always,
    /// Rule only applies after a qualifying `atlas_activity` has just been logged.
    PostActivity,
    /// Rule only applies when a scheduled `atlas_activity` is upcoming.
    PreActivity,
    /// Rule only applies when the score gap condition on the trigger dimension is active.
    OnScoreGap,
}

impl fmt::Display for ModeScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Always       => "always",
            Self::PostActivity => "post_activity",
            Self::PreActivity  => "pre_activity",
            Self::OnScoreGap   => "on_score_gap",
        })
    }
}

impl TryFrom<String> for ModeScope {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "always"        => Ok(Self::Always),
            "post_activity" => Ok(Self::PostActivity),
            "pre_activity"  => Ok(Self::PreActivity),
            "on_score_gap"  => Ok(Self::OnScoreGap),
            other => Err(format!("unknown ModeScope: '{other}'")),
        }
    }
}

// ── Typed JSONB structs ───────────────────────────────────────────────────────
//
// These replace `serde_json::Value` for all G-27 JSONB columns in service code.
//
// Contract:
//   Entity layer:  retains `serde_json::Value` at the SeaORM DB boundary.
//   Service layer: calls `serde_json::from_value::<T>(raw)?` after reading,
//                  calls `serde_json::to_value(&typed)?` before writing.
//   JSON output:   fully driven by serde — no manual key construction.

/// A single benchmark tier entry stored in the `benchmark_tiers` JSONB array
/// on `atlas_scorecard_dimensions`.
///
/// Which fields are populated depends on the dimension's `is_inverted` and `scale_type`:
/// - Normal rating/absolute dims: use `min_score`
/// - Inverted dims:               use `max_score`
/// - Boolean dims:                use `min_pct`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTier {
    /// Display label: "Outstanding", "Needs Work", "Most say clean"
    pub label: String,
    /// CSS hex color: "#00cc44"
    pub color: String,
    /// Normal rating/absolute: score must be >= this value to match this tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f64>,
    /// Inverted dimension: score must be <= this value to match this tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_score: Option<f64>,
    /// Boolean dimension: percent_true must be >= this value to match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_pct: Option<f64>,
    /// Optional prefix for the display value: "Stay inside: 180 μg/m³"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// If true, the actual numeric value is appended to the label in display.
    #[serde(default, skip_serializing_if = "is_false")]
    pub show_value: bool,
}

fn is_false(b: &bool) -> bool { !b }

/// The full `benchmark_tiers` JSONB array — typed, never a raw `Value`.
pub type BenchmarkTiers = Vec<BenchmarkTier>;

/// Typed context for `transcript_inferred` entries.
/// Stored in `atlas_scorecard_entries.context` JSONB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptInferredContext {
    /// Model confidence score 0.0–1.0.
    pub confidence: f64,
    /// Direct quote from the transcript that supports this inferred score.
    pub evidence_quote: String,
    /// Model version string for auditability and reproducibility.
    pub model_version: String,
}

/// Typed context for `community_rating` entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRatingContext {
    /// ISO month: "2024-03"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visit_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_days: Option<f64>,
    /// "work" | "leisure" | "transit" | "relocation"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

/// Typed context for `peer_review` entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReviewContext {
    /// "peer" | "manager" | "skip_level" | "direct_report"
    pub relationship: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worked_together_months: Option<f64>,
}

/// Typed context for `manager_review` entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerReviewContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_recording_url: Option<String>,
    /// ISO date string: "2024-06-01"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_duration_seconds: Option<i32>,
}

/// Typed context for `test_result` entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultContext {
    pub test_name: String,
    /// ISO date string: "2024-01-15"
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub administered_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passing_score: Option<f64>,
}

/// Union of all typed entry contexts.
///
/// The service selects the right variant based on `SourceType` — no runtime string
/// matching required. Use `serde_json::from_value::<EntryContext>(raw)?` to
/// deserialize from the `context` JSONB column.
///
/// `#[serde(untagged)]` means JSON serialization does not add a type discriminator key —
/// the shape of the fields determines which variant matches on deserialization.
/// This preserves backward compatibility with existing JSONB data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntryContext {
    TranscriptInferred(TranscriptInferredContext),
    PeerReview(PeerReviewContext),
    ManagerReview(ManagerReviewContext),
    TestResult(TestResultContext),
    CommunityRating(CommunityRatingContext),
    /// Catch-all for `BehavioralSignal`, `OfficialData`, and future source types
    /// that do not yet have a typed struct. Preserved as-is.
    Generic(serde_json::Value),
}

// ── Template display configuration ───────────────────────────────────────────
//
// `ScorecardTemplateDisplayConfig` is the authoritative Rust type for the
// `display_config` JSONB column on `atlas_scorecard_templates`.
//
// Contract (same as all other typed JSONB types in this module):
//   Entity layer:  `pub display_config: Option<serde_json::Value>`
//   Service layer: `ScorecardTemplateDisplayConfig::from_json(model.display_config.as_ref())?`
//                  `.to_json()?` before writing back to the entity.
//   Serde default: all bool fields default to `false`; all Option fields default to
//                  `None`. Rows where `display_config` IS NULL deserialize to the
//                  all-off default — no migration default value required.
//
// Adding a new surface:
//   1. Add a new field here with `#[serde(default)]` (implied by struct-level attr).
//   2. Read it in the relevant service check.
//   3. Add a toggle in the Configurator "Display Rules" tab.
//   No schema migration needed — JSONB is schema-flexible at the column level.

/// Per-surface display control for a scorecard template.
///
/// Stored as JSONB in `atlas_scorecard_templates.display_config`.
/// The `<Configurator>` template editor exposes this struct as a "Display Rules" tab —
/// a toggle grid where the landlord/admin enables surfaces without code changes.
/// All fields default to `false` / `None` — explicit opt-in required.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScorecardTemplateDisplayConfig {
    // ── Landlord dashboard surfaces ───────────────────────────────────────────

    /// Show composite score badge in the PortfolioTable row for this template's entity.
    /// Surface: Landlord God View `/dashboard` → PortfolioTable G27ScoreBadge column.
    pub show_on_portfolio_table: bool,

    /// Render this template's entities in the G-27 anomaly panel widget.
    /// Surface: Landlord God View `/dashboard` → G27AnomalyPanel sidebar.
    pub show_on_anomaly_panel: bool,

    /// Include entities rated by this template in the vendor/entity leaderboard widget.
    /// Surface: Landlord God View `/dashboard` → VendorLeaderboard sidebar.
    pub show_on_leaderboard: bool,

    /// Show the score badge inline on the maintenance dispatch queue vendor column.
    /// Surface: `/portfolio/maintenance` → assigned-vendor score badge.
    pub show_on_maintenance_queue: bool,

    /// Show the ScorecardDisplay drawer in the property detail unit tab.
    /// Surface: `/portfolio/properties/{id}` → Units tab score column.
    pub show_on_property_detail: bool,

    // ── Wholesaling surfaces ──────────────────────────────────────────────────

    /// Show the ScorecardDisplay inline on the lead detail page.
    /// Surface: `/wholesaling/leads/{id}` → Lead Quality Assessment inline widget.
    pub show_on_lead_card: bool,

    // ── Public / tenant-facing surfaces ──────────────────────────────────────

    /// Show the composite score badge on the public PropertyForge listing card.
    /// Surface: `/listings` and `/listings/{property_id}`.
    /// Requires explicit landlord opt-in — scores are NOT shown publicly by default.
    pub show_on_public_listing: bool,

    /// Allow the rated subject (e.g. tenant) to see their own score in their portal.
    /// Surface: `/tenant/dashboard` — optional "Your rental quality score" card.
    pub tenant_visible: bool,

    // ── Nudge / activity-trigger surfaces ────────────────────────────────────

    /// Fire a `<ScoreNudge>` WebSocket push when an `atlas_case` of type `maintenance`
    /// transitions to `status = closed`.
    /// Surface: any authenticated landlord page → toast overlay.
    /// Typically enabled on the Contractor Performance template.
    pub nudge_on_maintenance_case_close: bool,

    /// Fire a `<ScoreNudge>` WebSocket push when an STR reservation reaches `checked_out`.
    /// Surface: any authenticated page → toast overlay.
    /// Typically enabled on the STR Property Assessment template.
    pub nudge_on_str_checkout: bool,

    // ── Display gate ──────────────────────────────────────────────────────────

    /// Minimum number of verified entries before any display surface renders the score.
    /// `None` = fall through to `atlas_scorecard_templates.min_entries_to_publish`.
    ///
    /// Useful when a landlord wants a stricter public display gate (e.g. require 10
    /// public entries before showing the score on `/listings`, even if
    /// `min_entries_to_publish = 3` for internal dashboard use).
    pub min_entries_before_display: Option<i32>,

    /// When `true`, the `<ScorecardDisplay>` widget renders collapsed and expands on click.
    /// When `false` (default), it renders expanded inline.
    pub collapsed_by_default: bool,
}

impl ScorecardTemplateDisplayConfig {
    /// Deserialize from the raw JSONB value stored in the entity layer.
    ///
    /// Returns the all-false/all-None default if `raw` is `None`
    /// (template row predates this field — backward compatible).
    pub fn from_json(raw: Option<&serde_json::Value>) -> Result<Self, serde_json::Error> {
        match raw {
            Some(v) => serde_json::from_value(v.clone()),
            None    => Ok(Self::default()),
        }
    }

    /// Serialize to a `serde_json::Value` for writing back to the entity.
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Returns `true` if at least one display surface is enabled.
    ///
    /// Used by the Configurator "Display Rules" tab to render the active indicator dot.
    pub fn any_surface_enabled(&self) -> bool {
        self.show_on_portfolio_table
            || self.show_on_anomaly_panel
            || self.show_on_leaderboard
            || self.show_on_maintenance_queue
            || self.show_on_property_detail
            || self.show_on_lead_card
            || self.show_on_public_listing
            || self.tenant_visible
            || self.nudge_on_maintenance_case_close
            || self.nudge_on_str_checkout
    }
}

// ── Scorecard entity type ─────────────────────────────────────────────────────

/// Discriminator for the type of entity a scorecard template targets or a
/// scorecard record is attached to.
///
/// Used in:
///  - `atlas_scorecard_templates.entity_type` (what kinds of entities use this template)
///  - `atlas_scorecards.subject_entity_type` (the specific entity this scorecard is for)
///
/// Both columns store the same domain of values — this enum is shared.
///
/// Stored as VARCHAR in both columns. Services call `TryFrom<String>` after
/// reading; `Display::fmt` before writing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScorecardEntityType {
    // ── Geographic / place ────────────────────────────────────────────────────
    City,
    Property,
    Hotel,
    // ── People / professionals ────────────────────────────────────────────────
    Person,
    Agent,
    /// Service provider (contractor, vendor, etc.)
    Contractor,
    // ── Products / experiences ────────────────────────────────────────────────
    Restaurant,
    Product,
    Event,
    // ── Transport ────────────────────────────────────────────────────────────
    Airline,
    Carrier,
    // ── Platform generic entities ─────────────────────────────────────────────
    AtlasLead,
    AtlasOpportunity,
    AtlasAccount,
    AtlasAsset,
    AtlasCatalogEntry,
    AtlasServiceProvider,
    AtlasContact,
    AtlasPortfolio,
    /// Legacy — `listing` table (pre-G-10 `atlas_assets` promotion).
    Listing,
    /// Legacy — `profile` table.
    Profile,
}

impl fmt::Display for ScorecardEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::City                 => "city",
            Self::Property             => "property",
            Self::Hotel                => "hotel",
            Self::Person               => "person",
            Self::Agent                => "agent",
            Self::Contractor           => "contractor",
            Self::Restaurant           => "restaurant",
            Self::Product              => "product",
            Self::Event                => "event",
            Self::Airline              => "airline",
            Self::Carrier              => "carrier",
            Self::AtlasLead            => "atlas_lead",
            Self::AtlasOpportunity     => "atlas_opportunity",
            Self::AtlasAccount         => "atlas_account",
            Self::AtlasAsset           => "atlas_asset",
            Self::AtlasCatalogEntry    => "atlas_catalog_entry",
            Self::AtlasServiceProvider => "atlas_service_provider",
            Self::AtlasContact         => "atlas_contact",
            Self::AtlasPortfolio       => "atlas_portfolio",
            Self::Listing              => "listing",
            Self::Profile              => "profile",
        })
    }
}

impl TryFrom<String> for ScorecardEntityType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "city"                  => Ok(Self::City),
            "property"              => Ok(Self::Property),
            "hotel"                 => Ok(Self::Hotel),
            "person"                => Ok(Self::Person),
            "agent"                 => Ok(Self::Agent),
            "contractor"            => Ok(Self::Contractor),
            "restaurant"            => Ok(Self::Restaurant),
            "product"               => Ok(Self::Product),
            "event"                 => Ok(Self::Event),
            "airline"               => Ok(Self::Airline),
            "carrier"               => Ok(Self::Carrier),
            "atlas_lead"            => Ok(Self::AtlasLead),
            "atlas_opportunity"     => Ok(Self::AtlasOpportunity),
            "atlas_account"         => Ok(Self::AtlasAccount),
            "atlas_asset"           => Ok(Self::AtlasAsset),
            "atlas_catalog_entry"   => Ok(Self::AtlasCatalogEntry),
            "atlas_service_provider" => Ok(Self::AtlasServiceProvider),
            "atlas_contact"         => Ok(Self::AtlasContact),
            "atlas_portfolio"       => Ok(Self::AtlasPortfolio),
            "listing"               => Ok(Self::Listing),
            "profile"               => Ok(Self::Profile),
            other                   => Err(format!("unknown ScorecardEntityType: '{other}'")),
        }
    }
}

impl TryFrom<&str> for ScorecardEntityType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}
