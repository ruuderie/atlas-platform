#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_sequence_steps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub step_number: i32,

    /// 'email', 'sms', 'wait', 'condition', 'task', 'linkedin'
    pub step_type: String,

    // ── Content (email / sms) ─────────────────────────────────────────────────
    /// Supports {{first_name}}, {{company}} and spintax {A|B|C}
    pub subject_template: Option<String>,
    pub body_template: Option<String>,

    // ── Wait step ─────────────────────────────────────────────────────────────
    pub wait_days: Option<i32>,
    pub wait_hours: Option<i32>,
    /// 'business_hours', 'any_time', 'morning', 'afternoon'
    pub send_time_preference: Option<String>,

    // ── Condition step ────────────────────────────────────────────────────────
    /// 'opened', 'clicked', 'replied', 'not_opened_after'
    pub condition_type: Option<String>,
    pub condition_value: Option<serde_json::Value>,

    // ── Branch routing ────────────────────────────────────────────────────────
    /// Step number to jump to when condition is true.
    pub on_true_step: Option<i32>,
    /// Step number to jump to when condition is false.
    pub on_false_step: Option<i32>,

    // ── A/B test variants ─────────────────────────────────────────────────────
    /// [{subject: "...", body: "...", weight: 50}, ...]
    pub ab_variants: Option<serde_json::Value>,

    // ── Exit triggers ─────────────────────────────────────────────────────────
    pub exit_on_reply: bool,
    pub exit_on_conversion: bool,

    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
