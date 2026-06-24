//! Sea-ORM entity for `atlas_integration_events` (G-05).
//!
//! Immutable delivery ledger. One row is appended for every dispatch attempt
//! (success or failure) made by the `SyndicationEventBus` worker.
//!
//! Events survive outbox row cleanup — they provide a long-lived audit trail
//! for G-05 compliance and operator-facing delivery reports in platform-admin.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_integration_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// FK → atlas_syndication_outbox.id (nullable; outbox rows may be pruned)
    pub outbox_id: Option<Uuid>,

    /// Denormalised FK → atlas_app_instance_syndication.id
    pub link_id: Uuid,

    /// Denormalised FK → atlas_app_deployment_config.id
    pub source_config_id: Uuid,

    /// Mirrors atlas_syndication_outbox.event_type
    pub event_type: String,

    /// "outbound" | "inbound"
    pub direction: String,

    /// "success" | "failed" | "skipped"
    pub outcome: String,

    /// HTTP response status returned by the NI endpoint
    pub http_status: Option<i32>,

    /// Abbreviated response body or error message (max 2 KB)
    pub response_body: Option<String>,

    /// Round-trip latency in milliseconds
    pub latency_ms: Option<i32>,

    /// Attempt number (1-based)
    pub attempt_number: i32,

    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
