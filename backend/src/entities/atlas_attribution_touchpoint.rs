#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_attribution_touchpoints")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,

    // ── Visitor identity ──────────────────────────────────────────────────────
    /// Set when the visitor is a known platform user.
    pub user_id: Option<Uuid>,
    /// Email captured from a form before the visitor logs in.
    pub contact_email: Option<String>,
    /// Client-side cookie or device fingerprint. Resolved to `user_id` by
    /// `AttributionService::resolve_identity()` when the visitor identifies.
    pub anonymous_id: Option<String>,

    // ── Channel ───────────────────────────────────────────────────────────────
    /// VARCHAR — validated as `AttributionChannel` at the service layer.
    pub channel: String,

    // ── UTM parameters ────────────────────────────────────────────────────────
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,

    // ── Platform entity cross-references ─────────────────────────────────────
    /// FK to `atlas_campaigns` — which campaign drove this touchpoint.
    pub campaign_id: Option<Uuid>,
    /// FK to `atlas_campaign_enrollments` — which sequence step triggered this.
    pub enrollment_id: Option<Uuid>,
    /// FK to `atlas_events` (G21) — for event-driven touchpoints.
    pub event_id: Option<Uuid>,

    // ── Conversion (written by record_conversion) ─────────────────────────────
    /// Entity type of the converted record (e.g. "atlas_reservations").
    pub conversion_entity_type: Option<String>,
    pub conversion_entity_id: Option<Uuid>,
    /// GMV of the conversion in cents.
    pub conversion_value_cents: Option<i64>,
    /// Credit allocated to this touchpoint by the attribution model.
    pub attributed_revenue_cents: Option<i64>,
    /// VARCHAR — validated as `AttributionModel` at the service layer.
    pub attribution_model: Option<String>,

    // ── Visit context ─────────────────────────────────────────────────────────
    pub landing_page_url: Option<String>,
    pub referrer_url: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
