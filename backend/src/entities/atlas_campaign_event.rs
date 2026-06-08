#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_campaign_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub campaign_id: Uuid,
    pub tenant_id: Uuid,

    /// NULL for non-sequence events (PPC click, direct form fill).
    pub sequence_step_id: Option<Uuid>,

    /// VARCHAR — validated as `CampaignEventType` at the service layer.
    /// 'sent' | 'delivered' | 'opened' | 'clicked' | 'replied' | 'bounced' |
    /// 'unsubscribed' | 'spam_reported' | 'converted' | 'form_fill'
    pub event_type: String,

    /// VARCHAR — validated as `CampaignChannel` at the service layer.
    /// 'email' | 'sms' | 'ppc_click' | 'social' | 'event' | 'referral' | 'linkedin'
    pub channel: String,

    // ── Click / visit context ─────────────────────────────────────────────────
    pub link_clicked: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ip_address: Option<String>, // stored as INET; read as text
    pub user_agent: Option<String>,

    /// Free-form context: {step_number, subject_line, ab_variant, ...}
    pub metadata: Option<serde_json::Value>,

    pub occurred_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
