#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,

    // ── Identity ──────────────────────────────────────────────────────────────
    pub name: String,
    pub slug: Option<String>,
    /// VARCHAR — validated as `EventType` enum at the service layer.
    pub event_type: String,
    /// VARCHAR — validated as `EventStatus` enum at the service layer.
    pub status: String,

    // ── Location ──────────────────────────────────────────────────────────────
    pub is_virtual: bool,
    pub virtual_url: Option<String>,
    pub venue_name: Option<String>,
    pub venue_address: Option<String>,
    /// FK to `atlas_assets` — the managed property hosting the event.
    pub venue_asset_id: Option<Uuid>,

    // ── Capacity ──────────────────────────────────────────────────────────────
    /// NULL = unlimited capacity.
    pub max_capacity: Option<i32>,
    pub waitlist_enabled: bool,

    // ── Schedule ──────────────────────────────────────────────────────────────
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub registration_opens_at: Option<DateTime<Utc>>,
    pub registration_closes_at: Option<DateTime<Utc>>,

    // ── Campaign linkage (G19) ────────────────────────────────────────────────
    /// Which campaign promoted this event. Used for attribution and roll-up.
    pub campaign_id: Option<Uuid>,

    // ── Polymorphic subject entity ────────────────────────────────────────────
    /// e.g. "atlas_assets" (open house at a property) or "atlas_opportunities".
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,

    // ── Visibility ────────────────────────────────────────────────────────────
    pub is_public: bool,

    // ── Computed counters (maintained by EventService) ────────────────────────
    pub registered_count: i32,
    pub attended_count: i32,
    pub revenue_cents: i64,

    // ── Audit ─────────────────────────────────────────────────────────────────
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
