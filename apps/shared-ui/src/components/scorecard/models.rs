//! G-27 Scorecard — Shared data models for all scorecard UI components.
//!
//! These types mirror the backend entity shapes for form state management.
//! They are used by:
//!   - `Configurator` — template/dimension admin builder
//!   - `ScorecardWidget` — session form rendered on entity records
//!   - `NudgePrompt` — post-activity floating prompt
//!   - `PrepModePanel` — pre-activity dimension guidance panel
//!   - `RuleBuilder` — display rules configuration section

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ScaleType {
    Rating,
    Absolute,
    Boolean,
    PollSingle,
    PollMulti,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TriggerCategory {
    RecordState,
    TimeProximity,
    ActivityTrigger,
    ScoreGap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RuleOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    WithinDays,
    OverdueDays,
    DimensionScoreBelow,
    DimensionScoreAbove,
    ActivityTypeIs,
    DimensionUnrated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RuleAction {
    Show,
    Hide,
    Require,
    SurfaceAsNudge,
    ShowInPrepMode,
    ShowAlertBanner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ModeScope {
    Always,
    PostActivity,
    PreActivity,
    OnScoreGap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RenderMode {
    Normal,
    Nudge,
    Prep,
    Alert,
}

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
    /// `platform` | `tenant` — see docs/contracts/g27_scorecard_platform.md
    pub template_scope: String,
    pub cold_start_strategy: String,
    pub cold_start_saturation_threshold: i32,
    pub calibration_minimum_entries: i32,
    pub default_bayesian_prior_weight: Option<f64>,
    pub display_config: DisplayConfigForm,
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
            template_scope: "tenant".to_string(),
            cold_start_strategy: "suppress".to_string(),
            cold_start_saturation_threshold: 50,
            calibration_minimum_entries: 100,
            default_bayesian_prior_weight: None,
            display_config: DisplayConfigForm::default(),
        }
    }
}

/// Frontend mirror of `ScorecardTemplateDisplayConfig` (backend types/scorecard.rs).
/// Field-for-field with the JSONB `display_config` column.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfigForm {
    pub show_on_portfolio_table: bool,
    pub show_on_anomaly_panel: bool,
    pub show_on_leaderboard: bool,
    pub show_on_maintenance_queue: bool,
    pub show_on_property_detail: bool,
    pub show_on_lead_card: bool,
    pub show_on_public_listing: bool,
    pub tenant_visible: bool,
    pub nudge_on_maintenance_case_close: bool,
    pub nudge_on_str_checkout: bool,
    pub min_entries_before_display: Option<i32>,
    pub collapsed_by_default: bool,
}

impl DisplayConfigForm {
    pub fn any_surface_enabled(&self) -> bool {
        self.show_on_portfolio_table
            || self.show_on_anomaly_panel
            || self.show_on_leaderboard
            || self.show_on_maintenance_queue
            || self.show_on_property_detail
            || self.show_on_lead_card
            || self.show_on_public_listing
            || self.tenant_visible
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
    pub scale_type: ScaleType,
    pub scale_min: f64,
    pub scale_max: f64,
    pub unit_label: String,
    /// When true: lower score = better (e.g. timeline_slippage, competition_risk).
    /// Mirrors `atlas_scorecard_dimensions.is_inverted`.
    pub is_inverted: bool,
    pub is_community_ratable: bool,
    pub is_active: bool,
    pub sort_order: i32,
    /// Landlord/app-added dim — excluded from cross-tenant benchmark pool.
    pub is_tenant_extension: bool,
    pub min_entries_to_show: i32,
    pub bayesian_prior_weight: Option<f64>,
    pub global_reference_value: Option<f64>,
    pub global_reference_label: String,
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
            scale_type: ScaleType::Rating,
            scale_min: 1.0,
            scale_max: 10.0,
            unit_label: String::new(),
            is_inverted: false,
            is_community_ratable: true,
            is_active: true,
            sort_order,
            is_tenant_extension: false,
            min_entries_to_show: 1,
            bayesian_prior_weight: None,
            global_reference_value: None,
            global_reference_label: String::new(),
            options: Vec::new(),
            ideal_score: None,
            range_min: None,
            range_max: None,
            search_weight: None,
        }
    }

    /// True if this dimension's scale uses options (poll types).
    pub fn needs_options(&self) -> bool {
        self.scale_type == ScaleType::PollSingle || self.scale_type == ScaleType::PollMulti
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
    pub trigger_category: TriggerCategory,
    /// Field path on the subject entity. e.g. 'stage', 'close_date'
    pub field_reference: String,
    pub operator: RuleOperator,
    /// Scalar comparison value.
    pub value: String,
    /// List values for 'in', 'not_in', 'activity_type_is'.
    /// Stored as comma-separated in the UI; serialized as JSON array on save.
    pub value_list_raw: String,

    // ── Action axis ───────────────────────────────────────────────────────────
    pub action: RuleAction,
    pub alert_message: String,

    // ── Scope ─────────────────────────────────────────────────────────────────
    pub mode_scope: ModeScope,
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
            trigger_category: TriggerCategory::RecordState,
            field_reference: String::new(),
            operator: RuleOperator::Equals,
            value: String::new(),
            value_list_raw: String::new(),
            action: RuleAction::Show,
            alert_message: String::new(),
            mode_scope: ModeScope::Always,
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
        match self.trigger_category {
            TriggerCategory::RecordState     => "Record State",
            TriggerCategory::TimeProximity   => "Time Proximity",
            TriggerCategory::ActivityTrigger => "Activity Logged",
            TriggerCategory::ScoreGap        => "Score Gap",
        }
    }

    pub fn action_label(&self) -> &'static str {
        match self.action {
            RuleAction::Show             => "Show",
            RuleAction::Hide             => "Hide",
            RuleAction::Require          => "Require",
            RuleAction::SurfaceAsNudge   => "Surface as Nudge",
            RuleAction::ShowInPrepMode   => "Prep Mode Only",
            RuleAction::ShowAlertBanner  => "Show Alert Banner",
        }
    }

    /// True if the action surfaces this dimension in the compact nudge widget.
    pub fn is_nudge_action(&self) -> bool {
        matches!(
            self.action,
            RuleAction::SurfaceAsNudge | RuleAction::Require
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
    pub scale_type: ScaleType,
    pub scale_min: f64,
    pub scale_max: f64,
    pub unit_label: Option<String>,
    pub is_inverted: bool,
    pub is_required: bool,
    pub render_mode: RenderMode,
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
