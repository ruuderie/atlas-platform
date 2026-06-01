//! G-27 Scorecard — Shared data models for all scorecard UI components.
//!
//! These types mirror the backend entity shapes for form state management.
//! They are used by:
//!   - `Configurator` — template/dimension admin builder
//!   - `ScorecardWidget` — session form rendered on entity records
//!   - `NudgePrompt` — post-activity floating prompt
//!   - `PrepModePanel` — pre-activity dimension guidance panel
//!   - `RuleBuilder` — display rules configuration section

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── Template ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TemplateForm {
    pub id: Option<Uuid>,
    pub name: String,
    pub entity_type: String,
    pub description: String,
    pub scoring_method: String,
    pub default_scale_min: f64,
    pub default_scale_max: f64,
    pub min_entries_to_publish: i32,
    pub is_published: bool,
}

impl Default for TemplateForm {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            entity_type: "atlas_lead".to_string(),
            description: String::new(),
            scoring_method: "weighted_mean".to_string(),
            default_scale_min: 1.0,
            default_scale_max: 10.0,
            min_entries_to_publish: 3,
            is_published: false,
        }
    }
}

// ── Dimension ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DimensionForm {
    /// Client-side stable key for list keying (not persisted until save).
    pub local_id: usize,
    pub id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub category: String,
    pub weight: f64,
    /// 'rating' | 'absolute' | 'boolean' | 'poll_single' | 'poll_multi'
    pub scale_type: String,
    pub scale_min: f64,
    pub scale_max: f64,
    pub unit_label: String,
    /// When true: lower score = better (e.g. timeline_slippage, competition_risk).
    /// Mirrors `atlas_scorecard_dimensions.is_inverted`.
    pub is_inverted: bool,
    pub is_community_ratable: bool,
    pub is_active: bool,
    pub sort_order: i32,
    pub options: Vec<OptionForm>,
    // ── Combinator criteria ───────────────────────────────────────────────────
    pub ideal_score: Option<f64>,
    pub range_min: Option<f64>,
    pub range_max: Option<f64>,
    pub search_weight: Option<f64>,
}

impl DimensionForm {
    pub fn new(local_id: usize, sort_order: i32) -> Self {
        Self {
            local_id,
            id: None,
            name: String::new(),
            slug: String::new(),
            description: String::new(),
            category: String::new(),
            weight: 1.0,
            scale_type: "rating".to_string(),
            scale_min: 1.0,
            scale_max: 10.0,
            unit_label: String::new(),
            is_inverted: false,
            is_community_ratable: true,
            is_active: true,
            sort_order,
            options: Vec::new(),
            ideal_score: None,
            range_min: None,
            range_max: None,
            search_weight: None,
        }
    }

