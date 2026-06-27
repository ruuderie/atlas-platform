#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// GENERIC-07 ext: Atlas Notification Inbox
///
/// Persistent in-app notification record. Every notification dispatched by
/// NotificationService is stored here regardless of external channel delivery.
///
/// External channel delivery (Telegram, WhatsApp, SMS, Email) is tracked in
/// `channels_attempted` as a JSONB array of delivery receipts:
///   [{ "channel": "telegram", "status": "delivered"|"failed"|"skipped",
///      "attempted_at": "ISO8601", "error": "..." }]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_notification")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id:                 Uuid,
    pub tenant_id:          Uuid,
    pub user_id:            Uuid,

    /// Classification: lease_expiring | rent_due | maintenance_request |
    /// message_received | violation_filed | inspection_scheduled |
    /// payment_received | lead_assigned | scorecard_nudge | system | announcement
    pub notification_type:  String,
    pub title:              String,
    pub body:               String,
    /// low | normal | high | urgent
    pub priority:           String,

    /// Optional linked Atlas entity
    pub entity_type:        Option<String>,
    pub entity_id:          Option<Uuid>,

    /// Extra structured data: action_url, image_url, cta_label, etc.
    pub metadata:           Option<serde_json::Value>,

    /// Delivery receipt log (see doc above)
    pub channels_attempted: serde_json::Value,

    pub read_at:            Option<DateTime<Utc>>,
    pub dismissed_at:       Option<DateTime<Utc>>,
    pub created_at:         DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
