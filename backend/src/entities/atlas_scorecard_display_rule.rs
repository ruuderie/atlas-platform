#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};

/// G-27 Display Rules: `atlas_scorecard_display_rules`
///
/// A display rule is a condition → action pair that controls when and how
/// a scorecard dimension is shown in the session form.
///
/// All discriminator fields (trigger_category, operator, action, mode_scope)
/// are stored as VARCHAR strings in Postgres. The service layer converts them
/// to/from the typed enums in `crate::types::scorecard` at the read/write boundary.
///
/// # Evaluation contract
/// The frontend (or `ScorecardService::get_display_rules`) returns all active rules
/// for a template. The client evaluates them against current entity field values.
/// Conflict resolution: `Require` > `Hide` > `Show`. See `RuleAction::overrides`.
///
/// # Tier gate
/// `ScorecardService::get_display_rules` returns `Ok(vec![])` for tenants without
/// `scorecard_display_rules_enabled = true` in their `tenant_settings`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_display_rules")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// The template this rule belongs to.
    pub template_id: Uuid,

    /// Optional dimension target. If None, `category_target` must be set
    /// and the rule applies to all active dimensions in that category.
    pub dimension_id: Option<Uuid>,

    pub tenant_id: Uuid,

    /// Category-level target (used when dimension_id IS NULL).
    /// e.g. "deal_health", "stakeholder", "competitive"
    pub category_target: Option<String>,

    // ── Trigger axis ──────────────────────────────────────────────────────────
    // All VARCHAR fields below have typed Rust equivalents in crate::types::scorecard.
    // Convert with TryFrom<String> after reading; Display::fmt before writing.

    /// The condition category that fires this rule.
    /// Typed: `crate::types::scorecard::TriggerCategory`
    /// Values: 'record_state' | 'time_proximity' | 'activity_trigger' | 'score_gap'
    pub trigger_category: String,

    /// The field on the subject entity to evaluate.
    /// Used by `record_state` and `time_proximity` triggers.
    /// e.g. 'stage', 'close_date', 'lead_status'
    pub field_reference: Option<String>,

    /// The comparison operator to apply.
    /// Typed: `crate::types::scorecard::RuleOperator`
    /// Values: 'equals' | 'not_equals' | 'in' | 'not_in' | 'within_days' |
    ///         'overdue_days' | 'dimension_score_below' | 'dimension_score_above' |
    ///         'activity_type_is' | 'dimension_unrated'
    pub operator: String,

    /// Scalar comparison value for 'equals', 'within_days', score threshold operators.
    pub value: Option<String>,

    /// List comparison values for 'in', 'not_in', 'activity_type_is'.
    /// Stored as JSONB array of strings: `["call", "demo"]`
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub value_list: Option<Value>,

    // ── Action axis ───────────────────────────────────────────────────────────

    /// What happens to the targeted dimension when the condition fires.
    /// Typed: `crate::types::scorecard::RuleAction`
    /// Values: 'show' | 'hide' | 'require' | 'surface_as_nudge' |
    ///         'show_in_prep_mode' | 'show_alert_banner'
    pub action: String,

    /// Banner text for 'show_alert_banner' action.
    pub alert_message: Option<String>,

    // ── Scope ─────────────────────────────────────────────────────────────────

    /// Which rendering context must be active for this rule to apply.
    /// Typed: `crate::types::scorecard::ModeScope`
    /// Values: 'always' | 'post_activity' | 'pre_activity' | 'on_score_gap'
    pub mode_scope: String,

    /// Conflict resolution priority. Lower number = higher priority.
    /// Applied when multiple rules target the same dimension with conflicting actions.
    pub priority: i32,

    pub is_active: bool,

    /// Human-readable explanation of why this rule exists.
    pub description: Option<String>,

    pub created_by_user_id: Option<Uuid>,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Parse `trigger_category` into the typed enum.
    /// Returns an error if the stored value is not a known variant.
    pub fn trigger_category_typed(
        &self,
    ) -> Result<crate::types::scorecard::TriggerCategory, String> {
        crate::types::scorecard::TriggerCategory::try_from(self.trigger_category.clone())
    }

    /// Parse `operator` into the typed enum.
    pub fn operator_typed(&self) -> Result<crate::types::scorecard::RuleOperator, String> {
        crate::types::scorecard::RuleOperator::try_from(self.operator.clone())
    }

    /// Parse `action` into the typed enum.
    pub fn action_typed(&self) -> Result<crate::types::scorecard::RuleAction, String> {
        crate::types::scorecard::RuleAction::try_from(self.action.clone())
    }

    /// Parse `mode_scope` into the typed enum.
    pub fn mode_scope_typed(&self) -> Result<crate::types::scorecard::ModeScope, String> {
        crate::types::scorecard::ModeScope::try_from(self.mode_scope.clone())
    }

    /// Extract `value_list` as a `Vec<String>`.
    /// Used for 'in', 'not_in', and 'activity_type_is' operators.
    pub fn value_list_as_strings(&self) -> Vec<String> {
        self.value_list
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_owned()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
