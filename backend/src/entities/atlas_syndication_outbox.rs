//! Sea-ORM entity for `atlas_syndication_outbox` (G-05).
//!
//! Transactional outbox for outbound syndication events. One row per
//! (link, event) pair. Written inside the same DB transaction that mutates
//! the listing/asset so delivery is guaranteed at-least-once.
//!
//! # Delivery lifecycle
//! ```text
//! pending → processing → delivered
//!                      ↘ failed  (retry_count >= 5 → dead-letter)
//! ```

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_syndication_outbox")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// FK → atlas_app_instance_syndication.id
    pub link_id: Uuid,

    /// FK → atlas_app_deployment_config.id
    pub source_config_id: Uuid,

    /// Event type string, e.g. "listing.published"
    pub event_type: String,

    /// JSON payload to POST to the NI webhook URL
    pub payload: Json,

    /// Delivery status: "pending" | "processing" | "delivered" | "failed"
    pub status: String,

    /// How many HTTP dispatch attempts have been made
    pub retry_count: i32,

    /// Last HTTP status code returned by the NI (None if never attempted)
    pub last_http_status: Option<i32>,

    /// Abbreviated error message from the last failed attempt
    pub last_error: Option<String>,

    /// When the worker may next attempt delivery (enables exponential back-off)
    pub next_attempt_at: DateTimeWithTimeZone,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum delivery attempts before a row is marked `failed` (dead-letter).
pub const MAX_RETRY_COUNT: i32 = 5;

/// Known event types — enforced at enqueue time.
pub mod event_type {
    pub const LISTING_PUBLISHED:    &str = "listing.published";
    pub const LISTING_UPDATED:      &str = "listing.updated";
    pub const LISTING_UNPUBLISHED:  &str = "listing.unpublished";
    pub const ASSET_CREATED:        &str = "asset.created";
    pub const ASSET_UPDATED:        &str = "asset.updated";
    pub const INQUIRY_RECEIVED:     &str = "inquiry.received";
    pub const APPLICATION_RECEIVED: &str = "application.received";
}
