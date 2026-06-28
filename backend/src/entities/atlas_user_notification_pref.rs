#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// GENERIC-07 ext: User Notification Channel Preferences
///
/// One row per (user_id, tenant_id, channel). UPSERTed via handler.
///
/// ## Channel config shapes (JSONB)
///
/// ```json
/// // telegram — personal DM or a group/channel
/// { "chat_id": "-1001234567890", "scope": "personal" | "broadcast" }
///
/// // whatsapp — Twilio or Meta Cloud API
/// { "phone": "+15551234567", "provider": "twilio" | "meta" }
///
/// // sms — via TelephonyProvider (Twilio or Telnyx)
/// { "phone": "+15551234567" }
///
/// // email — overrides user.email for notifications
/// { "email": "landlord-alerts@example.com" }
///
/// // in_app — always on, config unused
/// {}
/// ```
///
/// ## Broadcast channels (tenant-level group)
/// Stored with user_id = tenant_id (sentinel) and scope = "broadcast".
/// e.g. a landlord's Telegram announcement group that all tenants see.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_user_notification_pref")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub tenant_id:  Uuid,

    /// in_app | sms | email | telegram | whatsapp
    pub channel:    String,

    /// Channel-specific config (see doc above)
    pub config:     serde_json::Value,

    /// Master on/off switch
    pub enabled:    bool,

    /// Notification types this pref applies to. Empty array = all types.
    /// e.g. vec!["rent_due", "lease_expiring"]
    pub applies_to: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
