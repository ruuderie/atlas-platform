#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_campaign_enrollments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub tenant_id: Uuid,

    // ── Contact identity (one of these populated) ─────────────────────────────
    /// FK to "user"(id) — set when enrolling an existing platform user.
    pub contact_user_id: Option<Uuid>,
    /// Email address for external contacts not yet in the platform.
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    /// Enrichment data: {company, title, linkedin_url, phone, ...}
    pub contact_metadata: Option<serde_json::Value>,

    // ── Progress ──────────────────────────────────────────────────────────────
    /// VARCHAR — validated as `EnrollmentStatus` at the service layer.
    pub status: String,
    pub current_step: i32,

    // ── Exit tracking ─────────────────────────────────────────────────────────
    /// 'replied', 'converted', 'unsubscribed', 'bounced', 'manually_removed'
    pub exit_reason: Option<String>,
    pub exit_at: Option<DateTime<Utc>>,

    // ── Conversion tracking ───────────────────────────────────────────────────
    pub converted_at: Option<DateTime<Utc>>,
    /// The entity type that was created by the conversion.
    pub conversion_entity_type: Option<String>,
    pub conversion_entity_id: Option<Uuid>,

    // ── External integration ──────────────────────────────────────────────────
    /// Instantly lead ID, Lemlist lead ID, etc.
    pub external_enrollment_id: Option<String>,

    // ── Timing ────────────────────────────────────────────────────────────────
    pub enrolled_at: DateTime<Utc>,
    /// Polled by the sequence scheduler background job.
    pub next_step_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
