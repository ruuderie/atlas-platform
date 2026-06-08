#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-23: AtlasReservation
///
/// Time-bounded asset reservation with inventory hold.
///
/// Used by: Folio (STR unit bookings), Direct Booking Engine (hotel rooms),
/// Flight+Hotel Builder (packages, flight seats), Equipment Rental, Truck Parking,
/// Beauty/Hair (service appointments), Events (event slots).
///
/// # Reservation types (`reservation_type`)
/// - `"str_unit"`            — Folio STR unit booking
/// - `"hotel_room"`          — Direct Booking Engine hotel room
/// - `"package"`             — Flight+Hotel multi-item bundle
/// - `"flight_seat"`         — Duffel-sourced flight seat
/// - `"equipment_rental"`    — Equipment rental time slot
/// - `"truck_parking"`       — Truck parking spot
/// - `"service_appointment"` — Beauty, hair, contractor appointment
/// - `"event_slot"`          — Ticketed event seat
///
/// # Status lifecycle
/// `hold` → `confirmed` → `checked_in` → `checked_out`
///                └─────────────────────→ `cancelled`
///                └─────────────────────→ `no_show`
/// `hold` ──────────────────────────────→ `hold_expired` (background job)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_reservations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,

    /// Discriminator: 'str_unit', 'hotel_room', 'equipment_rental', etc.
    pub reservation_type: String,

    /// Polymorphic asset reference — typically 'atlas_assets'.
    pub reserved_asset_type: Option<String>,
    /// FK to the reserved entity (atlas_assets.id, or external entity id).
    pub reserved_asset_id: Option<Uuid>,

    /// The guest or primary booker — FK to atlas_accounts.id.
    pub guest_account_id: Option<Uuid>,

    /// Inclusive start of the reservation window (TIMESTAMPTZ).
    pub starts_at: DateTime<Utc>,
    /// Exclusive end of the reservation window (TIMESTAMPTZ).
    pub ends_at: DateTime<Utc>,

    /// Current lifecycle status (see type docs above).
    pub status: String,

    /// External platform hold ID (Airbnb reservation code, Vrbo ID, Duffel hold_id).
    pub external_hold_id: Option<String>,
    /// Timestamp after which the hold auto-expires (background job enforces).
    pub hold_expires_at: Option<DateTime<Utc>>,

    /// Total reservation price in minor currency units (cents).
    pub total_price_cents: Option<i64>,
    /// Per-night rate for STR/hotel reservations.
    pub nightly_rate_cents: Option<i64>,
    /// ISO 4217 currency code.
    pub currency: Option<String>,

    /// App-specific payload: guest_count, ota_platform, room_type, notes, etc.
    pub reservation_metadata: Value,

    /// FK to atlas_quotes (G24) — the quote that produced this booking.
    pub quote_id: Option<Uuid>,
    /// FK to atlas_ledger_entries (G03) — set after payment is collected.
    pub ledger_entry_id: Option<Uuid>,

    /// Set when status transitions to 'confirmed'.
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Set when status transitions to 'cancelled'.
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Human-readable cancellation reason (guest or platform).
    pub cancellation_reason: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