    /// True if this dimension's scale uses options (poll types).
    pub fn needs_options(&self) -> bool {
        self.scale_type == "poll_single" || self.scale_type == "poll_multi"
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptionForm {
    pub local_id: usize,
    pub id: Option<Uuid>,
    pub label: String,
    pub value_key: String,
    pub description: String,
    pub sort_order: i32,
    pub is_write_in: bool,
}

impl OptionForm {
    pub fn new(local_id: usize, sort_order: i32) -> Self {
        Self {
            local_id,
            id: None,
            label: String::new(),
            value_key: String::new(),
            description: String::new(),
            sort_order,
            is_write_in: false,
        }
    }
}

// ── Display Rules ─────────────────────────────────────────────────────────────

/// A single display rule in the Rule Builder UI.
///
/// Maps 1:1 to `atlas_scorecard_display_rules` on the backend.
/// All discriminator fields match the Rust enum string representations
/// defined in `crate::types::scorecard`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DisplayRuleForm {
    pub local_id: usize,
    pub id: Option<Uuid>,

    /// None = category-level rule.
    pub dimension_id: Option<Uuid>,
    /// Human-readable name of the targeted dimension (for display only, not persisted).
    pub dimension_name: String,

    /// Category-level target (used when dimension_id is None).
    pub category_target: String,

    // ── Trigger axis ─────────────────────────────────────────────────────────
    /// 'record_state' | 'time_proximity' | 'activity_trigger' | 'score_gap'
    pub trigger_category: String,
    /// Field path on the subject entity. e.g. 'stage', 'close_date'
    pub field_reference: String,
    /// 'equals' | 'not_equals' | 'in' | 'not_in' | 'within_days' | 'overdue_days'
    /// | 'dimension_score_below' | 'dimension_score_above'
    /// | 'activity_type_is' | 'dimension_unrated'
    pub operator: String,
    /// Scalar comparison value.
    pub value: String,
    /// List values for 'in', 'not_in', 'activity_type_is'.
    /// Stored as comma-separated in the UI; serialized as JSON array on save.
    pub value_list_raw: String,

    // ── Action axis ───────────────────────────────────────────────────────────
    /// 'show' | 'hide' | 'require' | 'surface_as_nudge' | 'show_in_prep_mode' | 'show_alert_banner'
    pub action: String,
    pub alert_message: String,

    // ── Scope ─────────────────────────────────────────────────────────────────
    /// 'always' | 'post_activity' | 'pre_activity' | 'on_score_gap'
    pub mode_scope: String,
    pub priority: i32,
    pub is_active: bool,
    pub description: String,
}

impl DisplayRuleForm {
    pub fn new(local_id: usize, priority: i32) -> Self {
        Self {
            local_id,
            id: None,
            dimension_id: None,
            dimension_name: String::new(),
            category_target: String::new(),
            trigger_category: "record_state".to_string(),
            field_reference: String::new(),
            operator: "equals".to_string(),
            value: String::new(),
            value_list_raw: String::new(),
            action: "show".to_string(),
            alert_message: String::new(),
            mode_scope: "always".to_string(),
            priority,
            is_active: true,
            description: String::new(),
        }
    }

    /// Parse `value_list_raw` (comma-separated) into a Vec<String>.
    pub fn value_list(&self) -> Vec<String> {
        self.value_list_raw
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Human-readable summary of the rule for the rules list.
    pub fn summary(&self) -> String {
        let target = if !self.dimension_name.is_empty() {
            self.dimension_name.clone()
        } else if !self.category_target.is_empty() {
            format!("[{}]", self.category_target)
        } else {
            "?".to_string()
        };
        format!("{}: {} → {}", target, self.trigger_label(), self.action_label())
    }

    pub fn trigger_label(&self) -> &'static str {
        match self.trigger_category.as_str() {
            "record_state"      => "Record State",
            "time_proximity"    => "Time Proximity",
            "activity_trigger"  => "Activity Logged",
            "score_gap"         => "Score Gap",
            _                   => "—",
        }
    }

    pub fn action_label(&self) -> &'static str {
        match self.action.as_str() {
            "show"              => "Show",
            "hide"              => "Hide",
            "require"           => "Require",
            "surface_as_nudge"  => "Surface as Nudge",
            "show_in_prep_mode" => "Prep Mode Only",
            "show_alert_banner" => "Show Alert Banner",
            _                   => "—",
        }
    }

    /// True if the action surfaces this dimension in the compact nudge widget.
    pub fn is_nudge_action(&self) -> bool {
        matches!(
            self.action.as_str(),
            "surface_as_nudge" | "require"
        )
    }
}

// ── Session / Widget models ───────────────────────────────────────────────────

/// A dimension surfaced in the session form or nudge widget.
/// Used by `ScorecardWidget` and `NudgePrompt` to render inputs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionDimension {
    pub dimension_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub scale_type: String,
    pub scale_min: f64,
    pub scale_max: f64,
    pub unit_label: Option<String>,
    pub is_inverted: bool,
    pub is_required: bool,
    /// Rendering mode: 'normal' | 'nudge' | 'prep'
    pub render_mode: String,
    /// Current score draft (before submission).
    pub draft_score: Option<f64>,
    /// Pre-filled score from AI transcript inference (is_verified=false).
    pub inferred_score: Option<f64>,
    /// AI confidence for inferred_score (0.0–1.0).
    pub inferred_confidence: Option<f64>,
    pub draft_option_id: Option<Uuid>,
}

impl SessionDimension {
    /// True if this dimension has an AI-inferred score awaiting human confirmation.
    pub fn has_pending_inference(&self) -> bool {
        self.inferred_score.is_some()
    }
}
