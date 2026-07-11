#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_campaigns")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,

    // ── Hierarchy ─────────────────────────────────────────────────────────────────
    /// NULL = root campaign. Non-null = child of another campaign.
    /// Enables Program → Campaign → Tactic trees with roll-up stats.
    pub parent_campaign_id: Option<Uuid>,

    // ── Identity ──────────────────────────────────────────────────────────────
    pub name: String,
    /// Unique human-readable system id: `{app_id}_{slug(name)}` (snake_case).
    pub global_name: String,
    /// VARCHAR — validated as `CampaignType` enum at the service layer.
    pub campaign_type: String,
    /// VARCHAR — validated as `CampaignStatus` enum at the service layer.
    pub status: String,

    // ── Audience ──────────────────────────────────────────────────────────────
    /// Future FK to atlas_audience_segments.
    pub audience_segment_id: Option<Uuid>,
    /// JSONB filter for audience targeting: {"source": "open_house_2024", "geography": "miami"}
    pub audience_filter: Option<serde_json::Value>,

    // ── Goal ──────────────────────────────────────────────────────────────────
    /// 'lead_capture', 'booking', 'application', 'sale', 'registration'
    pub goal_type: Option<String>,
    /// Entity type that a successful conversion creates.
    pub goal_entity_type: Option<String>,
    pub target_conversion_count: Option<i32>,

    // ── Budget ────────────────────────────────────────────────────────────────
    pub budget_cents: Option<i64>,
    pub currency: Option<String>,
    /// Incremented by `CampaignService::record_event` for 'spent' events.
    pub spent_cents: i64,

    // ── Attribution ───────────────────────────────────────────────────────────
    pub attribution_window_days: i32,

    // ── External integration ──────────────────────────────────────────────────
    /// Instantly campaign ID, Google campaign ID, Meta campaign ID, etc.
    pub external_campaign_id: Option<String>,
    pub integration_id: Option<Uuid>,

    // ── Subject entity (polymorphic FK) ───────────────────────────────────────
    /// 'atlas_assets', 'atlas_events', 'atlas_opportunities', etc.
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,

    // ── Scheduling ────────────────────────────────────────────────────────────
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,

    // ── UTM parameters ────────────────────────────────────────────────────────
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,

    // ── Computed counters (updated by CampaignService) ────────────────────────
    pub total_contacts: i32,
    pub total_opens: i32,
    pub total_clicks: i32,
    pub total_replies: i32,
    pub total_conversions: i32,

    // ── Audit ─────────────────────────────────────────────────────────────────
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
