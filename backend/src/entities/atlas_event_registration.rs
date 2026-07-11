#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_event_registrations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub event_id: Uuid,
    pub ticket_type_id: Uuid,
    pub tenant_id: Uuid,

    // ── Attendee identity ─────────────────────────────────────────────────────
    pub attendee_email: String,
    pub attendee_name: Option<String>,
    /// FK to platform user — NULL if external attendee not yet on platform.
    pub attendee_user_id: Option<Uuid>,

    // ── Booking ───────────────────────────────────────────────────────────────
    pub quantity: i32,
    /// FK to `atlas_ledger_entries` (G03) — set for paid tickets after payment.
    pub ledger_entry_id: Option<Uuid>,

    // ── Check-in ──────────────────────────────────────────────────────────────
    /// 32-byte random hex string — encoded in QR code for event entry.
    pub check_in_token: String,
    /// VARCHAR — validated as `RegistrationStatus` at the service layer.
    pub status: String,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub checked_in_at: Option<DateTime<Utc>>,

    // ── Attribution (G20) ─────────────────────────────────────────────────────
    /// FK to `atlas_attribution_touchpoints` — the touchpoint that drove registration.
    pub attribution_touchpoint_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
